// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Hardware abstraction layer.

use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    timer::timg::TimerGroup
};

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Builtin LED handler.
    pub builtin_led: Output<'static>,
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

        let builtin_led_pin = p.GPIO2;
        let config = OutputConfig::default();
        let builtin_led = Output::new(builtin_led_pin, Level::High, config);

        Self {
            builtin_led,
        }
    }
}
