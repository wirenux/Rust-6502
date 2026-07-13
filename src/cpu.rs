use core::panic;
use crate::bus::Bus;
use crate::opcodes;

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
    ZeroPageX,
    Absolute,
    AbsoluteX,
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

    pub(crate) fn set_instr(&mut self, bytes: String, disasm: String, cycles: u8) {
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

    pub(crate) fn get_operand_address(&mut self, mode: &AddressingMode, bus: &mut Bus) -> u16 {
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
            AddressingMode::ZeroPageX => {
                let base = bus.read_ram(self.pc);

                let addr = base.wrapping_add(self.reg_x) as u16;

                self.pc = self.pc + 1;
                addr
            },
            AddressingMode::Absolute => {
                let low = bus.read_ram(self.pc) as u16;
                let high = bus.read_ram(self.pc + 1) as u16;
                self.pc = self.pc + 2;
                (high << 8) | low
            },
            AddressingMode::AbsoluteX => {
                let low = bus.read_ram(self.pc) as u16;
                let high = bus.read_ram(self.pc + 1) as u16;
                let base = (high << 8) | low;

                let addr = base.wrapping_add(self.reg_x as u16);

                self.pc = self.pc + 2;
                addr
            },
            AddressingMode::Implied => {
                0
            }
        }
    }

    pub(crate) fn push_stack(&mut self, bus: &mut Bus, value: u8) {
        bus.write_ram(0x0100 + self.sp as u16, value);
        self.sp  = self.sp.wrapping_sub(1);
    }

    pub(crate) fn pop_stack(&mut self, bus: &Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read_ram(0x0100 + self.sp as u16)
    }

    pub(crate) fn get_flag(&self, mask: u8) -> bool {
        (self.sr & mask) != 0
    }

    pub(crate) fn compare_registers(&mut self, register_value: u8, memory_value: u8) {
        if register_value >= memory_value {
            self.sr |= CPU::CARRY_FLAG;
        } else {
            self.sr &= !CPU::CARRY_FLAG;
        }

        let temp_result = register_value.wrapping_sub(memory_value);
        self.update_z_n_flags(temp_result);
    }

    pub(crate) fn adc(&mut self, value: u8) {
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

    pub fn clock_tick(&mut self, bus: &mut Bus) -> bool {
        let initial_pc = self.pc;
        let opcode = bus.read_ram(self.pc);
        self.pc = self.pc + 1;

        let mut keep_running = true;

        match opcode {
            0x00 => {
                opcodes::brk(self, bus, opcode);
                keep_running = false;
            },
            0x09 => opcodes::ora_immediate(self, bus, opcode),
            0x0A => opcodes::asl_accumulator(self, opcode),
            0x18 => opcodes::clc(self, opcode),
            0x20 => {
                let addr = self.get_operand_address(&AddressingMode::Absolute, bus);
                opcodes::jsr(self, bus, opcode, addr);
            },
            0x29 => opcodes::and_immediate(self, bus, opcode),
            0x38 => opcodes::sec(self, opcode),
            0x48 => opcodes::pha(self, bus, opcode),
            0x49 => opcodes::eor_immediate(self, bus, opcode),
            0x4C => opcodes::jmp_absolute(self, bus, opcode),
            0x4A => opcodes::lsr_accumulator(self, opcode),
            0x60 => opcodes::rts(self, bus, opcode),
            0x65 => opcodes::adc_zeropage(self, bus, opcode),
            0x68 => opcodes::pla(self, bus, opcode),
            0x69 => opcodes::adc_immediate(self, bus, opcode),
            0x6C => opcodes::jmp_indirect(self, bus, opcode),
            0x84 => opcodes::sty_zeropage(self, bus, opcode),
            0x85 => opcodes::sta_zeropage(self, bus, opcode),
            0x86 => opcodes::stx_zeropage(self, bus, opcode),
            0x88 => opcodes::dey(self, opcode),
            0x8A => opcodes::txa(self, opcode),
            0x8D => opcodes::sta_absolute(self, bus, opcode),
            0x98 => opcodes::tya(self, opcode),
            0xA0 => opcodes::ldy_immediate(self, bus, opcode),
            0xA2 => opcodes::ldx_immediate(self, bus, opcode),
            0xA4 => opcodes::ldy_zeropage(self, bus, opcode),
            0xA5 => opcodes::lda_zeropage(self, bus, opcode),
            0xA6 => opcodes::ldx_zeropage(self, bus, opcode),
            0xA8 => opcodes::tay(self, opcode),
            0xA9 => opcodes::lda_immediate(self, bus, opcode),
            0xAA => opcodes::tax(self, opcode),
            0xAC => opcodes::ldy_absolute(self, bus, opcode),
            0xAD => opcodes::lda_absolute(self, bus, opcode),
            0xAE => opcodes::ldx_absolute(self, bus, opcode),
            0xB5 => opcodes::lda_zeropage_x(self, bus, opcode),
            0xC0 => opcodes::cpy_immediate(self, bus, opcode),
            0xC8 => opcodes::iny(self, opcode),
            0xC9 => opcodes::cmp_immediate(self, bus, opcode),
            0xCA => opcodes::dex(self, opcode),
            0xD0 => opcodes::bne(self, bus, opcode),
            0xE0 => opcodes::cpx_immediate(self, bus, opcode),
            0xE5 => opcodes::sbc_zeropage(self, bus, opcode),
            0xE8 => opcodes::inx(self, opcode),
            0xE9 => opcodes::sbc_immediate(self, bus, opcode),
            0xEA => opcodes::nop(self, opcode),
            0xF0 => opcodes::beq(self, bus, opcode),
            _ => {
                panic!("Unknown opcode: {:#X} @ {:#X}", opcode, self.pc - 1);
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