const MEMORY_SIZE: usize = 0xFFFF;

pub struct Bus {
    memory: [u8; MEMORY_SIZE],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: [0_u8; MEMORY_SIZE],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn reset(&mut self) {
        self.memory.fill(0_u8);
    }
}

pub fn read_io(addr: u16) -> u8 {
    addr as u8
}

pub fn write_io(addr: u16, data: u8) {
    let _d = addr + data as u16;
}
