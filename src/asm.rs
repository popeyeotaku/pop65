//! Assembler struct stuff.

use std::{collections::HashMap, mem, rc::Rc};

use crate::{
    mac::{end_macro, Macro},
    parse::ParsedLine,
    source::{Line, LineNum, LineSlice, Source, SrcStack},
    symbol::Symbol,
};

/// Indicates how many bytes should be printed on a listing line.
const BYTES_PER_LINE: usize = 3;

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
    pub pc: u16,
    pub cur_line: Option<Rc<Line>>,
    building_comment: Option<String>,
    errcount: u32,
    pub output_flag: bool,
    pub if_stack: Vec<bool>,
    pub listing: Option<Vec<String>>,
    listing_index: Option<HashMap<(String, LineNum), usize>>,
    pub macros: HashMap<String, Rc<Macro>>,
}

/// The initial value of the assembler's program counter.
const DEFAULT_PC: u16 = 0;

impl Assembler {
    pub fn new(src: Source, list_file: bool) -> Self {
        let (listing, listing_index) = {
            if list_file {
                (Some(Vec::new()), Some(HashMap::new()))
            } else {
                (None, None)
            }
        };
        Self {
            macros: HashMap::new(),
            src_stk: Box::new(SrcStack::new(src)),
            symtab: HashMap::new(),
            pc: DEFAULT_PC,
            parsed_lines: Vec::new(),
            pass: Pass::None,
            cur_line: None,
            debug_str: String::new(),
            debug_fmt: None,
            building_comment: None,
            errcount: 0,
            output_flag: true,
            if_stack: Vec::new(),
            listing,
            listing_index,
        }
    }

    /// Run pass1 for a single line.
    fn pass1_line(&mut self, line: Rc<Line>) -> Result<(), String> {
        self.cur_line = Some(line.clone());
        let parsed = self.parse_line(line.clone())?;

        if let Some(listing) = self.listing.as_mut() {
            let index = self.listing_index.as_mut().unwrap();
            if index
                .insert((line.path.clone(), line.line_num), listing.len())
                .is_some()
            {
                panic!("saw the same line from the same file twice")
            }
            listing.push(format!(
                "{:06} {:04X}        {}",
                line.line_num, self.pc, line.text
            ));
        }

        if !*self.if_stack.last().unwrap_or(&true) {
            if let Some(action) = &parsed.action {
                if !action.is_if_affiliated() {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }

        let comment = parsed.filter_comment();
        let is_equ = {
            if let Some(action) = &parsed.action {
                action.is_equ()
            } else {
                false
            }
        };
        if let Some(label_slice) = &parsed.label {
            let comment_label = {
                if let Some(s) = self.building_comment.take() {
                    Some(s)
                } else {
                    comment.map(String::from)
                }
            };
            if !is_equ {
                self.def_label(label_slice.text(), label_slice.clone(), comment_label)?;
            }
        }
        if let Some(action) = &parsed.action {
            if let Some(name) = action.is_macro_def() {
                let mut mac = Macro::new();
                for line in self.src_stk.by_ref() {
                    mac.add_line(line.clone());
                    if end_macro(&line) {
                        break;
                    }
                }
                if let std::collections::hash_map::Entry::Vacant(e) =
                    self.macros.entry(name.clone())
                {
                    e.insert(Rc::new(mac));
                } else {
                    action
                        .line_slice()
                        .err(&format!("macro {} redefined", name))?;
                }
            } else {
                let size = action.pass1(self, parsed.label.clone())?;
                self.pc = self.pc.wrapping_add(size);
            }
        }
        if let Some(c) = comment {
            if parsed.label.is_none() && parsed.action.is_none() {
                let s = self.building_comment.get_or_insert_with(String::new);
                s.push_str(c);
                s.push('\n');
            }
        }

        if comment.is_none() || parsed.label.is_some() || parsed.action.is_some() {
            self.building_comment = None;
        }

        self.parsed_lines.push(parsed);

        Ok(())
    }

    /// Read the entire source, constructing the symbol table.
    pub fn pass1(&mut self) -> Result<(), String> {
        self.pass = Pass::Pass1;
        self.parsed_lines.clear();
        self.symtab.clear();
        self.pc = DEFAULT_PC;
        self.if_stack.clear();

        while let Some(line) = self.src_stk.next() {
            if let Err(msg) = self.pass1_line(line) {
                eprintln!("{}", msg);
                self.errcount += 1;
            }
        }

        if !self.if_stack.is_empty() {
            eprintln!("unmatched if statements");
            self.errcount += 1;
        }

        if self.errcount == 0 {
            Ok(())
        } else {
            Err(format!("{} errors in pass 1", self.errcount))
        }
    }

    /// Handle a single line in pass2.
    fn pass2_line(&mut self, line: &ParsedLine, output: &mut Vec<u8>) -> Result<(), String> {
        let old_pc = self.pc;
        if let Some(action) = &line.action {
            let new_bytes = action.pass2(self)?;
            self.pc = self.pc.wrapping_add(new_bytes.len() as u16);
            if let Some(listing) = self.listing.as_mut() {
                let index = self.listing_index.as_mut().unwrap();
                let i = index[&(line.line.path.clone(), line.line.line_num)];
                let mut l = String::new();
                l.push_str(&format!("{:06} {:04X} ", line.line.line_num, old_pc));
                for i in 0..BYTES_PER_LINE {
                    if let Some(b) = new_bytes.get(i) {
                        l.push_str(&format!("{:02X}", *b));
                    } else {
                        l.push_str("  ");
                    }
                }
                l.push_str(&format!(" {}\n", line.line.text));
                listing[i] = l;
            }
            if self.output_flag {
                output.extend(new_bytes);
            }
        }
        Ok(())
    }

    /// Final assembly.
    pub fn pass2(&mut self) -> Result<Vec<u8>, String> {
        assert!(self.errcount == 0);
        self.pc = DEFAULT_PC;
        self.pass = Pass::Pass2;
        let mut output: Vec<u8> = Vec::with_capacity((u16::MAX as usize) + 1);
        let lines = mem::take(&mut self.parsed_lines);

        for parsed_line in &lines {
            if let Err(msg) = self.pass2_line(parsed_line, &mut output) {
                eprintln!("{}", msg);
                self.errcount += 1;
            }
        }

        if self.errcount == 0 {
            Ok(output)
        } else {
            Err(format!("{} errors in pass 2", self.errcount))
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
                                .push_str(&comment.unwrap_or("").trim_end().replace("\n", " "));
                        }
                        Some('V') => {
                            let mut starting_offset: u32 = 0;
                            let mut wrapped_c = chars.next();
                            let neg_flag = {
                                if wrapped_c == Some('-') {
                                    wrapped_c = chars.next();
                                    true
                                } else {
                                    false
                                }
                            };
                            while let Some(c) = wrapped_c {
                                if c == '}' {
                                    break;
                                } else if let Some(digit) = c.to_digit(16) {
                                    wrapped_c = chars.next();
                                    starting_offset = starting_offset * 16 + digit;
                                } else {
                                    return slice.err("bad debug format string");
                                }
                            }
                            if neg_flag {
                                starting_offset = starting_offset.wrapping_neg();
                            }
                            self.debug_str.push_str(&format!(
                                "{:X}",
                                starting_offset.wrapping_add(value as u32)
                            ));
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
    ///
    /// In the first pass, this also outputs the debug string.
    pub fn def_label(
        &mut self,
        label: &str,
        slice: Rc<LineSlice>,
        comment_label: Option<String>,
    ) -> Result<(), String> {
        let pc = self.pc;
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
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        assemble,
        source::{self, from_str, LineSlice},
    };

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

        let mut asm = Box::new(Assembler::new(src, false));
        asm.pass = Pass::Pass1;

        asm.lookup("foobar", foobar.clone());
        asm.def_label("foo", foo.clone(), None).unwrap();
        asm.pc = asm.pc.wrapping_add(2);
        asm.def_label("bar", bar.clone(), None).unwrap();
        asm.pc = asm.pc.wrapping_add(2);
        asm.def_label("foobar", foobar.clone(), None).unwrap();
        asm.pc = asm.pc.wrapping_add(2);

        assert_eq!(asm.lookup("foo", foo).value, Some(0));
        assert_eq!(asm.lookup("bar", bar).value, Some(2));
        assert_eq!(asm.lookup("foobar", foobar).value, Some(4));
        assert_eq!(asm.pc, 6);
    }

    #[test]
    fn test_nodbg_equ() {
        let src = "
        .dbg '{L}:{V}'
foo     = 1234
bar     .equ foo*2
";
        let result = assemble(source::from_str(src, "src"), false).unwrap();
        assert!(result.debug_str.is_empty());
        assert_eq!(result.symtab["foo"].value, Some(1234));
        assert_eq!(result.symtab["bar"].value, Some(2468));
    }

    #[test]
    fn test_neg_dbg() {
        let src = "
        .org $1234
        .dbg '{L}:{V-1000}'
foo     .word foo";
        let info = assemble(source::from_str(src, src), false).unwrap();
        assert_eq!(info.debug_str.as_str(), "foo:234\n");
    }

    #[test]
    fn test_reset_building_comment() {
        let src = "
        .org $1234
        .dbg ';{C}'
        ;foobar
foo     .word bar
        ;foo
        ;bar
        
bar     .word foo";
        assert_eq!(
            &assemble(from_str(src, "src"), false).unwrap().debug_str,
            ";foobar\n;\n"
        );
    }

    #[test]
    fn test_relop() {
        let src = "
FOO = 2
BAR = 3
L = FOO < BAR
G = FOO > BAR
LE = FOO <= BAR
GE = FOO >= BAR
E = FOO = BAR
NE = FOO >< BAR
NE2 = FOO <> BAR
        ";
        let info = assemble(source::from_str(src, "src"), false).unwrap();
        assert_eq!(info.symtab["FOO"].value, Some(2));
        assert_eq!(info.symtab["BAR"].value, Some(3));
        assert_eq!(info.symtab["L"].value, Some(1));
        assert_eq!(info.symtab["G"].value, Some(0));
        assert_eq!(info.symtab["LE"].value, Some(1));
        assert_eq!(info.symtab["GE"].value, Some(0));
        assert_eq!(info.symtab["E"].value, Some(0));
        assert_eq!(info.symtab["NE"].value, Some(1));
        assert_eq!(info.symtab["NE2"].value, Some(1));
    }

    #[test]
    fn test_listing() {
        let s = "; listing test
.ORG $1234

FOO     .WORD BAR
BAR     .WORD FOO";
        let info = assemble(from_str(s, "{s}"), true).unwrap();
        assert_eq!(
            &info.listing.unwrap(),
            "LINENO PC   BYTES  LINE
000001 0000        ; listing test
000002 0000        .ORG $1234
000003 1234
000004 1234 3612   FOO     .WORD BAR
000005 1236 3412   BAR     .WORD FOO\n"
        );
    }
}
