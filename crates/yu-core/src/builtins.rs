use crate::diagnostic::{Diagnostic, ErrorKind};
use crate::host::Host;
use crate::lang::Lang;
use crate::span::Span;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Say,
    Ask,
    Length,
    Number,
    Text,
    Random,
    Round,
    Append,
}

/// Ukrainian and English name of every built-in.
pub const NAMES: [(Builtin, &str, &str); 8] = [
    (Builtin::Say, "скажи", "say"),
    (Builtin::Ask, "запитай", "ask"),
    (Builtin::Length, "довжина", "length"),
    (Builtin::Number, "число", "number"),
    (Builtin::Text, "текст", "text"),
    (Builtin::Random, "випадкове", "random"),
    (Builtin::Round, "округли", "round"),
    (Builtin::Append, "додай", "append"),
];

impl Builtin {
    pub fn lookup(name: &str) -> Option<Builtin> {
        NAMES
            .iter()
            .find(|(_, uk, en)| *uk == name || *en == name)
            .map(|(b, _, _)| *b)
    }

    pub fn name(self, lang: Lang) -> &'static str {
        let (_, uk, en) = NAMES
            .iter()
            .find(|(b, _, _)| *b == self)
            .expect("every built-in has names");
        lang.pick(uk, en)
    }

    /// Fewest and most arguments.
    fn arity(self) -> (usize, usize) {
        match self {
            Builtin::Say => (0, usize::MAX),
            Builtin::Ask => (0, 1),
            Builtin::Random | Builtin::Append => (2, 2),
            _ => (1, 1),
        }
    }
}

/// xorshift64: small, seeded by the host, identical in the terminal and the browser.
pub fn next_random(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn number_arg(v: &Value, span: Span) -> Result<f64, Diagnostic> {
    match v {
        Value::Number(n) => Ok(*n),
        other => Err(Diagnostic::new(ErrorKind::ExpectedNumber(other.ty()), span)),
    }
}

pub fn call(
    b: Builtin,
    args: Vec<Value>,
    span: Span,
    host: &mut dyn Host,
    lang: Lang,
    rng: &mut u64,
) -> Result<Value, Diagnostic> {
    let (min, max) = b.arity();
    if args.len() < min || args.len() > max {
        let expected = if args.len() < min { min } else { max };
        let kind = ErrorKind::ArgCount {
            name: b.name(lang).into(),
            expected,
            got: args.len(),
        };
        return Err(Diagnostic::new(kind, span));
    }
    let err = |kind: ErrorKind| Diagnostic::new(kind, span);
    Ok(match b {
        Builtin::Say => {
            let parts: Vec<String> = args.iter().map(|v| v.display(lang)).collect();
            host.print(&parts.join(" "));
            Value::Nothing
        }
        Builtin::Ask => {
            let prompt = args.first().map(|v| v.display(lang)).unwrap_or_default();
            let line = host.ask(&prompt).unwrap_or_default();
            Value::text(line.trim_end_matches(['\r', '\n']))
        }
        Builtin::Length => match &args[0] {
            Value::Text(s) => Value::Number(s.chars().count() as f64),
            Value::List(items) => Value::Number(items.borrow().len() as f64),
            other => return Err(err(ErrorKind::NotIndexable(other.ty()))),
        },
        Builtin::Number => match &args[0] {
            Value::Number(n) => Value::Number(*n),
            Value::Bool(b) => Value::Number(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => match s.trim().replace(',', ".").parse::<f64>() {
                Ok(n) if n.is_finite() => Value::Number(n),
                _ => return Err(err(ErrorKind::BadNumber(s.to_string()))),
            },
            other => return Err(err(ErrorKind::BadNumber(other.display(lang)))),
        },
        Builtin::Text => Value::text(&args[0].display(lang)),
        Builtin::Random => {
            let (a, b) = (number_arg(&args[0], span)?, number_arg(&args[1], span)?);
            let (lo, hi) = (a.min(b).ceil(), a.max(b).floor());
            if lo >= hi {
                Value::Number(lo)
            } else {
                let size = (hi - lo) as u64 + 1;
                Value::Number(lo + (next_random(rng) % size) as f64)
            }
        }
        Builtin::Round => Value::Number(number_arg(&args[0], span)?.round()),
        Builtin::Append => match &args[0] {
            Value::List(items) => {
                items.borrow_mut().push(args[1].clone());
                Value::Nothing
            }
            other => return Err(err(ErrorKind::NotIndexable(other.ty()))),
        },
    })
}
