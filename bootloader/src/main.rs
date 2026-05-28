// #![deny(unsafe_code)]
#![no_main]
#![no_std]

// Print panic message to probe console
use defmt_rtt as _;
use learn_bootloader::bl::jump_to_app;
use panic_probe as _;

use cortex_m_rt::entry;
use learn_bootloader::FirmwareHeader;
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
    // let gpioc = dp.GPIOC.split(&mut rcc);
    let mut led_red = gpiob.pb14.into_push_pull_output();
    led_red.set_low();
    // let button = gpioc.pc13.into_floating_input();

    let header_ptr = 0x0801_0000 as *const FirmwareHeader;
    let header = unsafe { core::ptr::read_volatile(header_ptr) };

    if header.magic == 0xA9B3ED {
        unsafe {
            // 魔法数对上了！说明那里面确实刷入了一个有效的 App。
            // 如果你加上了 CRC 算法，就可以根据 header.size 读取后续的 Flash 计算并对比 header.crc32
            jump_to_app(0x8001_0000);
        }
    } else {
        // 魔法数不对，可能是空片，点亮红灯，死循环等待烧录
        led_red.set_high();
    }

    loop {
        // led_green.set_high();
        // if button.is_high() {
        //     unsafe {
        //         // 已修改：指向 App 的真实起始地址
        //         jump_to_app(0x0801_0000);
        //     }
        // }
    }
}
