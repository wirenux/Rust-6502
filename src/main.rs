use std::env;

mod bus;
mod cpu;
mod opcodes;
mod tui;
mod disasm;

use bus::Bus;
use cpu::CPU;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).expect("Usage: rust6502 <file> [origin]");

    let origin_str = args.get(2).map(|s| s.as_str()).unwrap_or("8000");
    let origin = u16::from_str_radix(origin_str, 16).expect("Origin must be a hex value");

    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    let file_byte = std::fs::read(file_path).expect("Failed to read file");
    bus.load_rom(&file_byte, origin);

    cpu.reset_cpu(&bus);

    let _ = tui::run(&mut cpu, &mut bus, origin, file_path);
}