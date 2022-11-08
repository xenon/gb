pub struct Registers {
    r: [u8; 8], // a, f, b, c, d, e, h, l
    pub pc: u16,
    pub sp: u16,
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum Reg8 {
    A = 0,
    F = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    H = 6,
    L = 7,
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum Reg16 {
    AF = 0,
    BC = 1,
    DE = 2,
    HL = 3,
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Flag {
    C = (1 << 4),
    H = (1 << 5),
    N = (1 << 6),
    Z = (1 << 7),
}

impl Registers {
    pub fn new() -> Self {
        let mut r = Self {
            r: [0; 8],
            pc: 0,
            sp: 0,
        };
        r.reset();
        r
    }

    pub fn reset(&mut self) {
        self.r = [0x01, 0xB0, 0x00, 0x13, 0x00, 0xD8, 0x01, 0x4D];
        self.pc = 0x100;
        self.sp = 0xFFFE;
    }

    pub fn get_8(&self, r: Reg8) -> u8 {
        self.r[r as usize]
    }

    pub fn get_16(&self, r: Reg16) -> u16 {
        let base = 2 * (r as usize);
        ((self.r[base] as u16) << 8) | (self.r[base + 1] as u16)
    }

    pub fn set_8(&mut self, r: Reg8, value: u8) {
        if r == Reg8::F {
            self.r[r as usize] = value & 0xF0;
        } else {
            self.r[r as usize] = value;
        }
    }

    pub fn set_16(&mut self, r: Reg16, value: u16) {
        let base = 2 * (r as usize);
        let mask = if base == 0 { 0xF0 } else { 0xFF };
        self.r[base] = (value >> 8) as u8;
        self.r[base + 1] = (value & mask) as u8;
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.r[Reg8::F as usize] & (flag as u8) != 0
    }

    pub fn set_flag(&mut self, flag: Flag, is_1: bool) {
        if is_1 {
            self.r[Reg8::F as usize] |= flag as u8;
        } else {
            self.r[Reg8::F as usize] &= !(flag as u8);
        }
    }

    pub fn adc(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let carry = self.get_flag(Flag::C) as u8;
        let res = a_val.wrapping_add(n).wrapping_add(carry);
        self.set_flag(Flag::C, (a_val as u32) + (n as u32) + (carry as u32) > 0xFF);
        self.set_flag(Flag::H, (a_val & 0x0F) + (n & 0x0F) + carry > 0x0F);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn add(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let res = a_val.wrapping_add(n);
        self.set_flag(Flag::C, (a_val as u32) + (n as u32) > 0xFF);
        self.set_flag(Flag::H, (a_val & 0x0F) + (n & 0x0F) > 0x0F);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn and(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let res = a_val & n;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn bit(&mut self, n: u8, bit: u8) {
        let res = n & (1 << bit) == 0;
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res);
    }

    pub fn ccf(&mut self) {
        self.set_flag(Flag::C, !self.get_flag(Flag::C));
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
    }

    pub fn cp(&mut self, n: u8) {
        let a_val = self.get_8(Reg8::A);
        let res = a_val.wrapping_sub(n);
        self.set_flag(Flag::C, n as u32 > a_val as u32);
        self.set_flag(Flag::H, n & 0x0F > a_val & 0x0F);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::Z, res == 0);
    }

    pub fn cpl(&mut self) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let res = !a_val;
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::N, true);
        res
    }

    pub fn daa(&mut self) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let mut res = a_val;
        let mut correction = match (self.get_flag(Flag::C), self.get_flag(Flag::H)) {
            (true, true) => 0x66,
            (true, false) => 0x60,
            (false, true) => 0x06,
            _ => 0x00,
        };
        if !self.get_flag(Flag::N) {
            // was addition
            correction |= match (a_val > 0x99, a_val & 0x0F > 0x09) {
                (true, true) => 0x66,
                (true, false) => 0x60,
                (false, true) => 0x06,
                _ => 0x00,
            };
            res = res.wrapping_add(correction);
        } else {
            // was subtraction
            res = res.wrapping_sub(correction);
        }
        self.set_flag(Flag::C, correction >= 0x60);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn or(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let res = a_val | n;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn res(&mut self, n: u8, bit: u8) -> u8 {
        n & !(1 << bit)
    }

    pub fn rl(&mut self, n: u8) -> u8 {
        let carry = n >> 7 == 1;
        let res = (n << 1) + self.get_flag(Flag::C) as u8;
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn rlc(&mut self, n: u8) -> u8 {
        let carry = n >> 7 == 1;
        let res = (n << 1) + carry as u8;
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn rr(&mut self, n: u8) -> u8 {
        let carry = n & 0x01 == 1;
        let mut res = n >> 1;
        if self.get_flag(Flag::C) {
            res |= 0x80;
        }
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn rrc(&mut self, n: u8) -> u8 {
        let carry = n & 0x01 == 1;
        let mut res = n >> 1;
        if carry {
            res |= 0x80;
        }
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn sbc(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let carry = self.get_flag(Flag::C) as u8;
        let res = a_val.wrapping_sub(n).wrapping_sub(carry);
        self.set_flag(Flag::C, n as u32 + carry as u32 > a_val as u32);
        self.set_flag(Flag::H, (n & 0x0F) + carry > (a_val & 0x0F));
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn scf(&mut self) {
        self.set_flag(Flag::C, true);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
    }

    pub fn set(&mut self, reg: u8, bit: u8) -> u8 {
        reg | (1 << bit)
    }

    pub fn sla(&mut self, n: u8) -> u8 {
        let carry = n >> 7 == 1;
        let res = n << 1;
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn sra(&mut self, n: u8) -> u8 {
        let carry = n & 0x01 == 1;
        let res = n >> 1 | n & 0x80;
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn srl(&mut self, n: u8) -> u8 {
        let carry = n & 0x01 == 1;
        let res = n >> 1;
        self.set_flag(Flag::C, carry);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn sub(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let res = a_val.wrapping_sub(n);
        self.set_flag(Flag::C, n as u32 > a_val as u32);
        self.set_flag(Flag::H, (n & 0x0F) > (a_val & 0x0F));
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn swap(&mut self, n: u8) -> u8 {
        let res = n << 4 | n >> 4;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    pub fn xor(&mut self, n: u8) -> u8 {
        let a_val = self.get_8(Reg8::A);
        let res = a_val ^ n;
        self.set_flag(Flag::C, false);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, res == 0);
        res
    }

    // Untested instructions

    pub fn inc(&mut self, n: u8) -> u8 {
        let res = n.wrapping_add(1);
        self.set_flag(Flag::H, n & 0x0F == 0x0F);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::Z, n == 0);
        res
    }

    pub fn dec(&mut self, n: u8) -> u8 {
        let res = n.wrapping_sub(1);
        self.set_flag(Flag::H, n & 0x0F == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::Z, n == 0);
        res
    }

    pub fn add16(&mut self, n: u16) -> u16 {
        let hl_val = self.get_16(Reg16::HL);
        let res = hl_val.wrapping_add(n);
        self.set_flag(Flag::C, hl_val as u32 + n as u32 > 0xFFFF);
        self.set_flag(Flag::H, (hl_val & 0x07FF) + (n & 0x07FF) > 0x07FF);
        self.set_flag(Flag::N, false);
        res
    }

    pub fn inc16(&self, n: u16) -> u16 {
        n.wrapping_add(1)
    }

    pub fn dec16(&self, n: u16) -> u16 {
        n.wrapping_sub(1)
    }
}
