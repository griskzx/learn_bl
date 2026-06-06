#![no_std]

pub mod bl;
pub mod config;
pub mod crc;
pub mod firmware;

mod flash;
pub mod pac;
pub mod queue;
pub use firmware::{APP_CODE_ADDR, APP_HEADER_ADDR, FirmwareHeader, MAGIC_NUMBER};

pub mod prelude {
    pub use crate::bl::jump_to_app;
    pub use crate::firmware::{
        APP_CODE_ADDR, APP_HEADER_ADDR, CRC_ALGO, FirmwareHeader, MAGIC_NUMBER,
    };
}
