use std::{error::Error, fs::read, path::Path};

pub struct Bios {
    bios: Vec<u8>,
}

const BIOS_SIZE: usize = 0x100;

#[derive(Debug)]
pub enum BiosError {
    FileError(Box<dyn Error>),
    BiosSize,
}

impl std::fmt::Display for BiosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiosError::FileError(e) => write!(f, "File Error: {}!", e),
            BiosError::BiosSize => write!(f, "Bios is too large!"),
        }
    }
}

impl Bios {
    pub fn new_from_file(file: &Path) -> Result<Self, BiosError> {
        let bios = match read(file) {
            Ok(b) => b,
            Err(e) => return Err(BiosError::FileError(Box::new(e))),
        };
        if bios.len() > BIOS_SIZE {
            return Err(BiosError::BiosSize);
        }
        Ok(Self { bios })
    }

    pub fn b(&self, address: u16) -> u8 {
        let address = address as usize;
        if address < BIOS_SIZE {
            if address < self.bios.len() {
                self.bios[address]
            } else {
                0xFF
            }
        } else {
            unreachable!();
        }
    }

    #[allow(unused_variables)]
    pub fn wb(&self, address: u16, value: u8) {}
}
