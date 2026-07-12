use crate::bus::Bus;

pub struct CPU {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub sp: u8,
    pub pc: u16,
    pub sr: u8,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            sp: 0,
            pc: 0,
            sr: 0,
        }
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

        if (target_value & 0x80) != 0 {
            self.sr = self.sr | 0x80;
        } else {
            self.sr = self.sr & 0x7F;
        }
    }

    pub fn clock_tick(&mut self, bus: &Bus) -> bool {
        let initial_pc = self.pc;
        let opcode = bus.read_ram(self.pc);
        self.pc = self.pc + 1;

        let mut keep_running = true;
        let instr_bytes: String;
        let disasm_str: String;
        let cycles: u8;

        match opcode {
            0x00 => {
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "BRK".to_string();
                cycles = 7;
                keep_running = false;
            },
            0x88 => {
                self.reg_y = self.reg_y.wrapping_sub(1);
                self.update_z_n_flags(self.reg_y);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "DEY".to_string();
                cycles = 2;
            },
            0x8A => {
                self.reg_a = self.reg_x;
                self.update_z_n_flags(self.reg_a);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "TXA".to_string();
                cycles = 2;
            },
            0x98 => {
                self.reg_a = self.reg_y;
                self.update_z_n_flags(self.reg_a);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "TYA".to_string();
                cycles = 2;
            },
            0xA8 => {
                self.reg_y = self.reg_a;
                self.update_z_n_flags(self.reg_y);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "TAY".to_string();
                cycles = 2;
            },
            0xA9 => {
                let value = bus.read_ram(self.pc);
                self.pc = self.pc + 1;

                self.reg_a = value;
                self.update_z_n_flags(value);
                instr_bytes = format!("{:02X} {:02X}", opcode, value);
                disasm_str = format!("LDA #${:02X}", value);
                cycles = 2;
            },
            0xAA => {
                self.reg_x = self.reg_a;
                self.update_z_n_flags(self.reg_x);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "TAX".to_string();
                cycles = 2;
            },
            0xC8 => {
                self.reg_y = self.reg_y.wrapping_add(1);
                self.update_z_n_flags(self.reg_y);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "INY".to_string();
                cycles = 2;
            },
            0xCA => {
                self.reg_x = self.reg_x.wrapping_sub(1);
                self.update_z_n_flags(self.reg_x);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "DEX".to_string();
                cycles = 2;
            },
            0xE8 => {
                self.reg_x = self.reg_x.wrapping_add(1);
                self.update_z_n_flags(self.reg_x);
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "INX".to_string();
                cycles = 2;
            },
            0xEA => {
                instr_bytes = format!("{:02X}   ", opcode);
                disasm_str = "NOP".to_string();
                cycles = 2;
            },
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
            "{:04X} {:<5}      {:<12} |{:02X} {:02X} {:02X} {:02X}|{}|{}",
            initial_pc,
            instr_bytes,
            disasm_str,
            self.reg_a, self.reg_x, self.reg_y, self.sp,
            nvdizc_str,
            cycles
        );

        keep_running
    }
}