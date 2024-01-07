use crate::z80_registers;
use z80_registers::Z80Registers;

// Structure of the Z80 processor
pub struct Z80 {
    // Registers
    pub regs: Z80Registers,
    // Address bus
    pub abus: u16,
    // Data bus
    pub dbus: u8,
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
    pub _clock: bool,
}

impl Z80 {
    pub fn new() -> Self {
        Self {
            regs: Z80Registers::new(),
            abus: 0_u16,
            dbus: 0_u8,
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
            _clock: true,
        }
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.abus = 0_u16;
        self.dbus = 0_u8;
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
        self._clock = true;
    }
}
