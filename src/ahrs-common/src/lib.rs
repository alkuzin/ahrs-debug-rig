// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Shared declarations among IMU handler & gateway.

#![no_std]
#![no_main]

pub mod payload;
pub mod utils;

use crate::payload::Payload;
pub use idtp;
use idtp::IDTP_PACKET_MIN_SIZE;

/// IDTP frame size in bytes.
pub const FRAME_SIZE: usize = IDTP_PACKET_MIN_SIZE + size_of::<Payload>();
