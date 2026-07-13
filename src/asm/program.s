.org $8000

START:
    LDA #$10
    STA $05
    LDA #$05
    ADC $05
    STA $06
    SBC $05
    BRK