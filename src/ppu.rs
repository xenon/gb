const PPU_BANK_SIZE: usize = 0x2000;

pub struct Ppu {
    ram: [u8; PPU_BANK_SIZE], // CGB: 2 x PPU_BANK_SIZE for
                              //CGB: bank: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ram: [0; PPU_BANK_SIZE],
        }
    }

    pub fn b(&self, address: u16) -> u8 {
        self.ram[address as usize - 0x8000] // trivial rom for ROM only carts
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        self.ram[address as usize - 0x8000] = value
    }
}
