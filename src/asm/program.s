; ==============================================================================
; ca65 Assembly Integration for Dynamic Font Renderer
; Integrates directly with generated `font.inc` lookup tables & glyph maps
; ==============================================================================

.p02

; --- Memory Mapped I/O Registers ---
KBD_DATA   = $BFF0      ; Read PS/2 scancode byte
KBD_STATUS = $BFF1      ; Bit 0: Data ready flag
KBD_ACK    = $BFF2      ; Write to acknowledge/pop hardware queue

SCREEN     = $0200      ; Screen RAM (32x32 pixels, 1 byte/pixel)
SCREEN_W   = 32
IMAGE_W    = 16
IMAGE_H    = 16

; Centered image position: Row 8, Col 8 -> $0200 + (8 * 32) + 8 = $0308
START_ADDR = SCREEN + (8 * SCREEN_W) + 8

; --- Zero Page Storage ---
.segment "ZEROPAGE"
src_ptr:    .res 2
dst_ptr:    .res 2
shift_flag: .res 1
break_flag: .res 1
scancode:   .res 1
row_count:  .res 1

; --- Main Code Segment ---
.segment "CODE"

.proc main
    sei                 ; Disable interrupts
    cld                 ; Clear decimal mode

    lda #0
    sta shift_flag
    sta break_flag

    jsr clear_screen

poll_kbd:
    ; Wait for keyboard status bit 0
    lda KBD_STATUS
    and #$01
    beq poll_kbd

    ; Fetch scancode and pop hardware queue
    lda KBD_DATA
    sta KBD_ACK
    sta scancode

    ; Check if previous byte was PS/2 break prefix ($F0)
    lda break_flag
    beq check_break_prefix

    ; Currently handling key release event
    lda #0
    sta break_flag

    ; If Shift ($12 or $59) was released, clear shift flag
    lda scancode
    cmp #$12            ; Left Shift release
    beq release_shift
    cmp #$59            ; Right Shift release
    beq release_shift
    jmp poll_kbd

release_shift:
    lda #0
    sta shift_flag
    jmp poll_kbd

check_break_prefix:
    lda scancode
    cmp #$F0
    bne handle_press
    lda #1
    sta break_flag
    jmp poll_kbd

handle_press:
    ; If Shift ($12 or $59) was pressed, set shift flag
    cmp #$12            ; Left Shift press
    beq press_shift
    cmp #$59            ; Right Shift press
    beq press_shift
    jmp resolve_glyph

press_shift:
    lda #1
    sta shift_flag
    jmp poll_kbd

resolve_glyph:
    ldx scancode

    ; Select unshifted or shifted lookup table based on shift_flag
    lda shift_flag
    bne fetch_shifted

    ; Unshifted table fetch
    lda table_lo, x
    sta src_ptr
    lda table_hi, x
    sta src_ptr+1
    jmp validate_glyph

fetch_shifted:
    ; Shifted table fetch
    lda table_shift_lo, x
    sta src_ptr
    lda table_shift_hi, x
    sta src_ptr+1

validate_glyph:
    ; If high-byte address is $00, scancode is not mapped in Python table
    lda src_ptr+1
    beq poll_kbd

    ; Valid character found: render to screen
    jsr clear_screen
    jsr draw_glyph
    jmp poll_kbd
.endproc

; --- Subroutine: Clear Screen ($0200-$05FF) ---
.proc clear_screen
    lda #0
    ldx #0
clear_loop:
    sta SCREEN, x
    sta SCREEN + $100, x
    sta SCREEN + $200, x
    sta SCREEN + $300, x
    inx
    bne clear_loop
    rts
.endproc

; --- Subroutine: Draw 16x16 Pixel Glyph ---
.proc draw_glyph
    ; Initialize destination screen address ($0308)
    lda #<START_ADDR
    sta dst_ptr
    lda #>START_ADDR
    sta dst_ptr+1

    lda #IMAGE_H
    sta row_count

row_loop:
    ldy #0              ; Column offset (0 to 15)
col_loop:
    lda (src_ptr), y    ; Read pixel byte (0 or 1) from generated image array
    sta (dst_ptr), y    ; Write byte to screen RAM
    iny
    cpy #IMAGE_W
    bne col_loop

    ; Advance src_ptr by 16 bytes (1 image row)
    clc
    lda src_ptr
    adc #IMAGE_W
    sta src_ptr
    lda src_ptr+1
    adc #0
    sta src_ptr+1

    ; Advance dst_ptr by 32 bytes (1 screen row)
    clc
    lda dst_ptr
    adc #SCREEN_W
    sta dst_ptr
    lda dst_ptr+1
    adc #0
    sta dst_ptr+1

    dec row_count
    bne row_loop

    rts
.endproc

; --- Include Generated Font Data ---
; Contains image bytes, table_lo, table_hi, table_shift_lo, table_shift_hi
.include "font.inc"

; --- 6502 Interrupt Vectors ---
.segment "VECTORS"
    .word 0             ; NMI
    .word main          ; RESET ($FFFC)
    .word 0             ; IRQ/BRK
