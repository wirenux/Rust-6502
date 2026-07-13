.org $8000

START:
    ; --- SETUP Prepare data in the Zero Page ---
    LDA #$55        ; Load A with $55 (0101 0101 in binary)
    STA $42         ; Store it at Zero Page address $0042

    ; --- 1. Test Indexed Addressing (LDA ZeroPage,X) ---
    LDX #$02        ; Load X register with 2
    LDA $40,X       ; Read from Base $40 + X ($02). Reads address $0042.
                    ; > Accumulator should now be $55 again!

    ; --- 2. Test AND Immediate ---
    ; A is currently $55 (0101 0101)
    AND #$0F        ; AND with $0F (0000 1111)
                    ; > Accumulator should become $05 (0000 0101)

    ; --- 3. Test ORA Immediate ---
    ; A is currently $05 (0000 0101)
    ORA #$80        ; OR with $80 (1000 0000)
                    ; > Accumulator should become $85 (1000 0101)
                    ; > The Negative flag (N) should now be SET (1)!

    ; --- 4. Test EOR Immediate ---
    ; A is currently $85 (1000 0101)
    EOR #$FF        ; XOR with $FF (1111 1111)
                    ; > Accumulator should become $7A (0111 1010)
                    ; > The Negative flag (N) should now be CLEARED (0)!

    BRK             ; Stop execution