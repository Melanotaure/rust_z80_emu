use crate::Z80Registers as regs;

// Structure of the Z80 processor
pub struct Z80 {
    // Registers
    pub regs: regs,
    // Address bus
    pub abus: u16,
    // Data bus
    pub dbus: u8,
    // System control pins
    pub nM1: bool,
    pub nMREQ: bool,
    pub nIORQ: bool,
    pub nRD: bool,
    pub nWR: bool,
    pub nRFSH: bool,
    // CPU control
    pub nHALT: bool,
    pub nWAIT: bool,
    pub nINT: bool,
    pub nNMI: bool,
    pub nRESET: bool,
    // CPU bus control
    pub nBUSRQ: bool,
    pub nBUSACK: bool,
    // Added the clock pin but might not be used
    pub _clock: bool,
}

impl Z80 {
    pub fn new() -> Self {
        Self {
            regs: regs::new(),
            abus: 0_u16,
            dbus: 0_u8,
            nM1: true,
            nMREQ: true,
            nIORQ: true,
            nRD: true,
            nWR: true,
            nRFSH: true,
            nHALT: true,
            nWAIT: true,
            nINT: true,
            nNMI: true,
            nRESET: true,
            nBUSRQ: true,
            nBUSACK: true,
            _clock: true,
        }
    }

    pub fn reset(&mut self) {
        self.regs.reset();
        self.abus = 0_u16;
        self.dbus = 0_u8;
        self.nBUSACK = true;
        self.nBUSRQ = true;
        self.nHALT = true;
        self.nINT = true;
        self.nIORQ = true;
        self.nM1 = true;
        self.nMREQ = true;
        self.nNMI = true;
        self.nRD = true;
        self.nRESET = true;
        self.nRFSH = true;
        self.nWAIT = true;
        self.nWR = true;
        self._clock = true;
    }
}
