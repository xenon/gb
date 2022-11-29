use crate::cart::info::CartridgeInfo;

use super::{Mapper, RamLoadError, RamSaveError, RAM_BANK_SIZE, ROM_BANK_SIZE};

const RTC_S: u8 = 0x08;
const RTC_M: u8 = 0x09;
const RTC_H: u8 = 0x0A;
const RTC_DL: u8 = 0x0B;
const RTC_DH: u8 = 0x0C;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RamMode {
    None,
    Bank(usize),
    Seconds,
    Minutes,
    Hours,
    DayLow,
    DayHigh,
}

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    has_ram: bool,
    has_battery: bool,
    has_timer: bool,
    ram_rtc_enable: bool,
    rom_bank: usize,
    rom_offset: usize,
    ram_offset: usize,
    mode: RamMode,
    latch_read_0: bool,
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>, info: &CartridgeInfo) -> Self {
        Self {
            rom,
            ram: vec![0x00; info.ram_size],
            has_ram: info.ram,
            has_battery: info.battery,
            has_timer: info.time,
            ram_rtc_enable: false,
            rom_bank: 1,
            rom_offset: 0x4000,
            ram_offset: 0,
            mode: RamMode::None,
            latch_read_0: false,
        }
    }

    fn recalculate_offsets(&mut self) {
        let rom_banks = match self.rom.len() / ROM_BANK_SIZE {
            0 => 1,
            n => n,
        };
        self.rom_offset = (self.rom_bank % rom_banks) * ROM_BANK_SIZE;
        //eprintln!("rom_banks: {}, offset: {:#06x}", rom_banks, self.rom_offset);
        if let RamMode::Bank(ram_bank) = self.mode {
            let ram_banks = match self.ram.len() / RAM_BANK_SIZE {
                0 => 1,
                n => n,
            };
            self.ram_offset = (ram_bank % ram_banks) * RAM_BANK_SIZE;
            //eprintln!("ram_banks: {}, offset: {:#06x}", ram_banks, self.ram_offset);
        } else {
            self.ram_offset = 0;
        }
    }
}

impl Mapper for Mbc3 {
    fn reset(&mut self) {
        if !self.has_battery {
            self.reset_save();
        }
        self.ram_rtc_enable = false;
        self.rom_bank = 1;
        self.mode = RamMode::None;
        self.latch_read_0 = false;
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
            0x4000..=0x7FFF => self.rom[(address as usize - 0x4000) + self.rom_offset],
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                if self.has_ram || self.has_timer {
                    self.ram_rtc_enable = value == 0x0A;
                }
            }
            0x2000..=0x3FFF => {
                self.rom_bank = match value & 0b01111111 {
                    0 => 1,
                    b => b as usize,
                };
                self.recalculate_offsets();
            }
            0x4000..=0x5FFF => {
                self.mode = match value {
                    0x00..=0x03 if self.has_ram => RamMode::Bank(value as usize),
                    RTC_S if self.has_battery => RamMode::Seconds,
                    RTC_M if self.has_battery => RamMode::Minutes,
                    RTC_H if self.has_battery => RamMode::Hours,
                    RTC_DL if self.has_battery => RamMode::DayLow,
                    RTC_DH if self.has_battery => RamMode::DayHigh,
                    _ => RamMode::None,
                };
                self.recalculate_offsets();
            }
            0x6000..=0x7FFF => {
                if value == 0x00 && !self.latch_read_0 {
                    self.latch_read_0 = true;
                } else if self.latch_read_0 {
                    if value == 0x01 {
                        // update clock data
                        eprintln!("Clock data latched");
                    }
                    self.latch_read_0 = false;
                }
            }
            _ => unreachable!(),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_rtc_enable {
            match self.mode {
                RamMode::None => 0xFF,
                RamMode::Bank(_) => self.ram[(address - 0xA000) as usize + self.ram_offset],
                RamMode::Seconds => 0x00,
                RamMode::Minutes => 0x00,
                RamMode::Hours => 0x00,
                RamMode::DayLow => 0x00,
                RamMode::DayHigh => 0x00,
            }
        } else {
            0xFF
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_rtc_enable {
            match self.mode {
                RamMode::Bank(_) => self.ram[(address - 0xA000) as usize + self.ram_offset] = value,
                RamMode::Seconds => (),
                RamMode::Minutes => (),
                RamMode::Hours => (),
                RamMode::DayLow => (),
                RamMode::DayHigh => (),
                _ => (),
            }
        }
    }
}
