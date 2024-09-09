//! Assembler struct stuff.

use std::collections::HashMap;

use crate::{
    source::{Source, SrcStack},
    symbol::Symbol,
};

pub struct Assembler {
    src: Box<SrcStack>,
    symtab: HashMap<String, Box<Symbol>>,
    program_counter: Option<u16>,
}

impl Assembler {
    pub fn new(src: Source) -> Self {
        Self {
            src: Box::new(SrcStack::new(src)),
            symtab: HashMap::new(),
            program_counter: None,
        }
    }

    /// Read the entire source, constructing the symbol table.
    pub fn pass1(&mut self) {
        todo!()
    }

    /// Final assembly.
    pub fn pass2(&mut self) -> Vec<u8> {
        todo!()
    }
}
