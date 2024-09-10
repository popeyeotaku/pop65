//! Expression parsing.

use std::iter::Peekable;

use crate::{
    asm::Assembler,
    expr::{ExLab, ExprNode},
};

use super::LineChars;

impl Assembler {
    /// Assemble an expression.
    pub fn parse_expr(&mut self, chars: &mut Peekable<LineChars>) -> Result<Box<ExprNode>, String> {
        self.parse_hilo(chars)
    }

    /// Parse a >/< expression.
    fn parse_hilo(&mut self, chars: &mut Peekable<LineChars>) -> Result<Box<ExprNode>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            match c {
                '<' => {
                    chars.next();
                    let right = self.parse_hilo(chars)?;
                    let slice = start.join(&right.slice);
                    return Ok(ExprNode::new(ExLab::Lo(right), slice));
                }
                '>' => {
                    chars.next();
                    let right = self.parse_hilo(chars)?;
                    let slice = start.join(&right.slice);
                    return Ok(ExprNode::new(ExLab::Hi(right), slice));
                }
                _ => (),
            }
        }
        self.parse_addsub(chars)
    }

    /// Parse a '+'/'-' expression.
    fn parse_addsub(&mut self, chars: &mut Peekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let mut e = self.parse_muldiv(chars)?;

        self.skip_ws(chars);
        while let Some((c, _)) = chars.peek() {
            match c {
                '+' => {
                    chars.next();
                    let right = self.parse_muldiv(chars)?;
                    let slice = e.slice.join(&right.slice);
                    e = ExprNode::new(ExLab::Add(e, right), slice);
                    self.skip_ws(chars);
                }
                '-' => {
                    chars.next();
                    let right = self.parse_muldiv(chars)?;
                    let slice = e.slice.join(&right.slice);
                    e = ExprNode::new(ExLab::Sub(e, right), slice);
                    self.skip_ws(chars);
                }
                _ => break,
            }
        }
        Ok(e)
    }

    /// Parse a '*'/'/'/'%' expression.
    fn parse_muldiv(&mut self, chars: &mut Peekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let mut e = self.parse_unary(chars)?;
        self.skip_ws(chars);
        while let Some((c, _)) = chars.peek() {
            match c {
                '*' => {
                    chars.next();
                    let right = self.parse_unary(chars)?;
                    let slice = e.slice.join(&right.slice);
                    e = ExprNode::new(ExLab::Mul(e, right), slice);
                    self.skip_ws(chars);
                }
                '/' => {
                    chars.next();
                    let right = self.parse_unary(chars)?;
                    let slice = e.slice.join(&right.slice);
                    e = ExprNode::new(ExLab::Div(e, right), slice);
                    self.skip_ws(chars);
                }
                '%' => {
                    chars.next();
                    let right = self.parse_unary(chars)?;
                    let slice = e.slice.join(&right.slice);
                    e = ExprNode::new(ExLab::Mod(e, right), slice);
                    self.skip_ws(chars);
                }
                _ => break,
            }
        }
        Ok(e)
    }

    /// Parse a unary expression.
    fn parse_unary(&mut self, chars: &mut Peekable<LineChars>) -> Result<Box<ExprNode>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            if c == '-' {
                chars.next();
                let right = self.parse_unary(chars)?;
                let slice = start.join(&right.slice);
                return Ok(ExprNode::new(ExLab::Neg(right), slice));
            }
        }
        self.parse_primary(chars)
    }

    /// Parse a primary expression.
    pub fn parse_primary(
        &mut self,
        chars: &mut Peekable<LineChars>,
    ) -> Result<Box<ExprNode>, String> {
        self.skip_ws(chars);
        if let Some((c, start)) = chars.peek().cloned() {
            if c == '(' {
                chars.next();
                let e = self.parse_expr(chars)?;
                self.skip_ws(chars);
                if let Some((c, end)) = chars.next() {
                    if c == ')' {
                        return Ok(ExprNode::new(ExLab::Expr(e), start.join(&end)));
                    }
                }
                start.err("missing closing ')'")
            } else if c.is_ascii_digit() {
                self.parse_num(10, chars)
            } else if c == '$' {
                chars.next();
                self.parse_num(16, chars)
            } else if c.is_ascii_alphabetic() {
                let name = self.parse_name(chars).unwrap();
                // Ensure this reference to the symbol is noticed
                self.lookup(name.text(), &name);
                Ok(ExprNode::new(ExLab::Name, name))
            } else if c == '\'' || c == '"' {
                self.parse_str(chars)
            } else if c == '*' {
                chars.next();
                Ok(ExprNode::new(ExLab::Num(*self.pc()?), start))
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
        chars: &mut Peekable<LineChars>,
    ) -> Result<Box<ExprNode>, String> {
        let (c, start) = chars.peek().unwrap();
        let mut i: u16 = c.to_digit(base as u32).unwrap() as u16;
        let mut slice = start.clone();
        chars.next();
        while let Some((c, end)) = chars.peek() {
            if let Some(digit) = c.to_digit(base as u32) {
                i = i * (base as u16) + (digit as u16);
                slice = slice.join(end);
                chars.next();
            } else {
                break;
            }
        }
        Ok(ExprNode::new(ExLab::Num(i), slice))
    }

    /// Parse a string.
    fn parse_str(&mut self, chars: &mut Peekable<LineChars>) -> Result<Box<ExprNode>, String> {
        let (quote, start) = chars.next().unwrap();
        let mut s = String::new();
        for (c, end) in chars.by_ref() {
            if c == quote {
                let slice = start.join(&end);
                return Ok(ExprNode::new(ExLab::Str(s), slice));
            } else {
                s.push(c);
            }
        }
        start.err(&format!("missing closing quote: {quote}"))
    }
}
