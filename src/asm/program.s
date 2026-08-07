SERIAL_DATA   = $BFE0
SERIAL_STATUS = $BFE1

; Framebuffer start address (adjust if your display RAM lives at $0200, $A000, etc.)
SCREEN        = $0200
WHITE_COLOR   = $FF

.segment "CODE"

start:
    ldx #$FF
    txs                 ; Initialize stack pointer to $01FF
    ldx #$00            ; Use X register as the pixel screen offset

main_loop:
    ; Poll status register ($BFE1). Bit 7 (0x80) is set when RX data is ready.
    bit SERIAL_STATUS   ; Transfers bit 7 directly to the N (Negative) flag
    bpl main_loop       ; Branch if Positive (bit 7 == 0, no data waiting)

    ; Read character from RX buffer ($BFE0)
    lda SERIAL_DATA

    ; Echo character back to TX ($BFE0) so it appears in terminal
    sta SERIAL_DATA

    ; Draw white pixel to screen memory
    lda #WHITE_COLOR
    sta SCREEN, x

    inx                 ; Advance to next pixel memory address
    jmp main_loop

.segment "VECTORS"
    .word $0000         ; NMI
    .word start         ; RESET
    .word $0000         ; IRQ
