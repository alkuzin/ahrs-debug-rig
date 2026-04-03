// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! IMU readings transfer task related declarations.

use crate::{
    hal::peripherals::SpiDriver,
    prelude::*,
    tasks::{imu::get_imu_sample, status::set_system_status},
    types::SystemStatus,
};
use core::sync::atomic::{self, AtomicU32};
use embassy_stm32::gpio::{Input, Output};
use embassy_time::{Duration, Timer, with_timeout};
use indtp::{
    Frame,
    engines::{SwCryptoEngine, SwIntegrityEngine},
    payload::PayloadType,
    types::Packable,
};

/// Idle timeout in ms.
const IDLE_WAIT: Duration = Duration::from_millis(2);

/// Data aggregation timeout.
const TIMEOUT: Duration = Duration::from_millis(3);

/// Aligned buffer for DMA.
#[repr(align(32))]
struct AlignedBuffer<T>(pub T);

/// Size of DMA buffer in bytes.
const DMA_BUFFER_SIZE: usize = 56;

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
                match spi.write(&buffer[..size]).await {
                    Ok(()) => {
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

    frame.set_sequence(get_next_sequence() as u16);

    let collect = async {
        let sample = get_imu_sample().await;
        frame.push_single_sample(sample.timestamp, sample.data.to_bytes())?;
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
