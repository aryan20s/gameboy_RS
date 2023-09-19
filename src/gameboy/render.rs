use std::{
    time::{Duration, Instant},
};

use log::debug;
use minifb::{Key, Window, WindowOptions};

extern crate spin_sleep;

pub struct InputKey {
    state_just_changed: bool,
    hardware_key: Key,
    currently_held: bool
}

impl InputKey {
    pub fn new(hardware_key: Key) -> Self {
        Self { state_just_changed: false, hardware_key: hardware_key, currently_held: false }
    }

    pub fn get_held(&self) -> bool {
        self.currently_held
    }

    pub fn get_state_just_changed(&self) -> bool {
        self.state_just_changed
    }

    fn update(&mut self, window: &Window) {
        if window.is_key_down(self.hardware_key) {
            if self.currently_held {
                self.state_just_changed = false;
            } else {
                self.currently_held = true;
                self.state_just_changed = true;
            }
        }

        if !window.is_key_down(self.hardware_key) {
            if !self.currently_held {
                self.state_just_changed = false;
            } else {
                self.currently_held = false;
                self.state_just_changed = true;
            }
        }
    }

    pub fn copy_state_from_other(&mut self, other: &InputKey) {
        self.currently_held = other.currently_held;
        self.state_just_changed = other.state_just_changed;
    }
}

pub struct Renderer {
    window: Window,
    last_frame_time: Instant,
    pub keys: Vec<InputKey>
}

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;
const SIZE_MULTIPLIER: usize = 3;
const WINDOW_WIDTH: usize = GB_SCREEN_WIDTH * SIZE_MULTIPLIER;
const WINDOW_HEIGHT: usize = GB_SCREEN_HEIGHT * SIZE_MULTIPLIER;
const FRAME_TIME_MICROS: u64 = 16450;

impl Renderer {
    pub fn new() -> Renderer {
        let mut window = Window::new(
            "LegumeGB_rs",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("Failed to create window, {}", e);
        });
        window.limit_update_rate(Some(Duration::from_micros(16600)));
        window.limit_update_rate(None);

        let mut ret = Renderer {
            window,
            last_frame_time: Instant::now(),
            keys: Vec::<InputKey>::with_capacity(8)
        };

        ret.keys.push(InputKey::new(Key::Enter)); //start
        ret.keys.push(InputKey::new(Key::Space)); //sel
        ret.keys.push(InputKey::new(Key::S)); //b
        ret.keys.push(InputKey::new(Key::A)); //a
        ret.keys.push(InputKey::new(Key::Down)); //down
        ret.keys.push(InputKey::new(Key::Up)); //up
        ret.keys.push(InputKey::new(Key::Left)); //left
        ret.keys.push(InputKey::new(Key::Right)); //right
        ret.keys.push(InputKey::new(Key::Q)); //debug swap tilemap

        ret
    }

    pub fn process_frame(&mut self, display: &[u32]) -> bool {
        let mut buffer: Vec<u32> = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT];

        for y in 0..WINDOW_HEIGHT {
            for x in 0..WINDOW_WIDTH {
                let value = display[(x / SIZE_MULTIPLIER) + (y / SIZE_MULTIPLIER) * GB_SCREEN_WIDTH];
                buffer[x + y * WINDOW_WIDTH] = value;
            }
        }

        self.window
            .update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .unwrap();

        for i in &mut self.keys {
            i.update(&self.window);
        }      

        if !self.window.is_key_down(Key::LeftCtrl) {
            let this_frame_end_time = self
                .last_frame_time
                .checked_add(Duration::from_micros(FRAME_TIME_MICROS))
                .unwrap_or(Instant::now());
            let frame_delta = this_frame_end_time
                .checked_duration_since(Instant::now())
                .unwrap_or(Duration::from_micros(0));
        
            spin_sleep::sleep(frame_delta);
        }
        self.last_frame_time = Instant::now();

        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}
