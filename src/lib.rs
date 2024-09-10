//! Pop65: a simple 6502 assembler.

use asm::Assembler;
use source::Source;

/// Assemble code from a source string.
pub fn assemble_str(src: &str, path: &str) -> Result<Vec<u8>, String> {
    let src = source::from_str(src, path);
    assemble(src)
}

/// Assemble a source file.
pub fn assemble(src: Source) -> Result<Vec<u8>, String> {
    let mut asm = Box::new(Assembler::new(src));
    asm.pass1()?;
    asm.pass2()
}

mod action;
mod asm;
mod expr;
mod opcode;
mod parse;
mod pseudo;
mod source;
mod symbol;

#[cfg(test)]
mod tests {
    use crate::assemble_str;

    #[test]
    fn test_asm_str() {
        let src = "\t.org 0\n\tclc\nfoo\tbrk\n\tlda $1234,x\n\tjmp foo\n";
        assert_eq!(
            assemble_str(src, "src"),
            Ok(vec![0x18, 0x00, 0xbd, 0x34, 0x12, 0x4C, 0x01, 0x00])
        );
    }
}
