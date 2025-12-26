use atomic_float::AtomicF64;
use input::{
    Libinput, LibinputInterface,
    event::{
        Event,
        PointerEvent::{Button, Motion, ScrollFinger},
        gesture::{GestureEvent::Hold, GestureEventTrait, GestureHoldEvent::Begin},
        keyboard::{KeyState, KeyboardEventTrait},
        pointer::{Axis, ButtonState, PointerScrollEvent},
    },
};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use mio::{Events, Interest, Poll, Token, unix::SourceFd};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

const UNITS_PER_METER_MOUSE: f64 = 1000.0 / 0.0254;
const UNITS_PER_METER_SCROLL: f64 = 96.0 / 0.0254;
// NOTE: CHANGE THIS TO YOUR USERID
const PUT_URL: &str = "";

#[derive(Serialize, Debug)]
struct EventData {
    keypress: usize,
    right_click: usize,
    left_click: usize,
    middle_click: usize,
    mouse_distance: f64,
    scroll_distance: f64,
}

#[derive(Debug)]
struct InputStats {
    keypress: Arc<AtomicUsize>,
    right_click: Arc<AtomicUsize>,
    left_click: Arc<AtomicUsize>,
    middle_click: Arc<AtomicUsize>,
    mouse_distance: Arc<AtomicF64>,
    scroll_distance: Arc<AtomicF64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    let mut poll = Poll::new()?;
    let mut events_buffer = Events::with_capacity(128);
    poll.registry().register(
        &mut SourceFd(&input.as_raw_fd()),
        Token(0),
        Interest::READABLE,
    )?;

    let stats = Arc::new(InputStats {
        keypress: Arc::new(AtomicUsize::new(0)),
        right_click: Arc::new(AtomicUsize::new(0)),
        left_click: Arc::new(AtomicUsize::new(0)),
        middle_click: Arc::new(AtomicUsize::new(0)),
        mouse_distance: Arc::new(AtomicF64::new(0.0)),
        scroll_distance: Arc::new(AtomicF64::new(0.0)),
    });

    let stats_clone = Arc::clone(&stats);
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let data = EventData {
                keypress: stats_clone.keypress.load(Ordering::Relaxed),
                right_click: stats_clone.right_click.load(Ordering::Relaxed),
                left_click: stats_clone.left_click.load(Ordering::Relaxed),
                middle_click: stats_clone.middle_click.load(Ordering::Relaxed),
                mouse_distance: stats_clone.mouse_distance.load(Ordering::Relaxed),
                scroll_distance: stats_clone.scroll_distance.load(Ordering::Relaxed),
            };
            let resp = client.put(PUT_URL).json(&data).send();
            match resp {
                Ok(_) => {
                    stats_clone.keypress.store(0, Ordering::Relaxed);
                    stats_clone.right_click.store(0, Ordering::Relaxed);
                    stats_clone.left_click.store(0, Ordering::Relaxed);
                    stats_clone.middle_click.store(0, Ordering::Relaxed);
                    stats_clone.mouse_distance.store(0.0, Ordering::Relaxed);
                    stats_clone.scroll_distance.store(0.0, Ordering::Relaxed);
                }
                Err(err) => {
                    println!("Failed to make a post request: {}", err);
                }
            }
        }
    });

    loop {
        poll.poll(&mut events_buffer, None)?;
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Keyboard(keyboard_event) => {
                    if keyboard_event.key_state() == KeyState::Pressed {
                        stats.keypress.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Event::Pointer(pointer_event) => match pointer_event {
                    Button(pointer_button_event) => {
                        // 272 :- Left click
                        // 273 :- Right click
                        // 274 :- Middle click
                        if pointer_button_event.button() == 272
                            && pointer_button_event.button_state() == ButtonState::Pressed
                        {
                            stats.left_click.fetch_add(1, Ordering::Relaxed);
                        } else if pointer_button_event.button() == 273
                            && pointer_button_event.button_state() == ButtonState::Pressed
                        {
                            stats.right_click.fetch_add(1, Ordering::Relaxed);
                        } else if pointer_button_event.button() == 274
                            && pointer_button_event.button_state() == ButtonState::Pressed
                        {
                            stats.middle_click.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Motion(pointer_motion_event) => {
                        let dx = pointer_motion_event.dx();
                        let dy = pointer_motion_event.dy();
                        let distance = (dx.powi(2) + dy.powi(2)).sqrt() / UNITS_PER_METER_MOUSE;
                        stats.mouse_distance.fetch_add(distance, Ordering::Relaxed);
                    }
                    //ScrollWheel(pointer_scroll_event) => {
                    //    // 120 :- one unit of scrolling (thank you windows)
                    //    println!(
                    //        "{:?}",
                    //        pointer_scroll_event.scroll_value_v120(Axis::Vertical) / 120 as f64
                    //    );
                    //}
                    ScrollFinger(pointer_finger_event) => {
                        let x = pointer_finger_event.scroll_value(Axis::Horizontal);
                        let y = pointer_finger_event.scroll_value(Axis::Vertical);
                        let distance = (x.powi(2) + y.powi(2)).sqrt() / UNITS_PER_METER_SCROLL;
                        stats.scroll_distance.fetch_add(distance, Ordering::Relaxed);
                    }
                    _ => {}
                },
                Event::Gesture(gesture_event) => match gesture_event {
                    Hold(hold_event) => match hold_event {
                        Begin(_) => match hold_event.finger_count() {
                            1 => {
                                stats.left_click.fetch_add(1, Ordering::Relaxed);
                            }
                            2 => {
                                stats.right_click.fetch_add(1, Ordering::Relaxed);
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
