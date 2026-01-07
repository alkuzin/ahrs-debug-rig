// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Utilities for IMU handler firmware.

use core::ops::Range;
use crate::payload::Payload;

/// Pseudo-random numbers generator.
///
/// Based on the `Xorshift algorithm` (George Marsaglia) - a type of LFSR
/// (Linear Feedback Shift Register).
struct Rng {
    /// Initial state of the generator.
    state: u32,
}

impl Rng {
    /// Construct new `Rng` object.
    ///
    /// # Parameters
    /// - `seed` - given initial state of the generator.
    ///
    /// # Returns
    /// - New `Rng` object.
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Generate next pseudo-random number.
    ///
    /// # Returns
    /// - Next pseudo-random number as u32.
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Generate float pseudo-random number in range [min, max].
    ///
    /// # Parameters
    /// - `range` - given range of number to generate.
    ///
    /// # Returns
    /// - Next pseudo-random number as f32.
    pub fn next_f32(&mut self, range: Range<f32>) -> f32 {
        let (min, max) = (range.start, range.end);
        let r = self.next_u32() as f32 / u32::MAX as f32;
        min + r * (max - min)
    }
}

/// Pseudo-random accelerometer readings range.
const RNG_ACC_RANGE: Range<f32> = -39.22..39.22; // +-4g.

/// Pseudo-random gyroscope readings range.
const RNG_GYR_RANGE: Range<f32> = -34.91..34.91; // 2000 DPS.

/// Pseudo-random magnetometer readings range.
const RNG_MAG_RANGE: Range<f32> = -8.0..8.0; // +-8 Gauss.

/// Pseudo-random barometer readings range.
const RNG_BARO_RANGE: Range<f32> = 95000.0..105000.0; // ~1 atm.

/// Generate payload with pseudo-random IMU sensors readings.
///
/// # Parameters
/// - `state` - given pseudo-random numbers generator initial state.
///
/// # Returns
/// - New generated payload.
pub fn generate_payload(state: u32) -> Payload {
    let mut rng     = Rng::new(state);
    let mut payload = Payload::default();

    payload.acc1_x = rng.next_f32(RNG_ACC_RANGE);
    payload.acc1_y = rng.next_f32(RNG_ACC_RANGE);
    payload.acc1_z = rng.next_f32(RNG_ACC_RANGE);
    payload.gyr1_x = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr1_y = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr1_z = rng.next_f32(RNG_GYR_RANGE);
    payload.mag1_x = rng.next_f32(RNG_MAG_RANGE);
    payload.mag1_y = rng.next_f32(RNG_MAG_RANGE);
    payload.mag1_z = rng.next_f32(RNG_MAG_RANGE);

    payload.acc2_x = rng.next_f32(RNG_ACC_RANGE);
    payload.acc2_y = rng.next_f32(RNG_ACC_RANGE);
    payload.acc2_z = rng.next_f32(RNG_ACC_RANGE);
    payload.gyr2_x = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr2_y = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr2_z = rng.next_f32(RNG_GYR_RANGE);
    payload.mag2_x = rng.next_f32(RNG_MAG_RANGE);
    payload.mag2_y = rng.next_f32(RNG_MAG_RANGE);
    payload.mag2_z = rng.next_f32(RNG_MAG_RANGE);

    payload.acc3_x = rng.next_f32(RNG_ACC_RANGE);
    payload.acc3_y = rng.next_f32(RNG_ACC_RANGE);
    payload.acc3_z = rng.next_f32(RNG_ACC_RANGE);
    payload.gyr3_x = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr3_y = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr3_z = rng.next_f32(RNG_GYR_RANGE);
    payload.mag3_x = rng.next_f32(RNG_MAG_RANGE);
    payload.mag3_y = rng.next_f32(RNG_MAG_RANGE);
    payload.mag3_z = rng.next_f32(RNG_MAG_RANGE);

    payload.acc4_x = rng.next_f32(RNG_ACC_RANGE);
    payload.acc4_y = rng.next_f32(RNG_ACC_RANGE);
    payload.acc4_z = rng.next_f32(RNG_ACC_RANGE);
    payload.gyr4_x = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr4_y = rng.next_f32(RNG_GYR_RANGE);
    payload.gyr4_z = rng.next_f32(RNG_GYR_RANGE);
    payload.mag4_x = rng.next_f32(RNG_MAG_RANGE);
    payload.mag4_y = rng.next_f32(RNG_MAG_RANGE);
    payload.mag4_z = rng.next_f32(RNG_MAG_RANGE);

    payload.baro1 = rng.next_f32(RNG_BARO_RANGE);
    payload.baro2 = rng.next_f32(RNG_BARO_RANGE);
    payload.baro3 = rng.next_f32(RNG_BARO_RANGE);
    payload.baro4 = rng.next_f32(RNG_BARO_RANGE);

    payload
}
