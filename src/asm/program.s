.setcpu "6502"

.segment "CODE"

reset:
    ; ------------------------------------------------
    ; TEST 1: STA Absolute, X (Zero Offset)
    ; ------------------------------------------------
    ldx #$00        ; Set X index to 0
    lda #$42        ; The magic test value
    sta $0200,X     ; Write to $0200 + 0 = $0200

    lda $0200       ; Read it back directly
    cmp #$42        ; Is it $42?
    bne test_fail   ; If not, branch to failure

    ; ------------------------------------------------
    ; TEST 2: STA Absolute, X (Positive Offset)
    ; ------------------------------------------------
    ldx #$05        ; Set X index to 5
    lda #$99        ; A different test value
    sta $0200,X     ; Write to $0200 + 5 = $0205

    lda $0205       ; Read it back directly
    cmp #$99        ; Is it $99?
    bne test_fail   ; If not, branch to failure

    ; ------------------------------------------------
    ; SUCCESS!
    ; ------------------------------------------------
    ldx #$FF        ; Load $FF into X to indicate success
    brk             ; Halt the CPU

interrupt_handler:
    rti

test_fail:
    ldx #$EE        ; Load $EE into X to indicate failure
    brk             ; Halt the CPU


.segment "VECTORS"

.word 0                 ; NMI
.word reset             ; RESET
.word interrupt_handler ; IRQ/BRK