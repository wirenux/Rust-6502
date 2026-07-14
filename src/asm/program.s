; test loop and DEC INC
.org $8000

START:
    LDA #$FF        ; 1. Load A with 3
    STA $40         ; 2. Store it at Zero Page address $40 (our counter)

LOOP:
    DEC $40         ; 3. Decrement the value in memory $40 by 1
    BNE LOOP        ; 4. If the Zero flag (Z) is 0 (meaning the counter isn't 0 yet),
                    ;    branch back to LOOP.

    BRK             ; 5. Stop once the counter hits 0!