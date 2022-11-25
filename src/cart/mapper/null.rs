use super::Mapper;

pub struct NullMapper;

impl NullMapper {
    pub fn new() -> Self {
        Self
    }
}

impl Mapper for NullMapper {
    fn reset(&mut self) {}

    fn rom_b(&self, address: u16) -> u8 {
        0xFF
    }

    fn rom_wb(&mut self, address: u16, value: u8) {}

    fn ram_b(&self, address: u16) -> u8 {
        0xFF
    }

    fn ram_wb(&mut self, address: u16, value: u8) {}
}
