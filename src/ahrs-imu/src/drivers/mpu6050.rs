// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! MPU-6050 driver implementation.

use crate::{hal::peripherals::I2cDriver, prelude::*};
use embassy_time::{Duration, with_timeout};
use indtp::{
    payload::{Imu3Acc, Imu3Gyr, Imu6},
    types::F32,
};

/// MPU-6050 registers enumeration.
#[repr(u8)]
enum Register {
    /// Power Management 1 register.
    PwrMgmt1 = 0x6B,
    /// Accelerometer Configuration register.
    AccelConfig = 0x1C,
    /// Gyroscope Configuration register.
    GyroConfig = 0x1B,
    /// Sample Rate Divider register.
    SmplrtDiv = 0x19,
    /// Configuration register.
    Config = 0x1A,
    /// Accelerometer X high bits value register.
    AccelXOutH = 0x3B,
    /// Accelerometer X low bits value register.
    AccelXOutL = 0x3C,
    /// Accelerometer Y high bits value register.
    AccelYOutH = 0x3D,
    /// Accelerometer Y low bits value register.
    AccelYOutL = 0x3E,
    /// Accelerometer Z high bits value register.
    AccelZOutH = 0x3F,
    /// Accelerometer Z low bits value register.
    AccelZOutL = 0x40,
    /// Gyroscope X high bits value register.
    GyroXOutH = 0x43,
    /// Gyroscope X low bits value register.
    GyroXOutL = 0x44,
    /// Gyroscope Y high bits value register.
    GyroYOutH = 0x45,
    /// Gyroscope Y low bits value register.
    GyroYOutL = 0x46,
    /// Gyroscope Z high bits value register.
    GyroZOutH = 0x47,
    /// Gyroscope Z low bits value register.
    GyroZOutL = 0x48,
}

/// Accelerometer LSB sensitivity (+-2g).
const ACCEL_LSB_SENS: f32 = 16384.0;

/// Gyroscope LSB sensitivity (+-2000 deg/s).
const GYRO_LSB_SENS: f32 = 16.4;

impl From<Register> for u8 {
    fn from(val: Register) -> Self {
        val as Self
    }
}

/// I2C default address for MPU-6050 (when AD0 low (GND)).
const MPU6050_DEFAULT_ADDRESS: u8 = 0x68;

/// MPU-6050 async operations timeout.
const MPU6050_TIMEOUT: Duration = Duration::from_millis(50);

/// Mpu6050 driver.
pub struct Mpu6050 {
    /// I2C driver.
    i2c: I2cDriver,
}

impl Mpu6050 {
    /// I2C address for MPU-6050.
    const ADDRESS: u8 = MPU6050_DEFAULT_ADDRESS;

    /// Construct new MPU-6050 driver.
    ///
    /// # Parameters
    /// - `i2c` - given i2c driver to handle.
    ///
    /// # Returns
    /// - New constructed MPU-6050 driver.
    pub fn new(i2c: I2cDriver) -> Self {
        Self { i2c }
    }

    /// Initialize MPU-6050 driver.
    ///
    /// # Returns
    /// - `Ok` - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I2C errors.
    /// - Timeout.
    pub async fn init(&mut self) -> Result<()> {
        // Waking up the MPU-6050.
        self.write(Register::PwrMgmt1.into(), 0x00).await?;

        // Configuring the accelerometer (+-2g).
        self.write(Register::AccelConfig.into(), 0x00).await?;

        // Configuring the gyroscope (+-2000 deg/s).
        self.write(Register::GyroConfig.into(), 0x03 << 3).await?;

        // Setting Digital Low Pass Filter (DLPF) for both the gyroscopes and
        // accelerometers.
        self.write(Register::Config.into(), 0x01).await?;

        // Sample Rate = Gyroscope Output Rate / (1 + SMPLRT_DIV), where
        // Gyroscope Output Rate is 1 kHz and
        let sample_rate = (1000 / crate::SAMPLE_RATE_HZ) as u8;
        self.write(Register::SmplrtDiv.into(), sample_rate).await?;

        Ok(())
    }

    /// Read accelerometer data.
    ///
    /// # Returns
    /// - Accelerometer readings on 3 axes - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I2C errors.
    /// - Timeout.
    pub async fn read_acc(&mut self) -> Result<(i16, i16, i16)> {
        let acc_x = i16::from_be_bytes([
            self.read(Register::AccelXOutH.into()).await?,
            self.read(Register::AccelXOutL.into()).await?,
        ]);

        let acc_y = i16::from_be_bytes([
            self.read(Register::AccelYOutH.into()).await?,
            self.read(Register::AccelYOutL.into()).await?,
        ]);

        let acc_z = i16::from_be_bytes([
            self.read(Register::AccelZOutH.into()).await?,
            self.read(Register::AccelZOutL.into()).await?,
        ]);

        Ok((acc_x, acc_y, acc_z))
    }

    /// Read gyroscope data.
    ///
    /// # Returns
    /// - Gyroscope readings on 3 axes - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I2C errors.
    /// - Timeout.
    pub async fn read_gyr(&mut self) -> Result<(i16, i16, i16)> {
        let gyr_x = i16::from_be_bytes([
            self.read(Register::GyroXOutH.into()).await?,
            self.read(Register::GyroXOutL.into()).await?,
        ]);

        let gyr_y = i16::from_be_bytes([
            self.read(Register::GyroYOutH.into()).await?,
            self.read(Register::GyroYOutL.into()).await?,
        ]);

        let gyr_z = i16::from_be_bytes([
            self.read(Register::GyroZOutH.into()).await?,
            self.read(Register::GyroZOutL.into()).await?,
        ]);

        Ok((gyr_x, gyr_y, gyr_z))
    }

    /// Read accelerometer & gyroscope data.
    ///
    /// # Returns
    /// - Accelerometer & gyroscope readings on 3 axes - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I2C errors.
    /// - Timeout.
    pub async fn read_all(&mut self) -> Result<Imu6> {
        let (acc_x, acc_y, acc_z) = self.read_acc().await?;
        let (gyr_x, gyr_y, gyr_z) = self.read_gyr().await?;

        let acc = Imu3Acc {
            acc_x: F32::new(f32::from(acc_x) / ACCEL_LSB_SENS),
            acc_y: F32::new(f32::from(acc_y) / ACCEL_LSB_SENS),
            acc_z: F32::new(f32::from(acc_z) / ACCEL_LSB_SENS),
        };

        let gyr = Imu3Gyr {
            gyr_x: F32::new(f32::from(gyr_x) / GYRO_LSB_SENS),
            gyr_y: F32::new(f32::from(gyr_y) / GYRO_LSB_SENS),
            gyr_z: F32::new(f32::from(gyr_z) / GYRO_LSB_SENS),
        };

        Ok(Imu6 { acc, gyr })
    }

    /// Read register.
    ///
    /// # Parameters
    /// - `reg` - given register to handle.
    ///
    /// # Returns
    /// - Register contents - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I2C errors.
    /// - Timeout.
    async fn read(&mut self, reg: u8) -> Result<u8> {
        let mut buffer = [0; 1];

        match with_timeout(
            MPU6050_TIMEOUT,
            self.i2c.write_read(Self::ADDRESS, &[reg], &mut buffer),
        )
        .await
        {
            Ok(Ok(())) => Ok(buffer[0]),
            Ok(Err(_)) => Err(Error::I2cError),
            Err(_) => Err(Error::Timeout),
        }
    }

    /// Write data into register.
    ///
    /// # Parameters
    /// - `reg` - given register to handle.
    /// - `val` - given value to write.
    ///
    /// # Returns
    /// - `Ok` - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - I2C errors.
    /// - Timeout.
    async fn write(&mut self, reg: u8, val: u8) -> Result<()> {
        match with_timeout(
            MPU6050_TIMEOUT,
            self.i2c.write(Self::ADDRESS, &[reg, val]),
        )
        .await
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(Error::I2cError),
            Err(_) => Err(Error::Timeout),
        }
    }
}
