use crate::{cart::Cartridge, ppu::Ppu};

const WRAM_BANK_SIZE: usize = 0x1000;
const HRAM_SIZE: usize = 0x7F;

pub struct Mmu {
    pub cart: Cartridge,
    pub ppu: Ppu,
    wram: [u8; 2 * WRAM_BANK_SIZE], // CGB: 32768 bytes = 8 * WRAM_BANK_SIZE
    hram: [u8; HRAM_SIZE],
    interrupt_enable: u8,
}

impl Mmu {
    pub fn new(cart: Cartridge, ppu: Ppu) -> Self {
        let mut mmu = Mmu {
            cart,
            ppu,
            wram: [0; 2 * WRAM_BANK_SIZE],
            hram: [0; HRAM_SIZE],
            interrupt_enable: 0,
        };
        mmu.set_initial_state();
        mmu
    }

    pub fn set_initial_state(&mut self) {
        // TODO
        self.interrupt_enable = 0;
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.rom_b(address), // cart read rom
            0x8000..=0x9FFF => self.ppu.b(address),      // ppu read ram
            0xA000..=0xBFFF => self.cart.ram_b(address), // cart read ram
            0xC000..=0xCFFF => self.wram[(address as usize) - 0xC000], // wram bank 0
            0xD000..=0xDFFF => self.wram[(address as usize) - 0xC000], // wram bank 1, CGB: 1-7 switchable
            0xE000..=0xFDFF => self.wram[(address as usize) - 0xE000], // mirror of 0xC000..0xDDFF, prohibited to use, (used)
            0xFE00..=0xFE9F => 0,       // oam (sprite attribute table)
            0xFEA0..=0xFEFF => 0,       // unusable, prohibited to use
            0xFF00..=0xFF7F => todo!(), // io registers
            0xFF80..=0xFFFE => self.hram[(address as usize) - 0xFF80], // hram
            0xFFFF => self.interrupt_enable, // interrupt_enable
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
            0xFE00..=0xFE9F => (),      // oam (sprite attribute table)
            0xFEA0..=0xFEFF => (),      // unusable, prohibited to use
            0xFF00..=0xFF7F => todo!(), // io registers
            0xFF80..=0xFFFE => self.hram[(address as usize) - 0xFF80] = value, // hram
            0xFFFF => self.interrupt_enable = value, // interrupt_enable
        }
    }

    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8);
        self.wb(address + 1, (value >> 8) as u8);
    }
}
