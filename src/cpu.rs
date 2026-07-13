use core::panic;

use crate::bus::Bus;

pub struct CPU {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub sp: u8,
    pub pc: u16,
    pub sr: u8,

    pub last_instr_bytes: String,
    pub last_disasm: String,
    pub last_cycles: u8,
}

pub enum AddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
    Implied
}

impl CPU {
    pub const ZERO_FLAG: u8 = 0x02;
    pub const CARRY_FLAG: u8 = 0x01;
    pub const NEGATIVE_FLAG: u8 = 0x80;
    pub const OVERFLOW_FLAG: u8 = 0x40;

    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            sp: 0,
            pc: 0,
            sr: 0,

            last_instr_bytes: String::new(),
            last_disasm: String::new(),
            last_cycles: 0,
        }
    }

    fn set_instr(&mut self, bytes: String, disasm: String, cycles: u8) {
        self.last_instr_bytes = bytes;
        self.last_disasm = disasm;
        self.last_cycles = cycles;
    }

    pub fn reset_cpu(&mut self, bus: &Bus) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.sp = 0xFD; // to mimic 3 phantom cycle in the real chip
        self.sr = 0x24;

        let low_byte = bus.read_ram(0xFFFC);
        let high_byte = bus.read_ram(0xFFFD);
        self.pc = ((high_byte as u16) << 8) | (low_byte as u16); // as u16 transform a u8 var into a u16
    }

    pub fn update_z_n_flags(&mut self, target_value : u8) {
        if target_value == 0 {
            self.sr = self.sr | 0x02;
        } else {
            self.sr = self.sr & 0xFD;
        }

        if (target_value & CPU::NEGATIVE_FLAG) != 0 {
            self.sr = self.sr | CPU::NEGATIVE_FLAG;
        } else {
            self.sr = self.sr & 0x7F;
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode, bus: &mut Bus) -> u16 {
        match mode {
            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc + 1;
                addr
            },
            AddressingMode::ZeroPage => {
                let addr = bus.read_ram(self.pc) as u16;
                self.pc = self.pc + 1;
                addr
            },
            AddressingMode::Absolute => {
                let low = bus.read_ram(self.pc) as u16;
                let high = bus.read_ram(self.pc + 1) as u16;
                self.pc = self.pc + 2;
                (high << 8) | low
            },
            AddressingMode::Implied => {
                0
            }
        }
    }

    fn push_stack(&mut self, bus: &mut Bus, value: u8) {
        bus.write_ram(0x0100 + self.sp as u16, value);
        self.sp  = self.sp.wrapping_sub(1);
    }

    fn get_flag(&self, mask: u8) -> bool {
        (self.sr & mask) != 0
    }

    fn adc(&mut self, value: u8) {
        let carry = (self.sr & CPU::CARRY_FLAG) as u16;
        let a_u16 = self.reg_a as u16;
        let val_u16 = value as u16;

        let sum = a_u16 + val_u16 + carry;

        if sum > 0xFF {
            self.sr |= CPU::CARRY_FLAG;
        } else {
            self.sr &= !CPU::CARRY_FLAG;
        }

        let result = (sum & 0xFF) as u8;

        let overflow = (!((self.reg_a ^ value) as u16) & ((self.reg_a as u16 ^ result as u16)) & 0x80) != 0;

        if overflow {
            self.sr |= CPU::OVERFLOW_FLAG;
        } else {
            self.sr &= !CPU::OVERFLOW_FLAG;
        }

        self.reg_a = result;
        self.update_z_n_flags(self.reg_a);
    }

    fn adc_immediate(&mut self, bus: &mut Bus, opcode: u8) {
        let value = bus.read_ram(self.pc);
        self.pc += 1;
        self.adc(value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("ADC #${:02X}", value), 2);
    }

    fn adc_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        let value = bus.read_ram(addr);
        self.adc(value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("ADC ${:02X}", value), 2);
    }

    fn beq(&mut self, bus: &mut Bus, opcode: u8) {
        let offset = bus.read_ram(self.pc) as i8;
        self.pc += 1;

        if self.get_flag(CPU::ZERO_FLAG) {
            self.pc = (self.pc as i16 + offset as i16) as u16;
        }

        self.set_instr(format!("{:02X}", opcode), format!("BEQ"), 2);
    }

    fn bne(&mut self, bus: &mut Bus, opcode: u8) {
        let offset = bus.read_ram(self.pc) as i8;
        self.pc += 1;

        if !self.get_flag(CPU::ZERO_FLAG) {
            self.pc = (self.pc as i16 + offset as i16) as u16;
        }

        self.set_instr(format!("{:02X}", opcode), format!("BNE"), 2);
    }

    fn brk(&mut self, bus: &mut Bus, opcode: u8) {
        self.push_stack(bus, (self.pc >> 8) as u8);
        self.push_stack(bus, (self.pc & 0xFF) as u8);

        self.push_stack(bus, self.sr | 0x10);

        self.sr = self.sr | 0x04;

        let low = bus.read_ram(0xFFFE);
        let high = bus.read_ram(0xFFFF);
        self.pc = ((high as u16) << 8) | (low as u16);

        self.set_instr(format!("{:02X}", opcode), "BRK".to_string(), 7);
    }

    fn clc(&mut self, opcode: u8) {
        self.sr &= !CPU::CARRY_FLAG;

        self.set_instr(format!("{:02X}", opcode), "CLC".to_string(), 2);
    }

    fn dex(&mut self, opcode: u8) {
        self.reg_x = self.reg_x.wrapping_sub(1);
        self.update_z_n_flags(self.reg_x);

        self.set_instr(format!("{:02X}", opcode), "DEX".to_string(), 2);
    }

    fn dey(&mut self, opcode: u8) {
        self.reg_y = self.reg_y.wrapping_sub(1);
        self.update_z_n_flags(self.reg_y);

        self.set_instr(format!("{:02X}", opcode), "DEY".to_string(), 2);
    }

    fn inx(&mut self, opcode: u8) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_z_n_flags(self.reg_x);
        self.set_instr(format!("{:02X}", opcode), "INX".to_string(), 2);
    }

    fn iny(&mut self, opcode: u8) {
        self.reg_y = self.reg_y.wrapping_add(1);
        self.update_z_n_flags(self.reg_y);
        self.set_instr(format!("{:02X}", opcode), "INY".to_string(), 2);
    }

    fn jmp_absolute(&mut self, bus: &mut Bus, opcode: u8) {
        let target_addr = self.get_operand_address(&AddressingMode::Absolute, bus);

        self.pc = target_addr;

        let low = (target_addr & 0xFF) as u8;
        let high = (target_addr >> 8) as u8;

        self.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("JMP ${:04X}", target_addr), 3);
    }

    fn jmp_indirect(&mut self, bus: &mut Bus, opcode: u8) {
        let ptr = self.get_operand_address(&AddressingMode::Absolute, bus);

        let low = bus.read_ram(ptr) as u16;
        let high = if (ptr & 0x00FF) == 0x00FF {
            bus.read_ram(ptr & 0xFF00) as u16
        } else {
            bus.read_ram(ptr + 1) as u16
        };

        let target_addr = (high << 8) | low;

        self.pc = target_addr; // do the jump

        let ptr_low = (ptr & 0xFF) as u8;
        let ptr_high = (ptr >> 8) as u8;

        self.set_instr(format!("{:02X} {:02X} {:02X}", opcode, ptr_low, ptr_high), format!("JMP (${:04X})", ptr), 5);
    }

    fn lda_abosulte(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Absolute, bus);
        let value = bus.read_ram(addr);

        self.reg_a = value;
        self.update_z_n_flags(self.reg_a);

        let low = (addr & 0xFF) as u8;
        let high = (addr >> 8) as u8;

        self.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("LDA ${:04X}", addr), 4);
    }

    fn lda_immediate(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Immediate, bus);
        let value = bus.read_ram(addr);

        self.reg_a = value;
        self.update_z_n_flags(value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("LDA #${:04X}", addr), 2);
    }

    fn lda_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        let value = bus.read_ram(addr);

        self.reg_a = value;
        self.update_z_n_flags(self.reg_a);

        let op_byte = addr as u8;

        self.set_instr(format!("{:02X} {:02X}", opcode, op_byte), format!("LDA ${:04X}", addr), 3);
    }

    fn ldx_abosulte(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Absolute, bus);
        let value = bus.read_ram(addr);

        self.reg_x = value;
        self.update_z_n_flags(self.reg_x);

        let low = (addr & 0xFF) as u8;
        let high = (addr >> 8) as u8;

        self.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("LDX ${:04X}", addr), 4);
    }

    fn ldx_immediate(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Immediate, bus);
        let value = bus.read_ram(addr);

        self.reg_x = value;
        self.update_z_n_flags(value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("LDX #${:04X}", addr), 2);
    }

    fn ldx_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        let value = bus.read_ram(addr);

        self.reg_x = value;
        self.update_z_n_flags(self.reg_x);

        let op_byte = addr as u8;

        self.set_instr(format!("{:02X} {:02X}", opcode, op_byte), format!("LDX ${:04X}", addr), 3);
    }

    fn ldy_abosulte(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Absolute, bus);
        let value = bus.read_ram(addr);

        self.reg_y = value;
        self.update_z_n_flags(self.reg_y);

        let low = (addr & 0xFF) as u8;
        let high = (addr >> 8) as u8;

        self.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("LDY ${:04X}", addr), 4);
    }

    fn ldy_immediate(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Immediate, bus);
        let value = bus.read_ram(addr);

        self.reg_y = value;
        self.update_z_n_flags(value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("LDY #${:04X}", addr), 2);
    }

    fn ldy_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        let value = bus.read_ram(addr);

        self.reg_y = value;
        self.update_z_n_flags(self.reg_y);

        let op_byte = addr as u8;

        self.set_instr(format!("{:02X} {:02X}", opcode, op_byte), format!("LDY ${:04X}", addr), 3);
    }

    fn nop(&mut self, opcode: u8) {
        self.set_instr(format!("{:02X}", opcode), "NOP".to_string(), 2);
    }

    fn sbc_immediate(&mut self, bus: &mut Bus, opcode: u8) {
        let value = bus.read_ram(self.pc);
        self.pc += 1;
        let inverted_value = value ^ 0xFF;
        self.adc(inverted_value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("SBC #${:02X}", value), 2);
    }

    fn sbc_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        let value = bus.read_ram(addr);
        let inverted_value = value ^ 0xFF;
        self.adc(inverted_value);

        self.set_instr(format!("{:02X} {:02X}", opcode, value), format!("SBC ${:02X}", value), 2);
    }

    fn sec(&mut self, opcode: u8) {
        self.sr |= CPU::CARRY_FLAG;
        self.set_instr(format!("{:02X}", opcode), "SEC".to_string(), 2);
    }

    fn sta_absolute(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::Absolute, bus);
        bus.write_ram(addr, self.reg_a);

        let low = (addr & 0xFF) as u8;
        let high = (addr >> 8) as u8;

        self.set_instr(format!("{:02X} {:02X} {:02X}", opcode, low, high), format!("STA ${:04X}", addr), 4);
    }

    fn sta_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        bus.write_ram(addr, self.reg_a);

        let op_byte = addr as u8;

        self.set_instr(
            format!("{:02X} {:02X}", opcode, op_byte),
            format!("STA ${:02X}", op_byte),
            3
        );
    }

    fn stx_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        bus.write_ram(addr, self.reg_x);

        let op_byte = addr as u8;

        self.set_instr(format!("{:02X} {:02X}", opcode, op_byte),format!("STX ${:02X}", op_byte),3);
    }

    fn sty_zeropage(&mut self, bus: &mut Bus, opcode: u8) {
        let addr = self.get_operand_address(&AddressingMode::ZeroPage, bus);
        bus.write_ram(addr, self.reg_y);

        let op_byte = addr as u8;

        self.set_instr(format!("{:02X} {:02X}", opcode, op_byte),format!("STY ${:02X}", op_byte),3);
    }

    fn tax(&mut self, opcode: u8) {
        self.reg_x = self.reg_a;
        self.update_z_n_flags(self.reg_x);

        self.set_instr(format!("{:02X}", opcode), "TAX".to_string(), 2);
    }

    fn tay(&mut self, opcode: u8) {
        self.reg_y = self.reg_a;
        self.update_z_n_flags(self.reg_y);

        self.set_instr(format!("{:02X}", opcode), "TAY".to_string(), 2);
    }

    fn txa(&mut self, opcode: u8) {
        self.reg_a = self.reg_x;
        self.update_z_n_flags(self.reg_a);

        self.set_instr(format!("{:02X}", opcode), "TXA".to_string(), 2);
    }

    fn tya(&mut self, opcode: u8) {
        self.reg_a = self.reg_y;
        self.update_z_n_flags(self.reg_a);

        self.set_instr(format!("{:02X}", opcode), "TYA".to_string(), 2);
    }

    pub fn clock_tick(&mut self, bus: &mut Bus) -> bool {
        let initial_pc = self.pc;
        let opcode = bus.read_ram(self.pc);
        self.pc = self.pc + 1;

        let mut keep_running = true;

        match opcode {
            0x00 => {
                self.brk(bus, opcode);
                keep_running = false
            },
            0x18 => self.clc(opcode),
            0x38 => self.sec(opcode),
            0x4C => self.jmp_absolute(bus, opcode),
            0x65 => self.adc_zeropage(bus, opcode),
            0x69 => self.adc_immediate(bus, opcode),
            0x6C => self.jmp_indirect(bus, opcode),
            0x84 => self.sty_zeropage(bus, opcode),
            0x85 => self.sta_zeropage(bus, opcode),
            0x86 => self.stx_zeropage(bus, opcode),
            0x88 => self.dey(opcode),
            0x8A => self.txa(opcode),
            0x8D => self.sta_absolute(bus, opcode),
            0x98 => self.tya(opcode),
            0xA0 => self.ldy_immediate(bus, opcode),
            0xA2 => self.ldx_immediate(bus, opcode),
            0xA4 => self.ldy_zeropage(bus, opcode),
            0xA5 => self.lda_zeropage(bus, opcode),
            0xA6 => self.ldx_zeropage(bus, opcode),
            0xA8 => self.tay(opcode),
            0xA9 => self.lda_immediate(bus, opcode),
            0xAA => self.tax(opcode),
            0xAC => self.ldy_abosulte(bus, opcode),
            0xAD => self.lda_abosulte(bus, opcode),
            0xAE => self.ldx_abosulte(bus, opcode),
            0xC8 => self.iny(opcode),
            0xCA => self.dex(opcode),
            0xD0 => self.bne(bus, opcode),
            0xE5 => self.sbc_zeropage(bus, opcode),
            0xE8 => self.inx(opcode),
            0xE9 => self.sbc_immediate(bus, opcode),
            0xEA => self.nop(opcode),
            0xF0 => self.beq(bus, opcode),
            _ => {
                panic!("Unknow opcode: {:#X} @ {:#X}", opcode, self.pc - 1);
            }
        }

        let n = (self.sr >> 7) & 1;
        let v = (self.sr >> 6) & 1;
        let d = (self.sr >> 3) & 1;
        let i = (self.sr >> 2) & 1;
        let z = (self.sr >> 1) & 1;
        let c = (self.sr >> 0) & 1;
        let nvdizc_str = format!("{}{}{}{}{}{}", n, v, d, i, z, c);

        println!(
            "{:04X}  {:<8}  {:<12} | {:02X} {:02X} {:02X} {:02X} | {} | {}",
            initial_pc,
            self.last_instr_bytes,
            self.last_disasm,
            self.reg_a, self.reg_x, self.reg_y, self.sp,
            nvdizc_str,
            self.last_cycles
        );

        keep_running
    }
}