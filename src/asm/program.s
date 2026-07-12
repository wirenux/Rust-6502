.org $8000

START:
    LDA #$05
    LDX #$0A
    LDY #$0F
    TAX
    TAY
    TXA
    TYA
    INX
    INY
    DEX
    DEY
    STA $20
    STX $21
    STY $22
    LDA $20
    LDX $21
    LDY $22
    STA $8050
    LDA $8050
    NOP
    JMP TARGET

TARGET:
    JMP (POINTER)

.org $8040
POINTER:
    .word FINISH

FINISH:
    BRK

.org $FFFC
    .word START