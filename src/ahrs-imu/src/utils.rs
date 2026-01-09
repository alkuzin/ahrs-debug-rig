// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Utilities for IMU handler firmware.

use ahrs_common::{payload::Payload, utils::Rng};
use core::ops::Range;

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
    let mut rng = Rng::new(state);
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

/// Halt the CPU after a fatal error.
pub fn halt_cpu() {
    cortex_m::interrupt::disable();
    loop {
        cortex_m::asm::wfi();
    }
}
