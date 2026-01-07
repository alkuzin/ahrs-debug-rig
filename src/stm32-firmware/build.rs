// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

use std::process::Command;

fn main() {
    let objcopy_status = Command::new("arm-none-eabi-objcopy")
        .arg("-O")
        .arg("binary")
        .arg("target/thumbv7em-none-eabihf/release/stm32-firmware")
        .arg("firmware.bin")
        .status()
        .expect("Failed to execute objcopy");

    if objcopy_status.success() {
        let flash_status = Command::new("st-flash")
            .arg("write")
            .arg("firmware.bin")
            .arg("0x08000000")
            .status()
            .expect("Failed to execute st-flash");

        if !flash_status.success() {
            eprintln!("Firmware flash failed");
        }
    }
}