// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2026-present ahrs-debug-rig project and contributors.

//! Hardware abstraction layer.

mod led;

use crate::{
    error,
    types::{self, Error},
};
use core::str::FromStr;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Runner, Stack, StackResources, StaticConfigV4};
use enumset::enum_set;
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
    wifi::{
        AccessPointConfig, AuthMethod, Config, ModeConfig, WifiController,
        WifiDevice, WifiEvent,
    },
};
pub use led::StatusLed;
use smoltcp::wire::Ipv4Cidr;
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
    /// - Initialize IMU handler system peripherals - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - Task spawn error.
    /// - DMA buffer initialization error.
    /// - Wi-Fi initialization errors.
    /// - Network errors.
    pub async fn new(p: Peripherals, spawner: &Spawner) -> types::Result<Self> {
        Self::init_system();
        let host = Self::init_host_interface(p).await?;

        let (stack, controller) = Self::init_network_stack(spawner).await?;
        spawner
            .spawn(connection_task(controller))
            .map_err(|_| Error::Other)?;

        Ok(Self {
            host,
            net_stack: stack,
        })
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
    /// - Initialized host interface peripherals - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - DMA buffer initialization error.
    async fn init_host_interface(
        p: Peripherals,
    ) -> types::Result<HostInterface> {
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

        let dma_rx_buf =
            DmaRxBuf::new(rx_descriptors, rx_buffer).map_err(|_| {
                println!("Error to set DMA buffer");
                Error::Other
            })?;

        Ok(HostInterface {
            status_led,
            esp_ready,
            spi,
            dma_rx_buf,
        })
    }

    /// Initialize network stack.
    ///
    /// # Parameters
    /// - `spawner` - given task spawner to handle.
    ///
    /// # Returns
    /// - Network stack handler & Wi-Fi controller - in case of success.
    /// - `Err` - otherwise.
    ///
    /// # Errors
    /// - Wi-Fi initialization errors.
    /// - Network errors.
    async fn init_network_stack(
        spawner: &Spawner,
    ) -> types::Result<(Stack<'static>, WifiController<'static>)> {
        static STACK: StaticCell<Stack<'static>> = StaticCell::new();
        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        static RADIO_INIT: StaticCell<Controller<'static>> = StaticCell::new();

        if let Ok(radio) = esp_radio::init() {
            let radio_init = RADIO_INIT.init(radio);
            let wifi = unsafe { Peripherals::steal().WIFI };

            let (controller, interfaces) =
                esp_radio::wifi::new(radio_init, wifi, Config::default())?;

            let ip_addr = Ipv4Address::from_str(crate::IMU_GATEWAY_IP)
                .unwrap_or(Ipv4Address::UNSPECIFIED);

            let ipv4_config = StaticConfigV4 {
                address: Ipv4Cidr::new(ip_addr, 24),
                gateway: None,
                dns_servers: Default::default(),
            };

            let (stack, runner) = embassy_net::new(
                interfaces.ap,
                embassy_net::Config::ipv4_static(ipv4_config),
                RESOURCES.init(StackResources::new()),
                42,
            );

            let stack = STACK.init(stack);
            if spawner.spawn(net_task(runner)).is_err() {
                error(
                    "Error to spawn a network runner task",
                    Error::NetworkError,
                )
                .await;
            }

            Ok((*stack, controller))
        } else {
            println!("Failed to initialize Wi-Fi/BLE controller");
            Err(Error::NetworkError)
        }
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
    let ap_config = AccessPointConfig::default()
        .with_ssid(crate::IMU_GATEWAY_SSID.into())
        .with_password(crate::IMU_GATEWAY_PASSWORD.into())
        .with_auth_method(AuthMethod::Wpa2Personal)
        .with_channel(6);

    if let Err(e) = controller.set_config(&ModeConfig::AccessPoint(ap_config)) {
        error("Error to set Wi-Fi config", e.into()).await;
    }

    if let Err(e) = controller.start() {
        error("Error to start Wi-Fi", e.into()).await;
    }

    println!(
        "AP started: SSID='{}', IP={}",
        crate::IMU_GATEWAY_SSID,
        crate::IMU_GATEWAY_IP
    );

    let interested_events = enum_set!(
        WifiEvent::ApStaConnected
            | WifiEvent::ApStaDisconnected
            | WifiEvent::ApStart
            | WifiEvent::ApStop
    );

    loop {
        let occurred =
            controller.wait_for_events(interested_events, true).await;

        if occurred.contains(WifiEvent::ApStaConnected) {
            println!("Client connected to AP");
        }

        if occurred.contains(WifiEvent::ApStaDisconnected) {
            println!("Client disconnected from AP");
        }

        if occurred.contains(WifiEvent::ApStart) {
            println!("AP started");
        }

        if occurred.contains(WifiEvent::ApStop) {
            println!("AP stopped");
        }

        if !occurred.is_empty() {
            println!("Event occurred: {:?}", occurred);
        }
    }
}
