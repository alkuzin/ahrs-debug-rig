// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Mediator between IMU and AHRS.

#![no_std]
#![warn(clippy::all, clippy::correctness, clippy::suspicious)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::todo,
    clippy::unreachable,
    missing_docs
)]
