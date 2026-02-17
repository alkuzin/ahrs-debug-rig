// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! MCU peripherals related declarations.

use crate::types::StatusLed;
use embassy_stm32::{
    Peripherals,
    gpio::{Level, Output, Speed},
    bind_interrupts,
    peripherals,
    i2c::{self, I2c},
    mode::Async,
    time::Hertz,
    spi::{self, Spi},
};

/// Alias for I2C driver.
pub type I2cDriver = I2c<'static, Async, i2c::mode::Master>;

/// Alias for SPI driver.
pub type SpiDriver = Spi<'static, Async, spi::mode::Master>;

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Builtin LED handler.
    pub builtin_led: Output<'static>,
    /// Status LED handler.
    pub status_led: StatusLed<'static>,
    /// I2C handler for IMU.
    pub i2c: I2cDriver, 
    /// SPI handler.
    pub spi: SpiDriver,
    /// SPI slave select.
    pub spi_ss: Output<'static>,
}

impl SystemPeripherals {
    /// Construct & initialize IMU handler system peripherals.
    ///
    /// # Parameters
    /// - `p` - given STM32 peripherals to handle.
    ///
    /// # Returns
    /// - Initialize IMU handler system peripherals.
    pub fn new(p: Peripherals) -> Self {
        let led_status_red_pin = p.PA9;
        let led_status_green_pin = p.PA10;
        let led_status_blue_pin = p.PA11;
        let builtin_led_pin = p.PC13;

        let led_r = Output::new(led_status_red_pin, Level::High, Speed::Low);
        let led_g = Output::new(led_status_green_pin, Level::High, Speed::Low);
        let led_b = Output::new(led_status_blue_pin, Level::High, Speed::Low);

        let builtin_led = Output::new(builtin_led_pin, Level::High, Speed::Low);
        let status_led = StatusLed::new(led_r, led_g, led_b, false);

        // Setting I2C.
        let mut i2c_cfg = i2c::Config::default();
        // I2C fast mode (400 kHz).
        i2c_cfg.frequency = Hertz(400_000);

        let i2c_scl = p.PB6;
        let i2c_sda = p.PB7;
        let i2c_tx_dma = p.DMA1_CH6;
        let i2d_rx_dma = p.DMA1_CH5;

        let i2c = I2c::new(
            p.I2C1,
            i2c_scl,
            i2c_sda,
            Irqs,
            i2c_tx_dma,
            i2d_rx_dma,
            i2c_cfg,
        );

        // Setting SPI.
        let mut spi_cfg = spi::Config::default();
        // SPI frequency (8 MHz).
        spi_cfg.frequency = Hertz(8_000_000);

        let spi_sck =  p.PA5;
        let spi_miso = p.PA6;
        let spi_mosi = p.PA7;
        let spi_ss = Output::new(p.PA4, Level::High, Speed::VeryHigh);
        let spi_tx_dma = p.DMA2_CH3;
        let spi_rx_dma = p.DMA2_CH2;

        let spi = Spi::new(
            p.SPI1,
            spi_sck,
            spi_mosi,
            spi_miso,
            spi_tx_dma,
            spi_rx_dma,
            spi_cfg,
        );

        Self { builtin_led, status_led, i2c, spi, spi_ss }
    }
}

// Binding I2C interrupts to handlers.
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
