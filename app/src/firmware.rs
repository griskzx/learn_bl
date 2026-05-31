#[repr(C)]
pub struct FirmwareHeader {
    pub magic: u32,
    pub version: u32,
    pub size: u32,
    pub crc: u32,
}

#[unsafe(link_section = ".fw_header")]
#[unsafe(no_mangle)]
#[used]
pub static FW_HEADER: FirmwareHeader = FirmwareHeader {
    magic: 0xDEADBEEF,
    version: 0x0001_0000,
    size: 0xFFFFFFFF,
    crc: 0xFFFFFFFF,
};
