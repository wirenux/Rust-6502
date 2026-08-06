.p02

KBD_DATA = $BFF0    ; read PS/2 scancode byte
KBD_STATUS = $BFF1  ; bit 0: data ready flag
KBD_ACK = $BFF2     ; write to acknowledge/pop hardware queue

SCREEN = $0200
SCREEN_W = 32
IMAGE_W = 16
IMAGE_H = 16

; centered img
START_ADDR = SCREEN + (8 * SCREEN_W) + 8

.segment "ZEROPAGE"
src_ptr:    .res 2
dst_ptr:    .res 2
shift_flag: .res 1
break_flag: .res 1
scancode:   .res 1
row_count:  .res 1

.segment "CODE"

.proc main
    sei
    cld

    lda #0
    sta shift_flag
    sta break_flag

    jsr clear_screen

poll_kbd:
    lda KBD_STATUS
    and #$01
    beq poll_kbd

    lda KBD_DATA
    sta KBD_ACK
    sta scancode

    lda break_flag
    beq check_break_prefix

    lda #0
    sta break_flag

    lda scancode
    cmp #$12
    beq release_shift
    cmp #$59
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
    cmp #$12
    beq press_shift
    cmp #$59
    beq press_shift
    jmp resolve_glyph

press_shift:
    lda #1
    sta shift_flag
    jmp poll_kbd

resolve_glyph:
    ldx scancode

    lda shift_flag
    bne fetch_shifted

    lda table_lo, x
    sta src_ptr
    lda table_hi, x
    sta src_ptr+1
    jmp validate_glyph

fetch_shifted:
    lda table_shift_lo, x
    sta src_ptr
    lda table_shift_hi, x
    sta src_ptr+1

validate_glyph:
    lda src_ptr+1
    beq poll_kbd

    jsr clear_screen
    jsr draw_glyph
    jmp poll_kbd
.endproc

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

.proc draw_glyph
    lda #<START_ADDR
    sta dst_ptr
    lda #>START_ADDR
    sta dst_ptr+1

    lda #IMAGE_H
    sta row_count

row_loop:
    ldy #0
col_loop:
    lda (src_ptr), y
    sta (dst_ptr), y
    iny
    cpy #IMAGE_W
    bne col_loop

    clc
    lda src_ptr
    adc #IMAGE_W
    sta src_ptr
    lda src_ptr+1
    adc #0
    sta src_ptr+1

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

.include "font.inc"

.segment "VECTORS"
    .word 0     ; NMI
    .word main  ; RESET ($FFFC)
    .word 0     ; IRQ/BRK
