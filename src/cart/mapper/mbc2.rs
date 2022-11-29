use crate::cart::info::CartridgeInfo;

use super::{Mapper, RamLoadError, RamSaveError, ROM_BANK_SIZE};

pub struct Mbc2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    has_battery: bool,
    ram_enable: bool,
    bank: usize,
    rom_offset: usize,
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
            has_battery: info.battery,
            ram_enable: false,
            bank: 1,
            rom_offset: 0,
        }
    }

    fn recalculate_offsets(&mut self) {
        let rom_banks = match self.rom.len() / ROM_BANK_SIZE {
            0 => 1,
            n => n,
        };
        self.rom_offset = (self.bank as usize % rom_banks) * ROM_BANK_SIZE;
    }
}

impl Mapper for Mbc2 {
    fn reset(&mut self) {
        if !self.has_battery {
            self.reset_save();
        }
        self.ram_enable = false;
        self.bank = 1;
        self.recalculate_offsets();
    }

    fn save_size(&self) -> Option<usize> {
        if self.has_battery {
            Some(self.ram.len())
        } else {
            None
        }
    }
    fn load_save(&mut self, bytes: Vec<u8>) -> Result<(), RamLoadError> {
        if self.has_battery {
            if bytes.len() == self.ram.len() {
                self.ram = bytes;
                Ok(())
            } else {
                Err(match bytes.len() < self.ram.len() {
                    true => RamLoadError::TooSmall,
                    false => RamLoadError::TooLarge,
                })
            }
        } else {
            Err(RamLoadError::Incompatible)
        }
    }
    fn save_save(&mut self, bytes: Vec<u8>) -> Result<(), RamSaveError> {
        Err(RamSaveError::Incompatible)
    }
    fn reset_save(&mut self) {
        self.ram.fill(0);
    }

    fn rom_b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[address as usize - 0x4000 + self.rom_offset],
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                // Register selection determined by address bit 8
                if address & 0b100000000 == 0 {
                    self.ram_enable = value & 0x0F == 0x0A;
                } else {
                    self.bank = match value & 0x0F {
                        0 => 1,
                        b => b as usize,
                    };
                    self.recalculate_offsets();
                }
            }
            _ => (),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_enable {
            match address {
                0xA000..=0xBFFF => self.ram[(address % 0x200) as usize] | 0xF0,
                _ => unreachable!(),
            }
        } else {
            0xFF
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_enable {
            match address {
                0xA000..=0xBFFF => self.ram[(address % 0x200) as usize] = value & 0x0F,
                _ => unreachable!(),
            }
        }
    }
}
