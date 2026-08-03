use std::env;

mod bus;
mod cpu;
mod opcodes;
mod tui;
mod disasm;
mod ps2;

use bus::Bus;
use cpu::Cpu;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui::setup_panic_hook();
    let args: Vec<String> = env::args().collect();

    let mut bus = Bus::new();
    let mut cpu = Cpu::new();

    let file_path = args.get(1).cloned();

    let origin = args
        .get(2)
        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
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
