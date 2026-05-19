use rust_z80_emu::z80::*;
// use std::io;

fn main() {
    let mut z80 = Z80::new();

    let code = std::fs::read("resources/zexdoc.com").unwrap();
    for (addr, opcode) in code.iter().enumerate() {
        z80.bus.write((addr + 0x100) as u16, *opcode);
    }

    let mut cycles: usize = 0;
    z80.reg.pc = 0x0100_u16;

    loop {
        cycles += z80.execute() as usize;
        if z80.reg.pc < 0x0005_u16 {
            println!("CPU restarted!");
            break;
        }
        if z80.n_halt == false {
            println!("CPU halted!");
            break;
        }

        // z80.memory_dump(0x0000, 0x22ff);
        // println!("");
        // z80.display_regs();

        // let mut input = String::new();
        // io::stdin().read_line(&mut input).unwrap();
    }
    println!("cycles: {}", cycles);
}
