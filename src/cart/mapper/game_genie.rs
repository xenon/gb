use std::cell::RefCell;

use num_enum::IntoPrimitive;

use crate::cart::info::CartridgeInfo;

use super::{Mapper, RamLoadError, RamSaveError};

const CONTROL_0: u16 = 0x4000;
const CODE_ENABLE: u16 = 0x4001;
const CODE_0: u16 = 0x4003;
const CODE_0_END: u16 = 0x4007;
const CODE_1: u16 = 0x4008;
const CODE_1_END: u16 = 0x400C;
const CODE_2: u16 = 0x400D;
const CODE_2_END: u16 = 0x4011;
const CONTROL_1: u16 = 0x4012;
const CONTROL_2: u16 = 0x4013;

const CODE_SIZE: usize = 5;
const CODE_COUNT: usize = 3;

#[allow(dead_code)]
#[derive(Copy, Clone, Eq, IntoPrimitive, PartialEq)]
#[repr(u8)]
enum Control0Flag {
    CartPassthrough = 0b00000001,
    ActivityLight = 0b00000010,
    Lock = 0b00000100,
}

struct GameGenieState {
    m_control_0: u8,
}

impl GameGenieState {
    fn new() -> Self {
        Self { m_control_0: 0x01 }
    }

    fn reset(&mut self) {
        self.m_control_0 = 0x01;
    }

    fn get_flag(&self, flag: Control0Flag) -> bool {
        self.m_control_0 & (flag as u8) != 0
    }
    fn set_flag(&mut self, flag: Control0Flag, is_1: bool) {
        if is_1 {
            self.m_control_0 |= flag as u8;
        } else {
            self.m_control_0 &= !(flag as u8);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GameGenieCheat {
    address: u16,
    data: u8,
    data_compare: Option<u8>,
}

impl GameGenieCheat {
    pub fn update_from_code(&mut self, de: u8, fc: u8, gi: u8, ab: u8, _h: u8) {
        self.data = ab;
        self.address = ((fc as u16) << 8) | (de as u16);
        self.data_compare = if gi == 0xBA { None } else { Some(gi) };
    }
}

pub struct GameGenie {
    rom: Vec<u8>,
    mapper: Box<dyn Mapper>,
    state: RefCell<GameGenieState>,
    m_code_enable: u8,
    m_code: [[u8; CODE_SIZE]; CODE_COUNT],
    codes: [GameGenieCheat; CODE_COUNT],
}

impl GameGenie {
    pub fn new(mut rom: Vec<u8>, info: &CartridgeInfo, mapper: Box<dyn Mapper>) -> Self {
        if rom.len() < info.rom_size {
            // If the cart is too small, make space (game genie)
            rom.resize(info.rom_size, 0);
        };
        GameGenie {
            rom,
            mapper,
            state: RefCell::new(GameGenieState::new()),
            m_code_enable: 0x00,
            m_code: [[0; CODE_SIZE]; CODE_COUNT],
            codes: Default::default(),
        }
    }

    /// For resetting the genie state, which is different than a normal reset
    fn reset_genie(&mut self) {
        //self.m_control_0 = 0x01;
        self.m_code_enable = 0x00;
        self.m_code = [[0; CODE_SIZE]; CODE_COUNT];
        self.state.borrow_mut().reset();
    }

    fn update_codes(&mut self) {
        for (code, [de, fc, gi, ab, h]) in self.m_code.iter().enumerate() {
            self.codes[code].update_from_code(*de, *fc, *gi, *ab, *h);
        }
    }

    fn matches_code(&self, address: u16) -> (bool, u8) {
        if self.m_code_enable != 0 {
            for code in 0..=2 {
                if (self.m_code_enable & (0b1 << code)) != 0 && self.codes[code].address == address
                {
                    if let Some(comp) = self.codes[code].data_compare {
                        if comp == self.mapper.rom_b(address) {
                            return (true, self.codes[code].data);
                        } else {
                            return (false, 0);
                        }
                    } else {
                        return (true, self.codes[code].data);
                    }
                }
            }
        }
        (false, 0)
    }
}

#[allow(unused_variables)]
impl Mapper for GameGenie {
    fn reset(&mut self) {
        // Genie should keep it's state through a reset
        // However the game state is reset
        self.mapper.reset()
    }

    fn save_size(&self) -> Option<usize> {
        self.mapper.save_size()
    }
    fn load_save(&mut self, bytes: Vec<u8>) -> Result<(), RamLoadError> {
        self.mapper.load_save(bytes)
    }
    fn save_save(&mut self, bytes: Vec<u8>) -> Result<(), RamSaveError> {
        self.mapper.save_save(bytes)
    }
    fn reset_save(&mut self) {
        self.mapper.reset_save()
    }

    fn rom_b(&self, address: u16) -> u8 {
        let mut state = self.state.borrow_mut();

        if address == 0x100
            && state.get_flag(Control0Flag::CartPassthrough)
            && !state.get_flag(Control0Flag::Lock)
        {
            state.set_flag(Control0Flag::CartPassthrough, false);
        }

        if !state.get_flag(Control0Flag::CartPassthrough) {
            let res = match address {
                CONTROL_0 => state.m_control_0 & 0b1,
                CODE_ENABLE => self.m_code_enable,
                CODE_0..=CODE_0_END => self.m_code[0][(address - CODE_0) as usize],
                CODE_1..=CODE_1_END => self.m_code[1][(address - CODE_1) as usize],
                CODE_2..=CODE_2_END => self.m_code[2][(address - CODE_2) as usize],
                //x if x > 0x4000 => 0xFF,
                _ => self.rom[address as usize],
            };
            /*if address >= 0x2000 {
                eprintln!("GENIE ROM: [{:#06x}] reads {:#04x}", address, res);
            }*/
            res
        } else {
            if let (true, injected_val) = self.matches_code(address) {
                injected_val
            } else {
                self.mapper.rom_b(address)
            }
        }
    }
    fn rom_wb(&mut self, address: u16, value: u8) {
        if !self.state.borrow().get_flag(Control0Flag::CartPassthrough) {
            match address {
                CONTROL_0 => self.state.borrow_mut().m_control_0 = value,
                CODE_ENABLE => self.m_code_enable = value,
                CODE_0..=CODE_0_END => {
                    self.m_code[0][(address - CODE_0) as usize] = value;
                    self.update_codes();
                }
                CODE_1..=CODE_1_END => {
                    self.m_code[1][(address - CODE_1) as usize] = value;
                    self.update_codes();
                }
                CODE_2..=CODE_2_END => {
                    self.m_code[2][(address - CODE_2) as usize] = value;
                    self.update_codes();
                }
                _ => {}
            }
            //eprintln!("GENIE ROM: [{:#06x}] = {:#04x}", address, value);
        } else {
            self.mapper.rom_wb(address, value)
        }
    }
    fn ram_b(&self, address: u16) -> u8 {
        if self.state.borrow().get_flag(Control0Flag::CartPassthrough) {
            self.mapper.ram_b(address)
        } else {
            0xFF
        }
    }
    fn ram_wb(&mut self, address: u16, value: u8) {
        if self.state.borrow().get_flag(Control0Flag::CartPassthrough) {
            self.mapper.ram_wb(address, value)
        }
    }
}
