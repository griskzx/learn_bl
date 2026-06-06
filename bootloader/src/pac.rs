use stm32f4xx_hal::{hal_02::digital::v2::InputPin, prelude::*, rcc::Rcc};

pub fn get_button(
    gpioc: stm32f4xx_hal::pac::GPIOC,
    rcc: &mut Rcc,
) -> impl InputPin<Error = core::convert::Infallible> {
    let gpioc = gpioc.split(rcc);
    gpioc.pc13.into_floating_input()
}
