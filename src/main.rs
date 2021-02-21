extern crate minifb;
mod cpu;
use crate::cpu::CPU;

fn main () {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Missing file path");
    }
    let mut cpu = CPU::new();
    cpu.initialize();
    cpu.load_rom(&args[1]);
    cpu.emulate();
    let mut window = match minifb::Window::new("rust chip-8", 64, 32, minifb::WindowOptions::default()) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
     };
}