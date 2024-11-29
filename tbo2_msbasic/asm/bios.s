.setcpu "65C02"
.debuginfo

.zeropage
READ_PTR:         .res 1
WRITE_PTR:        .res 1

.segment "INPUT_BUFFER"
INPUT_BUFFER:     .res $100

.segment "BIOS"

CHR_DATA        = $5000
CHR_ACK         = $5001

; Check whether there is a key pressed.
; If there are, load the character to A reg,
; and set the carry bit.
; Modifies: A, flags
CHRIN:
                  phx
                  jsr       BUFFER_SIZE
                  beq       @no_keypressed    ; if no, clear the carry bit.
                  jsr       READ_BUFFER             
                  jsr       CHROUT            ; echo char.
                  plx
                  sec                         ; set the carry bit.
                  rts
@no_keypressed:   
                  plx
                  clc                         ; clear the carry bit.
                  rts

; Output a charater from A register to CHR_DATA
; Modifies: flags
CHROUT:
                  pha
                  sta       CHR_DATA
                  lda       #1
                  sta       CHR_ACK
@wait_echo:       lda       CHR_ACK
                  BNE       @wait_echo
                  pla
                  rts

; Modifies: A, flags
INIT_BUFFER:
                  lda       READ_PTR
                  sta       WRITE_PTR
                  rts

; Modifies: X, flags
WRITE_BUFFER:
                  ldx       WRITE_PTR
                  sta       INPUT_BUFFER,X
                  inc       WRITE_PTR
                  rts

; Modifies: A, X, flags
READ_BUFFER:
                  ldx       READ_PTR
                  lda       INPUT_BUFFER,X
                  inc       READ_PTR
                  rts

; No. of unread bytes in the buffer.
; Modifies: A, flags
BUFFER_SIZE:
                  lda       WRITE_PTR
                  sec 
                  sbc       READ_PTR
                  rts

IRQ_HANDLER:
                  pha
                  phx
                  lda       CHR_DATA
                  jsr       WRITE_BUFFER
                  plx                
                  pla
                  rti

.include "wozmon.s"

.segment "RESETVEC"
                  .word   $0F00               ; NMI vector
                  .word   RESET               ; RESET vector
                  .word   IRQ_HANDLER         ; IRQ vector
