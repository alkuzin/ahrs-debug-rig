// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU firmware status related declarations.

use embedded_hal::digital::OutputPin;

/// Status of IMU handler firmware.
pub enum Status {
    /// Show that MCU peripherals and IMU sensors initialized successfully.
    SetupSuccess,
    /// Show that IMU sensors readings were taken successfully.
    ImuSuccess,
    /// Show that successfully transmitted data over SPI.
    SpiSuccess,
    /// Show that fatal error occurred.
    Error,
    /// Reset status.
    Reset,
}

/// RGB LED status handling struct.
pub struct LedStatus<R, G, B> {
    /// Red LED pin.
    led_r: R,
    /// Green LED pin.
    led_g: G,
    /// Blue LED pin.
    led_b: B,
    /// Flag whether RGB LED has common anode or common cathode.
    is_common_anode: bool,
}

impl<R, G, B> LedStatus<R, G, B>
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
    fn set_state(&mut self, r: bool, g: bool, b: bool) {
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

    /// Set RGB LED status.
    ///
    /// # Parameters
    /// - `status` - given RGB LED status state to set.
    pub fn set_status(&mut self, status: Status) {
        match status {
            Status::SetupSuccess => self.set_state(true, true, true),
            Status::ImuSuccess => self.set_state(false, true, false),
            Status::SpiSuccess => self.set_state(false, false, true),
            Status::Error => self.set_state(true, false, false),
            Status::Reset => self.set_state(false, false, false),
        }
    }
}
