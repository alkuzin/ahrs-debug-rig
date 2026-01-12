// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU handling abstraction layer.

use crate::{
    status::{LedStatus, Status},
    utils::{self, halt_cpu},
};
use ahrs_common::{
    FRAME_SIZE,
    idtp::{IdtpFrame, IdtpHeader, Mode},
    payload::PAYLOAD_SIZE,
    utils::calculate_checksum,
};
use core::ptr;
use cortex_m::singleton;
use crc::Crc;
use embedded_hal::spi;
use stm32f4xx_hal::dma::config::DmaConfig;
use stm32f4xx_hal::dma::{StreamsTuple, Transfer};
use stm32f4xx_hal::pac::{DMA2, SPI1};
use stm32f4xx_hal::spi::Tx;
use stm32f4xx_hal::timer::SysDelay;
use stm32f4xx_hal::{
    dma,
    dwt::{Instant, MonoTimer},
    gpio::{Output, Pin},
    pac::{self, CorePeripherals, Peripherals},
    prelude::*,
    rcc,
    spi::Spi1,
    time::Hertz,
    timer::CounterHz,
};

/// Set of system's configurations.
pub struct SystemConfig {
    /// Clocks configuration.
    pub rcc_cfg: rcc::Config,
    /// Frequency of sampling IMU data (Hz).
    pub sampling_rate_hz: Hertz,
    /// SPI mode.
    pub spi_mode: spi::Mode,
    /// Frequency of SPI (Hz).
    pub spi_freq: Hertz,
    /// Pseudo-random number generator initial state.
    pub rng_initial_state: u32,
    /// IMU device identifier.
    pub device_id: u16,
    /// Initial delay for IMU sensors to initialize (ms).
    pub initial_delay_ms: u32,
    /// IDTP mode.
    pub protocol_mode: Mode,
}

/// Systems peripherals handler.
pub struct SystemContext {
    /// STM32 device peripherals.
    pub dp: Peripherals,
    /// Cortex-M peripherals.
    pub cp: CorePeripherals,
}

impl SystemContext {
    /// Construct new `SystemContext` object.
    ///
    /// # Parameters
    /// - `dp` - given STM32 device peripherals to handle.
    /// - `cp` - given Cortex-M peripherals to handle.
    ///
    /// # Returns
    /// - New `SystemContext` object.
    pub fn new(dp: Peripherals, cp: CorePeripherals) -> Self {
        Self { dp, cp }
    }

    /// Prepare peripherals.
    ///
    /// # Parameters
    /// - `cfg` - given set of system's configurations.
    ///
    /// # Returns
    /// - IMU system handler.
    pub fn init(self, cfg: SystemConfig) -> ImuSystem {
        // Setting timing.
        let mut rcc = self.dp.RCC.freeze(cfg.rcc_cfg);

        // Setting status RGB LED.
        let gpioa = self.dp.GPIOA.split(&mut rcc);

        let mut led_status = LedStatus::new(
            gpioa.pa9.into_push_pull_output(),
            gpioa.pa10.into_push_pull_output(),
            gpioa.pa11.into_push_pull_output(),
            false,
        );

        // Setting timers.
        let mut delay_timer = self.cp.SYST.delay(&rcc.clocks);
        let timestamp_timer =
            MonoTimer::new(self.cp.DWT, self.cp.DCB, &rcc.clocks);
        let mut sampling_timer = self.dp.TIM5.counter_hz(&mut rcc);

        if sampling_timer.start(cfg.sampling_rate_hz).is_err() {
            led_status.set_status(Status::Error);
            halt_cpu();
        }

        // Setting SPI.
        let spi_sck = gpioa.pa5.into_alternate().internal_pull_up(true);
        let spi_miso = gpioa.pa6.into_alternate().internal_pull_up(true);
        let spi_mosi = gpioa.pa7.into_alternate().internal_pull_up(true);
        let spi_ss = gpioa.pa4.into_push_pull_output();

        let spi = Spi1::new(
            self.dp.SPI1,
            (Some(spi_sck), Some(spi_miso), Some(spi_mosi)),
            cfg.spi_mode,
            cfg.spi_freq,
            &mut rcc,
        );

        let spi_tx = spi.use_dma().tx();

        // Setting DMA.
        let streams = StreamsTuple::new(self.dp.DMA2, &mut rcc);
        let spi_tx_stream = streams.3;

        let frame_buffer: &'static mut [u8; FRAME_SIZE] =
            singleton!(: [u8; FRAME_SIZE] = [0; FRAME_SIZE]).unwrap();

        // Wait for IMU sensors to initialize.
        delay_timer.delay_ms(cfg.initial_delay_ms);
        led_status.set_status(Status::SetupSuccess);

        let timestamp_freq = timestamp_timer.frequency().to_Hz();
        let start_time = timestamp_timer.now();

        ImuSystem {
            sampling_timer,
            delay_timer,
            spi_tx_stream,
            spi_tx,
            spi_ss,
            led_status,
            crc32: Crc::<u32>::new(&crc::CRC_32_AUTOSAR),
            cfg,
            start_time,
            timestamp_freq,
            sequence: 0,
            frame_buffer,
        }
    }
}

/// Alias for specific RGB LED.
pub type StatusLeds =
    LedStatus<Pin<'A', 9, Output>, Pin<'A', 10, Output>, Pin<'A', 11, Output>>;

/// SPI DMA data transmission stream handler.
type SpiTxStream = dma::Stream3<DMA2>;

/// IMU system handler.
pub struct ImuSystem {
    /// Timer for sampling IMU readings.
    sampling_timer: CounterHz<pac::TIM5>,
    /// Timer for time delays.
    delay_timer: SysDelay,
    /// SPI DMA data transmission stream handler.
    spi_tx_stream: SpiTxStream,
    /// SPI DMA data transmission handler.
    spi_tx: Tx<SPI1>,
    /// SPI Chip Select/Slave Select pin.
    spi_ss: Pin<'A', 4, Output>,
    /// System status RGB LED handler.
    led_status: StatusLeds,
    /// CRC32 hardware-assisted handler.
    crc32: Crc<u32>,
    /// Set of system's configurations.
    cfg: SystemConfig,
    /// Sampling start timestamp.
    start_time: Instant,
    /// Frequency of the timestamp timer.
    timestamp_freq: u32,
    /// Packet sequence number.
    sequence: u32,
    /// IDTP frame bytes buffer.
    frame_buffer: &'static mut [u8; FRAME_SIZE],
}

impl ImuSystem {
    /// Wait for the next sample.
    pub fn wait_next_sample(&mut self) {
        if nb::block!(self.sampling_timer.wait()).is_err() {
            self.led_status.set_status(Status::Error);
            halt_cpu();
        }
    }

    /// Get payload & pack IDTP frame before sending.
    pub fn pack_frame(&mut self) {
        // Generate payload.
        let ticks_since_start = self.start_time.elapsed();
        let timestamp =
            (ticks_since_start as u64 * 1000) / self.timestamp_freq as u64;

        let payload = utils::generate_payload(self.cfg.rng_initial_state);

        let payload_bytes = payload.as_bytes();
        self.led_status.set_status(Status::ImuSuccess);

        // Fill IDTP header.
        let mut header = IdtpHeader::new();
        header.mode = self.cfg.protocol_mode;
        header.device_id = self.cfg.device_id;
        header.timestamp = timestamp as u32;
        header.sequence = self.sequence;
        header.payload_size = PAYLOAD_SIZE as u32;

        // Fill IDTP frame.
        let mut idtp = IdtpFrame::new();

        idtp.set_header(&header);
        idtp.set_payload(&payload_bytes);

        if idtp.pack(self.frame_buffer).is_err() {
            self.led_status.set_status(Status::Error);
            halt_cpu();
        }

        // Calculating checksum/CRC32 for IDTP frame.
        let checksum = calculate_checksum(self.frame_buffer);
        let crc = match header.mode {
            Mode::Normal => 0,
            Mode::Safety => {
                let mut digest = self.crc32.digest();
                digest.update(self.frame_buffer);
                digest.finalize()
            }
            _ => 0,
        };

        header.checksum = checksum;
        header.crc = crc;
        idtp.set_header(&header);

        if idtp.pack(self.frame_buffer).is_err() {
            self.led_status.set_status(Status::Error);
            halt_cpu();
        }
    }

    /// Transfer prepared IDTP frame.
    pub fn transfer_frame(&mut self) {
        self.spi_ss.set_low();

        // Transfer frame over SPI with DMA.
        unsafe {
            let stream = ptr::read(&self.spi_tx_stream);
            let spi_tx = ptr::read(&self.spi_tx);
            let frame_buffer = ptr::read(&self.frame_buffer);

            let mut transfer = Transfer::init_memory_to_peripheral(
                stream,
                spi_tx,
                frame_buffer,
                None,
                DmaConfig::default()
                    .transfer_complete_interrupt(false)
                    .memory_increment(true),
            );

            transfer.start(|_| {});
            transfer.wait();

            let (stream_back, tx_back, buf_back, _) = transfer.release();

            ptr::write(&mut self.spi_tx_stream, stream_back);
            ptr::write(&mut self.spi_tx, tx_back);
            ptr::write(&mut self.frame_buffer, buf_back);
        }

        // Guard interval.
        self.delay_timer.delay_us(20);
        self.spi_ss.set_high();

        self.sequence += 1;
        self.led_status.set_status(Status::SpiSuccess);
    }
}
