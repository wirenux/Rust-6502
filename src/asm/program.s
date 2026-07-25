.segment "CODE"

start:
  lda #$01

loop:
  ror
  sta $0200
  jmp loop

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ
