#![allow(clippy::new_without_default)]

use std::{path::PathBuf, process::exit};

use bios::Bios;
use cart::Cartridge;
use clap::Parser;
use gb::Gb;
use trace::run_trace;
use window::launch_window;

pub mod apu;
pub mod bios;
pub mod cart;
pub mod cpu;
pub mod gb;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod serial;
pub mod thread;
pub mod timer;
mod trace;
pub mod window;

#[derive(Parser)]
enum Command {
    Emu(EmuArgs),
    Trace(TraceArgs),
    CartInfo(CartridgeArgs),
}

#[derive(Parser)]
struct EmuArgs {
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    cart: std::path::PathBuf,
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    bios: Option<std::path::PathBuf>,
}

#[derive(Parser)]
struct TraceArgs {
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    cart: std::path::PathBuf,
    #[clap(long)]
    cycles: Option<u64>,
    #[clap(long)]
    verbose: bool,
}

#[derive(Parser)]
struct CartridgeArgs {
    #[clap(short, long, value_hint = clap::ValueHint::FilePath)]
    cart: std::path::PathBuf,
}

fn make_gb(cart: PathBuf, bios: Option<PathBuf>) -> Gb {
    match Cartridge::new_from_file(&cart) {
        Ok(cart) => {
            let bios = match bios {
                Some(file) => {
                    let bios = Bios::new_from_file(&file);
                    if let Err(e) = bios {
                        eprintln!("Error reading bios:");
                        eprintln!("{}", e);
                        exit(-2)
                    }
                    Some(bios.unwrap())
                }
                None => None,
            };
            return Gb::new(cart, bios);
        }
        Err(e) => {
            eprintln!("Error reading cartridge:");
            eprintln!("{}", e);
            exit(-1)
        }
    }
}

fn main() {
    match Command::parse() {
        Command::Emu(args) => {
            let gb = make_gb(args.cart, args.bios);
            launch_window(gb);
        }
        Command::Trace(args) => {
            let gb = make_gb(args.cart, None);
            run_trace(gb, args.cycles, args.verbose);
        }
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
