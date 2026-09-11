use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::FnDef;
use crate::diagnostic::Ty;
use crate::env::Env;
use crate::lang::Lang;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Text(Rc<str>),
    Bool(bool),
    Nothing,
    List(Rc<RefCell<Vec<Value>>>),
    Function(Rc<Closure>),
}

/// A function together with the scope it was defined in.
pub struct Closure {
    pub def: Rc<FnDef>,
    pub env: Env,
}

/// Whole numbers without `.0`; everything else as Rust prints it.
pub fn format_number(n: f64) -> String {
    if n.is_infinite() {
        return if n > 0.0 { "∞".into() } else { "-∞".into() };
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

impl Value {
    pub fn text(s: &str) -> Value {
        Value::Text(Rc::from(s))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(items)))
    }

    pub fn ty(&self) -> Ty {
        match self {
            Value::Number(_) => Ty::Number,
            Value::Text(_) => Ty::Text,
            Value::Bool(_) => Ty::Bool,
            Value::Nothing => Ty::Nothing,
            Value::List(_) => Ty::List,
            Value::Function(_) => Ty::Function,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nothing => false,
            Value::Number(n) => *n != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::List(items) => !items.borrow().is_empty(),
            Value::Function(_) => true,
        }
    }

    /// How `скажи` shows the value.
    pub fn display(&self, lang: Lang) -> String {
        match self {
            Value::Number(n) => format_number(*n),
            Value::Text(s) => s.to_string(),
            Value::Bool(b) => if *b {
                lang.pick("так", "true")
            } else {
                lang.pick("ні", "false")
            }
            .into(),
            Value::Nothing => lang.pick("нічого", "nothing").into(),
            Value::List(items) => {
                let items: Vec<String> = items.borrow().iter().map(|v| v.repr(lang)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Function(f) => format!("{} {}", lang.pick("функція", "function"), f.def.name),
        }
    }

    /// Like `display`, but texts keep their quotes (used inside lists).
    fn repr(&self, lang: Lang) -> String {
        match self {
            Value::Text(s) => format!("\"{s}\""),
            other => other.display(lang),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nothing, Value::Nothing) => true,
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.repr(Lang::En))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_without_a_trailing_zero() {
        assert_eq!(format_number(3.0), "3");
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(-0.0), "0");
        assert_eq!(format_number(1e20), "100000000000000000000");
    }

    #[test]
    fn display_in_both_languages() {
        let l = Value::list(vec![
            Value::Number(1.0),
            Value::text("а"),
            Value::Bool(true),
            Value::Nothing,
        ]);
        assert_eq!(l.display(Lang::Uk), "[1, \"а\", так, нічого]");
        assert_eq!(l.display(Lang::En), "[1, \"а\", true, nothing]");
        assert_eq!(Value::text("Юрій").display(Lang::Uk), "Юрій");
    }

    #[test]
    fn truthiness() {
        for v in [
            Value::Bool(false),
            Value::Nothing,
            Value::Number(0.0),
            Value::text(""),
            Value::list(vec![]),
        ] {
            assert!(!v.truthy(), "{v:?}");
        }
        assert!(Value::Number(2.0).truthy());
        assert!(Value::text("0").truthy());
    }

    #[test]
    fn lists_compare_by_content() {
        let a = Value::list(vec![Value::Number(1.0)]);
        let b = Value::list(vec![Value::Number(1.0)]);
        assert_eq!(a, b);
        assert_ne!(a, Value::Number(1.0));
    }
}
