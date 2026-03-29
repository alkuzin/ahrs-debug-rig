// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Frame acquisition task related declarations.

use crate::{
    tasks::status::set_system_status,
    types::{SystemStatus, FrameChannel, FrameMessage},
    hal::{SpiDriver, DMA_BUFFER_SIZE},
};
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::{dma::DmaRxBuf, gpio::Output};

/// Frame communication channel.
static FRAME_CHANNEL: FrameChannel = Channel::new();

/// Get current frame.
///
/// # Returns
/// - Current frame.
pub async fn get_frame_message() -> FrameMessage {
    FRAME_CHANNEL.receive().await
}

/// Task for handling frame acquisition.
///
/// # Parameters
/// - `imu` - given IMU driver to handle.
#[embassy_executor::task]
pub async fn frame_acquisition_task(
    mut spi: SpiDriver<'static>,
    mut dma_rx_buf: DmaRxBuf,
    mut esp_ready: Output<'static>
) {
    loop {
        esp_ready.set_high();
        let transfer = match spi.read(DMA_BUFFER_SIZE, dma_rx_buf) {
            Ok(res) => res,
            Err(_) => {
                loop {
                    set_system_status(SystemStatus::Error).await;
                    Timer::after(Duration::from_millis(100)).await;
                    set_system_status(SystemStatus::Default).await;
                }
            },
        };

        let (spi_back, rx_buf_back) = transfer.wait();
        esp_ready.set_low();

        spi = spi_back;
        dma_rx_buf = rx_buf_back;

        let received_data = dma_rx_buf.as_slice();
        let msg = FrameMessage::new(received_data);
        FRAME_CHANNEL.send(msg).await;
    }
}
