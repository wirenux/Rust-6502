pub struct Bus {
    pub ram: [u8; 65536],
    pub irq_active: bool,
    pub nmi_active: bool,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 65536],
            irq_active: false,
            nmi_active: false,
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        return self.ram[addr as usize];
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        if addr == 0xBFFC {
            self.irq_active = (data & 0b0000_0001) != 0;
            self.nmi_active = (data & 0b0000_0010) != 0;
        }

        self.ram[addr as usize] = data;
    }

    pub fn load_rom(&mut self, rom: &[u8], origin: u16) {
        let start = origin as usize;
        for (i, &byte) in rom.iter().enumerate() {
            if start + i < 65536 {
                self.ram[start + i] = byte;
            }
        }
    }

    #[allow(dead_code)]
    pub fn memory_dump(&self, page: u8) {
        let start_addr = (page as u16) << 8;

        println!("\n=== MEMORY DUMP: Page {:02X} ({:04X} - {:04X}) ===", page, start_addr, start_addr + 0x00FF);
        println!("      00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
        println!("      -----------------------------------------------");

        for row in 0..16 {
            let row_start = start_addr + (row * 16);
            print!("{:04X}: ", row_start);

            for col in 0..16 {
                print!("{:02X} ", self.ram[(row_start + col) as usize]);
            }
            println!();
        }
        println!("===================================================\n");
    }
}