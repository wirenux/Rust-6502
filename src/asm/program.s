.segment "CODE"

start:
  lda #$04
  pha

  lda #$00
  sta $80

  pla
  sta $81

; loop:
;   lda #$02
;   sta $0200, X
;
;   cpx #255
;   inx
;
;   bne loop
;

halt:
  brk

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ
