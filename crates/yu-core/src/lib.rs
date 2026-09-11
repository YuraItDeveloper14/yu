//! Yu — a small programming language that speaks Ukrainian and English.

pub mod ast;
pub mod diagnostic;
pub mod keywords;
pub mod lang;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use diagnostic::{Diagnostic, ErrorKind};
pub use lang::Lang;
pub use span::Span;
