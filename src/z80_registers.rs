use crate::register::*;

pub struct Z80Registers {
    // Registers
    pub af: Register,
    pub bc: Register,
    pub de: Register,
    pub hl: Register,
    pub pc: Register,
    pub sp: Register,
    pub ix: Register,
    pub iy: Register,

    // Extra registers
    pub eaf: Register,
    pub ebc: Register,
    pub ede: Register,
    pub ehl: Register,
    pub epc: Register,
    pub esp: Register,
    pub eix: Register,
    pub eiy: Register,
}

impl Z80Registers {
    pub fn new() -> Self {
        Self {
            af: Register::new(),
            bc: Register::new(),
            de: Register::new(),
            hl: Register::new(),
            pc: Register::new(),
            sp: Register::new(),
            ix: Register::new(),
            iy: Register::new(),

            eaf: Register::new(),
            ebc: Register::new(),
            ede: Register::new(),
            ehl: Register::new(),
            epc: Register::new(),
            esp: Register::new(),
            eix: Register::new(),
            eiy: Register::new(),
        }
    }

    pub fn reset(&mut self) {
        self.af.set_reg16(0);
        self.bc.set_reg16(0);
        self.de.set_reg16(0);
        self.hl.set_reg16(0);
        self.pc.set_reg16(0);
        self.sp.set_reg16(0);
        self.ix.set_reg16(0);
        self.iy.set_reg16(0);

        self.eaf.set_reg16(0);
        self.ebc.set_reg16(0);
        self.ede.set_reg16(0);
        self.ehl.set_reg16(0);
        self.epc.set_reg16(0);
        self.esp.set_reg16(0);
        self.eix.set_reg16(0);
        self.eiy.set_reg16(0);
    }

    pub fn print(&self) {
        println!("PC: {:#06x}", self.pc.get_reg16());
        println!("SP: {:#06x}", self.sp.get_reg16());
        println!("IX: {:#06x}", self.ix.get_reg16());
        println!("IY: {:#06x}", self.iy.get_reg16());
        println!(
            "AF: {:#06x} A: {:#04x} F: {:#04x}",
            self.af.get_reg16(),
            self.af.get_reg8_h(),
            self.af.get_reg8_l()
        );
        println!(
            "BC: {:#06x} B: {:#04x} C: {:#04x}",
            self.bc.get_reg16(),
            self.bc.get_reg8_h(),
            self.bc.get_reg8_l()
        );
        println!(
            "DE: {:#06x} D: {:#04x} E: {:#04x}",
            self.de.get_reg16(),
            self.de.get_reg8_h(),
            self.de.get_reg8_l()
        );
        println!(
            "HL: {:#06x} H: {:#04x} L: {:#04x}",
            self.hl.get_reg16(),
            self.hl.get_reg8_h(),
            self.hl.get_reg8_l()
        );
    }
}
