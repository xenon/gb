use std::time::Duration;

use num_enum::IntoPrimitive;

use crate::cpu;

pub const ONE_FRAME_CYCLES: u32 = 70224;
pub const ONE_FRAME_DURATION: Duration =
    Duration::from_nanos((1_000_000_000_f64 / (cpu::HZ as f64 / ONE_FRAME_CYCLES as f64)) as u64);

// Screen information
pub const LCD_WIDTH: u32 = 160;
pub const LCD_HEIGHT: u32 = 144;

// Addresses
pub const LCDC: u16 = 0xFF40;
pub const STAT: u16 = 0xFF41;
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;
pub const LY: u16 = 0xFF44;
pub const LYC: u16 = 0xFF45;
pub const DMA: u16 = 0xFF46;
pub const BGP: u16 = 0xFF47;
pub const OBP0: u16 = 0xFF48;
pub const OBP1: u16 = 0xFF49;
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B;

const PPU_BANK_SIZE: usize = 0x2000;

pub struct Ppu {
    m_ram: [u8; PPU_BANK_SIZE], // CGB: 2 x PPU_BANK_SIZE for
    // CGB: bank: u8,
    m_lcdc: u8,
    m_stat: u8,
    m_scy: u8,
    m_scx: u8,
    m_ly: u8,
    m_lyc: u8,
    m_dma: u8,
    m_bgp: u8,
    m_obp0: u8,
    m_obp1: u8,
    m_wy: u8,
    m_wx: u8,
}

#[derive(Copy, Clone, Eq, IntoPrimitive, PartialEq)]
#[repr(u8)]
enum LcdcFlag {
    BgWindowEnable = 0b00000001,
    ObjEnable = 0b00000010,
    ObjSize = 0b00000100,
    BgTileArea = 0b00001000,
    BgWindowTileDataArea = 0b00010000,
    WindowEnable = 0b00100000,
    WindowTileArea = 0b01000000,
    LCDEnable = 0b10000000,
}

#[derive(Copy, Clone, Eq, IntoPrimitive, PartialEq)]
#[repr(u8)]
enum StatFlag {
    ModeFlag = 0b00000011,
    LycEqLy = 0b00000100,
    Mode0HblankInt = 0b00001000,
    Mode1VBlankInt = 0b00010000,
    Mode2OAMInt = 0b00100000,
    LycEqLyInt = 0b01000000,
}

impl Ppu {
    pub fn new() -> Self {
        let mut p = Self {
            m_ram: [0; PPU_BANK_SIZE],
            m_lcdc: 0x91,
            m_stat: 0x81, // CGB: ?
            m_scy: 0x00,
            m_scx: 0x00,
            m_ly: 0x91, // CGB: ?
            m_lyc: 0x00,
            m_dma: 0xFF, // CGB: 00
            m_bgp: 0xFC,
            m_obp0: 0x00, // UNCONFIRMED
            m_obp1: 0x00, // UNCONFIRMED
            m_wy: 0x00,
            m_wx: 0x00,
        };
        p.reset();
        p
    }

    pub fn reset(&mut self) {
        self.m_ram = [0; PPU_BANK_SIZE];
        self.m_lcdc = 0x91;
        self.m_stat = 0x81; // CGB: ?
        self.m_scy = 0x00;
        self.m_scx = 0x00;
        self.m_ly = 0x91; // CGB: ?
        self.m_lyc = 0x00;
        self.m_dma = 0xFF; // CGB: 00
        self.m_bgp = 0xFC;
        self.m_obp0 = 0x00; // UNCONFIRMED
        self.m_obp1 = 0x00; // UNCONFIRMED
        self.m_wy = 0x00;
        self.m_wx = 0x00;
    }

    fn set_lcdc_flag(&mut self, flag: LcdcFlag, is_1: bool) {
        if is_1 {
            self.m_lcdc |= flag as u8;
        } else {
            self.m_lcdc &= !(flag as u8);
        }
    }

    fn set_stat_flag(&mut self, flag: StatFlag, is_1: bool) {
        if is_1 {
            self.m_stat |= flag as u8;
        } else {
            self.m_stat &= !(flag as u8);
        }
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.m_ram[address as usize - 0x8000],
            LCDC => self.m_lcdc,
            STAT => self.m_stat,
            SCY => self.m_scy,
            SCX => self.m_scx,
            LY => self.m_ly,
            LYC => self.m_lyc, // TODO: update STAT
            DMA => self.m_dma,
            BGP => self.m_bgp,
            OBP0 => self.m_obp0,
            OBP1 => self.m_obp1,
            WY => self.m_wy,
            WX => self.m_wx,
            _ => unreachable!(),
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.m_ram[address as usize - 0x8000] = value,
            LCDC => {
                // set all bits of lcdc
                self.m_lcdc = value;
            }
            STAT => {
                // set all bits of stat
                self.m_stat = value & 0b01111000;
            }
            SCY => self.m_scy = value,
            SCX => self.m_scx = value,
            LY => (),
            LYC => {
                self.m_lyc = value;
                self.set_stat_flag(StatFlag::LycEqLy, self.m_lyc == self.m_ly);
            }
            DMA => self.m_dma = value,
            BGP => self.m_bgp = value,
            OBP0 => self.m_obp0 = value,
            OBP1 => self.m_obp1 = value,
            WY => self.m_wy = value,
            WX => self.m_wx = value,
            _ => (), // TODO: fallback??
        }
    }
}
