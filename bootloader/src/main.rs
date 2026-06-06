// #![deny(unsafe_code)]
#![no_main]
#![no_std]

use defmt_rtt as _;
use learn_bootloader::{bl::ota_recive, prelude::*, queue::Queue};
use panic_probe as _;

// 用于支持格式化打印
use core::fmt::Write as FmtWrite;

use cortex_m_rt::entry;
use stm32f4xx_hal::{nb, prelude::*, serial::Serial};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut rcc = learn_bootloader::config::init_clocks(dp.RCC);
    let _clock = rcc.clocks;
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpiod = dp.GPIOD.split(&mut rcc);

    let mut delay = cp.SYST.delay(&_clock);
    let mut timeout_ms = 3000;
    let poll_interval_ms = 10;

    // 串口2引脚复用配置
    let rx_pin = gpioa.pa3.into_alternate();
    let tx_pin = gpiod.pd5.into_alternate();
    let serial_config = stm32f4xx_hal::serial::config::Config::default().baudrate(115200.bps());
    let serial = Serial::new(dp.USART2, (tx_pin, rx_pin), serial_config, &mut rcc).unwrap();
    let (mut tx, mut rx) = serial.split();

    // 红灯：校验失败时亮（pb14）
    let mut led_red = gpiob.pb14.into_push_pull_output();
    led_red.set_low();

    // 获取按键引脚
    let mut button = learn_bootloader::pac::get_button(dp.GPIOC, &mut rcc);

    writeln!(tx, "\r\n---------------------------------------------").unwrap();
    writeln!(tx, "Bootloader Start! Waiting 3s for upgrade (0x5A)...").unwrap();
    defmt::info!("Bootloader started. Waiting 3s for upgrade command (0x5A)...");

    let mut upgrade_mode = false;
    let mut rx_queue: Queue<u8, 128> = Queue::new();
    loop {
        let _ = ota_recive(&mut button, &mut rx, &mut rx_queue);
    }
}
