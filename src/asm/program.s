.org $8000 ; say to the assembler code start @ addr 0x8000

START:
    LDA #$42
    LDX #$12

    STA $10

    TAX

    LDA $15

    JMP TARGET

TRAP:
    NOP

TARGET:
    BRK

.org $FFFC
    .word START