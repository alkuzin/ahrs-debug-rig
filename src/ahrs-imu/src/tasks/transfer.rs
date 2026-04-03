// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU readings transfer task related declarations.

use crate::{
    hal::peripherals::SpiDriver,
    prelude::*,
    tasks::{imu::get_imu_sample, status::set_system_status},
    types::SystemStatus,
};
use embassy_stm32::gpio::{Input, Output};
use embassy_time::{Duration, Timer, with_timeout};
use indtp::{
    Frame,
    engines::{SwCryptoEngine, SwIntegrityEngine},
    payload::PayloadType,
    types::Packable,
};

/// SPI timeout in ms.
const SPI_TIMEOUT: Duration = Duration::from_millis(50);

/// Idle timeout in ms.
const IDLE_WAIT: Duration = Duration::from_millis(2);

/// Number of samples per frame.
const AGGREGATION_SIZE: usize = 5;

/// Data aggregation timeout in ms.
pub const AGGREGATION_TIMEOUT: u32 =
    (AGGREGATION_SIZE as u32 * 1000 / crate::SAMPLE_RATE_HZ as u32) + 20;

/// Data aggregation timeout.
const TIMEOUT: Duration = Duration::from_millis(AGGREGATION_TIMEOUT as u64);

/// Aligned buffer for DMA.
#[repr(align(32))]
struct AlignedBuffer<T>(pub T);

/// Size of DMA buffer in bytes.
const DMA_BUFFER_SIZE: usize = 148;

/// Static DMA buffer.
static mut DMA_BUFFER: AlignedBuffer<[u8; DMA_BUFFER_SIZE]> =
    AlignedBuffer([0u8; DMA_BUFFER_SIZE]);

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
    let mut buffer = unsafe { DMA_BUFFER.0 };

    loop {
        if esp_ready.is_high() {
            buffer.fill(0);

            if let Ok(size) = pack_frame(&mut buffer).await {
                spi_ss.set_low();
                Timer::after(Duration::from_micros(1)).await;

                #[allow(clippy::indexing_slicing)]
                match with_timeout(SPI_TIMEOUT, spi.write(&buffer[..size]))
                    .await
                {
                    Ok(Ok(())) => {
                        // Guard interval.
                        Timer::after(Duration::from_micros(20)).await;
                        set_system_status(SystemStatus::Ok).await
                    }
                    _ => set_system_status(SystemStatus::Warning).await,
                }
            } else {
                set_system_status(SystemStatus::Error).await;
            }

            spi_ss.set_high();
        } else {
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
    let mut frame =
        Frame::new_lite(buffer, crate::DEVICE_ID, PayloadType::Imu6.into())?;

    frame.set_batch(true);

    let collect = async {
        let mut batch = frame.start_batch()?;

        for _ in 0..AGGREGATION_SIZE {
            let sample = get_imu_sample().await;
            batch.push_sample(sample.timestamp, sample.data.to_bytes())?;
        }

        drop(batch);
        Ok::<(), indtp::Error>(())
    };

    match with_timeout(TIMEOUT, collect).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => return Err(Error::Timeout),
    };

    frame
        .pack::<SwIntegrityEngine, SwCryptoEngine>(None)
        .map_err(|e| e.into())
}
