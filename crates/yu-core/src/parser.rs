use std::rc::Rc;

use crate::ast::{BinOp, Expr, Stmt, UnOp};
use crate::diagnostic::{Diagnostic, ErrorKind, TokDesc};
use crate::keywords::Keyword;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Builds the program's statements from the lexer's tokens.
pub fn parse(src: &str, tokens: &[Token]) -> Result<Vec<Stmt>, Diagnostic> {
    let mut p = Parser {
        src,
        tokens,
        pos: 0,
    };
    let mut program = Vec::new();
    while !p.at(&TokenKind::Eof) {
        program.push(p.statement()?);
    }
    Ok(program)
}

struct Parser<'a> {
    src: &'a str,
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &'a TokenKind {
        let tokens: &'a [Token] = self.tokens;
        &tokens[self.pos].kind
    }

    fn peek_at(&self, n: usize) -> &'a TokenKind {
        let tokens: &'a [Token] = self.tokens;
        &tokens[(self.pos + n).min(tokens.len() - 1)].kind
    }

    fn span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn at(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn at_kw(&self, k: Keyword) -> bool {
        *self.peek() == TokenKind::Kw(k)
    }

    fn advance(&mut self) -> Span {
        let span = self.span();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        span
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        let hit = self.at(kind);
        if hit {
            self.advance();
        }
        hit
    }

    /// The source text of the current token.
    fn word(&self) -> String {
        let s = self.span();
        self.src[s.start..s.end].to_string()
    }

    fn expected(&self, uk: &'static str, en: &'static str) -> Diagnostic {
        Diagnostic::new(ErrorKind::Expected { uk, en }, self.span())
    }

    fn expect(
        &mut self,
        kind: &TokenKind,
        uk: &'static str,
        en: &'static str,
    ) -> Result<Span, Diagnostic> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            Err(self.expected(uk, en))
        }
    }

    fn unexpected(&self) -> Diagnostic {
        let desc = match self.peek() {
            TokenKind::Newline => TokDesc::EndOfLine,
            TokenKind::Eof => TokDesc::EndOfFile,
            TokenKind::Indent => TokDesc::Indent,
            TokenKind::Dedent => TokDesc::Dedent,
            _ => TokDesc::Text(self.word()),
        };
        Diagnostic::new(ErrorKind::UnexpectedToken(desc), self.span())
    }

    fn statement(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek() {
            TokenKind::Indent => Err(self.unexpected()),
            _ => self.simple(Self::expr_or_assign),
        }
    }

    /// A one-line statement has to end its line.
    fn simple(&mut self, f: fn(&mut Self) -> Result<Stmt, Diagnostic>) -> Result<Stmt, Diagnostic> {
        let stmt = f(self)?;
        if self.eat(&TokenKind::Newline) || self.at(&TokenKind::Eof) || self.at(&TokenKind::Dedent)
        {
            Ok(stmt)
        } else {
            Err(self.unexpected())
        }
    }

    fn expr_or_assign(&mut self) -> Result<Stmt, Diagnostic> {
        // a keyword followed by `=` is an attempt to name a variable with it
        if matches!(self.peek(), TokenKind::Kw(_)) && *self.peek_at(1) == TokenKind::Assign {
            return Err(Diagnostic::new(
                ErrorKind::KeywordAsName(self.word()),
                self.span(),
            ));
        }
        let target = self.expression()?;
        if !self.at(&TokenKind::Assign) {
            return Ok(Stmt::Expr(target));
        }
        let eq = self.advance();
        let value = self.expression()?;
        let span = target.span().to(value.span());
        match target {
            Expr::Name(name, _) => Ok(Stmt::Assign { name, value, span }),
            Expr::Index { target, index, .. } => Ok(Stmt::SetIndex {
                target: *target,
                index: *index,
                value,
                span,
            }),
            _ => Err(Diagnostic::new(ErrorKind::InvalidTarget, eq)),
        }
    }

    pub(crate) fn expression(&mut self) -> Result<Expr, Diagnostic> {
        self.expr_bp(0)
    }

    /// Pratt loop: or 1 < and 2 < not 3 < comparisons 4 < + - 5 < * / % 6 < unary minus 7;
    /// calls and indexing bind tightest.
    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, Diagnostic> {
        let mut left = self.prefix()?;
        loop {
            let (op, bp) = match self.peek() {
                TokenKind::Kw(Keyword::Or) => (BinOp::Or, 1),
                TokenKind::Kw(Keyword::And) => (BinOp::And, 2),
                TokenKind::EqEq => (BinOp::Eq, 4),
                TokenKind::NotEq => (BinOp::Ne, 4),
                TokenKind::Less => (BinOp::Lt, 4),
                TokenKind::LessEq => (BinOp::Le, 4),
                TokenKind::Greater => (BinOp::Gt, 4),
                TokenKind::GreaterEq => (BinOp::Ge, 4),
                TokenKind::Plus => (BinOp::Add, 5),
                TokenKind::Minus => (BinOp::Sub, 5),
                TokenKind::Star => (BinOp::Mul, 6),
                TokenKind::Slash => (BinOp::Div, 6),
                TokenKind::Percent => (BinOp::Rem, 6),
                TokenKind::LParen => {
                    left = self.call(left)?;
                    continue;
                }
                TokenKind::LBracket => {
                    left = self.index(left)?;
                    continue;
                }
                _ => break,
            };
            if bp <= min_bp {
                break;
            }
            self.advance();
            let right = self.expr_bp(bp)?;
            let span = left.span().to(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn prefix(&mut self) -> Result<Expr, Diagnostic> {
        let span = self.span();
        let expr = match self.peek().clone() {
            TokenKind::Number(n) => {
                self.advance();
                Expr::Number(n, span)
            }
            TokenKind::Text(s) => {
                self.advance();
                Expr::Text(Rc::from(s.as_str()), span)
            }
            TokenKind::Name(n) => {
                self.advance();
                Expr::Name(n, span)
            }
            TokenKind::Kw(Keyword::True) | TokenKind::Kw(Keyword::False) => {
                let value = self.at_kw(Keyword::True);
                self.advance();
                Expr::Bool(value, span)
            }
            TokenKind::Kw(Keyword::Nothing) => {
                self.advance();
                Expr::Nothing(span)
            }
            TokenKind::Minus | TokenKind::Kw(Keyword::Not) => {
                let op = if self.at(&TokenKind::Minus) {
                    UnOp::Neg
                } else {
                    UnOp::Not
                };
                self.advance();
                let expr = self.expr_bp(if op == UnOp::Neg { 7 } else { 3 })?;
                Expr::Unary {
                    op,
                    span: span.to(expr.span()),
                    expr: Box::new(expr),
                }
            }
            TokenKind::LParen => {
                self.advance();
                let inner = self.expression()?;
                self.expect(&TokenKind::RParen, "дужка «)»", "')'")?;
                inner
            }
            TokenKind::LBracket => {
                self.advance();
                let items = self.items(&TokenKind::RBracket)?;
                let end =
                    self.expect(&TokenKind::RBracket, "дужка «]» або кома", "']' or a comma")?;
                Expr::List(items, span.to(end))
            }
            _ => return Err(self.expected("значення або вираз", "a value or an expression")),
        };
        Ok(expr)
    }

    /// Comma-separated expressions up to `close` (a trailing comma is fine).
    fn items(&mut self, close: &TokenKind) -> Result<Vec<Expr>, Diagnostic> {
        let mut items = Vec::new();
        while !self.at(close) {
            items.push(self.expression()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        Ok(items)
    }

    fn call(&mut self, callee: Expr) -> Result<Expr, Diagnostic> {
        self.advance();
        let args = self.items(&TokenKind::RParen)?;
        let end = self.expect(&TokenKind::RParen, "дужка «)» або кома", "')' or a comma")?;
        Ok(Expr::Call {
            span: callee.span().to(end),
            callee: Box::new(callee),
            args,
        })
    }

    fn index(&mut self, target: Expr) -> Result<Expr, Diagnostic> {
        self.advance();
        let index = self.expression()?;
        let end = self.expect(&TokenKind::RBracket, "дужка «]»", "']'")?;
        Ok(Expr::Index {
            span: target.span().to(end),
            target: Box::new(target),
            index: Box::new(index),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    pub(crate) fn parse_src(src: &str) -> Result<Vec<Stmt>, Diagnostic> {
        parse(src, &lex(src)?)
    }

    fn num(n: f64) -> String {
        if n.fract() == 0.0 {
            format!("{}", n as i64)
        } else {
            n.to_string()
        }
    }

    pub(crate) fn sexp(e: &Expr) -> String {
        match e {
            Expr::Number(n, _) => num(*n),
            Expr::Text(s, _) => format!("{s:?}"),
            Expr::Bool(b, _) => b.to_string(),
            Expr::Nothing(_) => "nothing".into(),
            Expr::Name(n, _) => n.clone(),
            Expr::List(items, _) => {
                format!("[{}]", items.iter().map(sexp).collect::<Vec<_>>().join(" "))
            }
            Expr::Unary { op, expr, .. } => {
                format!(
                    "({} {})",
                    if *op == UnOp::Neg { "-" } else { "not" },
                    sexp(expr)
                )
            }
            Expr::Binary {
                op, left, right, ..
            } => format!("({} {} {})", op.symbol(), sexp(left), sexp(right)),
            Expr::Call { callee, args, .. } => format!(
                "(call {}{})",
                sexp(callee),
                args.iter()
                    .map(|a| format!(" {}", sexp(a)))
                    .collect::<String>()
            ),
            Expr::Index { target, index, .. } => {
                format!("(index {} {})", sexp(target), sexp(index))
            }
        }
    }

    fn block(b: &[Stmt]) -> String {
        format!("{{{}}}", b.iter().map(show).collect::<Vec<_>>().join(" "))
    }

    pub(crate) fn show(s: &Stmt) -> String {
        match s {
            Stmt::Expr(e) => sexp(e),
            Stmt::Assign { name, value, .. } => format!("(= {name} {})", sexp(value)),
            Stmt::SetIndex {
                target,
                index,
                value,
                ..
            } => {
                format!("(set {} {} {})", sexp(target), sexp(index), sexp(value))
            }
            Stmt::If {
                branches,
                otherwise,
            } => {
                let mut out = String::from("(if");
                for (cond, body) in branches {
                    out += &format!(" {} {}", sexp(cond), block(body));
                }
                if let Some(body) = otherwise {
                    out += &format!(" else {}", block(body));
                }
                out + ")"
            }
            Stmt::While { cond, body, .. } => format!("(while {} {})", sexp(cond), block(body)),
            Stmt::Repeat { count, body, .. } => format!("(repeat {} {})", sexp(count), block(body)),
            Stmt::ForRange {
                var,
                from,
                to,
                step,
                body,
                ..
            } => format!(
                "(for {var} {} {}{} {})",
                sexp(from),
                sexp(to),
                step.as_ref()
                    .map(|s| format!(" {}", sexp(s)))
                    .unwrap_or_default(),
                block(body)
            ),
            Stmt::ForEach {
                var, iter, body, ..
            } => format!("(each {var} {} {})", sexp(iter), block(body)),
            Stmt::Function(f) => format!(
                "(fn {} ({}) {})",
                f.name,
                f.params.join(" "),
                block(&f.body)
            ),
            Stmt::Return { value: Some(v), .. } => format!("(return {})", sexp(v)),
            Stmt::Return { value: None, .. } => "(return)".into(),
            Stmt::Break(_) => "break".into(),
            Stmt::Continue(_) => "continue".into(),
        }
    }

    pub(crate) fn one(src: &str) -> String {
        let program = parse_src(src).unwrap();
        assert_eq!(program.len(), 1, "{src}");
        show(&program[0])
    }

    #[test]
    fn precedence() {
        assert_eq!(one("1 + 2 * 3"), "(+ 1 (* 2 3))");
        assert_eq!(one("10 - 3 - 2"), "(- (- 10 3) 2)");
        assert_eq!(one("-a * b"), "(* (- a) b)");
        assert_eq!(one("(1 + 2) * 3"), "(* (+ 1 2) 3)");
    }

    #[test]
    fn logic_words_in_both_languages() {
        assert_eq!(one("не a == b або c і d"), "(or (not (== a b)) (and c d))");
        assert_eq!(
            one("not a == b or c and d"),
            "(or (not (== a b)) (and c d))"
        );
    }

    #[test]
    fn calls_indexing_and_lists() {
        assert_eq!(one("f(1, g(2))[3]"), "(index (call f 1 (call g 2)) 3)");
        assert_eq!(one("[1,\n 2,\n]"), "[1 2]");
        assert_eq!(one("скажи()"), "(call скажи)");
    }

    #[test]
    fn literals() {
        assert_eq!(one("так"), "true");
        assert_eq!(one("нічого"), "nothing");
        assert_eq!(one("\"hi\""), "\"hi\"");
    }

    #[test]
    fn assignment_targets() {
        assert_eq!(one("x = 1"), "(= x 1)");
        assert_eq!(one("a[1] = 2"), "(set a 1 2)");
        assert_eq!(
            parse_src("5 = x").unwrap_err().kind,
            ErrorKind::InvalidTarget
        );
        assert_eq!(
            parse_src("до = 5").unwrap_err().kind,
            ErrorKind::KeywordAsName("до".into())
        );
    }

    #[test]
    fn expression_errors() {
        assert!(matches!(
            parse_src("x = 1 +").unwrap_err().kind,
            ErrorKind::Expected { .. }
        ));
        assert!(matches!(
            parse_src("скажи(1").unwrap_err().kind,
            ErrorKind::Expected { .. }
        ));
        assert_eq!(
            parse_src("x = 1 2").unwrap_err().kind,
            ErrorKind::UnexpectedToken(TokDesc::Text("2".into()))
        );
        assert_eq!(
            parse_src("    x = 1").unwrap_err().kind,
            ErrorKind::UnexpectedToken(TokDesc::Indent)
        );
    }
}
