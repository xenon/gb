use std::{error::Error, fmt};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::mapper::MapperType;

#[derive(Clone, Debug)]
pub enum CartridgeInfoError {
    InvalidCartridgeSize,
    InvalidRom,
    InvalidRam,
    OldLicenseeCodeFail,
    TitleStringFail,
    CartTypeFail,
    SGBFlagFail,
    RegionFail,
}

impl std::fmt::Display for CartridgeInfoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeInfoError::InvalidCartridgeSize => {
                write!(f, "Cartridge is too short, probably invalid!")
            }
            CartridgeInfoError::InvalidRom => write!(f, "Invalid Rom amount!"),
            CartridgeInfoError::InvalidRam => write!(f, "Invalid Ram amount!"),
            CartridgeInfoError::OldLicenseeCodeFail => write!(f, "Old Licensee code invalid!"),
            CartridgeInfoError::TitleStringFail => write!(f, "Title string invalid!"),
            CartridgeInfoError::CartTypeFail => write!(f, "Cart type invalid!"),
            CartridgeInfoError::SGBFlagFail => write!(f, "SGB flag invalid!"),
            CartridgeInfoError::RegionFail => write!(f, "Region code invalid!"),
        }
    }
}

impl Error for CartridgeInfoError {}

pub struct CartridgeInfo {
    pub cart_type: CartType,
    pub title: String,
    pub rom_size: usize,
    pub ram_size: usize,
    pub mapper: MapperType,
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
    pub old_licensee_code: OldLicenseeCode,
    pub new_licensee_code: Option<NewLicenseeCode>,
}

impl CartridgeInfo {
    pub fn new_from_cartridge(bytes: &Vec<u8>) -> Result<Self, CartridgeInfoError> {
        if bytes.len() > 0x14D {
            let old_licensee_code = match OldLicenseeCode::try_from(bytes[0x014B]) {
                Ok(l) => l,
                Err(_) => return Err(CartridgeInfoError::OldLicenseeCodeFail),
            };

            let new_licensee_code = match old_licensee_code {
                OldLicenseeCode::NewLicensee => {
                    match NewLicenseeCode::try_from(bytes[0x0145] as u16) {
                        Ok(nlc) => Some(nlc),
                        Err(_) => {
                            eprintln!("New Licensee code was indicated but it failed to read!");
                            None
                        }
                    }
                }
                _ => None,
            };

            let title_len = {
                let mut title_len = 0;
                for byte in bytes.iter().take(0x0143).skip(0x0134) {
                    if byte == &0 {
                        break;
                    }
                    title_len += 1;
                }
                title_len
            };
            let title = match std::str::from_utf8(&bytes[0x0134..(0x0134 + title_len)]) {
                Ok(s) => s.to_string(),
                Err(_) => return Err(CartridgeInfoError::TitleStringFail),
            };

            let cart_type = match CartType::try_from(bytes[0x0147]) {
                Ok(t) => t,
                Err(_) => return Err(CartridgeInfoError::CartTypeFail),
            };
            let cgb_flag = match CgbFlag::try_from(bytes[0x0143]) {
                Ok(f) => f,
                Err(_) => CgbFlag::Undefined,
            };
            let sgb_flag = match SgbFlag::try_from(bytes[0x0146]) {
                Ok(l) => l,
                Err(_) => return Err(CartridgeInfoError::SGBFlagFail),
            };

            let rom_size = match get_rom_size(bytes[0x0148]) {
                Some(s) => s,
                None => return Err(CartridgeInfoError::InvalidRom),
            };
            let ram_size = match get_ram_size(bytes[0x0149]) {
                Some(s) => s,
                None => return Err(CartridgeInfoError::InvalidRam),
            };

            let region = match Region::try_from(bytes[0x014A]) {
                Ok(r) => r,
                Err(_) => return Err(CartridgeInfoError::RegionFail),
            };
            let version = bytes[0x014C];
            let header_checksum = bytes[0x014D];

            let mapper = match cart_type {
                CartType::Rom => MapperType::Rom,
                CartType::Mbc1 | CartType::Mbc1Ram | CartType::Mbc1RamBat => MapperType::Mbc1,
                CartType::Mbc2 | CartType::Mbc2Bat => MapperType::Mbc2,
                CartType::Mmm01 | CartType::Mmm01Ram | CartType::Mmm01RamBat => MapperType::Mmm01,
                CartType::Mbc3BatTim
                | CartType::Mbc3RamBatTim
                | CartType::Mbc3
                | CartType::Mbc3Ram
                | CartType::Mbc3RamBat => MapperType::Mbc3,
                CartType::Mbc5
                | CartType::Mbc5Ram
                | CartType::Mbc5RamBat
                | CartType::Mbc5Rum
                | CartType::Mbc5RamRum
                | CartType::Mbc5RamBatRum => MapperType::Mbc5,
                CartType::Mbc6 => MapperType::Mbc6,
                CartType::Mbc7RamBatRumSen => MapperType::Mbc7,
                CartType::Huc3 => MapperType::Huc3,
                CartType::Huc1RamBat => MapperType::Huc1,
                _ => {
                    eprintln!("This type of cart is not supported");
                    unimplemented!()
                }
            };

            let battery = matches!(
                cart_type,
                CartType::Mbc1RamBat
                    | CartType::Mbc2Bat
                    | CartType::RomRamBat
                    | CartType::Mmm01RamBat
                    | CartType::Mbc3BatTim
                    | CartType::Mbc3RamBatTim
                    | CartType::Mbc3RamBat
                    | CartType::Mbc5RamBat
                    | CartType::Mbc5RamBatRum
                    | CartType::Mbc7RamBatRumSen
            );
            let ram = matches!(
                cart_type,
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
                    | CartType::Huc1RamBat
            );
            let rumble = matches!(
                cart_type,
                CartType::Mbc5Rum
                    | CartType::Mbc5RamRum
                    | CartType::Mbc5RamBatRum
                    | CartType::Mbc7RamBatRumSen
            );
            let sensor = matches!(cart_type, CartType::Mbc7RamBatRumSen);
            let time = matches!(cart_type, CartType::Mbc3BatTim | CartType::Mbc3RamBatTim);

            Ok(Self {
                cart_type,
                title,
                rom_size,
                ram_size,
                mapper,
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
                old_licensee_code,
                new_licensee_code,
            })
        } else {
            Err(CartridgeInfoError::InvalidCartridgeSize)
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

#[derive(Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum OldLicenseeCode {
    None = 0x00,
    Nintendo = 0x01,
    Capcom = 0x08,
    HotB = 0x09,
    Jaleco = 0x0A,
    CoconutsJapan = 0x0B,
    EliteSystems = 0x0C,
    EA = 0x13,
    Hudsonsoft = 0x18,
    ITCEntertainment = 0x19,
    Yanoman = 0x1A,
    JapanClary = 0x1D,
    VirginInteractive = 0x1F,
    PCMComplete = 0x24,
    SanX = 0x25,
    KotobukiSystems = 0x28,
    Seta = 0x29,
    Infogrames = 0x30,
    Nintendo2 = 0x31,
    Bandai = 0x32,
    NewLicensee = 0x33,
    Konami = 0x34,
    HectorSoft = 0x35,
    Capcom2 = 0x38,
    Banpresto = 0x39,
    Entertainmenti = 0x3C,
    Gremlin = 0x3E,
    Ubisoft = 0x41,
    Atlus = 0x42,
    Malibu = 0x44,
    Angel = 0x46,
    SpectrumHoloby = 0x47,
    Irem = 0x49,
    VirginInteractive2 = 0x4A,
    Malibu2 = 0x4D,
    USGold = 0x4F,
    Absolute = 0x50,
    Acclaim = 0x51,
    Activision = 0x52,
    AmericanSammy = 0x53,
    GameTek = 0x54,
    ParkPlace = 0x55,
    LJN = 0x56,
    Matchbox = 0x57,
    MiltonBradley = 0x59,
    Mindscape = 0x5A,
    Romstar = 0x5B,
    NaxatSoft = 0x5C,
    Tradewest = 0x5D,
    Titus = 0x60,
    VirginInteractive3 = 0x61,
    OceanInteractive = 0x67,
    EA2 = 0x69,
    EliteSystems2 = 0x6E,
    ElectroBrain = 0x6F,
    Infogrames2 = 0x70,
    Interplay = 0x71,
    Broderbund = 0x72,
    SculpteredSoft = 0x73,
    TheSalesCurve = 0x75,
    Thq = 0x78,
    Accolade = 0x79,
    TriffixEntertainment = 0x7A,
    Microprose = 0x7C,
    Kemco = 0x7F,
    MisawaEntertainment = 0x80,
    Lozc = 0x83,
    TokumaShotenIntermedia = 0x86,
    BulletProofSoftware = 0x8B,
    VicTokai = 0x8C,
    Ape = 0x8E,
    IMax = 0x8F,
    ChunsoftCo = 0x91,
    VideoSystem = 0x92,
    TsubarayaProductionsCo = 0x93,
    VarieCorporation = 0x95,
    YonezawaSPal = 0x96,
    Kaneko = 0x97,
    Arc = 0x99,
    NihonBussan = 0x9A,
    Tecmo = 0x9B,
    Imagineer = 0x9C,
    Banpresto2 = 0x9D,
    Nova = 0x9F,
    HoriElectric = 0xA1,
    Bandai2 = 0xA2,
    Konami2 = 0xA4,
    Kawada = 0xA6,
    Takara = 0xA7,
    TechnosJapan = 0xA9,
    Broderbund2 = 0xAA,
    ToeiAnimation = 0xAC,
    Toho = 0xAD,
    Namco = 0xAF,
    Acclaim2 = 0xB0,
    ASCIIorNexsoft = 0xB1,
    Bandai3 = 0xB2,
    SquareEnix = 0xB4,
    HALLaboratory = 0xB6,
    SNK = 0xB7,
    PonyCanyon = 0xB9,
    CultureBrain = 0xBA,
    Sunsoft = 0xBB,
    SonyImagesoft = 0xBD,
    Sammy = 0xBF,
    Taito = 0xC0,
    Kemco2 = 0xC2,
    Squaresoft = 0xC3,
    TokumaShotenIntermedia2 = 0xC4,
    DataEast = 0xC5,
    Tonkinhouse = 0xC6,
    Koei = 0xC8,
    UFL = 0xC9,
    Ultra = 0xCA,
    Vap = 0xCB,
    UseCorporation = 0xCC,
    Meldac = 0xCD,
    PonyCanyonor = 0xCE,
    Angel2 = 0xCF,
    Taito2 = 0xD0,
    Sofel = 0xD1,
    Quest = 0xD2,
    SigmaEnterprises = 0xD3,
    ASKKodanshaCo = 0xD4,
    NaxatSoft2 = 0xD6,
    CopyaSystem = 0xD7,
    Banpresto3 = 0xD9,
    Tomy = 0xDA,
    LJN2 = 0xDB,
    NCS = 0xDD,
    Human = 0xDE,
    Altron = 0xDF,
    Jaleco2 = 0xE0,
    TowaChiki = 0xE1,
    Yutaka = 0xE2,
    Varie = 0xE3,
    Epcoh = 0xE5,
    Athena = 0xE7,
    AsmikACEEntertainment = 0xE8,
    Natsume = 0xE9,
    KingRecords = 0xEA,
    Atlus2 = 0xEB,
    EpicSonyRecords = 0xEC,
    IGS = 0xEE,
    AWave = 0xF0,
    ExtremeEntertainment = 0xF3,
    LJN3 = 0xFF,
}

impl fmt::Display for OldLicenseeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
#[repr(u16)]
#[allow(clippy::upper_case_acronyms)]
pub enum NewLicenseeCode {
    None = 0x00,
    NintendoRandD1 = 0x01,
    Capcom = 0x08,
    ElectronicArts = 0x13,
    HudsonSoft = 0x18,
    Bai = 0x19,
    Kss = 0x20,
    Pow = 0x22,
    PCMComplete = 0x24,
    Sanx = 0x25,
    KemcoJapan = 0x28,
    Seta = 0x29,
    Viacom = 0x30,
    Nintendo = 0x31,
    Bandai = 0x32,
    OceanAcclaim = 0x33,
    Konami = 0x34,
    Hector = 0x35,
    Taito = 0x37,
    Hudson = 0x38,
    Banpresto = 0x39,
    UbiSoft = 0x41,
    Atlus = 0x42,
    Malibu = 0x44,
    Angel = 0x46,
    BulletProof = 0x47,
    Irem = 0x49,
    Absolute = 0x50,
    Acclaim = 0x51,
    Activision = 0x52,
    Americansammy = 0x53,
    Konami2 = 0x54,
    Hitechentertainment = 0x55,
    LJN = 0x56,
    Matchbox = 0x57,
    Mattel = 0x58,
    MiltonBradley = 0x59,
    Titus = 0x60,
    Virgin = 0x61,
    LucasArts = 0x64,
    Ocean = 0x67,
    ElectronicArts2 = 0x69,
    Infogrames = 0x70,
    Interplay = 0x71,
    Broderbund = 0x72,
    Sculptured = 0x73,
    Sci = 0x75,
    THQ = 0x78,
    Accolade = 0x79,
    Misawa = 0x80,
    Lozc = 0x83,
    TokumaShotenIntermedia = 0x86,
    TsukudaOriginal = 0x87,
    Chunsoft = 0x91,
    Videosystem = 0x92,
    OceanAcclaim2 = 0x93,
    Varie = 0x95,
    Yonezawaspal = 0x96,
    Kaneko = 0x97,
    Packinsoft = 0x99,
    KonamiYuGiOh = 0xA4,
}
impl fmt::Display for NewLicenseeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn get_rom_size(value: u8) -> Option<usize> {
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

fn get_ram_size(value: u8) -> Option<usize> {
    match value {
        0x0 => Some(0),
        //0x1 => Some(2048),   // Unused by any licensed cartridge
        0x2 => Some(8192),   // 1 bank
        0x3 => Some(32768),  // 4 x 8 bank
        0x4 => Some(131072), // 16 x 8 bank
        0x5 => Some(65536),  // 8 x 8 bank
        _ => None,
    }
}
