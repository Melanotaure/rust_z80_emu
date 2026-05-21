use rust_z80_emu::z80::*;
use std::io;

fn main() {
    let mut z80 = Z80::new();

    // Code
    // LD HL, 0x0300
    z80.bus.write(0x0100, 0x21);
    z80.bus.write(0x0101, 0x00);
    z80.bus.write(0x0102, 0x03);
    // LD SP, HL
    z80.bus.write(0x0103, 0xF9);
    // LD HL, 0x0000
    z80.bus.write(0x0104, 0x21);
    z80.bus.write(0x0105, 0x00);
    z80.bus.write(0x0106, 0x00);
    // PUSH HL
    z80.bus.write(0x0107, 0xE5);
    // POP AF
    z80.bus.write(0x0108, 0xF1);
    // LD BC, 0x1234
    z80.bus.write(0x0109, 0x01);
    z80.bus.write(0x010A, 0x34);
    z80.bus.write(0x010B, 0x12);
    // ADC HL, BC
    z80.bus.write(0x010C, 0xED);
    z80.bus.write(0x010D, 0x4A);
    // JP C, 0x0200 (error)
    z80.bus.write(0x010E, 0xDA);
    z80.bus.write(0x010F, 0x00);
    z80.bus.write(0x0110, 0x02);
    // SBC HL, BC
    z80.bus.write(0x0111, 0xED);
    z80.bus.write(0x0112, 0x42);
    // PUSH AF
    z80.bus.write(0x0113, 0xF5);
    // POP DE
    z80.bus.write(0x0114, 0xD1);
    // LD A, E
    z80.bus.write(0x0115, 0x7B);
    // CP 0x52 (expected flags)
    z80.bus.write(0x0116, 0xFE);
    z80.bus.write(0x0117, 0x52);
    // JP NZ, 0x0200 (error)
    z80.bus.write(0x0118, 0xC2);
    z80.bus.write(0x0119, 0x00);
    z80.bus.write(0x011A, 0x02);
    // INC HL
    z80.bus.write(0x011B, 0x23);
    // PUSH HL
    z80.bus.write(0x011C, 0xE5);
    // POP AF
    z80.bus.write(0x011D, 0xF1);
    // LD DE, 0xFFFF
    z80.bus.write(0x011E, 0x11);
    z80.bus.write(0x011F, 0xFE);
    z80.bus.write(0x0120, 0xFF);
    // ADC HL, DE
    z80.bus.write(0x0121, 0xED);
    z80.bus.write(0x0122, 0x5A);
    // PUSH AF
    z80.bus.write(0x0123, 0xF5);
    // POP BC
    z80.bus.write(0x0124, 0xC1);
    // LD A, C
    z80.bus.write(0x0125, 0x79);
    // CP 0x51
    z80.bus.write(0x0126, 0xFE);
    z80.bus.write(0x0127, 0x51);
    // JP NZ, 0x0200 (erreur)
    z80.bus.write(0x0128, 0xC2);
    z80.bus.write(0x0129, 0x00);
    z80.bus.write(0x012A, 0x02);
    // SBC HL, DE
    z80.bus.write(0x012B, 0xED);
    z80.bus.write(0x012C, 0x52);
    // PUSH AF
    z80.bus.write(0x012D, 0xF5);
    // POP BC
    z80.bus.write(0x012E, 0xC1);
    // LD A, C
    z80.bus.write(0x012F, 0x79);
    // CP 0x55
    z80.bus.write(0x0130, 0xFE);
    z80.bus.write(0x0131, 0x55);
    // JP NZ, 0x0200 (erreur)
    z80.bus.write(0x0132, 0xC2);
    z80.bus.write(0x0133, 0x00);
    z80.bus.write(0x0134, 0x02);
    // CALL 0x0000
    z80.bus.write(0x0135, 0xCD);
    z80.bus.write(0x0136, 0x00);
    z80.bus.write(0x0137, 0x00);
    // error: HALT
    z80.bus.write(0x0200, 0x76);

    let mut cycles: usize = 0;
    z80.reg.pc = 0x0100_u16;

    loop {
        cycles += z80.execute() as usize;
        if z80.reg.pc < 0x0005_u16 {
            println!("\nCPU restarted!");
            break;
        }
        if z80.n_halt == false {
            println!("\nCPU halted!");
            break;
        }

        z80.memory_dump(0x0000, 0x0310);
        println!("");
        z80.display_regs();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }
    println!("cycles: {}", cycles);
}
