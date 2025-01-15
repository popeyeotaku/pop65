//! Expression parsing.

use std::rc::Rc;

use better_peekable::BPeekable;

use crate::{
    asm::Assembler,
    expr::{ExLab, ExprNode, RelOp},
};

use super::LineChars;

impl Assembler {
    /// Assemble an expression.
    pub fn parse_expr(
        &mut self,
        chars: &mut BPeekable<LineChars>,
    ) -> Result<Box<ExprNode>, String> {
        self.parse_hilo(chars)
    }

    /// Parse a >/< expression.
    fn parse_hilo(&mut self, chars: &mut BPeekable<LineChars>) -> Result<Box<ExprNode>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            match c {
                '<' => {
                    chars.next();
                    let right = self.parse_hilo(chars)?;
                    let slice = Rc::new(start.join(&right.slice));
                    return Ok(ExprNode::new(ExLab::Lo(right), slice));
                }
                '>' => {
                    chars.next();
                    let right = self.parse_hilo(chars)?;
                    let slice = Rc::new(start.join(&right.slice));
                    return Ok(ExprNode::new(ExLab::Hi(right), slice));
                }
                _ => (),
            }
        }
        self.parse_relop(chars)
    }

    /// Parse a relational expression.
    fn parse_relop(&mut self, chars: &mut BPeekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let mut e = self.parse_addsub(chars)?;

        self.skip_ws(chars);
        while let Some((c, start)) = chars.peek().cloned() {
            let (op, slice) = match c {
                '<' => {
                    chars.next().unwrap();
                    match chars.peek().cloned() {
                        Some(('=', end)) => {
                            chars.next().unwrap();
                            (RelOp::LessEqu, Rc::new(start.join(&end)))
                        }
                        Some(('>', end)) => {
                            chars.next().unwrap();
                            (RelOp::Nequ, Rc::new(start.join(&end)))
                        }
                        _ => (RelOp::Less, start),
                    }
                }
                '>' => {
                    chars.next().unwrap();
                    match chars.peek().cloned() {
                        Some(('=', end)) => {
                            chars.next().unwrap();
                            (RelOp::GreatEqu, Rc::new(start.join(&end)))
                        }
                        Some(('<', end)) => {
                            chars.next().unwrap();
                            (RelOp::Nequ, Rc::new(start.join(&end)))
                        }
                        _ => (RelOp::Great, start),
                    }
                }
                '=' => {
                    chars.next().unwrap();
                    (RelOp::Equ, start)
                }
                _ => break,
            };
            let right = self.parse_addsub(chars)?;
            e = ExprNode::new(ExLab::RelOp(op, e, right), slice);
        }

        Ok(e)
    }

    /// Parse a '+'/'-' expression.
    fn parse_addsub(&mut self, chars: &mut BPeekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let mut e = self.parse_muldiv(chars)?;

        self.skip_ws(chars);
        while let Some((c, _)) = chars.peek() {
            match c {
                '+' => {
                    chars.next();
                    let right = self.parse_muldiv(chars)?;
                    let slice = Rc::new(e.slice.join(&right.slice));
                    e = ExprNode::new(ExLab::Add(e, right), slice);
                    self.skip_ws(chars);
                }
                '-' => {
                    chars.next();
                    let right = self.parse_muldiv(chars)?;
                    let slice = Rc::new(e.slice.join(&right.slice));
                    e = ExprNode::new(ExLab::Sub(e, right), slice);
                    self.skip_ws(chars);
                }
                _ => break,
            }
        }
        Ok(e)
    }

    /// Parse a '*'/'/'/'%' expression.
    fn parse_muldiv(&mut self, chars: &mut BPeekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let mut e = self.parse_unary(chars)?;
        self.skip_ws(chars);
        while let Some((c, _)) = chars.peek() {
            match c {
                '*' => {
                    chars.next();
                    let right = self.parse_unary(chars)?;
                    let slice = Rc::new(e.slice.join(&right.slice));
                    e = ExprNode::new(ExLab::Mul(e, right), slice);
                    self.skip_ws(chars);
                }
                '/' => {
                    chars.next();
                    let right = self.parse_unary(chars)?;
                    let slice = Rc::new(e.slice.join(&right.slice));
                    e = ExprNode::new(ExLab::Div(e, right), slice);
                    self.skip_ws(chars);
                }
                '%' => {
                    chars.next();
                    let right = self.parse_unary(chars)?;
                    let slice = Rc::new(e.slice.join(&right.slice));
                    e = ExprNode::new(ExLab::Mod(e, right), slice);
                    self.skip_ws(chars);
                }
                _ => break,
            }
        }
        Ok(e)
    }

    /// Parse a unary expression.
    fn parse_unary(&mut self, chars: &mut BPeekable<LineChars>) -> Result<Box<ExprNode>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            if c == '-' {
                chars.next();
                let right = self.parse_unary(chars)?;
                let slice = Rc::new(start.join(&right.slice));
                return Ok(ExprNode::new(ExLab::Neg(right), slice));
            }
        }
        self.parse_primary(chars)
    }

    /// Parse a primary expression.
    pub fn parse_primary(
        &mut self,
        chars: &mut BPeekable<LineChars>,
    ) -> Result<Box<ExprNode>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            if c == '(' {
                chars.next();
                let e = self.parse_expr(chars)?;
                self.skip_ws(chars);
                if let Some((c, end)) = chars.next() {
                    if c == ')' {
                        return Ok(ExprNode::new(ExLab::Expr(e), Rc::new(start.join(&end))));
                    }
                }
                start.err("missing closing ')'")
            } else if c.is_ascii_digit() {
                self.parse_num(10, chars)
            } else if c == '$' {
                chars.next();
                self.parse_num(16, chars)
            } else if c == '%' {
                chars.next();
                self.parse_num(2, chars)
            } else if c == '@' {
                chars.next();
                self.parse_num(8, chars)
            } else if c.is_ascii_alphabetic() {
                let name = self.parse_name(chars).unwrap();
                // Ensure this reference to the symbol is noticed
                self.lookup(name.text(), name.clone());
                Ok(ExprNode::new(ExLab::Name, name))
            } else if c == '\'' || c == '"' {
                self.parse_str(chars)
            } else if c == '*' {
                chars.next();
                Ok(ExprNode::new(ExLab::Num(self.pc), start))
            } else {
                start.err("Missing primary expression")
            }
        } else {
            self.cur_line
                .as_ref()
                .unwrap()
                .err("Missing primary expression")
        }
    }

    /// Parse a number in a given base.
    fn parse_num(
        &mut self,
        base: u8,
        chars: &mut BPeekable<LineChars>,
    ) -> Result<Box<ExprNode>, String> {
        let (c, start) = chars.peek().unwrap();
        let mut i = {
            if let Some(i) = c.to_digit(base as u32) {
                i as u16
            } else {
                return self
                    .cur_line
                    .as_ref()
                    .unwrap()
                    .err(&format!("'{}' isn't a digit in base {}", c, base));
            }
        };
        let mut slice = start.clone();
        chars.next();
        while let Some((c, end)) = chars.peek() {
            if let Some(digit) = c.to_digit(base as u32) {
                i = i * (base as u16) + (digit as u16);
                slice = Rc::new(slice.join(end));
                chars.next();
            } else {
                break;
            }
        }
        Ok(ExprNode::new(ExLab::Num(i), slice))
    }

    /// Parse a string.
    fn parse_str(&mut self, chars: &mut BPeekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let (quote, start) = chars.next().unwrap();
        let mut s = String::new();
        for (c, end) in chars.by_ref() {
            if c == quote {
                let slice = Rc::new(start.join(&end));
                return Ok(ExprNode::new(ExLab::Str(s), slice));
            } else {
                s.push(c);
            }
        }
        start.err(&format!("missing closing quote: {quote}"))
    }
}
