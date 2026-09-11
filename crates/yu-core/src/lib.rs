//! Yu — a small programming language that speaks Ukrainian and English.
//!
//! ```
//! struct Out(Vec<String>);
//! impl yu_core::Host for Out {
//!     fn print(&mut self, line: &str) { self.0.push(line.into()) }
//!     fn ask(&mut self, _: &str) -> Option<String> { None }
//! }
//! let mut out = Out(vec![]);
//! yu_core::run("скажи(2 + 2)", &mut out, Default::default()).unwrap();
//! assert_eq!(out.0, ["4"]);
//! ```

pub mod ast;
pub mod builtins;
pub mod diagnostic;
pub mod draw;
pub mod env;
pub mod host;
pub mod interp;
pub mod keywords;
pub mod lang;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod suggest;
pub mod token;
pub mod value;

pub use diagnostic::{Diagnostic, ErrorKind};
pub use host::Host;
pub use interp::Options;
pub use lang::Lang;
pub use span::Span;
pub use value::Value;

/// Keeps variables between runs — what the REPL needs.
pub struct Session {
    globals: env::Env,
    opts: Options,
}

impl Session {
    pub fn new(opts: Options) -> Self {
        Session {
            globals: env::Scope::global(),
            opts,
        }
    }

    pub fn options(&self) -> &Options {
        &self.opts
    }

    /// Runs `src`; returns the value of a final expression statement, if any.
    pub fn run(&mut self, src: &str, host: &mut dyn Host) -> Result<Option<Value>, Diagnostic> {
        let tokens = lexer::lex(src)?;
        let program = parser::parse(src, &tokens)?;
        interp::Interp::new(host, self.globals.clone(), self.opts).run(&program)
    }
}

/// Runs a whole program once.
pub fn run(src: &str, host: &mut dyn Host, opts: Options) -> Result<(), Diagnostic> {
    Session::new(opts).run(src, host).map(|_| ())
}
