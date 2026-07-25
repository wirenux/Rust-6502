.segment "CODE"

start:
  lda #$01 ; Load 1 into the A register

loop:
  ror ; Rotate to the right the value present in the A register
  sta $0200 ; Store the value present in the A register at $0200 (im memory)
  jmp loop

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ
