use std::time::Duration;

use num_enum::{IntoPrimitive, UnsafeFromPrimitive};

use crate::cpu;

pub const ONE_FRAME_CYCLES: u32 = 70224;
pub const ONE_FRAME_DURATION: Duration =
    Duration::from_nanos((1_000_000_000_f64 / (cpu::HZ as f64 / ONE_FRAME_CYCLES as f64)) as u64);

// Screen information
pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

// Addresses
pub const LCDC: u16 = 0xFF40;
pub const STAT: u16 = 0xFF41;
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;
pub const LY: u16 = 0xFF44;
pub const LYC: u16 = 0xFF45;
pub const BGP: u16 = 0xFF47;
pub const OBP0: u16 = 0xFF48;
pub const OBP1: u16 = 0xFF49;
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B;

pub const UNKNOWN_1: u16 = 0xFF4E; // UNKNOWN
pub const VBK: u16 = 0xFF4F; // CGB

pub const BCPS: u16 = 0xFF68; // CGB
pub const BCPD: u16 = 0xFF69; // CGB
pub const OCPS: u16 = 0xFF6A; // CGB
pub const OCPD: u16 = 0xFF6B; // CGB

pub const OAM: u16 = 0xFFE0;

const PPU_BANK_SIZE: usize = 0x2000;
const PPU_OAM_SIZE: usize = 0xA0;

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, IntoPrimitive, PartialEq)]
#[repr(u8)]
enum LcdcFlag {
    BgWindowEnable = 0b00000001,
    ObjEnable = 0b00000010,
    ObjSize = 0b00000100,
    BgTileMapArea = 0b00001000,
    BgWindowTileDataArea = 0b00010000,
    WindowEnable = 0b00100000,
    WindowTileArea = 0b01000000,
    LCDEnable = 0b10000000,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, IntoPrimitive, PartialEq)]
#[repr(u8)]
enum ObjAttribute {
    Palette = 0b00000111,       // CGB
    VRamBank = 0b00001000,      // CGB
    PaletteNumber = 0b00010000, // No-CGB
    XFlip = 0b00100000,
    YFlip = 0b01000000,
    BGandWindowOverObj = 0b10000000,
}

fn get_obj_attribute(data: u8, attribute: ObjAttribute) -> bool {
    data & (attribute as u8) != 0
}

#[derive(Copy, Clone, Eq, IntoPrimitive, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum Mode {
    HBlank = 0b00,
    VBlank = 0b01,
    InOAM = 0b10,
    TransferData = 0b11,
}

pub struct Ppu {
    m_ram: [u8; PPU_BANK_SIZE], // tile data, tile maps, CGB: 2 x PPU_BANK_SIZE for
    m_oam: [u8; PPU_OAM_SIZE],
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
    pub buf: [[u8; LCD_WIDTH]; LCD_HEIGHT],
    internal_cycles: u32,
    mode: Mode,
    palette_index: [u8; LCD_WIDTH],
    blank_frame: bool,
    enable_background: bool,
    enable_obj: bool,
}

impl Ppu {
    pub fn new() -> Self {
        let mut p = Self {
            m_ram: [0; PPU_BANK_SIZE],
            m_oam: [0; PPU_OAM_SIZE],
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
            buf: [[0x00; LCD_WIDTH]; LCD_HEIGHT],
            internal_cycles: 0,
            mode: Mode::VBlank,
            palette_index: [0x00; LCD_WIDTH],
            blank_frame: false,
            enable_background: true,
            enable_obj: true,
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
        self.buf = [[0x00; LCD_WIDTH]; LCD_HEIGHT];
        self.internal_cycles = 0;
        self.mode = Mode::VBlank;
        self.palette_index = [0x00; LCD_WIDTH];
        self.blank_frame = false;
    }

    fn switch_mode(&mut self, mode: Mode) -> bool {
        self.mode = mode;
        match mode {
            Mode::HBlank => self.get_stat_flag(StatFlag::Mode0HblankInt),
            Mode::VBlank => self.get_stat_flag(StatFlag::Mode1VBlankInt),
            Mode::InOAM => self.get_stat_flag(StatFlag::Mode2OAMInt),
            _ => false,
        }
    }

    pub fn step(&mut self, mut cycles: u32) -> (bool, bool) {
        let line_cycles: u32 = 456;
        let (mut intf_vblank, mut intf_lcdstat) = (false, false);
        if self.get_lcdc_flag(LcdcFlag::LCDEnable) {
            while cycles > 0 {
                if cycles >= 80 {
                    self.internal_cycles += 80;
                    cycles -= 80;
                } else {
                    self.internal_cycles += cycles;
                    cycles = 0;
                }
                if self.internal_cycles >= line_cycles {
                    self.m_ly = (self.m_ly + 1) % 154;
                    self.set_stat_flag(StatFlag::LycEqLy, self.m_lyc == self.m_ly);
                    self.internal_cycles -= line_cycles;
                    if self.m_ly == 144 {
                        intf_lcdstat |= self.switch_mode(Mode::VBlank);
                        intf_vblank = true;
                        self.blank_frame = false;
                    }
                } else if self.m_ly < 144 {
                    if self.internal_cycles <= 80 {
                        if self.mode != Mode::InOAM {
                            intf_lcdstat |= self.switch_mode(Mode::InOAM);
                        }
                    } else if self.internal_cycles <= 168 && self.mode != Mode::HBlank {
                        if self.mode != Mode::TransferData {
                            self.switch_mode(Mode::TransferData);
                        }
                    } else if self.mode != Mode::HBlank {
                        intf_lcdstat |= self.switch_mode(Mode::HBlank);
                        self.draw_line();
                    }
                }
            }
        }
        (intf_vblank, intf_lcdstat)
    }

    fn draw_line(&mut self) {
        self.buf[self.m_ly as usize] = [0x00; LCD_WIDTH];
        if !self.blank_frame {
            if self.enable_background {
                self.render_bg_line();
            }
            if self.enable_obj {
                self.render_obj_line();
            }
        }
    }

    fn render_bg_line(&mut self) {
        if !self.get_lcdc_flag(LcdcFlag::BgWindowEnable) {
            return;
        }

        let render_window = self.get_lcdc_flag(LcdcFlag::WindowEnable)
            && (self.m_wy <= self.m_ly || self.m_wy == 166);
        let window_x = self.m_wx.wrapping_sub(7);

        let full_y = if render_window {
            self.m_ly.wrapping_sub(self.m_wy)
        } else {
            self.m_scy.wrapping_add(self.m_ly) // y-position in 256x256 full screen
        };
        let tile_y = (full_y / 8) as u16;
        let tile_offset_y = (full_y % 8) as u16;

        let tiledata_unsigned = self.get_lcdc_flag(LcdcFlag::BgWindowTileDataArea);
        let tiledata_base = if tiledata_unsigned { 0x8000 } else { 0x8800 };

        for x in 0..LCD_WIDTH {
            let window_visible = render_window && (x as u8) >= window_x;
            let full_x = if window_visible {
                (x as u8) - window_x
            } else {
                (x as u8).wrapping_add(self.m_scx) // x-position in 256x256 full screen
            };
            let tile_x = (full_x / 8) as u16;
            let tile_offset_x = full_x % 8;

            let background_map_base: u16 = if window_visible {
                if self.get_lcdc_flag(LcdcFlag::WindowTileArea) {
                    0x9C00
                } else {
                    0x9800
                }
            } else if self.get_lcdc_flag(LcdcFlag::BgTileMapArea) {
                0x9C00
            } else {
                0x9800
            };

            let tilemap_addr = background_map_base + tile_x + (tile_y * 32);
            let tile_index = self.m_ram[tilemap_addr as usize - 0x8000];
            let tile_offset = if tiledata_unsigned {
                tile_index as i16 as u16
            } else {
                ((tile_index as i8) as i16 + 128) as u16
            } * 16;
            let tile_addr = tiledata_base + tile_offset;

            let tile_y_data = [
                self.m_ram[(tile_addr + tile_offset_y * 2) as usize - 0x8000],
                self.m_ram[(tile_addr + tile_offset_y * 2 + 1) as usize - 0x8000],
            ];

            let palette_index = ((tile_y_data[1] & (0x80 >> tile_offset_x) != 0) as u8) << 1
                | ((tile_y_data[0] & (0x80 >> tile_offset_x) != 0) as u8);
            self.palette_index[x] = palette_index;

            // Trivial mapping for now, but that's because my representation is exactly the same
            let color = (self.m_bgp >> (2 * palette_index)) & 0b11;
            self.buf[self.m_ly as usize][x] = color;
        }
    }

    fn render_obj_line(&mut self) {
        if !self.get_lcdc_flag(LcdcFlag::ObjEnable) {
            return;
        }

        if self.m_ly as usize >= LCD_HEIGHT {
            return;
        }
        let mut sprite_count = 0;
        let obj_height = if self.get_lcdc_flag(LcdcFlag::ObjSize) {
            16
        } else {
            8
        };
        for obj_num in 0..40 {
            let obj_index = obj_num * 4;
            let (y, x, tile_index, tile_attributes) = (
                self.m_oam[obj_index].wrapping_sub(16),
                self.m_oam[obj_index + 1].wrapping_sub(8),
                self.m_oam[obj_index + 2],
                self.m_oam[obj_index + 3],
            );
            let y_end = y.wrapping_add(obj_height);
            let x_end = x.wrapping_add(7);

            // Object does not intersect ly in Y
            if !(y..y_end).contains(&self.m_ly) {
                continue;
            }
            // Only render 10 sprites, does not check X coordinate

            if sprite_count < 10 {
                sprite_count += 1;
            } else {
                break;
            }

            // Is the object on screen?

            if !(0..LCD_WIDTH).contains(&(x as usize))
                && !(0..LCD_WIDTH).contains(&(x_end as usize))
            {
                continue;
            }

            let tile_offset_y = if get_obj_attribute(tile_attributes, ObjAttribute::YFlip) {
                (obj_height - 1 - (self.m_ly - y)) as u16
            } else {
                (self.m_ly - y) as u16
            };
            let tile_addr = 0x8000 + (tile_index as u16) * 16;
            let tile_y_data = [
                self.m_ram[(tile_addr + tile_offset_y * 2) as usize - 0x8000],
                self.m_ram[(tile_addr + tile_offset_y * 2 + 1) as usize - 0x8000],
            ];

            let palette = if get_obj_attribute(tile_attributes, ObjAttribute::PaletteNumber) {
                self.m_obp1
            } else {
                self.m_obp0
            };

            for rel_x in 0..8 {
                let x_pixel = x.wrapping_add(rel_x) as usize;
                if x_pixel >= LCD_WIDTH {
                    continue;
                }
                let tile_offset_x: u8 = if get_obj_attribute(tile_attributes, ObjAttribute::XFlip) {
                    7 - rel_x
                } else {
                    rel_x
                };

                if self.palette_index[x_pixel] != 0x00
                    && get_obj_attribute(tile_attributes, ObjAttribute::BGandWindowOverObj)
                {
                    continue;
                }

                let palette_index = ((tile_y_data[1] & (0x80 >> tile_offset_x) != 0) as u8) << 1
                    | ((tile_y_data[0] & (0x80 >> tile_offset_x) != 0) as u8);
                if palette_index == 0 {
                    continue;
                }

                let color = (palette >> (2 * palette_index)) & 0b11;
                self.buf[self.m_ly as usize][x_pixel] = color;
            }
        }
    }

    fn get_lcdc_flag(&self, flag: LcdcFlag) -> bool {
        self.m_lcdc & (flag as u8) != 0
    }
    fn set_lcdc_flag(&mut self, flag: LcdcFlag, is_1: bool) {
        if is_1 {
            self.m_lcdc |= flag as u8;
        } else {
            self.m_lcdc &= !(flag as u8);
        }
    }

    fn get_stat_flag(&self, flag: StatFlag) -> bool {
        self.m_stat & (flag as u8) != 0
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
            0xFE00..=0xFE9F => self.m_oam[address as usize - 0xFE00],
            LCDC => self.m_lcdc,
            STAT => self.m_stat,
            SCY => self.m_scy,
            SCX => self.m_scx,
            LY => self.m_ly,
            LYC => self.m_lyc,
            BGP => self.m_bgp,
            OBP0 => self.m_obp0,
            OBP1 => self.m_obp1,
            WY => self.m_wy,
            WX => self.m_wx,
            UNKNOWN_1 => 0xFF,
            VBK => 0xFE,
            BCPS => 0xC8,
            BCPD => 0xFF,
            OCPS => 0xD0,
            OCPD => 0xFF,
            _ => unreachable!(),
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.m_ram[address as usize - 0x8000] = value,
            0xFE00..=0xFE9F => self.m_oam[address as usize - 0xFE00] = value,
            LCDC => {
                let enable_toggle = (self.m_lcdc & (LcdcFlag::LCDEnable as u8))
                    ^ (value & (LcdcFlag::LCDEnable as u8))
                    != 0;
                // TODO: Implement LCDEnable == 0
                // set all bits of lcdc
                self.m_lcdc = value;
                if enable_toggle && !self.get_lcdc_flag(LcdcFlag::LCDEnable) {
                    self.m_ly = 0;
                    self.internal_cycles = 0;
                    self.mode = Mode::VBlank;
                    self.blank_frame = true;
                }
            }
            STAT => {
                // TODO: Do I need to trigger the STAT interrupts here?
                self.m_stat = value & 0b01111000;
            }
            SCY => self.m_scy = value,
            SCX => self.m_scx = value,
            LY => (),
            LYC => {
                self.m_lyc = value;
                self.set_stat_flag(StatFlag::LycEqLy, self.m_lyc == self.m_ly);
            }
            BGP => self.m_bgp = value,
            OBP0 => self.m_obp0 = value,
            OBP1 => self.m_obp1 = value,
            WY => self.m_wy = value,
            WX => self.m_wx = value,
            UNKNOWN_1 | VBK | BCPS | BCPD | OCPS | OCPD => (),
            _ => unreachable!(),
        }
    }
}
