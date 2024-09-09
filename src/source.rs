//! Source file handling.

/// Used to specify a line number.
pub type LineNum = u32;

/// Allows reading from source files.
pub type Source = Box<dyn Iterator<Item = Line>>;

/// Construct a fake-o source from a single string.
pub fn from_str(s: &str, path: &str) -> Source {
    Box::new(StrSrc::new(s, path, 1))
}

/// Source from a parent string.
struct StrSrc {
    lines: Vec<String>,
    path: String,
    line_num: LineNum,
}

impl StrSrc {
    pub fn new(src: &str, path: &str, line_num: LineNum) -> Self {
        Self {
            lines: Vec::from_iter(src.lines().rev().map(|line| line.to_string())),
            path: path.to_string(),
            line_num,
        }
    }
}

impl Iterator for StrSrc {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(text) = self.lines.pop() {
            let line_num = self.line_num;
            self.line_num += 1;
            Some(Line::new(&text, &self.path, line_num))
        } else {
            None
        }
    }
}

/// A stack of inputs: used as the main input for assemble().
pub struct SrcStack {
    sources: Vec<Source>,
}

impl SrcStack {
    pub fn new(starting_src: Source) -> Self {
        Self {
            sources: vec![starting_src],
        }
    }

    /// Remove the last source.
    pub fn push(&mut self, src: Source) {
        self.sources.push(src);
    }
}

impl Iterator for SrcStack {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(src) = self.sources.last_mut() {
                if let Some(line) = src.next() {
                    return Some(line);
                } else {
                    let popped = self.sources.pop();
                    debug_assert!(popped.is_some());
                }
            } else {
                return None;
            }
        }
    }
}

/// A single line of input.
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Line {
    pub text: String,
    pub path: String,
    pub line_num: LineNum,
}

impl Line {
    pub fn new(text: &str, path: &str, line_num: LineNum) -> Self {
        Self {
            text: text.to_string(),
            path: path.to_string(),
            line_num,
        }
    }

    /// Return a LineSlice into this line.
    pub fn slice(&self, start_char: u16, end_char: u16) -> LineSlice {
        LineSlice::new(self, start_char, end_char)
    }

    /// Return the position of the source line.
    ///
    /// A line with path "foo" and line_num 11 will pos() as
    /// "foo:11"
    pub fn pos(&self) -> String {
        format!("{}:{}", self.path, self.line_num)
    }

    /// Construct an error message at this line's pos() as a header.
    pub fn err<T>(&self, msg: &str) -> Result<T, String> {
        Err(format!("{}:{}", self.pos(), msg))
    }
}

/// A slice within a given line.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LineSlice {
    line: Line,
    pub start_char: u16,
    pub end_char: u16,
    start_index: u16,
    end_index: u16,
}

impl LineSlice {
    pub fn new(line: &Line, start_char: u16, end_char: u16) -> Self {
        let line = line.clone();
        let (start, _) = line
            .text
            .char_indices()
            .nth(start_char as usize)
            .unwrap_or((line.text.len(), ' '));
        let (end, _) = line
            .text
            .char_indices()
            .nth(end_char as usize)
            .unwrap_or((line.text.len(), ' '));

        Self {
            line,
            start_char,
            end_char,
            start_index: start as u16,
            end_index: end as u16,
        }
    }

    /// Return a string representing the position in the line.
    ///
    /// A line with path "foo", line_num 11, start_char 3 will pos() as
    /// "foo:11:3".
    pub fn pos(&self) -> String {
        format!("{}:{}:{}", self.path(), self.line_num(), self.start_char)
    }

    /// Construct an error message using this slice's pos() as a header.
    pub fn err<T>(&self, msg: &str) -> Result<T, String> {
        Err(format!("{}:{}", self.pos(), msg))
    }

    /// Return the underlying path.
    pub fn path(&self) -> &str {
        &self.line.path
    }

    /// Return the underlying line number.
    pub fn line_num(&self) -> LineNum {
        self.line.line_num
    }

    /// Return the underlying text of the complete line.
    pub fn line_text(&self) -> &str {
        &self.line.text
    }

    /// Return the text of this slice.
    pub fn text(&self) -> &str {
        let start = self.start_index as usize;
        let end = self.end_index as usize;
        &self.line_text()[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::{from_str, Line, LineSlice, SrcStack};

    #[test]
    fn test_strsrc() {
        let foobar = "foo\nbar\nfoobar\n";
        let src = from_str(foobar, "foobar");
        let cmp = vec![
            Line::new("foo", "foobar", 1),
            Line::new("bar", "foobar", 2),
            Line::new("foobar", "foobar", 3),
        ];
        assert_eq!(Vec::from_iter(src), cmp);
    }

    #[test]
    fn test_srcstk() {
        let foobar = "foo\nbar\nfoobar\n";
        let barfoo = "barfoo\nbar\nfoo\n";
        let mut stk = SrcStack::new(from_str(foobar, "foobar"));
        assert_eq!(stk.next(), Some(Line::new("foo", "foobar", 1)));
        stk.push(from_str(barfoo, "barfoo"));
        assert_eq!(
            Vec::from_iter(stk),
            vec![
                Line::new("barfoo", "barfoo", 1),
                Line::new("bar", "barfoo", 2),
                Line::new("foo", "barfoo", 3),
                Line::new("bar", "foobar", 2),
                Line::new("foobar", "foobar", 3)
            ]
        );
    }

    #[test]
    fn test_line_slice() {
        let foobar = Line::new("foobar", "foobar", 1);
        let foo = LineSlice::new(&foobar, 0, 3);
        let bar = LineSlice::new(&foobar, 3, 6);
        let f = LineSlice::new(&foobar, 0, 1);
        let none = LineSlice::new(&foobar, 3, 3);
        let all = LineSlice::new(&foobar, 0, 6);
        let end = LineSlice::new(&foobar, 6, 6);

        assert_eq!(foo.text(), "foo");
        assert_eq!(bar.text(), "bar");
        assert_eq!(f.text(), "f");
        assert_eq!(none.text(), "");
        assert_eq!(all.text(), "foobar");
        assert_eq!(end.text(), "");
    }

    #[test]
    fn test_pos() {
        let foo = Line::new("foobar", "foo", 11);
        let bar = LineSlice::new(&foo, 3, 6);
        assert_eq!(&foo.pos(), "foo:11");
        assert_eq!(&bar.pos(), "foo:11:3");
    }
}
