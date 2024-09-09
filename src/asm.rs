//! Assembler struct stuff.

use std::collections::HashMap;

use crate::{
    source::{Line, Source, SrcStack},
    symbol::Symbol,
};

pub struct Assembler {
    pass1_src: Box<SrcStack>,
    pass2_src: Vec<Line>,
    symtab: HashMap<String, Box<Symbol>>,
    program_counter: Option<u16>,
}

impl Assembler {
    pub fn new(src: Source) -> Self {
        Self {
            pass1_src: Box::new(SrcStack::new(src)),
            symtab: HashMap::new(),
            program_counter: None,
            pass2_src: Vec::new(),
        }
    }

    /// Read the entire source, constructing the symbol table.
    pub fn pass1(&mut self) {
        self.pass2_src.clear();
        self.symtab.clear();
        self.program_counter = None;
    }

    /// Final assembly.
    pub fn pass2(&mut self) -> Vec<u8> {
        todo!();
    }

    /// Return the current program counter, or an error if it hasn't been set.
    pub fn pc(&mut self) -> Result<&mut u16, String> {
        if let Some(pc) = self.program_counter.as_mut() {
            Ok(pc)
        } else {
            Err("program counter was never set".to_string())
        }
    }
}
