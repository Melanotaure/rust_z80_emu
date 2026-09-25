use rust_z80_emu::z80::{InterruptMode, Z80};
use serde::Deserialize;
use std::fs;
use std::path::Path;

// JSon structure
#[derive(Deserialize, Debug)]
struct Z80State {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    // Extra registers
    af_: u16,
    bc_: u16,
    de_: u16,
    hl_: u16,
    ix: u16,
    iy: u16,
    i: u8,
    r: u8,
    iff1: u8,
    iff2: u8,
    im: u8,
    // wz hidden internal register "MEMPTR"
    // #[serde(default)] if the register is not in the JSON
    #[serde(default)]
    wz: u16,
    ram: Vec<(u16, u8)>,
}

#[derive(Deserialize, Debug)]
struct Z80Test {
    name: String,
    initial: Z80State,
    #[serde(rename = "final")]
    final_state: Z80State,
}

/// This function takes one JSON test and executes it through the Z80 emulator
fn run_single_test(test: &Z80Test) {
    let mut cpu = Z80::new();

    let init = &test.initial;
    cpu.reg.pc = init.pc;
    cpu.reg.sp = init.sp;
    cpu.reg.a = init.a;
    cpu.reg.b = init.b;
    cpu.reg.c = init.c;
    cpu.reg.d = init.d;
    cpu.reg.e = init.e;
    cpu.reg.flags.from_byte(init.f);
    cpu.reg.h = init.h;
    cpu.reg.l = init.l;

    cpu.reg.eaf = init.af_;
    cpu.reg.ebc = init.bc_;
    cpu.reg.ede = init.de_;
    cpu.reg.ehl = init.hl_;

    cpu.reg.set_ix(init.ix);
    cpu.reg.set_iy(init.iy);
    cpu.reg.set_ir(u16::from_le_bytes([init.r, init.i]));

    cpu.iff1 = init.iff1 != 0;
    cpu.iff2 = init.iff2 != 0;
    cpu.im = match init.im {
        0 => InterruptMode::IM_0,
        1 => InterruptMode::IM_1,
        2 => InterruptMode::IM_2,
        _ => InterruptMode::IM_0,
    };

    for &(addr, val) in &init.ram {
        cpu.bus.write(addr, val);
    }

    let _cycles = cpu.execute();

    let fin = &test.final_state;

    assert_eq!(cpu.reg.pc, fin.pc, "[{}] PC error", test.name);
    assert_eq!(cpu.reg.sp, fin.sp, "[{}] SP error", test.name);
    assert_eq!(cpu.reg.a, fin.a, "[{}] Reg A error", test.name);
    assert_eq!(cpu.reg.b, fin.b, "[{}] Reg B error", test.name);
    assert_eq!(cpu.reg.c, fin.c, "[{}] Reg C error", test.name);
    assert_eq!(cpu.reg.d, fin.d, "[{}] Reg D error", test.name);
    assert_eq!(cpu.reg.e, fin.e, "[{}] Reg E error", test.name);
    assert_eq!(
        (cpu.reg.get_af() & 0x0F) as u8,
        fin.f,
        "[{}] Reg F error",
        test.name
    );
    assert_eq!(cpu.reg.h, fin.h, "[{}] Reg H error", test.name);
    assert_eq!(cpu.reg.l, fin.l, "[{}] Reg L error", test.name);

    for &(addr, val) in &fin.ram {
        let actual_val = cpu.bus.read(addr);
        assert_eq!(actual_val, val, "[{}] RAM error @{:04X}", test.name, addr);
    }
}

#[test]
fn test_tom_harte_z80() {
    let folder_path = Path::new("./tests/testdata/z80/v1");

    for entry in fs::read_dir(folder_path).expect("Can't find Tests folder") {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            println!(
                "Execution of the test suite: {:?}",
                path.file_name().unwrap()
            );

            let data = fs::read_to_string(&path).expect("Can't read the file.");

            let tests: Vec<Z80Test> = serde_json::from_str(&data)
                .unwrap_or_else(|e| panic!("Parsing error in {:?}: {}", path, e));

            for test in tests {
                run_single_test(&test);
            }
        }
    }
}
