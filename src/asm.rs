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
    cur_line: Option<Line>,
}

impl Assembler {
    pub fn new(src: Source) -> Self {
        Self {
            pass1_src: Box::new(SrcStack::new(src)),
            symtab: HashMap::new(),
            program_counter: None,
            pass2_src: Vec::new(),
            pass: Pass::None,
            cur_line: None,
        }
    }

    /// Read the entire source, constructing the symbol table.
    pub fn pass1(&mut self) -> Result<(), String> {
        self.pass = Pass::Pass1;
        self.pass2_src.clear();
        self.symtab.clear();
        self.program_counter = None;

        while let Some(line) = self.pass1_src.next() {
            self.cur_line = Some(line.clone());
            self.pass2_src.push(line.clone());
            let (label, action, _) = self.parse_line(&line)?;
            if let Some(label_slice) = &label {
                self.def_label(label_slice.text(), label_slice)?;
            }
            if let Some(action) = action {
                let size = action.pass1(self, &label)?;
                self.pc_add(size)?;
            }
        }
        Ok(())
    }

    /// Final assembly.
    pub fn pass2(&mut self) -> Result<Vec<u8>, String> {
        self.program_counter = None;
        self.pass = Pass::Pass2;

        todo!()
    }

    /// Add a value to the current PC.
    pub fn pc_add(&mut self, offset: u16) -> Result<(), String> {
        if let Some(new_pc) = self.pc()?.checked_add(offset) {
            *self.pc()? = new_pc;
            Ok(())
        } else {
            self.cur_line.as_ref().unwrap().err("PC overflow")
        }
    }

    /// Define a new label at the current PC, complaining if it was redefined.
    pub fn def_label(&mut self, label: &str, slice: &LineSlice) -> Result<(), String> {
        let pc = *self.pc()?;
        self.def_symbol(label, slice, pc)
    }

    /// Look-up the symbol in the symbol table, creating it as undefined if it didn't exist.
    pub fn lookup(&mut self, name: &str, ref_slice: &LineSlice) -> &mut Box<Symbol> {
        if !self.symtab.contains_key(name) {
            let already_there = self
                .symtab
                .insert(name.to_string(), Symbol::new(name, ref_slice));
            debug_assert!(already_there.is_none());
        }
        self.symtab.get_mut(name).unwrap()
    }

    /// Define a new symbol, complaining if it was redefined.
    pub fn def_symbol(&mut self, name: &str, slice: &LineSlice, value: u16) -> Result<(), String> {
        match self.pass {
            Pass::None => panic!("symbol def outside of pass"),
            Pass::Pass1 => {
                let sym = self.lookup(name, slice);
                sym.define(value, slice)
            }
            Pass::Pass2 => {
                if let Some(definition) = self.lookup(name, slice).value {
                    if definition == value {
                        Ok(())
                    } else {
                        slice.err(&format!(
                            "'{}' is {:X} in pass1, {:X} in pass2",
                            name, definition, value
                        ))
                    }
                } else {
                    slice.err(&format!(
                        "'{}' undefined in pass1, defined as {:X} in pass2",
                        name, value
                    ))
                }
            }
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

#[cfg(test)]
mod tests {
    use crate::source::{self, LineSlice};

    use super::{Assembler, Pass};

    #[test]
    fn test_symdef() {
        let mut src = Box::new(source::from_str("foo bar foobar", "foobar").peekable());
        let line = src.peek().unwrap();
        let foo = line.slice(0, 3);
        let bar = line.slice(4, 7);
        let foobar = line.slice(8, 8 + 6);
        assert_eq!(foo.text(), "foo");
        assert_eq!(bar.text(), "bar");
        assert_eq!(foobar.text(), "foobar");

        let mut asm = Box::new(Assembler::new(src));
        asm.pass = Pass::Pass1;
        asm.program_counter = Some(0);

        asm.lookup("foobar", &foobar);
        asm.def_label("foo", &foo).unwrap();
        asm.pc_add(2).unwrap();
        asm.def_label("bar", &bar).unwrap();
        asm.pc_add(2).unwrap();
        asm.def_label("foobar", &foobar).unwrap();
        asm.pc_add(2).unwrap();

        assert_eq!(asm.lookup("foo", &foo).value, Some(0));
        assert_eq!(asm.lookup("bar", &bar).value, Some(2));
        assert_eq!(asm.lookup("foobar", &foobar).value, Some(4));
        assert_eq!(*asm.pc().unwrap(), 6);
    }
}
