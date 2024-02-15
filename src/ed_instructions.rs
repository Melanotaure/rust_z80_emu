use crate::{cycles::CYCLES_ED, z80::*};
use crate::bus::{read_io, write_io};

impl Z80 {
    fn in_r_c(&mut self) -> u8 {
        let addr = self.reg.get_bc();
        let data = read_io(addr);
        self.reg.flags.s = data & 0x80 == 0x80;
        self.reg.flags.z = data == 0x00;
        self.reg.flags.h = false;
        self.reg.flags.p = data.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        data
    }

    fn out_c_r(&mut self, reg: u8) {
        let addr = self.reg.get_bc();
        write_io(addr, reg);
    }

    fn sbc_hl_rr(&mut self, reg: u16) -> u16 {
        let c = self.reg.flags.c as u16;
        let hl = self.reg.get_hl();
        let r = hl.wrapping_sub(reg.wrapping_add(c));
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = (r & 0x0FFF) < (reg.wrapping_add(c) & 0x0FFF);
        self.reg.flags.p = (hl as i16).overflowing_sub((reg.wrapping_add(c)) as i16).1;
        self.reg.flags.n = true;
        self.reg.flags.c = (hl as u32) < (reg as u32 + c as u32);
        r
    }

    fn adc_hl_rr(&mut self, reg: u16) -> u16 {
        let c = self.reg.flags.c as u16;
        let hl = self.reg.get_hl();
        let r = hl.wrapping_add(reg).wrapping_add(c);
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = ((hl & 0x0FFF) + (reg & 0x0FFF) + c) > 0x0FFF ;
        self.reg.flags.p = (hl as i16).overflowing_add((reg.wrapping_add(c)) as i16).1;
        self.reg.flags.n = false;
        self.reg.flags.c = (hl as u32) + (reg as u32 + c as u32) > 0x0000FFFF;
        r
    }

    pub fn ed_instructions(&mut self) -> u8 {
        let opcode = self.bus.read(self.reg.pc);
        let cycles = CYCLES_ED[opcode as usize];

        match opcode {
            // IN r, (C)
            0x40 => self.reg.b = self.in_r_c(),
            0x48 => self.reg.c = self.in_r_c(),
            0x50 => self.reg.d = self.in_r_c(),
            0x58 => self.reg.e = self.in_r_c(),
            0x60 => self.reg.h = self.in_r_c(),
            0x68 => self.reg.l = self.in_r_c(),
            0x70 =>          _ = self.in_r_c(),
            0x78 => self.reg.a = self.in_r_c(),
            // OUT (C), r
            0x41 => self.out_c_r(self.reg.b),
            0x49 => self.out_c_r(self.reg.c),
            0x51 => self.out_c_r(self.reg.d),
            0x59 => self.out_c_r(self.reg.e),
            0x61 => self.out_c_r(self.reg.h),
            0x69 => self.out_c_r(self.reg.l),
            0x71 => self.out_c_r(0x00),
            0x79 => self.out_c_r(self.reg.a),
            // SBC HL, rr
            0x42 => {
                let hl = self.sbc_hl_rr(self.reg.get_bc());
                self.reg.set_hl(hl);
            }
            0x52 => {
                let hl = self.sbc_hl_rr(self.reg.get_de());
                self.reg.set_hl(hl);
            }
            0x62 => {
                let hl = self.sbc_hl_rr(self.reg.get_hl());
                self.reg.set_hl(hl);
            }
            0x72 => {
                let hl = self.sbc_hl_rr(self.reg.sp);
                self.reg.set_hl(hl);
            }
            // ADC HL, rr
            0x4A => {
                let hl = self.adc_hl_rr(self.reg.get_bc());
                self.reg.set_hl(hl);
            }
            0x5A => {
                let hl = self.adc_hl_rr(self.reg.get_de());
                self.reg.set_hl(hl);
            }
            0x6A => {
                let hl = self.adc_hl_rr(self.reg.get_hl());
                self.reg.set_hl(hl);
            }
            0x7A => {
                let hl = self.adc_hl_rr(self.reg.sp);
                self.reg.set_hl(hl);
            }
            // LD (nn), rr
            0x43 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.reg.c);
                self.bus.write(nn.wrapping_add(1), self.reg.b);
            }
            0x53 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.reg.e);
                self.bus.write(nn.wrapping_add(1), self.reg.d);
            }
            0x63 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.reg.l);
                self.bus.write(nn.wrapping_add(1), self.reg.h);
            }
            0x73 => {
                let nn = self.get_nn();
                let [spl, sph] = self.reg.sp.to_le_bytes();
                self.bus.write(nn, spl);
                self.bus.write(nn.wrapping_add(1), sph);
            }
            // LD rr, (nn)
            0x4B => {
                let nn = self.get_nn();
                self.reg.c = self.bus.read(nn);
                self.reg.b = self.bus.read(nn.wrapping_add(1));
            }
            0x5B => {
                let nn = self.get_nn();
                self.reg.e = self.bus.read(nn);
                self.reg.d = self.bus.read(nn.wrapping_add(1));
            }
            0x6B => {
                let nn = self.get_nn();
                self.reg.l = self.bus.read(nn);
                self.reg.h = self.bus.read(nn.wrapping_add(1));
            }
            0x7B => {
                let nn = self.get_nn();
                let spl = self.bus.read(nn);
                let sph = self.bus.read(nn.wrapping_add(1));
                self.reg.sp = u16::from_le_bytes([spl, sph]);
            }

            _ => {}
        }
        cycles
    }
}