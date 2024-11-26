//! Expression tree enums.

use std::rc::Rc;

use crate::{asm::Assembler, source::LineSlice};

/// A single expression tree node.
pub struct ExprNode {
    pub label: ExLab,
    pub slice: Rc<LineSlice>,
}

impl ExprNode {
    pub fn new(label: ExLab, slice: Rc<LineSlice>) -> Box<Self> {
        Box::new(Self { label, slice })
    }
}

/// A given type of expression tree node, along with its operands.
pub enum ExLab {
    Name,
    Num(u16),
    Add(Box<ExprNode>, Box<ExprNode>),
    Sub(Box<ExprNode>, Box<ExprNode>),
    Mul(Box<ExprNode>, Box<ExprNode>),
    Div(Box<ExprNode>, Box<ExprNode>),
    Mod(Box<ExprNode>, Box<ExprNode>),
    Neg(Box<ExprNode>),
    Hi(Box<ExprNode>),
    Lo(Box<ExprNode>),
    Expr(Box<ExprNode>),
    Str(String),
    RelOp(RelOp, Box<ExprNode>, Box<ExprNode>),
}

/// A relational operator.
pub enum RelOp {
    Less,
    Great,
    Equ,
    Nequ,
    LessEqu,
    GreatEqu,
}

impl ExprNode {
    /// Try to evaluate an expression tree.
    pub fn eval(&self, asm: &mut Assembler) -> Result<u16, String> {
        match &self.label {
            ExLab::Name => {
                let sym = asm.lookup(self.slice.text(), self.slice.clone());
                if let Some(value) = sym.value {
                    Ok(value)
                } else {
                    self.slice
                        .err(&format!("'{}' undefined", self.slice.text()))
                }
            }
            ExLab::Num(i) => Ok(*i),
            ExLab::Add(left, right) => Ok(left.eval(asm)?.wrapping_add(right.eval(asm)?)),
            ExLab::Sub(left, right) => Ok(left.eval(asm)?.wrapping_sub(right.eval(asm)?)),
            ExLab::Mul(left, right) => Ok(left.eval(asm)?.wrapping_mul(right.eval(asm)?)),
            ExLab::Div(left, right) => Ok(left.eval(asm)?.wrapping_div(right.eval(asm)?)),
            ExLab::Mod(left, right) => Ok(left.eval(asm)?.wrapping_rem(right.eval(asm)?)),
            ExLab::Neg(e) => Ok(e.eval(asm)?.wrapping_neg()),
            ExLab::Hi(e) => Ok(e.eval(asm)? >> 8),
            ExLab::Lo(e) => Ok(e.eval(asm)? & 0xFF),
            ExLab::Expr(e) => e.eval(asm),
            ExLab::Str(s) => match s.len() {
                0 => self.slice.err("string must contain one character"),
                2.. => self.slice.err("string must consist of one byte only"),
                _ => Ok(s.bytes().next().unwrap() as u16),
            },
            ExLab::RelOp(rel_op, left, right) => {
                let left = left.eval(asm)?;
                let right = right.eval(asm)?;
                let result = match rel_op {
                    RelOp::Less => left < right,
                    RelOp::Great => left > right,
                    RelOp::Equ => left == right,
                    RelOp::Nequ => left != right,
                    RelOp::LessEqu => left <= right,
                    RelOp::GreatEqu => left >= right,
                };
                if result {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        asm::Assembler,
        parse::LineChars,
        source::{self, Line, LineSlice},
    };

    use super::{ExLab, ExprNode};

    #[test]
    fn test_expr_parse_eval() {
        let text = "(1 + 2) * 3 - 4";
        let mut asm = Assembler::new(source::from_str(text, "text"));
        let line = Rc::new(Line::new(text, "text", 1));
        let e = asm
            .parse_expr(&mut LineChars::new(&line).peekable())
            .unwrap();
        assert_eq!(e.eval(&mut asm), Ok((1 + 2) * 3 - 4));
    }

    #[test]
    fn test_neg() {
        let f = Rc::new(LineSlice::new(Rc::new(Line::new("foo", "foo", 1)), 0, 0));
        let n = ExprNode::new(
            ExLab::Neg(ExprNode::new(ExLab::Num(1), f.clone())),
            f.clone(),
        );
        let mut a = Assembler::new(source::from_str("foo", "foo"));
        assert_eq!(n.eval(&mut a), Ok(0xFFFF));
    }
}
