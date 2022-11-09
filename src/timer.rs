pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

pub struct Timer {
    m_div: u8,
    m_tima: u8,
    m_tma: u8,
    m_tac: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Self {
            m_div: 0x18, // CGB: ?
            m_tima: 0x00,
            m_tma: 0x00,
            m_tac: 0xF8,
        }
    }

    pub fn reset(&mut self) {
        self.m_div = 0x18;
        self.m_tima = 0x00;
        self.m_tma = 0x00;
        self.m_tac = 0xF8;
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            DIV => self.m_div,
            TIMA => self.m_tima,
            TMA => self.m_tma,
            TAC => self.m_tac,
            _ => unreachable!(),
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            DIV => self.m_div = value,
            TIMA => self.m_tima = value,
            TMA => self.m_tma = value,
            TAC => self.m_tac = value,
            _ => unreachable!(),
        }
    }
}
