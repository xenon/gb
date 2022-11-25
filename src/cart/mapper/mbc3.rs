use crate::cart::info::CartridgeInfo;

use super::{Mapper, RAM_BANK_SIZE, ROM_BANK_SIZE};

const RTC_S: u8 = 0x08;
const RTC_M: u8 = 0x09;
const RTC_H: u8 = 0x0A;
const RTC_DL: u8 = 0x0B;
const RTC_DH: u8 = 0x0C;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RamMode {
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
    ram_rtc_enable: bool,
    rom_bank: usize,
    mode: RamMode,
    latch_read_0: bool,
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>, info: &CartridgeInfo) -> Self {
        Self {
            rom,
            ram: vec![0x00; info.ram_size],
            ram_rtc_enable: false,
            rom_bank: 1,
            mode: RamMode::Bank(0),
            latch_read_0: false,
        }
    }
}

impl Mapper for Mbc3 {
    fn reset(&mut self) {
        self.ram.iter_mut().for_each(|v| *v = 0x00);
        self.ram_rtc_enable = false;
        self.rom_bank = 1;
        self.mode = RamMode::Bank(0);
        self.latch_read_0 = false;
    }
    fn rom_b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                let index = (address as usize - 0x4000) + (self.rom_bank * ROM_BANK_SIZE);
                self.rom[index]
            }
            _ => unreachable!(),
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        let value = value & 0x0F;
        match address {
            0x0000..=0x1FFF => {
                self.ram_rtc_enable = value == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = match value & 0b1111111 {
                    0 => 1,
                    b => b as usize,
                };
            }
            0x4000..=0x5FFF => {
                self.mode = match value {
                    0x00..=0x03 => RamMode::Bank(value as usize),
                    RTC_S => RamMode::Seconds,
                    RTC_M => RamMode::Minutes,
                    RTC_H => RamMode::Hours,
                    RTC_DL => RamMode::DayLow,
                    RTC_DH => RamMode::DayHigh,
                    _ => unreachable!(),
                }
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
            _ => (),
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.ram_rtc_enable {
            match self.mode {
                RamMode::Bank(n) => self.ram[(address - 0xA000) as usize + (n * RAM_BANK_SIZE)],
                RamMode::Seconds => 0x00,
                RamMode::Minutes => 0x00,
                RamMode::Hours => 0x00,
                RamMode::DayLow => 0x00,
                RamMode::DayHigh => 0x00,
            }
        } else {
            0x0F
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.ram_rtc_enable {
            match self.mode {
                RamMode::Bank(n) => {
                    self.ram[(address - 0xA000) as usize + (n * RAM_BANK_SIZE)] = value
                }
                RamMode::Seconds => (),
                RamMode::Minutes => (),
                RamMode::Hours => (),
                RamMode::DayLow => (),
                RamMode::DayHigh => (),
            }
        }
    }
}
