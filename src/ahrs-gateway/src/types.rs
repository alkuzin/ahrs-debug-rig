// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Common types declarations.

use core::{
    error,
    fmt::{Display, Formatter},
    result,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use crate::hal::DMA_BUFFER_SIZE;

/// System status levels.
#[derive(Copy, Clone)]
pub enum SystemStatus {
    /// Wi-Fi is enabled.
    WiFiEnabled,
    /// Critical failure.
    Error,
    /// Default state.
    Idle,
}

/// System errors enumeration.
#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    /// Async operation timeout.
    Timeout,
    /// INDTP protocol error.
    ProtocolError(indtp::Error),
    /// Network error.
    NetworkError,
    /// Other errors.
    Other,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Self {
        Error::Other
    }
}

/// Result alias.
pub type Result<T> = result::Result<T, Error>;

impl From<indtp::Error> for Error {
    fn from(error: indtp::Error) -> Self {
        Error::ProtocolError(error)
    }
}

/// Data frame message.
pub struct FrameMessage {
    /// Frame bytes.
    pub data: [u8; DMA_BUFFER_SIZE],
}

impl FrameMessage {
    /// Construct new frame message.
    ///
    /// # Parameters
    /// - `buffer` - given frame buffer to handle.
    ///
    /// # Returns
    /// - New frame message.
    pub fn new(buffer: &[u8]) -> Self {
        let mut data = [0u8; DMA_BUFFER_SIZE];
        let size = data.len().min(DMA_BUFFER_SIZE);

        data[..size].copy_from_slice(&buffer[..size]);
        Self { data }
    }
}

/// Alias for frame communication channel.
pub type FrameChannel = Channel<CriticalSectionRawMutex, FrameMessage, 4>;
