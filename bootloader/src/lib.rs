#![no_std]

pub mod bl;
pub mod config;
pub mod firmware;
pub mod queue;
pub mod crc;

pub use firmware::{FirmwareHeader,MAGIC_NUMBER,APP_CODE_ADDR,APP_HEADER_ADDR};

pub mod prelude {
    pub use crate::bl::jump_to_app;
    pub use crate::firmware::{
        FirmwareHeader, MAGIC_NUMBER, APP_CODE_ADDR, APP_HEADER_ADDR, CRC_ALGO,
    };
}
