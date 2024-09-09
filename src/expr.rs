//! Expression tree enums.

use crate::source::LineSlice;

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
