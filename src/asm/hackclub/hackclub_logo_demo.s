.segment "CODE"

start:
    ldx #$00

copy_loop:
    lda hacklub_image_data + $000, X
    sta $0200, X
    lda hacklub_image_data + $100, X
    sta $0300, X
    lda hacklub_image_data + $200, X
    sta $0400, X
    lda hacklub_image_data + $300, X
    sta $0500, X
    inx
    bne copy_loop

halt:
    brk

.segment "RODATA"

.include "logo.inc"

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ
