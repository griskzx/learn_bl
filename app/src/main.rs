#![no_main]
#![no_std]

use app as _; // global logger + panicking-behavior + memory layout
use stm32f4xx_hal::prelude::*;

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let config = stm32f4xx_hal::rcc::Config::hse(8.MHz())
        .bypass_hse_oscillator()
        .sysclk(168.MHz())
        .hclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz());
    let mut rcc = dp.RCC.freeze(config);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut led = gpiob.pb0.into_push_pull_output();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut delay = cp.SYST.delay(&rcc.clocks);
    loop {
        led.set_high();
        delay.delay_ms(500_u32);
        led.set_low();
        delay.delay_ms(500_u32);
    }
}
