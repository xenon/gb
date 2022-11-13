use num_enum::UnsafeFromPrimitive;

use crate::cart::info::CartridgeInfo;

use super::Mapper;

#[derive(Clone, Copy, Debug, Eq, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum BankingMode {
    Simple = 0b0,
    Advanced = 0b1,
}
pub struct Mbc1 {
    rom: Vec<u8>,
    rom_bank: usize,
    ram: Vec<u8>,
    ram_bank: usize,
    ram_enable: bool,
    ram_mode: BankingMode,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, info: &CartridgeInfo) -> Mbc1 {
        Mbc1 {
            rom,
            rom_bank: 1,
            ram: vec![0x00; info.ram_size],
            ram_bank: 0,
            ram_enable: false,
            ram_mode: BankingMode::Simple,
        }
    }
}

impl Mapper for Mbc1 {
    fn reset(&mut self) {}
    fn rom_b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[(address as usize - 0x4000) + (self.rom_bank * 0x4000)],
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enable = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let bank = match value & 0b11111 {
                    0 => 0b1,
                    b => b as usize,
                };
                self.rom_bank = (self.rom_bank & 0b01100000) | bank;
            }
            0x4000..=0x5FFF => {
                if self.ram_mode == BankingMode::Simple {
                    self.rom_bank = (self.rom_bank & 0b00011111) | (((value & 0b11) << 5) as usize);
                } else {
                    self.ram_bank = (value & 0b11) as usize;
                }
            }
            0x6000..=0x7FFF => {
                self.ram_mode = unsafe { BankingMode::from_unchecked(value & 0b1) };
            }
            _ => unreachable!(),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_enable {
            let bank = if self.ram_mode == BankingMode::Advanced {
                self.ram_bank
            } else {
                0
            };
            self.ram[(address as usize - 0xA000) + (bank * 0x2000)]
        } else {
            0xFF
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_enable {
            let bank = if self.ram_mode == BankingMode::Advanced {
                self.ram_bank
            } else {
                0
            };
            self.ram[(address as usize - 0xA000) + (bank * 0x2000)] = value;
        }
    }
}
