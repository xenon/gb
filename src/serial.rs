use crate::cpu::HZ;

pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub const SERIAL_CYCLES: u32 = HZ / 1024; // 1024KB/s transfer

enum TransferFlag {
    ShiftClock = 0b00000001, // 0=external, 1=internal
    ClockSpeed = 0b00000010, // CGB
    Start = 0b10000000,
}

fn control_flag(value: u8, flag: TransferFlag) -> bool {
    value & (flag as u8) != 0
}

pub struct Serial {
    m_data: u8,
    m_control: u8,
    in_transfer: bool,
    internal_timer: u32,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            m_data: 0xFF,
            m_control: 0b01111110,
            in_transfer: false,
            internal_timer: 0,
        }
    }

    pub fn reset(&mut self) {
        self.m_data = 0xFF;
        self.m_control = 0b01111110;
        self.in_transfer = false;
        self.internal_timer = 0;
    }

    pub fn step(&mut self, cycles: u32) -> bool {
        if self.in_transfer {
            self.internal_timer += cycles;
            if self.internal_timer >= SERIAL_CYCLES {
                // the byte is fully sent
                self.m_data = 0xFF; // assume disconnection, use 0xFF (invalid)
                self.set_control_flag(TransferFlag::Start, false);
                self.internal_timer = 0;
                self.in_transfer = false;
                return true;
            }
        }
        false
    }

    fn set_control_flag(&mut self, flag: TransferFlag, is_1: bool) {
        if is_1 {
            self.m_control = self.m_control | (flag as u8);
        } else {
            self.m_control = self.m_control & !(flag as u8);
        }
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
            SB => {
                self.m_data = value;
                /* Print the sent value:
                match char::try_from(value) {
                    Ok(c) => print!("{}", c),
                    Err(_) => print!("{:#04x}", value),
                }
                */
            }
            SC => {
                if control_flag(value, TransferFlag::Start) {
                    self.in_transfer = true;
                } else {
                    self.in_transfer = false;
                    self.internal_timer = 0;
                }
                self.m_control = value
            }
            _ => unreachable!(),
        }
    }
}
