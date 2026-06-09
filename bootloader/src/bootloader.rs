use embedded_storage::nor_flash::NorFlash;
use stm32f4xx_hal::{
    flash::{Error as FlashError, UnlockedFlash},
    otg_fs::{USB, UsbBus},
};
use usb_device::{UsbError, prelude::*};
use usbd_serial::SerialPort;

type MyBus = UsbBus<USB>;
type MyUsbDeV<'a> = UsbDevice<'a, MyBus>;
type MySerial<'a> = SerialPort<'a, MyBus>;

const APP_ADDR_BEGIN: u32 = 0x0801_0000;
const APP_ADDR_END: u32 = 0x0802_0000;
pub fn firmware_reicve<'a>(
    flash: &mut UnlockedFlash<'_>,
    usb_dev: &mut MyUsbDeV<'a>,
    serial: &mut MySerial<'a>,
    buf: &mut [u8],
) -> Result<(), UsbError> {
    let mut addr = APP_ADDR_BEGIN;
    NorFlash::erase(flash, APP_ADDR_BEGIN, APP_ADDR_END).unwrap();
    loop {
        if !usb_dev.poll(&mut [serial]) {
            continue;
            // return Err(UsbError::WouldBlock);
        }
        let count = serial.read(buf)?;
        if count == 0 {
            continue;
        }
        // flash_write(flash, addr, &buf[0..count]).unwrap();
        NorFlash::write(flash, addr, &buf[0..count]).unwrap();
        if &buf[0..count] == b"helloworld" {
            return Ok(());
        }
        addr += count as u32;
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
