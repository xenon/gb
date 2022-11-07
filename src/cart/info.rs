use std::{error::Error, fmt};

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Debug)]
pub enum CartridgeError {
    InvalidCartridgeSize,
    InvalidRom,
    InvalidRam,
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeError::InvalidCartridgeSize => {
                write!(f, "Cartridge is too short, probably invalid!")
            }
            CartridgeError::InvalidRom => write!(f, "Invalid Rom amount!"),
            CartridgeError::InvalidRam => write!(f, "Invalid Ram amount!"),
        }
    }
}

impl Error for CartridgeError {}

pub struct CartridgeInfo {
    pub cart_type: CartType,
    pub title: String,
    pub rom_size: u32,
    pub ram_size: u32,
    pub battery: bool,
    pub ram: bool,
    pub rumble: bool,
    pub sensor: bool,
    pub time: bool,
    pub cgb_flag: CgbFlag,
    pub sgb_flag: SgbFlag,
    pub region: Region,
    pub version: u8,
    pub header_checksum: u8,
}

impl CartridgeInfo {
    pub fn new_from_cartridge(bytes: &Vec<u8>) -> Result<Self, Box<dyn Error>> {
        if bytes.len() > 0x14D {
            let title = std::str::from_utf8(&bytes[0x0134..0x0143]).map(|s| s.to_string())?;

            let cart_type = CartType::try_from(bytes[0x0147])?;
            let cgb_flag = match CgbFlag::try_from(bytes[0x0143]) {
                Ok(f) => f,
                Err(_) => CgbFlag::Undefined,
            };
            let sgb_flag = SgbFlag::try_from(bytes[0x0146])?;

            let rom_size =
                get_rom_size(bytes[0x0148]).ok_or_else(|| Box::new(CartridgeError::InvalidRom))?;
            let ram_size =
                get_ram_size(bytes[0x0149]).ok_or_else(|| Box::new(CartridgeError::InvalidRam))?;

            let region = Region::try_from(bytes[0x014A])?;
            let version = bytes[0x014C];
            let header_checksum = bytes[0x014D];

            let battery = match cart_type {
                CartType::Mbc1RamBat
                | CartType::Mbc2Bat
                | CartType::RomRamBat
                | CartType::Mmm01RamBat
                | CartType::Mbc3BatTim
                | CartType::Mbc3RamBatTim
                | CartType::Mbc3RamBat
                | CartType::Mbc5RamBat
                | CartType::Mbc5RamBatRum
                | CartType::Mbc7RamBatRumSen => true,
                _ => false,
            };

            let ram = match cart_type {
                CartType::Mbc1Ram
                | CartType::Mbc1RamBat
                | CartType::RomRam
                | CartType::RomRamBat
                | CartType::Mmm01Ram
                | CartType::Mmm01RamBat
                | CartType::Mbc3RamBatTim
                | CartType::Mbc3Ram
                | CartType::Mbc3RamBat
                | CartType::Mbc5Ram
                | CartType::Mbc5RamBat
                | CartType::Mbc5RamRum
                | CartType::Mbc5RamBatRum
                | CartType::Mbc6
                | CartType::Mbc7RamBatRumSen
                | CartType::Huc3
                | CartType::Huc1RamBat => true,
                _ => false,
            };

            let rumble = match cart_type {
                CartType::Mbc5Rum
                | CartType::Mbc5RamRum
                | CartType::Mbc5RamBatRum
                | CartType::Mbc7RamBatRumSen => true,
                _ => false,
            };

            let sensor = match cart_type {
                CartType::Mbc7RamBatRumSen => true,
                _ => false,
            };

            let time = match cart_type {
                CartType::Mbc3BatTim | CartType::Mbc3RamBatTim => true,
                _ => false,
            };

            Ok(Self {
                cart_type,
                title,
                rom_size,
                ram_size,
                battery,
                ram,
                rumble,
                sensor,
                time,
                cgb_flag,
                sgb_flag,
                region,
                version,
                header_checksum,
            })
        } else {
            Err(Box::new(CartridgeError::InvalidCartridgeSize))
        }
    }
}

#[derive(Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum CartType {
    Rom = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBat = 0x03,
    Mbc2 = 0x05,
    Mbc2Bat = 0x06,
    RomRam = 0x08,    // Unused by any licensed cartridge
    RomRamBat = 0x09, // Unused by any licensed cartridge
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBat = 0x0D,
    Mbc3BatTim = 0x0F,
    Mbc3RamBatTim = 0x10, // Used only in Pocket Monsters: Crystal Version
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,    // Unsure of usage
    Mbc3RamBat = 0x13, // Unsure of usage
    //Mbc4 = 0x15,
    //Mbc4Ram = 0x16,
    //Mbc4RamBat = 0x1B,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1A,
    Mbc5RamBat = 0x1B,
    Mbc5Rum = 0x1C,
    Mbc5RamRum = 0x1D,
    Mbc5RamBatRum = 0x1E,
    Mbc6 = 0x20,
    Mbc7RamBatRumSen = 0x22,
    Camera = 0xFC,
    //Tama5 = 0xFD, // Mapper implementation unknown
    Huc3 = 0xFE,
    Huc1RamBat = 0xFF,
}

impl fmt::Display for CartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum CgbFlag {
    Undefined = 0x0,
    Supported = 0x80,
    Only = 0xC0,
}

impl fmt::Display for CgbFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum SgbFlag {
    Disabled = 0x0,
    Supported = 0x03,
}

impl fmt::Display for SgbFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Region {
    JapanOrWorld = 0x0,
    NonJapanese = 0x1,
    Undefined = 0x2,
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn get_rom_size(value: u8) -> Option<u32> {
    /* No known cartridges or roms use these values but they are in the docs:
        0x52 => 1179648,
        0x53 => 1310720,
        0x54 => 1572864,
    */
    if value <= 8 {
        Some(32768 * (1 << value))
    } else {
        None
    }
}

fn get_ram_size(value: u8) -> Option<u32> {
    match value {
        0x0 => Some(0),
        //0x1 => 2048, // Unused by any licensed cartridge
        0x2 => Some(8192),   // 1 bank
        0x3 => Some(32768),  // 4 x 8 bank
        0x4 => Some(131072), // 16 x 8 bank
        0x5 => Some(65536),  // 8 x 8 bank
        _ => None,
    }
}
