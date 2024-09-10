        .org 0
        .dbg "al ${V} .{L}"
; C64 PRG starting address
        .word $801
; BASIC programs load-in here
        .org $801
; C64 BASIC program to start main program.
; (bottom of program)
bashed  .word bashe1
        .word 10
        .byte $9E,'2061',0
bashe1  .word 0
        jmp start                               ; put your init code at `start`
chrout  = $ffd2                                 ; C64 putchar
point1  = $fb                                   ; zero-page pointer
msg     .byte 13,13,'HELLO WORLD',13,13,0       ; 'hello' message string
start   lda #<msg                               ; print the message and iloop
        sta point1
        lda #>msg
        sta point1+1
        jsr print
        jmp *
print   ldy #0                                  ; print the string at (point1)
print1  lda (point1),y
        beq print2
        jsr chrout
        iny
        bne print1
print2  rts
end                                             ; (top of program)
