#[repr(C)]
pub struct FirmwareHeader {
    pub magic: u32,
    pub version: u32,
    pub size: u32,
    pub crc: u32,
}
