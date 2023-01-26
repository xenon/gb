use super::{Mapper, RamLoadError, RamSaveError};

pub struct NullMapper;

impl NullMapper {
    pub fn new() -> Self {
        Self
    }
}

#[allow(unused_variables)]
impl Mapper for NullMapper {
    fn reset(&mut self) {}

    fn save_size(&self) -> Option<usize> {
        None
    }
    fn load_save(&mut self, bytes: Vec<u8>) -> Result<(), RamLoadError> {
        Err(RamLoadError::Incompatible)
    }
    fn save_save(&mut self) -> Result<Vec<u8>, RamSaveError> {
        Err(RamSaveError::Incompatible)
    }
    fn reset_save(&mut self) {}

    fn rom_b(&self, address: u16) -> u8 {
        0xFF
    }

    fn rom_wb(&mut self, address: u16, value: u8) {}

    fn ram_b(&self, address: u16) -> u8 {
        0xFF
    }

    fn ram_wb(&mut self, address: u16, value: u8) {}
}
