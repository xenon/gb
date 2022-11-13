use super::Mapper;

pub struct Rom {
    rom: Vec<u8>,
}

impl Rom {
    pub fn new(rom: Vec<u8>) -> Self {
        Rom { rom }
    }
}

#[allow(unused_variables)]
impl Mapper for Rom {
    fn reset(&mut self) {}
    fn rom_b(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn rom_wb(&mut self, address: u16, value: u8) {}
    fn ram_b(&self, address: u16) -> u8 {
        0x00
    }
    fn ram_wb(&mut self, address: u16, value: u8) {}
}
