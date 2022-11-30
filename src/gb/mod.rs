use self::{
    bios::Bios,
    cart::{info::CartridgeInfo, Cartridge},
    cpu::{
        registers::{Reg16, Reg8},
        Cpu,
    },
    joypad::Button,
    mmu::Mmu,
    ppu::{LCD_HEIGHT, LCD_WIDTH, ONE_FRAME_CYCLES},
};

pub mod apu;
pub mod bios;
pub mod cart;
pub mod cpu;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod serial;
pub mod timer;

pub struct Gb {
    cpu: Cpu,
}

impl Gb {
    pub fn new(cart: Cartridge, bios: Option<Bios>) -> Self {
        let mmu = Mmu::new(bios, cart);
        let cpu = Cpu::new(mmu);
        let mut gb = Self { cpu };
        gb.reset();
        gb
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu
            .r
            .reset(self.cpu.m.enable_bios && self.cpu.m.bios.is_some());
        self.cpu.m.reset();
    }

    pub fn step(&mut self) -> (u16, u8, u32) {
        self.cpu.step()
    }

    pub fn step_frame(&mut self, mut cycles: u32) -> u32 {
        while cycles < ONE_FRAME_CYCLES {
            cycles += self.cpu.step().2;
        }
        cycles -= ONE_FRAME_CYCLES;
        cycles // carry over remaining cycles
    }

    pub fn get_buf(&self) -> [[u8; LCD_WIDTH]; LCD_HEIGHT] {
        self.cpu.m.ppu.buf
    }

    pub fn button_release(&mut self, button: Button) {
        self.cpu.m.joypad.release(button)
    }

    pub fn button_press(&mut self, button: Button) {
        self.cpu.m.joypad.press(button)
    }

    pub fn cart_info(&self) -> &CartridgeInfo {
        &self.cpu.m.cart.info
    }

    pub(crate) fn next_step(&self) -> (u16, u8) {
        (self.cpu.r.pc, self.cpu.m.b(self.cpu.r.pc))
    }

    pub(crate) fn print(&self) -> String {
        format!(
            "a: {:#04x}\tf: {:#04x}\tb: {:#04x}\tc: {:#04x}\td: {:#04x}\te: {:#04x}\th: {:#04x}\tl: {:#04x}\tsp: {:#06x}",
            self.cpu.r.get_8(Reg8::A),
            self.cpu.r.get_8(Reg8::F),
            self.cpu.r.get_8(Reg8::B),
            self.cpu.r.get_8(Reg8::C),
            self.cpu.r.get_8(Reg8::D),
            self.cpu.r.get_8(Reg8::E),
            self.cpu.r.get_8(Reg8::H),
            self.cpu.r.get_8(Reg8::L),
            self.cpu.r.get_16(Reg16::SP),
        )
    }
}
