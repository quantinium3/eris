use atomic_float::AtomicF64;
use input::{
    Libinput, LibinputInterface,
    event::{
        Event,
        PointerEvent::{Button, Motion, ScrollFinger, ScrollWheel},
        gesture::{
            GestureEvent::{Hold, Pinch, Swipe},
            GestureHoldEvent::Begin,
        },
        keyboard::{KeyState, KeyboardEventTrait},
        pointer::{Axis, ButtonState, PointerScrollEvent},
    },
};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
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

struct Events {
    keypress: Arc<AtomicUsize>,
    mouse_right: Arc<AtomicUsize>,
    mouse_left: Arc<AtomicUsize>,
    mouse_middle: Arc<AtomicUsize>,
    mouse_distance: Arc<AtomicF64>,
    scroll_distance: Arc<AtomicF64>,
}

fn main() {
    let mut input = Libinput::new_with_udev(Interface);
    let events = Arc::new(Events {
        keypress: Arc::new(AtomicUsize::new(0)),
        mouse_right: Arc::new(AtomicUsize::new(0)),
        mouse_left: Arc::new(AtomicUsize::new(0)),
        mouse_middle: Arc::new(AtomicUsize::new(0)),
        mouse_distance: Arc::new(AtomicF64::new(0.0)),
        scroll_distance: Arc::new(AtomicF64::new(0.0)),
    });

    input.udev_assign_seat("seat0").unwrap();

    let events_clone = Arc::clone(&events);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
            println!(
                "Keypress: {}, Mouse right:{}, Mouse Left: {}, Mouse middle: {}, Mouse distance: {}, Scroll distance: {}",
                events_clone.keypress.load(Ordering::Relaxed),
                events_clone.mouse_right.load(Ordering::Relaxed),
                events_clone.mouse_left.load(Ordering::Relaxed),
                events_clone.mouse_middle.load(Ordering::Relaxed),
                events_clone.mouse_distance.load(Ordering::Relaxed),
                events_clone.scroll_distance.load(Ordering::Relaxed)
            );
        }
    });

    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Event::Keyboard(keyboard_event) => {
                    if keyboard_event.key_state() == KeyState::Pressed {
                        let _ = events.keypress.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Event::Pointer(pointer_event) => match pointer_event {
                    Button(pointer_button_event) => {
                        // 272 :- Right click
                        // 273 :- Left click
                        // 274 :- Middle click
                        println!("{:?}", pointer_button_event.button());
                        if pointer_button_event.button() == 272
                            && pointer_button_event.button_state() == ButtonState::Pressed
                        {
                            let _ = events.mouse_right.fetch_add(1, Ordering::Relaxed);
                        } else if pointer_button_event.button() == 273
                            && pointer_button_event.button_state() == ButtonState::Pressed
                        {
                            let _ = events.mouse_left.fetch_add(1, Ordering::Relaxed);
                        } else if pointer_button_event.button() == 274
                            && pointer_button_event.button_state() == ButtonState::Pressed
                        {
                            let _ = events.mouse_middle.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Motion(pointer_motion_event) => {
                        let _ = events.mouse_distance.fetch_add(
                            pointer_motion_event.dx().abs() + pointer_motion_event.dy().abs(),
                            Ordering::Relaxed,
                        );
                    }
                    ScrollWheel(pointer_scroll_event) => {
                        // 120 :- one unit of scrolling (thank you windows)
                        println!(
                            "{:?}",
                            pointer_scroll_event.scroll_value_v120(Axis::Vertical) / 120 as f64
                        );
                    }
                    ScrollFinger(pointer_finger_event) => {
                        let _ = events.scroll_distance.fetch_add(
                            pointer_finger_event.scroll_value(Axis::Vertical)
                                + pointer_finger_event.scroll_value(Axis::Horizontal),
                            Ordering::Relaxed,
                        );
                    }
                    _ => {}
                },
                Event::Gesture(gesture_event) => match gesture_event {
                    // NOTE: we ball with only right click as right click and left click as a
                    // gestue are mixed up
                    Hold(hold_event) => match hold_event {
                        Begin(hold_begin_event) => {
                            let _ = events.mouse_right.fetch_add(1, Ordering::Relaxed);
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
