#![allow(clippy::new_without_default)]

use cart::Cartridge;
use clap::Parser;
use cpu::Cpu;
use pixels::{Pixels, SurfaceTexture};
use ppu::Ppu;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

pub mod apu;
pub mod cart;
pub mod cpu;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod serial;
pub mod timer;

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
                // Init system
                println!("{}", cart.info.title);
                let mut cpu = Cpu::new(cart, Ppu::new());

                // Winit + Pixels
                let event_loop = EventLoop::new();
                let mut input = WinitInputHelper::new();
                let window = {
                    let size =
                        LogicalSize::new((ppu::LCD_WIDTH * 3) as f64, (ppu::LCD_HEIGHT * 3) as f64);
                    WindowBuilder::new()
                        .with_title("gb")
                        .with_inner_size(size)
                        .with_min_inner_size(size)
                        .build(&event_loop)
                        .unwrap()
                };

                let mut pixels = {
                    let window_size = window.inner_size();
                    let surface_texture =
                        SurfaceTexture::new(window_size.width, window_size.height, &window);
                    Pixels::new(ppu::LCD_WIDTH, ppu::LCD_HEIGHT, surface_texture)
                        .expect("Could not create Pixels struct")
                };
                pixels.resize_surface(ppu::LCD_WIDTH * 3, ppu::LCD_HEIGHT * 3);

                event_loop.run(move |event, _, control_flow| {
                    // Draw the current frame
                    if let Event::RedrawRequested(_) = event {
                        // run cpu
                        let step = cpu.step();
                        println!("{:#06x}:    {:#04x}   [{}]", step.0, step.1, step.2);
                        // draw frame
                        let frame = pixels.get_frame_mut();
                        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                            let x = (i % ppu::LCD_WIDTH as usize) as i16;
                            let y = (i / ppu::LCD_HEIGHT as usize) as i16;
                            let slice = &[
                                (x * y - x % 256) as u8,
                                (x * y - y % 256) as u8,
                                0xFF - (x * y - (x + y) % 256) as u8,
                                0xFF,
                            ];
                            pixel.copy_from_slice(slice);
                        }
                        if pixels
                            .render()
                            .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                            .is_err()
                        {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                    }

                    // Handle input events
                    if input.update(&event) {
                        // Close events
                        if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        // Resize the window
                        if let Some(size) = input.window_resized() {
                            pixels.resize_surface(size.width, size.height);
                        }

                        // Update internal state and request a redraw
                        window.request_redraw();
                    }
                });
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
