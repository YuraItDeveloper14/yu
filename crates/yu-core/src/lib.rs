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
pub mod library;
pub mod parser;
pub mod span;
pub mod suggest;
pub mod svg;
pub mod token;
pub mod value;

pub use diagnostic::{Diagnostic, ErrorKind};
pub use draw::{DrawCmd, Drawing, Rgb};
pub use host::Host;
pub use interp::Options;
pub use lang::Lang;
pub use span::Span;
pub use value::Value;

/// Keeps variables, the drawing and the random sequence between runs — what the REPL needs.
pub struct Session {
    globals: env::Env,
    opts: Options,
    drawing: Drawing,
}

impl Session {
    pub fn new(opts: Options) -> Self {
        Session {
            globals: env::Scope::global(),
            opts,
            drawing: Drawing::default(),
        }
    }

    pub fn options(&self) -> &Options {
        &self.opts
    }

    /// Everything drawn in this session so far.
    pub fn drawing(&self) -> &Drawing {
        &self.drawing
    }

    /// Runs `src`; returns the value of a final expression statement, if any.
    pub fn run(&mut self, src: &str, host: &mut dyn Host) -> Result<Option<Value>, Diagnostic> {
        let tokens = lexer::lex(src)?;
        let program = parser::parse(src, &tokens)?;
        let mut interp = interp::Interp::new(host, self.globals.clone(), self.opts);
        interp.drawing = std::mem::take(&mut self.drawing);
        let result = interp.run(&program);
        self.drawing = interp.drawing;
        // The next run continues the random sequence instead of starting it again.
        self.opts.seed = interp.rng;
        result
    }
}

/// Runs a whole program once.
pub fn run(src: &str, host: &mut dyn Host, opts: Options) -> Result<(), Diagnostic> {
    Session::new(opts).run(src, host).map(|_| ())
}
