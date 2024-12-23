//! Macro support.

use std::rc::Rc;

use better_peekable::BPeekable;

use crate::{
    action::Action,
    asm::Assembler,
    parse::LineChars,
    source::{Line, LineSlice},
};

pub struct Macro {
    replacement_lines: Vec<Rc<Line>>,
}

impl Macro {
    pub fn new() -> Self {
        Self {
            replacement_lines: Vec::new(),
        }
    }

    /// Add another line to the replacement text.
    pub fn add_line(&mut self, line: Rc<Line>) {
        self.replacement_lines.push(line);
    }
}

/// Return a flag for if we're at the end of a macro.
pub fn end_macro(text: &Line) -> bool {
    todo!()
}

/// A macro usage.
#[derive(Clone)]
pub struct MacUsage {
    mac: Rc<Macro>,
    args: Vec<String>,
    referenced_line: Rc<Line>,
}

impl MacUsage {
    pub fn new(mac: Rc<Macro>, args: Vec<String>, referenced_line: Rc<Line>) -> Self {
        Self {
            mac,
            args,
            referenced_line,
        }
    }

    /// Insert any macro argument replacements.
    pub fn replace_args(&self, line: Rc<Line>) -> Line {
        todo!()
    }

    /// Get a macro source.
    pub fn source(self) -> MacSource {
        MacSource { usage: self, i: 0 }
    }
}

/// An input source for a macro usage.
pub struct MacSource {
    usage: MacUsage,
    i: usize,
}

impl Iterator for MacSource {
    type Item = Rc<Line>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.usage.mac.replacement_lines.get(self.i) {
            self.i += 1;
            let line = self.usage.replace_args(line.clone());
            Some(Rc::new(line))
        } else {
            None
        }
    }
}

impl Action for MacUsage {
    fn pass1(
        &self,
        assembler: &mut crate::asm::Assembler,
        label: Option<Rc<crate::source::LineSlice>>,
    ) -> Result<u16, String> {
        assembler.src_stk.push(Box::new(self.clone().source()));
        if let Some(label) = label {
            assembler.def_label(label.text(), label.clone(), None)?;
        }
        Ok(0)
    }

    fn pass2(&self, assembler: &mut crate::asm::Assembler) -> Result<Vec<u8>, String> {
        let _ = assembler;
        panic!("shouldn't ever get a macro in pass 2")
    }

    fn line_slice(&self) -> Rc<crate::source::LineSlice> {
        Rc::new(LineSlice::new(
            self.referenced_line.clone(),
            0,
            self.referenced_line.text.chars().count() as u16,
        ))
    }
}

impl Assembler {
    fn parse_macro_arg(&mut self, chars: &mut BPeekable<LineChars>) -> String {
        let mut s = String::new();
        while !self.at_eol(chars) {
            let (c, _) = chars.peek().unwrap();
            if *c == ',' {
                break;
            } else {
                s.push(*c);
                chars.next().unwrap();
            }
        }
        s.trim().to_string()
    }

    pub fn parse_macro(
        &mut self,
        mac: Rc<Macro>,
        chars: &mut BPeekable<LineChars>,
        line: Rc<Line>,
    ) -> Result<Box<dyn Action>, String> {
        let mut args: Vec<String> = Vec::new();
        if !self.at_eol(chars) {
            args.push(self.parse_macro_arg(chars));
            while !self.at_eol(chars) {
                let (c, _) = chars.peek().unwrap();
                if *c != ',' {
                    break;
                }
            }
        }
        Ok(Box::new(MacUsage::new(mac, args, line)))
    }
}
