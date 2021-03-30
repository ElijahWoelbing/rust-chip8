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
            scale: minifb::Scale::X8,
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
        // emulate 500hz cpu speed
        for _ in 0..8 {
            cpu.cycle();

            let keys: [minifb::Key; 16] = [
                minifb::Key::Key1,
                minifb::Key::Key2,
                minifb::Key::Key3,
                minifb::Key::Key4,
                minifb::Key::Q,
                minifb::Key::W,
                minifb::Key::E,
                minifb::Key::R,
                minifb::Key::A,
                minifb::Key::S,
                minifb::Key::D,
                minifb::Key::F,
                minifb::Key::Z,
                minifb::Key::X,
                minifb::Key::C,
                minifb::Key::V,
            ];

            for (i, key) in keys.iter().enumerate() {
                if window.is_key_down(*key) {
                    cpu.keys[i] = 1;
                } else {
                    cpu.keys[i] = 0;
                }
            }
        }
        // (1 second / 60 = .016) emulate 60hz
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
        // update timers
        cpu.sub_dt();
        cpu.sub_st();
    }
}
