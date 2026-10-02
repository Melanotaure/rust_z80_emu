use crate::bus::{read_io, write_io};
use crate::cycles::{CYCLES, CYCLES_DD_FD};
use crate::z80::*;
// use std::io::{self, Write};

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
        self.reg.flags.alu = false;
        nn
    }

    fn jp_nn(&mut self) {
        let nn = self.get_nn();
        self.reg.pc = nn.wrapping_sub(1);
        self.reg.flags.alu = false;
        // PC is incremented at the end
    }

    fn jr_e(&mut self) {
        self.reg.inc_pc();
        let e = self.bus.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add((e as i8) as u16);
        self.reg.flags.alu = false;
    }

    fn call_nn(&mut self) {
        // let addrl = self.bus.read(self.reg.pc + 1);
        // let addrh = self.bus.read(self.reg.pc + 2);
        // let addr = u16::from_le_bytes([addrl, addrh]);
        // if addr == 0x0005 {
        //     let f = self.reg.c;
        //     if f == 0x02 {
        //         print!("{}", self.reg.e as char);
        //     } else if f == 0x09 {
        //         let mut addr = u16::from_le_bytes([self.reg.e, self.reg.d]);
        //         loop {
        //             let c = self.bus.read(addr);
        //             if c == 0x24 {
        //                 break;
        //             }
        //             print!("{}", c as char);
        //             addr = addr.wrapping_add(1);
        //         }
        //     }
        //     io::stdout().flush().unwrap();
        //     self.reg.inc_pc();
        //     self.reg.inc_pc();
        //     return;
        // }
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
        self.reg.flags.alu = false;
    }

    pub fn ret(&mut self) {
        let pcl = self.bus.read(self.reg.sp);
        self.reg.inc_sp();
        let pch = self.bus.read(self.reg.sp);
        self.reg.inc_sp();
        self.reg.pc = u16::from_le_bytes([pcl, pch]);
        self.reg.dec_pc();
        self.reg.flags.alu = false;
    }

    fn rst(&mut self, addr: u8) {
        let [pcl, pch] = self.reg.pc.to_le_bytes();
        self.reg.dec_sp();
        self.bus.write(self.reg.sp, pch);
        self.reg.dec_sp();
        self.bus.write(self.reg.sp, pcl);
        self.reg.pc = u16::from_le_bytes([addr, 0x00]);
        self.reg.dec_pc();
        self.reg.flags.alu = false;
    }

    fn add_a_r(&mut self, data: u8) {
        let a = self.reg.a;
        let r = a.wrapping_add(data);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.h = (a ^ data ^ r) & 0x10 != 0;
        self.reg.flags.p = ((a ^ r) & (data ^ r) & 0x80) != 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (a as u16) + (data as u16) > 0x00FF;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.a = r;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn adc_a_r(&mut self, data: u8) {
        let c = self.reg.flags.c as u8;
        let a = self.reg.a;
        let r = a.wrapping_add(data).wrapping_add(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 != 0;
        self.reg.flags.h = (a ^ data ^ r) & 0x10 != 0;
        self.reg.flags.p = ((a ^ r) & (data ^ r) & 0x80) != 0;
        self.reg.flags.n = false;
        self.reg.flags.c = (a as u16) + (data as u16) + (c as u16) > 0x00FF;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.a = r;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn sub_a_r(&mut self, data: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = (r as i8) < 0;
        self.reg.flags.h = (a ^ data ^ r) & 0x10 != 0;
        self.reg.flags.p = ((a ^ data) & (a ^ r) & 0x80) != 0;
        self.reg.flags.n = true;
        self.reg.flags.c = (a as u16) < (data as u16);
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.a = r;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn sbc_a_r(&mut self, data: u8) {
        let c = self.reg.flags.c as u8;
        let a = self.reg.a;
        let r = a.wrapping_sub(data).wrapping_sub(c);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.h = (a ^ data ^ r) & 0x10 != 0;
        self.reg.flags.p = ((a ^ data) & (a ^ r) & 0x80) != 0;
        self.reg.flags.n = true;
        self.reg.flags.c = (a as u16) < ((data as u16) + (c as u16));
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.a = r;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn bit_op_a_r(&mut self, bit_op: BitOp, data: u8) {
        let a = self.reg.a;
        let r = match bit_op {
            BitOp::AND => {
                self.reg.flags.h = true;
                a & data
            }
            BitOp::XOR => {
                self.reg.flags.h = false;
                a ^ data
            }
            BitOp::OR => {
                self.reg.flags.h = false;
                a | data
            }
        };
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.p = r.count_ones() % 2 == 0;
        self.reg.flags.n = false;
        self.reg.flags.c = false;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.a = r;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn cp_r(&mut self, data: u8) {
        let a = self.reg.a;
        let r = a.wrapping_sub(data);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.h = (a ^ data ^ r) & 0x10 != 0;
        self.reg.flags.p = (a as i8).overflowing_sub(data as i8).1;
        self.reg.flags.n = true;
        self.reg.flags.c = (a as u16) < (data as u16);
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn inc_r(&mut self, data: u8) -> u8 {
        let r = data.wrapping_add(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.h = (data ^ r) & 0x10 != 0;
        self.reg.flags.p = data == 0x7F;
        self.reg.flags.n = false;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
        r
    }

    pub fn dec_r(&mut self, data: u8) -> u8 {
        let r = data.wrapping_sub(1);
        self.reg.flags.z = r == 0x00;
        self.reg.flags.s = r & 0x80 == 0x80;
        self.reg.flags.h = (data ^ r) & 0x10 != 0;
        self.reg.flags.p = data == 0x80;
        self.reg.flags.n = true;
        self.reg.flags.b5 = r & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = r & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
        r
    }

    fn add_hl_ix_iy_rr(&mut self, reg: u16) {
        let hl = match self.p_inst {
            0xDD => self.reg.get_ix(),
            0xFD => self.reg.get_iy(),
            _ => self.reg.get_hl(),
        };
        let r = hl.wrapping_add(reg);
        self.reg.flags.b5 = r & 0b00100000_00000000 == 0b00100000_00000000;
        self.reg.flags.b3 = r & 0b00001000_00000000 == 0b00001000_00000000;
        self.reg.flags.h = (hl ^ reg ^ r) & 0x1000 != 0;
        self.reg.flags.n = false;
        self.reg.flags.c = hl as u32 + reg as u32 > 0xFFFF;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;

        match self.p_inst {
            0xDD => self.reg.set_ix(r),
            0xFD => self.reg.set_iy(r),
            _ => self.reg.set_hl(r),
        }
    }

    fn daa(&mut self) {
        let mut t = 0;

        if self.reg.flags.h || self.reg.a & 0x0F > 0x09 {
            t += 1;
        }

        if self.reg.flags.c || self.reg.a > 0x99 {
            t += 2;
            self.reg.flags.c = true;
        }

        if self.reg.flags.n && !self.reg.flags.h {
            self.reg.flags.h = false;
        } else {
            if self.reg.flags.n && self.reg.flags.h {
                self.reg.flags.h = self.reg.a & 0x0F < 0x06;
            } else {
                self.reg.flags.h = self.reg.a & 0x0F > 0x09;
            }
        }

        let correction = match t {
            1 => {
                if self.reg.flags.n {
                    0xFA
                } else {
                    0x06
                }
            }
            2 => {
                if self.reg.flags.n {
                    0xA0
                } else {
                    0x60
                }
            }
            3 => {
                if self.reg.flags.n {
                    0x9A
                } else {
                    0x66
                }
            }
            _ => 0x00,
        };
        self.reg.a = self.reg.a.wrapping_add(correction);

        self.reg.flags.s = self.reg.a & 0x80 != 0;
        self.reg.flags.z = self.reg.a == 0;
        self.reg.flags.p = self.reg.a.count_ones() % 2 == 0;
        self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn cpl(&mut self) {
        self.reg.a = !self.reg.a;
        self.reg.flags.h = true;
        self.reg.flags.n = true;
        self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
        self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn ccf(&mut self) {
        self.reg.flags.h = self.reg.flags.c;
        self.reg.flags.n = false;
        self.reg.flags.c = !self.reg.flags.c;
        if self.reg.flags.alu {
            self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
            self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
        } else {
            self.reg.flags.b5 = self.reg.flags.b5 || (self.reg.a & 0b00100000 != 0);
            self.reg.flags.b3 = self.reg.flags.b3 || (self.reg.a & 0b00001000 != 0);
        }
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn scf(&mut self) {
        self.reg.flags.h = false;
        self.reg.flags.n = false;
        self.reg.flags.c = true;
        if self.reg.flags.alu {
            self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
            self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
        } else {
            self.reg.flags.b5 = self.reg.flags.b5 || (self.reg.a & 0b00100000 != 0);
            self.reg.flags.b3 = self.reg.flags.b3 || (self.reg.a & 0b00001000 != 0);
        }
        self.reg.flags.alu = self.reg.flags.to_byte() != 0;
    }

    fn get_h_ixh_iyh(&mut self) -> u8 {
        self.reg.flags.alu = false;
        match self.p_inst {
            0xDD => self.reg.ixh,
            0xFD => self.reg.iyh,
            _ => self.reg.h,
        }
    }

    fn set_h_ixh_iyh(&mut self, reg: u8) {
        self.reg.flags.alu = false;
        match self.p_inst {
            0xDD => self.reg.ixh = reg,
            0xFD => self.reg.iyh = reg,
            _ => self.reg.h = reg,
        };
    }

    fn get_l_ixl_iyl(&mut self) -> u8 {
        self.reg.flags.alu = false;
        match self.p_inst {
            0xDD => self.reg.ixl,
            0xFD => self.reg.iyl,
            _ => self.reg.l,
        }
    }

    fn set_l_ixl_iyl(&mut self, reg: u8) {
        self.reg.flags.alu = false;
        match self.p_inst {
            0xDD => self.reg.ixl = reg,
            0xFD => self.reg.iyl = reg,
            _ => self.reg.l = reg,
        };
    }

    fn get_hl_ix_iy(&mut self) -> u16 {
        self.reg.flags.alu = false;
        match self.p_inst {
            0xDD => self.reg.get_ix(),
            0xFD => self.reg.get_iy(),
            _ => self.reg.get_hl(),
        }
    }

    fn set_hl_ix_iy(&mut self, data: u16) {
        self.reg.flags.alu = false;
        match self.p_inst {
            0xDD => self.reg.set_ix(data),
            0xFD => self.reg.set_iy(data),
            _ => self.reg.set_hl(data),
        };
    }

    pub fn read_hl_ix_iy(&mut self) -> u8 {
        self.reg.flags.alu = false;
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
            _ => self.bus.read(self.reg.get_hl()),
        }
    }

    fn write_hl_ix_iy(&mut self, reg: u8) {
        self.reg.flags.alu = false;
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
            _ => self.bus.write(self.reg.get_hl(), reg),
        }
    }

    // Main function to run the CPU's instructions
    pub fn execute(&mut self) -> u8 {
        let instr = self.bus.read(self.reg.pc);
        let mut cycles = CYCLES[instr as usize];

        // Increment R register at each instruction
        self.reg.inc_r();

        match instr {
            // NOP
            0x00 => self.reg.flags.alu = false,

            // 8-bit load group
            // Destination reg = b
            0x40 => self.reg.flags.alu = false, // LD B, B
            0x41 => {
                self.reg.b = self.reg.c; // LD B, C
                self.reg.flags.alu = false;
            }
            0x42 => {
                self.reg.b = self.reg.d; // LD B, D
                self.reg.flags.alu = false;
            }
            0x43 => {
                self.reg.b = self.reg.e; // LD B, E
                self.reg.flags.alu = false;
            }
            0x44 => self.reg.b = self.get_h_ixh_iyh(), // LD B,L IXL IYL
            0x45 => self.reg.b = self.get_l_ixl_iyl(), // LD B,H IXH IYH
            0x46 => self.reg.b = self.read_hl_ix_iy(), // LD B, (HL IX+d IY+d)
            0x47 => {
                self.reg.b = self.reg.a; // LD B, A
                self.reg.flags.alu = false;
            }
            // Destination reg = c
            0x48 => {
                self.reg.c = self.reg.b; // LD C, B
                self.reg.flags.alu = false;
            }
            0x49 => self.reg.flags.alu = false, // LD C, C
            0x4A => {
                self.reg.c = self.reg.d; // LD C, D
                self.reg.flags.alu = false;
            }
            0x4B => {
                self.reg.c = self.reg.e; // LD C, E
                self.reg.flags.alu = false;
            }
            0x4C => self.reg.c = self.get_h_ixh_iyh(), // LD C, H IXH IYH
            0x4D => self.reg.c = self.get_l_ixl_iyl(), // LD C, L IXL, IYL
            0x4E => self.reg.c = self.read_hl_ix_iy(), // LD C, (HL IX+d IY+d)
            0x4F => {
                self.reg.c = self.reg.a; // LD C, A
                self.reg.flags.alu = false;
            }
            // Destination reg = d
            0x50 => {
                self.reg.d = self.reg.b; // LD D, B
                self.reg.flags.alu = false;
            }
            0x51 => {
                self.reg.d = self.reg.c; // LD D, C
                self.reg.flags.alu = false;
            }
            0x52 => self.reg.flags.alu = false, // LD D, D
            0x53 => {
                self.reg.d = self.reg.e; // LD D, E
                self.reg.flags.alu = false;
            }
            0x54 => self.reg.d = self.get_h_ixh_iyh(), // LD D, H IXH IYH
            0x55 => self.reg.d = self.get_l_ixl_iyl(), // LD D, L IXL IYL
            0x56 => self.reg.d = self.read_hl_ix_iy(), // LD D, (HL IX+d IY+d)
            0x57 => {
                self.reg.d = self.reg.a; // LD D, A
                self.reg.flags.alu = false;
            }
            // Destination reg = e
            0x58 => {
                self.reg.e = self.reg.b; // LD E, B
                self.reg.flags.alu = false;
            }
            0x59 => {
                self.reg.e = self.reg.c; // LD E, C
                self.reg.flags.alu = false;
            }
            0x5A => {
                self.reg.e = self.reg.d; // LD E, D
                self.reg.flags.alu = false;
            }
            0x5B => self.reg.flags.alu = false,        // LD E, E
            0x5C => self.reg.e = self.get_h_ixh_iyh(), // LD E, H IXH IYH
            0x5D => self.reg.e = self.get_l_ixl_iyl(), // LD E, L IXL IYL
            0x5E => self.reg.e = self.read_hl_ix_iy(), // LD E, (HL IX+d IY+d)
            0x5F => {
                self.reg.e = self.reg.a; // LD E, A
                self.reg.flags.alu = false;
            }
            // Destination reg = h
            0x60 => self.set_h_ixh_iyh(self.reg.b), // LD H, B
            0x61 => self.set_h_ixh_iyh(self.reg.c), // LD H, C
            0x62 => self.set_h_ixh_iyh(self.reg.d), // LD H, D
            0x63 => self.set_h_ixh_iyh(self.reg.e), // LD H, E
            0x64 => self.reg.flags.alu = false,     // LD H, H
            0x65 => {
                let reg = self.get_l_ixl_iyl();
                self.set_h_ixh_iyh(reg); // LD H, L
            }
            0x66 => self.reg.h = self.read_hl_ix_iy(), // LD H, (HL IX+d IY+d)
            0x67 => self.set_h_ixh_iyh(self.reg.a),    // LD H, A
            // Destination reg = l
            0x68 => self.set_l_ixl_iyl(self.reg.b), // LD L, B
            0x69 => self.set_l_ixl_iyl(self.reg.c), // LD L, C
            0x6A => self.set_l_ixl_iyl(self.reg.d), // LD L, D
            0x6B => self.set_l_ixl_iyl(self.reg.e), // LD L, E
            0x6C => {
                let reg = self.get_h_ixh_iyh();
                self.set_l_ixl_iyl(reg); // LD L, H
            }
            0x6D => self.reg.flags.alu = false,        // LD L, L
            0x6E => self.reg.l = self.read_hl_ix_iy(), // LD L, (HL IX+d IY+d)
            0x6F => self.set_l_ixl_iyl(self.reg.a),    // LD L, A
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
            0x78 => {
                self.reg.a = self.reg.b; // LD A, B
                self.reg.flags.alu = false;
            }
            0x79 => {
                self.reg.a = self.reg.c; // LD A, C
                self.reg.flags.alu = false;
            }
            0x7A => {
                self.reg.a = self.reg.d; // LD A, D
                self.reg.flags.alu = false;
            }
            0x7B => {
                self.reg.a = self.reg.e; // LD A, E
                self.reg.flags.alu = false;
            }
            0x7C => self.reg.a = self.get_h_ixh_iyh(), // LD A, H
            0x7D => self.reg.a = self.get_l_ixl_iyl(), // LD A, L
            0x7E => self.reg.a = self.read_hl_ix_iy(), // LD A, (HL)
            0x7F => self.reg.flags.alu = false,        // LD A, A
            // LD r, n
            0x06 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.b = n;
                self.reg.flags.alu = false;
            }
            0x16 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.d = n;
                self.reg.flags.alu = false;
            }
            0x26 => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.set_h_ixh_iyh(n);
            }
            0x36 => {
                let n = if self.p_inst == 0xDD || self.p_inst == 0xFD {
                    // LD (IX+d IY+d), n -> d is first byte, n is second byte (xxyyddnn)
                    self.bus.read(self.reg.pc.wrapping_add(2))
                } else {
                    // LD (HL), n -> n is first byte (xxyynn)
                    self.bus.read(self.reg.pc.wrapping_add(1))
                };
                self.write_hl_ix_iy(n);
                self.reg.inc_pc();
            }
            0x0E => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.c = n;
                self.reg.flags.alu = false;
            }
            0x1E => {
                self.reg.inc_pc();
                let n = self.bus.read(self.reg.pc);
                self.reg.e = n;
                self.reg.flags.alu = false;
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
                self.reg.flags.alu = false;
            }
            // LD (BC), A
            0x02 => {
                self.bus.write(self.reg.get_bc(), self.reg.a);
                self.reg.flags.alu = false;
            }
            // LD (DE), A
            0x12 => {
                self.bus.write(self.reg.get_de(), self.reg.a);
                self.reg.flags.alu = false;
            }
            // LD (nn), A
            0x32 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.reg.a);
                self.reg.flags.alu = false;
            }
            // LD A, (BC)
            0x0A => {
                self.reg.a = self.bus.read(self.reg.get_bc());
                self.reg.flags.alu = false;
            }
            // LD A, (DE)
            0x1A => {
                self.reg.a = self.bus.read(self.reg.get_de());
                self.reg.flags.alu = false;
            }
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
                let data = self.get_l_ixl_iyl();
                self.bus.write(nn, data);
                let data = self.get_h_ixh_iyh();
                self.bus.write(nn.wrapping_add(1), data);
            }
            // LD SP, HL
            0xF9 => self.reg.sp = u16::from_le_bytes([self.get_l_ixl_iyl(), self.get_h_ixh_iyh()]),
            // PUSH BC
            0xC5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.b);
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.c);
                self.reg.flags.alu = false;
            }
            // PUSH DE
            0xD5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.d);
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.e);
                self.reg.flags.alu = false;
            }
            // PUSH HL IX IY
            0xE5 => {
                self.reg.dec_sp();
                let data = self.get_h_ixh_iyh();
                self.bus.write(self.reg.sp, data);
                self.reg.dec_sp();
                let data = self.get_l_ixl_iyl();
                self.bus.write(self.reg.sp, data);
            }
            // PUSH AF
            0xF5 => {
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.a);
                self.reg.dec_sp();
                self.bus.write(self.reg.sp, self.reg.flags.to_byte());
                self.reg.flags.alu = false;
            }
            // POP BC
            0xC1 => {
                self.reg.c = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.b = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.flags.alu = false;
            }
            // POP DE
            0xD1 => {
                self.reg.e = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.d = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.flags.alu = false;
            }
            // POP HL IX IY
            0xE1 => {
                self.set_l_ixl_iyl(self.bus.read(self.reg.sp));
                self.reg.inc_sp();
                self.set_h_ixh_iyh(self.bus.read(self.reg.sp));
                self.reg.inc_sp();
                self.reg.flags.alu = false;
            }
            // POP AF
            0xF1 => {
                let f = self.bus.read(self.reg.sp);
                self.reg.flags.from_byte(f);
                self.reg.inc_sp();
                self.reg.a = self.bus.read(self.reg.sp);
                self.reg.inc_sp();
                self.reg.flags.alu = false;
            }
            // Exchange
            // EX DE, HL
            0xEB => {
                let de = self.reg.get_de();
                let hl = self.reg.get_hl();
                self.reg.set_de(hl);
                self.reg.set_hl(de);
                self.reg.flags.alu = false;
            }
            // EX AF,AF'
            0x08 => {
                let af = self.reg.get_af();
                let eaf = self.reg.eaf;
                self.reg.set_af(eaf);
                self.reg.eaf = af;
                self.reg.flags.alu = false;
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
                self.reg.flags.alu = false;
            }
            // EX (SP), HL IX IY
            0xE3 => {
                let n = self.bus.read(self.reg.sp);
                let data = self.get_l_ixl_iyl();
                self.bus.write(self.reg.sp, data);
                self.set_l_ixl_iyl(n);
                self.reg.inc_sp();
                let n = self.bus.read(self.reg.sp);
                let data = self.get_h_ixh_iyh();
                self.bus.write(self.reg.sp, data);
                self.set_h_ixh_iyh(n);
                self.reg.flags.alu = false;
            }

            // Jump group
            // JP nn
            0xC3 => self.jp_nn(),
            // JP nz, nn
            0xC2 => {
                if !self.reg.flags.z {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP z, nn
            0xCA => {
                if self.reg.flags.z {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP nc, nn
            0xD2 => {
                if !self.reg.flags.c {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP c, nn
            0xDA => {
                if self.reg.flags.c {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP po, nn
            0xE2 => {
                if !self.reg.flags.p {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP pe, nn
            0xEA => {
                if self.reg.flags.p {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP p, nn
            0xF2 => {
                if !self.reg.flags.s {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JP m, nn
            0xFA => {
                if self.reg.flags.s {
                    self.jp_nn();
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(2);
                }
            }
            // JR e
            0x18 => self.jr_e(),
            // JR z, e
            0x28 => {
                if self.reg.flags.z {
                    self.jr_e();
                } else {
                    self.reg.inc_pc();
                }
                self.reg.flags.alu = false;
            }
            // JR c, e
            0x38 => {
                if self.reg.flags.c {
                    self.jr_e();
                } else {
                    self.reg.inc_pc();
                }
                self.reg.flags.alu = false;
            }
            // DJNZ e
            0x10 => {
                self.reg.b = self.reg.b.wrapping_sub(1);
                if self.reg.b != 0 {
                    self.jr_e();
                    cycles += 5;
                } else {
                    self.reg.inc_pc();
                }
                self.reg.flags.alu = false;
            }
            // JR nz, e
            0x20 => {
                if !self.reg.flags.z {
                    self.jr_e();
                } else {
                    self.reg.inc_pc();
                }
                self.reg.flags.alu = false;
            }
            // JR nc, nn
            0x30 => {
                if !self.reg.flags.c {
                    self.jr_e();
                } else {
                    self.reg.inc_pc();
                }
                self.reg.flags.alu = false;
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
                if !self.reg.flags.s {
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
            // CALL m, nn
            0xFC => {
                if self.reg.flags.s {
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
                if !self.reg.flags.s {
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
            // RET m
            0xF8 => {
                if self.reg.flags.s {
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
            0x84 => {
                let data = self.get_h_ixh_iyh();
                self.add_a_r(data);
            }
            0x85 => {
                let data = self.get_l_ixl_iyl();
                self.add_a_r(data);
            }
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
            0x8C => {
                let data = self.get_h_ixh_iyh();
                self.adc_a_r(data);
            }
            0x8D => {
                let data = self.get_l_ixl_iyl();
                self.adc_a_r(data);
            }
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
            0x94 => {
                let data = self.get_h_ixh_iyh();
                self.sub_a_r(data);
            }
            0x95 => {
                let data = self.get_l_ixl_iyl();
                self.sub_a_r(data);
            }
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
            0x9C => {
                let data = self.get_h_ixh_iyh();
                self.sbc_a_r(data);
            }
            0x9D => {
                let data = self.get_l_ixl_iyl();
                self.sbc_a_r(data);
            }
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
            0xA4 => {
                let data = self.get_h_ixh_iyh();
                self.bit_op_a_r(BitOp::AND, data);
            }
            0xA5 => {
                let data = self.get_l_ixl_iyl();
                self.bit_op_a_r(BitOp::AND, data);
            }
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
            0xAC => {
                let data = self.get_h_ixh_iyh();
                self.bit_op_a_r(BitOp::XOR, data);
            }
            0xAD => {
                let data = self.get_l_ixl_iyl();
                self.bit_op_a_r(BitOp::XOR, data);
            }
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
            0xB4 => {
                let data = self.get_h_ixh_iyh();
                self.bit_op_a_r(BitOp::OR, data);
            }
            0xB5 => {
                let data = self.get_l_ixl_iyl();
                self.bit_op_a_r(BitOp::OR, data);
            }
            0xB6 => {
                let data = self.read_hl_ix_iy();
                self.bit_op_a_r(BitOp::OR, data);
            }
            0xB7 => self.bit_op_a_r(BitOp::OR, self.reg.a),
            // CP A, r
            0xB8 => self.cp_r(self.reg.b),
            0xB9 => self.cp_r(self.reg.c),
            0xBA => self.cp_r(self.reg.d),
            0xBB => self.cp_r(self.reg.e),
            0xBC => {
                let data = self.get_h_ixh_iyh();
                self.cp_r(data);
            }
            0xBD => {
                let data = self.get_l_ixl_iyl();
                self.cp_r(data);
            }
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
                let data = self.get_h_ixh_iyh();
                let reg = self.inc_r(data);
                self.set_h_ixh_iyh(reg);
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            0x34 => {
                let mut n = self.read_hl_ix_iy();
                n = self.inc_r(n);
                self.write_hl_ix_iy(n);
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            0x0C => self.reg.c = self.inc_r(self.reg.c),
            0x1C => self.reg.e = self.inc_r(self.reg.e),
            0x2C => {
                let data = self.get_l_ixl_iyl();
                let reg = self.inc_r(data);
                self.set_l_ixl_iyl(reg);
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            0x3C => self.reg.a = self.inc_r(self.reg.a),
            // DEC r
            0x05 => self.reg.b = self.dec_r(self.reg.b),
            0x15 => self.reg.d = self.dec_r(self.reg.d),
            0x25 => {
                let data = self.get_h_ixh_iyh();
                let reg = self.dec_r(data);
                self.set_h_ixh_iyh(reg);
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            0x35 => {
                let mut n = self.read_hl_ix_iy();
                n = self.dec_r(n);
                self.write_hl_ix_iy(n);
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            0x0D => self.reg.c = self.dec_r(self.reg.c),
            0x1D => self.reg.e = self.dec_r(self.reg.e),
            0x2D => {
                let data = self.get_l_ixl_iyl();
                let reg = self.dec_r(data);
                self.set_l_ixl_iyl(reg);
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            0x3D => self.reg.a = self.dec_r(self.reg.a),

            // 16-bit arithmetic group
            // ADD HL, rr
            0x09 => self.add_hl_ix_iy_rr(self.reg.get_bc()),
            0x19 => self.add_hl_ix_iy_rr(self.reg.get_de()),
            0x29 => {
                let reg = self.get_hl_ix_iy();
                self.add_hl_ix_iy_rr(reg);
            }
            0x39 => self.add_hl_ix_iy_rr(self.reg.sp),
            // INC rr
            0x03 => {
                self.reg.set_bc(self.reg.get_bc().wrapping_add(1));
                self.reg.flags.alu = false;
            }
            0x13 => {
                self.reg.set_de(self.reg.get_de().wrapping_add(1));
                self.reg.flags.alu = false;
            }
            0x23 => {
                let data = self.get_hl_ix_iy().wrapping_add(1);
                self.set_hl_ix_iy(data);
            }
            0x33 => {
                self.reg.sp = self.reg.sp.wrapping_add(1);
                self.reg.flags.alu = false;
            }
            // DEC rr
            0x0B => {
                self.reg.set_bc(self.reg.get_bc().wrapping_sub(1));
                self.reg.flags.alu = false;
            }
            0x1B => {
                self.reg.set_de(self.reg.get_de().wrapping_sub(1));
                self.reg.flags.alu = false;
            }
            0x2B => {
                let data = self.get_hl_ix_iy().wrapping_sub(1);
                self.set_hl_ix_iy(data);
            }
            0x3B => {
                self.reg.sp = self.reg.sp.wrapping_sub(1);
                self.reg.flags.alu = false;
            }

            // Rotate group
            // RLCA
            0x07 => {
                let a = self.reg.a;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x80) == 0x80;
                self.reg.a = a.rotate_left(1);
                self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
                self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
                self.reg.flags.alu = self.reg.flags.to_byte() != 0;
            }
            // RLA
            0x17 => {
                let a = self.reg.a;
                let c = self.reg.flags.c as u8;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x80) == 0x80;
                self.reg.a = (a.rotate_left(1) & 0xFE) | c;
                self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
                self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
                self.reg.flags.alu = self.reg.flags.to_byte() != 0
            }
            // RRCA
            0x0F => {
                let a = self.reg.a;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                self.reg.flags.c = (a & 0x01) == 0x01;
                self.reg.a = a.rotate_right(1);
                self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
                self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
                self.reg.flags.alu = self.reg.flags.to_byte() != 0
            }
            // RRA
            0x1F => {
                let a = self.reg.a;
                self.reg.flags.h = false;
                self.reg.flags.n = false;
                let carry = (a & 0x01) == 0x01;
                self.reg.a =
                    (a.rotate_right(1) & 0x7F) | (if self.reg.flags.c { 0x80 } else { 0x00 });
                self.reg.flags.c = carry;
                self.reg.flags.b5 = self.reg.a & 0b00100000 == 0b00100000;
                self.reg.flags.b3 = self.reg.a & 0b00001000 == 0b00001000;
                self.reg.flags.alu = self.reg.flags.to_byte() != 0
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
                self.n_halt = false;
                self.reg.flags.alu = false;
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
            0xCB => cycles += self.cb_instructions(), // Bit instructions
            0xED => cycles += self.ed_instructions(), // Misc. instructions
            _ => {} // For 0xDD and 0xFD instructions do something depending on the next opcode
        }
        if self.p_inst == 0xDD || self.p_inst == 0xFD {
            cycles += CYCLES_DD_FD[instr as usize];
        }
        self.p_inst = instr;
        self.reg.inc_pc();
        cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // JR e
    #[test]
    fn jr_e_pos() {
        let mut cpu = Z80::new();

        cpu.bus.write(0x0001, 0x03);
        let flags = cpu.reg.get_af() & 0x0F;
        cpu.jr_e();
        assert_eq!(cpu.reg.pc, 0x0004);
        assert_eq!(cpu.reg.get_af() & 0x0F, flags);
    }

    #[test]
    fn jr_e_neg() {
        let mut cpu = Z80::new();

        cpu.bus.write(0x0481, 0xFA);
        cpu.reg.pc = 0x480;
        let flags = cpu.reg.get_af() & 0x0F;
        cpu.jr_e();
        assert_eq!(cpu.reg.pc, 0x047B);
        assert_eq!(cpu.reg.get_af() & 0x0F, flags);
    }

    #[test]
    fn call_nn_nominal() {
        let mut cpu = Z80::new();

        cpu.bus.write(0x0181, 0x35);
        cpu.bus.write(0x0182, 0x21);
        cpu.reg.pc = 0x0180;
        cpu.reg.sp = 0x3002;
        let flags = cpu.reg.get_af() & 0x0F;
        cpu.call_nn();
        assert_eq!(cpu.reg.pc, 0x2134); // PC-1 for PC is incremented at each fetch instruction loop
        assert_eq!(cpu.reg.sp, 0x3000);
        assert_eq!(cpu.bus.read(0x3001), 0x01);
        assert_eq!(cpu.bus.read(0x3000), 0x83);
        assert_eq!(cpu.reg.get_af() & 0x0F, flags);
    }

    #[test]
    fn ret_nominal() {
        let mut cpu = Z80::new();

        cpu.bus.write(0x2000, 0xB5);
        cpu.bus.write(0x2001, 0x18);
        cpu.reg.pc = 0x3535;
        cpu.reg.sp = 0x2000;
        let flags = cpu.reg.get_af() & 0x0F;
        cpu.ret();
        assert_eq!(cpu.reg.pc, 0x18B4); // PC-1 for PC is incremented at each fetch instruction loop
        assert_eq!(cpu.reg.sp, 0x2002);
        assert_eq!(cpu.reg.get_af() & 0x0F, flags);
    }

    #[test]
    fn rst_18() {
        let mut cpu = Z80::new();

        cpu.reg.pc = 0x0180;
        cpu.reg.sp = 0x3002;
        let flags = cpu.reg.get_af() & 0x0F;
        cpu.rst(0x18);
        assert_eq!(cpu.reg.pc, 0x0017); // PC-1 for PC is incremented at each fetch instruction loop
        assert_eq!(cpu.reg.sp, 0x3000);
        assert_eq!(cpu.bus.read(0x3001), 0x01);
        assert_eq!(cpu.bus.read(0x3000), 0x80);
        assert_eq!(cpu.reg.get_af() & 0x0F, flags);
    }

    // ADD a, r
    #[test]
    fn add_a_r_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x5A;
        cpu.add_a_r(0x11);
        assert_eq!(cpu.reg.a, 0x6B);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn add_a_r_zero() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0xFF;
        cpu.add_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x00);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, true);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn add_a_r_ovf_pos() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x7F;
        cpu.add_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x80);
        assert_eq!(cpu.reg.flags.s, true);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn add_a_r_ovf_neg() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x80;
        cpu.add_a_r(0xFF);
        assert_eq!(cpu.reg.a, 0x7F);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    // ADC a, r
    #[test]
    fn adc0_a_r_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x5A;
        cpu.reg.flags.c = false;
        cpu.adc_a_r(0x11);
        assert_eq!(cpu.reg.a, 0x6B);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn adc1_a_r_zero() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0xFE;
        cpu.reg.flags.c = true;
        cpu.adc_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x00);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, true);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn adc1_a_r_ovf_pos() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x7E;
        cpu.reg.flags.c = true;
        cpu.adc_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x80);
        assert_eq!(cpu.reg.flags.s, true);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn adc1_a_r_ovf_neg() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x80;
        cpu.reg.flags.c = true;
        cpu.adc_a_r(0xFE);
        assert_eq!(cpu.reg.a, 0x7F);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    // SUB a, r
    #[test]
    fn sub_a_r_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x5A;
        cpu.sub_a_r(0x11);
        assert_eq!(cpu.reg.a, 0x49);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn sub_a_r_zero() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x01;
        cpu.sub_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x00);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, true);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn sub_a_r_ovf_pos() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x7F;
        cpu.sub_a_r(0xFF);
        assert_eq!(cpu.reg.a, 0x80);
        assert_eq!(cpu.reg.flags.s, true);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn sub_a_r_ovf_neg() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x80;
        cpu.sub_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x7F);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, false);
    }

    // SBC a, r
    #[test]
    fn sbc0_a_r_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x5A;
        cpu.reg.flags.c = false;
        cpu.sbc_a_r(0x11);
        assert_eq!(cpu.reg.a, 0x49);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn sbc1_a_r_zero() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x02;
        cpu.reg.flags.c = true;
        cpu.sbc_a_r(0x01);
        assert_eq!(cpu.reg.a, 0x00);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, true);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn sbc1_a_r_ovf_pos() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x7F;
        cpu.reg.flags.c = true;
        cpu.sbc_a_r(0xFE);
        assert_eq!(cpu.reg.a, 0x80);
        assert_eq!(cpu.reg.flags.s, true);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn sbc1_a_r_ovf_neg() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x80;
        cpu.reg.flags.c = true;
        cpu.sbc_a_r(0x00);
        assert_eq!(cpu.reg.a, 0x7F);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, true);
        assert_eq!(cpu.reg.flags.c, false);
    }

    // ADD HL, rr
    #[test]
    fn add_hl_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.set_hl(0x5A5A);
        cpu.p_inst = 0x00;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x1111);
        assert_eq!(cpu.reg.get_hl(), 0x6B6B);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn add_hl_zero() {
        let mut cpu = Z80::new();

        cpu.reg.set_hl(0xFFFE);
        cpu.p_inst = 0x00;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x0002);
        assert_eq!(cpu.reg.get_hl(), 0x0000);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn add_hl_neg() {
        let mut cpu = Z80::new();

        cpu.reg.set_hl(0x7FFE);
        cpu.p_inst = 0x00;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x0003);
        assert_eq!(cpu.reg.get_hl(), 0x8001);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    // ADD IX, rr
    #[test]
    fn add_ix_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.set_ix(0x5A5A);
        cpu.p_inst = 0xDD; // for IX
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x1111);
        assert_eq!(cpu.reg.get_ix(), 0x6B6B);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn add_ix_zero() {
        let mut cpu = Z80::new();

        cpu.reg.set_ix(0xFFFE);
        cpu.p_inst = 0xdd;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x0002);
        assert_eq!(cpu.reg.get_ix(), 0x0000);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn add_ix_neg() {
        let mut cpu = Z80::new();

        cpu.reg.set_ix(0x7FFE);
        cpu.p_inst = 0xdd;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x0003);
        assert_eq!(cpu.reg.get_ix(), 0x8001);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    // ADD IY, rr
    #[test]
    fn add_iy_nominal() {
        let mut cpu = Z80::new();

        cpu.reg.set_iy(0x5A5A);
        cpu.p_inst = 0xfd;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x1111);
        assert_eq!(cpu.reg.get_iy(), 0x6B6B);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn add_iy_zero() {
        let mut cpu = Z80::new();

        cpu.reg.set_iy(0xFFFE);
        cpu.p_inst = 0xfd;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x0002);
        assert_eq!(cpu.reg.get_iy(), 0x0000);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn add_iy_neg() {
        let mut cpu = Z80::new();

        cpu.reg.set_iy(0x7FFE);
        cpu.p_inst = 0xfd;
        let s = cpu.reg.flags.s;
        let z = cpu.reg.flags.z;
        let p = cpu.reg.flags.p;
        cpu.add_hl_ix_iy_rr(0x0003);
        assert_eq!(cpu.reg.get_iy(), 0x8001);
        assert_eq!(cpu.reg.flags.s, s);
        assert_eq!(cpu.reg.flags.z, z);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, p);
        assert_eq!(cpu.reg.flags.n, false);
        assert_eq!(cpu.reg.flags.c, false);
    }

    // DAA
    #[test]
    fn daa_add_7_3() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x07;
        cpu.add_a_r(0x03);
        let n = cpu.reg.flags.n;
        cpu.daa();
        assert_eq!(cpu.reg.a, 0x10);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, n);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn daa_add_99_1() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x99;
        cpu.add_a_r(0x01);
        let n = cpu.reg.flags.n;
        cpu.daa();
        assert_eq!(cpu.reg.a, 0x00);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, true);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, n);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn daa_inc_99() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x99;
        cpu.reg.a = cpu.inc_r(cpu.reg.a);
        let n = cpu.reg.flags.n;
        cpu.daa();
        assert_eq!(cpu.reg.a, 0x00);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, true);
        assert_eq!(cpu.reg.flags.h, true);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, n);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn daa_sub_26_7() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x26;
        cpu.sub_a_r(0x07);
        let n = cpu.reg.flags.n;
        cpu.daa();
        assert_eq!(cpu.reg.a, 0x19);
        assert_eq!(cpu.reg.flags.s, false);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, false);
        assert_eq!(cpu.reg.flags.n, n);
        assert_eq!(cpu.reg.flags.c, false);
    }

    #[test]
    fn daa_sub_01_5() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x01;
        cpu.sub_a_r(0x05);
        let n = cpu.reg.flags.n;
        cpu.daa();
        assert_eq!(cpu.reg.a, 0x96);
        assert_eq!(cpu.reg.flags.s, true);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, n);
        assert_eq!(cpu.reg.flags.c, true);
    }

    #[test]
    fn daa_dec_00() {
        let mut cpu = Z80::new();

        cpu.reg.a = 0x00;
        cpu.reg.a = cpu.dec_r(cpu.reg.a);
        let n = cpu.reg.flags.n;
        cpu.daa();
        assert_eq!(cpu.reg.a, 0x99);
        assert_eq!(cpu.reg.flags.s, true);
        assert_eq!(cpu.reg.flags.z, false);
        assert_eq!(cpu.reg.flags.h, false);
        assert_eq!(cpu.reg.flags.p, true);
        assert_eq!(cpu.reg.flags.n, n);
        assert_eq!(cpu.reg.flags.c, true);
    }
}
