; this is a test of the include system
        .org $1234
foo     .word bar
        .inc "inc_2.s"
