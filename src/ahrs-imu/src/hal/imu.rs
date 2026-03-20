// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Inertial Measurement Unit (IMU) driver implementation.

use crate::{drivers::Mpu6050, hal::peripherals::I2cDriver, prelude::*};
use indtp::payload::Imu6;

/// Inertial Measurement Unit (IMU) driver.
pub struct Imu {
    imu: Mpu6050,
}

impl Imu {
    /// Construct new `Imu` object.
    ///
    /// # Parameters
    /// - `i2c` - given I2C driver to handle.
    ///
    /// # Returns
    /// - New `Imu` object.
    pub async fn new(i2c: I2cDriver) -> Result<Self> {
        let mut imu = Mpu6050::new(i2c);
        imu.init().await?;
        Ok(Self { imu })
    }

    /// Get all IMU sensors readings (accelerometer & gyroscope).
    ///
    /// # Returns
    /// - Accelerometer & gyroscope readings in case of success.
    /// - `Err` - otherwise.
    #[inline]
    pub async fn read_all(&mut self) -> Result<Imu6> {
        self.imu.read_all().await
    }
}
