use crate::{cycles::CYCLES_CB, z80::*};

impl Z80 {
    fn rlc_r(&mut self, reg: u8) -> u8 {
        let r = reg.rotate_left(1);
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn rrc_r(&mut self, reg: u8) -> u8 {
        let r = reg.rotate_right(1);
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x01) == 0x01;
        r
    }

    pub fn cb_instructions(&mut self) -> u8 {
        self.regs.inc_pc();
        let opcode = self.bus.read(self.regs.pc);
        let cycles = CYCLES_CB[opcode as usize];

        match opcode {
            // RLC r
            0x00 => self.regs.b = self.rlc_r(self.regs.b),
            0x01 => self.regs.c = self.rlc_r(self.regs.c),
            0x02 => self.regs.d = self.rlc_r(self.regs.d),
            0x03 => self.regs.e = self.rlc_r(self.regs.e),
            0x04 => self.regs.h = self.rlc_r(self.regs.h),
            0x05 => self.regs.l = self.rlc_r(self.regs.l),
            0x06 => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.rlc_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x07 => self.regs.a = self.rlc_r(self.regs.a),
            // RRC r
            0x08 => self.regs.b = self.rrc_r(self.regs.b),
            0x09 => self.regs.c = self.rrc_r(self.regs.c),
            0x0A => self.regs.d = self.rrc_r(self.regs.d),
            0x0B => self.regs.e = self.rrc_r(self.regs.e),
            0x0C => self.regs.h = self.rrc_r(self.regs.h),
            0x0D => self.regs.l = self.rrc_r(self.regs.l),
            0x0E => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.rrc_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x0F => self.regs.a = self.rrc_r(self.regs.a),
            _ => {}
        };

        cycles
    }
}