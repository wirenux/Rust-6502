mod bus;
mod cpu;
mod opcodes;

use bus::Bus;
use cpu::CPU;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    let file_byte = std::fs::read("build/asm/program.bin").expect("Failed to read program.bin");

    bus.load_program(0x8000, &file_byte);

    println!("ADDR  HEX       DISASM       | AC XR YR SP | NVDIZC | #");
    println!("-------------------------------------------------------");

    cpu.reset_cpu(&bus);

    loop {
        cpu.clock_tick(&mut bus);

        if cpu.halted {
            break;
        }
    }
}