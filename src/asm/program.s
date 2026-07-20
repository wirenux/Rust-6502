.segment "CODE"

start:
    ldx #$00
    lda #$02

fill_loop:
    sta $0200,X ; page 0
    sta $0300,X ; page 1
    sta $0400,X ; page 2
    sta $0500,X ; page 3

    inx
    bne fill_loop

done:
    jmp done

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ