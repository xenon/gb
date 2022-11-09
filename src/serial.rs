pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub struct Serial {
    m_data: u8,
    m_control: u8,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            m_data: 0x00,
            m_control: 0x7E,
        }
    }

    pub fn reset(&mut self) {
        self.m_data = 0x00;
        self.m_control = 0x7E;
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            SB => self.m_data,
            SC => self.m_control,
            _ => unreachable!(),
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            SB => self.m_data = value,
            SC => self.m_control = value,
            _ => unreachable!(),
        }
    }
}
