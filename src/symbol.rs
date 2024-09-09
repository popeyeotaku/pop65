//! Support for asm symbols.

use crate::source::LineSlice;

/// An entry in the symbol table.
pub struct Symbol {
    name: String,
    value: Option<u16>,
    defined: Option<LineSlice>,
    references: Vec<LineSlice>,
    source: Option<LineSlice>,
}
