// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! STM32 firmware for handling IMU.

#![no_std]

mod hardware;
pub mod status;
pub mod utils;

pub use hardware::{SystemConfig, SystemContext};
