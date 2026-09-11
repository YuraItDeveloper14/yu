//! Yu — a small programming language that speaks Ukrainian and English.

pub mod diagnostic;
pub mod lang;
pub mod span;

pub use diagnostic::{Diagnostic, ErrorKind};
pub use lang::Lang;
pub use span::Span;
