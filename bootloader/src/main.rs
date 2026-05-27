// #![deny(unsafe_code)]
#![no_main]
#![no_std]

// Print panic message to probe console
use defmt_rtt as _;
use learn_bootloader::bl::jump_to_app;
use panic_probe as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::gpio::GpioExt;

// 高速查找表，手动塞 CCMRAM
//   #[link_section = ".ccmram_fast"]
//   #[no_mangle]
//   static mut LUT_SINE: [f32; 256] = [0.0; 256];

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let mut rcc = learn_bootloader::config::init_clocks(dp.RCC);
    let _clock = rcc.clocks;
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led_green = gpiob.pb0.into_push_pull_output();
    let button = gpioc.pc13.into_floating_input();
    loop {
        led_green.set_high();
        if button.is_high() {
            unsafe {
                // 已修改：指向 App 的真实起始地址
                jump_to_app(0x0801_0000);
            }
        }
    }
}
