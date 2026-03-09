// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU readings transfer task related declarations.

use core::sync::atomic::{self, AtomicU32};
use embassy_stm32::gpio::{Input, Output};
use indtp::{
    engines::{SwCryptoEngine, SwIntegrityEngine},
    Frame,
    payload::PayloadType,
    types::Packable
};
use crate::{
    hal::peripherals::SpiDriver,
    tasks::{imu::get_imu_sample, status::set_system_status},
    types::{SystemStatus, Sample},
};

/// Task for handling IMU data transfer.
///
/// # Parameters
/// - `spi` - given SPI driver to handle.
/// - `spi_ss` - given SPI slave select to handle.
/// - `esp_ready` - given ESP ready pin to handle.
#[embassy_executor::task]
pub async fn transfer_data_task(
    mut spi: SpiDriver,
    mut spi_ss: Output<'static>,
    esp_ready: Input<'static>,
) {
    let mut buffer = [0u8; 128];

    loop {
        let sample = get_imu_sample().await;
        let size = pack_frame(sample, &mut buffer).await;

        if esp_ready.is_high() {
            spi_ss.set_low();

            if let Ok(size) = size && spi.write(&buffer[..size]).await.is_ok() {
                set_system_status(SystemStatus::Error).await;
            }
            else {
                set_system_status(SystemStatus::Warning).await;
            }

            spi_ss.set_high();
        }
    }
}

/// Frame sequence number.
static SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// Get next sequence number.
///
/// # Returns
/// - Next sequence number.
#[inline]
fn get_next_sequence() -> u32 {
    SEQUENCE.fetch_add(1, atomic::Ordering::Relaxed)
}

/// Pack frame before transfer.
///
/// # Parameters
/// - `sample` - given IMU sample to handle.
/// - `buffer` - given frame buffer to handle.
///
/// # Returns
/// - Raw frame - in case of success.
/// - `Err` - otherwise.
async fn pack_frame(sample: Sample, buffer: &mut [u8]) -> indtp::Result<usize> {
    // TODO: use data aggregation.

    // Constructing INDTP frame in Lite mode.
    let device_id = crate::DEVICE_ID;
    let payload_type: u8 = PayloadType::Imu6.into();

    let mut frame =
        Frame::new_lite(buffer, device_id, payload_type)?;

    let (payload, timestamp) = (sample.data.to_bytes(), sample.timestamp);
    let sequence = get_next_sequence() as u16;

    // Setting payload, sequence number & packing frame.
    frame.push_single_sample(timestamp, payload)?;
    frame.set_sequence(sequence);
    frame.pack::<SwIntegrityEngine, SwCryptoEngine>(None)
}
