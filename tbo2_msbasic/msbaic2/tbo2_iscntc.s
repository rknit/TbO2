ISCNTC:
      jsr     CHRIN
      bcc     @not_cntc
      cmp     #$3
      beq     @is_cntc
@not_cntc:
      rts
@is_cntc:
