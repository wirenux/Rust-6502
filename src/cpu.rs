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
        let opcode = bus.read_ram(self.pc);
        self.pc = self.pc + 1;

        match opcode {
            0x00 => {
                println!("Ex: BRK");
                return false;
            },
            0x88 => {
                self.reg_y = self.reg_y - 1;
                self.update_z_n_flags(self.reg_y);
                println!("Ex: DEY");
                return true;
            },
            0x8A => {
                self.reg_a = self.reg_x;
                self.update_z_n_flags(self.reg_a);
                println!("Ex: TXA");
                return true;
            },
            0x98 => {
                self.reg_a = self.reg_y;
                self.update_z_n_flags(self.reg_a);
                println!("Ex: TYA");
                return true;
            },
            0xA8 => {
                self.reg_y = self.reg_a;
                self.update_z_n_flags(self.reg_y);
                println!("Ex: TAY");
                return true;
            },
            0xA9 => {
                let value = bus.read_ram(self.pc);
                self.pc = self.pc + 1;

                self.reg_a = value;
                self.update_z_n_flags(value);
                println!("Ex: LDA {:#X}", value);
                return true;
            },
            0xAA => {
                self.reg_x = self.reg_a;
                self.update_z_n_flags(self.reg_x);
                println!("Ex: TAX");
                return true;
            },
            0xC8 => {
                self.reg_y = self.reg_y + 1;
                self.update_z_n_flags(self.reg_y);
                println!("Ex: INY");
                return true;
            },
            0xCA => {
                self.reg_x = self.reg_x - 1;
                self.update_z_n_flags(self.reg_x);
                println!("Ex: DEX");
                return true;
            },
            0xE8 => {
                self.reg_x = self.reg_x + 1;
                self.update_z_n_flags(self.reg_x);
                println!("Ex: INX");
                return true;
            },
            0xEA => {
                println!("Ex: NOP");
                return true;
            },
            _ => {
                panic!("Unknow opcode: {:#X} @ {:#X}", opcode, self.pc - 1);
            }
        }
    }
}