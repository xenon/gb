use num_enum::UnsafeFromPrimitive;

pub const P1: u16 = 0xFF00;

pub struct Joypad {
    m_p1: u8,
    directions: u8,
    actions: u8,
    read_type: ReadType,
    interrupt_request: bool,
}

#[allow(dead_code)] // Doesn't understand UnsafeFromPrimitive uses all the values
#[derive(Copy, Clone, Eq, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
enum ReadType {
    None = 0b00,
    Actions = 0b01,
    Directions = 0b10,
    Both = 0b11,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            m_p1: 0b11001111, // CGB: highest 2 bits are set
            directions: 0b00101111,
            actions: 0b00011111,
            read_type: ReadType::None,
            interrupt_request: false,
        }
    }

    pub fn reset(&mut self) {
        self.m_p1 = 0b11111111;
        self.directions = 0b11101111;
        self.actions = 0b11011111;
        self.read_type = ReadType::None;
        self.interrupt_request = false;
    }

    pub fn step(&mut self) -> bool {
        if self.interrupt_request {
            self.interrupt_request = false;
            true
        } else {
            false
        }
    }

    fn set_p1(&mut self) {
        let new_value = match self.read_type {
            ReadType::None => 0b001111,
            ReadType::Actions => self.actions,
            ReadType::Directions => self.directions,
            ReadType::Both => self.actions | self.directions,
        };

        // Assumption interrupt is triggered whenever the bottom 4 bits change
        if (self.m_p1 & 0x0F) != (new_value & 0x0F) {
            self.interrupt_request = true;
        }
        self.m_p1 = new_value;
    }

    pub fn release(&mut self, button: Button) {
        match button {
            Button::Right => self.directions |= 0b0001,
            Button::Left => self.directions |= 0b0010,
            Button::Up => self.directions |= 0b0100,
            Button::Down => self.directions |= 0b1000,
            Button::A => self.actions |= 0b0001,
            Button::B => self.actions |= 0b0010,
            Button::Select => self.actions |= 0b0100,
            Button::Start => self.actions |= 0b1000,
        }
        self.set_p1();
    }

    pub fn press(&mut self, button: Button) {
        match button {
            Button::Right => self.directions &= !0b0001,
            Button::Left => self.directions &= !0b0010,
            Button::Up => self.directions &= !0b0100,
            Button::Down => self.directions &= !0b1000,
            Button::A => self.actions &= !0b0001,
            Button::B => self.actions &= !0b0010,
            Button::Select => self.actions &= !0b0100,
            Button::Start => self.actions &= !0b1000,
        }
        self.set_p1();
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
            self.read_type = unsafe { ReadType::from_unchecked((value & 0b00110000) >> 4) };
            self.set_p1();
        } else {
            unreachable!()
        }
    }
}
