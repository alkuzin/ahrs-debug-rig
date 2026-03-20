// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU readings transfer task related declarations.

use core::sync::atomic::{self, AtomicU32};
use embassy_stm32::gpio::{Input, Output};
use embassy_time::{with_timeout, Duration, Timer};
use indtp::{engines::{SwCryptoEngine, SwIntegrityEngine}, Frame, payload::PayloadType, types::Packable};
use crate::{
    prelude::*,
    hal::peripherals::SpiDriver,
    tasks::{imu::get_imu_sample, status::set_system_status},
    types::SystemStatus,
};

/// SPI timeout in ms.
const SPI_TIMEOUT: Duration = Duration::from_millis(50);

/// Idle timeout in ms.
const IDLE_WAIT: Duration = Duration::from_millis(2);

/// Number of samples per frame.
const AGGREGATION_SIZE: usize = 5;

/// Data aggregation timeout in ms.
pub const AGGREGATION_TIMEOUT: u32 = {
    (AGGREGATION_SIZE as u32 * 1000 / crate::SAMPLE_RATE_HZ as u32) + 10
};

/// Data aggregation timeout.
const TIMEOUT: Duration = Duration::from_millis(AGGREGATION_TIMEOUT as u64);

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
    let mut buffer = [0u8; 256];

    loop {
        if esp_ready.is_high() {
            let size = pack_frame(&mut buffer).await;

            spi_ss.set_low();

            if let Ok(size) = size {
                match with_timeout(SPI_TIMEOUT, spi.write(&buffer[..size])).await {
                    Ok(Ok(())) => set_system_status(SystemStatus::Ok).await,
                    Ok(Err(_)) => set_system_status(SystemStatus::Warning).await,
                    Err(_) => {
                        defmt::warn!("SPI write timeout");
                        set_system_status(SystemStatus::Warning).await;
                    }
                }
            }
            else {
                set_system_status(SystemStatus::Error).await;
            }

            spi_ss.set_high();
        }
        else {
            Timer::after(IDLE_WAIT).await;
        }
    }
}

/// Pack frame before transfer.
///
/// # Parameters
/// - `buffer` - given frame buffer to handle.
///
/// # Returns
/// - Raw frame - in case of success.
/// - `Err` - otherwise.
///
/// # Errors
/// - Buffer underflow.
/// - Buffer overflow.
/// - Parse errors.
/// - Invalid operation.
async fn pack_frame(buffer: &mut [u8]) -> Result<usize> {
    let mut frame = Frame::new_lite(
        buffer,
        crate::DEVICE_ID,
        PayloadType::Imu6.into(),
    )?;

    frame.set_batch(true);
    frame.set_sequence(get_next_sequence() as u16);

    let mut batch = frame.start_batch()?;

    let collect = async {
        for _ in 0..AGGREGATION_SIZE {
            let sample = get_imu_sample().await;
            batch.push_sample(sample.timestamp, &sample.data.to_bytes())?;
        }
        Ok::<(), indtp::Error>(())
    };

    match with_timeout(TIMEOUT, collect).await {
        Ok(Ok(())) => {},
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => return Err(Error::Timeout),
    };

    drop(batch);

    frame.pack::<SwIntegrityEngine, SwCryptoEngine>(None).map_err(|e| e.into())
}
