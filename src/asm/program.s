ptr = $00

.segment "CODE"

start:
  lda #$00
  sta ptr
  sta ptr+1
  lda #$02
  ldy #$00

loop:
  sta (ptr), Y
  iny
  bne loop
  inc ptr+1

  lda ptr+1
  cmp #$D0
  bne continue
  rts

continue:
  lda #$02
  jmp loop

.segment "VECTORS"
  .word start
  .word start
  .word start
