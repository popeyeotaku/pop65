//! Assembly parsing.

use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use crate::{
    action::Action,
    asm::Assembler,
    source::{Line, LineSlice},
};

pub struct ParsedLine {
    pub label: Option<Box<LineSlice>>,
    pub action: Option<Box<dyn Action>>,
    pub comment: Option<Box<LineSlice>>,
}

/// Allows searching through individual characters in a line.
pub struct LineChars<'a> {
    line: &'a Line,
    chars: Enumerate<Chars<'a>>,
}

impl<'a> LineChars<'a> {
    pub fn new(line: &'a Line) -> Self {
        Self {
            line,
            chars: line.text.chars().enumerate(),
        }
    }
}

impl<'a> Iterator for LineChars<'a> {
    type Item = (char, LineSlice);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((char_index, c)) = self.chars.next() {
            Some((
                c,
                self.line.slice(char_index as u16, (char_index as u16) + 1),
            ))
        } else {
            None
        }
    }
}

impl Assembler {
    /// Parse a single line of input. Return the label (if any), opcode/pseudo-op (if any), and comment (if any).
    pub fn parse_line(&mut self, line: &Line) -> Result<ParsedLine, String> {
        let mut chars = LineChars::new(line).peekable();

        let label = self.parse_label(&mut chars)?.map(Box::new);
        let action = self.parse_action(&mut chars)?;
        let comment = self.parse_comment(&mut chars)?.map(Box::new);
        Ok(ParsedLine {
            label,
            action,
            comment,
        })
    }

    /// Skip leading whitespace.
    pub fn skip_ws(&mut self, chars: &mut Peekable<LineChars>) {
        while let Some((c, _)) = chars.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }
        }
    }

    /// Parse the leading line label, if any.
    fn parse_label(
        &mut self,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Option<LineSlice>, String> {
        Ok(self.parse_name(chars))
    }

    /// Grab a leading Name, if any.
    fn parse_name(&mut self, chars: &mut Peekable<LineChars>) -> Option<LineSlice> {
        if let Some((c, start)) = chars.peek().cloned() {
            if c.is_ascii_alphabetic() {
                chars.next();
                let mut end = start.end_char;
                while let Some((c, new_end)) = chars.peek() {
                    if !c.is_ascii_alphanumeric() {
                        break;
                    }
                    end = new_end.end_char;
                    chars.next();
                }
                Some(start.with_end(end))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse an action, if any.
    fn parse_action(
        &mut self,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Option<Box<dyn Action>>, String> {
        todo!()
    }

    /// Parse the trailing comment, if any.
    fn parse_comment(
        &mut self,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Option<LineSlice>, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        asm::Assembler,
        parse::LineChars,
        source::{self, Line},
    };

    #[test]
    fn test_parse_name() {
        let test = source::from_str("foo\nbar foobar\n\nfoobar\n", "foobar");
        let foo = Line::new("foo", "foobar", 1);
        let bar = Line::new("bar foobar", "foobar", 2);
        let bl = Line::new("", "foobar", 3);
        let foobar = Line::new("foobar", "foobar", 4);
        let mut asm = Assembler::new(test);

        assert_eq!(
            asm.parse_name(&mut LineChars::new(&foo).peekable()),
            Some(foo.slice(0, 3))
        );
        let mut bar_chars = LineChars::new(&bar).peekable();
        assert_eq!(asm.parse_name(&mut bar_chars), Some(bar.slice(0, 3)));
        assert_eq!(asm.parse_name(&mut bar_chars), None);
        assert!(bar_chars.next().is_some());
        assert_eq!(asm.parse_name(&mut bar_chars), Some(bar.slice(4, 4 + 6)));
        assert_eq!(asm.parse_name(&mut LineChars::new(&bl).peekable()), None);
        assert_eq!(
            asm.parse_name(&mut LineChars::new(&foobar).peekable()),
            Some(foobar.slice(0, 6))
        );
    }
}
