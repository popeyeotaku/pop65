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
    let s = skip_label(text);
    s.trim().starts_with(".endm")
}

fn skip_label(l: &Line) -> String {
    let s = l.text.to_lowercase().trim().to_string();
    let (first, rest) = split_at_first_blank(&s);
    if first.map(is_label).unwrap_or_default() {
        rest.to_string()
    } else {
        s
    }
}

fn is_label(s: &str) -> bool {
    s.chars()
        .next()
        .map(|c| c.is_alphabetic())
        .unwrap_or_default()
}

fn split_at_first_blank(s: &str) -> (Option<&str>, &str) {
    let mut quote: Option<char> = None;
    for (i, c) in s.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else {
            match c {
                '\'' | '"' => quote = Some(c),
                ';' => {
                    let (l, r) = s.split_at(i);
                    return (Some(l.trim()), r.trim());
                }
                _ => {
                    if c.is_whitespace() {
                        let (l, r) = s.split_at(i);
                        return (Some(l.trim()), r.trim());
                    }
                }
            }
        }
    }
    (None, s.trim())
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
        let mut s: String = line.text.clone();
        for (i, arg) in self.args.iter().enumerate() {
            s = s.replace(&format!(r"\{}", i + 1), arg);
        }
        Line::new(
            &s,
            &self.referenced_line.path,
            self.referenced_line.line_num,
        )
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
        let _ = label;
        assembler.src_stk.push(Box::new(self.clone().source()));
        Ok(0)
    }

    fn pass2(&self, assembler: &mut crate::asm::Assembler) -> Result<Vec<u8>, String> {
        let _ = assembler;
        Ok(Vec::new())
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

#[cfg(test)]
mod tests {
    use crate::{assemble, assemble_str, source::from_str};

    #[test]
    fn test_macro() {
        let msrc = r"
        .mac inw
            inc \1
            .if \1 < $100
                bne *+4
            .else
                bne *+5
            .endif
            inc \1+1
        .endm
        inw $02
        inw $1234";
        let rsrc = "
        foo=$02
        bar=$1234
        inc foo
        bne l1
        inc foo+1
    l1: inc bar
        bne l2
        inc bar+1
    l2:";
        assert_eq!(
            assemble_str(msrc, "msrc").unwrap(),
            assemble_str(rsrc, "rsrc").unwrap()
        );
    }

    #[test]
    fn test_list_macro() {
        let src = r"
        .mac inw
            inc \1
            .if \1 < $100
                bne *+4
            .else
                bne *+5
            .endif
            inc \1+1
        .endm
        inw $1234
        inw $5678
        ";
        let should_be = "LINENO PC   BYTES  LINE
000001 0000
000002 0000                .mac inw
000011 0000                inw $1234
000011 0000 EE3412             inc $1234
000011 0003                    .if $1234 < $100
000011 0003                        bne *+4
000011 0003                    .else
000011 0003 D003                   bne *+5
000011 0005                    .endif
000011 0005 EE3512             inc $1234+1
000011 0008                .endm
000012 0008                inw $5678
000012 0008 EE7856             inc $5678
000012 000B                    .if $5678 < $100
000012 000B                        bne *+4
000012 000B                    .else
000012 000B D003                   bne *+5
000012 000D                    .endif
000012 000D EE7956             inc $5678+1
000012 0010                .endm
000013 0010
";
        let is_be = assemble(from_str(src, "{src}"), true)
            .unwrap()
            .listing
            .unwrap();
        assert_eq!(should_be, &is_be);
    }
}
