; =======
;  Snake
; =======
; This program is released under the MIT license
; This program is built for the rust6502 emulator created by @wirenux

KBD_DATA    = $BFF0
KBD_STATUS  = $BFF1
KBD_ACK     = $BFF2

SCREEN      = $0200

BLACK       = 0
RED         = 2
GREEN       = 5

KEY_UP      = $75
KEY_DOWN    = $72
KEY_LEFT    = $6B
KEY_RIGHT   = $74

DIR_UP      = 0
DIR_RIGHT   = 1
DIR_DOWN    = 2
DIR_LEFT    = 3

DELAY_OUTER = 60
DELAY_INNER = 255   ; modify to slow down or speed up

DIR         = $00
NEXT_DIR    = $01
HEAD_X      = $02
HEAD_Y      = $03
APPLE_X     = $04
APPLE_Y     = $05
RNG_STATE   = $06
NEW_X       = $07
NEW_Y       = $08
KBD_BYTE    = $09
BREAK_FLAG  = $0A
HEAD_PTR    = $0B
TAIL_PTR    = $0C
LENGTH      = $0D
CALC_X      = $0E
CALC_Y      = $0F
MUL_LO      = $10
MUL_HI      = $11
PTR         = $12   ; PTR/PTR+1 = 16 bit screen pointer
TEMP_DIR    = $14
FRAME_CNT   = $15
DRAW_I      = $16

BODY_X      = $0700 ; 256 bytes
BODY_Y      = $0800 ; 256 bytes

.segment "CODE"

reset:
  cld
  ldx #$FF
  txs
  
  ; set default direction to right
  lda #DIR_RIGHT
  sta DIR
  sta NEXT_DIR

  ; initial snake: 3 segment head at (16, 16)
  lda #0
  sta TAIL_PTR
  lda #2
  sta HEAD_PTR
  lda #3
  sta LENGTH

  lda #14
  sta BODY_X+0
  lda #16
  sta BODY_Y+0

  lda #15
  sta BODY_X+1
  lda #16
  sta BODY_Y+1
  
  lda #16
  sta BODY_X+2
  sta HEAD_X
  lda #16
  sta BODY_Y+2
  sta HEAD_Y


  lda #$A5  ; RNG seed (!= 0)
  sta RNG_STATE
  lda #0
  sta FRAME_CNT
  sta BREAK_FLAG

  ; draw initial 3 segments
  lda #0
  sta DRAW_I

draw_init:
  ldx DRAW_I
  lda BODY_X,x
  sta CALC_X
  lda BODY_Y,x
  sta CALC_Y
  jsr calc_addr
  ldy #0
  lda #GREEN
  sta (PTR),y
  inc DRAW_I
  lda DRAW_I
  cmp #3          ; 3 segment are drew ?
  bne draw_init   ; No

  jsr spawn_apple
  
main_loop:
  jsr delay_and_poll
  jsr update
  jmp main_loop

delay_and_poll:
  ldx #DELAY_OUTER
dap_outer:
  jsr read_input
  inc FRAME_CNT
  ldy #DELAY_INNER
dap_inner:
  dey
  bne dap_inner
  dex
  bne dap_outer
  rts

read_input:
  lda KBD_STATUS
  and #$01
  beq ri_done

  lda KBD_DATA
  sta KBD_BYTE

  cmp #$F0
  beq ri_setbreak

  lda BREAK_FLAG
  beq ri_makecode

  lda #0
  sta BREAK_FLAG
  jmp ri_ack

ri_makecode:
  lda KBD_BYTE
  cmp #KEY_UP       ; Key Press is UP ?
  bne ri_chk_down   ; No
  lda #DIR_UP
  sta NEXT_DIR
  jmp ri_ack

ri_chk_down:
  cmp #KEY_DOWN     ; Key Press is DOWN ?
  bne ri_chk_left   ; No
  lda #DIR_DOWN
  sta NEXT_DIR
  jmp ri_ack

ri_chk_left:
  cmp #KEY_LEFT     ; Key Press is LEFT ?
  bne ri_chk_right  ; No
  lda #DIR_LEFT
  sta NEXT_DIR
  jmp ri_ack

ri_chk_right:
  cmp #KEY_RIGHT    ; Key Press is RIGHT ?
  bne ri_ack        ; No
  lda #DIR_RIGHT
  sta NEXT_DIR
  jmp ri_ack

ri_setbreak:
  lda #1
  sta BREAK_FLAG

ri_ack:
  lda #0
  sta KBD_ACK
  jmp read_input

ri_done:
  rts

update:
  lda DIR
  clc
  adc #2
  and #3
  sta TEMP_DIR
  lda NEXT_DIR
  cmp TEMP_DIR      ; new direction = current ?
  beq upd_keepdir   ; Yes
  sta DIR

upd_keepdir:
  lda HEAD_X
  sta NEW_X
  lda HEAD_Y
  sta NEW_Y

  lda DIR
  cmp #DIR_UP       ; current dir = UP ?
  bne upd_chkr      ; No
  dec NEW_Y         ; Move Up
  jmp upd_moved

upd_chkr:
  cmp #DIR_RIGHT    ; current dir = RIGHT ?
  bne upd_chkd      ; No
  inc NEW_X         ; Move Right
  jmp upd_moved

upd_chkd:
  cmp #DIR_DOWN     ; current dire = DOWN ?
  bne upd_left      ; No
  inc NEW_Y         ; Move Down
  jmp upd_moved

upd_left:
  dec NEW_X         ; Move Left

upd_moved:
  lda NEW_X
  cmp #32           ; New_X over 32 ? (hit wall)
  bcc upd_chk_y     ; No, check Y
  jmp game_over     ; Yes, jump to game_over 

upd_chk_y:
  lda NEW_Y
  cmp #32           ; New_Y over 32 ? (hit wall)
  bcc upd_chk_self  ; No, check self
  jmp game_over     ; Yes

upd_chk_self:
  lda NEW_X
  sta CALC_X
  lda NEW_Y
  sta CALC_Y
  jsr calc_addr
  ldy #0
  lda (PTR),y
  cmp #GREEN        ; Hit itself ?
  bne upd_chk_apple ; No
  jmp game_over     ; Yes

upd_chk_apple:
  cmp #RED          ; Eat apple ?
  beq upd_eat       ; Yes
  jmp upd_normal

upd_eat:
  jsr advance_head
  inc LENGTH
  jsr spawn_apple
  rts
  
upd_normal:
  jsr advance_head
  jsr remove_tail
  rts

advance_head:       ; push NEW_X, NEW_Y as new head, then draw it
  inc HEAD_PTR
  ldx HEAD_PTR
  lda NEW_X
  sta BODY_X,x
  sta HEAD_X
  lda NEW_Y
  sta BODY_Y,x
  sta HEAD_Y

  ldy #0
  lda #GREEN
  sta (PTR),y
  rts

remove_tail:        ; erase tail pixel and advance tail ptr
  ldx TAIL_PTR
  lda BODY_X,x
  sta CALC_X
  lda BODY_Y,x
  sta CALC_Y
  jsr calc_addr
  ldy #0
  lda #BLACK
  sta (PTR),y
  inc TAIL_PTR
  rts

calc_addr:
  lda CALC_Y
  sta MUL_LO
  lda #0
  sta MUL_HI
  ldx #5

ca_shift:
  asl MUL_LO
  rol MUL_HI
  dex
  bne ca_shift

  clc
  lda MUL_LO
  adc CALC_X
  sta PTR
  lda MUL_HI
  adc #0
  sta PTR+1

  clc
  lda PTR
  adc #<SCREEN
  sta PTR
  lda PTR+1
  adc #>SCREEN
  sta PTR+1
  rts

rng_next:
  lda RNG_STATE
  eor FRAME_CNT
  asl a
  bcc rng_noeor
  eor #$B8

rng_noeor:
  sta RNG_STATE
  rts

spawn_apple:
  jsr rng_next
  and #$1F
  sta APPLE_X
  jsr rng_next
  and #$1F
  sta APPLE_Y

  lda APPLE_X
  sta CALC_X
  lda APPLE_Y
  sta CALC_Y
  jsr calc_addr
  ldy #0
  lda (PTR),y
  bne spawn_apple       ; occupied (snake) -> retry

  lda #RED
  sta (PTR),y
  rts

game_over:
  ldx #0
  lda #RED

go_fill:
  sta $0200,x
  sta $0300,x
  sta $0400,x
  sta $0500,x
  inx
  bne go_fill
  brk

.segment "VECTORS"
    .word reset         ; NMI
    .word reset         ; RESET
    .word reset         ; IRQ
