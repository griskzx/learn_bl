use embedded_storage::nor_flash::NorFlash;
use stm32f4xx_hal::{
    flash::UnlockedFlash,
    otg_fs::{USB, UsbBus},
};
use usb_device::{UsbError, prelude::*};
use usbd_serial::SerialPort;

type MyBus = UsbBus<USB>;
type MyUsbDeV<'a> = UsbDevice<'a, MyBus>;
type MySerial<'a> = SerialPort<'a, MyBus>;

pub const APP_ADDR_BEGIN: u32 = 0x0801_0000;
pub const APP_ADDR_END: u32 = 0x0802_0000;

/// 接收固件并写入 Flash。
/// 注意：调用前须已完成 Flash 擦除（擦除耗时数百毫秒会导致 USB 断连）。
/// 以 b"helloworld" 作为结束标志，收到后返回 Ok(())。
pub fn firmware_reicve<'a>(
    flash: &mut UnlockedFlash<'_>,
    usb_dev: &mut MyUsbDeV<'a>,
    serial: &mut MySerial<'a>,
    buf: &mut [u8],
) -> Result<(), UsbError> {
    let mut addr = APP_ADDR_BEGIN;
    // Erase 已在调用方完成，此处直接进入接收循环
    loop {
        if !usb_dev.poll(&mut [serial]) {
            continue;
        }
        match serial.read( buf) {
            Ok(count) if count > 0 => {
                NorFlash::write(flash, addr, &buf[0..count]).unwrap();
                if &buf[0..count] == b"helloworld" {
                    return Ok(());
                }
                serial.write(&buf[0..count]).ok();
                addr += count as u32;
            }
            _ => {}
        }
        // flash_write(flash, addr, &buf[0..count]).unwrap();
        
        
    }
}

// pub fn flash_write(
//     flash: &mut UnlockedFlash<'_>,
//     address: u32,
//     data: &[u8],
// ) -> Result<(), FlashError> {
//     let end_addr = address + data.len() as u32;
//     NorFlash::erase(flash, address, end_addr)?;
//     NorFlash::write(flash, address, data)?;
//     Ok(())
// }
// pub fn flash_earse(
//     flash: &mut UnlockedFlash<'_>,
//     address: u32,
//     len: u32,
// ) -> Result<(), FlashError> {
//     let end_addr = address + len;
//     NorFlash::erase(flash, address, end_addr)?;
//     Ok(())
// }
