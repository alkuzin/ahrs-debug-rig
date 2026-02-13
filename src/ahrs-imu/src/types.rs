// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Common types declarations.

use embassy_stm32::gpio::Output;
use crate::drivers::RgbLed;

/// Status RGB LED alias.
pub type StatusLed<'a> = RgbLed<Output<'a>, Output<'a>, Output<'a>>;
