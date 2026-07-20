TMP = $00

.segment "CODE"

start:
    ldx #$00

fill_loop:
    txa
    lsr
    lsr
    lsr
    lsr
    lsr
    sta TMP

    txa
    clc
    adc TMP
    sta $0200,X ; page 0

    inx
    bne fill_loop

    brk

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ