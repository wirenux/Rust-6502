* = $8000 ; say to the assembler code start @ addr 0x8000

    LDA #$42
    TAX
    TAY

    INX
    INY

    DEX
    DEY

    TXA
    TYA

    LDA #$01
    TAX
    DEX

    DEY
    TYA
    LDA #$00
    TAY
    DEY

    NOP