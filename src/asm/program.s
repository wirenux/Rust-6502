* = $8000

START:
    CLC
    BCC BCC_OK
    JMP FAIL
BCC_OK:

    SEC
    BCS BCS_OK
    JMP FAIL
BCS_OK:

    LDA #$10
    BPL BPL_OK
    JMP FAIL
BPL_OK:

    LDA #$80
    BMI BMI_OK
    JMP FAIL
BMI_OK:

    CLV
    BVC BVC_OK
    JMP FAIL
BVC_OK:

    LDA #$7F
    CLC
    ADC #$01
    BVS BVS_OK
    JMP FAIL
BVS_OK:

    LDA #$C0
    STA $30

    LDA #$00
    BIT $30
    BNE FAIL
    BPL FAIL
    BVC FAIL

    LDA #$80
    BIT $30
    BEQ FAIL
    BPL FAIL
    BVC FAIL

    LDA #$00
    BIT BIT_DATA
    BNE FAIL
    BMI FAIL
    BVC FAIL

    LDX #$FF
    BRK

FAIL:
    LDX #$EE
    BRK

BIT_DATA:
    .byte $40