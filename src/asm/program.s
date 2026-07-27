.segment "CODE"

reset_handler:
  sei
  cld
  ldx #$FF
  txs

  lda #$00
  sta $00 ; IRQ counter
  sta $01 ; NMI counter

  cli ; clear I flag, Interrupt can happen

infinite_loop:
  jmp infinite_loop


nmi_handler:
  ; save the context in stack
  pha
  txa
  pha
  tya
  pha

  ; increment NMI counter
  inc $01

  ; recover context from the stack
  pla
  tay
  pla
  tax
  pla

  rti


irq_handler:
  ; save the context in stack
  pha
  txa
  pha
  tya
  pha

  ; increment IRQ counter
  inc $00

  ; recover context from the stack
  pla
  tay
  pla
  tax
  pla

  rti

.segment "VECTORS"
  .word nmi_handler   ; NMI   ($FFFA)
  .word reset_handler ; RESET ($FFFC)
  .word irq_handler   ; IRQ   ($FFFE)
