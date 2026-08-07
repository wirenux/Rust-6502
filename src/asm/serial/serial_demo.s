; ===============
;  Serial Demo 1
; ===============
;
; To use this program you should use the rust6502 emulator built by @wirenux
; To use the Serial Monitor you should try connect to it when the emulator is running
; using : nc 127.0.0.1 8080

.segment "CODE"

start:
    ldx #$FF
    txs
    ldx #$00

main_loop:
    bit $BFE1
    bpl main_loop


    lda $BFE0

    sta $BFE0

    
    lda #$02
    sta $0200, x

    inx
    jmp main_loop

.segment "VECTORS"
  .word $0000
  .word start
  .word $0000


; ===============
;  Serial Demo 2
; ===============

; .segment "CODE"
;
; start:
;     lda #'A'
;     sta $BFE0
;
; .segment "VECTORS"
;   .word $0000
;   .word start
;   .word $0000
