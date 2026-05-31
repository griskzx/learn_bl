// #![deny(unsafe_code)]
#![no_main]
#![no_std]

use defmt_rtt as _;
use learn_bootloader::prelude::*;
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
    let serial_config = stm32f4xx_hal::serial::config::Config::default()
        .baudrate(115200.bps());
    let serial = Serial::new(dp.USART2, (tx_pin, rx_pin), serial_config, &mut rcc).unwrap();
    let (mut tx, mut rx) = serial.split();

    // 红灯：校验失败时亮（pb14）
    let mut led_red = gpiob.pb14.into_push_pull_output();
    led_red.set_low();

    writeln!(tx, "\r\n---------------------------------------------").unwrap();
    writeln!(tx, "Bootloader Start! Waiting 3s for upgrade (0x5A)...").unwrap();
    defmt::info!("Bootloader started. Waiting 3s for upgrade command (0x5A)...");

    let mut upgrade_mode = false;

    loop {
        // 非阻塞轮询读取串口
        match rx.read() {
            Ok(0x5A) => {
                writeln!(tx, "\r\nUpgrade command (0x5A) received!").unwrap();
                defmt::info!("Upgrade command (0x5A) received! Entering upgrade mode.");
                upgrade_mode = true;
                break;
            }
            Ok(other) => {
                // 如果收到其他非指令字符，可以回显出来
                let _ = nb::block!(tx.write(other));
            }
            Err(nb::Error::WouldBlock) => {
                // 暂时没收到数据，这是最常见的情况，什么都不做
            }
            Err(nb::Error::Other(_err)) => {
                // 忽略硬件校验或帧格式错误，防止卡死
            }
        }

        delay.delay_ms(poll_interval_ms as u32);
        if timeout_ms > 0 {
            timeout_ms -= poll_interval_ms;
        } else {
            writeln!(tx, "\r\nTimeout! Booting App...").unwrap();
            defmt::info!("3s timeout reached. Checking normal firmware...");
            break;
        }
    }

    // ===================== 升级模式分支 =====================
    if upgrade_mode {
        writeln!(tx, "Now in upgrade mode. (Flash erase/write logic goes here)").unwrap();
        // 进入一个指示灯快速闪烁的死循环，后续我们在这里写 Flash 传输写入逻辑
        loop {
            led_red.toggle();
            delay.delay_ms(200_u32);
        }
    }

    // ===================== 正常启动分支 =====================
    
    // 1. 读取固件头
    let header_ptr = APP_HEADER_ADDR as *const FirmwareHeader;
    let header = unsafe { core::ptr::read_volatile(header_ptr) };

    // 2. 验证魔数
    if header.magic != MAGIC_NUMBER {
        writeln!(tx, "Error: Magic mismatch! Expected 0xDEADBEEF").unwrap();
        defmt::error!(
            "Magic mismatch! expected=0x{:08x}, got=0x{:08x}",
            MAGIC_NUMBER,
            header.magic
        );
        led_red.set_high();
        loop {}
    }

    // 3. 计算并校验 CRC32
    let app_code_slice: &[u8] = unsafe {
        core::slice::from_raw_parts(APP_CODE_ADDR as *const u8, header.size as usize)
    };
    let computed_crc = CRC_ALGO.checksum(app_code_slice);

    defmt::info!(
        "CRC check: computed=0x{:08x}, expected=0x{:08x}",
        computed_crc,
        header.crc
    );

    if computed_crc != header.crc {
        writeln!(tx, "Error: CRC mismatch! Firmware corrupted.").unwrap();
        defmt::error!("CRC MISMATCH! Firmware corrupted, refusing to boot.");
        led_red.set_high();
        loop {}
    }

    // 4. 校验通过，开始跳转！
    writeln!(tx, "Firmware verified. Jumping to App...").unwrap();
    defmt::info!("Firmware OK! Jumping to 0x{:08x}", APP_CODE_ADDR);
    delay.delay_ms(10_u32); // 给串口留一点发送完最后几个字符的时间

    unsafe {
        jump_to_app(APP_CODE_ADDR);
    }

    loop {}
}
