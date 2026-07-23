# Writing & Compiling 6502 Assembly

This guide walk through writing custom 6502 assembly programs for the `rust6502` emulator using the `cc65` toolchain (`ca65` assembler and `ld65` linker).

## System Memory Map

When writing assembly for this emulator| keep the internal memory layout in mind:

| Memory Range | Function | Description |
| ------------ | -------- | ----------- |
| `$0000 - $00FF` |Zero Page|Fast 8-bit addressing mode RAM
| `$0100 - $01FF` |Stack Pointer|System Call/Return Stack
| `$0200 - $05FF` |Video RAM|"32x32 pixels (1,024 bytes, 1 byte = 1 color)"
| `$0600 - $BFFF` |General RAM|Free memory for user variables
| `$C000 - $FFFF` |ROM Space|Program Code & Read-Only Data (16 KB)
| `$FFFC - $FFFD` |Reset Vector|16-bit address pointing to program entry point

## 1. Linker Configuration

The linker configuration tells `ld65` how to lay out you code into memory and ensures the 16-bit reset vector is placed at `$FFFC`.

Create a file name `linker.cfg`:

```config
MEMORY {
    RAM: start = $0000, size = $0800, type = rw, file = "";
    ROM: start = $C000, size = $4000, type = ro, file = %O, fill = yes, fillval = $00;
}

SEGMENTS {
    ZEROPAGE: load = RAM, type = zp;
    BSS:      load = RAM, type = bss;
    CODE:     load = ROM, type = ro;
    RODATA:   load = ROM, type = ro, optional = yes;
    VECTORS:  load = ROM, type = ro, start = $FFFC;
}
```

## 2. Sample Program (`program.s`)

Here is a basic program that initialize the CPU and paints the top 4 rows of the virtual screen with <span style="color: rgb(255, 0, 0); font-size: 1.2em;">■</span> **Red** pixels (color `$02`).

Create a file named `program.s` :

```asm
.segment "CODE"

RESET:
    cld             ; Disable decimal mode
    ldx #$FF
    txs             ; Reset stack pointer to $01FF

    ; --- Fill top part of Video RAM ($0200-$02FF) ---
    lda #$02        ; Palette color #6 (Red)
    ldx #$00        ; Offset counter

fill_screen:
    sta $0200, x    ; Write color byte to Video RAM
    inx
    cpx #128        ; Fill first 128 pixels (4 rows)
    bne fill_screen

loop:
    jmp loop        ; Infinite loop to keep CPU running

; --- Hardware Interrupt Vectors ---
.segment "VECTORS"
    .word RESET     ; NMI Vector (unused, points to RESET)
    .word RESET     ; Reset Vector ($FFFC - CPU boot entry point)
    .word RESET     ; IRQ/BRK Vector (unused, points to RESET)
```

## 3. How to compile

### Prerequisites

Install the `cc65` toolchain:

* macOS: `brew install cc65`
* Ubuntu / Debian: `sudo apt install cc65`
* Windows: Download binaries from cc65.github.io or use `winget install cc65`.

### Compilation Steps

Run those tow commands in your terminal :

```bash
# 1. Assemble source file into an object file (.o)
ca65 program.s -o program.o

# 2. Link object file into a raw binary ROM (.bin)
ld65 program.o -C linker.cfg -o program.bin
```

## 4. Running in `rust6502`

Once compiled, launch your executable directly with the  generated `.bin` file:

```bash
rust6502
```