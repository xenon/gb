use super::{Mapper, RamLoadError, RamSaveError};

pub struct Rom {
    rom: Vec<u8>,
}

impl Rom {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

#[allow(unused_variables)]
impl Mapper for Rom {
    fn reset(&mut self) {}

    fn save_size(&self) -> Option<usize> {
        None
    }
    fn load_save(&mut self, bytes: Vec<u8>) -> Result<(), RamLoadError> {
        Err(RamLoadError::Incompatible)
    }
    fn save_save(&mut self, bytes: Vec<u8>) -> Result<(), RamSaveError> {
        Err(RamSaveError::Incompatible)
    }
    fn reset_save(&mut self) {}

    fn rom_b(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn rom_wb(&mut self, address: u16, value: u8) {}
    fn ram_b(&self, address: u16) -> u8 {
        0x00
    }
    fn ram_wb(&mut self, address: u16, value: u8) {}
}
