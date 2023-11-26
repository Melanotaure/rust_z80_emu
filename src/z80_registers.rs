use crate::register::*;

pub struct Z80Registers {
    pub af: Register,
    pub bc: Register,
    pub de: Register,
    pub hl: Register,
    pub pc: Register,
    pub sp: Register,
    pub ix: Register,
    pub iy: Register,
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
        }
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
