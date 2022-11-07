use self::registers::Registers;

//pub mod info;
pub mod registers;

pub struct Cpu {
    r: Registers,
    halt: bool,
    interrupt: bool,
}
