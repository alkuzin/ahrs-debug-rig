// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

#![no_std]
#![no_main]

use stm32_firmware::{status::{LedStatus, Status}};
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};
use cortex_m_rt::entry;
use panic_halt as _;


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc_cfg = Config::hsi().sysclk(24.MHz());
    let mut rcc = dp.RCC.freeze(rcc_cfg);
    let mut timer = dp.TIM2.counter_ms(&mut rcc);
    let gpioa = dp.GPIOA.split(&mut rcc);

    let mut led_status = LedStatus::new(
        gpioa.pa9.into_push_pull_output(),
        gpioa.pa10.into_push_pull_output(),
        gpioa.pa11.into_push_pull_output(),
        false
    );

    // Wait 3 sec. for IMU sensors to initialize.
    if let Err(_) = timer.start(3000.millis()) {
        led_status.set_status(Status::Error);
        loop {}
    } 

    led_status.set_status(Status::SetupSuccess);

    if let Err(_) = nb::block!(timer.wait()) {
        led_status.set_status(Status::Error);
        loop {}
    } 

    loop {

    }
}
