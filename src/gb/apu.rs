// Square 1
pub const NR10: u16 = 0xFF10; // sweep
pub const NR11: u16 = 0xFF11; // length and duty
pub const NR12: u16 = 0xFF12; // volume and envelope
pub const NR13: u16 = 0xFF13; // wavelength low
pub const NR14: u16 = 0xFF14; // wavelength high and control

// Square 2
// 0xFF15 unused
pub const NR21: u16 = 0xFF16; // length and duty
pub const NR22: u16 = 0xFF17; // volume and envelope
pub const NR23: u16 = 0xFF18; // wavelength low
pub const NR24: u16 = 0xFF19; // wavelength high and control

// Wave
pub const NR30: u16 = 0xFF1A; // dac enable
pub const NR31: u16 = 0xFF1B; // length timer
pub const NR32: u16 = 0xFF1C; // output level
pub const NR33: u16 = 0xFF1D; // wavelength low
pub const NR34: u16 = 0xFF1E; // wavelength high and control

// Noise
pub const NR40: u16 = 0xFF1F; // UNUSED
pub const NR41: u16 = 0xFF20; // length timer
pub const NR42: u16 = 0xFF21; // volume and envelope
pub const NR43: u16 = 0xFF22; // frequency and randomness
pub const NR44: u16 = 0xFF23; // control

// Control/Status
pub const NR50: u16 = 0xFF24; // master volume and VIN panning
pub const NR51: u16 = 0xFF25; // sound panning
pub const NR52: u16 = 0xFF26; // sound on/off

// 0xFF27..0xFF2F unused

// Wave Table
pub const WAVE_BEGIN: u16 = 0xFF30; // Samples 0 and 1
pub const WAVE_END: u16 = 0xFF3F; // Samples 30 and 31

pub struct Apu {
    m_squares: [[u8; 5]; 2],
    m_wave: [u8; 5],
    m_noise: [u8; 5],
    m_control: [u8; 3],
    m_wave_table: [u8; 16],
}

impl Apu {
    pub fn new() -> Self {
        let mut a = Apu {
            m_squares: [[0; 5]; 2],
            m_wave: [0; 5],
            m_noise: [0; 5],
            m_control: [0; 3],
            m_wave_table: [0; 16],
        };
        a.reset();
        a
    }

    pub fn reset(&mut self) {
        self.m_squares = [
            [0x80, 0xBF, 0xF3, 0xFF, 0xBF],
            [0xFF, 0x3F, 0x00, 0xFF, 0xBF],
        ];
        self.m_wave = [0x7F, 0xFF, 0x9F, 0xFF, 0xBF];
        self.m_noise = [0xFF, 0xFF, 0x00, 0x00, 0xBF];
        self.m_control = [0x77, 0xF3, 0xF1];
    }

    pub fn b(&self, address: u16) -> u8 {
        match address {
            NR10..=NR24 => {
                self.m_squares[((address - NR10) / 5) as usize][((address - NR10) % 5) as usize]
            }
            NR30..=NR34 => self.m_wave[(address - NR30) as usize],
            NR40..=NR44 => self.m_noise[(address - NR40) as usize],
            NR50..=NR52 => self.m_control[(address - NR50) as usize],
            WAVE_BEGIN..=WAVE_END => self.m_wave_table[(address - WAVE_BEGIN) as usize],
            _ => {
                if (NR10..=WAVE_END).contains(&address) {
                    0xFF // unused values
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn wb(&mut self, address: u16, value: u8) {
        match address {
            NR10..=NR24 => {
                self.m_squares[((address - NR10) / 5) as usize][((address - NR10) % 5) as usize] =
                    value;
            }
            NR30..=NR34 => self.m_wave[(address - NR30) as usize] = value,
            NR40..=NR44 => self.m_noise[(address - NR40) as usize] = value,
            NR50..=NR52 => self.m_control[(address - NR50) as usize] = value,
            WAVE_BEGIN..=WAVE_END => self.m_wave_table[(address - WAVE_BEGIN) as usize] = value,
            _ => {
                if !(NR10..=WAVE_END).contains(&address) {
                    unreachable!()
                }
            }
        }
    }
}
