mod register;

use register::*;

struct AmsRegs {
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,
    pc: Register,
    sp: Register,
    ix: Register,
    iy: Register,
}

impl AmsRegs {
    fn new() -> Self {
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

    fn print(&self) {
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

fn main() {
    let mut ams_regs = AmsRegs::new();

    // Print the default new registers' values
    println!("Default values of new registers.");
    ams_regs.print();

    // Set some registers
    ams_regs.hl.set_reg16(0xfeed);
    ams_regs.bc.set_reg8_h(0xbe);
    ams_regs.bc.set_reg8_l(0xef);
    // Print them
    println!("\nRegisters values after setting HL and B and C.");
    ams_regs.print();

    println!("\nHL <- HL + BC");
    ams_regs.hl.add_r16_r16(&ams_regs.bc);
    // Print r16 + r16 addition results
    ams_regs.print();

    println!("\nHL <- HL + B");
    ams_regs.hl.add_r16_i8(ams_regs.bc.get_reg8_h());
    // Print r16+ i8 addition results
    ams_regs.print();
}
