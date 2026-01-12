// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Mediator between IMU and AHRS.

#![no_std]

use ahrs_common::{
    FRAME_SIZE,
    idtp::{IDTP_PREAMBLE, IDTP_TRAILER, IDTP_TRAILER_SIZE, IdtpFrame, Mode},
    utils::calculate_checksum,
};
use crc::Crc;

/// Validate IDTP frame.
///
/// # Parameters
/// - `frame_bytes` - given raw IDTP frame bytes slice.
/// - `prev_seq` - given previous frame sequence number.
///
/// # Returns
/// - Current frame sequence number.
pub fn validate_frame(frame_bytes: &[u8], prev_seq: u32) -> Result<u32, ()> {
    let mut frame = IdtpFrame::from(frame_bytes);
    let mut header = frame.header();

    // Checking preamble & trailer;
    let preamble = header.preamble.as_slice();
    let trailer = &frame_bytes[FRAME_SIZE - IDTP_TRAILER_SIZE..FRAME_SIZE];

    if preamble != IDTP_PREAMBLE || trailer != IDTP_TRAILER {
        return Err(());
    }

    // Checking correctness of the sequence number.
    let sequence = header.sequence;

    if sequence <= prev_seq {
        return Err(());
    }

    // Checking checksum & CRC if needed.
    let checksum = header.checksum;
    let crc = header.crc;

    header.checksum = 0;
    header.crc = 0;

    let mut buffer = [0u8; FRAME_SIZE];
    frame.set_header(&header);

    if frame.pack(&mut buffer).is_err() {
        return Err(());
    }

    let correct_checksum = calculate_checksum(&buffer);

    if checksum != correct_checksum {
        return Err(());
    }

    let mode = header.mode;

    match mode {
        Mode::Normal => {
            // From IDTP specification:
            // https://github.com/alkuzin/idtp/blob/main/docs/SPECIFICATION.md
            //
            // In IDTP-N (Normal mode) the crc field MUST be unused
            // and filled with zeros.
            if crc != 0 {
                return Err(());
            }
        }
        Mode::Safety => {
            let crc_instance = Crc::<u32>::new(&crc::CRC_32_AUTOSAR);
            let mut digest = crc_instance.digest();
            digest.update(&buffer);

            let correct_crc = digest.finalize();

            if crc != correct_crc {
                return Err(());
            }
        }
        Mode::Unknown => return Err(()),
    }

    Ok(sequence)
}
