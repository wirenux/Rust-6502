; ===========
; WOZ Monitor
; ===========

; Modified for the rust6502 emulator by Wirenux
; Use the ca65 compiler and the ld65 linker

XAML = $24
XAMH = $25
STL = $26
STH = $27
L = $28
H = $29
YSAV = $2A
MODE = $2B

IN = $0600 ; input buffer

KBD = $BFE0
KBDCR = $BFE1
DSP = $BFE0 ; display; here the Serial Console

.segment "CODE"

reset:
  cld               ; Clear decimal arithmetic mode
  lda #$5C          ; Print '\' prompt on boot
  jsr echo          ; Output it
  ldy #$01

notcr:
  cmp #$08          ; Backspace ?
  beq backspace     ; Yes
  cmp #$7F          ; ESC ?
  beq escape        ; Yes
  iny               ; Advance text index
  bpl nextchar      ; Auto ESC if line > 127

escape:
  lda #$5C          ; '\'
  jsr echo          ; Output it

getline:
  lda #$0D          ; CR
  jsr echo          ; Output it
  lda #$5C          ; '\'
  jsr echo          ; Output it
  ldy #$01          ; Initialize text index
  bne nextchar

backspace:
  cpy #$01          ; Don't delete past the '\' prompt
  beq nextchar
  dey               ; Back up text index
  lda #$08          ; Backspace
  jsr echo          ; Output it
  lda #$20          ; Space
  jsr echo          ; Output it
  lda #$08          ; Backspace
  jsr echo          ; Output it
  jmp nextchar

nextchar:
  bit KBDCR         ; Key ready ?
  bpl nextchar      ; Loop untik ready
  lda KBD           ; Load character
  sta IN, Y         ; Add to text buffer
  jsr echo          ; Display character
  cmp #$0D          ; CR ?
  bne notcr         ; No

  ldy #$FF          ; Reset text index
  lda #$00          ; For XAM mode
  tax               ; 0 -> X
  beq setmode       ; Branch over the mode setters

setblock:
  lda #$AE          ; Force a bit7-set sentinel for BLOCK XAM
  bne setmode       ; Always taken

setstor:
  asl               ; Leaves $74 if setting STOR mode

setmode:
  sta MODE          ; $00 = XAM, $74 = STOR, $AE = BLOCK XAM

blskip:
  iny               ; Advance text index

nextitem:
  lda IN, Y         ; Get character
  cmp #$0D          ; CR?
  beq getline       ; Yes, done this line
  cmp #$2E          ; "."?
  bcc blskip        ; Skip delimiter
  beq setblock      ; Set BLOCK XAM mode
  cmp #$3A          ; ":"?
  beq setstor       ; Set STOR mode
  cmp #$52          ; "R"?
  beq run           ; Run user program
  stx L             ; $00 -> L
  stx H             ;        and H
  sty YSAV          ; Save Y for comparison

nexthex:
  lda IN,Y          ; Get character for hex text
  eor #$30          ; Map digits to $00-09
  cmp #$0A          ; Digit?
  bcc dig           ; Yes
  adc #$88          ; Map letter "A"-"F" to $FA-FF
  cmp #$FA          ; Hex letter?
  bcc nothex        ; No

dig:
  asl
  asl               ; Hex digit to MSD of A
  asl
  asl
  ldx #$04          ; Shift count

hexshift:
  asl               ; Hex digit left, MSB to carry
  rol L             ; Rotate into LSD
  rol H             ; Rotate into MSD's
  dex               ; Done 4 shifts?
  bne hexshift      ; No
  iny               ; Advance text index
  bne nexthex       ; Always taken

nothex:
  cpy YSAV          ; Check if L, H empty
  bne @not_esc      ; If not equal, continue
  jmp escape        ; Otherwise jump far to ESCAPE

@not_esc:
  bit MODE          ; Test MODE byte
  bvc notstor       ; B6=0 for STOR, 1 for XAM and BLOCK XAM
  lda L             ; LSD's of hex data
  sta (STL,X)       ; Store at current 'store index'
  inc STL           ; Increment store index
  bne nextitem      ; Get next item (no carry)
  inc STH           ; Add carry to high order

tonextitem:
  jmp nextitem      ; Get next command item

run:
  jmp (XAML)        ; Run at current XAM index

notstor:
  bmi xamnext       ; B7=0 for XAM, 1 for BLOCK XAM
  ldx #$02          ; Byte count

setadr:
  lda L-1,X         ; Copy hex data to
  sta STL-1,X       ;     'store index'
  sta XAML-1,X      ;     'XAM index'
  dex
  bne setadr

nxtprnt:
  bne prdata        ; NE means no address to print
  lda #$0D          ; CR
  jsr echo          ; Output it
  lda XAMH          ; High-order byte
  jsr prbyte        ; Output in hex
  lda XAML          ; Low-order byte
  jsr prbyte        ; Output in hex
  lda #$3A          ; ":"
  jsr echo


prdata:
  lda #$20          ; Blank
  jsr echo          ; Output it
  lda (XAML,X)      ; Get data byte
  jsr prbyte        ; Output in hex

xamnext:
  stx MODE          ; 0 -> Mode
  lda XAML
  cmp L             ; Acts as the lower-byte subtraction. Must set Carry if A >= M.
  lda XAMH
  sbc H             ; Subtracts H and the inverted Carry (borrow).
  bcs tonextitem    ; Branches if the 16-bit value XAM >= L/H.

  inc XAML          ; Advance to the next address
  bne mod8chk       ; If no overflow, skip the high-byte increment
  inc XAMH          ; Carry over to the high byte

mod8chk:
  lda XAML
  and #$07
  bpl nxtprnt

prbyte:
  pha               ; Save A for LSD
  lsr
  lsr
  lsr
  lsr               ; MSD to LSD position
  jsr prhex         ; Output hex digit
  pla               ; Restore A

prhex:
  and #$0F          ; Mask LSD
  ora #$30          ; Add "0"
  cmp #$3A          ; Digit?
  bcc echo          ; Yes
  adc #$06          ; Add offset for letter

echo:
  sta DSP           ; Write to custom Serial Port
  rts               ; Return instantly

.segment "VECTORS"
        .word $0000     ; NMI
        .word reset     ; RESET
        .word $0000     ; IRQ