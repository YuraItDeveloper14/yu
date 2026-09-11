use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// A scope shared by the code that can see it.
pub type Env = Rc<RefCell<Scope>>;

#[derive(Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

impl Scope {
    pub fn global() -> Env {
        Rc::new(RefCell::new(Scope::default()))
    }

    pub fn child(parent: &Env) -> Env {
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: Some(parent.clone()),
        }))
    }
}

pub fn lookup(env: &Env, name: &str) -> Option<Value> {
    let scope = env.borrow();
    match scope.vars.get(name) {
        Some(v) => Some(v.clone()),
        None => scope.parent.as_ref().and_then(|p| lookup(p, name)),
    }
}

pub fn define(env: &Env, name: &str, value: Value) {
    env.borrow_mut().vars.insert(name.to_string(), value);
}

/// Updates the nearest scope that already has `name`, otherwise creates it in `env`.
pub fn assign(env: &Env, name: &str, value: Value) {
    let mut current = Some(env.clone());
    while let Some(scope) = current {
        if let Some(slot) = scope.borrow_mut().vars.get_mut(name) {
            *slot = value;
            return;
        }
        current = scope.borrow().parent.clone();
    }
    define(env, name, value);
}

/// Every name visible from `env` (for "did you mean").
pub fn names(env: &Env) -> Vec<String> {
    let scope = env.borrow();
    let mut out: Vec<String> = scope.vars.keys().cloned().collect();
    if let Some(parent) = &scope.parent {
        out.extend(names(parent));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_updates_the_nearest_existing_variable() {
        let global = Scope::global();
        define(&global, "бал", Value::Number(0.0));
        let local = Scope::child(&global);
        assign(&local, "бал", Value::Number(5.0));
        assign(&local, "нова", Value::Number(1.0));
        assert_eq!(lookup(&global, "бал"), Some(Value::Number(5.0)));
        assert_eq!(lookup(&global, "нова"), None);
        assert_eq!(lookup(&local, "нова"), Some(Value::Number(1.0)));
    }

    #[test]
    fn names_lists_every_visible_variable() {
        let global = Scope::global();
        define(&global, "a", Value::Nothing);
        let local = Scope::child(&global);
        define(&local, "b", Value::Nothing);
        let mut names = names(&local);
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
