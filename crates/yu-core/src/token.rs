use crate::keywords::Keyword;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Text(String),
    Name(String),
    Kw(Keyword),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    EqEq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
