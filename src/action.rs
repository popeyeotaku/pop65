//! Implements opcodes and pseudo-ops;

use crate::{asm::Assembler, source::LineSlice};

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
