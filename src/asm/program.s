.setcpu "6502"

.segment "CODE"

reset:
    clc
    lda #$42
    brk

    cmp #$42
    bne test_fail
    bcc test_fail

    ldx #$FF
    brk

interrupt_handler:
    rti

test_fail:
    ldx #$EE
    brk


.segment "VECTORS"

.word 0
.word reset
.word interrupt_handler