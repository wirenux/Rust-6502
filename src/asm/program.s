; test rotate
.org $8000

START:
    ; --- SETUP Put $80 (1000 0000) into Zero Page $40 ---
    LDA #$80
    STA $40

    ; --- 1. Test ROL (Rotate Left) with Carry Set ---
    SEC             ; Set Carry Flag (C = 1)
    ROL $40         ; Rotate Left memory $40
                    ; > Old Carry (1) should go into Bit 0
                    ; > Bit 7 (1) should go into the Carry Flag
                    ; > Memory $40 should become $01 (0000 0001)
                    ; > Carry (C) should remain 1

    ; --- 2. Test ROR (Rotate Right) with Carry Set ---
    ; Memory $40 is currently $01 (0000 0001), Carry is 1
    ROR $40         ; Rotate Right memory $40
                    ; > Old Carry (1) should go into Bit 7
                    ; > Bit 0 (1) should go into the Carry Flag
                    ; > Memory $40 should slide back to $80 (1000 0000)
                    ; > Carry (C) should remain 1

    ; --- 3. Test ROR with Carry Cleared ---
    ; Memory $40 is currently $80 (1000 0000), Carry is 1
    CLC             ; Clear Carry Flag (C = 0)
    ROR $40         ; Rotate Right memory $40
                    ; > Old Carry (0) should go into Bit 7
                    ; > Bit 0 (0) should go into the Carry Flag
                    ; > Memory $40 should become $40 (0100 0000)
                    ; > Carry (C) should become 0

    BRK             ; Stop execution