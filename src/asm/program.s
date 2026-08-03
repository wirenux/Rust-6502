KBD_DATA   = $BFF0
KBD_STATUS = $BFF1
KBD_ACK    = $BFF2
KBD_READY  = $01

BREAK_CODE = $F0
EXT_CODE   = $E0

.segment "CODE"

reset:
    LDA #$00
    STA $00

main_loop:
    LDA KBD_STATUS
    AND #KBD_READY
    BEQ main_loop

    LDA KBD_DATA
    
    CMP #BREAK_CODE
    BEQ ack_and_loop
    
    CMP #EXT_CODE
    BEQ ack_and_loop

    STA $00

ack_and_loop:
    STA KBD_ACK
    JMP main_loop

.segment "VECTORS"
  .word reset     ; NMI
  .word reset     ; RESET
  .word reset     ; IRQ
