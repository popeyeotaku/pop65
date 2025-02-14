//! Assembly parsing.

use std::{iter::Enumerate, rc::Rc, str::Chars};

use better_peekable::{BPeekable, BetterPeekable};

use crate::{
    action::Action,
    asm::Assembler,
    expr::ExprNode,
    opcode::{find_op, AMode, OpCode},
    pseudo::PseudoOp,
    source::{Line, LineSlice},
};

pub struct ParsedLine {
    pub line: Rc<Line>,
    pub label: Option<Rc<LineSlice>>,
    pub action: Option<Box<dyn Action>>,
    pub comment: Option<Rc<LineSlice>>,
}

impl ParsedLine {
    /// If we have a comment, remove its leading ';' and strip whitespace.
    /// Returns Some(s) if the resulting string is non-empty.
    pub fn filter_comment(&self) -> Option<&str> {
        if let Some(l) = &self.comment {
            let s = l.text()[1..].trim();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        } else {
            None
        }
    }
}

/// Allows searching through individual characters in a line.
#[derive(Clone)]
pub struct LineChars<'a> {
    line: &'a Rc<Line>,
    chars: Enumerate<Chars<'a>>,
}

impl<'a> LineChars<'a> {
    pub fn new(line: &'a Rc<Line>) -> Self {
        Self {
            line,
            chars: line.text.chars().enumerate(),
        }
    }
}

impl Iterator for LineChars<'_> {
    type Item = (char, Rc<LineSlice>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((char_index, c)) = self.chars.next() {
            Some((
                c,
                Rc::new(LineSlice::new(
                    self.line.clone(),
                    char_index as u16,
                    (char_index as u16) + 1,
                )),
            ))
        } else {
            None
        }
    }
}

impl Assembler {
    /// Parse a single line of input. Return the label (if any), opcode/pseudo-op (if any), and comment (if any).
    pub fn parse_line(&mut self, line: Rc<Line>) -> Result<ParsedLine, String> {
        let og_line = line.clone();
        let og_chars = LineChars::new(&line);
        let mut chars = og_chars.clone().better_peekable();

        let label = self.parse_label(&mut chars)?;
        if label.is_none() {
            chars = og_chars.better_peekable();
        }
        let action = self.parse_action(&mut chars, &og_line)?;
        let comment = self.parse_comment(&mut chars)?;

        self.skip_ws(&mut chars);
        if let Some((_, pos)) = chars.next() {
            pos.err("unexpected characters past end of line")
        } else {
            Ok(ParsedLine {
                line,
                label,
                action,
                comment,
            })
        }
    }

    /// Skip leading whitespace.
    pub fn skip_ws(&mut self, chars: &mut BPeekable<LineChars>) {
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
        chars: &mut BPeekable<LineChars>,
    ) -> Result<Option<Rc<LineSlice>>, String> {
        self.skip_ws(chars);
        if let Some(name) = self.parse_name(chars) {
            if let Some((c, _)) = chars.peek() {
                if *c == ':' {
                    chars.next();
                }
            }
            if self.macros.contains_key(name.text()) {
                Ok(None)
            } else {
                let opchk = name.text().to_ascii_lowercase();
                if find_op(opchk.as_str()).is_some() {
                    Ok(None)
                } else {
                    Ok(Some(name))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Grab a leading Name, if any.
    fn parse_name(&mut self, chars: &mut BPeekable<LineChars>) -> Option<Rc<LineSlice>> {
        if let Some((c, start)) = chars.peek().cloned() {
            if is_alpha(c) {
                chars.next();
                let mut end = start.end_char;
                while let Some((c, new_end)) = chars.peek() {
                    if !is_alphanum(*c) {
                        break;
                    }
                    end = new_end.end_char;
                    chars.next();
                }
                Some(Rc::new(start.with_end(end)))
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
        chars: &mut BPeekable<LineChars>,
        line: &Rc<Line>,
    ) -> Result<Option<Box<dyn Action>>, String> {
        self.skip_ws(chars);

        if let Some((c, start)) = chars.peek().cloned() {
            if c == '=' {
                chars.next();
                return Ok(Some(Box::new(PseudoOp::new(
                    start,
                    vec![self.parse_expr(chars)?],
                ))));
            }
            if c == '.' {
                chars.next();
                return self.parse_pseudo(start, chars).map(Some);
            }
        }

        if let Some(name) = self.parse_name(chars) {
            if let Some(mac) = self.macros.get(name.text()) {
                self.parse_macro(mac.clone(), chars, line.clone()).map(Some)
            } else {
                self.parse_opcode(name, chars).map(Some)
            }
        } else {
            Ok(None)
        }
    }

    /// Return a flag if we're at end-of-line.
    /// (skips whitespace and also exits on a comment).
    pub fn at_eol(&mut self, chars: &mut BPeekable<LineChars>) -> bool {
        self.skip_ws(chars);
        if let Some((c, _)) = chars.peek() {
            *c == ';'
        } else {
            true
        }
    }

    /// Parse a psuedo-op.
    fn parse_pseudo(
        &mut self,
        start: Rc<LineSlice>,
        chars: &mut BPeekable<LineChars>,
    ) -> Result<Box<dyn Action>, String> {
        if let Some(name) = self.parse_name(chars) {
            let name = Rc::new(start.join(&name));
            if self.at_eol(chars) {
                Ok(Box::new(PseudoOp::new(name, Vec::new())))
            } else {
                let mut args = vec![self.parse_expr(chars)?];
                while let Some((c, _)) = chars.peek() {
                    if *c == ',' {
                        chars.next();
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
        opcode: Rc<LineSlice>,
        chars: &mut BPeekable<LineChars>,
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
        chars: &mut BPeekable<LineChars>,
    ) -> Result<(AMode, Option<Box<ExprNode>>), String> {
        self.skip_ws(chars);

        let head = {
            if let Some((_, slice)) = chars.peek() {
                slice.clone()
            } else {
                Rc::new(LineSlice::new(self.cur_line.clone().unwrap(), 0, 0))
            }
        };

        match chars.peek() {
            Some(('a', _)) | Some(('A', _)) => {
                if let Some((c, _)) = chars.peek_n(1) {
                    if c.is_whitespace() || *c == ';' {
                        chars.next().unwrap();
                        return Ok((AMode::Imp, None));
                    }
                } else {
                    chars.next().unwrap();
                    return Ok((AMode::Imp, None));
                }
            }
            _ => (),
        }

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
                                self.skip_ws(chars);
                                if let Some((c, _)) = chars.peek() {
                                    if *c == ',' {
                                        chars.next();
                                        self.skip_ws(chars);
                                        if let Some((c, _)) = chars.peek() {
                                            if *c == 'y' || *c == 'Y' {
                                                chars.next();
                                                return Ok((AMode::IndY, Some(expr)));
                                            }
                                        }
                                    } else {
                                        return Ok((AMode::Ind, Some(expr)));
                                    }
                                } else {
                                    return Ok((AMode::Ind, Some(expr)));
                                }
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
                        if self.at_eol(chars) {
                            return Ok((AMode::Abs, Some(expr)));
                        }
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
        chars: &mut BPeekable<LineChars>,
    ) -> Result<Option<Rc<LineSlice>>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            if c == ';' {
                if let Some((_, end)) = chars.last() {
                    Ok(Some(Rc::new(start.join(&end))))
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

/// Return a flag if the character counts as alphabetic.
#[inline]
pub fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Return a flag if the character counts as alphabetic or numeric.
#[inline]
fn is_alphanum(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use better_peekable::BetterPeekable;

    use crate::{
        asm::Assembler,
        assemble_str,
        parse::LineChars,
        source::{self, Line, LineSlice},
    };

    #[test]
    fn test_parse_name() {
        let test = source::from_str("foo\nbar foobar\n\nfoobar\n", "foobar");
        let foo = Rc::new(Line::new("foo", "foobar", 1));
        let bar = Rc::new(Line::new("bar foobar", "foobar", 2));
        let bl = Rc::new(Line::new("", "foobar", 3));
        let foobar = Rc::new(Line::new("foobar", "foobar", 4));
        let mut asm = Assembler::new(test, false);

        assert_eq!(
            asm.parse_name(&mut LineChars::new(&foo).better_peekable()),
            Some(Rc::new(LineSlice::new(foo.clone(), 0, 3)))
        );
        let mut bar_chars = LineChars::new(&bar).better_peekable();
        assert_eq!(
            asm.parse_name(&mut bar_chars),
            Some(Rc::new(LineSlice::new(bar.clone(), 0, 3)))
        );
        assert_eq!(asm.parse_name(&mut bar_chars), None);
        assert!(bar_chars.next().is_some());
        assert_eq!(
            asm.parse_name(&mut bar_chars),
            Some(Rc::new(LineSlice::new(bar.clone(), 4, 4 + 6)))
        );
        assert_eq!(
            asm.parse_name(&mut LineChars::new(&bl).better_peekable()),
            None
        );
        assert_eq!(
            asm.parse_name(&mut LineChars::new(&foobar).better_peekable()),
            Some(Rc::new(LineSlice::new(foobar, 0, 6)))
        );
    }

    #[test]
    fn test_amode() {
        let src = "lsr a
        ROR A;foo bar
        asl a\t";
        let data = assemble_str(src, "{src}").unwrap();
        assert_eq!(data, vec![0x4A, 0x6A, 0x0A]);
    }
}
