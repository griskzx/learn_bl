#![no_main]
#![no_std]

use bootloader as _; // global logger + panicking-behavior + memory layout
use cortex_m::Peripherals as CrxPeripherals;
use defmt::Format; // <- derive attribute
use stm32f4xx_hal::{
    otg_fs::{USB, UsbBus},
    pac::Peripherals as Stm32Peripherals,
    prelude::*,
    rcc, serial,
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
    // 1.配置外设
    let usb = USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (gpioa.pa11, gpioa.pa12),
        &clocks,
    );
    // 2.构建端点总线 UsbBus
    let usb_bus = UsbBus::new(usb, unsafe { &mut *core::ptr::addr_of_mut!(EP_MEMORY) });
    let mut serial = usbd_serial::SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(usbd_serial::USB_CLASS_CDC)
        .strings(&[StringDescriptors::default()
            .manufacturer("My Company")
            .product("Serial Port")
            .serial_number("001")])
        .unwrap()
        .build();
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
    loop {
        // defmt::info!("test frimware...");
        // led_toggle();
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }
        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                for c in buf[0..count].iter_mut() {
                    if *c >= b'a' && *c <= b'z' {
                        *c -= 32;
                    }
                }
                serial.write(&buf[0..count]).ok();
            }
            _ => {}
        }
    }
}
