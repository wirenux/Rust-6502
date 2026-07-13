use std::ptr::addr_of;

use crate::bus::Bus;
use crate::cpu::{AddressingMode, CPU};

pub fn adc_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let value = bus.read_ram(cpu.pc);
    cpu.pc += 1;
    cpu.adc(value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("ADC #${:02X}", value), 2);
}

pub fn adc_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    let value = bus.read_ram(addr);
    cpu.adc(value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("ADC ${:02X}", value), 2);
}

pub fn and_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = cpu.reg_a & value;

    cpu.update_z_n_flags(cpu.reg_a);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("AND #${:02X}", value), 2);
}

pub fn beq(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let offset = bus.read_ram(cpu.pc) as i8;
    cpu.pc += 1;

    if cpu.get_flag(CPU::ZERO_FLAG) {
        cpu.pc = (cpu.pc as i16 + offset as i16) as u16;
    }

    cpu.set_instr(format!("{:02X}", opcode), format!("BEQ"), 2);
}

pub fn bne(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let offset = bus.read_ram(cpu.pc) as i8;
    cpu.pc += 1;

    if !cpu.get_flag(CPU::ZERO_FLAG) {
        cpu.pc = (cpu.pc as i16 + offset as i16) as u16;
    }

    cpu.set_instr(format!("{:02X}", opcode), format!("BNE"), 2);
}

pub fn brk(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    cpu.push_stack(bus, (cpu.pc >> 8) as u8);
    cpu.push_stack(bus, (cpu.pc & 0xFF) as u8);

    cpu.push_stack(bus, cpu.sr | 0x10);

    cpu.sr = cpu.sr | 0x04;

    let low = bus.read_ram(0xFFFE);
    let high = bus.read_ram(0xFFFF);
    cpu.pc = ((high as u16) << 8) | (low as u16);

    cpu.set_instr(format!("{:02X}", opcode), "BRK".to_string(), 7);
}

pub fn clc(cpu: &mut CPU, opcode: u8) {
    cpu.sr &= !CPU::CARRY_FLAG;

    cpu.set_instr(format!("{:02X}", opcode), "CLC".to_string(), 2);
}

pub fn cmp_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.compare_registers(cpu.reg_a, value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("CMP #${:02X}", value), 2);
}

pub fn cpx_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.compare_registers(cpu.reg_x, value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("CPX #${:02X}", value), 2);
}

pub fn cpy_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.compare_registers(cpu.reg_y, value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("CPY #${:02X}", value), 2);
}

pub fn dex(cpu: &mut CPU, opcode: u8) {
    cpu.reg_x = cpu.reg_x.wrapping_sub(1);
    cpu.update_z_n_flags(cpu.reg_x);

    cpu.set_instr(format!("{:02X}", opcode), "DEX".to_string(), 2);
}

pub fn dey(cpu: &mut CPU, opcode: u8) {
    cpu.reg_y = cpu.reg_y.wrapping_sub(1);
    cpu.update_z_n_flags(cpu.reg_y);

    cpu.set_instr(format!("{:02X}", opcode), "DEY".to_string(), 2);
}

pub fn eor_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = cpu.reg_a ^ value;

    cpu.update_z_n_flags(cpu.reg_a);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("EOR #${:02X}", value), 2);
}

pub fn inx(cpu: &mut CPU, opcode: u8) {
    cpu.reg_x = cpu.reg_x.wrapping_add(1);
    cpu.update_z_n_flags(cpu.reg_x);
    cpu.set_instr(format!("{:02X}", opcode), "INX".to_string(), 2);
}

pub fn iny(cpu: &mut CPU, opcode: u8) {
    cpu.reg_y = cpu.reg_y.wrapping_add(1);
    cpu.update_z_n_flags(cpu.reg_y);
    cpu.set_instr(format!("{:02X}", opcode), "INY".to_string(), 2);
}

pub fn jmp_absolute(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let target_addr = cpu.get_operand_address(&AddressingMode::Absolute, bus);

    cpu.pc = target_addr;

    let low = (target_addr & 0xFF) as u8;
    let high = (target_addr >> 8) as u8;

    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("JMP ${:04X}", target_addr), 3);
}

pub fn jmp_indirect(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let ptr = cpu.get_operand_address(&AddressingMode::Absolute, bus);

    let low = bus.read_ram(ptr) as u16;
    let high = if (ptr & 0x00FF) == 0x00FF {
        bus.read_ram(ptr & 0xFF00) as u16
    } else {
        bus.read_ram(ptr + 1) as u16
    };

    let target_addr = (high << 8) | low;

    cpu.pc = target_addr; // do the jump

    let ptr_low = (ptr & 0xFF) as u8;
    let ptr_high = (ptr >> 8) as u8;

    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, ptr_low, ptr_high), format!("JMP (${:04X})", ptr), 5);
}

pub fn jsr(cpu: &mut CPU, bus: &mut Bus, opcode: u8, target_addr: u16) {
    let return_addr = cpu.pc.wrapping_sub(1);

    cpu.push_stack(bus, (return_addr >> 8) as u8);
    cpu.push_stack(bus, (return_addr & 0xFF) as u8);

    cpu.pc = target_addr;

    let low = (target_addr & 0xFF) as u8;
    let high = (target_addr >> 8) as u8;
    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("JSR ${:04X}", target_addr), 6);
}

pub fn lda_absolute(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Absolute, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = value;
    cpu.update_z_n_flags(cpu.reg_a);

    let low = (addr & 0xFF) as u8;
    let high = (addr >> 8) as u8;

    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("LDA ${:04X}", addr), 4);
}

pub fn lda_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = value;
    cpu.update_z_n_flags(value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("LDA #${:04X}", addr), 2);
}

pub fn lda_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = value;
    cpu.update_z_n_flags(cpu.reg_a);

    let op_byte = addr as u8;

    cpu.set_instr(format!("{:02X} {:02X}", opcode, op_byte), format!("LDA ${:04X}", addr), 3);
}

pub fn lda_zeropage_x(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage_X, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = value;
    cpu.update_z_n_flags(cpu.reg_a);

    let base_addr: u8 = bus.read_ram(cpu.pc - 1);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, base_addr), format!("LDA ${:02X},X", base_addr), 4);
}

pub fn ldx_absolute(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Absolute, bus);
    let value = bus.read_ram(addr);

    cpu.reg_x = value;
    cpu.update_z_n_flags(cpu.reg_x);

    let low = (addr & 0xFF) as u8;
    let high = (addr >> 8) as u8;

    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("LDX ${:04X}", addr), 4);
}

pub fn ldx_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.reg_x = value;
    cpu.update_z_n_flags(value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("LDX #${:04X}", addr), 2);
}

pub fn ldx_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    let value = bus.read_ram(addr);

    cpu.reg_x = value;
    cpu.update_z_n_flags(cpu.reg_x);

    let op_byte = addr as u8;

    cpu.set_instr(format!("{:02X} {:02X}", opcode, op_byte), format!("LDX ${:04X}", addr), 3);
}

pub fn ldy_absolute(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Absolute, bus);
    let value = bus.read_ram(addr);

    cpu.reg_y = value;
    cpu.update_z_n_flags(cpu.reg_y);

    let low = (addr & 0xFF) as u8;
    let high = (addr >> 8) as u8;

    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("LDY ${:04X}", addr), 4);
}

pub fn ldy_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.reg_y = value;
    cpu.update_z_n_flags(value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("LDY #${:04X}", addr), 2);
}

pub fn ldy_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    let value = bus.read_ram(addr);

    cpu.reg_y = value;
    cpu.update_z_n_flags(cpu.reg_y);

    let op_byte = addr as u8;

    cpu.set_instr(format!("{:02X} {:02X}", opcode, op_byte), format!("LDY ${:04X}", addr), 3);
}

pub fn nop(cpu: &mut CPU, opcode: u8) {
    cpu.set_instr(format!("{:02X}", opcode), "NOP".to_string(), 2);
}

pub fn ora_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Immediate, bus);
    let value = bus.read_ram(addr);

    cpu.reg_a = cpu.reg_a | value;

    cpu.update_z_n_flags(cpu.reg_a);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("ORA #${:02X}", value), 2);
}

pub fn rts(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let low = cpu.pop_stack(bus) as u16;
    let high = cpu.pop_stack(bus) as u16;

    let return_addr = (high << 8) | low;

    cpu.pc = return_addr + 1;

    cpu.set_instr(format!("{:02X}", opcode), "RTS".to_string(), 6);
}

pub fn sbc_immediate(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let value = bus.read_ram(cpu.pc);
    cpu.pc += 1;
    let inverted_value = value ^ 0xFF;
    cpu.adc(inverted_value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("SBC #${:02X}", value), 2);
}

pub fn sbc_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    let value = bus.read_ram(addr);
    let inverted_value = value ^ 0xFF;
    cpu.adc(inverted_value);

    cpu.set_instr(format!("{:02X} {:02X}", opcode, value), format!("SBC ${:02X}", value), 2);
}

pub fn sec(cpu: &mut CPU, opcode: u8) {
    cpu.sr |= CPU::CARRY_FLAG;
    cpu.set_instr(format!("{:02X}", opcode), "SEC".to_string(), 2);
}

pub fn sta_absolute(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::Absolute, bus);
    bus.write_ram(addr, cpu.reg_a);

    let low = (addr & 0xFF) as u8;
    let high = (addr >> 8) as u8;

    cpu.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("STA ${:04X}", addr), 4);
}

pub fn sta_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    bus.write_ram(addr, cpu.reg_a);

    let op_byte = addr as u8;

    cpu.set_instr(
        format!("{:02X} {:02X}", opcode, op_byte),
        format!("STA ${:02X}", op_byte),
        3
    );
}

pub fn stx_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    bus.write_ram(addr, cpu.reg_x);

    let op_byte = addr as u8;

    cpu.set_instr(format!("{:02X} {:02X}", opcode, op_byte),format!("STX ${:02X}", op_byte),3);
}

pub fn sty_zeropage(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    let addr = cpu.get_operand_address(&AddressingMode::ZeroPage, bus);
    bus.write_ram(addr, cpu.reg_y);

    let op_byte = addr as u8;

    cpu.set_instr(format!("{:02X} {:02X}", opcode, op_byte),format!("STY ${:02X}", op_byte),3);
}

pub fn tax(cpu: &mut CPU, opcode: u8) {
    cpu.reg_x = cpu.reg_a;
    cpu.update_z_n_flags(cpu.reg_x);

    cpu.set_instr(format!("{:02X}", opcode), "TAX".to_string(), 2);
}

pub fn tay(cpu: &mut CPU, opcode: u8) {
    cpu.reg_y = cpu.reg_a;
    cpu.update_z_n_flags(cpu.reg_y);

    cpu.set_instr(format!("{:02X}", opcode), "TAY".to_string(), 2);
}

pub fn txa(cpu: &mut CPU, opcode: u8) {
    cpu.reg_a = cpu.reg_x;
    cpu.update_z_n_flags(cpu.reg_a);

    cpu.set_instr(format!("{:02X}", opcode), "TXA".to_string(), 2);
}

pub fn tya(cpu: &mut CPU, opcode: u8) {
    cpu.reg_a = cpu.reg_y;
    cpu.update_z_n_flags(cpu.reg_a);

    cpu.set_instr(format!("{:02X}", opcode), "TYA".to_string(), 2);
}