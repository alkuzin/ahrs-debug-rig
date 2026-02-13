// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Common types declarations.

use crate::drivers::RgbLed;
use embassy_stm32::gpio::Output;

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
