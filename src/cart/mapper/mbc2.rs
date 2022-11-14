use num_enum::UnsafeFromPrimitive;

use crate::cart::info::CartridgeInfo;

use super::{Mapper, RAM_BANK_SIZE, ROM_BANK_SIZE};

pub struct Mbc2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enable: bool,
    bank: usize,
}

impl Mbc2 {
    pub fn new(rom: Vec<u8>, info: &CartridgeInfo) -> Self {
        // Check if RAM size is invalid and if so set to default
        let ram_size = match info.ram_size {
            0 => 512,
            n => n,
        };
        Self {
            rom,
            ram: vec![0x00; ram_size],
            ram_enable: false,
            bank: 1,
        }
    }
}

impl Mapper for Mbc2 {
    fn reset(&mut self) {
        self.ram.iter_mut().for_each(|v| *v = 0x00);
        self.ram_enable = false;
        self.bank = 1;
    }
    fn rom_b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let index = (address as usize - 0x4000) + (self.bank * ROM_BANK_SIZE);
                self.rom[index]
            }
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        let value = value & 0x0F;
        match address {
            0x0000..=0x1FFF => {
                self.ram_enable = value == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.bank = match value {
                    0 => 1,
                    b => b as usize,
                } & 0b11111;
            }
            _ => (),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_enable {
            match address {
                0xA000..=0xA1FF => self.ram[(address - 0xA000) as usize] & 0x0F,
                0xA200..=0xBFFF => self.ram[(address % 0x200) as usize] & 0x0F,
                _ => unreachable!(),
            }
        } else {
            0x0F
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_enable {
            match address {
                0xA000..=0xA1FF => self.ram[(address - 0xA000) as usize] = value & 0x0F,
                0xA200..=0xBFFF => self.ram[(address % 0x200) as usize] = value & 0x0F,
                _ => unreachable!(),
            }
        }
    }
}
