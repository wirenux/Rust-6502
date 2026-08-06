use std::collections::VecDeque;
use crossterm::event::KeyCode;
use crate::ps2;
use crate::serial::{
    Serial,
    SERIAL_DATA_ADDR,
    SERIAL_STATUS_ADDR
};

pub const KBD_DATA_ADDR: u16 = 0xBFF0;      // read: peak at the next scancode byte
pub const KBD_STATUS_ADDR: u16 = 0xBFF1;    // read: bit0 = byte waiting, bit1 = capture active
pub const KBD_ACK_ADDR: u16 = 0xBFF2;       // write: pop the byte that was just read
pub const KBD_STATUS_DATA_READY: u8 = 0b0000_0001;
pub const KBD_STATUS_CAPUTRE_ACTIVE: u8 = 0b0000_0010;

pub struct Bus {
    pub ram: [u8; 65536],
    pub irq_active: bool,
    pub nmi_active: bool,
    kbd_queue: VecDeque<u8>,
    pub kbd_capture_active: bool,
    pub serial: Serial,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 65536],
            irq_active: false,
            nmi_active: false,
            kbd_queue: VecDeque::new(),
            kbd_capture_active: false,
            serial: Serial::new(8080),
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        match addr {
            KBD_DATA_ADDR => self.kbd_queue.front().copied().unwrap_or(0),

            KBD_STATUS_ADDR => {
                let mut status = 0u8;

                if !self.kbd_queue.is_empty() {
                    status |= KBD_STATUS_DATA_READY;
                }

                if self.kbd_capture_active {
                    status |= KBD_STATUS_CAPUTRE_ACTIVE;
                }

                status
            },
            
            SERIAL_DATA_ADDR | SERIAL_STATUS_ADDR => self.serial.read_register(addr),

            _ => self.ram[addr as usize],
        }
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        if addr == 0xBFFC {
            self.irq_active = (data & 0b0000_0001) != 0;
            self.nmi_active = (data & 0b0000_0010) != 0;
        }

        if addr == KBD_ACK_ADDR {
            self.kbd_queue.pop_front();
        }

        if addr == SERIAL_DATA_ADDR {
            self.serial.write_register(addr, data);
        }

        self.ram[addr as usize] = data;
    }

    pub fn kbd_reset(&mut self) {
        self.kbd_queue.clear();
        self.kbd_capture_active = false;
    }

    pub fn kbd_press(&mut self, key: KeyCode) {
        if let Some(bytes) = ps2::encode_tap(key) {
            self.kbd_queue.extend(bytes);
        }
    }

    pub fn load_rom(&mut self, rom: &[u8], origin: u16) {
        let start = origin as usize;
        for (i, &byte) in rom.iter().enumerate() {
            if start + i < 65536 {
                self.ram[start + i] = byte;
            }
        }
    }
}
