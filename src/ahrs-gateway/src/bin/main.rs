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

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
};
use panic_halt as _;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_rtos::start(timg0.timer0);

    let _ = spawner;

    let config = OutputConfig::default();
    let mut green_led = Output::new(p.GPIO16, Level::High, config);
    let mut red_led = Output::new(p.GPIO4, Level::Low, config);

    loop {
        green_led.toggle();
        Timer::after(Duration::from_secs(1)).await;

        red_led.toggle();
        Timer::after(Duration::from_secs(1)).await;
    }
}
