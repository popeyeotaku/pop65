//! Implements opcodes and pseudo-ops;

use std::rc::Rc;

use crate::{asm::Assembler, source::LineSlice};

pub trait Action {
    /// Handle pass-1 parsing. Return the size in bytes to advance the PC.
    fn pass1(&self, assembler: &mut Assembler, label: Option<Rc<LineSlice>>)
        -> Result<u16, String>;

    /// Handle pass-2 parsing. Return the bytes to be sent to the output.
    fn pass2(&self, assembler: &mut Assembler) -> Result<Vec<u8>, String>;

    /// Return the underlying LineSlice for this entire action.
    fn line_slice(&self) -> Rc<LineSlice>;

    /// Return a flag for if this is the `.equ`/`=` pseudo-op.
    fn is_equ(&self) -> bool {
        false
    }

    /// Return a flag for if this is an ".else" or ".endif" pseudo-op.
    fn is_if_affiliated(&self) -> bool {
        false
    }

    /// If this is the start of a new macro, return the name of the macro.
    fn is_macro_def(&self) -> Option<String> {
        None
    }

    /// If this is an equ macro, try to evaluate it.
    fn eval_equ(&self, assembler: &mut Assembler) -> Option<u16> {
        let _ = assembler;
        None
    }
}
