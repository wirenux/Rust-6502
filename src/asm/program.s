.segment "CODE"

start:
    ldx #$FF
    txs

    ; =========================================================
    ; ORA — exercise all 7 addressing modes
    ; Pattern: reg_a always starts at $0F (00001111)
    ; ORA'd with $F0 (11110000) -> result $FF in every case
    ; =========================================================

    ; --- ORA immediate ---
    lda #$0F
    ora #$F0
    sta $10             ; expect mem[$0010] = $FF

    ; --- ORA zeropage ---
    lda #$F0
    sta $20             ; operand lives at zp $20
    lda #$0F
    ora $20
    sta $11             ; expect mem[$0011] = $FF

    ; --- ORA zeropage,X ---
    lda #$F0
    sta $25             ; operand at $23 + X(=$02) = $25
    ldx #$02
    lda #$0F
    ora $23,X
    sta $12             ; expect mem[$0012] = $FF

    ; --- ORA absolute ---
    lda #$F0
    sta $0300           ; operand at $0300
    lda #$0F
    ora $0300
    sta $13             ; expect mem[$0013] = $FF

    ; --- ORA absolute,X ---
    lda #$F0
    sta $0310           ; operand at $0305 + X(=$0B)
    ldx #$0B
    lda #$0F
    ora $0305,X
    sta $14             ; expect mem[$0014] = $FF

    ; --- ORA absolute,Y ---
    lda #$F0
    sta $0320           ; operand at $0315 + Y(=$0B)
    ldy #$0B
    lda #$0F
    ora $0315,Y
    sta $15             ; expect mem[$0015] = $FF

    ; --- ORA (indirect,X) ---
    lda #$00
    sta $34             ; pointer low  -> target $0330
    lda #$03
    sta $35             ; pointer high -> target $0330
    lda #$F0
    sta $0330           ; operand value at the target
    ldx #$02
    lda #$0F
    ora ($32,X)          ; ptr = $32+X = $34 -> reads $0330
    sta $16             ; expect mem[$0016] = $FF

    ; --- ORA (indirect),Y ---
    lda #$00
    sta $40             ; base low  -> $0400
    lda #$04
    sta $41             ; base high -> $0400
    lda #$F0
    sta $0410           ; base($0400) + Y($10)
    ldy #$10
    lda #$0F
    ora ($40),Y
    sta $17             ; expect mem[$0017] = $FF


    ; =========================================================
    ; SBC — exercise all 7 addressing modes
    ; Pattern: SEC (carry=1, so no borrow), reg_a = $50,
    ; subtract $10 each time -> result $40, carry stays set (no borrow)
    ; =========================================================

    ; --- SBC immediate ---
    sec
    lda #$50
    sbc #$10
    sta $50             ; expect mem[$0050] = $40, carry=1

    ; --- SBC zeropage ---
    lda #$10
    sta $60             ; operand at zp $60
    sec
    lda #$50
    sbc $60
    sta $51             ; expect mem[$0051] = $40

    ; --- SBC zeropage,X ---
    lda #$10
    sta $65             ; operand at $63 + X(=$02)
    ldx #$02
    sec
    lda #$50
    sbc $63,X
    sta $52             ; expect mem[$0052] = $40

    ; --- SBC absolute ---
    lda #$10
    sta $0500
    sec
    lda #$50
    sbc $0500
    sta $53             ; expect mem[$0053] = $40

    ; --- SBC absolute,X ---
    lda #$10
    sta $0510
    ldx #$0B
    sec
    lda #$50
    sbc $0505,X
    sta $54             ; expect mem[$0054] = $40

    ; --- SBC absolute,Y ---
    lda #$10
    sta $0520
    ldy #$0B
    sec
    lda #$50
    sbc $0515,Y
    sta $55             ; expect mem[$0055] = $40

    ; --- SBC (indirect,X) ---
    lda #$00
    sta $70
    lda #$05
    sta $71             ; pointer -> $0530
    lda #$10
    sta $0530
    ldx #$02
    sec
    lda #$50
    sbc ($6E,X)          ; ptr = $6E+X = $70 -> reads $0530
    sta $56             ; expect mem[$0056] = $40

    ; --- SBC (indirect),Y ---
    lda #$00
    sta $80
    lda #$06
    sta $81             ; base -> $0600
    lda #$10
    sta $0610           ; base($0600) + Y($10)
    ldy #$10
    sec
    lda #$50
    sbc ($80),Y
    sta $57             ; expect mem[$0057] = $40


    ; =========================================================
    ; Bug re-checks: CPX/CPY zeropage, LDX absolute,Y, STX zeropage,Y
    ; =========================================================

    ; CPX zeropage: X == mem -> Z=1, C=1
    lda #$07
    sta $90
    ldx #$07
    cpx $90

    ; CPY zeropage: Y < mem -> Z=0, C=0
    lda #$09
    sta $91
    ldy #$05
    cpy $91

    ; LDX absolute,Y
    lda #$77
    sta $0710           ; target = $0705 + Y($0B)
    ldy #$0B
    ldx $0705,Y          ; expect reg_x = $77

    ; STX zeropage,Y
    ldx #$99
    ldy #$05
    stx $95,Y            ; expect mem[$009A] = $99

done:
    inx
    jmp done

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ