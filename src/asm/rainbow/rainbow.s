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

    clc
    adc #8
    sta $0300,X ; page 1

    clc
    adc #8
    sta $0400,X ; page 2

    clc
    adc #8
    sta $0500,X ; page 3

    inx
    bne fill_loop

    brk

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ