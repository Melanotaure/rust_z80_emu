use crate::cycles::CYCLES;
use crate::registers::Registers;
use crate::bus::Bus;

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

    // Main function to run the CPU's instructions
    pub fn execute(&mut self) -> u8 {
        let instr = self.bus.read(self.regs.pc);
        let mut cycles = CYCLES[instr as usize];

        match instr {
            // NOP
            0x00 => {},

            // 8-bit load group
            // Destination reg = b
            0x40 => {},                         // LD B, B
            0x41 => self.regs.b = self.regs.c,  // LD B, C
            0x42 => self.regs.b = self.regs.d,  // LD B, D
            0x43 => self.regs.b = self.regs.e,  // LD B, E
            0x44 => self.regs.b = self.regs.h,  // LD B, H
            0x45 => self.regs.b = self.regs.l,  // LD B, L
            0x46 => self.regs.b = self.bus.read(self.regs.get_hl()), // LD B, (HL)
            0x47 => self.regs.b = self.regs.a,  // LD B, A
            // Destination reg = c
            0x48 => self.regs.c = self.regs.b,  // LD C, B
            0x49 => {},                         // LD C, C
            0x4A => self.regs.c = self.regs.d,  // LD C, D
            0x4B => self.regs.c = self.regs.e,  // LD C, E
            0x4C => self.regs.c = self.regs.h,  // LD C, H
            0x4D => self.regs.c = self.regs.l,  // LD C, L
            0x4E => self.regs.c = self.bus.read(self.regs.get_hl()), // LD C, (HL)
            0x4F => self.regs.c = self.regs.a,  // LD C, A
            // Destination reg = d
            0x50 => self.regs.d = self.regs.b,  // LD D, B
            0x51 => self.regs.d = self.regs.c,  // LD D, C
            0x52 => {},                         // LD D, D
            0x53 => self.regs.d = self.regs.e,  // LD D, E
            0x54 => self.regs.d = self.regs.h,  // LD D, H
            0x55 => self.regs.d = self.regs.l,  // LD D, L
            0x56 => self.regs.d = self.bus.read(self.regs.get_hl()), // LD D, (HL)
            0x57 => self.regs.d = self.regs.a,  // LD D, A
            // Destination reg = e
            0x58 => self.regs.e = self.regs.b,  // LD E, B
            0x59 => self.regs.e = self.regs.c,  // LD E, C
            0x5A => self.regs.e = self.regs.d,  // LD E, D
            0x5B => {},                         // LD E, E
            0x5C => self.regs.e = self.regs.h,  // LD E, H
            0x5D => self.regs.e = self.regs.l,  // LD E, L
            0x5E => self.regs.e = self.bus.read(self.regs.get_hl()), // LD E, (HL)
            0x5F => self.regs.e = self.regs.a,  // LD E, A
            // Destination reg = h
            0x60 => self.regs.h = self.regs.b,  // LD H, B
            0x61 => self.regs.h = self.regs.c,  // LD H, C
            0x62 => self.regs.h = self.regs.d,  // LD H, D
            0x63 => self.regs.h = self.regs.e,  // LD H, E
            0x64 => {},                         // LD H, H
            0x65 => self.regs.h = self.regs.l,  // LD H, L
            0x66 => self.regs.h = self.bus.read(self.regs.get_hl()), // LD H, (HL)
            0x67 => self.regs.h = self.regs.a,  // LD H, A
            // Destination reg = l
            0x68 => self.regs.l = self.regs.b,  // LD L, B
            0x69 => self.regs.l = self.regs.c,  // LD L, C
            0x6A => self.regs.l = self.regs.d,  // LD L, D
            0x6B => self.regs.l = self.regs.e,  // LD L, E
            0x6C => self.regs.l = self.regs.h,  // LD L, H
            0x6D => {},                         // LD L, L
            0x6E => self.regs.l = self.bus.read(self.regs.get_hl()), // LD L, (HL)
            0x6F => self.regs.l = self.regs.a,  // LD L, A
            // Destination reg = (hl)
            0x70 => self.bus.write(self.regs.get_hl(), self.regs.b),  // LD (HL), B
            0x71 => self.bus.write(self.regs.get_hl(), self.regs.c),  // LD (HL), C
            0x72 => self.bus.write(self.regs.get_hl(), self.regs.d),  // LD (HL), D
            0x73 => self.bus.write(self.regs.get_hl(), self.regs.e),  // LD (HL), E
            0x74 => self.bus.write(self.regs.get_hl(), self.regs.h),  // LD (HL), H
            0x75 => self.bus.write(self.regs.get_hl(), self.regs.l),  // LD (HL), L
            // 0x76 => HALT treated elsewhere
            0x77 => self.bus.write(self.regs.get_hl(), self.regs.a),  // LD (HL), A
            // Destination reg = a
            0x78 => self.regs.a = self.regs.b,  // LD A, B
            0x79 => self.regs.a = self.regs.c,  // LD A, C
            0x7A => self.regs.a = self.regs.d,  // LD A, D
            0x7B => self.regs.a = self.regs.e,  // LD A, E
            0x7C => self.regs.a = self.regs.h,  // LD A, H
            0x7D => self.regs.a = self.regs.l,  // LD A, L
            0x7E => self.regs.a = self.bus.read(self.regs.get_hl()), // LD A, (HL)
            0x7F => {},                         // LD A, A
            // LD r, n
            0x06 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.b = n;
            },
            0x16 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.d = n;
            },
            0x26 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.h = n;
            },
            0x36 => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.bus.write(self.regs.get_hl(), n);
            },
            0x0E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.c = n;
            },
            0x1E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.e = n;
            },
            0x2E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.l = n;
            },
            0x3E => {
                self.regs.inc_pc();
                let n = self.bus.read(self.regs.pc);
                self.regs.a = n;
            }

            _ => {
                println!("Unknown instruction.");
            }
        }
        self.regs.inc_pc();
        cycles
    }
}

