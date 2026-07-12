.org $8000

START:
    LDA #$05     ; A = 5
    ADC #$03     ; 5 + 3 = 8 (No flags set)

    LDA #$FF     ; A = 255
    ADC #$01     ; 255 + 1 = 0 (Carry flag set)

    LDA #$40     ; A = 64
    ADC #$40     ; 64 + 64 = 128 (V (Overflow) flag set, Result is Negative)

    BRK