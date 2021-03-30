extern crate minifb;
mod cpu;
use crate::cpu::CPU;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Missing file path");
    }
    let mut cpu = CPU::new();
    cpu.initialize();
    cpu.load_rom(&args[1]);
    let mut window = match minifb::Window::new(
        "Chip8",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        minifb::WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: minifb::Scale::X4,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            topmost: false,
            transparency: false,
            none: false,
        },
    ) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
    };
    loop {
        window.get_keys_released().map(|keys| {
            for t in keys {
                match t {
                    minifb::Key::Key1 => cpu.keys[0] = 0,
                    minifb::Key::Key2 => cpu.keys[1] = 0,
                    minifb::Key::Key3 => cpu.keys[2] = 0,
                    minifb::Key::Key4 => cpu.keys[3] = 0,
                    minifb::Key::Q => cpu.keys[4] = 0,
                    minifb::Key::W => cpu.keys[5] = 0,
                    minifb::Key::E => cpu.keys[6] = 0,
                    minifb::Key::R => cpu.keys[7] = 0,
                    minifb::Key::A => cpu.keys[8] = 0,
                    minifb::Key::S => cpu.keys[9] = 0,
                    minifb::Key::D => cpu.keys[10] = 0,
                    minifb::Key::F => cpu.keys[11] = 0,
                    minifb::Key::Z => cpu.keys[12] = 0,
                    minifb::Key::X => cpu.keys[13] = 0,
                    minifb::Key::C => cpu.keys[14] = 0,
                    minifb::Key::V => cpu.keys[15] = 0,
                    _ => (),
                }
            }
        });

        window.get_keys_pressed(minifb::KeyRepeat::No).map(|keys| {
            for t in keys {
                match t {
                    minifb::Key::Key1 => cpu.keys[0] = 1,
                    minifb::Key::Key2 => cpu.keys[1] = 1,
                    minifb::Key::Key3 => cpu.keys[2] = 1,
                    minifb::Key::Key4 => cpu.keys[3] = 1,
                    minifb::Key::Q => cpu.keys[4] = 1,
                    minifb::Key::W => cpu.keys[5] = 1,
                    minifb::Key::E => cpu.keys[6] = 1,
                    minifb::Key::R => cpu.keys[7] = 1,
                    minifb::Key::A => cpu.keys[8] = 1,
                    minifb::Key::S => cpu.keys[9] = 1,
                    minifb::Key::D => cpu.keys[10] = 1,
                    minifb::Key::F => cpu.keys[11] = 1,
                    minifb::Key::Z => cpu.keys[12] = 1,
                    minifb::Key::X => cpu.keys[13] = 1,
                    minifb::Key::C => cpu.keys[14] = 1,
                    minifb::Key::V => cpu.keys[15] = 1,
                    _ => (),
                }
            }
        });
        for _ in 0..8 {
            cpu.cycle();
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
        if cpu.update_screen {
            if window
                .update_with_buffer(cpu.get_screen_buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)
                .is_err()
            {
                panic!("error updating pixel buffer");
            }
            cpu.update_screen = false;
        }
        cpu.sub_dt();
        cpu.sub_st();
    }
}
