// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! LEDs driver implementation.

use embedded_hal::digital::OutputPin;
use esp_hal::gpio::Output;

/// Alias for status LEDs handler.
pub type StatusLed<'a> = Leds<Output<'a>, Output<'a>>;

/// Status LEDs handler.
pub struct Leds<R, G> {
    /// Red LED.
    led_r: R,
    /// Green LED.
    led_g: G,
}

impl<R, G> Leds<R, G>
where
    R: OutputPin,
    G: OutputPin,
{
    /// Construct new LEDs handler.
    ///
    /// # Parameters
    /// - `led_r` - given red LED.
    /// - `led_g` - given green LED.
    ///
    /// # Returns
    /// - New LEDs handler.
    pub const fn new(led_r: R, led_g: G) -> Self {
        Self { led_r, led_g }
    }

    /// Set LEDs state.
    ///
    /// # Parameters
    /// - `r` - given flag whether to set high red LED.
    /// - `g` - given flag whether to set high green LED.
    pub fn set_state(&mut self, r: bool, g: bool) {
        let _ = if r {
            &self.led_r.set_high()
        } else {
            &self.led_r.set_low()
        };
        let _ = if g {
            &self.led_g.set_high()
        } else {
            &self.led_g.set_low()
        };
    }
}
