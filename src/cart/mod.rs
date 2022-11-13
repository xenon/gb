use std::{error::Error, fs::read, path::Path};

use self::{
    info::{CartridgeInfo, CartridgeInfoError},
    mapper::Mapper,
};

pub mod info;
mod mapper;

// Eventually I'll need mapper information to go here...
pub struct Cartridge {
    mapper: Box<dyn Mapper>,
    pub info: CartridgeInfo,
}

pub enum CartridgeError {
    FileError(Box<dyn Error>),
    CartridgeInfoError(CartridgeInfoError),
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeError::FileError(e) => write!(f, "File Error: {}!", e),
            CartridgeError::CartridgeInfoError(e) => write!(f, "Cartridge Error: {}", e),
        }
    }
}

impl Cartridge {
    pub fn new_from_file(file: &Path) -> Result<Self, CartridgeError> {
        let bytes = match read(file) {
            Ok(b) => b,
            Err(e) => return Err(CartridgeError::FileError(Box::new(e))),
        };
        let info = match CartridgeInfo::new_from_cartridge(&bytes) {
            Ok(i) => i,
            Err(e) => return Err(CartridgeError::CartridgeInfoError(e)),
        };
        let mapper = mapper::new(bytes, &info);
        Ok(Self { mapper, info })
    }

    pub fn reset(&mut self) {}

    pub fn header_checksum(&self) -> bool {
        self.mapper.calculate_header_checksum() == self.info.header_checksum
    }

    pub fn global_checksum(&self) -> bool {
        todo!()
    }

    pub fn rom_b(&self, address: u16) -> u8 {
        self.mapper.rom_b(address)
    }

    pub fn rom_wb(&mut self, address: u16, value: u8) {
        self.mapper.rom_wb(address, value)
    }

    pub fn ram_b(&self, address: u16) -> u8 {
        self.mapper.ram_b(address)
    }

    pub fn ram_wb(&mut self, address: u16, value: u8) {
        self.mapper.ram_wb(address, value)
    }
}
