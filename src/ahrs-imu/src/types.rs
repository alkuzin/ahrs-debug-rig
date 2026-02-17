// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Common types declarations.

use crate::drivers::RgbLed;
use embassy_stm32::gpio::Output;
use idtp::payload::{IdtpPayload, Imu6};

/// Status RGB LED alias.
pub type StatusLed<'a> = RgbLed<Output<'a>, Output<'a>, Output<'a>>;

/// System status levels.
#[derive(Copy, Clone)]
pub enum SystemStatus {
    /// All subsystems operational.
    Ok,
    /// Non-critical issue.
    Warning,
    /// Critical failure.
    Error,
    /// Initialization phase.
    Initializing,
}

/// Generic IMU sample.
pub struct ImuSample<T: IdtpPayload> {
    /// IMU sensors readings.
    pub data: T,
    /// IMU local time in milliseconds.
    pub timestamp: u32,
}

/// Alias for 6-axes IMU sample.
pub type Sample = ImuSample<Imu6>;
