//! Pop65: a simple 6502 assembler.

use std::{collections::HashMap, mem};

use asm::Assembler;
pub use source::from_file;
use source::Source;
pub use symbol::Symbol;

/// Assemble code from a source string.
pub fn assemble_str(src: &str, path: &str) -> Result<Vec<u8>, String> {
    let src = source::from_str(src, path);
    let info = assemble(src, false)?;
    Ok(info.bytes)
}

/// The returned info from the assembler.
pub struct AsmInfo {
    pub bytes: Vec<u8>,
    pub symtab: HashMap<String, Box<Symbol>>,
    pub debug_str: String,
    pub listing: Option<String>,
}

impl AsmInfo {
    /// Dump the symbol table to a sorted string.
    pub fn dump_symtab(&self) -> String {
        let mut symstr = String::new();
        let mut symbols = Vec::from_iter(self.symtab.values());
        symbols.sort();
        for symbol in symbols {
            symstr.push_str(&format!("{}\n", symbol));
        }
        symstr
    }
}

/// Assemble a source file.
pub fn assemble(src: Source, listing: bool) -> Result<AsmInfo, String> {
    let mut asm = Box::new(Assembler::new(src, listing));
    asm.pass1()?;
    let bytes = asm.pass2()?;
    let listing = {
        if let Some(lines) = asm.listing.as_ref() {
            let mut s = "LINENO PC   BYTES  LINE\n".to_string();
            for line in lines {
                s.push_str(&format!("{}\n", line.trim()));
            }
            Some(s)
        } else {
            None
        }
    };
    Ok(AsmInfo {
        bytes,
        symtab: mem::take(&mut asm.symtab),
        debug_str: mem::take(&mut asm.debug_str),
        listing,
    })
}

mod action;
mod asm;
mod expr;
mod mac;
mod opcode;
mod parse;
mod pseudo;
mod source;
mod symbol;

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{assemble, assemble_str, from_file, source};

    #[test]
    fn test_asm_str() {
        let src = "\t.org 0\n\tclc\nfoo\tbrk\n\tlda $1234,x\n\tjmp foo\n";
        assert_eq!(
            assemble_str(src, "src"),
            Ok(vec![0x18, 0x00, 0xbd, 0x34, 0x12, 0x4C, 0x01, 0x00])
        );
    }

    #[test]
    fn test_c64_hello() {
        let src = "
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
print2  rts";
        assert_eq!(
            assemble_str(src, "src"),
            Ok(vec![
                1, 8, 11, 8, 10, 0, 158, 50, 48, 54, 49, 0, 0, 0, 76, 32, 8, 13, 13, 72, 69, 76,
                76, 79, 32, 87, 79, 82, 76, 68, 13, 13, 0, 169, 16, 133, 251, 169, 8, 133, 252, 32,
                46, 8, 76, 43, 8, 160, 0, 177, 251, 240, 6, 32, 210, 255, 200, 208, 246, 96
            ])
        )
    }

    #[test]
    fn test_include() {
        let info = assemble(from_file("inc_test.s").unwrap(), false).unwrap();
        assert_eq!(info.bytes, vec![0x36, 0x12, 0x34, 0x12])
    }

    #[test]
    fn test_dbg() {
        let src = "
        .org $8000
        .dbg \"P:{VC000}:{L}\"
foo     .word foo";
        let info = assemble(source::from_str(src, "src"), false).unwrap();
        assert_eq!(&info.bytes, &vec![0x00, 0x80]);
        assert_eq!(info.symtab["foo"].value, Some(0x8000));
        assert_eq!(&info.debug_str, "P:14000:foo\n");
    }

    #[test]
    fn test_bin() {
        let foo: [u8; 4] = [1, 2, 3, 4];
        fs::write("foo.bin", foo).unwrap();
        let src = "
foo     .incbin \"foo.bin\"
        .ds $10-*
bar     ";
        let info = assemble(source::from_str(src, "src"), false).unwrap();
        let mut long_foo: Vec<u8> = vec![0; 0x10];
        long_foo[0] = 1;
        long_foo[1] = 2;
        long_foo[2] = 3;
        long_foo[3] = 4;
        assert_eq!(&info.bytes, &long_foo);
        assert_eq!(info.symtab["foo"].value, Some(0));
        assert_eq!(info.symtab["bar"].value, Some(0x10));
    }

    #[test]
    fn test_label_comments() {
        let src = "
; Foo does a 
; bunch of different things.
foo     .word foo   ; ignored
; this is also ignored
        .word bar
bar     .word foo   ;   
foobar  .word *     ; foobar's comment
                    ; bar's comment is also ignored
                    ; despite also having whitespace after it
";
        let info = assemble(source::from_str(src, "src"), false).unwrap();
        assert_eq!(
            info.symtab["foo"].comment,
            Some("Foo does a\nbunch of different things.\n".to_string())
        );
        assert!(info.symtab["bar"].comment.is_none());
        assert_eq!(
            info.symtab["foobar"].comment,
            Some("foobar's comment".to_string())
        )
    }
}
