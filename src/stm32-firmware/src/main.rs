// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU handler firmware entry point.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::spi::MODE_1;
use idtp::Mode;
use panic_halt as _;
use stm32_firmware::{SystemConfig, SystemContext};
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};

#[entry]
fn main() -> ! {
    // Handling system's peripherals.
    let context = SystemContext::new(
        pac::Peripherals::take().unwrap(),
        pac::CorePeripherals::take().unwrap(),
    );

    // Setting system's configurations.
    let config = SystemConfig {
        rcc_cfg: Config::hsi().sysclk(84.MHz()),
        sampling_rate_hz: 200.Hz(),
        spi_mode: MODE_1,
        spi_freq: 3.MHz(),
        rng_initial_state: 0xABCDEF12,
        device_id: 0xABCD,
        initial_delay_ms: 3000,
        protocol_mode: Mode::Safety,
    };

    // Initializing IMU system handler.
    let mut system = context.init(config);

    loop {
        system.wait_next_sample();
        system.pack_frame();
        system.transfer_frame();
    }
}
