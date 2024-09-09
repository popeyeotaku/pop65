//! Source file handling.

/// Allows reading from source files.
pub type Source = Box<dyn Iterator<Item = Line>>;

/// Construct a fake-o source from a single string.
pub fn from_str(s: &str, path: &str) -> Source {
    todo!()
}

/// A single line of input.
pub struct Line {}
