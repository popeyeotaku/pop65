        .org 0
        .dbg "al ${V} .{L}"
        .word $801
        .org $801
bashed  .word bashe1
        .word 10
        .byte $9E,'2061',0
bashe1  .word 0
        jmp start
chrout  = $ffd2
point1  = $fb
msg     .byte 13,13,'HELLO WORLD',13,13,0
start   lda #<msg
        sta point1
        lda #>msg
        sta point1+1
        jsr print
        jmp *
print   ldy #0
print1  lda (point1),y
        beq print2
        jsr chrout
        iny
        bne print1
print2  rts
