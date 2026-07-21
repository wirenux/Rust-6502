use std::env;

mod bus;
mod cpu;
mod opcodes;
mod tui;
mod disasm;

use bus::Bus;
use cpu::CPU;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    let file_path = args.get(1).cloned();

    let origin = args
        .get(2)
        .and_then(|s| u16::from_str_radix(s, 16).ok())
        .unwrap_or(0x8000);

    if let Some(ref path) = file_path {
        match std::fs::read(path) {
            Ok(file_bytes) => {
                bus.load_rom(&file_bytes, origin);
                cpu.reset_cpu(&bus);
                cpu.pc = origin;
            }
            Err(err) => {
                eprintln!("Error loading file '{}': {}", path, err);
                std::process::exit(1);
            }
        }
    }

    tui::run(&mut cpu, &mut bus, origin, file_path)?;

    Ok(())
}