.segment "CODE"

start:
    ldx #0
    lda #0

    lda #<$0306 ; row 8 col 6
    sta $00
    lda #>$0306
    sta $01
    ldx #<text_hello ; load low bytes in x
    ldy #>text_hello ; load high bytes in y
    jsr print_string

    lda #<$0405 ; row 16 col 5
    sta $00
    lda #>$0405
    sta $01
    ldx #<text_world ; load low bytes in x
    ldy #>text_world ; load high bytes in y
    jsr print_string

    brk

print_string:
    stx $05 ; save string pointer low
    sty $06 ; save string pointer high
    ldy #0

@char_loop: ; local function (can reuse the name)
    lda ($05), y ; read char index
    cmp #$FF ; compare A & #$FF
    beq @done ; if equal 0 then jmp

    ; set working pointer ($02/$04) = screen target ($00/$01)
    pha
    lda $00
    sta $02
    lda $01
    sta $03
    pla

    ; calculate font index offset
    sta $04 ; save glyph index
    asl ; A = index * 2
    asl ; A = index * 4
    clc
    adc $04 ; A = index * 5
    tax ; X = index in font_data

    lda #5
    sta $04 ; row loop counter

@row_loop:
    lda font_data, x
    pha

    ; check byte 2 (left pixel)
    and #%100
    beq @s1
    lda #1 ; white
    ldy #0
    sta ($02), y

@s1:
    pla
    pha
    and #%010
    beq @s2
    lda #1 ; white
    ldy #1
    sta ($02), y

@s2:
    pla
    and #%001
    beq @s3
    lda #1 ; white
    ldy #2
    sta ($02), y

@s3:
    inx ; go to next row byte in font table
    clc
    lda $02
    adc #32
    sta $02
    bcc @no_carry
    inc $03

@no_carry:
    dec $04
    bne @row_loop

    ; move base screen pointer right by 4 bytes (3px char + 1px space)
    clc
    lda $00
    adc #4
    sta $00
    bcc @no_carry2
    inc $01

@no_carry2:

    ; move to next char in text string
    inc $05
    bne @no_carry3
    inc $06

@no_carry3:
    ldy #0
    jmp @char_loop

@done:
    rts

text_hello:
    .byte 0, 1, 2, 2, 3, $FF ; H E L L O

text_world:
    .byte 4, 3, 5, 2, 6, 7, $FF ; W O R L D !

font_data:
    ; H
    .byte %101, %101, %111, %101, %101
    ; E
    .byte %111, %100, %110, %100, %111
    ; L
    .byte %100, %100, %100, %100, %111
    ; O
    .byte %111, %101, %101, %101, %111
    ; W
    .byte %101, %101, %101, %111, %101
    ; R
    .byte %110, %101, %110, %101, %101
    ; D
    .byte %110, %101, %101, %101, %110
    ; !
    .byte %010, %010, %010, %000, %010

.segment "VECTORS"
    .word start     ; NMI
    .word start     ; RESET
    .word start     ; IRQ