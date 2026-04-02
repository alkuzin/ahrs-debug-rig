// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Hardware abstraction layer.

mod led;

use alloc::string::ToString;
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::Timer;
use esp_hal::{
    Blocking,
    dma::DmaRxBuf,
    dma_buffers,
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    spi::{
        Mode,
        slave::{Spi, dma::SpiDma},
    },
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_radio::{
    Controller,
    wifi::{ClientConfig, Config, ModeConfig, WifiController, WifiDevice},
};
pub use led::StatusLed;
use static_cell::StaticCell;

/// Alias for SPI driver.
pub type SpiDriver<'a> = SpiDma<'a, Blocking>;

/// SPI DMA buffer size in bytes.
pub const DMA_BUFFER_SIZE: usize = 56;

/// Host interface peripherals.
pub struct HostInterface {
    /// Status LEDs handler.
    pub status_led: StatusLed<'static>,
    /// ESP ready pin.
    pub esp_ready: Output<'static>,
    /// SPI handler.
    pub spi: SpiDriver<'static>,
    /// SPI DMA buffer for incoming data.
    pub dma_rx_buf: DmaRxBuf,
}

/// IMU handler system peripherals.
pub struct SystemPeripherals {
    /// Host interface peripherals.
    pub host: HostInterface,
    /// Network stack.
    pub net_stack: Stack<'static>,
}

impl SystemPeripherals {
    /// Construct & initialize IMU handler system peripherals.
    ///
    /// # Parameters
    /// - `p` - given STM32 peripherals to handle.
    ///
    /// # Returns
    /// - Initialize IMU handler system peripherals.
    pub async fn new(p: Peripherals, spawner: &Spawner) -> Self {
        Self::init_system();
        let host = Self::init_host_interface(p).await;

        let (stack, controller) = Self::init_network_stack(spawner);
        spawner.spawn(connection_task(controller)).unwrap();

        Self {
            host,
            net_stack: stack,
        }
    }

    /// Initialize the system.
    #[inline]
    fn init_system() {
        esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
        let timg0 = unsafe { TimerGroup::new(Peripherals::steal().TIMG0) };
        esp_rtos::start(timg0.timer0);
    }

    /// Initialize host interface peripherals.
    ///
    /// # Parameters
    /// - `p` - given MCU peripherals to handle.
    ///
    /// # Returns
    /// - Initialized host interface peripherals.
    async fn init_host_interface(p: Peripherals) -> HostInterface {
        let config = OutputConfig::default();

        let status_led_red = Output::new(p.GPIO4, Level::High, config);
        let status_led_green = Output::new(p.GPIO16, Level::High, config);
        let status_led = StatusLed::new(status_led_red, status_led_green);
        let esp_ready = Output::new(p.GPIO2, Level::Low, config);

        // Setting SPI + DMA.
        let dma_channel = p.DMA_SPI2;
        let spi_sck = p.GPIO18;
        let spi_miso = p.GPIO19;
        let spi_mosi = p.GPIO23;
        let spi_ss = p.GPIO5;

        let (rx_buffer, rx_descriptors, _, _) = dma_buffers!(DMA_BUFFER_SIZE);

        let spi = Spi::new(p.SPI2, Mode::_1)
            .with_sck(spi_sck)
            .with_miso(spi_miso)
            .with_mosi(spi_mosi)
            .with_cs(spi_ss)
            .with_dma(dma_channel);

        let dma_rx_buf = match DmaRxBuf::new(rx_descriptors, rx_buffer) {
            Ok(buf) => buf,
            Err(e) => panic!("Error creating DMA buffer: {:?}", e),
        };

        HostInterface {
            status_led,
            esp_ready,
            spi,
            dma_rx_buf,
        }
    }

    /// Initialize network stack.
    ///
    /// # Parameters
    /// - `spawner` - given task spawner to handle.
    ///
    /// # Returns
    /// - Network stack handler & Wi-Fi controller.
    fn init_network_stack(
        spawner: &Spawner,
    ) -> (Stack<'static>, WifiController<'static>) {
        static STACK: StaticCell<Stack<'static>> = StaticCell::new();
        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        static RADIO_INIT: StaticCell<Controller<'static>> = StaticCell::new();

        let radio = esp_radio::init()
            .expect("Failed to initialize Wi-Fi/BLE controller");
        let radio_init = RADIO_INIT.init(radio);

        let wifi = unsafe { Peripherals::steal().WIFI };

        let (controller, interfaces) =
            esp_radio::wifi::new(radio_init, wifi, Config::default())
                .expect("Failed to initialize Wi-Fi controller");

        let (stack, runner) = embassy_net::new(
            interfaces.sta,
            embassy_net::Config::dhcpv4(Default::default()),
            RESOURCES.init(StackResources::new()),
            42,
        );

        let stack = STACK.init(stack);
        spawner.spawn(net_task(runner)).unwrap();

        (*stack, controller)
    }
}

/// Process network events.
///
/// # Parameters
/// - `runner` - given network stack runner to handle.
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

/// Handle network connections.
///
/// # Parameters
/// - `controller` - given Wi-Fi controller to handle.
#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    let sta_config = ClientConfig::default()
        .with_ssid(crate::WIFI_SSID.to_string())
        .with_password(crate::WIFI_PASSWORD.to_string());

    controller
        .set_config(&ModeConfig::Client(sta_config))
        .unwrap();
    controller.start().expect("WiFi start failed");

    loop {
        if let Err(e) = controller.connect() {
            println!("WiFi connection error: {:?}", e);
        }

        Timer::after_secs(3).await;
    }
}
