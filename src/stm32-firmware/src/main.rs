// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU handler firmware entry point.

#![no_std]
#![no_main]

use stm32_firmware::{SystemConfig, SystemContext};
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};
use embedded_hal::spi::MODE_0;
use cortex_m_rt::entry;
use idtp::Mode;
use panic_halt as _;


#[entry]
fn main() -> ! {
    let context = SystemContext::new(
        pac::Peripherals::take().unwrap(),
        pac::CorePeripherals::take().unwrap(),
    );

    let config = SystemConfig {
        rcc_cfg: Config::hsi().sysclk(84.MHz()),
        sampling_rate_hz: 200.Hz(),
        spi_mode: MODE_0,
        spi_freq: 3.MHz(),
        rng_initial_state: 0xABCDEF12,
        device_id: 0xABCD,
        initial_delay_ms: 3000,
        protocol_mode: Mode::Safety,
    };

    let mut system = context.init(config);

    loop {
        system.wait_next_sample();
        system.pack_frame();
        system.transfer_frame();
    }
}