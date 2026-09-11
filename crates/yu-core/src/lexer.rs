use crate::diagnostic::{Diagnostic, ErrorKind};
use crate::keywords::keyword;
use crate::span::Span;
use crate::token::{Token, TokenKind};

const APOSTROPHES: [char; 3] = ['\'', '’', 'ʼ'];

/// Turns source text into tokens, with Indent/Dedent for blocks.
pub fn lex(src: &str) -> Result<Vec<Token>, Diagnostic> {
    Lexer::new(src).run()
}

struct Lexer<'a> {
    src: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    tokens: Vec<Token>,
    indents: Vec<usize>,
    depth: usize,
    indent_char: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        let body = src.strip_prefix('\u{feff}').unwrap_or(src);
        let skip = src.len() - body.len();
        Lexer {
            src,
            chars: body.char_indices().map(|(i, c)| (i + skip, c)).collect(),
            pos: 0,
            tokens: Vec::new(),
            indents: vec![0],
            depth: 0,
            indent_char: None,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).map(|&(_, c)| c)
    }

    fn offset(&self) -> usize {
        self.chars.get(self.pos).map_or(self.src.len(), |&(i, _)| i)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        let span = Span::new(start, self.offset());
        self.tokens.push(Token { kind, span });
    }

    fn error(&self, kind: ErrorKind, start: usize) -> Diagnostic {
        Diagnostic::new(kind, Span::new(start, self.offset()))
    }

    fn run(mut self) -> Result<Vec<Token>, Diagnostic> {
        self.line_start()?;
        while let Some(c) = self.peek() {
            let start = self.offset();
            match c {
                '\n' => {
                    self.bump();
                    if self.depth == 0 {
                        self.newline(start);
                        self.line_start()?;
                    }
                }
                ' ' | '\t' | '\r' => {
                    self.bump();
                }
                '#' => self.skip_comment(),
                '"' => self.text()?,
                c if c.is_ascii_digit() => self.number(),
                c if c.is_alphabetic() || c == '_' => self.name(),
                _ => self.symbol()?,
            }
        }
        let end = self.src.len();
        self.newline(end);
        while self.indents.len() > 1 {
            self.indents.pop();
            self.tokens.push(Token {
                kind: TokenKind::Dedent,
                span: Span::new(end, end),
            });
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(end, end),
        });
        Ok(self.tokens)
    }

    /// Ends a logical line: never two Newlines in a row, none before the first token.
    fn newline(&mut self, at: usize) {
        let last = self.tokens.last().map(|t| &t.kind);
        if !matches!(
            last,
            None | Some(TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent)
        ) {
            let span = Span::new(at, (at + 1).min(self.src.len()));
            self.tokens.push(Token {
                kind: TokenKind::Newline,
                span,
            });
        }
    }

    /// At the start of a line outside brackets: skip blank and comment-only lines,
    /// then turn the indentation width into Indent / Dedent tokens.
    fn line_start(&mut self) -> Result<(), Diagnostic> {
        loop {
            let begin = self.offset();
            let (mut width, mut spaces, mut tabs) = (0, false, false);
            while let Some(c @ (' ' | '\t')) = self.peek() {
                if c == ' ' {
                    width += 1;
                    spaces = true;
                } else {
                    width += 4;
                    tabs = true;
                }
                self.bump();
            }
            match self.peek() {
                None => return Ok(()),
                Some('\n') => {
                    self.bump();
                    continue;
                }
                Some('\r') if self.peek_at(1) == Some('\n') => {
                    self.bump();
                    self.bump();
                    continue;
                }
                Some('#') => {
                    self.skip_comment();
                    continue;
                }
                _ => {}
            }
            let span = Span::new(begin, self.offset());
            if spaces && tabs {
                return Err(Diagnostic::new(ErrorKind::MixedIndent, span));
            }
            if width > 0 {
                let kind = if tabs { '\t' } else { ' ' };
                match self.indent_char {
                    None => self.indent_char = Some(kind),
                    Some(k) if k != kind => {
                        return Err(Diagnostic::new(ErrorKind::MixedIndent, span))
                    }
                    Some(_) => {}
                }
            }
            let current = *self.indents.last().unwrap();
            if width > current {
                self.indents.push(width);
                self.tokens.push(Token {
                    kind: TokenKind::Indent,
                    span,
                });
            } else {
                while width < *self.indents.last().unwrap() {
                    self.indents.pop();
                    let at = self.offset();
                    self.tokens.push(Token {
                        kind: TokenKind::Dedent,
                        span: Span::new(at, at),
                    });
                }
                if width != *self.indents.last().unwrap() {
                    return Err(Diagnostic::new(ErrorKind::BadDedent, span));
                }
            }
            return Ok(());
        }
    }

    /// Skips from `#` to the end of the line; the line break stays for the caller,
    /// so the main loop still ends the logical line there.
    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn text(&mut self) -> Result<(), Diagnostic> {
        let start = self.offset();
        self.bump();
        let mut value = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => return Err(self.error(ErrorKind::UnterminatedText, start)),
                Some('"') => {
                    self.bump();
                    break;
                }
                Some('\\') => {
                    let esc = self.offset();
                    self.bump();
                    let c = match self.peek() {
                        None | Some('\n') => {
                            return Err(self.error(ErrorKind::UnterminatedText, start))
                        }
                        Some(c) => c,
                    };
                    self.bump();
                    value.push(match c {
                        'n' => '\n',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => return Err(self.error(ErrorKind::BadEscape(other), esc)),
                    });
                }
                Some(c) => {
                    value.push(c);
                    self.bump();
                }
            }
        }
        self.push(TokenKind::Text(value), start);
        Ok(())
    }

    fn number(&mut self) {
        let start = self.offset();
        let mut digits = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                digits.push(c);
                self.bump();
            } else if c == '_' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()) {
                self.bump();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()) {
            digits.push('.');
            self.bump();
            while let Some(c) = self.peek().filter(|c| c.is_ascii_digit()) {
                digits.push(c);
                self.bump();
            }
        }
        self.push(
            TokenKind::Number(digits.parse().expect("digits only")),
            start,
        );
    }

    fn name(&mut self) {
        let start = self.offset();
        let mut word = String::new();
        while let Some(c) = self.peek() {
            let joins = APOSTROPHES.contains(&c)
                && word.chars().last().is_some_and(|p| p.is_alphabetic())
                && self.peek_at(1).is_some_and(|n| n.is_alphabetic());
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
            } else if joins {
                word.push('\'');
            } else {
                break;
            }
            self.bump();
        }
        let kind = match keyword(&word) {
            Some(k) => TokenKind::Kw(k),
            None => TokenKind::Name(word),
        };
        self.push(kind, start);
    }

    fn or_eq(&mut self, yes: TokenKind, no: TokenKind) -> TokenKind {
        if self.peek() == Some('=') {
            self.bump();
            yes
        } else {
            no
        }
    }

    fn symbol(&mut self) -> Result<(), Diagnostic> {
        let start = self.offset();
        let c = self.bump().expect("called on a char");
        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '(' | '[' => {
                self.depth += 1;
                if c == '(' {
                    TokenKind::LParen
                } else {
                    TokenKind::LBracket
                }
            }
            ')' | ']' => {
                self.depth = self.depth.saturating_sub(1);
                if c == ')' {
                    TokenKind::RParen
                } else {
                    TokenKind::RBracket
                }
            }
            '=' => self.or_eq(TokenKind::EqEq, TokenKind::Assign),
            '<' => self.or_eq(TokenKind::LessEq, TokenKind::Less),
            '>' => self.or_eq(TokenKind::GreaterEq, TokenKind::Greater),
            '!' if self.peek() == Some('=') => {
                self.bump();
                TokenKind::NotEq
            }
            other => return Err(self.error(ErrorKind::UnexpectedChar(other), start)),
        };
        self.push(kind, start);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keywords::Keyword::*;
    use crate::token::TokenKind::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    fn name(s: &str) -> TokenKind {
        Name(s.into())
    }

    #[test]
    fn numbers_and_assignment() {
        assert_eq!(
            kinds("x = 1_000.5\n"),
            vec![name("x"), Assign, Number(1000.5), Newline, Eof]
        );
    }

    #[test]
    fn apostrophe_joins_a_name_and_is_normalised() {
        assert_eq!(
            kinds("ім’я = \"Юрій\""),
            vec![name("ім'я"), Assign, Text("Юрій".into()), Newline, Eof]
        );
    }

    #[test]
    fn keywords_in_both_languages() {
        let uk = kinds("якщо так і не ні");
        assert_eq!(
            uk,
            vec![Kw(If), Kw(True), Kw(And), Kw(Not), Kw(False), Newline, Eof]
        );
        assert_eq!(kinds("if true and not false"), uk);
    }

    #[test]
    fn operators() {
        assert_eq!(
            kinds("a == b != c <= d >= e < f > g + h - i * j / k % l"),
            vec![
                name("a"),
                EqEq,
                name("b"),
                NotEq,
                name("c"),
                LessEq,
                name("d"),
                GreaterEq,
                name("e"),
                Less,
                name("f"),
                Greater,
                name("g"),
                Plus,
                name("h"),
                Minus,
                name("i"),
                Star,
                name("j"),
                Slash,
                name("k"),
                Percent,
                name("l"),
                Newline,
                Eof,
            ]
        );
    }

    #[test]
    fn text_escapes_and_errors() {
        assert_eq!(
            kinds(r#""a\n\"b""#),
            vec![Text("a\n\"b".into()), Newline, Eof]
        );
        assert_eq!(lex("\"abc").unwrap_err().kind, ErrorKind::UnterminatedText);
        assert_eq!(lex(r#""a\q""#).unwrap_err().kind, ErrorKind::BadEscape('q'));
        let err = lex("x = 1 @ 2").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedChar('@'));
        assert_eq!(err.span, Span::new(6, 7));
    }

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        assert_eq!(
            kinds("x = 1 # коментар\n\n# лише коментар\ny = 2"),
            vec![
                name("x"),
                Assign,
                Number(1.0),
                Newline,
                name("y"),
                Assign,
                Number(2.0),
                Newline,
                Eof
            ]
        );
    }

    #[test]
    fn indentation_blocks() {
        let src = "якщо x:\n    скажи(1)\n\n    скажи(2)\nскажи(3)\n";
        assert_eq!(
            kinds(src),
            vec![
                Kw(If),
                name("x"),
                Colon,
                Newline,
                Indent,
                name("скажи"),
                LParen,
                Number(1.0),
                RParen,
                Newline,
                name("скажи"),
                LParen,
                Number(2.0),
                RParen,
                Newline,
                Dedent,
                name("скажи"),
                LParen,
                Number(3.0),
                RParen,
                Newline,
                Eof,
            ]
        );
    }

    #[test]
    fn nested_blocks_close_at_end_of_file() {
        assert_eq!(
            kinds("a:\n  b:\n    c"),
            vec![
                name("a"),
                Colon,
                Newline,
                Indent,
                name("b"),
                Colon,
                Newline,
                Indent,
                name("c"),
                Newline,
                Dedent,
                Dedent,
                Eof
            ]
        );
    }

    #[test]
    fn newlines_inside_brackets_are_ignored() {
        assert_eq!(
            kinds("x = [1,\n  2]\n"),
            vec![
                name("x"),
                Assign,
                LBracket,
                Number(1.0),
                Comma,
                Number(2.0),
                RBracket,
                Newline,
                Eof
            ]
        );
    }

    #[test]
    fn indentation_errors() {
        assert_eq!(
            lex("a:\n\tb\n    c").unwrap_err().kind,
            ErrorKind::MixedIndent
        );
        assert_eq!(
            lex("a:\n    b\n  c").unwrap_err().kind,
            ErrorKind::BadDedent
        );
    }

    #[test]
    fn bom_and_crlf() {
        assert_eq!(kinds("\u{feff}x"), vec![name("x"), Newline, Eof]);
        assert_eq!(
            kinds("a:\r\n  b\r\n"),
            vec![
                name("a"),
                Colon,
                Newline,
                Indent,
                name("b"),
                Newline,
                Dedent,
                Eof
            ]
        );
    }
}
