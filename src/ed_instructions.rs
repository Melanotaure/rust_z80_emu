use crate::bus::{read_io, write_io};
use crate::{cycles::CYCLES_ED, z80::*};

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
        self.reg.flags.h = ((hl & 0x0FFF) + (reg & 0x0FFF) + c) > 0x0FFF;
        self.reg.flags.p = (hl as i16).overflowing_add((reg.wrapping_add(c)) as i16).1;
        self.reg.flags.n = false;
        self.reg.flags.c = (hl as u32) + (reg as u32 + c as u32) > 0x0000FFFF;
        r
    }

    fn neg(&mut self) {
        let a = self.reg.a;
        let r = 0_u8.wrapping_sub(a);
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = a & 0x0F > 0;
        self.reg.flags.p = a == 0x80;
        self.reg.flags.n = true;
        self.reg.flags.c = a != 0;
        self.reg.a = a;
    }

    fn ldi(&mut self) {
        let s = self.reg.get_hl();
        let d = self.reg.get_de();
        let data = self.bus.read(s);
        self.bus.write(d, data);
        self.reg.set_hl(s.wrapping_add(1));
        self.reg.set_de(d.wrapping_add(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        self.reg.flags.b5 = data & 0b00000010 == 0b00000010;
        self.reg.flags.b3 = data & 0b00001000 == 0b00001000;
        self.reg.flags.h = false;
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = false;
    }

    fn ldd(&mut self) {
        let s = self.reg.get_hl();
        let d = self.reg.get_de();
        let data = self.bus.read(s);
        self.bus.write(d, data);
        self.reg.set_hl(s.wrapping_sub(1));
        self.reg.set_de(d.wrapping_sub(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        self.reg.flags.b5 = data & 0b00000010 == 0b00000010;
        self.reg.flags.b3 = data & 0b00001000 == 0b00001000;
        self.reg.flags.h = false;
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = false;
    }

    fn cpi(&mut self) {
        let s = self.reg.get_hl();
        let data = self.bus.read(s);
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.set_hl(s.wrapping_add(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = (data & 0x0F) > (a & 0x0F);
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
        let n = a.wrapping_sub(data).wrapping_sub(self.reg.flags.h as u8);
        self.reg.flags.b5 = n & 0b00000010 == 0b00000010;
        self.reg.flags.b3 = n & 0b00001000 == 0b00001000;
    }

    fn cpd(&mut self) {
        let s = self.reg.get_hl();
        let data = self.bus.read(s);
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.set_hl(s.wrapping_sub(1));
        let bc = self.reg.get_bc();
        self.reg.set_bc(bc.wrapping_sub(1));
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.z = r == 0;
        self.reg.flags.h = (data & 0x0F) > (a & 0x0F);
        self.reg.flags.p = self.reg.get_bc() != 0;
        self.reg.flags.n = true;
        let n = a.wrapping_sub(data).wrapping_sub(self.reg.flags.h as u8);
        self.reg.flags.b5 = n & 0b00000010 == 0b00000010;
        self.reg.flags.b3 = n & 0b00001000 == 0b00001000;
    }

    fn ini(&mut self) {
        let s = self.reg.get_bc();
        let data = read_io(s);
        let d = self.reg.get_hl();
        self.bus.write(d, data);
        self.reg.set_hl(d.wrapping_add(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let k = data as u16 + self.reg.c.wrapping_add(1) as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
    }

    fn ind(&mut self) {
        let s = self.reg.get_bc();
        let data = read_io(s);
        let d = self.reg.get_hl();
        self.bus.write(d, data);
        self.reg.set_hl(d.wrapping_sub(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let k = data as u16 + self.reg.c.wrapping_sub(1) as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
    }

    fn outi(&mut self) {
        let s = self.reg.get_hl();
        let data = self.bus.read(s);
        self.reg.set_hl(s.wrapping_add(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let d = self.reg.get_bc();
        write_io(d, data);
        let k = data as u16 + self.reg.l as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
    }

    fn outd(&mut self) {
        let s = self.reg.get_hl();
        let data = self.bus.read(s);
        self.reg.set_hl(s.wrapping_sub(1));
        self.reg.b = self.dec_r(self.reg.b);
        self.reg.flags.n = data & 0x80 == 0x80;
        let d = self.reg.get_bc();
        write_io(d, data);
        let k = data as u16 + self.reg.l as u16;
        self.reg.flags.c = k > 0x00FF;
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.p = ((k & 0x0007) as u8 ^ self.reg.b).count_ones() & 0x01 == 0;
    }

    fn ld_a_ri(&mut self, reg: u8) {
        self.reg.a = reg;
        self.reg.flags.s = reg & 0x80 == 0x80;
        self.reg.flags.z = reg == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = self.iff2;
        self.reg.flags.n = false;
    }

    fn rld(&mut self) {
        let n = self.bus.read(self.reg.get_hl());
        let a = self.reg.a;
        let tmp = a & 0x0F;
        let a = (a & 0xF0) | (n >> 4);
        let n = (n << 4) | tmp;
        self.reg.flags.s = a & 0x80 == 0x80;
        self.reg.flags.z = a == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = a.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.a = a;
        self.bus.write(self.reg.get_hl(), n);
    }

    fn rrd(&mut self) {
        let n = self.bus.read(self.reg.get_hl());
        let a = self.reg.a;
        let tmp = a << 4;
        let a = (a & 0xF0) | (n & 0x0F);
        let n = (n >> 4) | tmp;
        self.reg.flags.s = a & 0x80 == 0x80;
        self.reg.flags.z = a == 0;
        self.reg.flags.h = false;
        self.reg.flags.p = a.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.a = a;
        self.bus.write(self.reg.get_hl(), n);
    }

    pub fn ed_instructions(&mut self) -> u8 {
        let opcode = self.bus.read(self.reg.pc);
        let mut cycles = CYCLES_ED[opcode as usize];

        match opcode {
            // IN r, (C)
            0x40 => self.reg.b = self.in_r_c(),
            0x48 => self.reg.c = self.in_r_c(),
            0x50 => self.reg.d = self.in_r_c(),
            0x58 => self.reg.e = self.in_r_c(),
            0x60 => self.reg.h = self.in_r_c(),
            0x68 => self.reg.l = self.in_r_c(),
            0x70 => _ = self.in_r_c(),
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
            0x44 | 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C => self.neg(),
            // Interrupt mode
            0x46 | 0x4E | 0x66 | 0x6E => self.im = InterruptMode::IM_0,
            0x56 | 0x76 => self.im = InterruptMode::IM_1,
            0x5E | 0x7E => self.im = InterruptMode::IM_2,
            // LD I,A ; LD A,I ; LD R,A ; LD A,R
            0x47 => self.reg.i = self.reg.a,
            0x57 => self.ld_a_ri(self.reg.i),
            0x4F => self.reg.r = self.reg.a,
            0x5F => self.ld_a_ri(self.reg.r),
            // LDI ; LDIR
            0xA0 => self.ldi(),
            0xB0 => {
                self.ldi();
                if self.reg.flags.p {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // LDD ; LDDR
            0xA8 => self.ldd(),
            0xB8 => {
                self.ldd();
                if self.reg.flags.p {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // CPI ; CPIR
            0xA1 => self.cpi(),
            0xB1 => {
                self.cpi();
                if self.reg.flags.p {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // CPD ; CPDR
            0xA9 => self.cpd(),
            0xB9 => {
                self.cpd();
                if self.reg.flags.p {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // INI ; INIR
            0xA2 => self.ini(),
            0xB2 => {
                self.ini();
                if !self.reg.flags.z {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // IND ; INDR
            0xAA => self.ind(),
            0xBA => {
                self.ind();
                if !self.reg.flags.z {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // OUTI ; OUTIR
            0xA3 => self.outi(),
            0xB3 => {
                self.outi();
                if !self.reg.flags.z {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // OUTD ; OUTDR
            0xAB => self.outd(),
            0xBB => {
                self.outd();
                if !self.reg.flags.z {
                    self.reg.pc = self.reg.pc.wrapping_sub(3);
                    cycles += 5;
                }
            }
            // RETN
            0x45 | 0x55 | 0x5D | 0x65 | 0x6D | 0x75 | 0x7D => {
                self.iff1 = self.iff2;
                self.ret();
            }
            // RETI
            0x4D => {
                self.iff1 = self.iff2;
                self.ret();
            }

            // RRD and RLD
            0x67 => self.rrd(),
            0x6F => self.rld(),

            // NOP
            0x77 | 0x7F => {}

            _ => {}
        }
        cycles
    }
}
