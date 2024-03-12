use crate::bus::Bus;
use crate::registers::Registers;

pub enum InterruptMode {
    IM_0,
    IM_1,
    IM_2,
}

// Structure of the Z80 processor
pub struct Z80 {
    // Registers
    pub reg: Registers,
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
    pub iff1: bool,
    pub iff2: bool,
    pub im: InterruptMode,
    // CPU bus control
    pub n_busrq: bool,
    pub n_busack: bool,
    // Previous instruction
    pub p_inst: u8,
    // Added the clock pin but might not be used
    pub _clock: u64,
}

impl Z80 {
    pub fn new() -> Self {
        Self {
            reg: Registers::new(),
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
            iff1: false,
            iff2: false,
            im: InterruptMode::IM_0,
            n_busrq: true,
            n_busack: true,
            p_inst: 0,
            _clock: 0_u64,
        }
    }

    pub fn reset(&mut self) {
        self.reg.reset();
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
        self.iff1 = false;
        self.iff2 = false;
        self.im = InterruptMode::IM_0;
        self.n_rfsh = true;
        self.n_wait = true;
        self.n_wr = true;
        self.p_inst;
        self._clock = 0;
    }

    pub fn display_regs(&self) {
        println!("PC: {:04X} SP: {:04X}", self.reg.pc, self.reg.sp);
        println!("AF: {:04X} BC: {:04X}", self.reg.get_af(), self.reg.get_bc());
        println!("DE: {:04X} HL: {:04X}", self.reg.get_de(), self.reg.get_hl());
    }
}
