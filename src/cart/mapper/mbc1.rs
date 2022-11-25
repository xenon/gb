use num_enum::UnsafeFromPrimitive;

use crate::cart::info::CartridgeInfo;

use super::{Mapper, RAM_BANK_SIZE, ROM_BANK_SIZE};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum BankingMode {
    Simple = 0b0,
    Advanced = 0b1,
}
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    has_ram: bool,
    has_battery: bool,
    ram_enable: bool,
    bank: u8,
    bank_mask: u8,
    mode: BankingMode,
    rom_lo_offset: usize,
    rom_hi_offset: usize,
    ram_offset: usize,
}

impl Mbc1 {
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

        let bank_mask = match info.rom_size / 1024 {
            32 => 0b1,
            64 => 0b11,
            128 => 0b111,
            256 => 0b1111,
            512 | 1024 | 2048 => 0b11111,
            _ => unreachable!(),
        };

        Self {
            rom,
            ram: vec![0x00; ram_size],
            has_ram: info.ram,
            has_battery: info.battery,
            ram_enable: false,
            bank: 1,
            bank_mask,
            mode: BankingMode::Simple,
            rom_lo_offset: 0,
            rom_hi_offset: 0x4000,
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
        let (rom_lo, ram) = match self.mode {
            BankingMode::Simple => (0, 0),
            BankingMode::Advanced => {
                let bank_hi = ((self.bank >> 5) & 0b11) as usize;
                ((bank_hi << 5) % rom_banks, bank_hi % ram_banks)
            }
        };

        self.rom_lo_offset = rom_lo * ROM_BANK_SIZE;
        self.rom_hi_offset = (self.bank as usize % rom_banks) * ROM_BANK_SIZE;
        self.ram_offset = ram * RAM_BANK_SIZE;
    }
}

impl Mapper for Mbc1 {
    fn reset(&mut self) {
        self.ram.iter_mut().for_each(|v| *v = 0x00);
        self.ram_enable = false;
        self.bank = 1;
        self.mode = BankingMode::Simple;
        self.recalculate_offsets();
    }
    fn rom_b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize + self.rom_lo_offset],
            0x4000..=0x7FFF => self.rom[address as usize - 0x4000 + self.rom_hi_offset],
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF if self.has_ram => self.ram_enable = (value & 0x0F) == 0x0A,
            0x0000..=0x1FFF => {} // cart without ram attempted to write ram
            0x2000..=0x3FFF => {
                let bank = match value & 0b11111 {
                    0 => 1,
                    b => b,
                };
                self.bank = (self.bank & 0b01100000) | (bank & self.bank_mask);
                self.recalculate_offsets();
            }
            0x4000..=0x5FFF => {
                self.bank = (self.bank & 0b10011111) | ((value & 0b11) << 5);
                self.recalculate_offsets();
            }
            0x6000..=0x7FFF => {
                self.mode = unsafe { BankingMode::from_unchecked(value & 0b1) };
                self.recalculate_offsets();
            }
            _ => unreachable!(),
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
