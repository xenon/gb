use std::{error::Error, fs::read, path::Path};

use self::info::CartridgeInfo;

pub mod info;

pub struct Cartridge {
    bytes: Vec<u8>,
    pub info: CartridgeInfo,
}

impl Cartridge {
    pub fn new_from_file(file: &Path) -> Result<Self, Box<dyn Error>> {
        let bytes = read(file)?;
        let info = CartridgeInfo::new_from_cartridge(&bytes)?;
        Ok(Self { bytes, info })
    }

    pub fn header_checksum(&self) -> bool {
        let mut checksum: u8 = 0;
        for index in 0x0134..0x014D {
            checksum = checksum.wrapping_sub(self.bytes[index]).wrapping_sub(1);
        }
        checksum == self.info.header_checksum
    }

    pub fn global_checksum(&self) -> bool {
        todo!()
    }
}
