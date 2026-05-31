use crc::{Crc, CRC_32_ISO_HDLC};

#[repr(C)]
pub struct FirmwareHeader {
    pub magic: u32,
    pub version: u32,
    pub size: u32,
    pub crc: u32,
}

pub const CRC_ALGO: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// App 固件头在 Flash 中的起始地址
pub const APP_HEADER_ADDR: u32 = 0x0801_0000;
pub const APP_CODE_ADDR: u32 = 0x0801_0100; // App 真实向量表起点（跳过 256 字节 Header）
pub const MAGIC_NUMBER: u32 = 0xDEADBEEF;