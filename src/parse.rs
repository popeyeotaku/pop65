//! Assembly parsing.

use crate::{
    action::Action,
    asm::Assembler,
    source::{Line, LineSlice},
};

impl Assembler {
    /// Parse a single line of input. Return the label (if any), opcode/pseudo-op (if any), and comment (if any).
    pub fn parse_line(
        &mut self,
        line: &Line,
    ) -> Result<
        (
            Option<LineSlice>,
            Option<Box<dyn Action>>,
            Option<LineSlice>,
        ),
        String,
    > {
        todo!()
    }
}
