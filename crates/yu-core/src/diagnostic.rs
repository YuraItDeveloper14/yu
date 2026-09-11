use crate::draw::PALETTE;
use crate::lang::Lang;
use crate::span::{line_col, line_text, Span};
use crate::value::format_number;

/// Type names used in messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Number,
    Text,
    Bool,
    Nothing,
    List,
    Function,
}

impl Ty {
    pub fn name(self, lang: Lang) -> &'static str {
        match self {
            Ty::Number => lang.pick("число", "a number"),
            Ty::Text => lang.pick("текст", "text"),
            Ty::Bool => lang.pick("так/ні", "true/false"),
            Ty::Nothing => lang.pick("нічого", "nothing"),
            Ty::List => lang.pick("список", "a list"),
            Ty::Function => lang.pick("функція", "a function"),
        }
    }
}

/// What the parser met instead of what it wanted.
#[derive(Debug, Clone, PartialEq)]
pub enum TokDesc {
    Text(String),
    EndOfLine,
    EndOfFile,
    Indent,
    Dedent,
}

impl TokDesc {
    fn show(&self, lang: Lang) -> String {
        match self {
            TokDesc::Text(s) => quote(s, lang),
            TokDesc::EndOfLine => lang.pick("кінець рядка", "end of line").into(),
            TokDesc::EndOfFile => lang.pick("кінець програми", "end of program").into(),
            TokDesc::Indent => lang.pick("зайвий відступ", "extra indentation").into(),
            TokDesc::Dedent => lang.pick("кінець блоку", "end of block").into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    UnexpectedChar(char),
    UnterminatedText,
    BadEscape(char),
    MixedIndent,
    BadDedent,
    Expected {
        uk: &'static str,
        en: &'static str,
    },
    UnexpectedToken(TokDesc),
    KeywordAsName(String),
    BuiltinAsName(String),
    InvalidTarget,
    OutsideLoop(String),
    OutsideFunction(String),
    UnknownName {
        name: String,
        suggestion: Option<String>,
    },
    TypeMismatch {
        op: String,
        left: Ty,
        right: Ty,
    },
    ExpectedNumber(Ty),
    BadCount,
    BadStep,
    NotCallable(Ty),
    ArgCount {
        name: String,
        expected: usize,
        got: usize,
    },
    NotIndexable(Ty),
    BadIndex,
    IndexOutOfRange {
        index: i64,
        len: usize,
    },
    DivisionByZero,
    BadNumber(String),
    RecursionTooDeep,
    TooLong,
    UnknownColor {
        name: String,
        suggestion: Option<String>,
    },
    ColorNeedsQuotes(String),
    NegativeSize(f64),
    BadCanvas,
    FillNotStarted,
    TooManyShapes,
}

/// «…» in Ukrainian, '…' in English.
pub fn quote(s: &str, lang: Lang) -> String {
    match lang {
        Lang::Uk => format!("«{s}»"),
        Lang::En => format!("'{s}'"),
    }
}

/// Ukrainian noun form for a count: 1 аргумент, 2 аргументи, 5 аргументів.
pub fn plural_uk<'a>(n: u64, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let (d, h) = (n % 10, n % 100);
    if d == 1 && h != 11 {
        one
    } else if (2..=4).contains(&d) && !(12..=14).contains(&h) {
        few
    } else {
        many
    }
}

impl ErrorKind {
    pub fn message(&self, lang: Lang) -> String {
        use ErrorKind::*;
        let q = |s: &str| quote(s, lang);
        let uk = lang == Lang::Uk;
        match self {
            UnexpectedChar(c) => format!(
                "{} {}",
                lang.pick("незрозумілий символ", "unexpected character"),
                q(&c.to_string())
            ),
            UnterminatedText => lang
                .pick(
                    "текст не закрито лапками \"",
                    "text is missing its closing quote \"",
                )
                .into(),
            BadEscape(c) => format!(
                "{} {}",
                lang.pick(
                    "невідома послідовність у тексті:",
                    "unknown escape in text:"
                ),
                q(&format!("\\{c}"))
            ),
            MixedIndent => lang
                .pick(
                    "у відступах змішано табуляцію й пробіли",
                    "indentation mixes tabs and spaces",
                )
                .into(),
            BadDedent => lang
                .pick(
                    "відступ не збігається з жодним зовнішнім рівнем",
                    "this indentation does not match any outer level",
                )
                .into(),
            Expected { uk: u, en: e } => {
                if uk {
                    format!("тут потрібно: {u}")
                } else {
                    format!("expected {e}")
                }
            }
            UnexpectedToken(d) => {
                if uk {
                    format!("тут не очікувалося: {}", d.show(lang))
                } else {
                    format!("unexpected {}", d.show(lang))
                }
            }
            KeywordAsName(w) => {
                if uk {
                    format!("{} — слово мови, його не можна брати за назву", q(w))
                } else {
                    format!("{} is a reserved word and can't be used as a name", q(w))
                }
            }
            BuiltinAsName(w) => {
                if uk {
                    format!("{} — вбудована команда, її не можна перезаписати", q(w))
                } else {
                    format!("{} is a built-in command and can't be reassigned", q(w))
                }
            }
            InvalidTarget => lang
                .pick(
                    "ліворуч від «=» має бути назва змінної або елемент списку",
                    "left of '=' must be a variable name or a list item",
                )
                .into(),
            OutsideLoop(w) => {
                if uk {
                    format!("{} можна писати лише всередині циклу", q(w))
                } else {
                    format!("{} only works inside a loop", q(w))
                }
            }
            OutsideFunction(w) => {
                if uk {
                    format!("{} можна писати лише всередині функції", q(w))
                } else {
                    format!("{} only works inside a function", q(w))
                }
            }
            UnknownName { name, .. } => format!(
                "{} {}",
                lang.pick("невідома назва", "unknown name"),
                q(name)
            ),
            TypeMismatch { op, left, right } => {
                if uk {
                    format!(
                        "не можна виконати {} для: {} і {}",
                        q(op),
                        left.name(lang),
                        right.name(lang)
                    )
                } else {
                    format!(
                        "can't apply {} to {} and {}",
                        q(op),
                        left.name(lang),
                        right.name(lang)
                    )
                }
            }
            ExpectedNumber(t) => {
                if uk {
                    format!("тут потрібне число, а маємо: {}", t.name(lang))
                } else {
                    format!("expected a number, got {}", t.name(lang))
                }
            }
            BadCount => lang
                .pick(
                    "кількість повторів має бути цілим числом, не меншим за 0",
                    "the repeat count must be a whole number, 0 or more",
                )
                .into(),
            BadStep => lang
                .pick("крок не може бути нулем", "the step can't be zero")
                .into(),
            NotCallable(t) => {
                if uk {
                    format!("{} не можна викликати як функцію", t.name(lang))
                } else {
                    format!("{} can't be called like a function", t.name(lang))
                }
            }
            ArgCount {
                name,
                expected,
                got,
            } => {
                if uk {
                    let noun = plural_uk(*expected as u64, "аргумент", "аргументи", "аргументів");
                    format!("{} чекає {expected} {noun}, а отримала {got}", q(name))
                } else {
                    let noun = if *expected == 1 {
                        "argument"
                    } else {
                        "arguments"
                    };
                    format!("{} takes {expected} {noun} but got {got}", q(name))
                }
            }
            NotIndexable(t) => {
                if uk {
                    format!("{} не має елементів за номерами", t.name(lang))
                } else {
                    format!("{} has no numbered items", t.name(lang))
                }
            }
            BadIndex => lang
                .pick(
                    "номер елемента має бути цілим числом від 1",
                    "an item number must be a whole number from 1",
                )
                .into(),
            IndexOutOfRange { index, len } => {
                if uk {
                    format!("немає елемента з номером {index}: їх усього {len}")
                } else {
                    format!("there is no item {index}: there are only {len}")
                }
            }
            DivisionByZero => lang.pick("ділення на нуль", "division by zero").into(),
            BadNumber(s) => {
                if uk {
                    format!("не вдалося перетворити {} на число", q(s))
                } else {
                    format!("can't turn {} into a number", q(s))
                }
            }
            RecursionTooDeep => lang
                .pick(
                    "забагато вкладених викликів (понад 1000)",
                    "too many nested calls (over 1000)",
                )
                .into(),
            TooLong => lang
                .pick("програма працює занадто довго", "the program runs too long")
                .into(),
            UnknownColor { name, .. } => format!(
                "{} {}",
                lang.pick("невідомий колір", "unknown colour"),
                q(name)
            ),
            ColorNeedsQuotes(w) => {
                if uk {
                    format!("{} — це колір, його треба взяти в лапки", q(w))
                } else {
                    format!("{} is a colour and needs quotes", q(w))
                }
            }
            NegativeSize(n) => format!(
                "{} {}",
                lang.pick(
                    "розмір не може бути від'ємним:",
                    "a size can't be negative:"
                ),
                format_number(*n)
            ),
            BadCanvas => lang
                .pick(
                    "полотно може мати від 1 до 4000 точок з кожного боку",
                    "the canvas can be 1 to 4000 points on each side",
                )
                .into(),
            FillNotStarted => lang
                .pick("заливку ще не почато", "no fill has been started")
                .into(),
            TooManyShapes => lang
                .pick(
                    "забагато фігур (понад 50 000)",
                    "too many shapes (over 50,000)",
                )
                .into(),
        }
    }

    pub fn hint(&self, lang: Lang) -> Option<String> {
        use ErrorKind::*;
        let q = |s: &str| quote(s, lang);
        match self {
            UnknownName {
                suggestion: Some(s),
                ..
            }
            | UnknownColor {
                suggestion: Some(s),
                ..
            } => Some(format!(
                "{} {}?",
                lang.pick("Можливо, ти мав на увазі", "Did you mean"),
                q(s)
            )),
            UnknownName {
                name,
                suggestion: None,
            } => Some(match lang {
                Lang::Uk => format!("Змінна з'являється, коли їй щось присвоять: {name} = …"),
                Lang::En => format!("A variable appears once you assign to it: {name} = …"),
            }),
            KeywordAsName(w) => Some(format!(
                "{} {}",
                lang.pick(
                    "Вибери іншу назву, наприклад",
                    "Pick another name, for example"
                ),
                q(&format!("{w}_"))
            )),
            BuiltinAsName(_) => Some(lang.pick("Вибери іншу назву", "Pick another name").into()),
            RecursionTooDeep => Some(
                lang.pick(
                    "Можливо, функція викликає себе без кінця",
                    "The function may be calling itself forever",
                )
                .into(),
            ),
            TooLong | TooManyShapes => Some(
                lang.pick(
                    "Можливо, цикл ніколи не закінчується",
                    "A loop may never end",
                )
                .into(),
            ),
            UnknownColor {
                suggestion: None, ..
            } => {
                let names: Vec<String> = PALETTE.iter().map(|c| c.name(lang)).collect();
                Some(format!(
                    "{} {} {} \"#ff8800\"",
                    lang.pick("Кольори:", "Colours:"),
                    names.join(", "),
                    lang.pick("або", "or")
                ))
            }
            ColorNeedsQuotes(w) => Some(format!(
                "{} \"{w}\"",
                lang.pick("Напиши так:", "Write it like this:")
            )),
            FillNotStarted => Some(
                lang.pick("Спершу виклич почни_заливку()", "Call begin_fill() first")
                    .into(),
            ),
            _ => None,
        }
    }
}

/// An error with the place in the source it points at.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub kind: ErrorKind,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Diagnostic { kind, span }
    }

    /// Header line, the source line, a caret line and an optional hint.
    pub fn render(&self, src: &str, lang: Lang) -> String {
        let (line, col) = line_col(src, self.span.start);
        let text = line_text(src, line);
        let shown: String = text
            .chars()
            .map(|c| {
                if c == '\t' {
                    "    ".to_string()
                } else {
                    c.to_string()
                }
            })
            .collect();
        let before: usize = text
            .chars()
            .take(col - 1)
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum();
        let end = self.span.end.min(src.len()).max(self.span.start);
        let width = src[self.span.start..end]
            .chars()
            .take_while(|c| *c != '\n')
            .count()
            .max(1);
        let gutter = line.to_string();
        let pad = " ".repeat(gutter.len());
        let mut out = format!(
            "{} {line}: {}\n  {gutter} | {shown}\n  {pad} | {}{}",
            lang.pick("Помилка в рядку", "Error on line"),
            self.kind.message(lang),
            " ".repeat(before),
            "^".repeat(width)
        );
        if let Some(hint) = self.kind.hint(lang) {
            out.push_str("\n  ");
            out.push_str(&hint);
        }
        out
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.kind.message(Lang::Uk))
    }
}

impl std::error::Error for Diagnostic {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ukrainian_plurals() {
        let p = |n| plural_uk(n, "аргумент", "аргументи", "аргументів");
        assert_eq!(
            (p(1), p(2), p(5), p(11), p(12), p(21), p(22), p(25)),
            (
                "аргумент",
                "аргументи",
                "аргументів",
                "аргументів",
                "аргументів",
                "аргумент",
                "аргументи",
                "аргументів"
            )
        );
    }

    #[test]
    fn renders_unknown_name_with_hint_in_both_languages() {
        let src = "бал = 1\nскаж(\"Привіт\")";
        let at = src.find("скаж").unwrap();
        let d = Diagnostic::new(
            ErrorKind::UnknownName {
                name: "скаж".into(),
                suggestion: Some("скажи".into()),
            },
            Span::new(at, at + "скаж".len()),
        );
        assert_eq!(
            d.render(src, Lang::Uk),
            "Помилка в рядку 2: невідома назва «скаж»\n  2 | скаж(\"Привіт\")\n    | ^^^^\n  Можливо, ти мав на увазі «скажи»?"
        );
        assert_eq!(
            d.render(src, Lang::En),
            "Error on line 2: unknown name 'скаж'\n  2 | скаж(\"Привіт\")\n    | ^^^^\n  Did you mean 'скажи'?"
        );
    }

    #[test]
    fn arg_count_uses_plural_forms() {
        let k = ErrorKind::ArgCount {
            name: "квадрат".into(),
            expected: 1,
            got: 2,
        };
        assert_eq!(
            k.message(Lang::Uk),
            "«квадрат» чекає 1 аргумент, а отримала 2"
        );
        assert_eq!(k.message(Lang::En), "'квадрат' takes 1 argument but got 2");
    }

    #[test]
    fn caret_at_end_of_file_is_one_wide() {
        let src = "x = (";
        let d = Diagnostic::new(
            ErrorKind::UnexpectedToken(TokDesc::EndOfFile),
            Span::new(5, 5),
        );
        assert_eq!(
            d.render(src, Lang::Uk),
            "Помилка в рядку 1: тут не очікувалося: кінець програми\n  1 | x = (\n    |      ^"
        );
    }

    #[test]
    fn tabs_are_expanded_under_the_caret() {
        let src = "\tx";
        let d = Diagnostic::new(ErrorKind::UnexpectedChar('x'), Span::new(1, 2));
        assert!(d
            .render(src, Lang::En)
            .ends_with("  1 |     x\n    |     ^"));
    }

    #[test]
    fn colour_errors_say_what_to_write() {
        let k = ErrorKind::ColorNeedsQuotes("червоний".into());
        assert_eq!(
            k.message(Lang::Uk),
            "«червоний» — це колір, його треба взяти в лапки"
        );
        assert_eq!(
            k.message(Lang::En),
            "'червоний' is a colour and needs quotes"
        );
        assert_eq!(k.hint(Lang::Uk).unwrap(), "Напиши так: \"червоний\"");
        let k = ErrorKind::UnknownColor {
            name: "червний".into(),
            suggestion: Some("червоний".into()),
        };
        assert_eq!(k.message(Lang::Uk), "невідомий колір «червний»");
        assert_eq!(
            k.hint(Lang::Uk).unwrap(),
            "Можливо, ти мав на увазі «червоний»?"
        );
        let k = ErrorKind::UnknownColor {
            name: "x".into(),
            suggestion: None,
        };
        assert_eq!(
            k.hint(Lang::En).unwrap(),
            "Colours: red, orange, yellow, green, lightblue, blue, purple, pink, white, black, gray, brown or \"#ff8800\""
        );
    }

    #[test]
    fn drawing_errors_in_both_languages() {
        assert_eq!(
            ErrorKind::NegativeSize(-5.0).message(Lang::Uk),
            "розмір не може бути від'ємним: -5"
        );
        assert_eq!(
            ErrorKind::NegativeSize(-2.5).message(Lang::En),
            "a size can't be negative: -2.5"
        );
        assert_eq!(
            ErrorKind::BadCanvas.message(Lang::En),
            "the canvas can be 1 to 4000 points on each side"
        );
        assert_eq!(
            ErrorKind::FillNotStarted.hint(Lang::Uk).unwrap(),
            "Спершу виклич почни_заливку()"
        );
        assert_eq!(
            ErrorKind::TooManyShapes.message(Lang::Uk),
            "забагато фігур (понад 50 000)"
        );
        assert_eq!(
            ErrorKind::TooManyShapes.hint(Lang::En).unwrap(),
            "A loop may never end"
        );
    }
}
