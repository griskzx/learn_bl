use stm32f4xx_hal::{
    pac::RCC,
    prelude::*,
    rcc::{Config, Rcc},
};

//临时占位，地址后续必须修改
pub const COS_ADDR:u32 =0x0808_0000; 

/// Configure the system clocks using the High-Speed External (HSE) oscillator.
///
/// This configures:
/// - HSE oscillator bypass (8 MHz)
/// - SYSCLK: 168 MHz
/// - HCLK: 168 MHz
/// - PCLK1 (APB1): 42 MHz
/// - PCLK2 (APB2): 84 MHz
pub fn init_clocks(rcc: RCC) -> Rcc {
    let config = Config::hse(8.MHz())
        .bypass_hse_oscillator()
        .sysclk(168.MHz())
        .hclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz());
    rcc.freeze(config)
}
