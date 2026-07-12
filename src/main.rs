mod bus;
mod cpu;

use bus::Bus;
use cpu::CPU;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    let file_byte = std::fs::read("program.bin").expect("Failed to read program.bin");

    bus.load_program(0x8000, &file_byte);

    bus.write_ram(0xFFFC, 0x00);
    bus.write_ram(0xFFFD, 0x80);

    cpu.reset_cpu(&bus);

    loop {
        cpu.clock_tick(&bus);
    }
}