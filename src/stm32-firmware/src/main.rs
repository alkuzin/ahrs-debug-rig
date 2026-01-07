// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

#![no_std]
#![no_main]

use stm32_firmware::{status::{LedStatus, Status}, payload::Payload, utils};
use stm32f4xx_hal::{pac, prelude::*, rcc::Config, crc32::Crc32, spi::Spi};
use cortex_m_rt::entry;
use idtp::{IdtpFrame, IdtpHeader, Mode, IDTP_PACKET_MIN_SIZE};
use panic_halt as _;
use stm32f4xx_hal::hal_02::spi::MODE_0;
use stm32_firmware::payload::PAYLOAD_SIZE;
use stm32_firmware::utils::halt_cpu;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc_cfg = Config::hsi().sysclk(24.MHz());
    let mut rcc = dp.RCC.freeze(rcc_cfg);
    let mut timer = dp.TIM2.counter_ms(&mut rcc);

    let mut crc_hardware = Crc32::new(dp.CRC, &mut rcc);
    let gpioa = dp.GPIOA.split(&mut rcc);

    let spi_sck = gpioa.pa5.into_alternate().internal_pull_up(true);
    let spi_miso = gpioa.pa6.into_alternate().internal_pull_up(true);
    let spi_mosi = gpioa.pa7.into_alternate().internal_pull_up(true);
    let mut spi_ss = gpioa.pa4.into_push_pull_output();

    let mut spi = Spi::new(
        dp.SPI1,
        (Some(spi_sck), Some(spi_miso), Some(spi_mosi)),
        MODE_0,
        3.MHz(),
        &mut rcc
    );

    let mut led_status = LedStatus::new(
        gpioa.pa9.into_push_pull_output(),
        gpioa.pa10.into_push_pull_output(),
        gpioa.pa11.into_push_pull_output(),
        false
    );

    // Wait 3 sec. for IMU sensors to initialize.
    if let Err(_) = timer.start(3000.millis()) {
        led_status.set_status(Status::Error);
        halt_cpu();
    }

    led_status.set_status(Status::SetupSuccess);

    if let Err(_) = nb::block!(timer.wait()) {
        led_status.set_status(Status::Error);
        halt_cpu();
    }

    if let Err(_) = timer.start(2.millis()) {
        led_status.set_status(Status::Error);
        halt_cpu();
    }

    const RNG_INITIAL_STATE: u32 = 0xABCDEF12;
    const IMU_DEVICE_ID: u16 = 0xABCD;
    const FRAME_SIZE: usize = IDTP_PACKET_MIN_SIZE + size_of::<Payload>();

    let mut sequence = 0;

    loop {
        let payload = utils::generate_payload(RNG_INITIAL_STATE);
        let payload_bytes = payload.as_bytes();
        led_status.set_status(Status::ImuSuccess);

        let mut header = IdtpHeader::new();
        header.mode = Mode::Safety;
        header.device_id = IMU_DEVICE_ID;
        header.timestamp = timer.now().ticks();
        header.sequence = sequence;
        header.payload_size = PAYLOAD_SIZE as u32;

        let mut idtp = IdtpFrame::new();

        idtp.set_header(&header);
        idtp.set_payload(&payload_bytes);

        let mut raw_frame = [0u8; FRAME_SIZE];

        if idtp.pack(&mut raw_frame).is_err() {
            led_status.set_status(Status::Error);
            halt_cpu();
        }

        let checksum = utils::calculate_checksum(&raw_frame);
        let crc = match header.mode {
            Mode::Normal => 0,
            Mode::Safety => crc_hardware.update_bytes(&raw_frame),
            _ => 0,
        };

        header.checksum = checksum;
        header.crc      = crc;
        idtp.set_header(&header);

        if idtp.pack(&mut raw_frame).is_err() {
            led_status.set_status(Status::Error);
            halt_cpu();
        }

        // Transferring data over SPI.
        spi_ss.set_low();
        match spi.write(&mut raw_frame) {
            Ok(_)  => led_status.set_status(Status::SpiSuccess),
            Err(_) => led_status.set_status(Status::Error),
        }
        spi_ss.set_high();

        sequence += 1;

        if let Err(_) = nb::block!(timer.wait()) {
            led_status.set_status(Status::Error);
            halt_cpu();
        }
    }
}