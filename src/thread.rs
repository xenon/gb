use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{sleep, Builder, JoinHandle},
    time::Instant,
};

use crate::{
    cpu::Cpu,
    joypad::Button,
    ppu::{ONE_FRAME_CYCLES, ONE_FRAME_DURATION},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemInput {
    Exit,
    TogglePause,
    JoypadOn(Button),
    JoypadOff(Button),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemEvent {
    ExitNow,
    Frame,
    Serial,
}

pub fn system_thread(cpu: Cpu) -> (JoinHandle<()>, Sender<SystemInput>, Receiver<SystemEvent>) {
    let (input_send, input_recv) = channel();
    let (event_send, event_recv) = channel();
    let handle = Builder::new()
        .name("gb system".to_string())
        .spawn(move || {
            system_loop(cpu, input_recv, event_send);
        })
        .unwrap_or_else(|_| panic!("Failed to build GB thread"));
    (handle, input_send, event_recv)
}

/// The System starts paused and must be sent SystemEvent::TogglePause to start it
/// Sending SystemInput::Exit will cause the thread to exit and send out SystemEvent::ExitNow
fn system_loop(mut cpu: Cpu, input: Receiver<SystemInput>, event: Sender<SystemEvent>) {
    let mut cycles = 0;
    let mut paused = true;
    loop {
        let mut start = Instant::now();
        // Handle inputs, block for next event if paused
        while let Ok(e) = if paused {
            input.recv().map_err(|_| ())
        } else {
            input.try_recv().map_err(|_| ())
        } {
            match e {
                SystemInput::Exit => {
                    let _ = event.send(SystemEvent::ExitNow); // Ok if the other thread doesn't care about this event
                    return;
                }
                SystemInput::TogglePause => {
                    if paused {
                        start = Instant::now();
                    }
                    paused = !paused;
                }
                SystemInput::JoypadOn(b) => cpu.m.joypad.press(b),
                SystemInput::JoypadOff(b) => cpu.m.joypad.release(b),
            }
        }

        if !paused {
            // Run CPU
            while cycles < ONE_FRAME_CYCLES {
                cycles += cpu.step().2;
            }
            cycles -= ONE_FRAME_CYCLES; // carry over the remaining cycles

            // Get next frame and send it
            event
                .send(SystemEvent::Frame)
                .expect("Failed to send the frame!");

            // Try to make emulation run at gb speed
            // TODO: Currently sleeps to long, causing the frames to be slow...
            let elapsed = Instant::now().duration_since(start);
            if elapsed < ONE_FRAME_DURATION {
                sleep(ONE_FRAME_DURATION - elapsed);
            }
        }
    }
}
