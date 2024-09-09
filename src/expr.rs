//! Expression tree enums.

use crate::{asm::Assembler, source::LineSlice};

/// A single expression tree node.
pub struct ExprNode {
    pub label: ExLab,
    pub slice: LineSlice,
}

impl ExprNode {
    pub fn new(label: ExLab, slice: LineSlice) -> Box<Self> {
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
}

impl ExprNode {
    /// Try to evaluate an expression tree.
    pub fn eval(&self, asm: &mut Assembler) -> Result<u16, String> {
        match &self.label {
            ExLab::Name => {
                let sym = asm.lookup(self.slice.text(), &self.slice);
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
        }
    }
}