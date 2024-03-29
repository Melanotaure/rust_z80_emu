pub const CYCLES: [u8; 256] = [
     4, 10,  7,  6,  4,  4,  7,  4,  4, 11,  7,  6,  4,  4,  7,  4,
     8, 10,  7,  6,  4,  4,  7,  4, 12, 11,  7,  6,  4,  4,  7,  4,
     7, 10, 16,  6,  4,  4,  7,  4,  7, 11, 16,  6,  4,  4,  7,  4,
     7, 10, 13,  6, 11, 11, 10,  4,  7, 11, 13,  6,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     7,  7,  7,  7,  7,  7,  4,  7,  4,  4,  4,  4,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     4,  4,  4,  4,  4,  4,  7,  4,  4,  4,  4,  4,  4,  4,  7,  4,
     5, 10, 10, 10, 10, 11,  7, 11,  5, 10, 10,  0, 10, 17,  7, 11,
     5, 10, 10, 11, 10, 11,  7, 11,  5,  4, 10, 11, 10,  4,  7, 11,
     5, 10, 10, 19, 10, 11,  7, 11,  5,  4, 10,  4, 10,  4,  7, 11,
     5, 10, 10,  4, 10, 11,  7, 11,  5,  6, 10,  4, 10,  4,  7, 11,
];

pub const CYCLES_CB: [u8; 256] = [
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
    8, 8, 8, 8, 8, 8, 15, 8, 8, 8, 8, 8, 8, 8, 15, 8,
];

pub const CYCLES_ED: [u8; 256] = [
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0, 11, 16, 4, 10,  4,  5,  8,  8, 11, 16,  0, 10,  0,  5,
     0,  0, 11, 16, 0,  0,  4,  5,  8,  8, 11, 16,  0,  0,  8,  5,
     0,  0, 11, 16, 0,  0,  0, 14,  8,  8, 11, 16,  0,  0,  0, 14,
     8,  8, 11, 16, 0,  0,  0,  0,  8,  8, 11, 16,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    12, 12, 12, 12, 0,  0,  0,  0, 12, 12, 12, 12,  0,  0,  0,  0,
    12, 12, 12, 12, 0,  0,  0,  0, 12, 12, 12, 12,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
];

pub const CYCLES_DD_FD: [u8; 256] = [
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
     4,  4,  4,  4, 12, 12,  9,  4,  4,  4,  4,  4,  4,  4,  4,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
    12, 12, 12, 12, 12, 12,  4, 12,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4, 12,  4,  4,  4,  4,  4,  4,  4, 12,  4,
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  0,  4,  4,  4,  4,
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
     4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
];

pub const CYCLES_DD_FD_CB: [u8; 256] = [
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    12, 12, 12, 12, 12, 12,  8, 12, 12, 12, 12, 12, 12, 12,  8, 12,
    12, 12, 12, 12, 12, 12,  8, 12, 12, 12, 12, 12, 12, 12,  8, 12,
    12, 12, 12, 12, 12, 12,  8, 12, 12, 12, 12, 12, 12, 12,  8, 12,
    12, 12, 12, 12, 12, 12,  8, 12, 12, 12, 12, 12, 12, 12,  8, 12,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
    15, 15, 15, 15, 15, 15,  8, 15, 15, 15, 15, 15, 15, 15,  8, 15,
];