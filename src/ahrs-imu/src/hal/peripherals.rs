// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! MCU peripherals related declarations.

use crate::types::StatusLed;
use embassy_stm32::{
    Peripherals,
    gpio::{Level, Output, Speed},
    bind_interrupts,
    peripherals,
    i2c::{self, I2c, Master},
    mode::Async,
    time::Hertz
};

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Builtin LED handler.
    pub builtin_led: Output<'static>,
    /// Status LED handler.
    pub status_led: StatusLed<'static>,
    /// I2C handler for IMU.
    pub i2c: I2c<'static, Async, Master>,
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

        let builtin_led = Output::new(builtin_led_pin, Level::High, Speed::Low);
        let status_led = StatusLed::new(led_r, led_g, led_b, false);

        let mut i2c_cfg = i2c::Config::default();
        // I2C fast mode (400 kHz).
        i2c_cfg.frequency = Hertz(400_000);

        let i2c = I2c::new(
            p.I2C1,
            p.PB6,
            p.PB7,
            Irqs,
            p.DMA1_CH6,
            p.DMA1_CH5  ,
            i2c_cfg,
        );

        Self { builtin_led, status_led, i2c }
    }
}

// Binding I2C interrupts to handlers.
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
