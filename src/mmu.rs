use crate::{apu::Apu, cart::Cartridge, joypad::Joypad, ppu::Ppu, serial::Serial, timer::Timer};

// Sizes
const WRAM_BANK_SIZE: usize = 0x1000;
const HRAM_SIZE: usize = 0x100;

// Addresses

pub const INTF: u16 = 0xFF0F;

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

pub const INTE: u16 = 0xFFFF;

pub struct Mmu {
    pub cart: Cartridge,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub serial: Serial,
    pub timer: Timer,
    pub apu: Apu,
    wram: [u8; 2 * WRAM_BANK_SIZE], // CGB: 32768 bytes = 8 * WRAM_BANK_SIZE
    hram: [u8; HRAM_SIZE],
    intf: u8,
    inte: u8,
}

impl Mmu {
    pub fn new(cart: Cartridge, ppu: Ppu) -> Self {
        let mut mmu = Mmu {
            cart,
            ppu,
            joypad: Joypad::new(),
            serial: Serial::new(),
            timer: Timer::new(),
            apu: Apu::new(),
            wram: [0; 2 * WRAM_BANK_SIZE],
            hram: [0; HRAM_SIZE],
            intf: 0xE1,
            inte: 0x00,
        };
        mmu.reset();
        mmu
    }

    pub fn reset(&mut self) {
        self.cart.reset();
        self.ppu.reset();

        self.intf = 0xE1;

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

        self.inte = 0x00;
    }

    pub fn step(&mut self, cycles: u32) {
        if self.joypad.step() {
            self.intf |= 0b10000;
        }
        if self.timer.step(cycles) {
            self.intf |= 0b00100;
        }
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
            0xFF00 => self.joypad.b(address), // io registers begin
            0xFF01..=0xFF02 => self.serial.b(address),
            0xFF04..=0xFF07 => self.timer.b(address),
            0xFF0F => self.intf,
            0xFF10..=0xFF3F => self.apu.b(address),
            0xFFFF => self.inte,
            0xFF03..=0xFFFF => self.hram[(address as usize) - 0xFF00], // hram that is not special
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
            0xFF00 => self.joypad.wb(address, value), // io registers begin
            0xFF01..=0xFF02 => self.serial.wb(address, value),
            0xFF04..=0xFF07 => self.timer.wb(address, value),
            0xFF0F => self.intf = value,
            0xFF10..=0xFF3F => self.apu.wb(address, value),
            0xFFFF => self.inte = value,
            0xFF03..=0xFFFF => self.hram[(address as usize) - 0xFF00] = value, // hram that is not special
        }
    }

    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8);
        self.wb(address + 1, (value >> 8) as u8);
    }
}
