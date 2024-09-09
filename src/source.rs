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
    use super::{from_str, Line};

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
}
