//! Implements opcodes and pseudo-ops;

use crate::{asm::Assembler, expr::ExprNode, source::LineSlice};

pub trait Action {
    /// Handle pass-1 parsing. Return the size in bytes to advance the PC.
    fn pass1(
        &self,
        assembler: &mut Assembler,
        label: &Option<Box<LineSlice>>,
    ) -> Result<u16, String>;

    /// Handle pass-2 parsing. Return the bytes to be sent to the output.
    fn pass2(&self, assembler: &mut Assembler) -> Result<Vec<u8>, String>;

    /// Return the underlying LineSlice for this entire action.
    fn line_slice(&self) -> LineSlice;
}

/// Indicates a pseudo-op.
pub struct PseudoOp {
    op_name: LineSlice,
    #[allow(clippy::vec_box)]
    args: Vec<Box<ExprNode>>,
}

impl PseudoOp {
    #[allow(clippy::vec_box)]
    pub fn new(op_name: LineSlice, args: Vec<Box<ExprNode>>) -> Self {
        Self { op_name, args }
    }
}

impl Action for PseudoOp {
    fn pass1(
        &self,
        assembler: &mut Assembler,
        label: &Option<Box<LineSlice>>,
    ) -> Result<u16, String> {
        todo!()
    }

    fn pass2(&self, assembler: &mut Assembler) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn line_slice(&self) -> LineSlice {
        if let Some(last_arg) = self.args.last() {
            self.op_name.join(&last_arg.slice)
        } else {
            self.op_name.clone()
        }
    }
}
