use std::{
    mem,
    sync::mpsc::Receiver,
    thread::{Builder, JoinHandle},
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

use crate::{
    gb::Gb,
    joypad::Button,
    ppu::{LCD_HEIGHT, LCD_WIDTH},
    thread::{system_thread, SystemEvent, SystemInput},
};

#[derive(Clone, Copy, Debug)]
pub enum EventWrapper {
    Exit,
    SystemEvent(SystemEvent),
}

fn relay_thread(
    system_event: Receiver<SystemEvent>,
    winit_event: EventLoopProxy<EventWrapper>,
) -> JoinHandle<()> {
    Builder::new()
        .name("Winit event relay thread".to_string())
        .spawn(move || loop {
            if let Ok(e) = system_event.recv() {
                if let SystemEvent::ExitNow = e {
                    return;
                } else {
                    winit_event
                        .send_event(EventWrapper::SystemEvent(e))
                        .expect("Failed to relay SystemEvent to winit!");
                }
            }
        })
        .unwrap_or_else(|_| panic!("Failed to build relay thread!"))
}

pub fn launch_window(gb: Gb) {
    // Init system and thread
    let title = gb.cart_info().title.clone();
    let (sh, system_input, system_event) = system_thread(gb);
    let mut system_handle = Some(sh);

    // Winit + Pixels
    let event_loop = EventLoopBuilder::<EventWrapper>::with_user_event().build();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new((LCD_WIDTH * 2) as f64, (LCD_HEIGHT * 2) as f64);
        WindowBuilder::new()
            .with_title(format!("gb | {}", title))
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(LCD_WIDTH as u32, LCD_HEIGHT as u32, surface_texture)
            .expect("Could not create Pixels struct")
    };
    pixels.resize_surface(LCD_WIDTH as u32 * 2, LCD_HEIGHT as u32 * 2);

    // Custom Events
    let exit_event = event_loop.create_proxy();
    let winit_event = event_loop.create_proxy();
    let mut relay_handle = Some(relay_thread(system_event, winit_event));

    system_input
        .send(SystemInput::TogglePause)
        .expect("Failed to initially unpause the system");
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::UserEvent(EventWrapper::Exit) => {
                system_input
                    .send(SystemInput::Exit)
                    .expect("Failed to tell system thread to exit!");
                // Exit system and relay thread cleanly
                let mut s = None;
                mem::swap(&mut system_handle, &mut s);
                s.unwrap().join().expect("Failed to join system thread");
                let mut r = None;
                mem::swap(&mut relay_handle, &mut r);
                r.unwrap().join().expect("Failed to join relay thread");
                println!("Exiting!");
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(EventWrapper::SystemEvent(SystemEvent::Frame(buf))) => {
                let frame = pixels.get_frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let x = i % LCD_WIDTH as usize;
                    let y = i / LCD_WIDTH as usize;

                    let slice: [u8; 4] = match buf[y][x] {
                        /*0 => [0xFF, 0xFF, 0xFF, 0xFF],
                        1 => [0xCC, 0xCC, 0xCC, 0xFF],
                        2 => [0x77, 0x77, 0x77, 0xFF],
                        3 => [0x00, 0x00, 0x00, 0xFF],*/
                        0 => [255, 239, 206, 0xFF],
                        1 => [222, 148, 74, 0xFF],
                        2 => [173, 41, 33, 0xFF],
                        3 => [49, 24, 82, 0xFF],
                        _ => unreachable!(),
                    };

                    pixel.copy_from_slice(&slice);
                }
                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                // draw frame
                if pixels
                    .render()
                    .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    exit_event
                        .send_event(EventWrapper::Exit)
                        .expect("Could not exit cleanly");
                    return;
                }
            }
            _ => (),
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                exit_event
                    .send_event(EventWrapper::Exit)
                    .expect("Could not exit cleanly");
                return;
            }

            // Pause emulation
            if input.key_pressed(VirtualKeyCode::P) {
                system_input.send(SystemInput::TogglePause).unwrap();
            }

            // Reset emulation
            if input.key_pressed(VirtualKeyCode::R) {
                system_input.send(SystemInput::Reset).unwrap();
            }

            // Joypad
            // TODO: Vastly simplify by using a mapping
            // -> also would allow bindings to change
            // Directional
            if input.key_pressed(VirtualKeyCode::W) {
                system_input
                    .send(SystemInput::JoypadOn(Button::Up))
                    .unwrap();
            } else if input.key_released(VirtualKeyCode::W) {
                system_input
                    .send(SystemInput::JoypadOff(Button::Up))
                    .unwrap();
            }
            if input.key_pressed(VirtualKeyCode::S) {
                system_input
                    .send(SystemInput::JoypadOn(Button::Down))
                    .unwrap();
            } else if input.key_released(VirtualKeyCode::S) {
                system_input
                    .send(SystemInput::JoypadOff(Button::Down))
                    .unwrap();
            }
            if input.key_pressed(VirtualKeyCode::A) {
                system_input
                    .send(SystemInput::JoypadOn(Button::Left))
                    .unwrap();
            } else if input.key_released(VirtualKeyCode::A) {
                system_input
                    .send(SystemInput::JoypadOff(Button::Left))
                    .unwrap();
            }
            if input.key_pressed(VirtualKeyCode::D) {
                system_input
                    .send(SystemInput::JoypadOn(Button::Right))
                    .unwrap();
            } else if input.key_released(VirtualKeyCode::D) {
                system_input
                    .send(SystemInput::JoypadOff(Button::Right))
                    .unwrap();
            }
            // Action
            if input.key_pressed(VirtualKeyCode::Comma) {
                system_input.send(SystemInput::JoypadOn(Button::B)).unwrap();
            } else if input.key_released(VirtualKeyCode::Comma) {
                system_input
                    .send(SystemInput::JoypadOff(Button::B))
                    .unwrap();
            }
            if input.key_pressed(VirtualKeyCode::Period) {
                system_input.send(SystemInput::JoypadOn(Button::A)).unwrap();
            } else if input.key_released(VirtualKeyCode::Period) {
                system_input
                    .send(SystemInput::JoypadOff(Button::A))
                    .unwrap();
            }
            if input.key_pressed(VirtualKeyCode::Return) {
                system_input
                    .send(SystemInput::JoypadOn(Button::Start))
                    .unwrap();
            } else if input.key_released(VirtualKeyCode::Return) {
                system_input
                    .send(SystemInput::JoypadOff(Button::Start))
                    .unwrap();
            }
            if input.key_pressed(VirtualKeyCode::RShift) {
                system_input
                    .send(SystemInput::JoypadOn(Button::Select))
                    .unwrap();
            } else if input.key_released(VirtualKeyCode::RShift) {
                system_input
                    .send(SystemInput::JoypadOff(Button::Select))
                    .unwrap();
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
