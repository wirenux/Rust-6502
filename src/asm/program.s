.org $8000

; --- Test BEQ (Branch if Equal) ---
LDA #$00    ; Load 0 into A (Sets Zero Flag to 1)
BEQ forward ; Should branch because Z=1
LDX #$FF    ; If it doesn't branch, X becomes 0xFF (Failure)
forward:
LDX #$01    ; If it branches, X becomes 0x01 (Success)

; --- Test BNE (Branch if Not Equal) ---
LDA #$05    ; Load 5 into A (Sets Zero Flag to 0)
BNE branch  ; Should branch because Z=0
LDY #$FF    ; If it doesn't branch, Y becomes 0xFF (Failure)
branch:
LDY #$01    ; If it branches, Y becomes 0x01 (Success)