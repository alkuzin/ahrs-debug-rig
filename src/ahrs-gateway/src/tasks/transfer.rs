// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Frame transfer over WiFi task related declarations.

use crate::{
    tasks::{status::set_system_status, recv::get_frame_message},
    types::{Result, FrameMessage, SystemStatus},
    hal::DMA_BUFFER_SIZE,
};
use indtp::{
    Frame,
    engines::{SwCryptoEngine, SwIntegrityEngine, CryptographyEngine, IntegrityEngine},
    types::{Packable, CryptoKeys},
    payload::Imu6,
};

/// Task for handling IMU data transfer.
///
/// # Parameters
/// - `spi` - given SPI driver to handle.
/// - `spi_ss` - given SPI slave select to handle.
/// - `esp_ready` - given ESP ready pin to handle.
#[embassy_executor::task]
pub async fn transfer_data_task() {
    let msg = get_frame_message().await;

    if let Err(_) = transfer_frame::<SwIntegrityEngine, SwCryptoEngine>(msg).await {
        set_system_status(SystemStatus::Error).await;
    }
}

/// Repack & transfer frame.
///
/// # Parameters
/// - `msg` - given frame message to handle.
///
/// # Returns
/// - `Ok` - in case of success.
/// - `Err` - otherwise.
///
/// # Errors
/// - Buffer underflow.
/// - Buffer overflow.
/// - Parse errors.
/// - Invalid operation.
async fn transfer_frame<I, C>(mut msg: FrameMessage) -> Result<()>
where
    I: IntegrityEngine,
    C: CryptographyEngine,
{
    let keys = CryptoKeys::new(*crate::AES_KEY, *crate::HMAC_KEY);
    let received_frame = Frame::parse::<I, C>(msg.data.as_mut_slice(), Some(&keys))?;
    let iterator = received_frame.read_batch_samples(size_of::<Imu6>())?;

    for (_, result) in iterator.enumerate() {
        let (timestamp, data) = result?;
        let sample = Imu6::from_bytes(data)?;

        let mut buffer = [0u8; DMA_BUFFER_SIZE];
        let mut frame = Frame::new_trusted(
            buffer.as_mut_slice(),
            received_frame.header().device_id,
            received_frame.header().payload_type,
        )?;

        frame.set_batch(false);
        frame.push_single_sample(timestamp, sample.to_bytes())?;

        let _ = frame.pack::<I, C>(Some(&keys))?;
        let _packed_frame = frame.frame_mut()?;

        // TODO: transfer over WiFi
    }

    Ok(())
}
