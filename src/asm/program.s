; Goal Jump to a routine set X to 5 return and confirm PC
.org $8000

START:
    LDA #$01        ; Load A with 1
    JSR SUB_ROUTINE ; Jump to address $8005
    LDA #$02        ; We should end up here after RTS
    BRK             ; Stop

SUB_ROUTINE:
    LDX #$05        ; Set X to 5
    RTS             ; Return to after JSR