use std::{error::Error, fs::read, path::Path};

use self::info::CartridgeInfo;

pub mod info;

// Eventually I'll need mapper information to go here...
pub struct Cartridge {
    rom: Vec<u8>,
    pub info: CartridgeInfo,
}

impl Cartridge {
    pub fn new_from_file(file: &Path) -> Result<Self, Box<dyn Error>> {
        let bytes = read(file)?;
        let info = CartridgeInfo::new_from_cartridge(&bytes)?;
        Ok(Self { rom: bytes, info })
    }

    pub fn reset(&mut self) {}

    pub fn header_checksum(&self) -> bool {
        let mut checksum: u8 = 0;
        for index in 0x0134..0x014D {
            checksum = checksum.wrapping_sub(self.rom[index]).wrapping_sub(1);
        }
        checksum == self.info.header_checksum
    }

    pub fn global_checksum(&self) -> bool {
        todo!()
    }

    pub fn rom_b(&self, address: u16) -> u8 {
        self.rom[address as usize] // trivial rom for ROM only carts
    }

    pub fn rom_wb(&self, address: u16, value: u8) {}

    pub fn ram_b(&self, address: u16) -> u8 {
        0 // trivial rom for ROM only carts
    }

    pub fn ram_wb(&self, address: u16, value: u8) {}
}
