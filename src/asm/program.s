.segment "CODE"

start:
  lda #$00
  sta $80

  ldx #$02

next_quarter:
  stx $81
  txa
  ldy #$00

draw_pixel:
  sta ($80), Y
  
  iny
  bne draw_pixel
  
  inx
  cpx #$06
  bne next_quarter

halt:
  brk

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ
