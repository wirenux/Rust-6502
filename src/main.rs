mod bus;
mod cpu;

use bus::Bus;
use cpu::CPU;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = CPU::new();

    bus.write_ram(0xFFFC, 0x00);
    bus.write_ram(0xFFFD, 0x80);

    // LDA #$42
    bus.write_ram(0x8000, 0xA9);
    bus.write_ram(0x8001, 0x42);

    // TAX
    bus.write_ram(0x8002, 0xAA);

    // NOP
    bus.write_ram(0x8003, 0xEA);

    cpu.reset_cpu(&bus);

    cpu.clock_tick(&bus);
    cpu.clock_tick(&bus);
    cpu.clock_tick(&bus);

    println!("Reg A: {:#X}", cpu.reg_a);
    println!("Reg X: {:#X}", cpu.reg_x);
    println!("SR:    {:#X}", cpu.sr);
}