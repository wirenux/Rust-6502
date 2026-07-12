pub struct Bus {
    pub ram: [u8; 65536],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 65536],
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        return self.ram[addr as usize];
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    pub fn load_program(&mut self, start_addr: u16, data: &Vec<u8>) {
        let mut current_addr = start_addr as usize;

        for byte in data {
            self.ram[current_addr] = *byte;
            current_addr = current_addr + 1;
        }
    }
}