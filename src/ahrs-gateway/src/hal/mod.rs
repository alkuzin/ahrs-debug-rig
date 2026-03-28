// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Hardware abstraction layer.

mod led;
pub use led::StatusLed;

use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    timer::timg::TimerGroup
};

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Builtin LED handler.
    pub builtin_led: Output<'static>,
    /// Status LEDs handler.
    pub status_led: StatusLed<'static>,
}

impl SystemPeripherals {
    /// Construct & initialize IMU handler system peripherals.
    ///
    /// # Parameters
    /// - `p` - given STM32 peripherals to handle.
    ///
    /// # Returns
    /// - Initialize IMU handler system peripherals.
    pub fn new(p: Peripherals) -> Self {
        esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

        let timg0 = TimerGroup::new(p.TIMG0);
        esp_rtos::start(timg0.timer0);

        let config = OutputConfig::default();

        let builtin_led = Output::new(p.GPIO2, Level::High, config);
        let status_led_red = Output::new(p.GPIO4, Level::High, config);
        let status_led_green = Output::new(p.GPIO16, Level::High, config);
        let status_led = StatusLed::new(status_led_red, status_led_green);

        Self {
            builtin_led,
            status_led,
        }
    }
}
