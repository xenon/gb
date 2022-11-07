pub struct Registers {
    r: [u8; 8], // a, f, b, c, d, e, h, l
    pub pc: u16,
    pub sp: u16,
}

#[derive(PartialEq)]
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

#[repr(u32)]
pub enum Reg16 {
    AF = 0,
    BC = 1,
    DE = 2,
    HL = 3,
}

#[repr(u8)]
pub enum Flag {
    C = (1 << 4),
    H = (1 << 5),
    N = (1 << 6),
    Z = (1 << 7),
}

impl Registers {
    pub fn new() -> Self {
        Self {
            r: [0x01, 0xB0, 0x00, 0x13, 0x00, 0xD8, 0x01, 0x4D],
            pc: 0x100,
            sp: 0xFFFE,
        }
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
            self.r[Reg8::F as usize] &= !(flag as u8) & 0xF0;
        }
    }
}
