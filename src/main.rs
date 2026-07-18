use std::{thread, time::Duration};
use std::env;

mod bus;
mod cpu;
mod opcodes;

use bus::Bus;
use cpu::CPU;

const TARGET_HZ: u64 = 1_000_000; // 1 MHz
const NS_PER_CYCLE: u64 = 1_000_000_000 / TARGET_HZ; // nanosecond per cycle

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).expect("Usage: rust6502 <file> [origin]");

    let origin_str = args.get(2).map(|s| s.as_str()).unwrap_or("8000");
    let origin = u16::from_str_radix(origin_str, 16).expect("Origin must be a hex value");

    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    let file_byte = std::fs::read(file_path).expect("Failed to read file");
    bus.load_rom(&file_byte, origin);

    println!("ADDR  HEX       DISASM       | AC XR YR SP | NVDIZC | #");
    println!("-------------------------------------------------------");

    cpu.reset_cpu(&bus);

    loop {
        cpu.clock_tick(&mut bus);

        let delay_ns = NS_PER_CYCLE * cpu.last_cycles as u64;
        thread::sleep(Duration::from_nanos(delay_ns));

        if cpu.halted {
            break;
        }
    }
}