pub mod register;
pub mod z80;
pub mod z80_registers;

use z80::*;

fn main() {
    let mut z80 = Z80::new();

    // Print the default new registers' values
    println!("Default values of new registers.");
    z80.regs.print();

    // Set some registers
    z80.regs.hl.set_reg16(0xfeed);
    z80.regs.bc.set_reg8_h(0xbe);
    z80.regs.bc.set_reg8_l(0xef);
    // Print them
    println!("\nRegisters values after setting HL and B and C.");
    z80.regs.print();

    println!("\nHL <- HL + BC");
    z80.regs.hl.add_r16_r16(&z80.regs.bc);
    // Print r16 + r16 addition results
    z80.regs.print();

    println!("\nHL <- HL + B");
    z80.regs.hl.add_r16_i8(z80.regs.bc.get_reg8_h());
    // Print r16+ i8 addition results
    z80.regs.print();

    z80.reset();
    // Print the default new registers' values
    println!("\nReset Z80.\nCurrent values of registers.");
    z80.regs.print();

    z80.instructions[0](&mut z80);
    z80.instructions[1](&mut z80);
}
