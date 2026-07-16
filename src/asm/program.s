.segment "CODE"

start:
    ldx #$FF
    txs

    ; --- STA zeropage,X ---
    ; write $AA to $10 + reg_x(=$05) = $15
    ldx #$05
    lda #$AA
    sta $10,X          ; expect mem[$0015] = $AA

    ; --- STA absolute,Y ---
    ; write $BB to $0200 + reg_y(=$03) = $0203
    ldy #$03
    lda #$BB
    sta $0200,Y         ; expect mem[$0203] = $BB

    ; --- STA (indirect,X) ---
    ; pointer table entry at zp $30 + reg_x(=$02) = $32/$33
    ; must contain the target address ourselves first
    lda #$00
    sta $32             ; low byte of target addr  -> $0300
    lda #$03
    sta $33             ; high byte of target addr -> $0300
    ldx #$02
    lda #$CC
    sta ($30,X)          ; ptr = $30+X = $32 -> reads $0300 ; expect mem[$0300] = $CC

    ; --- STA (indirect),Y ---
    ; pointer at zp $40 must contain a base address, then add Y
    lda #$00
    sta $40             ; low byte of base addr  -> $0400
    lda #$04
    sta $41             ; high byte of base addr -> $0400
    ldy #$10
    lda #$DD
    sta ($40),Y          ; base=$0400 + Y($10) = $0410 ; expect mem[$0410] = $DD

    ; --- CPY zeropage ---
    lda #$07
    sta $50             ; store comparison value $07 at zp $50
    ldy #$07
    cpy $50             ; Y == mem[$50] -> Z=1, C=1

    ; --- CPY absolute ---
    lda #$09
    sta $0250           ; store comparison value $09 at $0250
    ldy #$05
    cpy $0250           ; Y($05) < mem($09) -> Z=0, C=0

done:
    jmp done            ; trap so execution stops here

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ