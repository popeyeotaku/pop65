# Pop65

*Pop65* is a simple, bare-bones 6502 assembler. I wrote it to use myself, since most 6502 assemblers on modern platforms add many features; *Pop65* is more like the old 8-bit ones. I also wanted to be able to write a LSP server for it — this has not yet been developed, but should be doable with minimum modifications, including the easy ability to check the value of a symbol and go to its definition.

*Pop65* outputs bytes as they're encountered, ignoring the `.org` commands except to set the current internal address. For example, if you wanted to develop an NES game, you would `.org 0` and output the header, then `.org $8000` and output the first bank, `.org $8000` and output the second bank, etc. You can fill the empty space at the end of a bank by going `.ds *-$C000`, for example.

## Line Format

Lines are formatted as:

`[{label}[:]] [{operation}] [;{comment}]`

Whitespace is ignored outside of strings.

## Expressions

All expressions are in unsigned 16-bits, with overflow ignored ($FFFF+1=0).

The operators are, in order of precedence (highest to lowest):

1. `<`/`>`: unary. Get the low/high byte of the following expression.
2. `<`/`>`/`<=`/`>=`/`=`/`<>`/`><`: binary, relational. Takes its two arguments and compares less than/greater/less than or equal/greater or equal, equal to, or two forms of not equal.
3. `+`/`-`: binary. Add or subtract.
4. `*`/`/`/`%`: binary. Multiply, divide, modulo.
5. `-`: unary. Negate (flip all bits and add 1).

You can change grouping with parenthesis.

You can specify the base of a numeric constant like this:

* `$`: hexadecimal (0-9,A-F), case is ignored.
* `%`: binary (0-1).
* `@`: ocatl (0-7).

A string can be enclosed in either `'...'` or `"..."`, as long as the right quote matches the left quote. At present, no string escapes are allowed. A one character string may be employed anywhere a constant integer might; for instance, `'3'` evaluates to `$33`, and `"9"` evaluates to `$39`.

The `*` symbol evaluates to the present *Program Counter*.

## Pseudo-Ops

All pseudo-ops start with a `.` character. Case is ignored.

* `.if {expr}`: If the expression evaluates to zero, everything up until the matching `.endif` is skipped, and not assembled. The expression must be evaluated in the first pass (no forward references).
* `.else`: the sense of the closest matching active `.if` is flipped; `.if 0 foo .else bar .endif` would assemble `bar`.
* `.endif`: ends the closest matching active `.if`.
* `.assert {expr}`: if the expression evaluates to zero, the assembler will issue an assertion error. The expression is only evaluated in the second pass, so forward references are allowed.
* `.dbg {string}`/`.dbg`: the current debug format string is set (see below). Without a string (in the second form), it disabled debug output until set again.
* `.ds {expr1} [, {expr2}]`: places `expr1` bytes in the output. If `expr2` is provided, it is evaluated and its value is used;  otherwise, 0 is used instead. For example, `.ds 2` outputs `0,0`, while `.ds 3,4` outputs `4,4,4`.
* `.bin {string}`/`.incbin {string}`: the file with the `string` name is loaded, and its raw bytes placed into the output.
* `.inc {string}`/`.lib {string}`/`.fil {string}`: the file with the `string` name is treated as a new assembly file and included here.
* `{label} = {expr}`/`{label} .equ {expr}`: assign the label a specific value. The expression must be evaluated in the first pass (no forward references). Labels created in this way are *not* sent to the debug file, but *are* sent to the symbol table file.
* `.org {expr}`: set the *Program Counter* to the value; the expression must be evaluated in the first pass (no forward references).
* `.byte {expr} , {expr} ...`: evaluate each expression and place it as a single byte in the output file.
* `.word {expr} , {expr} ...`: evaluate each expression and place it as a 16-bit little endian word in the output file.
* `.off`: disable output of any bytes; useful for generating RAM labels via `.ds` pseudo-ops.
* `.on`: enable output of any bytes; see `.off`.

Filename strings can use Windows or Unix style path seperators (`/` or `\\`) interchangably.

## Debug File

*Pop65* can be set to output to a "debug file." At any time, the `.dbg` pseudo-op can be used to set a debug format string, or to disable debug output (debugging starts disabled).

The string is output directly, with special escapes contained in braces. Here are the escapes:

* `{L}`: outputs the label name.
* `{C}`: outputs the surrounding comments for the label, with newlines replaced by spaces.
* `{V}`: output the 16-bit value of the label in hexadecimal, with leading 0's stripped out.
* `{V(signed hex number)}`: (for instance, `{V3FF}`). This adds a *signed, 32-bit* hexadecimal number to the unsigned, 16-bit value of the label, and outputs that in hexadecimal here.

For instance, code:

```{.6502}
.org $8000
.dbg "P:{V-8000}:{L}:{C}"
foo .word bar   ; description of foo

; description of...
; bar!
bar .word foo
```

Would output to the debug file:

```{.mlb}
P:0:foo:description of foo
P:2:bar:description of... bar!
```

Which is the proper debug format for the *Mesen* emulator.
