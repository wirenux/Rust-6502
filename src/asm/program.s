* = $8000

START:
    LDX #$42
    TXS
    LDX #$00
    TSX

    LDA #$AA
    STA $0050
    LDA #$BB
    STA $0055

    LDA #$50
    STA $0020
    LDA #$00
    STA $0021

    LDA #$55
    STA $0025
    LDA #$00
    STA $0026

    LDA #$00
    LDX #$05
    LDA ($20,X)

    LDA #$00
    LDY #$05
    LDA ($20),Y

    LDX #$00
    LDY #$10
    LDX $45,Y

    LDX #$00
    LDY #$01
    LDX DATA_BLOCK,Y

    BRK

DATA_BLOCK:
    .byte $88, $99, $CC