use crate::{
    cart::Cartridge,
    mmu::Mmu,
    ppu::{Ppu, LCD_HEIGHT, LCD_WIDTH},
};

use self::{
    info::{CYCLES, CYCLES_CB, CYCLES_CB_BIT_HL},
    registers::{Flag, Reg16, Reg8, Registers},
};

pub mod info;
pub mod registers;
#[cfg(test)]
mod test;

pub const HZ: u32 = 4194304; // 2^22

pub struct Cpu {
    r: Registers,
    pub m: Mmu,
    halt: bool,
    stop: bool,
    ime: bool,
    pending_ei: bool,
    pending_di: bool,
    halt_bug: bool,
}

impl Cpu {
    pub(crate) fn print(&self) -> String {
        format!(
            "a: {:#04x}\tf: {:#04x}\tb: {:#04x}\tc: {:#04x}\td: {:#04x}\te: {:#04x}\th: {:#04x}\tl: {:#04x}\tsp: {:#06x}",
            self.r.get_8(Reg8::A),
            self.r.get_8(Reg8::F),
            self.r.get_8(Reg8::B),
            self.r.get_8(Reg8::C),
            self.r.get_8(Reg8::D),
            self.r.get_8(Reg8::E),
            self.r.get_8(Reg8::H),
            self.r.get_8(Reg8::L),
            self.r.get_16(Reg16::SP),
        )
    }

    pub(crate) fn next_step(&self) -> (u16, u8) {
        (self.r.pc, self.m.b(self.r.pc))
    }

    pub fn new(cart: Cartridge, ppu: Ppu) -> Self {
        let m = Mmu::new(cart, ppu);
        Self {
            r: Registers::new(),
            m,
            halt: false,
            stop: false,
            ime: false,
            pending_ei: false,
            pending_di: false,
            halt_bug: false,
        }
    }

    pub fn reset(&mut self) {
        self.r.reset();
        self.m.reset();

        self.halt = false;
        self.stop = false;
        self.ime = false;
        self.pending_ei = false;
        self.pending_di = false;
        self.halt_bug = false;
    }

    pub fn toggle_interrupt(&mut self) {
        if self.pending_di {
            self.ime = false;
            self.pending_di = false;
        }
        if self.pending_ei {
            self.ime = true;
            self.pending_ei = false;
        }
    }

    pub fn handle_interrupt(&mut self) -> bool {
        if (self.ime || self.halt || self.stop) && self.m.has_pending_interrupts() {
            self.halt = false;
            self.stop = false;
            if self.ime {
                self.ime = false;
                let interrupt = self.m.next_interrupt();
                self.m.disable_interrupt(interrupt);
                self.push(self.r.pc);
                self.r.pc = self.m.get_interrupt_handler(interrupt);
                return true;
            }
        }
        false
    }

    pub fn step(&mut self) -> (u16, u8, u32) {
        self.toggle_interrupt();
        let pc = self.r.pc;
        let instr = self.m.b(self.r.pc);
        let cycles = if self.handle_interrupt() || self.halt {
            4
        } else {
            self.step_instr()
        };
        self.m.step(cycles); // run other devices
        (pc, instr, cycles)
    }

    pub fn get_buf(&mut self) -> [[u8; LCD_WIDTH]; LCD_HEIGHT] {
        self.m.ppu.buf
    }

    fn alu_arg_get(&self, offset: u32) -> u8 {
        match offset {
            0..=5 => self.r.get_8(Reg8::get(offset + 2)),
            6 => self.m.b(self.r.get_16(Reg16::HL)),
            7 => self.r.get_8(Reg8::A),
            _ => unreachable!(),
        }
    }

    fn alu_arg_set(&mut self, offset: u32, value: u8) {
        match offset {
            0..=5 => self.r.set_8(Reg8::get(offset + 2), value),
            6 => self.m.wb(self.r.get_16(Reg16::HL), value),
            7 => self.r.set_8(Reg8::A, value),
            _ => unreachable!(),
        };
    }

    fn step_pc_b(&mut self) -> u8 {
        let next = self.m.b(self.r.pc);
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.r.pc = self.r.pc.wrapping_add(1);
        }
        next
    }

    fn step_pc_w(&mut self) -> u16 {
        let next = self.m.w(self.r.pc);
        self.r.pc = self.r.pc.wrapping_add(2);
        next
    }

    fn pop(&mut self) -> u16 {
        self.m.w(self.r.get_sp_pop())
    }

    fn push(&mut self, value: u16) {
        self.m.ww(self.r.get_sp_push(), value)
    }

    fn step_instr(&mut self) -> u32 {
        let instr = self.step_pc_b();
        let mut cycles = CYCLES[instr as usize];
        match instr {
            0x00 => { /* nop */ }
            0x10 => {
                self.stop = true;
            }
            0x01 | 0x11 | 0x21 | 0x31 => {
                let dest = Reg16::get((instr as u32 / 16) + 1); // ld d16
                let res = self.step_pc_w();
                self.r.set_16(dest, res);
            }
            0x02 | 0x12 | 0x22 | 0x32 => {
                let address = if instr < 0x22 {
                    self.r.get_16(Reg16::get((instr as u32 / 16) + 1))
                } else {
                    self.r.get_hl(instr == 0x22)
                };
                self.m.wb(address, self.r.get_8(Reg8::A)); // ld (N16), a
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                let dest = Reg16::get((instr as u32 / 16) + 1); // inc16
                self.r.set_16(dest, self.r.inc16(self.r.get_16(dest)))
            }
            0x04 | 0x14 | 0x24 => {
                let dest = Reg8::get(((instr as u32 / 16) * 2) + 2); // inc
                let res = self.r.inc(self.r.get_8(dest));
                self.r.set_8(dest, res);
            }
            0x34 => {
                let hl = self.r.get_16(Reg16::HL);
                let val = self.m.b(hl);
                let res = self.r.inc(val);
                self.m.wb(hl, res); // inc (hl)
            }
            0x05 | 0x15 | 0x25 => {
                let dest = Reg8::get(((instr as u32 / 16) * 2) + 2); // dec
                let res = self.r.dec(self.r.get_8(dest));
                self.r.set_8(dest, res);
            }
            0x35 => {
                let hl = self.r.get_16(Reg16::HL);
                let val = self.m.b(hl);
                let res = self.r.dec(val);
                self.m.wb(hl, res); // dec (hl)
            }
            0x06 | 0x16 | 0x26 => {
                let imm = self.step_pc_b();
                let dest = Reg8::get(((instr as u32 / 16) * 2) + 2);
                self.r.set_8(dest, imm); // ld n, d8
            }
            0x36 => {
                let imm = self.step_pc_b();
                self.m.wb(self.r.get_16(Reg16::HL), imm); // ld (hl) d8
            }
            0x07 => {
                let res = self.r.rlc(self.r.get_8(Reg8::A)); // rlc a
                self.r.set_flag(Flag::Z, false);
                self.r.set_8(Reg8::A, res);
            }
            0x17 => {
                let res = self.r.rl(self.r.get_8(Reg8::A)); // rl a
                self.r.set_flag(Flag::Z, false);
                self.r.set_8(Reg8::A, res);
            }
            0x27 => {
                let res = self.r.daa(); // daa
                self.r.set_8(Reg8::A, res);
            }
            0x37 => {
                self.r.scf(); // scf
            }
            0x08 => {
                let imm = self.step_pc_w(); // ld (a16) SP
                self.m.ww(imm, self.r.get_16(Reg16::SP));
            }
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                // jr r8 | jr z, r8 | jr c, r8 | jr nz, r8 | jr nc, r8
                let imm = self.step_pc_b() as i8;
                let cond = match instr {
                    0x18 => true,
                    0x20 => !self.r.get_flag(Flag::Z),
                    0x28 => self.r.get_flag(Flag::Z),
                    0x30 => !self.r.get_flag(Flag::C),
                    0x38 => self.r.get_flag(Flag::C),
                    _ => unreachable!(),
                };
                if cond {
                    if imm >= 0 {
                        self.r.pc = self.r.pc.wrapping_add(imm as u16);
                    } else {
                        self.r.pc = self.r.pc.wrapping_sub(imm.wrapping_neg() as u16);
                    }
                    cycles += 4;
                }
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let src = Reg16::get((instr as u32 / 16) + 1); // add16
                let res = self.r.add16(self.r.get_16(src));
                self.r.set_16(Reg16::HL, res);
            }
            0x0A | 0x1A | 0x2A | 0x3A => {
                let address = if instr < 0x2A {
                    self.r.get_16(Reg16::get((instr as u32 / 16) + 1))
                } else {
                    self.r.get_hl(instr == 0x2A)
                };
                self.r.set_8(Reg8::A, self.m.b(address)); // ld a, (N16)
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                let dest = Reg16::get((instr as u32 / 16) + 1); // dec16
                self.r.set_16(dest, self.r.dec16(self.r.get_16(dest)))
            }
            0x0C | 0x1C | 0x2C => {
                let dest = Reg8::get(((instr as u32 / 16) * 2) + 3); // inc
                let res = self.r.inc(self.r.get_8(dest));
                self.r.set_8(dest, res);
            }
            0x3C => {
                let res = self.r.inc(self.r.get_8(Reg8::A)); // inc a
                self.r.set_8(Reg8::A, res);
            }
            0x0D | 0x1D | 0x2D => {
                let dest = Reg8::get(((instr as u32 / 16) * 2) + 3); // dec
                let res = self.r.dec(self.r.get_8(dest));
                self.r.set_8(dest, res);
            }
            0x3D => {
                let res = self.r.dec(self.r.get_8(Reg8::A)); // dec a
                self.r.set_8(Reg8::A, res);
            }
            0x0E | 0x1E | 0x2E | 0x3E => {
                let imm = self.step_pc_b();
                let dest = if instr == 0x3E {
                    Reg8::A
                } else {
                    Reg8::get(((instr as u32 / 16) * 2) + 3)
                };
                self.r.set_8(dest, imm); // ld n, d8
            }
            0x0F => {
                let res = self.r.rrc(self.r.get_8(Reg8::A)); // rrc a
                self.r.set_flag(Flag::Z, false);
                self.r.set_8(Reg8::A, res);
            }
            0x1F => {
                let res = self.r.rr(self.r.get_8(Reg8::A)); // rr a
                self.r.set_flag(Flag::Z, false);
                self.r.set_8(Reg8::A, res);
            }
            0x2F => {
                let res = self.r.cpl(); // cpl
                self.r.set_8(Reg8::A, res);
            }
            0x3F => {
                self.r.ccf(); // ccf
            }
            0x40..=0x75 | 0x77 | 0x78..=0x7F => {
                let dest = (instr as u32 - 0x40) / 8; // ld
                let res = self.alu_arg_get((instr as u32 - 0x40) % 8);
                self.alu_arg_set(dest, res);
            }
            0x76 => {
                if !self.ime && self.m.has_pending_interrupts() {
                    self.halt_bug = true;
                } else {
                    self.halt = true; // halt
                }
            }
            0x80..=0x87 => {
                let offset = instr as u32 - 0x80; // add
                let n = self.alu_arg_get(offset);
                let res = self.r.add(n);
                self.r.set_8(Reg8::A, res);
            }
            0x88..=0x8F => {
                let offset = instr as u32 - 0x88; // adc
                let n = self.alu_arg_get(offset);
                let res = self.r.adc(n);
                self.r.set_8(Reg8::A, res);
            }
            0x90..=0x97 => {
                let offset = instr as u32 - 0x90; // sub
                let n = self.alu_arg_get(offset);
                let res = self.r.sub(n);
                self.r.set_8(Reg8::A, res);
            }
            0x98..=0x9F => {
                let offset = instr as u32 - 0x98; // sbc
                let n = self.alu_arg_get(offset);
                let res = self.r.sbc(n);
                self.r.set_8(Reg8::A, res);
            }
            0xA0..=0xA7 => {
                let offset = instr as u32 - 0xA0; // and
                let n = self.alu_arg_get(offset);
                let res = self.r.and(n);
                self.r.set_8(Reg8::A, res);
            }
            0xA8..=0xAF => {
                let offset = instr as u32 - 0xA8; // xor
                let n = self.alu_arg_get(offset);
                let res = self.r.xor(n);
                self.r.set_8(Reg8::A, res);
            }
            0xB0..=0xB7 => {
                let offset = instr as u32 - 0xB0; // or
                let n = self.alu_arg_get(offset);
                let res = self.r.or(n);
                self.r.set_8(Reg8::A, res);
            }
            0xB8..=0xBF => {
                let offset = instr as u32 - 0xB8; // cp
                let n = self.alu_arg_get(offset);
                self.r.cp(n);
            }
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                // ret nz, z, nc, c
                let cond = match instr {
                    0xC0 => !self.r.get_flag(Flag::Z),
                    0xC8 => self.r.get_flag(Flag::Z),
                    0xD0 => !self.r.get_flag(Flag::C),
                    0xD8 => self.r.get_flag(Flag::C),
                    _ => unreachable!(),
                };
                if cond {
                    self.r.pc = self.pop();
                    cycles += 12;
                }
            }
            0xE0 => {
                let imm = self.step_pc_b() as u16; // ldh (a8), a
                self.m.wb(0xFF00 + imm, self.r.get_8(Reg8::A));
            }
            0xF0 => {
                let imm = self.step_pc_b() as u16; // ldh a, (a8)
                self.r.set_8(Reg8::A, self.m.b(0xFF00 + imm));
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let dest = if instr == 0xF1 {
                    Reg16::AF
                } else {
                    Reg16::get(((instr as u32 - 0xC0) / 16) + 1)
                };
                let res = self.pop();
                self.r.set_16(dest, res); // pop
            }
            0xE2 => {
                // ld (c), a
                self.m
                    .wb(0xFF00 + self.r.get_8(Reg8::C) as u16, self.r.get_8(Reg8::A));
            }
            0xF2 => {
                let res = self.m.b(0xFF00 + self.r.get_8(Reg8::C) as u16);
                self.r.set_8(Reg8::A, res); // ld a, (c)
            }
            0xC2 | 0xD2 | 0xC3 | 0xCA | 0xDA => {
                // jp nz a16 | jp nc a16 | jp a16 | jp z, a16 | jp c, a16
                let imm = self.step_pc_w();
                let cond = match instr {
                    0xC3 => true,
                    0xC2 => !self.r.get_flag(Flag::Z),
                    0xCA => self.r.get_flag(Flag::Z),
                    0xD2 => !self.r.get_flag(Flag::C),
                    0xDA => self.r.get_flag(Flag::C),
                    _ => unreachable!(),
                };
                if cond {
                    self.r.pc = imm;
                    cycles += 4;
                }
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let src = if instr == 0xF5 {
                    Reg16::AF
                } else {
                    Reg16::get(((instr as u32 - 0xC0) / 16) + 1)
                }; // push
                self.push(self.r.get_16(src));
            }
            0xC6 | 0xD6 | 0xE6 | 0xF6 | 0xCE | 0xDE | 0xEE => {
                let imm = self.step_pc_b();
                let res = match instr {
                    0xC6 => self.r.add(imm),
                    0xD6 => self.r.sub(imm),
                    0xE6 => self.r.and(imm),
                    0xF6 => self.r.or(imm),
                    0xCE => self.r.adc(imm),
                    0xDE => self.r.sbc(imm),
                    0xEE => self.r.xor(imm),
                    _ => unreachable!(),
                };
                self.r.set_8(Reg8::A, res);
            }
            0xC7 | 0xD7 | 0xE7 | 0xF7 | 0xCF | 0xDF | 0xEF | 0xFF => {
                self.push(self.r.pc); // rst 00 10 20 30, 08 18 28 38
                self.r.pc = instr as u16 - 0xC7;
            }
            0xE8 | 0xF8 => {
                // add16 SP, r8
                // ld hl, sp+r8
                let dest = if instr == 0xE8 { Reg16::SP } else { Reg16::HL };
                let sp = self.r.get_16(Reg16::SP);
                let imm = self.step_pc_b() as i8;
                let res = self.r.add16_imm_i8(sp, imm);
                self.r.set_16(dest, res);
            }
            0xC9 => {
                self.r.pc = self.pop(); // ret
            }
            0xD9 => {
                self.r.pc = self.pop(); // reti
                self.pending_ei = true;
            }
            0xE9 => {
                // jp (hl)
                let hl = self.r.get_16(Reg16::HL);
                self.r.pc = hl;
            }
            0xF9 => {
                self.r.set_16(Reg16::SP, self.r.get_16(Reg16::HL)); // ld sp, hl
            }
            0xEA => {
                let imm = self.step_pc_w(); // ld (a16), a
                self.m.wb(imm, self.r.get_8(Reg8::A));
            }
            0xFA => {
                let imm = self.step_pc_w();
                let res = self.m.b(imm);
                self.r.set_8(Reg8::A, res); // ld a, (a16)
            }
            0xCB => {
                // CB instruction amounts already include the cost of CB itself. (speculation)
                cycles = self.step_instr_cb();
            }
            0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC => {
                // call nz, a16 | call nc, a16 | call a16 | call z, a16 | call c, a16
                let imm = self.step_pc_w();
                let cond = match instr {
                    0xCD => true,
                    0xC4 => !self.r.get_flag(Flag::Z),
                    0xCC => self.r.get_flag(Flag::Z),
                    0xD4 => !self.r.get_flag(Flag::C),
                    0xDC => self.r.get_flag(Flag::C),
                    _ => unreachable!(),
                };
                if cond {
                    self.push(self.r.pc);
                    self.r.pc = imm;
                    cycles += 12;
                }
            }
            0xFE => {
                let imm = self.step_pc_b(); // cp d8
                self.r.cp(imm);
            }
            0xF3 => {
                self.pending_di = true;
            }
            0xFB => {
                self.pending_ei = true;
            }
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                eprintln!("Invalid opcode: {:#04x}", instr);
                unreachable!();
            }
        };
        cycles
    }

    fn step_instr_cb(&mut self) -> u32 {
        let instr = self.step_pc_b();
        match instr {
            0x00..=0x07 => {
                let offset = instr as u32; // rlc
                let n = self.alu_arg_get(offset);
                let res = self.r.rlc(n);
                self.alu_arg_set(offset, res);
            }
            0x08..=0x0F => {
                let offset = instr as u32 - 0x08; // rrc
                let n = self.alu_arg_get(offset);
                let res = self.r.rrc(n);
                self.alu_arg_set(offset, res);
            }
            0x10..=0x17 => {
                let offset = instr as u32 - 0x10; // rl
                let n = self.alu_arg_get(offset);
                let res = self.r.rl(n);
                self.alu_arg_set(offset, res);
            }
            0x18..=0x1F => {
                let offset = instr as u32 - 0x18; // rr
                let n = self.alu_arg_get(offset);
                let res = self.r.rr(n);
                self.alu_arg_set(offset, res);
            }
            0x20..=0x27 => {
                let offset = instr as u32 - 0x20; // sla
                let n = self.alu_arg_get(offset);
                let res = self.r.sla(n);
                self.alu_arg_set(offset, res);
            }
            0x28..=0x2F => {
                let offset = instr as u32 - 0x28; // sra
                let n = self.alu_arg_get(offset);
                let res = self.r.sra(n);
                self.alu_arg_set(offset, res);
            }
            0x30..=0x37 => {
                let offset = instr as u32 - 0x30; // swap
                let n = self.alu_arg_get(offset);
                let res = self.r.swap(n);
                self.alu_arg_set(offset, res);
            }
            0x38..=0x3F => {
                let offset = instr as u32 - 0x38; // srl
                let n = self.alu_arg_get(offset);
                let res = self.r.srl(n);
                self.alu_arg_set(offset, res);
            }
            0x40..=0x7F => {
                let offset = (instr as u32 - 0x40) % 8; //bit
                let n = self.alu_arg_get(offset);
                self.r.bit(n, (instr - 0x40) / 8);
            }
            0x80..=0xBF => {
                let offset = (instr as u32 - 0x80) % 8; // res
                let n = self.alu_arg_get(offset);
                let res = self.r.res(n, (instr - 0x80) / 8);
                self.alu_arg_set(offset, res);
            }
            0xC0..=0xFF => {
                let offset = (instr as u32 - 0xC0) % 8; // set
                let n = self.alu_arg_get(offset);
                let res = self.r.set(n, (instr - 0xC0) / 8);
                self.alu_arg_set(offset, res);
            }
        }
        match instr {
            0x46 | 0x56 | 0x66 | 0x76 | 0x4E | 0x5E | 0x6E | 0x7E => CYCLES_CB_BIT_HL,
            _ => CYCLES_CB[instr as usize % 16],
        }
    }
}
