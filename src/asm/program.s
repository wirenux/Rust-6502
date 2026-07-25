.segment "CODE"

start:
  lda #$01
  sta $0200

loop:
  ror
  sta $0204

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ
