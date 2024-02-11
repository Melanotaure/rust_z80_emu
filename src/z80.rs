use crate::bus::Bus;
use crate::cycles::CYCLES;
use crate::registers::Registers;

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
        self.bus.write(self.regs.sp, pch);
        self.regs.pc = u16::from_le_bytes([addr, 0x00]);
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

            _ => {
                println!("Unknown instruction.");
            }
        }
        self.regs.inc_pc();
        cycles
    }
}
