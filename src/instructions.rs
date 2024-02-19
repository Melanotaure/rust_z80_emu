use crate::bus::{read_io, write_io};
use crate::cycles::CYCLES;
use crate::z80::*;

enum BitOp {
    AND,
    XOR,
    OR,
}

impl Z80 {
    pub fn get_nn(&mut self) -> u16 {
        self.reg.inc_pc();
        let nl = self.bus.read(self.reg.pc);
        self.reg.inc_pc();
        let nh = self.bus.read(self.reg.pc);
        let nn = u16::from_le_bytes([nl, nh]);
        nn
    }

    fn jp_nn(&mut self) {
        let nn = self.get_nn();
        self.reg.pc = nn.wrapping_sub(1);
        // PC is incremented at the end
    }

    fn jr_e(&mut self) {
        self.reg.inc_pc();
        let e = self.bus.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add((e as i8) as u16);
    }

    fn call_nn(&mut self) {
        // PC is first incremented by 3 to resume the flow after this 3-byte instruction
        let pc = self.reg.pc.wrapping_add(3);
        let [mut pcl, mut pch] = pc.to_le_bytes();
        self.reg.dec_sp();
        self.bus.write(self.reg.sp, pch);
        self.reg.dec_sp();
        self.bus.write(self.reg.sp, pcl);
        self.reg.inc_pc();
        pcl = self.bus.read(self.reg.pc);
        self.reg.inc_pc();
        pch = self.bus.read(self.reg.pc);
        self.reg.pc = u16::from_le_bytes([pcl, pch]);
        self.reg.dec_pc();
    }

    pub fn ret(&mut self) {
        let pcl = self.bus.read(self.reg.sp);
        self.reg.inc_sp();
        let pch = self.bus.read(self.reg.sp);
        self.reg.pc = u16::from_be_bytes([pcl, pch]);
        self.reg.dec_pc();
    }

    fn rst(&mut self, addr: u8) {
        let [pcl, pch] = self.reg.pc.to_be_bytes();
        self.reg.dec_sp();
        self.bus.write(self.reg.sp, pch);
        self.reg.dec_sp();
        self.bus.write(self.reg.sp, pcl);
        self.reg.pc = u16::from_le_bytes([addr, 0x00]);
        self.reg.dec_pc();
    }

    fn add_a_r(&mut self, data: u8) {
        let a = self.reg.a;
        let r = a.wrapping_add(data);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = (a & 0x0F) + (data & 0x0F) > 0x0F;
        self.reg.flags.p = (a as i8).overflowing_add(data as i8).1;
        self.reg.flags.n = false;
        self.reg.flags.c = (a as u16) + (data as u16) > 0x00FF;
        self.reg.a = r;
    }

    fn adc_a_r(&mut self, data: u8) {
        let c = self.reg.flags.c as u8;
        let a = self.reg.a;
        let r = a.wrapping_add(data).wrapping_add(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = (a & 0x0F) + (data & 0x0F) + c > 0x0F;
        self.reg.flags.p = (a as i8).overflowing_add((data.wrapping_add(c)) as i8).1;
        self.reg.flags.n = false;
        self.reg.flags.c = (a as u16) + (data as u16) + (c as u16) > 0x00FF;
        self.reg.a = r;
    }

    fn sub_a_r(&mut self, data: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = (a & 0x0F) < (data & 0x0F);
        self.reg.flags.p = (a as i8).overflowing_sub(data as i8).1;
        self.reg.flags.n = true;
        self.reg.flags.c = (a as u16) < (data as u16);
        self.reg.a = r;
    }

    fn sbc_a_r(&mut self, data: u8) {
        let c = self.reg.flags.c as u8;
        let a = self.reg.a;
        let r = a.wrapping_sub(data).wrapping_sub(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.h = (a & 0x0F) < (data & 0x0F).wrapping_add(c);
        self.reg.flags.p = (a as i8).overflowing_sub((data.wrapping_add(c)) as i8).1;
        self.reg.flags.n = true;
        self.reg.flags.c = (a as u16) < ((data as u16) + (c as u16));
        self.reg.a = r;
    }

    fn bit_op_a_r(&mut self, bit_op: BitOp, data: u8) {
        let a = self.reg.a;
        let r = match bit_op {
            BitOp::AND => a & data,
            BitOp::XOR => a ^ data,
            BitOp::OR => a | data,
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = true;
        self.reg.flags.p = r.count_ones() & 0x01 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = false;
        self.reg.a = r;
    }

    fn cp_r(&mut self, data: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = (a & 0x0F) < (data & 0x0F);
        self.reg.flags.p = (a as i8).overflowing_sub(data as i8).1;
        self.reg.flags.n = true;
        self.reg.flags.c = (a as u16) < (data as u16);
    }

    fn inc_r(&mut self, data: u8) -> u8 {
        let r = data.wrapping_add(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = data & 0x0F == 0x0F;
        self.reg.flags.p = data == 0x7F;
        self.reg.flags.n = false;
        r
    }

    fn dec_r(&mut self, data: u8) -> u8 {
        let r = data.wrapping_sub(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = data & 0x1F == 0x10;
        self.reg.flags.p = data == 0x80;
        self.reg.flags.n = true;
        r
    }

    fn add_hl_ix_iy_rr(&mut self, reg: u16) {
        let hl = match self.p_inst {
            0xDD => self.reg.get_ix(),
            0xFD => self.reg.get_iy(),
            _ => self.reg.get_hl(),
        };
        self.reg.flags.h = (hl & 0x0FFF) + (reg & 0x0FFF) > 0x0FFF;
        self.reg.flags.n = false;
        self.reg.flags.c = hl as u32 + reg as u32 > 0xFFFF;

        match self.p_inst {
            0xDD => self.reg.set_ix(hl.wrapping_add(reg)),
            0xFD => self.reg.set_iy(hl.wrapping_add(reg)),
            _ => self.reg.set_hl(hl.wrapping_add(reg)),
        }
    }

    fn daa(&mut self) {
        let a = self.reg.a;
        let rl = if self.reg.a & 0x0F > 0x09 || self.reg.flags.h {
            0x06_u8
        } else {
            0x00_u8
        };
        let rh = if self.reg.a & 0xF0 > 0x90 || self.reg.flags.c {
            0x60u8
        } else {
            0x00_u8
        };
        if !self.reg.flags.n {
            (self.reg.a, self.reg.flags.c) = a.overflowing_add(rh | rl);
        } else {
            (self.reg.a, self.reg.flags.c) = a.overflowing_sub(rh | rl);
        }
        self.reg.flags.s = (self.reg.a as i8) < 0;
        self.reg.flags.z = self.reg.a == 0;
        self.reg.flags.h = rl == 0x06_u8;
        self.reg.flags.p = self.reg.a.count_ones() & 0x01 == 0;
        self.reg.flags.c = rh == 0x60_u8;
    }

    fn cpl(&mut self) {
        self.reg.a = !self.reg.a;
        self.reg.flags.h = true;
        self.reg.flags.n = true;
    }

    fn ccf(&mut self) {
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.n = false;
        self.reg.flags.c = !self.reg.flags.c;
    }

    fn scf(&mut self) {
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.flags.c = true;
    }

    fn get_h_ixh_iyh(&self) -> u8 {
        match self.p_inst {
            0xDD => self.reg.ixh,
            0xFD => self.reg.iyh,
            _ => self.reg.h,
        }
    }

    fn set_h_ixh_iyh(&mut self, reg: u8) {
        match self.p_inst {
            0xDD => self.reg.ixh = reg,
            0xFD => self.reg.iyh = reg,
            _ => self.reg.h = reg,
        };
    }

    fn get_l_ixl_iyl(&self) -> u8 {
        match self.p_inst {
            0xDD => self.reg.ixl,
            0xFD => self.reg.iyl,
            _ => self.reg.l,
        }
    }

    fn set_l_ixl_iyl(&mut self, reg: u8) {
        match self.p_inst {
            0xDD => self.reg.ixl = reg,
            0xFD => self.reg.iyl = reg,
            _ => self.reg.l = reg,
        };
    }

    fn get_hl_ix_iy(&self) -> u16 {
        match self.p_inst {
            0xDD => self.reg.get_ix(),
            0xFD => self.reg.get_iy(),
            _ => self.reg.get_hl(),
        }
    }

    fn set_hl_ix_iy(&mut self, data: u16) {
        match self.p_inst {
            0xDD => self.reg.set_ix(data),
            0xFF => self.reg.set_iy(data),
            _ => self.reg.set_hl(data),
        };
    }

    fn read_hl_ix_iy(&mut self) -> u8 {
        match self.p_inst {
            0xDD => {
                self.reg.inc_pc();
                let d = self.bus.read(self.reg.pc);
                let ix = self.reg.get_ix();
                let addr = ix.wrapping_add((d as i8) as u16);
                self.bus.read(addr)
            }
            0xFD => {
                self.reg.inc_pc();
                let d = self.bus.read(self.reg.pc);
                let iy = self.reg.get_iy();
                let addr = iy.wrapping_add((d as i8) as u16);
                self.bus.read(addr)
            }
            _ => self.bus.read(self.reg.get_hl()), // LD C, (HL)
        }
    }

    fn write_hl_ix_iy(&mut self, reg: u8) {
        match self.p_inst {
            0xDD => {
                self.reg.inc_pc();
                let d = self.bus.read(self.reg.pc);
                let ix = self.reg.get_ix();
                let addr = ix.wrapping_add((d as i8) as u16);
                self.bus.write(addr, reg);
            }
            0xFD => {
                self.reg.inc_pc();
                let d = self.bus.read(self.reg.pc);
                let iy = self.reg.get_iy();
                let addr = iy.wrapping_add((d as i8) as u16);
                self.bus.write(addr, reg);
            }
            _ => self.bus.write(self.reg.get_hl(), reg), // LD C, (HL)
        }
    }

    // Main function to run the CPU's instructions
    pub fn execute(&mut self) -> u8 {
        let instr = self.bus.read(self.reg.pc);
        let mut cycles = CYCLES[instr as usize];

        match instr {
            // NOP
            0x00 => {}

            // 8-bit load group
            // Destination reg = b
            0x40 => {}                                 // LD B, B
            0x41 => self.reg.b = self.reg.c,           // LD B, C
            0x42 => self.reg.b = self.reg.d,           // LD B, D
            0x43 => self.reg.b = self.reg.e,           // LD B, E
            0x44 => self.reg.b = self.get_h_ixh_iyh(), // LD B,L IXL IYL
            0x45 => self.reg.b = self.get_l_ixl_iyl(), // LD B,H IXH IYH
            0x46 => self.reg.b = self.read_hl_ix_iy(), // LD B, (HL IX+d IY+d)
            0x47 => self.reg.b = self.reg.a,           // LD B, A
            // Destination reg = c
            0x48 => self.reg.c = self.reg.b,           // LD C, B
            0x49 => {}                                 // LD C, C
            0x4A => self.reg.c = self.reg.d,           // LD C, D
            0x4B => self.reg.c = self.reg.e,           // LD C, E
            0x4C => self.reg.c = self.get_h_ixh_iyh(), // LD C, H IXH IYH
            0x4D => self.reg.c = self.get_l_ixl_iyl(), // LD C, L IXL, IYL
            0x4E => self.reg.c = self.read_hl_ix_iy(), // LD C, (HL IX+d IY+d)
            0x4F => self.reg.c = self.reg.a,           // LD C, A
            // Destination reg = d
            0x50 => self.reg.d = self.reg.b,           // LD D, B
            0x51 => self.reg.d = self.reg.c,           // LD D, C
            0x52 => {}                                 // LD D, D
            0x53 => self.reg.d = self.reg.e,           // LD D, E
            0x54 => self.reg.d = self.get_h_ixh_iyh(), // LD D, H IXH IYH
            0x55 => self.reg.d = self.get_l_ixl_iyl(), // LD D, L IXL IYL
            0x56 => self.reg.d = self.read_hl_ix_iy(), // LD D, (HL IX+d IY+d)
            0x57 => self.reg.d = self.reg.a,           // LD D, A
            // Destination reg = e
            0x58 => self.reg.e = self.reg.b,           // LD E, B
            0x59 => self.reg.e = self.reg.c,           // LD E, C
            0x5A => self.reg.e = self.reg.d,           // LD E, D
            0x5B => {}                                 // LD E, E
            0x5C => self.reg.e = self.get_h_ixh_iyh(), // LD E, H IXH IYH
            0x5D => self.reg.e = self.get_l_ixl_iyl(), // LD E, L IXL IYL
            0x5E => self.reg.e = self.read_hl_ix_iy(), // LD E, (HL IX+d IY+d)
            0x5F => self.reg.e = self.reg.a,           // LD E, A
            // Destination reg = h
            0x60 => self.set_h_ixh_iyh(self.reg.b), // LD H, B
            0x61 => self.set_h_ixh_iyh(self.reg.c), // LD H, C
            0x62 => self.set_h_ixh_iyh(self.reg.d), // LD H, D
            0x63 => self.set_h_ixh_iyh(self.reg.e), // LD H, E
            0x64 => {}                              // LD H, H
            0x65 => self.set_h_ixh_iyh(self.get_l_ixl_iyl()), // LD H, L
            0x66 => self.reg.h = self.read_hl_ix_iy(), // LD H, (HL IX+d IY+d)
            0x67 => self.set_h_ixh_iyh(self.reg.a), // LD H, A
            // Destination reg = l
            0x68 => self.set_l_ixl_iyl(self.reg.b), // LD L, B
            0x69 => self.set_l_ixl_iyl(self.reg.c), // LD L, C
            0x6A => self.set_l_ixl_iyl(self.reg.d), // LD L, D
            0x6B => self.set_l_ixl_iyl(self.reg.e), // LD L, E
            0x6C => self.set_l_ixl_iyl(self.get_h_ixh_iyh()), // LD L, H
            0x6D => {}                              // LD L, L
            0x6E => self.reg.l = self.read_hl_ix_iy(), // LD L, (HL IX+d IY+d)
            0x6F => self.set_l_ixl_iyl(self.reg.a), // LD L, A
            // Destination reg = (HL IX+d IY+d)
            0x70 => self.write_hl_ix_iy(self.reg.b), // LD (HL), B
            0x71 => self.write_hl_ix_iy(self.reg.c), // LD (HL), C
            0x72 => self.write_hl_ix_iy(self.reg.d), // LD (HL), D
            0x73 => self.write_hl_ix_iy(self.reg.e), // LD (HL), E
            0x74 => self.write_hl_ix_iy(self.reg.h), // LD (HL), H
            0x75 => self.write_hl_ix_iy(self.reg.l), // LD (HL), L
            // 0x76 => HALT treated elsewhere
            0x77 => self.write_hl_ix_iy(self.reg.a), // LD (HL), A
            // Destination reg = a
            0x78 => self.reg.a = self.reg.b,           // LD A, B
            0x79 => self.reg.a = self.reg.c,           // LD A, C
            0x7A => self.reg.a = self.reg.d,           // LD A, D
            0x7B => self.reg.a = self.reg.e,           // LD A, E
            0x7C => self.reg.a = self.get_h_ixh_iyh(), // LD A, H
            0x7D => self.reg.a = self.get_l_ixl_iyl(), // LD A, L
            0x7E => self.reg.a = self.read_hl_ix_iy(), // LD A, (HL)
            0x7F => {}                                 // LD A, A
            // LD r, n
            0x06 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.b = n;
            }
            0x16 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.d = n;
            }
            0x26 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.set_h_ixh_iyh(n);
            }
            0x36 => {
                // LD (HL IX+d IY+d), n when d, n is the second byte (xxyyddnn)
                let n = self.bus.read(self.reg.pc.wrapping_add(2));
                self.write_hl_ix_iy(n);
                self.reg.inc_pc();
            }
            0x0E => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.c = n;
            }
            0x1E => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.e = n;
            }
            0x2E => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.set_l_ixl_iyl(n);
            }
            0x3E => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.a = n;
            }
            // LD (BC), A
            0x02 => self.bus.write(self.reg.get_bc(), self.reg.a),
            // LD (DE), A
            0x12 => self.bus.write(self.reg.get_de(), self.reg.a),
            // LD (nn), A
            0x32 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.reg.a);
            }
            // LD A, (BC)
            0x0A => self.reg.a = self.bus.read(self.reg.get_bc()),
            // LD A, (DE)
            0x1A => self.reg.a = self.bus.read(self.reg.get_de()),
            // LD A, (nn)
            0x3A => {
                let nn = self.get_nn();
                self.reg.a = self.bus.read(nn);
            }

            // 16-bit Load Group
            // LD BC, nn
            0x01 => {
                let nn = self.get_nn();
                self.reg.set_bc(nn);
            }
            // LD DE, nn
            0x11 => {
                let nn = self.get_nn();
                self.reg.set_de(nn);
            }
            // LD HL, nn
            0x21 => {
                let nn = self.get_nn();
                self.set_hl_ix_iy(nn);
            }
            // LD SP, nn
            0x31 => {
                let nn = self.get_nn();
                self.reg.sp = nn;
            }
            // LD HL, (nn)
            0x2A => {
                let nn = self.get_nn();
                let l = self.bus.read(nn);
                let h = self.bus.read(nn.wrapping_add(1));
                self.set_hl_ix_iy(u16::from_le_bytes([l, h]));
            }
            // LD (nn), HL
            0x22 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.get_l_ixl_iyl());
                self.bus.write(nn.wrapping_add(1), self.get_h_ixh_iyh());
            }
            // LD SP, HL
            0xF9 => self.reg.sp = u16::from_le_bytes([self.get_l_ixl_iyl(), self.get_h_ixh_iyh()]),
            // PUSH BC
            0xC5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.b);
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.c);
            }
            // PUSH DE
            0xD5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.d);
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.e);
            }
            // PUSH HL IX IY
            0xE5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.get_h_ixh_iyh());
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.get_l_ixl_iyl());
            }
            // PUSH AF
            0xF5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.a);
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.flags.to_byte());
            }
            // POP BC
            0xC1 => {
                self.reg.c = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.b = self.bus.read(self.reg.sp);
            }
            // POP DE
            0xD1 => {
                self.reg.e = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.d = self.bus.read(self.reg.sp);
            }
            // POP HL IX IY
            0xE1 => {
                self.set_l_ixl_iyl(self.bus.read(self.reg.sp));
                self.reg.inc_sp();
                self.set_h_ixh_iyh(self.bus.read(self.reg.sp));
            }
            // POP AF
            0xF1 => {
                let f = self.bus.read(self.reg.sp);
                self.reg.flags.from_byte(f);
                self.reg.inc_sp();
                self.reg.a = self.bus.read(self.reg.sp);
            }
            // Exchange
            // EX DE, HL
            0xEB => {
                let de = self.reg.get_de();
                let hl = self.reg.get_hl();
                self.reg.set_de(hl);
                self.reg.set_hl(de);
            }
            // EX AF,AF'
            0x08 => {
                let af = self.reg.get_af();
                let eaf = self.reg.eaf;
                self.reg.set_af(eaf);
                self.reg.eaf = af;
            }
            // EXX
            0xD9 => {
                let tmp = self.reg.get_bc();
                self.reg.set_bc(self.reg.ebc);
                self.reg.ebc = tmp;
                let tmp = self.reg.get_de();
                self.reg.set_de(self.reg.ede);
                self.reg.ede = tmp;
                let tmp = self.reg.get_hl();
                self.reg.set_hl(self.reg.ehl);
                self.reg.ehl = tmp;
            }
            // EX (SP), HL IX IY
            0xE3 => {
                let n = self.bus.read(self.reg.sp);
                self.bus.write(self.reg.sp, self.get_l_ixl_iyl());
                self.set_l_ixl_iyl(n);
                self.reg.inc_sp();
                let n = self.bus.read(self.reg.sp);
                self.bus.write(self.reg.sp, self.get_h_ixh_iyh());
                self.set_h_ixh_iyh(n);
            }

            // Jump group
            // JP nn
            0xC3 => self.jp_nn(),
            // JP nz, nn
            0xC2 => {
                if !self.reg.flags.z {
                    self.jp_nn();
                }
            }
            // JP z
            0xCA => {
                if self.reg.flags.z {
                    self.jp_nn();
                }
            }
            // JP nc, nn
            0xD2 => {
                if !self.reg.flags.c {
                    self.jp_nn();
                }
            }
            // JP c, nn
            0xDA => {
                if self.reg.flags.c {
                    self.jp_nn();
                }
            }
            // JP po, nn
            0xE2 => {
                if !self.reg.flags.p {
                    self.jp_nn();
                }
            }
            // JP pe, nn
            0xEA => {
                if self.reg.flags.p {
                    self.jp_nn();
                }
            }
            // JP p, nn
            0xF2 => {
                if !self.reg.flags.s {
                    self.jp_nn();
                }
            }
            // JP m, nn
            0xFA => {
                if self.reg.flags.s {
                    self.jp_nn();
                }
            }
            // JR e
            0x18 => self.jr_e(),
            // JR z, e
            0x28 => {
                if self.reg.flags.z {
                    self.jr_e();
                }
            }
            // JR c, e
            0x38 => {
                if self.reg.flags.c {
                    self.jr_e();
                }
            }
            // DJNZ e
            0x10 => {
                self.reg.b = self.reg.b.wrapping_sub(1);
                if self.reg.b != 0 {
                    self.jr_e();
                    cycles += 5;
                }
            }
            // JR nz, e
            0x20 => {
                if !self.reg.flags.z {
                    self.jr_e();
                }
            }
            // JR nc, nn
            0x30 => {
                if !self.reg.flags.c {
                    self.jr_e();
                }
            }
            // JP (HL)
            0xE9 => {
                self.reg.pc =
                    u16::from_le_bytes([self.get_l_ixl_iyl(), self.get_h_ixh_iyh()]).wrapping_sub(1)
            }

            // Call & Return Group
            // CALL nn
            0xCD => self.call_nn(),
            // CALL nz, nn
            0xC4 => {
                if !self.reg.flags.z {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL nc, nn
            0xD4 => {
                if !self.reg.flags.c {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL po, nn
            0xE4 => {
                if !self.reg.flags.p {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL p, nn
            0xF4 => {
                if !self.reg.flags.n {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL z, nn
            0xCC => {
                if self.reg.flags.z {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL c, nn
            0xDC => {
                if self.reg.flags.c {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL pe, nn
            0xEC => {
                if self.reg.flags.p {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // CALL n, nn
            0xFC => {
                if self.reg.flags.n {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // RET
            0xC9 => self.ret(),
            // RET nz
            0xC0 => {
                if !self.reg.flags.z {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET nc
            0xD0 => {
                if !self.reg.flags.c {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET po
            0xE0 => {
                if !self.reg.flags.p {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET p
            0xF0 => {
                if !self.reg.flags.n {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET z
            0xC8 => {
                if self.reg.flags.z {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET c
            0xD8 => {
                if self.reg.flags.c {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET pe
            0xE8 => {
                if self.reg.flags.p {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET n
            0xF8 => {
                if self.reg.flags.n {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RST 0x00..0x38
            0xC7 => self.rst(0x00),
            0xCF => self.rst(0x08),
            0xD7 => self.rst(0x10),
            0xDF => self.rst(0x18),
            0xE7 => self.rst(0x20),
            0xEF => self.rst(0x28),
            0xF7 => self.rst(0x30),
            0xFF => self.rst(0x38),

            // Input & Output Group
            // IN A, (n)
            0xDB => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                let addr = u16::from_le_bytes([n, self.reg.a]);
                self.reg.a = read_io(addr);
            }
            // OUT (n), A
            0xD3 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                let addr = u16::from_le_bytes([n, self.reg.a]);
                write_io(addr, self.reg.a);
            }

            // 8-bit arithmetic group
            // ADD A, r
            0x80 => self.add_a_r(self.reg.b),
            0x81 => self.add_a_r(self.reg.c),
            0x82 => self.add_a_r(self.reg.d),
            0x83 => self.add_a_r(self.reg.e),
            0x84 => self.add_a_r(self.get_h_ixh_iyh()),
            0x85 => self.add_a_r(self.get_l_ixl_iyl()),
            0x86 => {
                let data = self.read_hl_ix_iy();
                self.add_a_r(data);
            }
            0x87 => self.add_a_r(self.reg.a),
            // ADC A, r
            0x88 => self.adc_a_r(self.reg.b),
            0x89 => self.adc_a_r(self.reg.c),
            0x8A => self.adc_a_r(self.reg.d),
            0x8B => self.adc_a_r(self.reg.e),
            0x8C => self.adc_a_r(self.get_h_ixh_iyh()),
            0x8D => self.adc_a_r(self.get_l_ixl_iyl()),
            0x8E => {
                let data = self.read_hl_ix_iy();
                self.adc_a_r(data);
            }
            0x8F => self.adc_a_r(self.reg.a),
            // SUB A, r
            0x90 => self.sub_a_r(self.reg.b),
            0x91 => self.sub_a_r(self.reg.c),
            0x92 => self.sub_a_r(self.reg.d),
            0x93 => self.sub_a_r(self.reg.e),
            0x94 => self.sub_a_r(self.get_h_ixh_iyh()),
            0x95 => self.sub_a_r(self.get_l_ixl_iyl()),
            0x96 => {
                let data = self.read_hl_ix_iy();
                self.sub_a_r(data);
            }
            0x97 => self.sub_a_r(self.reg.a),
            // SBC A, r
            0x98 => self.sbc_a_r(self.reg.b),
            0x99 => self.sbc_a_r(self.reg.c),
            0x9A => self.sbc_a_r(self.reg.d),
            0x9B => self.sbc_a_r(self.reg.e),
            0x9C => self.sbc_a_r(self.get_h_ixh_iyh()),
            0x9D => self.sbc_a_r(self.get_l_ixl_iyl()),
            0x9E => {
                let data = self.read_hl_ix_iy();
                self.sbc_a_r(data);
            }
            0x9F => self.sbc_a_r(self.reg.a),
            // AND A, r
            0xA0 => self.bit_op_a_r(BitOp::AND, self.reg.b),
            0xA1 => self.bit_op_a_r(BitOp::AND, self.reg.c),
            0xA2 => self.bit_op_a_r(BitOp::AND, self.reg.d),
            0xA3 => self.bit_op_a_r(BitOp::AND, self.reg.e),
            0xA4 => self.bit_op_a_r(BitOp::AND, self.get_h_ixh_iyh()),
            0xA5 => self.bit_op_a_r(BitOp::AND, self.get_l_ixl_iyl()),
            0xA6 => {
                let data = self.read_hl_ix_iy();
                self.bit_op_a_r(BitOp::AND, data);
            }
            0xA7 => self.bit_op_a_r(BitOp::AND, self.reg.a),
            // XOR A, r
            0xA8 => self.bit_op_a_r(BitOp::XOR, self.reg.b),
            0xA9 => self.bit_op_a_r(BitOp::XOR, self.reg.c),
            0xAA => self.bit_op_a_r(BitOp::XOR, self.reg.d),
            0xAB => self.bit_op_a_r(BitOp::XOR, self.reg.e),
            0xAC => self.bit_op_a_r(BitOp::XOR, self.get_h_ixh_iyh()),
            0xAD => self.bit_op_a_r(BitOp::XOR, self.get_l_ixl_iyl()),
            0xAE => {
                let data = self.read_hl_ix_iy();
                self.bit_op_a_r(BitOp::XOR, data);
            }
            0xAF => self.bit_op_a_r(BitOp::XOR, self.reg.a),
            // OR A, r
            0xB0 => self.bit_op_a_r(BitOp::OR, self.reg.b),
            0xB1 => self.bit_op_a_r(BitOp::OR, self.reg.c),
            0xB2 => self.bit_op_a_r(BitOp::OR, self.reg.d),
            0xB3 => self.bit_op_a_r(BitOp::OR, self.reg.e),
            0xB4 => self.bit_op_a_r(BitOp::OR, self.get_h_ixh_iyh()),
            0xB5 => self.bit_op_a_r(BitOp::OR, self.get_l_ixl_iyl()),
            0xB6 => {
                let data = self.read_hl_ix_iy();
                self.bit_op_a_r(BitOp::OR, data);
            }
            0xB7 => self.cp_r(self.reg.a),
            // CP A, r
            0xB8 => self.cp_r(self.reg.b),
            0xB9 => self.cp_r(self.reg.c),
            0xBA => self.cp_r(self.reg.d),
            0xBB => self.cp_r(self.reg.e),
            0xBC => self.cp_r(self.get_h_ixh_iyh()),
            0xBD => self.cp_r(self.get_l_ixl_iyl()),
            0xBE => {
                let data = self.read_hl_ix_iy();
                self.cp_r(data);
            }
            0xBF => self.cp_r(self.reg.a),
            // ADD a, n
            0xC6 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.add_a_r(n);
            }
            // SUB A, n
            0xD6 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.sub_a_r(n);
            }
            // AND A, n
            0xE6 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.bit_op_a_r(BitOp::AND, n);
            }
            // OR A, n
            0xF6 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.bit_op_a_r(BitOp::OR, n);
            }
            // ADC A, n
            0xCE => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.adc_a_r(n);
            }
            // SBC A, n
            0xDE => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.sbc_a_r(n);
            }
            // XOR A, n
            0xEE => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.bit_op_a_r(BitOp::XOR, n);
            }
            // CP A, n
            0xFE => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.cp_r(n);
            }
            // INC r
            0x04 => self.reg.b = self.inc_r(self.reg.b),
            0x14 => self.reg.d = self.inc_r(self.reg.d),
            0x24 => {
                let reg = self.inc_r(self.get_h_ixh_iyh());
                self.set_h_ixh_iyh(reg);
            }
            0x34 => {
                let mut n = self.read_hl_ix_iy();
                n = self.inc_r(n);
                self.write_hl_ix_iy(n);
            }
            0x0C => self.reg.c = self.inc_r(self.reg.c),
            0x1C => self.reg.e = self.inc_r(self.reg.e),
            0x2C => {
                let reg = self.inc_r(self.get_l_ixl_iyl());
                self.set_l_ixl_iyl(reg);
            }
            0x3C => self.reg.a = self.inc_r(self.reg.a),
            // DEC r
            0x05 => self.reg.b = self.dec_r(self.reg.b),
            0x15 => self.reg.d = self.dec_r(self.reg.d),
            0x25 => {
                let reg = self.dec_r(self.get_h_ixh_iyh());
                self.set_h_ixh_iyh(reg);
            }
            0x35 => {
                let mut n = self.read_hl_ix_iy();
                n = self.dec_r(n);
                self.write_hl_ix_iy(n);
            }
            0x0D => self.reg.c = self.dec_r(self.reg.c),
            0x1D => self.reg.e = self.dec_r(self.reg.e),
            0x2D => {
                let reg = self.dec_r(self.get_l_ixl_iyl());
                self.set_l_ixl_iyl(reg);
            }
            0x3D => self.reg.a = self.dec_r(self.reg.a),

            // 16-bit arithmetic group
            // ADD HL, rr
            0x09 => self.add_hl_ix_iy_rr(self.reg.get_bc()),
            0x19 => self.add_hl_ix_iy_rr(self.reg.get_de()),
            0x29 => self.add_hl_ix_iy_rr(self.get_hl_ix_iy()),
            0x39 => self.add_hl_ix_iy_rr(self.reg.sp),
            // INC rr
            0x03 => self.reg.set_bc(self.reg.get_bc().wrapping_add(1)),
            0x13 => self.reg.set_de(self.reg.get_de().wrapping_add(1)),
            0x23 => self.set_hl_ix_iy(self.get_hl_ix_iy().wrapping_add(1)),
            0x33 => self.reg.sp = self.reg.sp.wrapping_add(1),
            // DEC rr
            0x0B => self.reg.set_bc(self.reg.get_bc().wrapping_sub(1)),
            0x1B => self.reg.set_de(self.reg.get_de().wrapping_sub(1)),
            0x2B => self.set_hl_ix_iy(self.get_hl_ix_iy().wrapping_sub(1)),
            0x3B => self.reg.sp = self.reg.sp.wrapping_sub(1),

            // Rotate group
            // RLCA
            0x07 => {
                let a = self.reg.a;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x80) == 0x80;
                self.reg.a = a.rotate_left(1);
            }
            // RLA
            0x17 => {
                let a = self.reg.a;
                let c = self.reg.flags.c as u8;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x80) == 0x80;
                self.reg.a = (a.rotate_left(1) & 0xFE) | c;
            }
            // RRCA
            0x0F => {
                let a = self.reg.a;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x01) == 0x01;
                self.reg.a = a.rotate_right(1);
            }
            // RRA
            0x1F => {
                let a = self.reg.a;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x01) == 0x01;
                self.reg.a =
                    (a.rotate_right(1) & 0x7F) | (if self.reg.flags.c { 0x80 } else { 0x00 });
            }
            // DAA
            0x27 => self.daa(),
            // CPL A
            0x2F => self.cpl(),
            // CCF
            0x3F => self.ccf(),
            // SCF
            0x37 => self.scf(),
            // HALT
            0x76 => {
                self.reg.dec_pc();
                self.n_halt = false;
            }
            // DI
            0xF3 => {
                self.iff1 = false;
                self.iff2 = false;
            }
            // EI
            0xFB => {
                self.iff1 = true;
                self.iff2 = true;
            }
            // Special instructions
            0xCB => cycles += self.cb_instructions(), // Bit istructions
            0xED => cycles += self.ed_instructions(), // Misc. instructions
            _ => {} // For 0xDD and 0xFD instructions do something depending on the next opcode
        }
        self.p_inst = instr;
        self.reg.inc_pc();
        cycles
    }
}
