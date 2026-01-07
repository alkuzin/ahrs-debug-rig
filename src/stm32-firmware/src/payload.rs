// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU data readings payload declaration.

use core::mem;

/// IDTP payload struct.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct Payload {
    pub acc1_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 1)
    /// along the Y axis (m/s^2).
    pub acc1_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 1)
    /// along the Z axis (m/s^2).
    pub acc1_z: f32,
    /// Angular velocity (gyroscope 1) along the X axis (rad/s).
    pub gyr1_x: f32,
    /// Angular velocity (gyroscope 1) along the Y axis (rad/s).
    pub gyr1_y: f32,
    /// Angular velocity (gyroscope 1) along the Z axis (rad/s).
    pub gyr1_z: f32,
    /// (Magnetometer 1) value along the X axis (Gauss).
    pub mag1_x: f32,
    /// (Magnetometer 1) value along the Y axis (Gauss).
    pub mag1_y: f32,
    /// (Magnetometer 1) value along the Z axis (Gauss).
    pub mag1_z: f32,
    /// The value of the projection of the acceleration vector (accelerometer 2)
    /// along the X axis (m/s^2).
    pub acc2_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 2)
    /// along the Y axis (m/s^2).
    pub acc2_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 2)
    /// along the Z axis (m/s^2).
    pub acc2_z: f32,
    /// Angular velocity (gyroscope 2) along the X axis (rad/s).
    pub gyr2_x: f32,
    /// Angular velocity (gyroscope 2) along the Y axis (rad/s).
    pub gyr2_y: f32,
    /// Angular velocity (gyroscope 2) along the Z axis (rad/s).
    pub gyr2_z: f32,
    /// (Magnetometer 2) value along the X axis (Gauss).
    pub mag2_x: f32,
    /// (Magnetometer 2) value along the Y axis (Gauss).
    pub mag2_y: f32,
    /// (Magnetometer 2) value along the Z axis (Gauss).
    pub mag2_z: f32,
    /// The value of the projection of the acceleration vector (accelerometer 3)
    /// along the X axis (m/s^2).
    pub acc3_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 3)
    /// along the Y axis (m/s^2).
    pub acc3_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 3)
    /// along the Z axis (m/s^2).
    pub acc3_z: f32,
    /// Angular velocity (gyroscope 3) along the X axis (rad/s).
    pub gyr3_x: f32,
    /// Angular velocity (gyroscope 3) along the Y axis (rad/s).
    pub gyr3_y: f32,
    /// Angular velocity (gyroscope 3) along the Z axis (rad/s).
    pub gyr3_z: f32,
    /// (Magnetometer 3) value along the X axis (Gauss).
    pub mag3_x: f32,
    /// (Magnetometer 3) value along the Y axis (Gauss).
    pub mag3_y: f32,
    /// (Magnetometer 3) value along the Z axis (Gauss).
    pub mag3_z: f32,
    /// The value of the projection of the acceleration vector (accelerometer 4)
    /// along the X axis (m/s^2).
    pub acc4_x: f32,
    /// The value of the projection of the acceleration vector (accelerometer 4)
    /// along the Y axis (m/s^2).
    pub acc4_y: f32,
    /// The value of the projection of the acceleration vector (accelerometer 4)
    /// along the Z axis (m/s^2).
    pub acc4_z: f32,
    /// Angular velocity (gyroscope 4) along the X axis (rad/s).
    pub gyr4_x: f32,
    /// Angular velocity (gyroscope 4) along the Y axis (rad/s).
    pub gyr4_y: f32,
    /// Angular velocity (gyroscope 4) along the Z axis (rad/s).
    pub gyr4_z: f32,
    /// (Magnetometer 4) value along the X axis (Gauss).
    pub mag4_x: f32,
    /// (Magnetometer 4) value along the Y axis (Gauss).
    pub mag4_y: f32,
    /// (Magnetometer 4) value along the Z axis (Gauss).
    pub mag4_z: f32,
    /// Pressure value (barometer 1) (Pascal).
    pub baro1: f32,
    /// Pressure value (barometer 2) (Pascal).
    pub baro2: f32,
    /// Pressure value (barometer 3) (Pascal).
    pub baro3: f32,
    /// Pressure value (barometer 4) (Pascal).
    pub baro4: f32,
}

/// Payload size in bytes.
pub const PAYLOAD_SIZE: usize = size_of::<Payload>();

impl Payload {
    /// Convert payload to bytes.
    ///
    /// # Returns
    /// - Payload byte array.
    pub fn as_bytes(&self) -> [u8; PAYLOAD_SIZE] {
        unsafe {
            mem::transmute::<Self, [u8; PAYLOAD_SIZE]>(*self)
        }
    }

    /// Convert a byte slice to a `Payload` struct.
    ///
    /// # Parameters
    /// - `bytes` - given bytes to convert.
    ///
    /// # Returns
    /// - Payload from bytes.
    pub fn from_bytes(bytes: &[u8; PAYLOAD_SIZE]) -> Self {
        unsafe {
            mem::transmute::<[u8; PAYLOAD_SIZE], Self>(*bytes)
        }
    }
}
