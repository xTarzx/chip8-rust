extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

mod chip8;
use chip8::Chip8;
use chip8::{VIDEO_HEIGHT, VIDEO_WIDTH};

const SCALE: u32 = 8;

fn main() {
    let sdl2_context = sdl2::init().unwrap();
    let video_subsystem = sdl2_context.video().unwrap();

    let window = video_subsystem
        .window(
            "CHIP-8",
            SCALE * VIDEO_WIDTH as u32,
            SCALE * VIDEO_HEIGHT as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl2_context.event_pump().unwrap();

    let mut chip8 = Chip8::new();
    // chip8.load_rom("AnimalRace.ch8");
    // chip8.load_rom("Airplane.ch8");
    // chip8.load_rom("Maze.ch8");
    // chip8.load_rom("test_opcode.ch8");

    'run: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'run;
                }

                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => match scancode {
                    Scancode::Num1 => chip8.set_key_state(0x1, 0xff),
                    Scancode::Num2 => chip8.set_key_state(0x2, 0xff),
                    Scancode::Num3 => chip8.set_key_state(0x3, 0xff),
                    Scancode::Num4 => chip8.set_key_state(0xc, 0xff),
                    Scancode::Q => chip8.set_key_state(0x4, 0xff),
                    Scancode::W => chip8.set_key_state(0x5, 0xff),
                    Scancode::E => chip8.set_key_state(0x6, 0xff),
                    Scancode::R => chip8.set_key_state(0xd, 0xff),
                    Scancode::A => chip8.set_key_state(0x7, 0xff),
                    Scancode::S => chip8.set_key_state(0x8, 0xff),
                    Scancode::D => chip8.set_key_state(0x9, 0xff),
                    Scancode::F => chip8.set_key_state(0xe, 0xff),
                    Scancode::Z => chip8.set_key_state(0xA, 0xff),
                    Scancode::X => chip8.set_key_state(0x0, 0xff),
                    Scancode::C => chip8.set_key_state(0xB, 0xff),
                    Scancode::V => chip8.set_key_state(0xf, 0xff),

                    _ => {
                        // println!("ignored")
                    }
                },
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => match scancode {
                    Scancode::Num1 => chip8.set_key_state(0x1, 0),
                    Scancode::Num2 => chip8.set_key_state(0x2, 0),
                    Scancode::Num3 => chip8.set_key_state(0x3, 0),
                    Scancode::Num4 => chip8.set_key_state(0xc, 0),
                    Scancode::Q => chip8.set_key_state(0x4, 0),
                    Scancode::W => chip8.set_key_state(0x5, 0),
                    Scancode::E => chip8.set_key_state(0x6, 0),
                    Scancode::R => chip8.set_key_state(0xd, 0),
                    Scancode::A => chip8.set_key_state(0x7, 0),
                    Scancode::S => chip8.set_key_state(0x8, 0),
                    Scancode::D => chip8.set_key_state(0x9, 0),
                    Scancode::F => chip8.set_key_state(0xe, 0),
                    Scancode::Z => chip8.set_key_state(0xA, 0),
                    Scancode::X => chip8.set_key_state(0x0, 0),
                    Scancode::C => chip8.set_key_state(0xB, 0),
                    Scancode::V => chip8.set_key_state(0xf, 0),

                    _ => {
                        // println!("ignored")
                    }
                },

                _ => {}
            }
        }

        chip8.cycle();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for row in 0..VIDEO_WIDTH {
            for col in 0..VIDEO_HEIGHT {
                let pixel = chip8.get_pixel_at(row, col);

                let rect = Rect::new(
                    (SCALE as i32 * row as i32),
                    (SCALE as i32 * col as i32),
                    SCALE,
                    SCALE,
                );
                if pixel != 0 {
                    canvas.draw_rect(rect).unwrap();
                }
            }
        }

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
