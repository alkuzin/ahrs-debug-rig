// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Common types declarations.

use core::{
    error,
    fmt::{Display, Formatter},
    result,
};

/// System status levels.
#[derive(Copy, Clone)]
pub enum SystemStatus {
    /// All subsystems operational.
    Ok,
    /// Critical failure.
    Error,
}

/// System errors enumeration.
#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    /// Async operation timeout.
    Timeout,
    /// INDTP protocol error.
    ProtocolError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

/// Result alias.
pub type Result<T> = result::Result<T, Error>;

impl From<indtp::Error> for Error {
    fn from(_: indtp::Error) -> Self {
        Error::ProtocolError
    }
}
