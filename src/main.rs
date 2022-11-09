#![allow(arithmetic_overflow)]

use cart::Cartridge;
use clap::Parser;
use cpu::Cpu;
use ppu::Ppu;

pub mod cart;
pub mod cpu;
pub mod mmu;
pub mod ppu;

#[derive(Parser)]
enum Command {
    Emu(EmuArgs),
    CartInfo(CartridgeArgs),
}

#[derive(Parser)]
struct EmuArgs {
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    cart: std::path::PathBuf,
}

#[derive(Parser)]
struct CartridgeArgs {
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    cart: std::path::PathBuf,
}

fn main() {
    match Command::parse() {
        Command::Emu(args) => match Cartridge::new_from_file(&args.cart) {
            Ok(cart) => {
                let ppu = Ppu::new();
                let cpu = Cpu::new(cart, ppu);
            }
            Err(e) => {
                eprintln!("Error reading cartridge:");
                eprintln!("{}", e);
            }
        },
        Command::CartInfo(args) => {
            let cart = Cartridge::new_from_file(&args.cart);
            match cart {
                Ok(c) => {
                    println!("{}", c.info.title);
                    println!("Type: {}", c.info.cart_type);
                    println!("Rom Size: {}", c.info.rom_size);
                    println!("Ram Size: {}", c.info.ram_size);
                    println!("Cgb Flag: {}", c.info.cgb_flag);
                    println!("Sgb Flag: {}", c.info.sgb_flag);
                    println!("Region: {}", c.info.region);
                    println!("Version: {}", c.info.version);
                    if let Some(l) = &c.info.new_licensee_code {
                        println!("New Licensee: {}", l);
                    } else {
                        println!("Old Licensee: {}", c.info.old_licensee_code);
                    }
                    print!("Header checksum...");
                    if c.header_checksum() {
                        println!("passed!");
                    } else {
                        println!("failed!");
                    }
                }
                Err(e) => {
                    eprintln!("Error reading cartridge:");
                    eprintln!("{}", e);
                }
            }
        }
    }
}
