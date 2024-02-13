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

    fn rl_r(&mut self, reg: u8) -> u8 {
        let c = self.regs.flags.c as u8;
        let r = (reg.rotate_left(1) & 0xFE) | c;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn rr_r(&mut self, reg: u8) -> u8 {
        let c = self.regs.flags.c as u8;
        let r = (reg.rotate_right(1) & 0x7F) | c;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn sla_r(&mut self, reg: u8) -> u8 {
        let r = reg << 1;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn sra_r(&mut self, reg: u8) -> u8 {
        let r = ((reg as i8) >> 1) as u8;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn sll_r(&mut self, reg: u8) -> u8 {
        let r = (reg << 1) | 0x01;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn srl_r(&mut self, reg: u8) -> u8 {
        let r = reg >> 1;
        self.regs.flags.s = false;
        self.regs.flags.z = r == 0;
        self.regs.flags.h = false;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn bit_b_r(&mut self, bit: u8, reg: u8) {
        let mask = 0x01_u8 << bit;
        self.regs.flags.z = reg & mask == 0x00;
        self.regs.flags.h = true;
        self.regs.flags.n = false;
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
            // RL r
            0x10 => self.regs.b = self.rl_r(self.regs.b),
            0x11 => self.regs.c = self.rl_r(self.regs.c),
            0x12 => self.regs.d = self.rl_r(self.regs.d),
            0x13 => self.regs.e = self.rl_r(self.regs.e),
            0x14 => self.regs.h = self.rl_r(self.regs.h),
            0x15 => self.regs.l = self.rl_r(self.regs.l),
            0x16 => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.rl_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x17 => self.regs.a = self.rl_r(self.regs.a),
            // RR r
            0x18 => self.regs.b = self.rr_r(self.regs.b),
            0x19 => self.regs.c = self.rr_r(self.regs.c),
            0x1A => self.regs.d = self.rr_r(self.regs.d),
            0x1B => self.regs.e = self.rr_r(self.regs.e),
            0x1C => self.regs.h = self.rr_r(self.regs.h),
            0x1D => self.regs.l = self.rr_r(self.regs.l),
            0x1E => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.rr_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x1F => self.regs.a = self.rr_r(self.regs.a),
            // SLA r
            0x20 => self.regs.b = self.sla_r(self.regs.b),
            0x21 => self.regs.c = self.sla_r(self.regs.c),
            0x22 => self.regs.d = self.sla_r(self.regs.d),
            0x23 => self.regs.e = self.sla_r(self.regs.e),
            0x24 => self.regs.h = self.sla_r(self.regs.h),
            0x25 => self.regs.l = self.sla_r(self.regs.l),
            0x26 => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.sla_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x27 => self.regs.a = self.sla_r(self.regs.a),
            // SRA r
            0x28 => self.regs.b = self.sra_r(self.regs.b),
            0x29 => self.regs.c = self.sra_r(self.regs.c),
            0x2A => self.regs.d = self.sra_r(self.regs.d),
            0x2B => self.regs.e = self.sra_r(self.regs.e),
            0x2C => self.regs.h = self.sra_r(self.regs.h),
            0x2D => self.regs.l = self.sra_r(self.regs.l),
            0x2E => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.sra_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x2F => self.regs.a = self.sra_r(self.regs.a),
            // SLL r (undocumented)
            0x30 => self.regs.b = self.sll_r(self.regs.b),
            0x31 => self.regs.c = self.sll_r(self.regs.c),
            0x32 => self.regs.d = self.sll_r(self.regs.d),
            0x33 => self.regs.e = self.sll_r(self.regs.e),
            0x34 => self.regs.h = self.sll_r(self.regs.h),
            0x35 => self.regs.l = self.sll_r(self.regs.l),
            0x36 => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.sll_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x37 => self.regs.a = self.sll_r(self.regs.a),
            // SRL r
            0x38 => self.regs.b = self.srl_r(self.regs.b),
            0x39 => self.regs.c = self.srl_r(self.regs.c),
            0x3A => self.regs.d = self.srl_r(self.regs.d),
            0x3B => self.regs.e = self.srl_r(self.regs.e),
            0x3C => self.regs.h = self.srl_r(self.regs.h),
            0x3D => self.regs.l = self.srl_r(self.regs.l),
            0x3E => {
                let mut data = self.bus.read(self.regs.get_hl());
                data = self.srl_r(data);
                self.bus.write(self.regs.get_hl(), data);
            }
            0x3F => self.regs.a = self.srl_r(self.regs.a),
            // Bit b, r
            0x40 => self.bit_b_r(0, self.regs.b),
            0x41 => self.bit_b_r(0, self.regs.c),
            0x42 => self.bit_b_r(0, self.regs.d),
            0x43 => self.bit_b_r(0, self.regs.e),
            0x44 => self.bit_b_r(0, self.regs.h),
            0x45 => self.bit_b_r(0, self.regs.l),
            0x46 => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(0, data);
            }
            0x47 => self.bit_b_r(0, self.regs.a),
            0x48 => self.bit_b_r(1, self.regs.b),
            0x49 => self.bit_b_r(1, self.regs.c),
            0x4A => self.bit_b_r(1, self.regs.d),
            0x4B => self.bit_b_r(1, self.regs.e),
            0x4C => self.bit_b_r(1, self.regs.h),
            0x4D => self.bit_b_r(1, self.regs.l),
            0x4E => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(1, data);
            }
            0x4F => self.bit_b_r(1, self.regs.a),
            0x50 => self.bit_b_r(2, self.regs.b),
            0x51 => self.bit_b_r(2, self.regs.c),
            0x52 => self.bit_b_r(2, self.regs.d),
            0x53 => self.bit_b_r(2, self.regs.e),
            0x54 => self.bit_b_r(2, self.regs.h),
            0x55 => self.bit_b_r(2, self.regs.l),
            0x56 => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(2, data);
            }
            0x57 => self.bit_b_r(2, self.regs.a),
            0x58 => self.bit_b_r(3, self.regs.b),
            0x59 => self.bit_b_r(3, self.regs.c),
            0x5A => self.bit_b_r(3, self.regs.d),
            0x5B => self.bit_b_r(3, self.regs.e),
            0x5C => self.bit_b_r(3, self.regs.h),
            0x5D => self.bit_b_r(3, self.regs.l),
            0x5E => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(3, data);
            }
            0x5F => self.bit_b_r(3, self.regs.a),
            0x60 => self.bit_b_r(4, self.regs.b),
            0x61 => self.bit_b_r(4, self.regs.c),
            0x62 => self.bit_b_r(4, self.regs.d),
            0x63 => self.bit_b_r(4, self.regs.e),
            0x64 => self.bit_b_r(4, self.regs.h),
            0x65 => self.bit_b_r(4, self.regs.l),
            0x66 => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(4, data);
            }
            0x67 => self.bit_b_r(4, self.regs.a),
            0x68 => self.bit_b_r(5, self.regs.b),
            0x69 => self.bit_b_r(5, self.regs.c),
            0x6A => self.bit_b_r(5, self.regs.d),
            0x6B => self.bit_b_r(5, self.regs.e),
            0x6C => self.bit_b_r(5, self.regs.h),
            0x6D => self.bit_b_r(5, self.regs.l),
            0x6E => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(5, data);
            }
            0x6F => self.bit_b_r(5, self.regs.a),
            0x70 => self.bit_b_r(6, self.regs.b),
            0x71 => self.bit_b_r(6, self.regs.c),
            0x72 => self.bit_b_r(6, self.regs.d),
            0x73 => self.bit_b_r(6, self.regs.e),
            0x74 => self.bit_b_r(6, self.regs.h),
            0x75 => self.bit_b_r(6, self.regs.l),
            0x76 => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(6, data);
            }
            0x77 => self.bit_b_r(6, self.regs.a),
            0x78 => self.bit_b_r(7, self.regs.b),
            0x79 => self.bit_b_r(7, self.regs.c),
            0x7A => self.bit_b_r(7, self.regs.d),
            0x7B => self.bit_b_r(7, self.regs.e),
            0x7C => self.bit_b_r(7, self.regs.h),
            0x7D => self.bit_b_r(7, self.regs.l),
            0x7E => {
                let data = self.bus.read(self.regs.get_hl());
                self.bit_b_r(7, data);
            }
            0x7F => self.bit_b_r(7, self.regs.a),
            _ => {}
        };

        cycles
    }
}