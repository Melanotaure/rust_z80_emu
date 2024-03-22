use rust_z80_emu::z80::*;

fn main() {
    let mut z80 = Z80::new();

    let code = std::fs::read("resources/multiply_u16.bin").unwrap();
    for (addr, opcode) in code.iter().enumerate() {
        z80.bus.write(addr as u16, *opcode);
    }

    let mut cycles: usize = 0;
    z80.display_regs();
    println!("");
    loop {
        cycles += z80.execute() as usize;
        z80.display_regs();
        println!("");
        if z80.reg.pc == 0x0019 {
            break;
        }
    }
    println!("cycles: {}", cycles);
}
