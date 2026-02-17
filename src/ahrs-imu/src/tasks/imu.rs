// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU readings acquisition task related declarations.

use crate::{
    tasks::status::set_system_status,
    types::{Sample, SystemStatus, ImuChannel},
    drivers::Imu,
};
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};

/// IMU communication channel.
static IMU_CHANNEL: ImuChannel = Channel::new();

/// Get current IMU sample.
///
/// # Returns
/// - Current IMU sample.
pub async fn get_imu_sample() -> Sample {
    IMU_CHANNEL.receive().await
}

/// Task for handling IMU data acquisition.
///
/// # Parameters
/// - `imu` - given IMU driver to handle.
#[embassy_executor::task]
pub async fn imu_acquisition_task(mut imu: Imu) {
    loop {
        Timer::after_millis(5).await;

        match imu.read_all().await {
            Ok(data) => {
                let sample = Sample {
                    data,
                    timestamp: Instant::now().as_millis() as u32,
                };

                IMU_CHANNEL.send(sample).await;
                set_system_status(SystemStatus::Ok).await;
            },
            Err(_) => set_system_status(SystemStatus::Error).await,
        }
    }
}
