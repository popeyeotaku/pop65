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
#[derive(PartialEq, Debug)]
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
}

#[cfg(test)]
mod tests {
    use super::{from_str, Line, SrcStack};

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
}
