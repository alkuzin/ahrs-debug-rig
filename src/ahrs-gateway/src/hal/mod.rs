// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Hardware abstraction layer.

mod led;

use embassy_time::{Duration, Timer};
pub use led::StatusLed;
use esp_hal::{
    Blocking,
    dma::DmaRxBuf,
    dma_buffers,
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    spi::{
        Mode,
        slave::{Spi, dma::SpiDma},
    },
    timer::timg::TimerGroup,
};

/// Alias for SPI driver.
pub type SpiDriver<'a> = SpiDma<'a, Blocking>;

/// SPI DMA buffer size in bytes.
pub const DMA_BUFFER_SIZE: usize = 256;

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Status LEDs handler.
    pub status_led: StatusLed<'static>,
    /// ESP ready pin.
    pub esp_ready: Output<'static>,
    /// SPI handler.
    pub spi: SpiDriver<'static>,
    /// SPI DMA buffer for incoming data.
    pub dma_rx_buf: DmaRxBuf,
}

impl SystemPeripherals {
    /// Construct & initialize IMU handler system peripherals.
    ///
    /// # Parameters
    /// - `p` - given STM32 peripherals to handle.
    ///
    /// # Returns
    /// - Initialize IMU handler system peripherals.
    pub async fn new(p: Peripherals) -> Self {
        esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

        let timg0 = TimerGroup::new(p.TIMG0);
        esp_rtos::start(timg0.timer0);

        let config = OutputConfig::default();

        let status_led_red = Output::new(p.GPIO4, Level::High, config);
        let status_led_green = Output::new(p.GPIO16, Level::High, config);
        let mut status_led = StatusLed::new(status_led_red, status_led_green);
        let esp_ready = Output::new(p.GPIO2, Level::Low, config);

        // Setting SPI + DMA.
        let dma_channel = p.DMA_SPI2;
        let spi_sck = p.GPIO18;
        let spi_miso = p.GPIO19;
        let spi_mosi = p.GPIO23;
        let spi_ss = p.GPIO5;

        let (rx_buffer, rx_descriptors, _, _) = dma_buffers!(DMA_BUFFER_SIZE);

        let spi = Spi::new(p.SPI2, Mode::_1)
            .with_sck(spi_sck)
            .with_miso(spi_miso)
            .with_mosi(spi_mosi)
            .with_cs(spi_ss)
            .with_dma(dma_channel);

        let dma_rx_buf = match DmaRxBuf::new(rx_descriptors, rx_buffer) {
            Ok(buf) => buf,
            Err(_) => {
                loop {
                    status_led.set_state(true, false);
                    Timer::after(Duration::from_millis(100)).await;
                    status_led.set_state(false, false);
                }
            },
        };

        Self {
            esp_ready,
            status_led,
            spi,
            dma_rx_buf,
        }
    }
}
