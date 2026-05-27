pub unsafe fn jump_to_app(app_addr: u32) {
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
    rcc.cfgr().write(|w| unsafe { w.bits(0x0000_0000) });
    while rcc.cfgr().read().sws().bits() != 0 {}

    // 3. 关闭 HSE, CSS, PLL
    rcc.cr().modify(|_, w| {
        w.hseon().clear_bit()
         .csson().clear_bit()
         .pllon().clear_bit()
    });

    // 4. 恢复 PLLCFGR 和 CIR (中断) 寄存器
    rcc.pllcfgr().write(|w| unsafe { w.bits(0x2400_3010) });
    rcc.cir().write(|w| unsafe { w.bits(0x0000_0000) });
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

// pub fn start_up_app(){

// }
