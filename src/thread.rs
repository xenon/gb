use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{sleep, Builder, JoinHandle},
    time::Instant,
};

use crate::{
    gb::joypad::Button,
    gb::ppu::{LCD_HEIGHT, LCD_WIDTH, ONE_FRAME_DURATION},
    gb::Gb,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemInput {
    Exit,
    Reset,
    TogglePause,
    JoypadOn(Button),
    JoypadOff(Button),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemEvent {
    ExitNow,
    Frame([[u8; LCD_WIDTH]; LCD_HEIGHT]),
    Serial,
}

pub fn system_thread(gb: Gb) -> (JoinHandle<()>, Sender<SystemInput>, Receiver<SystemEvent>) {
    let (input_send, input_recv) = channel();
    let (event_send, event_recv) = channel();
    let handle = Builder::new()
        .name("gb system".to_string())
        .spawn(move || {
            system_loop(gb, input_recv, event_send);
        })
        .unwrap_or_else(|_| panic!("Failed to build GB thread"));
    (handle, input_send, event_recv)
}

/// The System starts paused and must be sent SystemEvent::TogglePause to start it
/// Sending SystemInput::Exit will cause the thread to exit and send out SystemEvent::ExitNow
fn system_loop(mut gb: Gb, input: Receiver<SystemInput>, event: Sender<SystemEvent>) {
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
                SystemInput::Reset => {
                    gb.reset();
                }
                SystemInput::TogglePause => {
                    if paused {
                        start = Instant::now();
                    }
                    paused = !paused;
                }
                SystemInput::JoypadOn(b) => gb.button_press(b),
                SystemInput::JoypadOff(b) => gb.button_release(b),
            }
        }

        if !paused {
            // Run CPU
            cycles = gb.step_frame(cycles);

            let buf = gb.get_buf();

            // Get next frame and send it
            event
                .send(SystemEvent::Frame(buf))
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
