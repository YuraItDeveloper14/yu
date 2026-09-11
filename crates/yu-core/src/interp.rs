use std::rc::Rc;

use crate::ast::{BinOp, Expr, Stmt, UnOp};
use crate::diagnostic::{Diagnostic, ErrorKind};
use crate::env::{self, Env, Scope};
use crate::host::Host;
use crate::lang::Lang;
use crate::span::Span;
use crate::value::{Closure, Value};

/// Settings for one run.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub lang: Lang,
    /// Loop iterations plus calls allowed before `TooLong` (the playground sets this).
    pub step_budget: Option<u64>,
    pub max_depth: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            lang: Lang::Uk,
            step_budget: None,
            max_depth: 1000,
        }
    }
}

/// What a statement tells the loop or function around it.
enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

type Res<T> = Result<T, Diagnostic>;

pub struct Interp<'h> {
    pub(crate) host: &'h mut dyn Host,
    globals: Env,
    pub(crate) opts: Options,
    depth: usize,
    steps: u64,
}

impl<'h> Interp<'h> {
    pub fn new(host: &'h mut dyn Host, globals: Env, opts: Options) -> Self {
        Interp {
            host,
            globals,
            opts,
            depth: 0,
            steps: 0,
        }
    }

    /// Runs a program in the global scope; returns the value of a final expression statement.
    pub fn run(&mut self, program: &[Stmt]) -> Res<Option<Value>> {
        let env = self.globals.clone();
        let mut last = None;
        for stmt in program {
            last = None;
            if let Stmt::Expr(e) = stmt {
                last = Some(self.eval(e, &env)?);
            } else {
                self.exec(stmt, &env)?;
            }
        }
        Ok(last)
    }

    fn tick(&mut self, span: Span) -> Res<()> {
        self.steps += 1;
        match self.opts.step_budget {
            Some(max) if self.steps > max => Err(Diagnostic::new(ErrorKind::TooLong, span)),
            _ => Ok(()),
        }
    }

    fn exec_block(&mut self, body: &[Stmt], env: &Env) -> Res<Flow> {
        for stmt in body {
            match self.exec(stmt, env)? {
                Flow::Normal => {}
                flow => return Ok(flow),
            }
        }
        Ok(Flow::Normal)
    }

    /// One pass through a loop body; `Some(flow)` means leave the loop with that flow.
    fn iteration(&mut self, body: &[Stmt], env: &Env, span: Span) -> Res<Option<Flow>> {
        self.tick(span)?;
        Ok(match self.exec_block(body, env)? {
            Flow::Break => Some(Flow::Normal),
            Flow::Return(v) => Some(Flow::Return(v)),
            Flow::Normal | Flow::Continue => None,
        })
    }

    fn exec(&mut self, stmt: &Stmt, env: &Env) -> Res<Flow> {
        match stmt {
            Stmt::Expr(e) => {
                self.eval(e, env)?;
            }
            Stmt::Assign { name, value, span } => {
                let v = self.eval(value, env)?;
                self.assign(name, v, *span, env)?;
            }
            Stmt::SetIndex {
                target,
                index,
                value,
                ..
            } => {
                let list = self.eval(target, env)?;
                let i = self.eval(index, env)?;
                let v = self.eval(value, env)?;
                match list {
                    Value::List(items) => {
                        let mut items = items.borrow_mut();
                        let k = position(&i, items.len(), index.span())?;
                        items[k] = v;
                    }
                    other => {
                        return Err(Diagnostic::new(
                            ErrorKind::NotIndexable(other.ty()),
                            target.span(),
                        ))
                    }
                }
            }
            Stmt::If {
                branches,
                otherwise,
            } => {
                for (cond, body) in branches {
                    if self.eval(cond, env)?.truthy() {
                        return self.exec_block(body, env);
                    }
                }
                if let Some(body) = otherwise {
                    return self.exec_block(body, env);
                }
            }
            Stmt::While { cond, body, span } => {
                while self.eval(cond, env)?.truthy() {
                    if let Some(flow) = self.iteration(body, env, *span)? {
                        return Ok(flow);
                    }
                }
            }
            Stmt::Repeat { count, body, span } => {
                let times = match self.eval(count, env)? {
                    Value::Number(x) if x >= 0.0 && x.fract() == 0.0 => x as u64,
                    Value::Number(_) => {
                        return Err(Diagnostic::new(ErrorKind::BadCount, count.span()))
                    }
                    other => {
                        return Err(Diagnostic::new(
                            ErrorKind::ExpectedNumber(other.ty()),
                            count.span(),
                        ))
                    }
                };
                for _ in 0..times {
                    if let Some(flow) = self.iteration(body, env, *span)? {
                        return Ok(flow);
                    }
                }
            }
            Stmt::ForRange {
                var,
                from,
                to,
                step,
                body,
                span,
            } => {
                let start = self.number(from, env)?;
                let end = self.number(to, env)?;
                let step_by = match step {
                    Some(s) => self.number(s, env)?,
                    None if start <= end => 1.0,
                    None => -1.0,
                };
                if step_by == 0.0 {
                    let at = step.as_ref().map_or(*span, |s| s.span());
                    return Err(Diagnostic::new(ErrorKind::BadStep, at));
                }
                let mut i = start;
                while (step_by > 0.0 && i <= end) || (step_by < 0.0 && i >= end) {
                    self.assign(var, Value::Number(i), *span, env)?;
                    if let Some(flow) = self.iteration(body, env, *span)? {
                        return Ok(flow);
                    }
                    i += step_by;
                }
            }
            Stmt::ForEach {
                var,
                iter,
                body,
                span,
            } => {
                let items: Vec<Value> = match self.eval(iter, env)? {
                    Value::List(list) => list.borrow().clone(),
                    Value::Text(s) => s.chars().map(|c| Value::text(&c.to_string())).collect(),
                    other => {
                        return Err(Diagnostic::new(
                            ErrorKind::NotIndexable(other.ty()),
                            iter.span(),
                        ))
                    }
                };
                for item in items {
                    self.assign(var, item, *span, env)?;
                    if let Some(flow) = self.iteration(body, env, *span)? {
                        return Ok(flow);
                    }
                }
            }
            Stmt::Function(def) => {
                let closure = Closure {
                    def: def.clone(),
                    env: env.clone(),
                };
                env::define(env, &def.name, Value::Function(Rc::new(closure)));
            }
            Stmt::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Nothing,
                };
                return Ok(Flow::Return(v));
            }
            Stmt::Break(_) => return Ok(Flow::Break),
            Stmt::Continue(_) => return Ok(Flow::Continue),
        }
        Ok(Flow::Normal)
    }

    fn assign(&mut self, name: &str, value: Value, _span: Span, env: &Env) -> Res<()> {
        env::assign(env, name, value);
        Ok(())
    }

    fn lookup(&self, name: &str, span: Span, env: &Env) -> Res<Value> {
        env::lookup(env, name).ok_or_else(|| {
            Diagnostic::new(
                ErrorKind::UnknownName {
                    name: name.into(),
                    suggestion: None,
                },
                span,
            )
        })
    }

    fn number(&mut self, e: &Expr, env: &Env) -> Res<f64> {
        match self.eval(e, env)? {
            Value::Number(n) => Ok(n),
            other => Err(Diagnostic::new(
                ErrorKind::ExpectedNumber(other.ty()),
                e.span(),
            )),
        }
    }

    fn eval(&mut self, e: &Expr, env: &Env) -> Res<Value> {
        Ok(match e {
            Expr::Number(n, _) => Value::Number(*n),
            Expr::Text(s, _) => Value::Text(s.clone()),
            Expr::Bool(b, _) => Value::Bool(*b),
            Expr::Nothing(_) => Value::Nothing,
            Expr::Name(name, span) => self.lookup(name, *span, env)?,
            Expr::List(items, _) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.eval(item, env)?);
                }
                Value::list(out)
            }
            Expr::Unary {
                op: UnOp::Not,
                expr,
                ..
            } => Value::Bool(!self.eval(expr, env)?.truthy()),
            Expr::Unary {
                op: UnOp::Neg,
                expr,
                span,
            } => match self.eval(expr, env)? {
                Value::Number(n) => Value::Number(-n),
                other => {
                    return Err(Diagnostic::new(
                        ErrorKind::ExpectedNumber(other.ty()),
                        *span,
                    ))
                }
            },
            Expr::Binary {
                op: BinOp::And,
                left,
                right,
                ..
            } => Value::Bool(self.eval(left, env)?.truthy() && self.eval(right, env)?.truthy()),
            Expr::Binary {
                op: BinOp::Or,
                left,
                right,
                ..
            } => Value::Bool(self.eval(left, env)?.truthy() || self.eval(right, env)?.truthy()),
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let l = self.eval(left, env)?;
                let r = self.eval(right, env)?;
                binary(*op, l, r, *span, self.opts.lang)?
            }
            Expr::Call { callee, args, span } => self.call(callee, args, *span, env)?,
            Expr::Index { target, index, .. } => {
                let t = self.eval(target, env)?;
                let i = self.eval(index, env)?;
                get_index(&t, &i, target.span(), index.span())?
            }
        })
    }

    fn call(&mut self, callee: &Expr, args: &[Expr], span: Span, env: &Env) -> Res<Value> {
        let f = self.eval(callee, env)?;
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(a, env)?);
        }
        match f {
            Value::Function(closure) => self.call_closure(&closure, values, span),
            other => Err(Diagnostic::new(
                ErrorKind::NotCallable(other.ty()),
                callee.span(),
            )),
        }
    }

    fn call_closure(&mut self, closure: &Closure, args: Vec<Value>, span: Span) -> Res<Value> {
        let def = &closure.def;
        if args.len() != def.params.len() {
            let kind = ErrorKind::ArgCount {
                name: def.name.clone(),
                expected: def.params.len(),
                got: args.len(),
            };
            return Err(Diagnostic::new(kind, span));
        }
        if self.depth >= self.opts.max_depth {
            return Err(Diagnostic::new(ErrorKind::RecursionTooDeep, span));
        }
        self.tick(span)?;
        let local = Scope::child(&closure.env);
        for (param, value) in def.params.iter().zip(args) {
            env::define(&local, param, value);
        }
        self.depth += 1;
        let flow = self.exec_block(&def.body, &local);
        self.depth -= 1;
        Ok(match flow? {
            Flow::Return(v) => v,
            _ => Value::Nothing,
        })
    }
}

fn binary(op: BinOp, l: Value, r: Value, span: Span, lang: Lang) -> Res<Value> {
    let mismatch = |l: &Value, r: &Value| {
        Diagnostic::new(
            ErrorKind::TypeMismatch {
                op: op.symbol().into(),
                left: l.ty(),
                right: r.ty(),
            },
            span,
        )
    };
    Ok(match op {
        BinOp::Eq => Value::Bool(l == r),
        BinOp::Ne => Value::Bool(l != r),
        BinOp::Add => match (&l, &r) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Text(_), _) | (_, Value::Text(_)) => {
                Value::text(&format!("{}{}", l.display(lang), r.display(lang)))
            }
            (Value::List(a), Value::List(b)) => {
                let mut items = a.borrow().clone();
                items.extend(b.borrow().iter().cloned());
                Value::list(items)
            }
            _ => return Err(mismatch(&l, &r)),
        },
        BinOp::Mul => match (&l, &r) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            (Value::Text(s), Value::Number(k)) | (Value::Number(k), Value::Text(s))
                if *k >= 0.0 && k.fract() == 0.0 =>
            {
                Value::text(&s.repeat(*k as usize))
            }
            _ => return Err(mismatch(&l, &r)),
        },
        BinOp::Sub | BinOp::Div | BinOp::Rem => match (&l, &r) {
            (Value::Number(a), Value::Number(b)) => match op {
                BinOp::Sub => Value::Number(a - b),
                _ if *b == 0.0 => return Err(Diagnostic::new(ErrorKind::DivisionByZero, span)),
                BinOp::Div => Value::Number(a / b),
                _ => Value::Number(a.rem_euclid(*b)),
            },
            _ => return Err(mismatch(&l, &r)),
        },
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ord = match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
                (Value::Text(a), Value::Text(b)) => Some(a.cmp(b)),
                _ => None,
            };
            let ord = ord.ok_or_else(|| mismatch(&l, &r))?;
            Value::Bool(match op {
                BinOp::Lt => ord.is_lt(),
                BinOp::Le => ord.is_le(),
                BinOp::Gt => ord.is_gt(),
                _ => ord.is_ge(),
            })
        }
        BinOp::And | BinOp::Or => unreachable!("and/or short-circuit in eval"),
    })
}

/// A 1-based item number turned into a 0-based position.
fn position(i: &Value, len: usize, span: Span) -> Res<usize> {
    match i {
        Value::Number(n) if n.fract() == 0.0 && *n >= 1.0 => {
            if (*n as usize) <= len {
                Ok(*n as usize - 1)
            } else {
                Err(Diagnostic::new(
                    ErrorKind::IndexOutOfRange {
                        index: *n as i64,
                        len,
                    },
                    span,
                ))
            }
        }
        Value::Number(_) => Err(Diagnostic::new(ErrorKind::BadIndex, span)),
        other => Err(Diagnostic::new(ErrorKind::ExpectedNumber(other.ty()), span)),
    }
}

fn get_index(target: &Value, i: &Value, target_span: Span, index_span: Span) -> Res<Value> {
    match target {
        Value::List(items) => {
            let items = items.borrow();
            let k = position(i, items.len(), index_span)?;
            Ok(items[k].clone())
        }
        Value::Text(s) => {
            let chars: Vec<char> = s.chars().collect();
            let k = position(i, chars.len(), index_span)?;
            Ok(Value::text(&chars[k].to_string()))
        }
        other => Err(Diagnostic::new(
            ErrorKind::NotIndexable(other.ty()),
            target_span,
        )),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::diagnostic::Ty;
    use crate::lexer::lex;
    use crate::parser::parse;
    use std::collections::VecDeque;

    #[derive(Default)]
    pub(crate) struct Capture {
        pub out: Vec<String>,
        pub input: VecDeque<String>,
    }

    impl Host for Capture {
        fn print(&mut self, line: &str) {
            self.out.push(line.to_string());
        }
        fn ask(&mut self, _prompt: &str) -> Option<String> {
            self.input.pop_front()
        }
    }

    pub(crate) fn run_with(
        src: &str,
        opts: Options,
    ) -> (Result<Option<Value>, Diagnostic>, Env, Capture) {
        let globals = Scope::global();
        let mut host = Capture::default();
        let result = parse(src, &lex(src).unwrap())
            .and_then(|program| Interp::new(&mut host, globals.clone(), opts).run(&program));
        (result, globals, host)
    }

    /// Runs `src` and returns the global `name`.
    fn get(src: &str, name: &str) -> Value {
        let (result, globals, _) = run_with(src, Options::default());
        result.unwrap();
        env::lookup(&globals, name).unwrap()
    }

    fn err(src: &str) -> ErrorKind {
        run_with(src, Options::default()).0.unwrap_err().kind
    }

    fn n(x: f64) -> Value {
        Value::Number(x)
    }

    #[test]
    fn arithmetic_and_text() {
        assert_eq!(get("x = 2 + 3 * 4", "x"), n(14.0));
        assert_eq!(get("x = -7 % 3", "x"), n(2.0));
        assert_eq!(get("x = 7 / 2", "x"), n(3.5));
        assert_eq!(get("x = \"Бал: \" + 5", "x"), Value::text("Бал: 5"));
        assert_eq!(get("x = \"ab\" * 3", "x"), Value::text("ababab"));
        assert_eq!(get("x = [1] + [2]", "x"), Value::list(vec![n(1.0), n(2.0)]));
    }

    #[test]
    fn comparisons_and_logic() {
        assert_eq!(get("x = 1 < 2 і не (2 == 3)", "x"), Value::Bool(true));
        assert_eq!(get("x = \"а\" < \"б\"", "x"), Value::Bool(true));
        assert_eq!(get("x = [1, 2] == [1, 2]", "x"), Value::Bool(true));
        assert_eq!(get("x = 0 або \"\"", "x"), Value::Bool(false));
    }

    #[test]
    fn variables_update_the_nearest_scope() {
        let src = "бал = 0\nфункція додай_бал():\n    бал = бал + 1\nдодай_бал()\nдодай_бал()";
        assert_eq!(get(src, "бал"), n(2.0));
    }

    #[test]
    fn if_and_loops() {
        assert_eq!(
            get(
                "x = 0\nякщо x > 0:\n    y = 1\nінакше якщо x == 0:\n    y = 2\nінакше:\n    y = 3",
                "y"
            ),
            n(2.0)
        );
        assert_eq!(
            get("s = 0\nдля i від 1 до 10:\n    s = s + i", "s"),
            n(55.0)
        );
        assert_eq!(get("для i від 5 до 1:\n    last = i", "last"), n(1.0));
        assert_eq!(
            get("s = 0\nдля i від 0 до 10 крок 5:\n    s = s + i", "s"),
            n(15.0)
        );
        assert_eq!(get("k = 0\nповтори 4 рази:\n    k = k + 1", "k"), n(4.0));
        assert_eq!(
            get(
                "i = 0\nпоки так:\n    i = i + 1\n    якщо i == 3:\n        стоп",
                "i"
            ),
            n(3.0)
        );
        assert_eq!(
            get(
                "s = 0\nдля i від 1 до 5:\n    якщо i % 2 == 0:\n        далі\n    s = s + i",
                "s"
            ),
            n(9.0)
        );
    }

    #[test]
    fn lists_and_text_are_numbered_from_one() {
        assert_eq!(get("l = [10, 20, 30]\nx = l[1]", "x"), n(10.0));
        assert_eq!(
            get("l = [10, 20, 30]\nl[2] = 5\nx = l", "x"),
            Value::list(vec![n(10.0), n(5.0), n(30.0)])
        );
        assert_eq!(get("t = \"Юрій\"\nx = t[1]", "x"), Value::text("Ю"));
        assert_eq!(get("s = 0\nдля v у [1, 2, 3]:\n    s = s + v", "s"), n(6.0));
        assert_eq!(
            get("s = \"\"\nдля c в \"abc\":\n    s = c + s", "s"),
            Value::text("cba")
        );
    }

    #[test]
    fn functions_closures_and_recursion() {
        assert_eq!(
            get(
                "функція квадрат(x):\n    поверни x * x\ny = квадрат(7)",
                "y"
            ),
            n(49.0)
        );
        let fact = "функція факт(n):\n    якщо n <= 1:\n        поверни 1\n    поверни n * факт(n - 1)\ny = факт(10)";
        assert_eq!(get(fact, "y"), n(3628800.0));
        let counter = "function counter():\n    count = 0\n    function next():\n        count = count + 1\n        return count\n    return next\nc = counter()\nc()\ny = c()";
        assert_eq!(get(counter, "y"), n(2.0));
        assert_eq!(
            get("функція тиша():\n    x = 1\ny = тиша()", "y"),
            Value::Nothing
        );
    }

    #[test]
    fn runtime_errors() {
        assert_eq!(err("x = 1 / 0"), ErrorKind::DivisionByZero);
        assert_eq!(
            err("x = [1, 2][3]"),
            ErrorKind::IndexOutOfRange { index: 3, len: 2 }
        );
        assert_eq!(err("x = [1, 2][0]"), ErrorKind::BadIndex);
        assert_eq!(
            err("x = 1 - \"a\""),
            ErrorKind::TypeMismatch {
                op: "-".into(),
                left: Ty::Number,
                right: Ty::Text
            }
        );
        assert_eq!(
            err("x = y"),
            ErrorKind::UnknownName {
                name: "y".into(),
                suggestion: None
            }
        );
        assert_eq!(err("повтори 1.5 рази:\n    x = 1"), ErrorKind::BadCount);
        assert_eq!(
            err("для i від 1 до 3 крок 0:\n    x = 1"),
            ErrorKind::BadStep
        );
        assert_eq!(
            err("функція f(a):\n    поверни a\nf(1, 2)"),
            ErrorKind::ArgCount {
                name: "f".into(),
                expected: 1,
                got: 2
            }
        );
        assert_eq!(err("x = 5\nx()"), ErrorKind::NotCallable(Ty::Number));
    }

    #[test]
    fn endless_recursion_is_an_error_not_a_crash() {
        let kind = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(|| err("функція f(n):\n    поверни f(n + 1)\nf(1)"))
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(kind, ErrorKind::RecursionTooDeep);
    }

    #[test]
    fn step_budget_stops_endless_loops_at_the_loop() {
        let opts = Options {
            step_budget: Some(1000),
            ..Options::default()
        };
        let e = run_with("поки так:\n    x = 1", opts).0.unwrap_err();
        assert_eq!(e.kind, ErrorKind::TooLong);
        assert_eq!(e.span, Span::new(0, "поки".len()));
    }

    #[test]
    fn the_last_expression_is_returned() {
        assert_eq!(
            run_with("x = 2\nx * 3", Options::default()).0.unwrap(),
            Some(n(6.0))
        );
        assert_eq!(run_with("x = 2", Options::default()).0.unwrap(), None);
    }
}
