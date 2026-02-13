// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! RGB LED driver implementation.

use embedded_hal::digital::OutputPin;

/// RGB LED handler.
pub struct RgbLed<R, G, B> {
    /// Red LED pin.
    led_r: R,
    /// Green LED pin.
    led_g: G,
    /// Blue LED pin.
    led_b: B,
    /// Flag whether RGB LED has common anode or common cathode.
    is_common_anode: bool,
}

impl<R, G, B> RgbLed<R, G, B>
where
    R: OutputPin,
    G: OutputPin,
    B: OutputPin,
{
    /// Construct new `LedStatus` object.
    ///
    /// # Parameters
    /// - `led_r` - given red LED pin.
    /// - `led_g` - given green LED pin.
    /// - `led_b` - given blue LED pin.
    /// - `is_common_anode` - given flag whether RGB LED has common anode
    ///   or common cathode.
    ///
    /// # Returns
    /// - New `LedStatus` object.
    pub fn new(led_r: R, led_g: G, led_b: B, is_common_anode: bool) -> Self {
        Self {
            led_r,
            led_g,
            led_b,
            is_common_anode,
        }
    }

    /// Set RGB LED state.
    ///
    /// # Parameters
    /// - `r` - given flag whether to set high red LED.
    /// - `g` - given flag whether to set high green LED.
    /// - `b` - given flag whether to set high blue LED.
    pub fn set_state(&mut self, r: bool, g: bool, b: bool) {
        let (r, g, b) = if self.is_common_anode {
            (!r, !g, !b)
        } else {
            (r, g, b)
        };

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
        let _ = if b {
            &self.led_b.set_high()
        } else {
            &self.led_b.set_low()
        };
    }
}
