pub mod registers;
pub mod flags;
pub mod bus;
pub mod cycles;
pub mod z80;

use z80::*;

fn main() {
    let mut z80 = Z80::new();

}
