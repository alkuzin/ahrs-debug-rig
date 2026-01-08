// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! STM32 firmware for debugging communication over SPI.

#![no_std]

pub mod status;
pub mod payload;
pub mod utils;
mod hardware;

pub use hardware::{SystemConfig, SystemContext};
