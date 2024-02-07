use crate::flags::Flags;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub ixh: u8,
    pub ixl: u8,
    pub iyh: u8,
    pub iyl: u8,
    pub i: u8,
    pub r: u8,
    pub sp: u16,
    pub pc: u16,
    pub flags: Flags,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            ixh: 0,
            ixl: 0,
            iyh: 0,
            iyl: 0,
            i: 0,
            r: 0,
            sp: 0,
            pc: 0,
            flags: Flags::new(),
        }
    }

    pub fn get_af(&self) -> u16 {
        u16::from_le_bytes([self.flags.to_byte(), self.a])
    }

    pub fn set_af(&mut self, val: u16) {
        let f: u8;
        [f, self.a] = val.to_le_bytes();
        self.flags.from_byte(f);
    }

    pub fn get_bc(&self) -> u16 {
        u16::from_le_bytes([self.c, self.b])
    }

    pub fn set_bc(&mut self, val: u16) {
        [self.c, self.b] = val.to_le_bytes();
    }

    pub fn get_de(&self) -> u16 {
        u16::from_le_bytes([self.e, self.d])
    }

    pub fn set_de(&mut self, val: u16) {
        [self.e, self.d] = val.to_le_bytes();
    }

    pub fn get_hl(&self) -> u16 {
        u16::from_le_bytes([self.l, self.h])
    }

    pub fn set_hl(&mut self, val: u16) {
        [self.l, self.h] = val.to_le_bytes();
    }

    pub fn get_ix(&self) -> u16 {
        u16::from_le_bytes([self.ixl, self.ixh])
    }

    pub fn set_ix(&mut self, val: u16) {
        [self.ixl, self.ixh] = val.to_le_bytes();
    }

    pub fn get_iy(&self) -> u16 {
        u16::from_le_bytes([self.iyl, self.iyh])
    }

    pub fn set_iy(&mut self, val: u16) {
        [self.iyl, self.iyh] = val.to_le_bytes();
    }

    pub fn get_ir(&self) -> u16 {
        u16::from_le_bytes([self.r, self.i])
    }

    pub fn set_ir(&mut self, val: u16) {
        [self.r, self.i] = val.to_le_bytes();
    }

    pub fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub fn dec_pc(&mut self) {
        self.pc = self.pc.wrapping_sub(1);
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.d = 0;
        self.e = 0;
        self.h = 0;
        self.l = 0;
        self.ixh = 0;
        self.ixl = 0;
        self.iyh = 0;
        self.iyl = 0;
        self.i = 0;
        self.r = 0;
        self.sp = 0;
        self.pc = 0;
        self.flags.reset();
    }
}
