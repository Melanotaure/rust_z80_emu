use crate::bus::{read_io, write_io, Bus};
use crate::cycles::CYCLES;
use crate::registers::Registers;

enum BitOp {
    AND,
    XOR,
    OR,
}

// Structure of the Z80 processor
pub struct Z80 {
    // Registers
    pub regs: Registers,
    // Address bus and Data bus
    pub bus: Bus,
    // System control pins
    pub n_m1: bool,
    pub n_mreq: bool,
    pub n_iorq: bool,
    pub n_rd: bool,
    pub n_wr: bool,
    pub n_rfsh: bool,
    // CPU control
    pub n_halt: bool,
    pub n_wait: bool,
    pub n_int: bool,
    pub n_nmi: bool,
    pub n_reset: bool,
    // CPU bus control
    pub n_busrq: bool,
    pub n_busack: bool,
    // Added the clock pin but might not be used
    pub _clock: u64,
}

impl Z80 {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            bus: Bus::new(),
            n_m1: true,
            n_mreq: true,
            n_iorq: true,
            n_rd: true,
            n_wr: true,
            n_rfsh: true,
            n_halt: true,
            n_wait: true,
            n_int: true,
            n_nmi: true,
            n_reset: true,
            n_busrq: true,
            n_busack: true,
            _clock: 0_u64,
        }
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.bus.reset();
        self.n_busack = true;
        self.n_busrq = true;
        self.n_halt = true;
        self.n_int = true;
        self.n_iorq = true;
        self.n_m1 = true;
        self.n_mreq = true;
        self.n_nmi = true;
        self.n_rd = true;
        self.n_reset = true;
        self.n_rfsh = true;
        self.n_wait = true;
        self.n_wr = true;
        self._clock = 0;
    }

    fn get_nn(&mut self) -> u16 {
        self.regs.inc_pc();
        let nl = self.bus.read(self.regs.pc);
        self.regs.inc_pc();
        let nh = self.bus.read(self.regs.pc);
        let nn = u16::from_le_bytes([nl, nh]);
        nn
    }

    fn jp_nn(&mut self) {
        let nn = self.get_nn();
        self.regs.pc = nn.wrapping_sub(1);
        // PC is incremented at the end
    }

    fn jr_e(&mut self) {
        self.regs.inc_pc();
        let e = self.bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add((e as i8) as u16);
    }

    fn call_nn(&mut self) {
        // PC is first incremented by 3 to resume the flow after this 3-byte instruction
        let pc = self.regs.pc.wrapping_add(3);
        let [mut pcl, mut pch] = pc.to_le_bytes();
        self.regs.dec_sp();
        self.bus.write(self.regs.sp, pch);
        self.regs.dec_sp();
        self.bus.write(self.regs.sp, pcl);
        self.regs.inc_pc();
        pcl = self.bus.read(self.regs.pc);
        self.regs.inc_pc();
        pch = self.bus.read(self.regs.pc);
        self.regs.pc = u16::from_le_bytes([pcl, pch]);
        self.regs.dec_pc();
    }

    fn ret(&mut self) {
        let pcl = self.bus.read(self.regs.sp);
        self.regs.inc_sp();
        let pch = self.bus.read(self.regs.sp);
        self.regs.pc = u16::from_be_bytes([pcl, pch]);
        self.regs.dec_pc();
    }

    fn rst(&mut self, addr: u8) {
        let [pcl, pch] = self.regs.pc.to_be_bytes();
        self.regs.dec_sp();
        self.bus.write(self.regs.sp, pch);
        self.regs.dec_sp();
        self.bus.write(self.regs.sp, pcl);
        self.regs.pc = u16::from_le_bytes([addr, 0x00]);
        self.regs.dec_pc();
    }

    fn add_a_r(&mut self, data: u8) {
        let a = self.regs.a;
        let r = a.wrapping_add(data);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = (a & 0x0F) + (data & 0x0F) > 0x0F;
        self.regs.flags.p = (a as i8).overflowing_add(data as i8).1;
        self.regs.flags.n = false;
        self.regs.flags.c = (a as u16) + (data as u16) > 0x00FF;
        self.regs.a = r;
    }

    fn adc_a_r(&mut self, data: u8) {
        let c = self.regs.flags.c as u8;
        let a = self.regs.a;
        let r = a.wrapping_add(data).wrapping_add(c);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = (a & 0x0F) + (data & 0x0F) + c > 0x0F;
        self.regs.flags.p = (a as i8).overflowing_add((data.wrapping_add(c)) as i8).1;
        self.regs.flags.n = false;
        self.regs.flags.c = (a as u16) + (data as u16) + (c as u16) > 0x00FF;
        self.regs.a = r;
    }

    fn sub_a_r(&mut self, data: u8) {
        let a = self.regs.a;
        let r = a.wrapping_sub(data);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = (a & 0x0F) < (data & 0x0F);
        self.regs.flags.p = (a as i8).overflowing_sub(data as i8).1;
        self.regs.flags.n = true;
        self.regs.flags.c = (a as u16) < (data as u16);
        self.regs.a = r;
    }

    fn sbc_a_r(&mut self, data: u8) {
        let c = self.regs.flags.c as u8;
        let a = self.regs.a;
        let r = a.wrapping_sub(data).wrapping_sub(c);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = (a & 0x0F) < (data & 0x0F).wrapping_add(c);
        self.regs.flags.p = (a as i8).overflowing_sub((data.wrapping_add(c)) as i8).1;
        self.regs.flags.n = true;
        self.regs.flags.c = (a as u16) < ((data as u16) + (c as u16));
        self.regs.a = r;
    }

    fn bit_op_a_r(&mut self, bit_op: BitOp, data: u8) {
        let a = self.regs.a;
        let r = match bit_op {
            BitOp::AND => a & data,
            BitOp::XOR => a ^ data,
            BitOp::OR => a | data,
        };
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = true;
        self.regs.flags.p = r.count_ones() & 0x01 == 0;
        self.regs.flags.n = false;
        self.regs.flags.c = false;
        self.regs.a = r;
    }

    fn cp_r(&mut self, data: u8) {
        let a = self.regs.a;
        let r = a.wrapping_sub(data);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = (a & 0x0F) < (data & 0x0F);
        self.regs.flags.p = (a as i8).overflowing_sub(data as i8).1;
        self.regs.flags.n = true;
        self.regs.flags.c = (a as u16) < (data as u16);
    }

    fn inc_r(&mut self, data: u8) -> u8 {
        let r = data.wrapping_add(1);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = data & 0x0F == 0x0F;
        self.regs.flags.p = data == 0x7F;
        self.regs.flags.n = false;
        r
    }

    fn dec_r(&mut self, data: u8) -> u8 {
        let r = data.wrapping_sub(1);
        self.regs.flags.z = r == 0x00;
        self.regs.flags.s = (r as i8) < 0;
        self.regs.flags.h = data & 0x1F == 0x10;
        self.regs.flags.p = data == 0x80;
        self.regs.flags.n = true;
        r
    }

    fn add_hl_rr(&mut self, reg: u16) {
        let hl = self.regs.get_hl();
        self.regs.flags.h = (hl & 0x0FFF) + (reg & 0x0FFF) > 0x0FFF;
        self.regs.flags.n = false;
        self.regs.flags.c = hl as u32 + reg as u32 > 0xFFFF;
        self.regs.set_hl(hl.wrapping_add(reg));
    }

    // Main function to run the CPU's instructions
    pub fn execute(&mut self) -> u8 {
        let instr = self.bus.read(self.regs.pc);
        let mut cycles = CYCLES[instr as usize];

        match instr {
            // NOP
            0x00 => {}

            // 8-bit load group
            // Destination reg = b
            0x40 => {}                                               // LD B, B
            0x41 => self.regs.b = self.regs.c,                       // LD B, C
            0x42 => self.regs.b = self.regs.d,                       // LD B, D
            0x43 => self.regs.b = self.regs.e,                       // LD B, E
            0x44 => self.regs.b = self.regs.h,                       // LD B, H
            0x45 => self.regs.b = self.regs.l,                       // LD B, L
            0x46 => self.regs.b = self.bus.read(self.regs.get_hl()), // LD B, (HL)
            0x47 => self.regs.b = self.regs.a,                       // LD B, A
            // Destination reg = c
            0x48 => self.regs.c = self.regs.b, // LD C, B
            0x49 => {}                         // LD C, C
            0x4A => self.regs.c = self.regs.d, // LD C, D
            0x4B => self.regs.c = self.regs.e, // LD C, E
            0x4C => self.regs.c = self.regs.h, // LD C, H
            0x4D => self.regs.c = self.regs.l, // LD C, L
            0x4E => self.regs.c = self.bus.read(self.regs.get_hl()), // LD C, (HL)
            0x4F => self.regs.c = self.regs.a, // LD C, A
            // Destination reg = d
            0x50 => self.regs.d = self.regs.b, // LD D, B
            0x51 => self.regs.d = self.regs.c, // LD D, C
            0x52 => {}                         // LD D, D
            0x53 => self.regs.d = self.regs.e, // LD D, E
            0x54 => self.regs.d = self.regs.h, // LD D, H
            0x55 => self.regs.d = self.regs.l, // LD D, L
            0x56 => self.regs.d = self.bus.read(self.regs.get_hl()), // LD D, (HL)
            0x57 => self.regs.d = self.regs.a, // LD D, A
            // Destination reg = e
            0x58 => self.regs.e = self.regs.b, // LD E, B
            0x59 => self.regs.e = self.regs.c, // LD E, C
            0x5A => self.regs.e = self.regs.d, // LD E, D
            0x5B => {}                         // LD E, E
            0x5C => self.regs.e = self.regs.h, // LD E, H
            0x5D => self.regs.e = self.regs.l, // LD E, L
            0x5E => self.regs.e = self.bus.read(self.regs.get_hl()), // LD E, (HL)
            0x5F => self.regs.e = self.regs.a, // LD E, A
            // Destination reg = h
            0x60 => self.regs.h = self.regs.b, // LD H, B
            0x61 => self.regs.h = self.regs.c, // LD H, C
            0x62 => self.regs.h = self.regs.d, // LD H, D
            0x63 => self.regs.h = self.regs.e, // LD H, E
            0x64 => {}                         // LD H, H
            0x65 => self.regs.h = self.regs.l, // LD H, L
            0x66 => self.regs.h = self.bus.read(self.regs.get_hl()), // LD H, (HL)
            0x67 => self.regs.h = self.regs.a, // LD H, A
            // Destination reg = l
            0x68 => self.regs.l = self.regs.b, // LD L, B
            0x69 => self.regs.l = self.regs.c, // LD L, C
            0x6A => self.regs.l = self.regs.d, // LD L, D
            0x6B => self.regs.l = self.regs.e, // LD L, E
            0x6C => self.regs.l = self.regs.h, // LD L, H
            0x6D => {}                         // LD L, L
            0x6E => self.regs.l = self.bus.read(self.regs.get_hl()), // LD L, (HL)
            0x6F => self.regs.l = self.regs.a, // LD L, A
            // Destination reg = (hl)
            0x70 => self.bus.write(self.regs.get_hl(), self.regs.b), // LD (HL), B
            0x71 => self.bus.write(self.regs.get_hl(), self.regs.c), // LD (HL), C
            0x72 => self.bus.write(self.regs.get_hl(), self.regs.d), // LD (HL), D
            0x73 => self.bus.write(self.regs.get_hl(), self.regs.e), // LD (HL), E
            0x74 => self.bus.write(self.regs.get_hl(), self.regs.h), // LD (HL), H
            0x75 => self.bus.write(self.regs.get_hl(), self.regs.l), // LD (HL), L
            // 0x76 => HALT treated elsewhere
            0x77 => self.bus.write(self.regs.get_hl(), self.regs.a), // LD (HL), A
            // Destination reg = a
            0x78 => self.regs.a = self.regs.b, // LD A, B
            0x79 => self.regs.a = self.regs.c, // LD A, C
            0x7A => self.regs.a = self.regs.d, // LD A, D
            0x7B => self.regs.a = self.regs.e, // LD A, E
            0x7C => self.regs.a = self.regs.h, // LD A, H
            0x7D => self.regs.a = self.regs.l, // LD A, L
            0x7E => self.regs.a = self.bus.read(self.regs.get_hl()), // LD A, (HL)
            0x7F => {}                         // LD A, A
            // LD r, n
            0x06 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.b = n;
            }
            0x16 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.d = n;
            }
            0x26 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.h = n;
            }
            0x36 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.bus.write(self.regs.get_hl(), n);
            }
            0x0E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.c = n;
            }
            0x1E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.e = n;
            }
            0x2E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.l = n;
            }
            0x3E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.a = n;
            }
            // LD (BC), A
            0x02 => self.bus.write(self.regs.get_bc(), self.regs.a),
            // LD (DE), A
            0x12 => self.bus.write(self.regs.get_de(), self.regs.a),
            // LD (nn), A
            0x32 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.regs.a);
            }
            // LD A, (BC)
            0x0A => self.regs.a = self.bus.read(self.regs.get_bc()),
            // LD A, (DE)
            0x1A => self.regs.a = self.bus.read(self.regs.get_de()),
            // LD A, (nn)
            0x3A => {
                let nn = self.get_nn();
                self.regs.a = self.bus.read(nn);
            }

            // 16-bit Load Group
            // LD BC, nn
            0x01 => {
                let nn = self.get_nn();
                self.regs.set_bc(nn);
            }
            // LD DE, nn
            0x11 => {
                let nn = self.get_nn();
                self.regs.set_de(nn);
            }
            // LD HL, nn
            0x21 => {
                let nn = self.get_nn();
                self.regs.set_hl(nn);
            }
            // LD SP, nn
            0x31 => {
                let nn = self.get_nn();
                self.regs.sp = nn;
            }
            // LD HL, (nn)
            0x2A => {
                let nn = self.get_nn();
                self.regs.l = self.bus.read(nn);
                self.regs.h = self.bus.read(nn.wrapping_add(1));
            }
            // LD (nn), HL
            0x22 => {
                let nn = self.get_nn();
                self.bus.write(nn, self.regs.l);
                self.bus.write(nn.wrapping_add(1), self.regs.h);
            }
            // LD SP, HL
            0xF9 => self.regs.sp = self.regs.get_hl(),
            // PUSH BC
            0xC5 => {
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.b);
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.c);
            }
            // PUSH DE
            0xD5 => {
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.d);
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.e);
            }
            // PUSH HL
            0xE5 => {
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.h);
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.l);
            }
            // PUSH AF
            0xF5 => {
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.a);
                self.regs.dec_sp();
                self.bus.write(self.regs.sp, self.regs.flags.to_byte());
            }
            // POP BC
            0xC1 => {
                self.regs.c = self.bus.read(self.regs.sp);
                self.regs.inc_sp();
                self.regs.b = self.bus.read(self.regs.sp);
            }
            // POP DE
            0xD1 => {
                self.regs.e = self.bus.read(self.regs.sp);
                self.regs.inc_sp();
                self.regs.d = self.bus.read(self.regs.sp);
            }
            // POP HL
            0xE1 => {
                self.regs.l = self.bus.read(self.regs.sp);
                self.regs.inc_sp();
                self.regs.h = self.bus.read(self.regs.sp);
            }
            // POP AF
            0xF1 => {
                let f = self.bus.read(self.regs.sp);
                self.regs.flags.from_byte(f);
                self.regs.inc_sp();
                self.regs.a = self.bus.read(self.regs.sp);
            }
            // Exchange
            // EX DE, HL
            0xEB => {
                let de = self.regs.get_de();
                let hl = self.regs.get_hl();
                self.regs.set_de(hl);
                self.regs.set_hl(de);
            }
            // EX AF,AF'
            0x08 => {
                let af = self.regs.get_af();
                let eaf = self.regs.eaf;
                self.regs.set_af(eaf);
                self.regs.eaf = af;
            }
            // EXX
            0xD9 => {
                let tmp = self.regs.get_bc();
                self.regs.set_bc(self.regs.ebc);
                self.regs.ebc = tmp;
                let tmp = self.regs.get_de();
                self.regs.set_de(self.regs.ede);
                self.regs.ede = tmp;
                let tmp = self.regs.get_hl();
                self.regs.set_hl(self.regs.ehl);
                self.regs.ehl = tmp;
            }
            // EX (SP), HL
            0xE3 => {
                let n = self.bus.read(self.regs.sp);
                self.bus.write(self.regs.sp, self.regs.l);
                self.regs.l = n;
                self.regs.inc_sp();
                let n = self.bus.read(self.regs.sp);
                self.bus.write(self.regs.sp, self.regs.h);
                self.regs.h = n;
            }

            // Jump group
            // JP nn
            0xC3 => self.jp_nn(),
            // JP nz, nn
            0xC2 => {
                if !self.regs.flags.z {
                    self.jp_nn();
                }
            }
            // JP z
            0xCA => {
                if self.regs.flags.z {
                    self.jp_nn();
                }
            }
            // JP nc, nn
            0xD2 => {
                if !self.regs.flags.c {
                    self.jp_nn();
                }
            }
            // JP c, nn
            0xDA => {
                if self.regs.flags.c {
                    self.jp_nn();
                }
            }
            // JP po, nn
            0xE2 => {
                if !self.regs.flags.p {
                    self.jp_nn();
                }
            }
            // JP pe, nn
            0xEA => {
                if self.regs.flags.p {
                    self.jp_nn();
                }
            }
            // JP p, nn
            0xF2 => {
                if !self.regs.flags.s {
                    self.jp_nn();
                }
            }
            // JP m, nn
            0xFA => {
                if self.regs.flags.s {
                    self.jp_nn();
                }
            }
            // JR e
            0x18 => self.jr_e(),
            // JR z, e
            0x28 => {
                if self.regs.flags.z {
                    self.jr_e();
                }
            }
            // JR c, e
            0x38 => {
                if self.regs.flags.c {
                    self.jr_e();
                }
            }
            // DJNZ e
            0x10 => {
                self.regs.b = self.regs.b.wrapping_sub(1);
                if self.regs.b != 0 {
                    self.jr_e();
                    cycles += 5;
                }
            }
            // JR nz, e
            0x20 => {
                if !self.regs.flags.z {
                    self.jr_e();
                }
            }
            // JR nc, nn
            0x30 => {
                if !self.regs.flags.c {
                    self.jr_e();
                }
            }
            // JP (HL)
            0xE9 => self.regs.pc = self.regs.get_hl().wrapping_sub(1),

            // Call & Return Group
            // CALL nn
            0xCD => self.call_nn(),
            // CALL nz, nn
            0xC4 => {
                if !self.regs.flags.z {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL nc, nn
            0xD4 => {
                if !self.regs.flags.c {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL po, nn
            0xE4 => {
                if !self.regs.flags.p {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL p, nn
            0xF4 => {
                if !self.regs.flags.n {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL z, nn
            0xCC => {
                if self.regs.flags.z {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL c, nn
            0xDC => {
                if self.regs.flags.c {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL pe, nn
            0xEC => {
                if self.regs.flags.p {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // CALL n, nn
            0xFC => {
                if self.regs.flags.n {
                    self.call_nn();
                    cycles += 7;
                } else {
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                }
            }
            // RET
            0xC9 => self.ret(),
            // RET nz
            0xC0 => {
                if !self.regs.flags.z {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET nc
            0xD0 => {
                if !self.regs.flags.c {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET po
            0xE0 => {
                if !self.regs.flags.p {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET p
            0xF0 => {
                if !self.regs.flags.n {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET z
            0xC8 => {
                if self.regs.flags.z {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET c
            0xD8 => {
                if self.regs.flags.c {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET pe
            0xE8 => {
                if self.regs.flags.p {
                    self.ret();
                    cycles = cycles.wrapping_add(6);
                }
            }
            // RET n
            0xF8 => {
                if self.regs.flags.n {
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
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                let addr = u16::from_le_bytes([n, self.regs.a]);
                self.regs.a = read_io(addr);
            }
            // OUT (n), A
            0xD3 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                let addr = u16::from_le_bytes([n, self.regs.a]);
                write_io(addr, self.regs.a);
            }

            // 8-bit arithmetic group
            // LD A, r
            0x80 => self.add_a_r(self.regs.b),
            0x81 => self.add_a_r(self.regs.c),
            0x82 => self.add_a_r(self.regs.d),
            0x83 => self.add_a_r(self.regs.e),
            0x84 => self.add_a_r(self.regs.h),
            0x85 => self.add_a_r(self.regs.l),
            0x86 => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.add_a_r(data);
            }
            0x87 => self.add_a_r(self.regs.a),
            // ADC A, r
            0x88 => self.adc_a_r(self.regs.b),
            0x89 => self.adc_a_r(self.regs.c),
            0x8A => self.adc_a_r(self.regs.d),
            0x8B => self.adc_a_r(self.regs.e),
            0x8C => self.adc_a_r(self.regs.h),
            0x8D => self.adc_a_r(self.regs.l),
            0x8E => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.adc_a_r(data);
            }
            0x8F => self.adc_a_r(self.regs.a),
            // SUB A, r
            0x90 => self.sub_a_r(self.regs.b),
            0x91 => self.sub_a_r(self.regs.c),
            0x92 => self.sub_a_r(self.regs.d),
            0x93 => self.sub_a_r(self.regs.e),
            0x94 => self.sub_a_r(self.regs.h),
            0x95 => self.sub_a_r(self.regs.l),
            0x96 => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.sub_a_r(data);
            }
            0x97 => self.sub_a_r(self.regs.a),
            // SBC A, r
            0x98 => self.sbc_a_r(self.regs.b),
            0x99 => self.sbc_a_r(self.regs.c),
            0x9A => self.sbc_a_r(self.regs.d),
            0x9B => self.sbc_a_r(self.regs.e),
            0x9C => self.sbc_a_r(self.regs.h),
            0x9D => self.sbc_a_r(self.regs.l),
            0x9E => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.sbc_a_r(data);
            }
            0x9F => self.sbc_a_r(self.regs.a),
            // AND A, r
            0xA0 => self.bit_op_a_r(BitOp::AND, self.regs.b),
            0xA1 => self.bit_op_a_r(BitOp::AND, self.regs.c),
            0xA2 => self.bit_op_a_r(BitOp::AND, self.regs.d),
            0xA3 => self.bit_op_a_r(BitOp::AND, self.regs.e),
            0xA4 => self.bit_op_a_r(BitOp::AND, self.regs.h),
            0xA5 => self.bit_op_a_r(BitOp::AND, self.regs.l),
            0xA6 => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.bit_op_a_r(BitOp::AND, data);
            }
            0xA7 => self.bit_op_a_r(BitOp::AND, self.regs.a),
            // XOR A, r
            0xA8 => self.bit_op_a_r(BitOp::XOR, self.regs.b),
            0xA9 => self.bit_op_a_r(BitOp::XOR, self.regs.c),
            0xAA => self.bit_op_a_r(BitOp::XOR, self.regs.d),
            0xAB => self.bit_op_a_r(BitOp::XOR, self.regs.e),
            0xAC => self.bit_op_a_r(BitOp::XOR, self.regs.h),
            0xAD => self.bit_op_a_r(BitOp::XOR, self.regs.l),
            0xAE => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.bit_op_a_r(BitOp::XOR, data);
            }
            0xAF => self.bit_op_a_r(BitOp::XOR, self.regs.a),
            // OR A, r
            0xB0 => self.bit_op_a_r(BitOp::OR, self.regs.b),
            0xB1 => self.bit_op_a_r(BitOp::OR, self.regs.c),
            0xB2 => self.bit_op_a_r(BitOp::OR, self.regs.d),
            0xB3 => self.bit_op_a_r(BitOp::OR, self.regs.e),
            0xB4 => self.bit_op_a_r(BitOp::OR, self.regs.h),
            0xB5 => self.bit_op_a_r(BitOp::OR, self.regs.l),
            0xB6 => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.bit_op_a_r(BitOp::OR, data);
            }
            0xB7 => self.cp_r(self.regs.a),
            // CP A, r
            0xB8 => self.cp_r(self.regs.b),
            0xB9 => self.cp_r(self.regs.c),
            0xBA => self.cp_r(self.regs.d),
            0xBB => self.cp_r(self.regs.e),
            0xBC => self.cp_r(self.regs.h),
            0xBD => self.cp_r(self.regs.l),
            0xBE => {
                let addr = self.regs.get_hl();
                let data = self.bus.read(addr);
                self.cp_r(data);
            }
            0xBF => self.cp_r(self.regs.a),
            // ADD a, n
            0xC6 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.add_a_r(n);
            }
            // SUB A, n
            0xD6 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.sub_a_r(n);
            }
            // AND A, n
            0xE6 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.bit_op_a_r(BitOp::AND, n);
            }
            // OR A, n
            0xF6 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.bit_op_a_r(BitOp::OR, n);
            }
            // ADC A, n
            0xCE => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.adc_a_r(n);
            }
            // SBC A, n
            0xDE => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.sbc_a_r(n);
            }
            // XOR a, n
            0xEE => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.bit_op_a_r(BitOp::XOR, n);
            }
            // CP A, n
            0xFE => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.cp_r(n);
            }
            // INC r
            0x04 => self.regs.b = self.inc_r(self.regs.b),
            0x14 => self.regs.d = self.inc_r(self.regs.d),
            0x24 => self.regs.h = self.inc_r(self.regs.h),
            0x34 => {
                let mut n = self.bus.read(self.regs.get_hl());
                n = self.inc_r(n);
                self.bus.write(self.regs.get_hl(), n);
            }
            0x0C => self.regs.c = self.inc_r(self.regs.c),
            0x1C => self.regs.e = self.inc_r(self.regs.e),
            0x2C => self.regs.l = self.inc_r(self.regs.l),
            0x3C => self.regs.a = self.inc_r(self.regs.a),
            // DEC r
            0x05 => self.regs.b = self.dec_r(self.regs.b),
            0x15 => self.regs.d = self.dec_r(self.regs.d),
            0x25 => self.regs.h = self.dec_r(self.regs.h),
            0x35 => {
                let mut n = self.bus.read(self.regs.get_hl());
                n = self.dec_r(n);
                self.bus.write(self.regs.get_hl(), n);
            }
            0x0D => self.regs.c = self.dec_r(self.regs.c),
            0x1D => self.regs.e = self.dec_r(self.regs.e),
            0x2D => self.regs.l = self.dec_r(self.regs.l),
            0x3D => self.regs.a = self.dec_r(self.regs.a),

            // 16-bit arithmetic group
            // ADD HL, rr
            0x09 => self.add_hl_rr(self.regs.get_bc()),
            0x19 => self.add_hl_rr(self.regs.get_de()),
            0x29 => self.add_hl_rr(self.regs.get_hl()),
            0x39 => self.add_hl_rr(self.regs.sp),
            // INC rr
            0x03 => self.regs.set_bc(self.regs.get_bc().wrapping_add(1)),
            0x13 => self.regs.set_de(self.regs.get_de().wrapping_add(1)),
            0x23 => self.regs.set_hl(self.regs.get_hl().wrapping_add(1)),
            0x33 => self.regs.sp = self.regs.sp.wrapping_add(1),
            // DEC rr
            0x0B => self.regs.set_bc(self.regs.get_bc().wrapping_sub(1)),
            0x1B => self.regs.set_de(self.regs.get_de().wrapping_sub(1)),
            0x2B => self.regs.set_hl(self.regs.get_hl().wrapping_sub(1)),
            0x3B => self.regs.sp = self.regs.sp.wrapping_sub(1),
            _ => {
                println!("Unknown instruction.");
            }
        }
        self.regs.inc_pc();
        cycles
    }
}
