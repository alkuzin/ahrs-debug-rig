// SPDX-License-Identifier: Apache-2.0.
// Copyright (C) 2025-present ahrs-debug-rig project and contributors.

#![no_std]
#![no_main]

use stm32f4xx_hal::{pac, prelude::*, rcc::Config};
use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc_cfg = Config::hsi().sysclk(24.MHz());
    let mut rcc = dp.RCC.freeze(rcc_cfg);

    let mut timer = dp.TIM2.counter_ms(&mut rcc);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    let mut led_builtin  = gpioc.pc13.into_push_pull_output();
    let mut led_status_r = gpioa.pa9.into_push_pull_output();
    let mut led_status_g = gpioa.pa10.into_push_pull_output();
    let mut led_status_b = gpioa.pa11.into_push_pull_output();

    timer.start(200.millis()).unwrap();

    for _ in 0..8 {
        led_builtin.toggle();
        nb::block!(timer.wait()).unwrap();
    }

    timer.start(500.millis()).unwrap();

    loop {
        led_status_r.set_high();
        led_status_g.set_low();
        led_status_b.set_low();
        nb::block!(timer.wait()).unwrap();

        led_status_r.set_low();
        led_status_g.set_high();
        led_status_b.set_low();
        nb::block!(timer.wait()).unwrap();

        led_status_r.set_low();
        led_status_g.set_low();
        led_status_b.set_high();
        nb::block!(timer.wait()).unwrap();

        led_status_r.set_high();
        led_status_g.set_high();
        led_status_b.set_high();
        nb::block!(timer.wait()).unwrap();

        led_status_r.set_low();
        led_status_g.set_low();
        led_status_b.set_low();
        nb::block!(timer.wait()).unwrap();
    }
}
