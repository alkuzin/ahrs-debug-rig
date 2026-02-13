// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU firmware status related declarations.

use crate::types::{StatusLed, SystemStatus};
use core::ops::{Deref, DerefMut};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex,
};
use embassy_time::Ticker;

/// Current system status.
static SYSTEM_STATUS: Mutex<CriticalSectionRawMutex, SystemStatus> =
    Mutex::new(SystemStatus::Initializing);

/// Set current system status.
///
/// # Parameters
/// - `status` - given system status to set.
pub async fn set_system_status(status: SystemStatus) {
    let mut guard = SYSTEM_STATUS.lock().await;
    let system_status = guard.deref_mut();
    *system_status = status;
}

/// Get current system status.
///
/// # Returns
/// - Current system status.
pub async fn get_system_status() -> SystemStatus {
    let guard = SYSTEM_STATUS.lock().await;
    let system_status = guard.deref();
    *system_status
}

/// Task for handling system status update.
///
/// # Parameters
/// - `led` - given status led to handle.
/// - `ticker` - given status ticker to handle.
#[embassy_executor::task]
pub async fn system_status_task(
    mut led: StatusLed<'static>,
    mut ticker: Ticker,
) {
    loop {
        // Waiting for the next tick.
        ticker.next().await;

        // Handling current system status.
        let status = get_system_status().await;

        match status {
            // Green color.
            SystemStatus::Ok => led.set_state(false, true, false),
            // Yellow color.
            SystemStatus::Warning => led.set_state(true, true, false),
            // Red color.
            SystemStatus::Error => led.set_state(true, false, false),
            // White color.
            SystemStatus::Initializing => led.set_state(true, true, true),
        }
    }
}
