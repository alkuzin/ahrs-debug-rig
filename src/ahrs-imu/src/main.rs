// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU handler firmware entry point.

#![no_std]
#![no_main]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::todo,
    clippy::unreachable,
    missing_docs
)]

mod drivers;
mod hal;
mod tasks;
mod types;

use crate::{
    hal::SystemPeripherals,
    tasks::status::{set_system_status, system_status_task},
    types::SystemStatus,
};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Peripherals;
use embassy_time::Timer;
use panic_probe as _;

/// IMU handler firmware entry point.
///
/// # Parameters
/// - `spawner` - given tasks spawner to handle.
#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Initializing system peripherals.
    let p: Peripherals = embassy_stm32::init(Default::default());
    let mut sp = SystemPeripherals::new(p);

    // Spawning task for handling system status update.
    let _ = spawner.spawn(system_status_task(sp.status_led, sp.status_ticker));

    loop {
        sp.builtin_led.toggle();
        Timer::after_millis(100).await;

        set_system_status(SystemStatus::Initializing).await;
        Timer::after_millis(200).await;

        set_system_status(SystemStatus::Ok).await;
        Timer::after_millis(300).await;

        set_system_status(SystemStatus::Error).await;
        Timer::after_millis(400).await;

        set_system_status(SystemStatus::Warning).await;
        Timer::after_millis(500).await;
    }
}
