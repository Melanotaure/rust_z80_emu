# rust_z80_emu

My own implementation of the Zilog Z80 processor in RUST. The goal is to develop an Amstrad 6128 emulator afterwards.

## How to run it

For now, this is an early, experimental, implementation that will be formatted as a RUST crate later.

I'll try to always keep it executable then just run it with cargo as followed:

    cargo run --release

You can also run some examples:
1. Data Copy

```
    LD      HL, @DATA       ; START ADDRESS OF DATA STRING
    LD      DE, @BUFFER     ; START ADDRESS OF TARGET BUFFER
    LD      BC, 0x0010      ; LENGTH OF DATA STRING
    LDIR                    ; MOVE STRING–TRANSFER MEMORY POINTED
                            ; TO BY HL INTO MEMORY LOCATION POINTED
                            ; TO BY DE INCREMENT HL AND DE,
                            ; DECREMENT BC PROCESS UNTIL BC = 0
@DATA:
.byte 0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF
@BUFFER:                    ; BUFFER STARTS JUST AFTER THE DATA
```

```
    cargo run --release --example data_copy
```

2. Data Copy 2

```
    LD      HL, @DATA       ; STARTING ADDRESS OF DATA STRING
    LD      DE, @BUFFER     ; STARTING ADDRESS OF TARGET BUFFER
    LD      BC, 132         ; MAXIMUM STRING LENGTH
    LD       A, '$'         ; STRING DELIMITER CODE
@LOOP:
    CP      (HL)            ; COMPARE MEMORY CONTENTS WITH
                            ; DELIMITER
    JR      Z, @END         ; GO TO END IF CHARACTERS EQUAL
    LDI                     ; MOVE CHARACTER (HL) to (DE)
                            ; INCREMENT HL AND DE, DECREMENT BC
    JP      PE, @LOOP       ; GO TO LOOP IF MORE CHARACTERS
@END:                       ; OTHERWISE, FALL THROUGH
                            ; NOTE: P/V FLAG IS USED
                            ; TO INDICATE THAT REGISTER BC WAS
                            ; DECREMENTED TO ZERO
@DATA:
.byte 0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, '$', 0xCA, 0xFE, 0xCA, 0xFE
@BUFFER:                    ; BUFFER STARTS JUST AFTER THE DATA
```

```
    cargo run --release --example data_copy_2
```

3. U16 Multiplication

```
    LD      HL, 127         ; 16-bit multiplication of 127
    LD      DE, 11          ; by 11
    LD       B, 16          ; Init the number of bits
    LD       C, D           ; move multiplier
    LD       A, E           ;
    EX      DE, HL          ; move multiplicand
    LD      HL, 0           ; clear partial result
@MLOOP:
    SRL     C               ; shift multiplier to the right
    RRA                     ; least significant bit is in carry
    JR      NC, @NOADD      ; if no carry skip the ADD
    ADD     HL, DE          ; else add mutliplicand to partial result
@NOADD:
    EX      DE, HL          ; shift multiplicand to the left
    ADD     HL, HL          ; by multiplying it by two
    EX      DE, HL
    DJNZ    @MLOOP          ; repeat until no more bits.
@END:
```

```
    cargo run --release --example multiply_u16
```