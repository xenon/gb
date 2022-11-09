const PPU_BANK_SIZE: usize = 0x2000;

pub struct Ppu {
    m_ram: [u8; PPU_BANK_SIZE], // CGB: 2 x PPU_BANK_SIZE for
                                // CGB: bank: u8,
}

impl Ppu {
    pub fn new() -> Self {
        let mut p = Self {
            m_ram: [0; PPU_BANK_SIZE],
        };
        p.reset();
        p
    }

    pub fn reset(&mut self) {
        self.m_ram = [0; PPU_BANK_SIZE];
    }

    pub fn b(&self, address: u16) -> u8 {
        let index = address as usize - 0x8000;
        if index < PPU_BANK_SIZE {
            self.m_ram[index]
        } else {
            unreachable!()
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        let index = address as usize - 0x8000;
        if index < PPU_BANK_SIZE {
            self.m_ram[index] = value
        } else {
            unreachable!()
        }
    }
}
