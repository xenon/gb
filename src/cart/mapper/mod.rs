use self::{mbc1::Mbc1, rom::Rom};

use super::info::CartridgeInfo;

mod mbc1;
mod rom;

pub enum MapperType {
    Rom,
    Mbc1,
    Mbc2,
    Mmm01,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    Huc3,
    Huc1,
}

pub trait Mapper: Send {
    fn reset(&mut self);
    fn rom_b(&self, address: u16) -> u8;
    fn rom_wb(&mut self, address: u16, value: u8);
    fn ram_b(&self, address: u16) -> u8;
    fn ram_wb(&mut self, address: u16, value: u8);

    fn calculate_header_checksum(&self) -> u8 {
        let mut checksum: u8 = 0;
        for index in 0x0134..0x014D {
            checksum = checksum.wrapping_sub(self.rom_b(index)).wrapping_sub(1);
        }
        checksum
    }
}

pub fn new(bytes: Vec<u8>, info: &CartridgeInfo) -> Box<dyn Mapper> {
    match info.mapper {
        MapperType::Rom => Box::new(Rom::new(bytes)),
        MapperType::Mbc1 => Box::new(Mbc1::new(bytes, info)),
        MapperType::Mbc2 => todo!(),
        MapperType::Mmm01 => todo!(),
        MapperType::Mbc3 => todo!(),
        MapperType::Mbc5 => todo!(),
        MapperType::Mbc6 => todo!(),
        MapperType::Mbc7 => todo!(),
        MapperType::Huc3 => todo!(),
        MapperType::Huc1 => todo!(),
    }
}
