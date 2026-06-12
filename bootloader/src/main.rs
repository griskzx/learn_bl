#![no_main]
#![no_std]

use bootloader as _; // global logger + panicking-behavior + memory layout
use bootloader::bootloader::{APP_ADDR_BEGIN, APP_ADDR_END};
use cortex_m::Peripherals as CrxPeripherals;
use embedded_storage::nor_flash::NorFlash;
use stm32f4xx_hal::{
    flash::{FlashExt, LockedFlash},
    otg_fs::{USB, UsbBus},
    pac::Peripherals as Stm32Peripherals,
    prelude::*,
    rcc,
};
use usb_device::{device::StringDescriptors, prelude::*};

//端点缓冲区内存
static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[cortex_m_rt::entry]
fn main() -> ! {
    //获取srtm32外设
    let dp = Stm32Peripherals::take().unwrap();
    //获取cortex外设
    let cp = CrxPeripherals::take().unwrap();

    //配置时钟树
    let rcc = rcc::Config::hse(8.MHz())
        .bypass_hse_oscillator() // 告诉芯片时钟是有源旁路输入
        .sysclk(168.MHz())
        .hclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz())
        .require_pll48clk();
    let mut rcc = dp.RCC.freeze(rcc);

    let clocks = rcc.clocks;
    //分离引脚
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    //推晚输出
    let mut led_green = gpiob.pb0.into_push_pull_output();
    let mut led_bule = gpiob.pb7.into_push_pull_output();
    let mut led_red = gpiob.pb14.into_push_pull_output();

    led_green.set_low();
    led_bule.set_low();
    led_red.set_low();

    let button = gpioc.pc13.into_floating_input();

    //设置延迟
    let mut delay = cp.SYST.delay(&clocks);

    //init usb
    // 1.配置外设 绑定物理硬件
    let usb = USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &clocks,
    );
    // 2.构建端点总线 UsbBus
    // 总线管理者：将底层的硬件驱动和上层的应用协议连接起来
    let usb_bus = UsbBus::new(usb, unsafe { &mut *core::ptr::addr_of_mut!(EP_MEMORY) });
    //虚拟串口协议
    let mut serial = usbd_serial::SerialPort::new(&usb_bus);
    //设备描述符
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(usbd_serial::USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("My Company")
            .product("Serial Port")
            .serial_number("001")])
        .unwrap()
        .build();

    let mut locked_flash = LockedFlash::new(dp.FLASH);
    let mut unlocked_flash = locked_flash.unlocked();

    let mut buf = [0u8; 64];

    let mut led_toggle = || {
        led_green.set_high();
        delay.delay_ms(500);
        led_bule.set_high();
        delay.delay_ms(500);
        led_red.set_high();
        delay.delay_ms(500);
        led_green.set_low();
        led_bule.set_low();
        led_red.set_low();
        delay.delay_ms(500);
    };
    let mut erased = false; // 避免每次按键都重复擦除
    loop {
        // USB 必须在所有状态下持续 poll，否则主机会认为设备断连
        usb_dev.poll(&mut [&mut serial]);

        if button.is_high() {
            // 第一步：擦除 Flash（耗时长，但外层 loop 依然在 poll USB）
            if !erased {
                NorFlash::erase(&mut unlocked_flash, APP_ADDR_BEGIN, APP_ADDR_END).unwrap();
                erased = true;
            }
            // 第二步：接收固件并写入（每包写入仅微秒级，不影响 USB）
            if let Ok(_) = bootloader::bootloader::firmware_reicve(
                &mut unlocked_flash,
                &mut usb_dev,
                &mut serial,
                &mut buf,
            ) {
                erased = false; // 下次按键重新擦除
                led_toggle();   // 接收成功后再闪灯
            }
        }

        // }
        // defmt::info!("test frimware...");
        // led_toggle();
        // if !usb_dev.poll(&mut [&mut serial]) {
        //     continue;
        // }
        // match serial.read(&mut buf) {
        //     Ok(count) if count > 0 => {
        //         for c in buf[0..count].iter_mut() {
        //             if *c >= b'a' && *c <= b'z' {
        //                 *c -= 32;
        //             }
        //         }
        //         serial.write(&buf[0..count]).ok();
        //     }
        //     _ => {}
        // }
    }
}
