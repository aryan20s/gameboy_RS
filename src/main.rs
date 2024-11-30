mod gameboy;

use crate::gameboy::{Gameboy, SystemType};
use gameboy::render::Renderer;
use log::{error, info};
use simplelog::*;
use std::{
    env, fs,
    fs::File,
    io::{BufWriter, Write},
    ops::BitAnd,
    time::Instant,
    vec,
};

const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create("legumeGB.log").unwrap(),
        ),
    ])
    .unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        error!(
            "Arguments: {} <bootrom file> <rom file>",
            args[0]
        );
        return;
    }

    let mut renderer = Renderer::new();
    let mut gb = Gameboy::new(SystemType::DMG, &args[2], &args[1]);

    let mut last_frame = vec![0u32; GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT];
    let blank_frame = vec![0u32; GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT];
    let mut frames_run: u128 = 0;
    let start_time = Instant::now();

    while renderer.process_frame(&last_frame) {
        frames_run += 1;
        match gameboy::run_frame(&mut gb, &renderer.keys) {
            Ok(frame) => {
                if frame.len() == GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT {
                    last_frame = frame;
                } else {
                    last_frame = blank_frame.clone();
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    let time_run = start_time.elapsed().as_secs_f64();
    let fps = (frames_run as f64) / time_run;
    let frametime_ms: f64 = 1000f64 / fps;
    info!(
        "Ran at avg {:.2} FPS ({:.2} MS frametime)\n\n",
        fps, frametime_ms
    );

    let mut gb_memmap = Vec::<u8>::new();
    for i in 0x0000u16..=0xffffu16 {
        gb_memmap.push(gb.read_byte_raw(core::num::Wrapping(i)).0);
    }
    fs::write("memcopy.bin", &gb_memmap).unwrap();

    let mut frames_run: u128 = 0;
    let start_time = Instant::now();
    while renderer.process_frame(&last_frame) {
        frames_run += 1;
    }

    let time_run = start_time.elapsed().as_secs_f64();
    let fps = (frames_run as f64) / time_run;
    let frametime_ms: f64 = 1000f64 / fps;
    info!(
        "Ran at avg {:.2} FPS ({:.2} MS frametime)\n\n",
        fps, frametime_ms
    );
}
