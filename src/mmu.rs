use num_enum::{IntoPrimitive, UnsafeFromPrimitive};

use crate::{apu::Apu, cart::Cartridge, joypad::Joypad, ppu::Ppu, serial::Serial, timer::Timer};

// Sizes
const WRAM_BANK_SIZE: usize = 0x1000;
const HRAM_SIZE: usize = 0x100;

// Addresses

pub const INTF: u16 = 0xFF0F;

pub const KEY0: u16 = 0xFF4C;
pub const KEY1: u16 = 0xFF4D;
pub const VBK: u16 = 0xFF4F;

pub const DMA: u16 = 0xFF46;

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

pub const UNUSED_1: u16 = 0xFF72; // UNUSED_X are CGB Registers, unused on DMG
pub const UNUSED_2: u16 = 0xFF73;
pub const UNUSED_3: u16 = 0xFF74; // DMG: locked to 0xFF
pub const UNUSED_4: u16 = 0xFF75;

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

#[allow(dead_code)] // Doesn't understand UnsafeFromPrimitive uses all the values
#[derive(Copy, Clone, Debug, Eq, IntoPrimitive, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Interrupt {
    VBlank = 0,
    LCDStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
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
            hram: [0xFF; HRAM_SIZE],
            intf: 0xE1,
            inte: 0x00,
        };
        mmu.reset();
        mmu
    }

    pub fn reset(&mut self) {
        self.cart.reset();
        self.joypad.reset();
        self.serial.reset();
        self.timer.reset();
        self.ppu.reset();

        self.intf = 0xE1;

        //self.wb(KEY1, 0xFF);
        //self.wb(VBK, 0xFF);

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

        self.wb(0xFF03, 0xFF);
        self.wb(UNUSED_1, 0x00); // CGB
        self.wb(UNUSED_2, 0x00); // CGB
        self.wb(UNUSED_3, 0xFF); // DMG: locked to 0xFF, CGB: 0x00
        self.wb(UNUSED_4, 0x8F);

        self.wb(0xFF76, 0x00);
        self.wb(0xFF77, 0x00);

        self.inte = 0x00;
    }

    pub fn has_pending_interrupts(&self) -> bool {
        (self.inte & self.intf) != 0
    }

    pub fn next_interrupt(&self) -> Interrupt {
        let pending = self.inte & self.intf;
        if pending & 0b00001 != 0 {
            Interrupt::VBlank
        } else if pending & 0b00010 != 0 {
            Interrupt::LCDStat
        } else if pending & 0b00100 != 0 {
            Interrupt::Timer
        } else if pending & 0b01000 != 0 {
            Interrupt::Serial
        } else if pending & 0b10000 != 0 {
            Interrupt::Joypad
        } else {
            unreachable!();
        }
    }

    pub fn disable_interrupt(&mut self, i: Interrupt) {
        let mask: u8 = 1 << <Interrupt as Into<u8>>::into(i);
        self.intf &= !mask;
    }

    pub fn get_interrupt_handler(&self, i: Interrupt) -> u16 {
        0x0040 + (8 * <Interrupt as Into<u8>>::into(i)) as u16
    }

    pub fn step(&mut self, cycles: u32) {
        if self.joypad.step() {
            self.intf |= 0b10000;
        }
        if self.serial.step(cycles) {
            self.intf |= 0b01000;
        }
        if self.timer.step(cycles) {
            self.intf |= 0b00100;
        }
        let (intf_vblank, intf_lcdstat) = self.ppu.step(cycles);
        if intf_vblank {
            self.intf |= 0b00001;
        }
        if intf_lcdstat {
            self.intf |= 0b00010;
        }
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.rom_b(address), // cart read rom
            0x8000..=0x9FFF => self.ppu.b(address),      // ppu read ram
            0xA000..=0xBFFF => self.cart.ram_b(address), // cart read ram
            0xC000..=0xCFFF => self.wram[(address as usize) - 0xC000], // wram bank 0
            0xD000..=0xDFFF => self.wram[(address as usize) - 0xC000], // wram bank 1, CGB: 1-7 switchable
            0xE000..=0xFDFF => self.wram[(address as usize) - 0xE000], // mirror of 0xC000..0xDDFF, prohibited to use, (used in some cases)
            0xFE00..=0xFE9F => self.ppu.b(address), // oam (sprite attribute table)
            0xFEA0..=0xFEFF => 0,                   // unusable, prohibited to use
            0xFF00 => self.joypad.b(address),       // io registers begin
            0xFF01..=0xFF02 => self.serial.b(address),
            0xFF04..=0xFF07 => self.timer.b(address),
            INTF => self.intf | 0b11100000,
            0xFF10..=0xFF3F => self.apu.b(address),
            DMA => 0xFF, // KLUDGE: not sure what real hardware does in this case
            KEY0 | KEY1 => self.hram[(address as usize) - 0xFF00],
            0xFF40..=0xFF45 | 0xFF47..=0xFF4F | 0xFF68..=0xFF6B => self.ppu.b(address),
            0xFF03..=0xFFFE => self.hram[(address as usize) - 0xFF00], // hram that is not special
            INTE => self.inte | 0b11100000,
        }
    }

    pub fn w(&self, address: u16) -> u16 {
        (self.b(address + 1) as u16) << 8 | (self.b(address) as u16)
    }

    fn dma_transfer(&mut self, value: u8) {
        // KLUDGE:
        // Typically OAM transfer takes 160 cycles but the CPU can only access HRAM and usually just busy idles
        // So effectively we can skip the cycle accuracy and memory restrictions and just copy the whole memory at once.
        let src = (value as u16) << 8;
        if src <= 0xDF00 {
            for i in 0..=0x9F {
                self.ppu.wb(0xFE00 + i, self.b(src + i));
            }
        } else {
            unreachable!();
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.rom_wb(address, value), // cart read rom
            0x8000..=0x9FFF => self.ppu.wb(address, value),      // ppu read ram
            0xA000..=0xBFFF => self.cart.ram_wb(address, value), // cart read ram
            0xC000..=0xCFFF => self.wram[(address as usize) - 0xC000] = value, // wram bank 0
            0xD000..=0xDFFF => self.wram[(address as usize) - 0xC000] = value, // wram bank 1, CGB: 1-7 switchable
            0xE000..=0xFDFF => self.wram[(address as usize) - 0xE000] = value, // mirror of 0xC000..0xDDFF, prohibited to use, (used)
            0xFE00..=0xFE9F => self.ppu.wb(address, value), // oam (sprite attribute table)
            0xFEA0..=0xFEFF => (),                          // unusable, prohibited to use
            0xFF00 => self.joypad.wb(address, value),       // io registers begin
            0xFF01..=0xFF02 => self.serial.wb(address, value),
            0xFF04..=0xFF07 => self.timer.wb(address, value),
            INTF => self.intf = value & 0b00011111,
            0xFF10..=0xFF3F => self.apu.wb(address, value),
            DMA => self.dma_transfer(value),
            KEY0 | KEY1 => self.hram[(address as usize) - 0xFF00] = value,
            0xFF40..=0xFF45 | 0xFF47..=0xFF4F | 0xFF68..=0xFF6B => self.ppu.wb(address, value),
            0xFF03..=0xFFFE => self.hram[(address as usize) - 0xFF00] = value, // hram that is not special
            INTE => self.inte = value & 0b00011111,
        }
    }

    pub fn ww(&mut self, address: u16, value: u16) {
        self.wb(address, (value & 0xFF) as u8);
        self.wb(address + 1, (value >> 8) as u8);
    }
}
