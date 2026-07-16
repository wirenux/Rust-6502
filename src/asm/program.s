.segment "CODE"

start:
    ldx #$FF
    txs
    lda #$01
    sta $00
loop:
    jmp loop

.segment "VECTORS"
    .word start
    .word start
    .word start