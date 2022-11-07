use self::registers::Registers;

//pub mod info;
pub mod registers;
#[cfg(test)]
mod test;

pub struct Cpu {
    r: Registers,
    halt: bool,
    interrupt: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            r: Registers::new(),
            halt: false,
            interrupt: false,
        }
    }
}
