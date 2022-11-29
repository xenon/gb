use crate::cart::info::CartridgeInfo;

use super::{Mapper, RamLoadError, RamSaveError, RAM_BANK_SIZE, ROM_BANK_SIZE};

pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    has_ram: bool,
    has_battery: bool,
    has_rumble: bool,
    ram_enable: bool,
    rom_bank: u16,
    rom_offset: usize,
    ram_bank: u8,
    ram_offset: usize,
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>, info: &CartridgeInfo) -> Self {
        // Check if RAM size is invalid and if so set to default
        let ram_size = if info.ram {
            match info.ram_size {
                0 => 32768,
                n => n,
            }
        } else {
            0
        };

        Self {
            rom,
            ram: vec![0x00; ram_size],
            has_ram: info.ram,
            has_battery: info.battery,
            has_rumble: info.rumble,
            ram_enable: false,
            rom_bank: 1,
            rom_offset: 0x4000,
            ram_bank: 0,
            ram_offset: 0,
        }
    }

    fn recalculate_offsets(&mut self) {
        let rom_banks = match self.rom.len() / ROM_BANK_SIZE {
            0 => 1,
            n => n,
        };
        let ram_banks = match self.ram.len() / RAM_BANK_SIZE {
            0 => 1,
            n => n,
        };
        self.rom_offset = (self.rom_bank as usize % rom_banks) * ROM_BANK_SIZE;
        self.ram_offset = (self.ram_bank as usize % ram_banks) * RAM_BANK_SIZE;
    }
}

impl Mapper for Mbc5 {
    fn reset(&mut self) {
        if !self.has_battery {
            self.reset_save();
        }
        self.ram_enable = false;
        self.rom_bank = 1;
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
            0x0000..=0x1FFF if self.has_ram => self.ram_enable = (value & 0x0F) == 0x0A,
            0x0000..=0x1FFF => {} // cart without ram attempted to write ram
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x100) | (value as u16);
                self.recalculate_offsets();
            }
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0xFF) | (((value as u16) & 0b1) << 8);
                self.recalculate_offsets();
            }
            0x4000..=0x5FFF => {
                if value <= 0x0F {
                    self.ram_bank = value;
                    self.recalculate_offsets();
                }
            }
            _ => (),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_enable {
            self.ram[(address as usize - 0xA000) % RAM_BANK_SIZE + self.ram_offset]
        } else {
            0xFF
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_enable {
            self.ram[(address as usize - 0xA000) % RAM_BANK_SIZE + self.ram_offset] = value;
        }
    }
}
