use std::{error::Error, fs::read, path::Path};

use self::{
    info::{CartridgeInfo, CartridgeInfoError},
    mapper::Mapper,
};

pub mod info;
mod mapper;

pub struct Cartridge {
    mapper: Box<dyn Mapper>,
    pub info: CartridgeInfo,
}

pub enum CartridgeError {
    File(Box<dyn Error>),
    CartridgeInfo(CartridgeInfoError),
    GameGenie,
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeError::File(e) => write!(f, "File Error: {}!", e),
            CartridgeError::CartridgeInfo(e) => write!(f, "Cartridge Error: {}", e),
            CartridgeError::GameGenie => {
                write!(f, "Game Genie ROM does not look like a game genie!")
            }
        }
    }
}

impl Cartridge {
    fn load_cart(file: &Path) -> Result<(Vec<u8>, CartridgeInfo), CartridgeError> {
        let bytes = match read(file) {
            Ok(b) => b,
            Err(e) => return Err(CartridgeError::File(Box::new(e))),
        };
        let info = match CartridgeInfo::new_from_cartridge(&bytes) {
            Ok(i) => i,
            Err(e) => return Err(CartridgeError::CartridgeInfo(e)),
        };
        Ok((bytes, info))
    }

    pub fn new_from_file(file: &Path) -> Result<Self, CartridgeError> {
        let (bytes, info) = Cartridge::load_cart(file)?;
        let mapper = mapper::new(bytes, &info);
        Ok(Self { mapper, info })
    }

    pub fn new_from_file_genie(file: &Path, genie: &Path) -> Result<Self, CartridgeError> {
        let (cart_bytes, cart_info) = Cartridge::load_cart(file)?;
        let cart_mapper = mapper::new(cart_bytes, &cart_info);

        let (genie_bytes, genie_info) = Cartridge::load_cart(genie)?;
        // Not sure if game genie has a unique way to identify itself...
        let genie_mapper = mapper::new_genie(genie_bytes, &genie_info, cart_mapper);
        Ok(Self {
            mapper: genie_mapper,
            info: cart_info,
        })
    }

    pub fn reset(&mut self) {
        self.mapper.reset()
    }

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
