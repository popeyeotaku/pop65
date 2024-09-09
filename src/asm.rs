//! Assembler struct stuff.

use std::collections::HashMap;

use crate::{
    source::{Line, LineSlice, Source, SrcStack},
    symbol::Symbol,
};

/// Represents the current assembly pass.
pub enum Pass {
    None,
    Pass1,
    Pass2,
}

pub struct Assembler {
    pass1_src: Box<SrcStack>,
    pass2_src: Vec<Line>,
    pass: Pass,
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
            pass: Pass::None,
        }
    }

    /// Read the entire source, constructing the symbol table.
    pub fn pass1(&mut self) -> Result<(), String> {
        self.pass = Pass::Pass1;
        self.pass2_src.clear();
        self.symtab.clear();
        self.program_counter = None;

        while let Some(line) = self.pass1_src.next() {
            self.pass2_src.push(line.clone());
            let (label, action, _) = self.parse_line(&line)?;
            if let Some(label_slice) = &label {
                self.def_label(label_slice.text(), label_slice)?;
            }
            if let Some(action) = action {
                let size = action.pass1(self, &label)?;
                *self.pc()? = self.pc()?.wrapping_add(size);
            }
        }
        Ok(())
    }

    /// Final assembly.
    pub fn pass2(&mut self) -> Vec<u8> {
        self.program_counter = None;
        self.pass = Pass::Pass2;

        todo!()
    }

    /// Define a new label at the current PC, complaining if it was redefined.
    pub fn def_label(&mut self, label: &str, slice: &LineSlice) -> Result<(), String> {
        let pc = *self.pc()?;
        self.def_symbol(label, slice, pc)
    }

    /// Define a new symbol, complaining if it was redefined.
    pub fn def_symbol(&mut self, label: &str, slice: &LineSlice, value: u16) -> Result<(), String> {
        match self.pass {
            Pass::None => panic!("symbol def outside of pass"),
            Pass::Pass1 => todo!(),
            Pass::Pass2 => todo!(),
        }
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
