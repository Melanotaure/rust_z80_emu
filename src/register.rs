pub union Register {
    reg16: u16,
    reg8: [u8; 2],
}

impl Register {
    pub fn new() -> Self {
        Self { reg16: 0 }
    }

    pub fn get_reg16(&self) -> u16 {
        unsafe { self.reg16 }
    }

    pub fn get_reg8_h(&self) -> u8 {
        unsafe { self.reg8[1] }
    }

    pub fn get_reg8_l(&self) -> u8 {
        unsafe { self.reg8[0] }
    }

    pub fn set_reg16(&mut self, value: u16) {
        self.reg16 = value;
    }

    pub fn set_reg8_h(&mut self, value: u8) {
        unsafe {
            self.reg8[1] = value;
        }
    }

    pub fn set_reg8_l(&mut self, value: u8) {
        unsafe {
            self.reg8[0] = value;
        }
    }

    pub fn add_r16_r16(&mut self, rh: &Register) {
        self.set_reg16(self.get_reg16() + rh.get_reg16());
    }

    pub fn add_r16_i8(&mut self, rh: u8) {
        self.set_reg16(self.get_reg16() + rh as u16);
    }

    pub fn inc(&mut self) {
        self.add_r16_i8(1_u8);
    }
}
