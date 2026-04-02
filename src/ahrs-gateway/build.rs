// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! ESP32 firmware build related declarations.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

/// Path for generated firmware configs.
const FIRMWARE_DIR: &str = "../configs/firmware";

/// Path for generated firmware config for ESP32.
const ESP32_CONFIG_PATH: &str = "esp32_config.rs";

fn main() {
    linker_be_nice();
    println!("cargo:rustc-link-arg=-Tlinkall.x");

    let config_dir = Path::new(FIRMWARE_DIR);
    let source_file = config_dir.join(ESP32_CONFIG_PATH);

    let out_dir = env::var("OUT_DIR").expect("Cannot get OUT_DIR");
    let dest_file = PathBuf::from(&out_dir).join(ESP32_CONFIG_PATH);

    if !config_dir.is_dir() {
        eprintln!(
            "cargo:warning=Build error: configs/firmware/ directory was not found"
        );
        eprintln!(
            "Generate configs using AHRS Monitor & copy it into the src/ directory"
        );
        process::exit(1);
    }

    if !source_file.exists() {
        eprintln!(
            "cargo:warning=Build error: file {} was not found",
            source_file.display()
        );
        process::exit(1);
    }

    if let Err(e) = fs::copy(&source_file, &dest_file) {
        eprintln!("cargo:warning=Error to copy file: {e}");
        process::exit(1);
    }

    println!("cargo:rerun-if-changed={}", source_file.display());
}

fn linker_be_nice() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                "esp_rtos_initialized"
                | "esp_rtos_yield_task"
                | "esp_rtos_task_create" => {
                    eprintln!();
                    eprintln!(
                        "💡 `esp-radio` has no scheduler enabled. Make sure you have initialized `esp-rtos` or provided an external scheduler."
                    );
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!(
                        "💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests"
                    );
                    eprintln!();
                }
                _ => (),
            },
            _ => process::exit(1),
        }

        process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        env::current_exe().unwrap().display()
    );
}
