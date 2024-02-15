use crate::{cycles::CYCLES_CB, z80::*};

impl Z80 {
    fn rlc_r(&mut self, reg: u8) -> u8 {
        let r = reg.rotate_left(1);
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn rrc_r(&mut self, reg: u8) -> u8 {
        let r = reg.rotate_right(1);
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn rl_r(&mut self, reg: u8) -> u8 {
        let c = self.reg.flags.c as u8;
        let r = (reg.rotate_left(1) & 0xFE) | c;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn rr_r(&mut self, reg: u8) -> u8 {
        let c = self.reg.flags.c as u8;
        let r = (reg.rotate_right(1) & 0x7F) | c;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn sla_r(&mut self, reg: u8) -> u8 {
        let r = reg << 1;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn sra_r(&mut self, reg: u8) -> u8 {
        let r = ((reg as i8) >> 1) as u8;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn sll_r(&mut self, reg: u8) -> u8 {
        let r = (reg << 1) | 0x01;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x80) == 0x80;
        r
    }

    fn srl_r(&mut self, reg: u8) -> u8 {
        let r = reg >> 1;
        self.reg.flags.s = false;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (reg & 0x01) == 0x01;
        r
    }

    fn bit_b_r(&mut self, bit: u8, reg: u8) {
        let mask = 0x01_u8 << bit;
        self.reg.flags.z = reg & mask == 0x00;
        self.reg.flags.h = true;
        self.reg.flags.n = false;
    }

    fn res_b_r(&mut self, bit: u8, reg: u8) -> u8 {
        let mask = 0xFE_u8 << bit;
        reg & mask
    }

    fn set_b_r(&mut self, bit: u8, reg: u8) -> u8 {
        let mask = 0x01_u8 << bit;
        reg | mask
    }

    pub fn cb_instructions(&mut self) -> u8 {
        self.reg.inc_pc();
        let opcode = self.bus.read(self.reg.pc);
        let cycles = CYCLES_CB[opcode as usize];

        match opcode {
            // RLC r
            0x00 => self.reg.b = self.rlc_r(self.reg.b),
            0x01 => self.reg.c = self.rlc_r(self.reg.c),
            0x02 => self.reg.d = self.rlc_r(self.reg.d),
            0x03 => self.reg.e = self.rlc_r(self.reg.e),
            0x04 => self.reg.h = self.rlc_r(self.reg.h),
            0x05 => self.reg.l = self.rlc_r(self.reg.l),
            0x06 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.rlc_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x07 => self.reg.a = self.rlc_r(self.reg.a),
            // RRC r
            0x08 => self.reg.b = self.rrc_r(self.reg.b),
            0x09 => self.reg.c = self.rrc_r(self.reg.c),
            0x0A => self.reg.d = self.rrc_r(self.reg.d),
            0x0B => self.reg.e = self.rrc_r(self.reg.e),
            0x0C => self.reg.h = self.rrc_r(self.reg.h),
            0x0D => self.reg.l = self.rrc_r(self.reg.l),
            0x0E => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.rrc_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x0F => self.reg.a = self.rrc_r(self.reg.a),
            // RL r
            0x10 => self.reg.b = self.rl_r(self.reg.b),
            0x11 => self.reg.c = self.rl_r(self.reg.c),
            0x12 => self.reg.d = self.rl_r(self.reg.d),
            0x13 => self.reg.e = self.rl_r(self.reg.e),
            0x14 => self.reg.h = self.rl_r(self.reg.h),
            0x15 => self.reg.l = self.rl_r(self.reg.l),
            0x16 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.rl_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x17 => self.reg.a = self.rl_r(self.reg.a),
            // RR r
            0x18 => self.reg.b = self.rr_r(self.reg.b),
            0x19 => self.reg.c = self.rr_r(self.reg.c),
            0x1A => self.reg.d = self.rr_r(self.reg.d),
            0x1B => self.reg.e = self.rr_r(self.reg.e),
            0x1C => self.reg.h = self.rr_r(self.reg.h),
            0x1D => self.reg.l = self.rr_r(self.reg.l),
            0x1E => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.rr_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x1F => self.reg.a = self.rr_r(self.reg.a),
            // SLA r
            0x20 => self.reg.b = self.sla_r(self.reg.b),
            0x21 => self.reg.c = self.sla_r(self.reg.c),
            0x22 => self.reg.d = self.sla_r(self.reg.d),
            0x23 => self.reg.e = self.sla_r(self.reg.e),
            0x24 => self.reg.h = self.sla_r(self.reg.h),
            0x25 => self.reg.l = self.sla_r(self.reg.l),
            0x26 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.sla_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x27 => self.reg.a = self.sla_r(self.reg.a),
            // SRA r
            0x28 => self.reg.b = self.sra_r(self.reg.b),
            0x29 => self.reg.c = self.sra_r(self.reg.c),
            0x2A => self.reg.d = self.sra_r(self.reg.d),
            0x2B => self.reg.e = self.sra_r(self.reg.e),
            0x2C => self.reg.h = self.sra_r(self.reg.h),
            0x2D => self.reg.l = self.sra_r(self.reg.l),
            0x2E => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.sra_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x2F => self.reg.a = self.sra_r(self.reg.a),
            // SLL r (undocumented)
            0x30 => self.reg.b = self.sll_r(self.reg.b),
            0x31 => self.reg.c = self.sll_r(self.reg.c),
            0x32 => self.reg.d = self.sll_r(self.reg.d),
            0x33 => self.reg.e = self.sll_r(self.reg.e),
            0x34 => self.reg.h = self.sll_r(self.reg.h),
            0x35 => self.reg.l = self.sll_r(self.reg.l),
            0x36 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.sll_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x37 => self.reg.a = self.sll_r(self.reg.a),
            // SRL r
            0x38 => self.reg.b = self.srl_r(self.reg.b),
            0x39 => self.reg.c = self.srl_r(self.reg.c),
            0x3A => self.reg.d = self.srl_r(self.reg.d),
            0x3B => self.reg.e = self.srl_r(self.reg.e),
            0x3C => self.reg.h = self.srl_r(self.reg.h),
            0x3D => self.reg.l = self.srl_r(self.reg.l),
            0x3E => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.srl_r(data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x3F => self.reg.a = self.srl_r(self.reg.a),
            // Bit b, r
            0x40 => self.bit_b_r(0, self.reg.b),
            0x41 => self.bit_b_r(0, self.reg.c),
            0x42 => self.bit_b_r(0, self.reg.d),
            0x43 => self.bit_b_r(0, self.reg.e),
            0x44 => self.bit_b_r(0, self.reg.h),
            0x45 => self.bit_b_r(0, self.reg.l),
            0x46 => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(0, data);
            }
            0x47 => self.bit_b_r(0, self.reg.a),
            0x48 => self.bit_b_r(1, self.reg.b),
            0x49 => self.bit_b_r(1, self.reg.c),
            0x4A => self.bit_b_r(1, self.reg.d),
            0x4B => self.bit_b_r(1, self.reg.e),
            0x4C => self.bit_b_r(1, self.reg.h),
            0x4D => self.bit_b_r(1, self.reg.l),
            0x4E => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(1, data);
            }
            0x4F => self.bit_b_r(1, self.reg.a),
            0x50 => self.bit_b_r(2, self.reg.b),
            0x51 => self.bit_b_r(2, self.reg.c),
            0x52 => self.bit_b_r(2, self.reg.d),
            0x53 => self.bit_b_r(2, self.reg.e),
            0x54 => self.bit_b_r(2, self.reg.h),
            0x55 => self.bit_b_r(2, self.reg.l),
            0x56 => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(2, data);
            }
            0x57 => self.bit_b_r(2, self.reg.a),
            0x58 => self.bit_b_r(3, self.reg.b),
            0x59 => self.bit_b_r(3, self.reg.c),
            0x5A => self.bit_b_r(3, self.reg.d),
            0x5B => self.bit_b_r(3, self.reg.e),
            0x5C => self.bit_b_r(3, self.reg.h),
            0x5D => self.bit_b_r(3, self.reg.l),
            0x5E => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(3, data);
            }
            0x5F => self.bit_b_r(3, self.reg.a),
            0x60 => self.bit_b_r(4, self.reg.b),
            0x61 => self.bit_b_r(4, self.reg.c),
            0x62 => self.bit_b_r(4, self.reg.d),
            0x63 => self.bit_b_r(4, self.reg.e),
            0x64 => self.bit_b_r(4, self.reg.h),
            0x65 => self.bit_b_r(4, self.reg.l),
            0x66 => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(4, data);
            }
            0x67 => self.bit_b_r(4, self.reg.a),
            0x68 => self.bit_b_r(5, self.reg.b),
            0x69 => self.bit_b_r(5, self.reg.c),
            0x6A => self.bit_b_r(5, self.reg.d),
            0x6B => self.bit_b_r(5, self.reg.e),
            0x6C => self.bit_b_r(5, self.reg.h),
            0x6D => self.bit_b_r(5, self.reg.l),
            0x6E => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(5, data);
            }
            0x6F => self.bit_b_r(5, self.reg.a),
            0x70 => self.bit_b_r(6, self.reg.b),
            0x71 => self.bit_b_r(6, self.reg.c),
            0x72 => self.bit_b_r(6, self.reg.d),
            0x73 => self.bit_b_r(6, self.reg.e),
            0x74 => self.bit_b_r(6, self.reg.h),
            0x75 => self.bit_b_r(6, self.reg.l),
            0x76 => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(6, data);
            }
            0x77 => self.bit_b_r(6, self.reg.a),
            0x78 => self.bit_b_r(7, self.reg.b),
            0x79 => self.bit_b_r(7, self.reg.c),
            0x7A => self.bit_b_r(7, self.reg.d),
            0x7B => self.bit_b_r(7, self.reg.e),
            0x7C => self.bit_b_r(7, self.reg.h),
            0x7D => self.bit_b_r(7, self.reg.l),
            0x7E => {
                let data = self.bus.read(self.reg.get_hl());
                self.bit_b_r(7, data);
            }
            0x7F => self.bit_b_r(7, self.reg.a),
            // RES b, r
            0x80 => self.reg.b = self.res_b_r(0, self.reg.b),
            0x81 => self.reg.c = self.res_b_r(0, self.reg.c),
            0x82 => self.reg.d = self.res_b_r(0, self.reg.d),
            0x83 => self.reg.e = self.res_b_r(0, self.reg.e),
            0x84 => self.reg.h = self.res_b_r(0, self.reg.h),
            0x85 => self.reg.l = self.res_b_r(0, self.reg.l),
            0x86 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(0, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x87 => self.reg.a = self.res_b_r(0, self.reg.a),
            0x88 => self.reg.b = self.res_b_r(1, self.reg.b),
            0x89 => self.reg.c = self.res_b_r(1, self.reg.c),
            0x8A => self.reg.d = self.res_b_r(1, self.reg.d),
            0x8B => self.reg.e = self.res_b_r(1, self.reg.e),
            0x8C => self.reg.h = self.res_b_r(1, self.reg.h),
            0x8D => self.reg.l = self.res_b_r(1, self.reg.l),
            0x8E => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(1, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x8F => self.reg.a = self.res_b_r(1, self.reg.a),
            0x90 => self.reg.b = self.res_b_r(2, self.reg.b),
            0x91 => self.reg.c = self.res_b_r(2, self.reg.c),
            0x92 => self.reg.d = self.res_b_r(2, self.reg.d),
            0x93 => self.reg.e = self.res_b_r(2, self.reg.e),
            0x94 => self.reg.h = self.res_b_r(2, self.reg.h),
            0x95 => self.reg.l = self.res_b_r(2, self.reg.l),
            0x96 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(2, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x97 => self.reg.a = self.res_b_r(2, self.reg.a),
            0x98 => self.reg.b = self.res_b_r(3, self.reg.b),
            0x99 => self.reg.c = self.res_b_r(3, self.reg.c),
            0x9A => self.reg.d = self.res_b_r(3, self.reg.d),
            0x9B => self.reg.e = self.res_b_r(3, self.reg.e),
            0x9C => self.reg.h = self.res_b_r(3, self.reg.h),
            0x9D => self.reg.l = self.res_b_r(3, self.reg.l),
            0x9E => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(3, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0x9F => self.reg.a = self.res_b_r(3, self.reg.a),
            0xA0 => self.reg.b = self.res_b_r(4, self.reg.b),
            0xA1 => self.reg.c = self.res_b_r(4, self.reg.c),
            0xA2 => self.reg.d = self.res_b_r(4, self.reg.d),
            0xA3 => self.reg.e = self.res_b_r(4, self.reg.e),
            0xA4 => self.reg.h = self.res_b_r(4, self.reg.h),
            0xA5 => self.reg.l = self.res_b_r(4, self.reg.l),
            0xA6 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(4, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xA7 => self.reg.a = self.res_b_r(4, self.reg.a),
            0xA8 => self.reg.b = self.res_b_r(5, self.reg.b),
            0xA9 => self.reg.c = self.res_b_r(5, self.reg.c),
            0xAA => self.reg.d = self.res_b_r(5, self.reg.d),
            0xAB => self.reg.e = self.res_b_r(5, self.reg.e),
            0xAC => self.reg.h = self.res_b_r(5, self.reg.h),
            0xAD => self.reg.l = self.res_b_r(5, self.reg.l),
            0xAE => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(5, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xAF => self.reg.a = self.res_b_r(5, self.reg.a),
            0xB0 => self.reg.b = self.res_b_r(6, self.reg.b),
            0xB1 => self.reg.c = self.res_b_r(6, self.reg.c),
            0xB2 => self.reg.d = self.res_b_r(6, self.reg.d),
            0xB3 => self.reg.e = self.res_b_r(6, self.reg.e),
            0xB4 => self.reg.h = self.res_b_r(6, self.reg.h),
            0xB5 => self.reg.l = self.res_b_r(6, self.reg.l),
            0xB6 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(6, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xB7 => self.reg.a = self.res_b_r(6, self.reg.a),
            0xB8 => self.reg.b = self.res_b_r(7, self.reg.b),
            0xB9 => self.reg.c = self.res_b_r(7, self.reg.c),
            0xBA => self.reg.d = self.res_b_r(7, self.reg.d),
            0xBB => self.reg.e = self.res_b_r(7, self.reg.e),
            0xBC => self.reg.h = self.res_b_r(7, self.reg.h),
            0xBD => self.reg.l = self.res_b_r(7, self.reg.l),
            0xBE => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.res_b_r(7, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xBF => self.reg.a = self.res_b_r(7, self.reg.a),
            // SET b, r
            0xC0 => self.reg.b = self.set_b_r(0, self.reg.b),
            0xC1 => self.reg.c = self.set_b_r(0, self.reg.c),
            0xC2 => self.reg.d = self.set_b_r(0, self.reg.d),
            0xC3 => self.reg.e = self.set_b_r(0, self.reg.e),
            0xC4 => self.reg.h = self.set_b_r(0, self.reg.h),
            0xC5 => self.reg.l = self.set_b_r(0, self.reg.l),
            0xC6 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(0, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xC7 => self.reg.a = self.set_b_r(0, self.reg.a),
            0xC8 => self.reg.b = self.set_b_r(1, self.reg.b),
            0xC9 => self.reg.c = self.set_b_r(1, self.reg.c),
            0xCA => self.reg.d = self.set_b_r(1, self.reg.d),
            0xCB => self.reg.e = self.set_b_r(1, self.reg.e),
            0xCC => self.reg.h = self.set_b_r(1, self.reg.h),
            0xCD => self.reg.l = self.set_b_r(1, self.reg.l),
            0xCE => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(1, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xCF => self.reg.a = self.set_b_r(1, self.reg.a),
            0xD0 => self.reg.b = self.set_b_r(2, self.reg.b),
            0xD1 => self.reg.c = self.set_b_r(2, self.reg.c),
            0xD2 => self.reg.d = self.set_b_r(2, self.reg.d),
            0xD3 => self.reg.e = self.set_b_r(2, self.reg.e),
            0xD4 => self.reg.h = self.set_b_r(2, self.reg.h),
            0xD5 => self.reg.l = self.set_b_r(2, self.reg.l),
            0xD6 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(2, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xD7 => self.reg.a = self.set_b_r(2, self.reg.a),
            0xD8 => self.reg.b = self.set_b_r(3, self.reg.b),
            0xD9 => self.reg.c = self.set_b_r(3, self.reg.c),
            0xDA => self.reg.d = self.set_b_r(3, self.reg.d),
            0xDB => self.reg.e = self.set_b_r(3, self.reg.e),
            0xDC => self.reg.h = self.set_b_r(3, self.reg.h),
            0xDD => self.reg.l = self.set_b_r(3, self.reg.l),
            0xDE => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(3, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xDF => self.reg.a = self.set_b_r(3, self.reg.a),
            0xE0 => self.reg.b = self.set_b_r(4, self.reg.b),
            0xE1 => self.reg.c = self.set_b_r(4, self.reg.c),
            0xE2 => self.reg.d = self.set_b_r(4, self.reg.d),
            0xE3 => self.reg.e = self.set_b_r(4, self.reg.e),
            0xE4 => self.reg.h = self.set_b_r(4, self.reg.h),
            0xE5 => self.reg.l = self.set_b_r(4, self.reg.l),
            0xE6 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(4, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xE7 => self.reg.a = self.set_b_r(4, self.reg.a),
            0xE8 => self.reg.b = self.set_b_r(5, self.reg.b),
            0xE9 => self.reg.c = self.set_b_r(5, self.reg.c),
            0xEA => self.reg.d = self.set_b_r(5, self.reg.d),
            0xEB => self.reg.e = self.set_b_r(5, self.reg.e),
            0xEC => self.reg.h = self.set_b_r(5, self.reg.h),
            0xED => self.reg.l = self.set_b_r(5, self.reg.l),
            0xEE => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(5, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xEF => self.reg.a = self.set_b_r(5, self.reg.a),
            0xF0 => self.reg.b = self.set_b_r(6, self.reg.b),
            0xF1 => self.reg.c = self.set_b_r(6, self.reg.c),
            0xF2 => self.reg.d = self.set_b_r(6, self.reg.d),
            0xF3 => self.reg.e = self.set_b_r(6, self.reg.e),
            0xF4 => self.reg.h = self.set_b_r(6, self.reg.h),
            0xF5 => self.reg.l = self.set_b_r(6, self.reg.l),
            0xF6 => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(6, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xF7 => self.reg.a = self.set_b_r(6, self.reg.a),
            0xF8 => self.reg.b = self.set_b_r(7, self.reg.b),
            0xF9 => self.reg.c = self.set_b_r(7, self.reg.c),
            0xFA => self.reg.d = self.set_b_r(7, self.reg.d),
            0xFB => self.reg.e = self.set_b_r(7, self.reg.e),
            0xFC => self.reg.h = self.set_b_r(7, self.reg.h),
            0xFD => self.reg.l = self.set_b_r(7, self.reg.l),
            0xFE => {
                let mut data = self.bus.read(self.reg.get_hl());
                data = self.set_b_r(7, data);
                self.bus.write(self.reg.get_hl(), data);
            }
            0xFF => self.reg.a = self.set_b_r(7, self.reg.a)
        };

        cycles
    }
}