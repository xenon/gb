use crate::{cart::Cartridge, ppu::Ppu};

// Sizes
const WRAM_BANK_SIZE: usize = 0x1000;
const HRAM_SIZE: usize = 0x80;

// Addresses
pub const P1: u16 = 0xFF00;
pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub const TIMER_DIV: u16 = 0xFF04;
pub const TIMER_TIMA: u16 = 0xFF05;
pub const TIMER_TMA: u16 = 0xFF06;
pub const TIMER_TAC: u16 = 0xFF07;

pub const INT_FLAGS: u16 = 0xFF0F;

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
pub const KEY1: u16 = 0xFF4D;
pub const VBK: u16 = 0xFF4F;

pub const HDMA1: u16 = 0xFF51;
pub const HDMA2: u16 = 0xFF52;
pub const HDMA3: u16 = 0xFF53;
pub const HDMA4: u16 = 0xFF54;
pub const HDMA5: u16 = 0xFF55;

pub const RP: u16 = 0xFF56;
pub const BCPS: u16 = 0xFF68;
pub const BCPD: u16 = 0xFF69;
pub const OCPS: u16 = 0xFF6A;
pub const OCPD: u16 = 0xFF6B;
pub const SVBK: u16 = 0xFF70;

pub const INT_ENABLE: u16 = 0xFFFF;

pub struct Mmu {
    pub cart: Cartridge,
    pub ppu: Ppu,
    wram: [u8; 2 * WRAM_BANK_SIZE], // CGB: 32768 bytes = 8 * WRAM_BANK_SIZE
    hram: [u8; HRAM_SIZE],
}

impl Mmu {
    pub fn new(cart: Cartridge, ppu: Ppu) -> Self {
        let mut mmu = Mmu {
            cart,
            ppu,
            wram: [0; 2 * WRAM_BANK_SIZE],
            hram: [0; HRAM_SIZE],
        };
        mmu.reset();
        mmu
    }

    pub fn reset(&mut self) {
        self.cart.reset();
        self.ppu.reset();
        self.wb(P1, 0xCF); // CGB: C7 or CF
        self.wb(SB, 0x00);
        self.wb(SC, 0x7E); // CGB: 7F

        // Timer
        self.wb(TIMER_DIV, 0x18); // CGB: ?
        self.wb(TIMER_TIMA, 0x00);
        self.wb(TIMER_TMA, 0x00);
        self.wb(TIMER_TAC, 0xF8);

        self.wb(INT_FLAGS, 0xE1);

        // NR
        self.wb(0xFF10, 0x80);
        self.wb(0xFF11, 0xBF);
        self.wb(0xFF12, 0xF3);
        self.wb(0xFF13, 0xFF);
        self.wb(0xFF14, 0xBF);
        self.wb(0xFF16, 0x3F);
        self.wb(0xFF17, 0x00);
        self.wb(0xFF18, 0xFF);
        self.wb(0xFF19, 0xBF);
        self.wb(0xFF1A, 0x7F);
        self.wb(0xFF1B, 0xFF);
        self.wb(0xFF1C, 0x9F);
        self.wb(0xFF1D, 0xFF);
        self.wb(0xFF1E, 0xBF);
        self.wb(0xFF20, 0xFF);
        self.wb(0xFF21, 0x00);
        self.wb(0xFF22, 0x00);
        self.wb(0xFF23, 0xBF);
        self.wb(0xFF24, 0x77);
        self.wb(0xFF25, 0xF3);
        self.wb(0xFF26, 0xF1);

        self.wb(LCDC, 0x91);
        self.wb(STAT, 0x81); // CGB: ?
        self.wb(SCY, 0x00);
        self.wb(SCX, 0x00);
        self.wb(LY, 0x91); // CGB: ?
        self.wb(LYC, 0x00);
        self.wb(DMA, 0xFF); // CGB: 00
        self.wb(BGP, 0xFC);
        self.wb(WY, 0x00);
        self.wb(WX, 0x00);
        self.wb(KEY1, 0xFF);
        self.wb(VBK, 0xFF);
        self.wb(HDMA1, 0xFF);
        self.wb(HDMA2, 0xFF);
        self.wb(HDMA3, 0xFF);
        self.wb(HDMA4, 0xFF);
        self.wb(HDMA5, 0xFF);
        self.wb(RP, 0xFF);
        self.wb(BCPS, 0xFF); // CGB: ?
        self.wb(BCPD, 0xFF); // CGB: ?
        self.wb(OCPS, 0xFF); // CGB: ?
        self.wb(OCPD, 0xFF); // CGB: ?
        self.wb(SVBK, 0xFF);
        self.wb(INT_ENABLE, 0x00);
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.rom_b(address), // cart read rom
            0x8000..=0x9FFF => self.ppu.b(address),      // ppu read ram
            0xA000..=0xBFFF => self.cart.ram_b(address), // cart read ram
            0xC000..=0xCFFF => self.wram[(address as usize) - 0xC000], // wram bank 0
            0xD000..=0xDFFF => self.wram[(address as usize) - 0xC000], // wram bank 1, CGB: 1-7 switchable
            0xE000..=0xFDFF => self.wram[(address as usize) - 0xE000], // mirror of 0xC000..0xDDFF, prohibited to use, (used)
            0xFE00..=0xFE9F => todo!(), // oam (sprite attribute table)
            0xFEA0..=0xFEFF => 0,       // unusable, prohibited to use
            0xFF00..=0xFF7F => todo!(), // io registers
            0xFF80..=0xFFFF => self.hram[(address as usize) - 0xFF80], // hram
        }
    }

    pub fn w(&self, address: u16) -> u16 {
        (self.b(address + 1) as u16) << 8 | (self.b(address) as u16)
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.rom_wb(address, value), // cart read rom
            0x8000..=0x9FFF => self.ppu.wb(address, value),      // ppu read ram
            0xA000..=0xBFFF => self.cart.ram_wb(address, value), // cart read ram
            0xC000..=0xCFFF => self.wram[(address as usize) - 0xC000] = value, // wram bank 0
            0xD000..=0xDFFF => self.wram[(address as usize) - 0xC000] = value, // wram bank 1, CGB: 1-7 switchable
            0xE000..=0xFDFF => self.wram[(address as usize) - 0xE000] = value, // mirror of 0xC000..0xDDFF, prohibited to use, (used)
            0xFE00..=0xFE9F => todo!(), // oam (sprite attribute table)
            0xFEA0..=0xFEFF => (),      // unusable, prohibited to use
            0xFF00..=0xFF7F => todo!(), // io registers
            0xFF80..=0xFFFF => self.hram[(address as usize) - 0xFF80] = value, // hram
        }
    }

    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8);
        self.wb(address + 1, (value >> 8) as u8);
    }
}
