//! Assembly parsing.

use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

use crate::{
    action::{Action, PseudoOp},
    asm::Assembler,
    expr::ExprNode,
    opcode::{find_op, AMode, OpCode},
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

        self.skip_ws(&mut chars);
        if let Some((_, pos)) = chars.next() {
            pos.err("unexpected characters past end of line")
        } else {
            Ok(ParsedLine {
                label,
                action,
                comment,
            })
        }
    }

    /// Skip leading whitespace.
    pub fn skip_ws(&mut self, chars: &mut Peekable<LineChars>) {
        while let Some((c, _)) = chars.peek() {
            if !c.is_ascii_whitespace() {
                break;
            } else {
                chars.next();
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
        self.skip_ws(chars);

        if let Some((c, start)) = chars.peek().cloned() {
            if c == '.' {
                chars.next();
                return self.parse_pseudo(start, chars).map(Some);
            }
        }

        if let Some(name) = self.parse_name(chars) {
            self.parse_opcode(name, chars).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Return a flag if we're at end-of-line.
    /// (skips whitespace and also exits on a comment).
    fn at_eol(&mut self, chars: &mut Peekable<LineChars>) -> bool {
        self.skip_ws(chars);
        if let Some((c, _)) = chars.next() {
            c == ';'
        } else {
            true
        }
    }

    /// Parse a psuedo-op.
    fn parse_pseudo(
        &mut self,
        start: LineSlice,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Box<dyn Action>, String> {
        if let Some(name) = self.parse_name(chars) {
            let name = start.join(&name);
            if self.at_eol(chars) {
                Ok(Box::new(PseudoOp::new(name, Vec::new())))
            } else {
                let mut args = vec![self.parse_expr(chars)?];
                while let Some((c, _)) = chars.next() {
                    if c == ',' {
                        args.push(self.parse_expr(chars)?);
                    } else {
                        break;
                    }
                }
                Ok(Box::new(PseudoOp::new(name, args)))
            }
        } else {
            start.err("missing pseudo-op name")
        }
    }

    /// Parse an opcode.
    fn parse_opcode(
        &mut self,
        opcode: LineSlice,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Box<dyn Action>, String> {
        let op_name = opcode.text().to_ascii_lowercase();
        if let Some(op) = find_op(&op_name) {
            let (amode, expr) = self.parse_operand(chars)?;
            Ok(Box::new(OpCode::new(op, opcode, amode, expr)))
        } else {
            opcode.err(&format!("unknown opcode '{}'", opcode.text()))
        }
    }

    /// Parse an opcode operand.
    fn parse_operand(
        &mut self,
        chars: &mut Peekable<LineChars>,
    ) -> Result<(AMode, Option<Box<ExprNode>>), String> {
        self.skip_ws(chars);
        let head = {
            if let Some((_, slice)) = chars.peek() {
                slice.clone()
            } else {
                self.cur_line.as_ref().unwrap().slice(0, 0)
            }
        };
        if let Some((c, _)) = chars.peek() {
            match c {
                '#' => {
                    chars.next();
                    return Ok((AMode::Imm, Some(self.parse_expr(chars)?)));
                }
                '(' => {
                    chars.next();
                    let expr = self.parse_expr(chars)?;
                    self.skip_ws(chars);
                    if let Some((c, _)) = chars.peek() {
                        match c {
                            ')' => {
                                chars.next();
                                return Ok((AMode::Ind, Some(expr)));
                            }
                            ',' => {
                                chars.next();
                                self.skip_ws(chars);
                                if let Some((c, _)) = chars.peek() {
                                    let c = *c;
                                    chars.next();
                                    self.skip_ws(chars);
                                    if let Some((rparen, _)) = chars.peek() {
                                        if *rparen == ')' {
                                            chars.next();
                                            match c {
                                                'x' | 'X' => return Ok((AMode::IndX, Some(expr))),
                                                'y' | 'Y' => return Ok((AMode::IndY, Some(expr))),
                                                _ => (),
                                            }
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => {
                    if self.at_eol(chars) {
                        return Ok((AMode::Imp, None));
                    } else {
                        let expr = self.parse_expr(chars)?;
                        self.skip_ws(chars);
                        if let Some((c, _)) = chars.peek() {
                            if *c == ',' {
                                chars.next();
                                self.skip_ws(chars);
                                if let Some((c, _)) = chars.peek() {
                                    let c = *c;
                                    chars.next();
                                    match c {
                                        'x' | 'X' => return Ok((AMode::AbsX, Some(expr))),
                                        'y' | 'Y' => return Ok((AMode::AbsY, Some(expr))),
                                        _ => (),
                                    }
                                }
                            } else {
                                return Ok((AMode::Abs, Some(expr)));
                            }
                        }
                    }
                }
            }
        } else if self.at_eol(chars) {
            return Ok((AMode::Imp, None));
        }
        head.err("bad operand")
    }

    /// Parse the trailing comment, if any.
    fn parse_comment(
        &mut self,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Option<LineSlice>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            if c == ';' {
                if let Some((_, end)) = chars.last() {
                    Ok(Some(start.join(&end)))
                } else {
                    Ok(Some(start))
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

mod expr;

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
