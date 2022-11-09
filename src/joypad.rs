pub const P1: u16 = 0xFF00;

pub struct Joypad {
    m_p1: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            m_p1: 0xCF, // CGB: C7 or CF
        }
    }

    pub fn reset(&mut self) {
        self.m_p1 = 0xCF;
    }

    pub fn b(&self, address: u16) -> u8 {
        if address == P1 {
            self.m_p1
        } else {
            unreachable!()
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        if address == P1 {
            self.m_p1 = value;
        } else {
            unreachable!()
        }
    }
}
