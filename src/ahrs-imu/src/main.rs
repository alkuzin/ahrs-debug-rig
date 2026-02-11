// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU handler firmware entry point.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    Peripherals,
    gpio::{Level, Output, Speed},
};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p: Peripherals = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    let mut led_r = Output::new(p.PA9, Level::High, Speed::Low);
    let mut led_g = Output::new(p.PA10, Level::High, Speed::Low);
    let mut led_b = Output::new(p.PA11, Level::High, Speed::Low);

    loop {
        led.toggle();
        Timer::after_millis(100).await;
        led_r.toggle();
        Timer::after_millis(100).await;
        led_g.toggle();
        Timer::after_millis(200).await;
        led_b.toggle();
        Timer::after_millis(300).await;
    }
}
