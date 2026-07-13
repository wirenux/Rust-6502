; test shift
.org $8000

START:
    ; --- SETUP Prepare data in the Zero Page ---
    LDA #$81        ; Load A with $81 (1000 0001 in binary)
    STA $40         ; Store it at Zero Page address $0040

    ; --- 1. Test ASL (Arithmetic Shift Left) Memory ---
    ; Memory $40 is currently $81 (1000 0001)
    ASL $40         ; Shift bits left
                    ; > Memory $40 should become $02 (0000 0010)
                    ; > Carry Flag (C) should be SET (1) because Bit 7 was 1.
                    ; > Negative Flag (N) should be CLEARED (0).

    ; --- 2. Test LSR (Logical Shift Right) Memory ---
    ; Memory $40 is currently $02 (0000 0010)
    LSR $40         ; Shift bits right
                    ; > Memory $40 should become $01 (0000 0001)
                    ; > Carry Flag (C) should be CLEARED (0) because Bit 0 was 0.

    ; --- 3. Test LSR into Zero ---
    ; Memory $40 is currently $01 (0000 0001)
    LSR $40         ; Shift bits right again
                    ; > Memory $40 should become $00 (0000 0000)
                    ; > Carry Flag (C) should be SET (1) because Bit 0 was 1.
                    ; > Zero Flag (Z) should be SET (1) because the result is 0.

    ; --- 4. Final Verification ---
    LDA $40         ; Load the final memory value into A to see it in the logs
                    ; > Accumulator should be $00.

    BRK             ; Stop execution