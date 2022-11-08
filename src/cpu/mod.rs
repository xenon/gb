use crate::{cart::Cartridge, mmu::Mmu, ppu::Ppu};

use self::registers::Registers;

//pub mod info;
pub mod registers;
#[cfg(test)]
mod test;

pub struct Cpu {
    r: Registers,
    m: Mmu,
    halt: bool,
    interrupt: bool,
}

impl Cpu {
    pub fn new(cart: Cartridge, ppu: Ppu) -> Self {
        let m = Mmu::new(cart, ppu);
        Self {
            r: Registers::new(),
            m,
            halt: false,
            interrupt: false,
        }
    }

    pub fn reset(&mut self) {
        self.r.reset();
        self.m.reset();

        self.halt = false;
        self.interrupt = false;
    }
}
