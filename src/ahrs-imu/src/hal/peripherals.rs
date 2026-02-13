// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! MCU peripherals related declarations.

use embassy_stm32::{gpio::{Level, Output, Speed}, Peripherals};
use embassy_time::{Duration, Ticker};
use crate::types::StatusLed;

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Builtin LED handler.
    pub builtin_led: Output<'static>,
    /// Status LED handler.
    pub status_led: StatusLed<'static>,
    /// Status task ticker.
    pub status_ticker: Ticker,
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
        let led_status_red_pin = p.PA9;
        let led_status_green_pin = p.PA10;
        let led_status_blue_pin = p.PA11;
        let builtin_led_pin = p.PC13;

        let led_r = Output::new(led_status_red_pin, Level::High, Speed::Low);
        let led_g = Output::new(led_status_green_pin, Level::High, Speed::Low);
        let led_b = Output::new(led_status_blue_pin, Level::High, Speed::Low);

        Self {
            builtin_led: Output::new(builtin_led_pin, Level::High, Speed::Low),
            status_led: StatusLed::new(led_r, led_g, led_b, false),
            status_ticker: Ticker::every(Duration::from_millis(10)),
        }
    }
}
