# IMU data acquisition & AHRS debugging stand

## Overview

This repository contains the firmware and hardware schemes for a high-performance Inertial Measurement Unit (IMU) data acquisition system, developed as part of the `"Software for Autonomous Navigation Systems"` thesis project.

![Debug Rig](/res/debug-rig1.jpg)

![Debug Rig](/res/debug-rig2.jpg)

The system implements a reliable, low-latency bridge between a sensor processing unit (**STM32**) and a communication gateway (**ESP32**) using the custom [`IDTP (Inertial Measurement Unit Data Transfer Protocol)`](https://github.com/alkuzin/idtp).

## 🛰 System Architecture

The debugging stand consists of two primary hardware layers:

> **Sensor Node (STM32F401CCU6)**:
Generates accelerometer, gyroscope, magnetometer, and barometer sensors readings.
Performs primary processing and packs data into IDTP frames.
Transmits data via SPI + DMA to gateway node.

> **Gateway Node: (ESP32)**: Receives IDTP frames via SPI + DMA.

## 🛠 Hardware Setup

![Debug Rig Scheme](/res/scheme.png)

The project uses a custom-built prototype board (breadboard-based) designed for signal integrity testing:

- **Interconnect**: SPI (SCK, MISO, MOSI) + SS (Slave Select).
- **Debugging**: Dedicated pins for logic analyzer connection.
- **Status Indication**: LED indicators for protocol state and error tracking.

## 📦 IDTP Protocol Overview

![SPI Debugging](/res/spi_debug1.png)

**Inertial Measurement Unit Data Transfer Protocol (IDTP)** — it is a binary protocol that can be used by different transport layers, such as SPI, I2C, UART, UDP or TCP. This protocol designed for transfering navigation data in systems with strict real-time requirements (unmanned vehicles, robotics).

IDTP solves the problem of unifying data exchange between different types of IMUs and host systems, providing a multi-level data integrity checking.

> For complete technical implementation of protocol read [specification](https://github.com/alkuzin/idtp/blob/main/docs/SPECIFICATION.md).

## 🚀 Key Features & Optimizations

> **Zero-Copy with DMA**:

Both STM32 and ESP32 use **Direct Memory Access (DMA)** to handle SPI transactions, minimizing CPU overhead.

> **Embedded Rust**:

Entirely written in `Rust`, leveraging type safety and memory ownership to prevent common memory & concurrency bugs.

> **Synchronization Logic**:

- **Guard Intervals**: Implemented micro-delays (20µs/5µs) to account for ESP32 RTOS context switching.

- **Frame Alignment**: Strict IDTP frame-sized DMA buffer synchronization in order to prevent byte-shifting during high-frequency transmission.

## 📈 Debugging Results

![SPI Debugging](/res/spi_debug2.jpg)

During testing, the system achieved a **0-2% Packet Error Rate (PER)** after the initial synchronization phase with sample rate: **200 Hz** and SPI frequency: **3 MHz**.

## 🛠 How to Build & Run

STM32 firmware (OpenOCD + GDB + st-flash):

In separate terminal run:
```console
openocd -f interface/stlink.cfg -f target/stm32f4x.cfg
```

In separate terminal run:
```console
cd src/ahrs-imu
cargo run --release
```

ESP32 firmware (espflash):
```console
cd src/ahrs-gateway
cargo run --release
```

> WARNING: Use code with caution.

## 📜 License

> This project is developed for academic purposes as part of a Bachelor's thesis.

Copyright (C) 2026-present ahrs-debug-rig project and contributors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
