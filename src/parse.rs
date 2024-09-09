//! Assembly parsing.

use crate::{
    action::Action,
    asm::Assembler,
    source::{Line, LineSlice},
};

pub struct ParsedLine {
    pub label: Option<Box<LineSlice>>,
    pub action: Option<Box<dyn Action>>,
    pub comment: Option<Box<LineSlice>>,
}

impl Assembler {
    /// Parse a single line of input. Return the label (if any), opcode/pseudo-op (if any), and comment (if any).
    pub fn parse_line(&mut self, line: &Line) -> Result<ParsedLine, String> {
        todo!()
    }
}
