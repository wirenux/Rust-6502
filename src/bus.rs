pub struct Bus {
    pub ram: [u8; 65536],
}

impl Bus {
    pub fn read_ram(&self, addr: u16) -> u8 {
        return self.ram[addr as usize];
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}