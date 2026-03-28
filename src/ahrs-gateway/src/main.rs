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

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use panic_halt as _;
use crate::hal::SystemPeripherals;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let mut sp = SystemPeripherals::new(esp_hal::init(config));

    loop {
        sp.status_led.set_state(true, false);
        Timer::after(Duration::from_secs(1)).await;

        sp.status_led.set_state(false, true);
        Timer::after(Duration::from_secs(1)).await;

        sp.status_led.set_state(false, false);
        Timer::after(Duration::from_secs(1)).await;
    }
}
