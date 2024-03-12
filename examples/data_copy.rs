use rust_z80_emu::z80::*;

fn main() {
    let mut z80 = Z80::new();

    let code = std::fs::read("resources/data_copy.bin").unwrap();
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
        if z80.reg.pc == 0x000B {
            break;
        }
    }
    println!("cycles: {}", cycles);

    for addr in 11..43 {
        println!("addr: {:04X} data: {:02X}", addr, z80.bus.read(addr));
    }
}
