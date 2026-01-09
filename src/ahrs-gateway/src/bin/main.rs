// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! AHRS gateway entry point.

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types,\
    especially those holding buffers for the duration of a data transfer."
)]

use ahrs_common::idtp::IdtpFrame;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    dma::DmaRxBuf,
    dma_buffers,
    gpio::{Level, Output, OutputConfig},
    main,
    spi::Mode,
    spi::slave::Spi,
    timer::timg::TimerGroup,
};
use log::info;
use ahrs_common::FRAME_SIZE;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let dp = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(dp.TIMG0);
    esp_rtos::start(timg0.timer0);

    let mut led = Output::new(dp.GPIO2, Level::Low, OutputConfig::default());
    let mut _led_error =
        Output::new(dp.GPIO4, Level::Low, OutputConfig::default());

    info!("Initialized successfully");

    let dma_channel = dp.DMA_SPI2;
    let spi_sck = dp.GPIO18;
    let spi_miso = dp.GPIO19;
    let spi_mosi = dp.GPIO23;
    let spi_ss = dp.GPIO5;

    let (rx_buffer, rx_descriptors, _tx_buffer, _tx_descriptors) =
        dma_buffers!(FRAME_SIZE);

    let mut spi = Spi::new(dp.SPI2, Mode::_1)
        .with_sck(spi_sck)
        .with_miso(spi_miso)
        .with_mosi(spi_mosi)
        .with_cs(spi_ss)
        .with_dma(dma_channel);

    info!("SPI Slave initialized, waiting for STM32...");

    let mut dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();

    let mut error_counter: u32 = 0;
    let mut prev_sequence: u32 = 0;
    let mut counter = 0;

    loop {
        let transfer = spi.read(FRAME_SIZE, dma_rx_buf).unwrap();
        let (spi_back, rx_buf_back) = transfer.wait();

        spi = spi_back;
        dma_rx_buf = rx_buf_back;

        let received_data = dma_rx_buf.as_slice();

        let idtp = IdtpFrame::from(&received_data[0..196]);
        let header = idtp.header();

        let sequence = header.sequence;

        if sequence < prev_sequence || sequence - prev_sequence > 1 {
            error_counter += 1;
        }

        if counter == 100 {
            counter = 0;
            info!("Error counter: {}", error_counter);
            info!("Header: {:X?}", &received_data[0..32]);
        }

        prev_sequence = sequence;

        led.toggle();
        counter += 1;
    }
}
