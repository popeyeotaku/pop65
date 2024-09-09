//! Support for asm symbols.

use crate::source::LineSlice;

pub struct Symbol {
    name: String,
    value: u16,
    source: Option<LineSlice>,
}
