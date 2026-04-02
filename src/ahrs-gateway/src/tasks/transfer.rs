// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Frame transfer over WiFi task related declarations.

use crate::{
    hal::DMA_BUFFER_SIZE,
    tasks::{recv::get_frame_message, status::set_system_status},
    types::{self, Error, FrameMessage, SystemStatus},
};
use core::str::FromStr;
use embassy_net::{Stack, udp::UdpSocket};
use indtp::{
    Frame,
    engines::{
        CryptographyEngine, IntegrityEngine, SwCryptoEngine, SwIntegrityEngine,
    },
    types::CryptoKeys,
};
use smoltcp::wire::{IpAddress, IpEndpoint};

/// Task for handling IMU data transfer.
///
/// # Parameters
/// - `net_stack` - given network stack to handle.
#[embassy_executor::task]
pub async fn transfer_data_task(net_stack: Stack<'static>) {
    let mut rx_meta = [embassy_net::udp::PacketMetadata::EMPTY; 1];
    let mut rx_buffer = [0u8; 256]; // 256
    let mut tx_meta = [embassy_net::udp::PacketMetadata::EMPTY; 1];
    let mut tx_buffer = [0u8; 256]; // 512

    let mut socket = UdpSocket::new(
        net_stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let monitor_ip = IpAddress::from_str(crate::MONITOR_IP.trim())
        .unwrap_or(IpAddress::v4(0, 0, 0, 0));
    let endpoint = IpEndpoint::new(monitor_ip, crate::MONITOR_PORT);

    if let Err(e) = socket.bind(0) {
        set_system_status(SystemStatus::Error).await;
        panic!("failed to bind socket: {:?}", e);
    }

    loop {
        let msg = get_frame_message().await;

        if let Err(_) = transfer_frame::<SwIntegrityEngine, SwCryptoEngine>(
            msg, &socket, endpoint,
        )
        .await
        {
            set_system_status(SystemStatus::Error).await;
        }
    }
}

/// Repack & transfer frame.
///
/// # Parameters
/// - `msg` - given frame message to handle.
/// - `socket` - given UDP socket to handle.
/// - `endpoint` - given AHRS Monitor endpoint.
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
async fn transfer_frame<I, C>(
    mut msg: FrameMessage,
    socket: &UdpSocket<'_>,
    endpoint: IpEndpoint,
) -> types::Result<()>
where
    I: IntegrityEngine,
    C: CryptographyEngine,
{
    let keys = CryptoKeys::new(*crate::AES_KEY, *crate::HMAC_KEY);
    let data = msg.data.as_mut_slice();

    if let Ok(received_frame) = Frame::parse::<I, C>(data, None) {
        let (timestamp, sample) = received_frame.read_single_sample()?;

        let mut buffer = [0u8; DMA_BUFFER_SIZE];
        let mut frame = Frame::new_trusted(
            buffer.as_mut_slice(),
            received_frame.header().device_id,
            received_frame.header().payload_type,
        )?;

        frame.set_batch(false);
        frame.set_encrypted(true);
        frame.push_single_sample(timestamp, sample)?;

        let _ = frame.pack::<I, C>(Some(&keys))?;
        let frame_to_send = frame.frame()?;

        if let Err(_) = socket.send_to(frame_to_send, endpoint).await {
            return Err(Error::NetworkError);
        }
    }

    Ok(())
}
