// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Inertial Measurement Unit (IMU) driver implementation.

use embassy_stm32::{i2c::{I2c, Master}, mode::Async};
use idtp::payload::Imu6;

/// Inertial Measurement Unit (IMU) driver.
pub struct Imu;

impl Imu {
    /// Construct new `Imu` object.
    /// 
    /// # Parameters
    /// - `i2c` - given I2C driver to handle.
    /// 
    /// # Returns
    /// - New `Imu` object.
    pub async fn new(_i2c: I2c<'static, Async, Master>) -> Self {
        Self {}
    }

    /// Get all IMU sensors readings (accelerometer & gyroscope).
    /// 
    /// # Returns
    /// - Accelerometer & gyroscope readings in case of success.
    /// - `Err` - otherwise.
    pub async fn read_all(&mut self) -> Result<Imu6, ()> {
        Ok(Imu6::default())
    }
}
