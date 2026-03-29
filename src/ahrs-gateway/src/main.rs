// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! AHRS gateway entry point.

#![no_std]
#![no_main]
#![deny(clippy::mem_forget)]
#![warn(clippy::all, clippy::correctness, clippy::suspicious)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::todo,
    clippy::unreachable,
    missing_docs
)]

pub mod hal;
pub mod types;
mod tasks;

use crate::{
    hal::SystemPeripherals,
    tasks::status::{system_status_task, set_system_status},
    types::SystemStatus
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker, Timer};
use esp_hal::clock::CpuClock;
use panic_halt as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let sp = SystemPeripherals::new(esp_hal::init(config)).await;

    // Spawning task for handling system status update.
    let ticker = Ticker::every(Duration::from_millis(10));
    let _ = spawner.spawn(system_status_task(sp.status_led, ticker));

    loop {
        set_system_status(SystemStatus::Ok).await;
        Timer::after(Duration::from_millis(1000)).await;

        set_system_status(SystemStatus::Error).await;
        Timer::after(Duration::from_millis(500)).await;
    }
}
