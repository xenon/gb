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
    ram: Vec<u8>,
    ram_enable: bool,
    bank: usize,
    mode: BankingMode,
    cached_rom_bank: usize,
    cached_ram_bank: usize,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, info: &CartridgeInfo) -> Mbc1 {
        // Check if RAM size is invalid and if so set to default
        let ram_size = match info.ram_size {
            0 => 32768,
            n => n,
        };
        //let rom_size = info.rom_size * 8;
        //rom.resize_with(rom_size, || 0x00);
        Mbc1 {
            rom,
            ram: vec![0x00; ram_size],
            ram_enable: false,
            bank: 1,
            mode: BankingMode::Simple,
            cached_rom_bank: 0,
            cached_ram_bank: 0,
        }
    }

    fn recalculate_banks(&mut self) {
        let (rom_bank, ram_bank) = match self.mode {
            BankingMode::Simple => (self.bank & 0b1111111, 0),
            BankingMode::Advanced => (self.bank & 0b0011111, (self.bank & 0b1100000) >> 5),
        };
        self.cached_rom_bank = rom_bank;
        self.cached_ram_bank = ram_bank;
    }
}

impl Mapper for Mbc1 {
    fn reset(&mut self) {}
    fn rom_b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let index = (address as usize - 0x4000) + (self.cached_rom_bank * 0x4000);
                self.rom[index]
            }
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enable = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                let bank = match value & 0b11111 {
                    0 => 1,
                    b => b as usize,
                };
                self.bank = (self.bank & 0b01100000) | bank;
                self.recalculate_banks();
            }
            0x4000..=0x5FFF => {
                self.bank = (self.bank & 0b10011111) | (((value & 0b11) << 5) as usize);
                self.recalculate_banks();
            }
            0x6000..=0x7FFF => {
                self.mode = unsafe { BankingMode::from_unchecked(value & 0b1) };
                self.recalculate_banks();
            }
            _ => unreachable!(),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_enable {
            let index = (address as usize - 0xA000) + (self.cached_ram_bank * 0x2000);
            self.ram[index]
        } else {
            0xFF
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_enable {
            let index = (address as usize - 0xA000) + (self.cached_ram_bank * 0x2000);
            self.ram[index] = value;
        }
    }
}
