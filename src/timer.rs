pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

pub struct Timer {
    m_div: u8,
    m_tima: u8,
    m_tma: u8,
    m_tac: u8,
    div_count: u32,
    tima_count: u32,
    enable_tima: bool,
    tima_step: u32,
}

impl Timer {
    pub fn new() -> Timer {
        Self {
            m_div: 0x18, // CGB: ?
            m_tima: 0x00,
            m_tma: 0x00,
            m_tac: 0xF8,
            div_count: 0,
            tima_count: 0,
            enable_tima: false,
            tima_step: 1024,
        }
    }

    pub fn reset(&mut self) {
        self.m_div = 0x18;
        self.m_tima = 0x00;
        self.m_tma = 0x00;
        self.m_tac = 0xF8;
        self.div_count = 0;
        self.tima_count = 0;
        self.enable_tima = false;
        self.tima_step = 1024;
    }

    pub fn step(&mut self, cycles: u32) -> bool {
        self.step_div(cycles);
        self.step_timer(cycles)
    }

    fn step_div(&mut self, cycles: u32) {
        self.div_count += cycles;
        let (div_div_256, div_mod_256) = (self.div_count / 256, self.div_count % 256);
        self.div_count = div_mod_256;
        self.m_div = self.m_div.wrapping_add(div_div_256 as u8);
    }

    fn step_timer(&mut self, cycles: u32) -> bool {
        if self.enable_tima {
            self.tima_count += cycles;
            let (tima_div_step, tima_mod_step) = (
                self.tima_count / self.tima_step,
                self.tima_count % self.tima_step,
            );
            self.tima_count = tima_mod_step;

            if (self.m_tima as u32 + tima_div_step) >= 0x100 {
                self.m_tima = self.m_tma + (self.m_tima as u32 + tima_div_step - 0xFF) as u8;

                return true;
            } else if tima_div_step >= 1 {
                self.m_tima += tima_div_step as u8;
            }
        }
        false
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
            DIV => {
                self.m_div = 0x00;
                self.div_count = 0;
                self.tima_count = 0; // In the hardware div and tima are the same counter, so they are both reset here
            }
            TIMA => self.m_tima = value,
            TMA => self.m_tma = value,
            TAC => {
                self.m_tac = 0xF8 & value;
                self.enable_tima = value & 0b100 != 0;
                self.tima_step = match value & 0b011 {
                    0 => 1024,
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => unreachable!(),
                };
                self.m_tima = self.m_tma;
                //self.tima_count = 0;
            }
            _ => unreachable!(),
        }
    }
}
