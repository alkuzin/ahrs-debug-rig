// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! STM32 firmware build related declarations.

use std::{env, fs, path::{Path, PathBuf}, process};

/// Path for generated firmware configs.
const FIRMWARE_DIR: &str = "../configs/firmware";

/// Path for generated firmware config for STM32.
const STM32_CONFIG_PATH: &str = "stm32_config.rs";

fn main() {
    let config_dir = Path::new(FIRMWARE_DIR);
    let source_file = config_dir.join(STM32_CONFIG_PATH);

    let out_dir = env::var("OUT_DIR").expect("Не удалось получить OUT_DIR");
    let dest_file = PathBuf::from(&out_dir).join(STM32_CONFIG_PATH);

    if !config_dir.is_dir() {
        eprintln!("cargo:warning=Build error: configs/firmware/ directory was not found");
        eprintln!("Generate configs using AHRS Monitor & copy it into the src/ directory");
        process::exit(1);
    }

    if !source_file.exists() {
        eprintln!("cargo:warning=Build error: file {} was not found", source_file.display());
        process::exit(1);
    }

    if let Err(e) = fs::copy(&source_file, &dest_file) {
        eprintln!("cargo:warning=Error to copy file: {e}");
        process::exit(1);
    }

    println!("cargo:rerun-if-changed={}", source_file.display());
}
