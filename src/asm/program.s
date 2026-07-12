.org $8000

START:
    SEC         ; to test CLC
    LDA #$FF    ; A = 255
    CLC         ; Clear Carry Flag before addition
    ADC #$02    ; 255 + 2 = 1, Carry is now 1 (0x01FF + 0x0002 = 0x0201)
    STA $20     ; Store Low Byte result (01) at $20

    LDA #$00    ; Load 0 for the high byte
    ADC #$00    ; 0 + 0 + 1 (Carry from previous instruction) = 1
    STA $21     ; Store High Byte result (01) at $21

    ; SEC use
    SEC         ; Carry = 1
    LDA #$01    ; A = 1
    ADC #$00    ; 1 + 0 + 1 (Carry) = 2

    BRK