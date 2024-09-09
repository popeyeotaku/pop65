//! Implements opcodes and pseudo-ops;

use crate::{asm::Assembler, source::LineSlice};

pub trait Action {
    /// Handle pass-1 parsing. Return the size in bytes to advance the PC.
    fn pass1(&self, assembler: &mut Assembler, label: &Option<LineSlice>) -> Result<u16, String> {
        todo!()
    }

    /// Return the underlying LineSlice for this entire action.
    fn line_slice(&self) -> &LineSlice {
        todo!()
    }
}
