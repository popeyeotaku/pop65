//! Assembler struct stuff.

use std::{collections::HashMap, mem, rc::Rc};

use crate::{
    parse::ParsedLine,
    source::{Line, LineSlice, Source, SrcStack},
    symbol::Symbol,
};

/// Represents the current assembly pass.
#[derive(PartialEq)]
pub enum Pass {
    None,
    Pass1,
    Pass2,
}

pub struct Assembler {
    pub src_stk: Box<SrcStack>,
    parsed_lines: Vec<ParsedLine>,
    pub debug_str: String,
    pub debug_fmt: Option<String>,
    pub pass: Pass,
    pub symtab: HashMap<String, Box<Symbol>>,
    pub program_counter: Option<u16>,
    pub cur_line: Option<Rc<Line>>,
    building_comment: Option<String>,
}

impl Assembler {
    pub fn new(src: Source) -> Self {
        Self {
            src_stk: Box::new(SrcStack::new(src)),
            symtab: HashMap::new(),
            program_counter: None,
            parsed_lines: Vec::new(),
            pass: Pass::None,
            cur_line: None,
            debug_str: String::new(),
            debug_fmt: None,
            building_comment: None,
        }
    }

    /// Read the entire source, constructing the symbol table.
    pub fn pass1(&mut self) -> Result<(), String> {
        self.pass = Pass::Pass1;
        self.parsed_lines.clear();
        self.symtab.clear();
        self.program_counter = None;

        while let Some(line) = self.src_stk.next() {
            self.cur_line = Some(line.clone());
            let parsed = self.parse_line(line.clone())?;
            let comment = parsed.filter_comment();
            if let Some(label_slice) = &parsed.label {
                let comment_label = {
                    if let Some(s) = self.building_comment.take() {
                        Some(s)
                    } else {
                        comment.map(String::from)
                    }
                };
                self.def_label(label_slice.text(), label_slice.clone(), comment_label)?;
            }
            if let Some(action) = &parsed.action {
                let size = action.pass1(self, parsed.label.clone())?;
                self.pc_add(size)?;
            }
            if let Some(c) = comment {
                if parsed.label.is_none() && parsed.action.is_none() {
                    let s = self.building_comment.get_or_insert_with(String::new);
                    s.push_str(c);
                    s.push('\n');
                }
            }
            if parsed.label.is_some() || parsed.action.is_some() {
                self.building_comment = None;
            }
            self.parsed_lines.push(parsed);
        }
        Ok(())
    }

    /// Final assembly.
    pub fn pass2(&mut self) -> Result<Vec<u8>, String> {
        self.program_counter = None;
        self.pass = Pass::Pass2;
        let mut output: Vec<u8> = Vec::with_capacity((u16::MAX as usize) + 1);
        let lines = mem::take(&mut self.parsed_lines);

        for parsed_line in &lines {
            if let Some(action) = &parsed_line.action {
                let new_bytes = action.pass2(self)?;
                self.pc_add(new_bytes.len() as u16)?;
                output.extend(new_bytes);
            }
        }

        Ok(output)
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

    /// Output a debug info string.
    fn debug_label(
        &mut self,
        label: &str,
        slice: Rc<LineSlice>,
        value: u16,
        comment: Option<&str>,
    ) -> Result<(), String> {
        if let Some(f) = &self.debug_fmt {
            let mut chars = f.chars();
            while let Some(c) = chars.next() {
                if c == '{' {
                    match chars.next() {
                        Some('C') => {
                            if chars.next() != Some('}') {
                                return slice.err("bad debug format string");
                            }
                            self.debug_str
                                .push_str(&comment.unwrap_or("").replace("\n", " "));
                        }
                        Some('V') => {
                            let mut starting_offset: u32 = 0;
                            for c in chars.by_ref() {
                                if c == '}' {
                                    break;
                                } else if let Some(digit) = c.to_digit(16) {
                                    starting_offset = starting_offset * 16 + digit;
                                } else {
                                    return slice.err("bad debug format string");
                                }
                            }
                            self.debug_str
                                .push_str(&format!("{:X}", starting_offset + (value as u32)));
                        }
                        Some('L') => {
                            if chars.next() != Some('}') {
                                return slice.err("bad debug format string");
                            }
                            self.debug_str.push_str(label);
                        }
                        _ => {
                            return slice.err("bad dbg format string");
                        }
                    }
                } else {
                    self.debug_str.push(c);
                }
            }
            self.debug_str.push('\n');
            Ok(())
        } else {
            panic!()
        }
    }

    /// Define a new label at the current PC, complaining if it was redefined.
    pub fn def_label(
        &mut self,
        label: &str,
        slice: Rc<LineSlice>,
        comment_label: Option<String>,
    ) -> Result<(), String> {
        let pc = *self.pc()?;
        if self.pass == Pass::Pass1 && self.debug_fmt.is_some() {
            self.debug_label(label, slice.clone(), pc, comment_label.as_deref())?
        }
        self.def_symbol(label, slice, pc)?;
        if let Some(comment) = comment_label {
            let sym = self.symtab.get_mut(label).unwrap();
            sym.comment = Some(comment);
        }
        Ok(())
    }

    /// Look-up the symbol in the symbol table, creating it as undefined if it didn't exist.
    pub fn lookup(&mut self, name: &str, ref_slice: Rc<LineSlice>) -> &mut Box<Symbol> {
        if !self.symtab.contains_key(name) {
            let already_there = self
                .symtab
                .insert(name.to_string(), Symbol::new(name, ref_slice.clone()));
            debug_assert!(already_there.is_none());
        }
        let sym = self.symtab.get_mut(name).unwrap();
        sym.add_ref(ref_slice);
        sym
    }

    /// Define a new symbol, complaining if it was redefined.
    pub fn def_symbol(
        &mut self,
        name: &str,
        slice: Rc<LineSlice>,
        value: u16,
    ) -> Result<(), String> {
        match self.pass {
            Pass::None => panic!("symbol def outside of pass"),
            Pass::Pass1 => {
                let sym = self.lookup(name, slice.clone());
                sym.define(value, slice)
            }
            Pass::Pass2 => {
                if let Some(definition) = self.lookup(name, slice.clone()).value {
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
    use std::rc::Rc;

    use crate::source::{self, LineSlice};

    use super::{Assembler, Pass};

    #[test]
    fn test_symdef() {
        let mut src = Box::new(source::from_str("foo bar foobar", "foobar").peekable());
        let line = src.peek().unwrap();
        let foo = Rc::new(LineSlice::new(line.clone(), 0, 3));
        let bar = Rc::new(LineSlice::new(line.clone(), 4, 7));
        let foobar = Rc::new(LineSlice::new(line.clone(), 8, 8 + 6));
        assert_eq!(foo.text(), "foo");
        assert_eq!(bar.text(), "bar");
        assert_eq!(foobar.text(), "foobar");

        let mut asm = Box::new(Assembler::new(src));
        asm.pass = Pass::Pass1;
        asm.program_counter = Some(0);

        asm.lookup("foobar", foobar.clone());
        asm.def_label("foo", foo.clone(), None).unwrap();
        asm.pc_add(2).unwrap();
        asm.def_label("bar", bar.clone(), None).unwrap();
        asm.pc_add(2).unwrap();
        asm.def_label("foobar", foobar.clone(), None).unwrap();
        asm.pc_add(2).unwrap();

        assert_eq!(asm.lookup("foo", foo).value, Some(0));
        assert_eq!(asm.lookup("bar", bar).value, Some(2));
        assert_eq!(asm.lookup("foobar", foobar).value, Some(4));
        assert_eq!(*asm.pc().unwrap(), 6);
    }
}
