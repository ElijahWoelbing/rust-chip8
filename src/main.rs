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
    let mut window =
        match minifb::Window::new("rust chip-8", SCREEN_WIDTH, SCREEN_HEIGHT, minifb::WindowOptions::default()) {
            Ok(win) => win,
            Err(err) => {
                println!("Unable to create window {}", err);
                return;
            }
        };
    loop {
        let mut cycles = 0;
        while cycles < 8 {
            cpu.cycle();
            cycles +=1;
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
        if cpu.update_screen {
            window.update_with_buffer(cpu.get_sreen_buffer(), SCREEN_WIDTH, SCREEN_HEIGHT);
            cpu.update_screen = false;
        }
        cpu.sub_dt();
        cpu.sub_st();
    }
}
