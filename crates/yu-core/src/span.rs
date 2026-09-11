/// A byte range in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// The smallest span that covers both.
    pub fn to(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

/// 1-based line and column (counted in characters) of a byte offset.
pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let (mut line, mut col) = (1, 1);
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// The text of the 1-based `line`, without the line break.
pub fn line_text(src: &str, line: usize) -> &str {
    src.lines().nth(line.saturating_sub(1)).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_characters_not_bytes() {
        let src = "а = 1\nскажи(а)";
        assert_eq!(line_col(src, 0), (1, 1));
        assert_eq!(line_col(src, src.find("скажи").unwrap()), (2, 1));
        assert_eq!(line_col(src, src.find('(').unwrap()), (2, 6));
    }

    #[test]
    fn line_text_returns_one_line() {
        assert_eq!(line_text("a\nb\r\nc", 2), "b");
        assert_eq!(line_text("a", 5), "");
    }

    #[test]
    fn to_covers_both_spans() {
        assert_eq!(Span::new(3, 5).to(Span::new(1, 4)), Span::new(1, 5));
    }
}
