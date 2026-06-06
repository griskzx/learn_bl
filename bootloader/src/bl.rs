use stm32f4xx_hal::hal_02::digital::v2::InputPin;
// OutputPin Trait 来自 embedded-hal，stm32f4xx_hal 已经通过 hal_02 内部重新导出了它
// 用 use stm32f4xx_hal::hal_02::digital::v2::OutputPin 即可引入
use stm32f4xx_hal::hal_02::digital::v2::OutputPin;
use stm32f4xx_hal::hal_02::serial::Read;
use stm32f4xx_hal::nb;
use stm32f4xx_hal::{
    serial::Rx2,
    pac::Peripherals
};
use crate::bl::OTA_ERROR::ERROR1;
use crate::bl::OTA_ERROR::ERROR2;
use crate::queue::Queue;
use crate::{FirmwareHeader, APP_CODE_ADDR, APP_HEADER_ADDR, MAGIC_NUMBER};
use crate::firmware::CRC_ALGO;
// ✅ 正确写法：用泛型参数 P 并约束它必须实现 OutputPin Trait
// 这样任何 GPIO 输出引脚（如 pb14）都可以作为参数传入，而不是写死为某个具体的引脚类型
pub fn check_and_jump_app<P: OutputPin>(led_red: &mut P) -> ! {
    // ===================== 第一步：读取固件头 =====================
    let header_ptr = APP_HEADER_ADDR as *const FirmwareHeader;
    let header = unsafe { core::ptr::read_volatile(header_ptr) };

    // ===================== 第二步：验证魔数 =====================
    if header.magic != MAGIC_NUMBER {
        defmt::error!(
            "Magic mismatch! expected=0x{:08x}, got=0x{:08x}",
            MAGIC_NUMBER,
            header.magic
        );
        // OutputPin::set_high() 返回 Result，用 ok() 忽略错误
        let _ = led_red.set_high();
        loop {}
    }
    defmt::info!(
        "Magic OK. version=0x{:08x}, size={} bytes",
        header.version,
        header.size
    );

    // ===================== 第三步：计算并校验 CRC32 =====================
    let app_code_slice: &[u8] =
        unsafe { core::slice::from_raw_parts(APP_CODE_ADDR as *const u8, header.size as usize) };

    let computed_crc = CRC_ALGO.checksum(app_code_slice);

    defmt::info!(
        "CRC check: computed=0x{:08x}, expected=0x{:08x}",
        computed_crc,
        header.crc
    );

    if computed_crc != header.crc {
        defmt::error!("CRC MISMATCH! Firmware corrupted, refusing to boot.");
        let _ = led_red.set_high();
        loop {}
    }

    // ===================== 校验通过，跳转！ =====================
    defmt::info!("Firmware OK! Jumping to 0x{:08x}", APP_CODE_ADDR);

    unsafe {
        jump_to_app(APP_CODE_ADDR);
    }
    // jump_to_app 内部用 options(noreturn) 的汇编永远不会返回
    // 但 Rust 类型系统看不到这一点，所以加一个 loop {} 来满足 -> ! 的返回类型
    loop {}
}

pub unsafe fn jump_to_app(app_addr: u32) {
    unsafe {
        // 建议注释掉全局中断关闭，因为这会设置 PRIMASK，导致 app 默认无法响应任何中断
        // cortex_m::interrupt::disable();
        // 获取对应外设对应的safe为take
        let mut cp = cortex_m::peripheral::Peripherals::steal();
        //停SysTick
        cp.SYST.disable_counter();
        cp.SYST.disable_interrupt();

        //清理向量表 NVIC
        const NVIC_ICER: *mut u32 = 0xE000_E180 as *mut u32;
        const NVIC_ICPR: *mut u32 = 0xE000_E280 as *mut u32;
        for i in 0..8 {
            core::ptr::write_volatile(NVIC_ICER.add(i), 0xFFFF_FFFF);
            core::ptr::write_volatile(NVIC_ICPR.add(i), 0xFFFF_FFFF);
        }

        //复位外设
        let dp = stm32f4xx_hal::pac::Peripherals::steal();
        let rcc = &dp.RCC;

        // ============ 解决时钟树 (RCC) 冲突：复位 RCC 到出厂状态 ============
        // 1. 开启内部高速时钟 HSI
        rcc.cr().modify(|_, w| w.hsion().set_bit());
        while rcc.cr().read().hsirdy().bit_is_clear() {}

        // 2. 恢复 CFGR 寄存器到默认值，这会将系统主时钟切回 HSI，并清除所有分频
        rcc.cfgr().write(|w| w.bits(0x0000_0000));
        while rcc.cfgr().read().sws().bits() != 0 {}

        // 3. 关闭 HSE, CSS, PLL
        rcc.cr().modify(|_, w| {
            w.hseon()
                .clear_bit()
                .csson()
                .clear_bit()
                .pllon()
                .clear_bit()
        });

        // 4. 恢复 PLLCFGR 和 CIR (中断) 寄存器
        rcc.pllcfgr().write(|w| w.bits(0x2400_3010));
        rcc.cir().write(|w| w.bits(0x0000_0000));
        // ==============================================================

        rcc.ahb1rstr().write(|w| w.bits(0xFFFF_FFFF));
        rcc.ahb1rstr().write(|w| w.bits(0));

        rcc.ahb2rstr().write(|w| w.bits(0xFFFF_FFFF));
        rcc.ahb2rstr().write(|w| w.bits(0));

        rcc.apb1rstr().write(|w| w.bits(0xFFFF_FFFF));
        rcc.apb1rstr().write(|w| w.bits(0));

        rcc.apb2rstr().write(|w| w.bits(0xFFFF_FFFF));
        rcc.apb2rstr().write(|w| w.bits(0));

        //切换向量表
        cp.SCB.vtor.write(app_addr);
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        let app_vertor = app_addr as *const u32;
        // let sp = *app_vertor;
        // let rv = *(app_vertor.add(1));

        let sp = core::ptr::read_volatile(app_vertor);
        let rv = core::ptr::read_volatile(app_vertor.add(1));

        core::arch::asm!(
            "msr msp, {0}",
            "bx {1}",
            in(reg) sp,
            in(reg) rv,
            options(noreturn)
        );
    }
}

pub enum OTA_ERROR {
    //Temporary placeholder
    ERROR1,
    ERROR2,
}

pub fn ota_recive<P:InputPin>(
    button:&mut P,
    rx:&mut Rx2,
    rx_queue:&mut Queue<u8,128>
)->Result<(),OTA_ERROR>{
    if button.is_low().unwrap_or(false){
        return Err(ERROR1);
    }
    let mut count =0 ;
    let mut has_started = false;
    loop {
        match rx.read() {
            Ok(byte) =>{
                has_started =true;
                rx_queue.force_enqueue(byte);
            }
            Err(nb::Error::WouldBlock) =>{}
            Err(_)=>{
                return Err(ERROR2);
            }
        }
        if rx_queue.is_full(){
            use crate::config::COS_ADDR;
            //强制缓冲区大小为128
            on_frame_full(rx_queue,COS_ADDR+(count*124));
            count +=1;
            rx_queue.clear();
        }
        if has_started {
            let usart = unsafe { &*stm32f4xx_hal::pac::USART2::ptr() };
            if usart.sr().read().idle().bit_is_set() {
                let _dummy = usart.dr().read().bits();
                break;
            }
        }
    }
    Ok(())
}
pub fn on_frame_full(queue:&mut Queue<u8,128>,addr:u32){
    //由于是一个满队列进入此函数所以正常情况下队列里面不可能为空
    let mut data = [0u32;124/4];
    for i in 0..124/4 {
        let mut values =[0u8;4];
        for j in 0..4 {
            values[j] = queue.dequeue().unwrap();
        }
        data[i] = u32::from_le_bytes(values);
    }
    let mut crc_bytes =[0u8;4];
    for i in 0..4 {
        crc_bytes[i] = queue.dequeue().unwrap();
    }

    //将4字节还原为u32
    let expected_crc = u32::from_le_bytes(crc_bytes);
    //计算校验和
    let computed_crc = crate::crc::hw_crc32(&data);

    if computed_crc == expected_crc {
        unsafe {
            let _=crate::flash::flash_write_words(addr,&data);
        }
    }
    // unimplemented!()
}
