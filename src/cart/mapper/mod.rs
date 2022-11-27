use self::{game_genie::GameGenie, mbc1::Mbc1, mbc2::Mbc2, mbc3::Mbc3, null::NullMapper, rom::Rom};

use super::info::CartridgeInfo;

mod game_genie;
mod mbc1;
mod mbc2;
mod mbc3;
mod null;
mod rom;

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        MapperType::Rom => Box::new(Rom::new(bytes, info)),
        MapperType::Mbc1 => Box::new(Mbc1::new(bytes, info)),
        MapperType::Mbc2 => Box::new(Mbc2::new(bytes, info)),
        MapperType::Mbc3 => Box::new(Mbc3::new(bytes, info)),
        MapperType::Mmm01
        | MapperType::Mbc5
        | MapperType::Mbc6
        | MapperType::Mbc7
        | MapperType::Huc3
        | MapperType::Huc1 => {
            eprintln!("Mapper type unsupported: {:?}!", info.mapper);
            Box::new(NullMapper::new())
        }
    }
}

pub fn new_genie(
    genie_bytes: Vec<u8>,
    genie_info: &CartridgeInfo,
    mapper: Box<dyn Mapper>,
) -> Box<GameGenie> {
    Box::new(GameGenie::new(genie_bytes, genie_info, mapper))
}
