mod register;
mod z80_registers;

use z80_registers::*;

fn main() {
    let mut z80_regs = Z80Registers::new();

    // Print the default new registers' values
    println!("Default values of new registers.");
    z80_regs.print();

    // Set some registers
    z80_regs.hl.set_reg16(0xfeed);
    z80_regs.bc.set_reg8_h(0xbe);
    z80_regs.bc.set_reg8_l(0xef);
    // Print them
    println!("\nRegisters values after setting HL and B and C.");
    z80_regs.print();

    println!("\nHL <- HL + BC");
    z80_regs.hl.add_r16_r16(&z80_regs.bc);
    // Print r16 + r16 addition results
    z80_regs.print();

    println!("\nHL <- HL + B");
    z80_regs.hl.add_r16_i8(z80_regs.bc.get_reg8_h());
    // Print r16+ i8 addition results
    z80_regs.print();
}
