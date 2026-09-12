use crate::diagnostic::{Diagnostic, ErrorKind};
use crate::draw::{parse_color, Drawing, Rgb};
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
    Canvas,
    Background,
    Color,
    Thickness,
    Circle,
    Rect,
    Line,
    Label,
    Forward,
    Back,
    Right,
    Left,
    PenUp,
    PenDown,
    BeginFill,
    EndFill,
}

/// Ukrainian and English name of every built-in.
pub const NAMES: [(Builtin, &str, &str); 24] = [
    (Builtin::Say, "скажи", "say"),
    (Builtin::Ask, "запитай", "ask"),
    (Builtin::Length, "довжина", "length"),
    (Builtin::Number, "число", "number"),
    (Builtin::Text, "текст", "text"),
    (Builtin::Random, "випадкове", "random"),
    (Builtin::Round, "округли", "round"),
    (Builtin::Append, "додай", "append"),
    (Builtin::Canvas, "полотно", "canvas"),
    (Builtin::Background, "фон", "background"),
    (Builtin::Color, "колір", "color"),
    (Builtin::Thickness, "товщина", "thickness"),
    (Builtin::Circle, "коло", "circle"),
    (Builtin::Rect, "прямокутник", "rect"),
    (Builtin::Line, "лінія", "line"),
    (Builtin::Label, "напис", "label"),
    (Builtin::Forward, "вперед", "forward"),
    (Builtin::Back, "назад", "back"),
    (Builtin::Right, "праворуч", "right"),
    (Builtin::Left, "ліворуч", "left"),
    (Builtin::PenUp, "підніми_перо", "pen_up"),
    (Builtin::PenDown, "опусти_перо", "pen_down"),
    (Builtin::BeginFill, "почни_заливку", "begin_fill"),
    (Builtin::EndFill, "заверши_заливку", "end_fill"),
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

    /// How to call it: `коло(x, y, радіус)` or `circle(x, y, r)`.
    pub fn signature(self, lang: Lang) -> &'static str {
        let [uk, en, _, _] = self.help();
        lang.pick(uk, en)
    }

    /// One line on what it does.
    pub fn doc(self, lang: Lang) -> &'static str {
        let [_, _, uk, en] = self.help();
        lang.pick(uk, en)
    }

    /// Ukrainian and English signature, then Ukrainian and English description.
    fn help(self) -> [&'static str; 4] {
        match self {
            Builtin::Say => [
                "скажи(…)",
                "say(…)",
                "друкує значення через пробіл",
                "prints values separated by spaces",
            ],
            Builtin::Ask => [
                "запитай(питання)",
                "ask(question)",
                "чекає, поки введуть рядок, і повертає його",
                "waits for a line of input and returns it",
            ],
            Builtin::Length => [
                "довжина(x)",
                "length(x)",
                "кількість символів у тексті або елементів у списку",
                "the number of characters in text or items in a list",
            ],
            Builtin::Number => [
                "число(x)",
                "number(x)",
                "перетворює текст на число",
                "turns text into a number",
            ],
            Builtin::Text => [
                "текст(x)",
                "text(x)",
                "перетворює значення на текст",
                "turns a value into text",
            ],
            Builtin::Random => [
                "випадкове(a, b)",
                "random(a, b)",
                "випадкове ціле від a до b включно",
                "a random whole number from a to b",
            ],
            Builtin::Round => [
                "округли(x)",
                "round(x)",
                "найближче ціле число",
                "the nearest whole number",
            ],
            Builtin::Append => [
                "додай(список, x)",
                "append(list, x)",
                "додає x у кінець списку",
                "adds x to the end of a list",
            ],
            Builtin::Canvas => [
                "полотно(ш, в)",
                "canvas(w, h)",
                "розмір полотна, спершу 600×400",
                "the canvas size, 600×400 at first",
            ],
            Builtin::Background => [
                "фон(\"колір\")",
                "background(\"colour\")",
                "зафарбовує все полотно",
                "paints the whole canvas",
            ],
            Builtin::Color => [
                "колір(\"колір\")",
                "color(\"colour\")",
                "колір наступних фігур і ліній",
                "the colour of the next shapes and lines",
            ],
            Builtin::Thickness => [
                "товщина(n)",
                "thickness(n)",
                "товщина ліній",
                "the line width",
            ],
            Builtin::Circle => [
                "коло(x, y, радіус)",
                "circle(x, y, r)",
                "зафарбоване коло",
                "a filled circle",
            ],
            Builtin::Rect => [
                "прямокутник(x, y, ш, в)",
                "rect(x, y, w, h)",
                "зафарбований прямокутник",
                "a filled rectangle",
            ],
            Builtin::Line => [
                "лінія(x1, y1, x2, y2)",
                "line(x1, y1, x2, y2)",
                "пряма лінія",
                "a straight line",
            ],
            Builtin::Label => [
                "напис(текст, x, y)",
                "label(text, x, y)",
                "текст на полотні",
                "text on the canvas",
            ],
            Builtin::Forward => [
                "вперед(n)",
                "forward(n)",
                "черепашка йде вперед і малює",
                "the turtle moves ahead and draws",
            ],
            Builtin::Back => [
                "назад(n)",
                "back(n)",
                "черепашка йде назад",
                "the turtle moves back",
            ],
            Builtin::Right => [
                "праворуч(°)",
                "right(°)",
                "черепашка повертає за годинниковою стрілкою",
                "the turtle turns clockwise",
            ],
            Builtin::Left => [
                "ліворуч(°)",
                "left(°)",
                "черепашка повертає проти годинникової стрілки",
                "the turtle turns anticlockwise",
            ],
            Builtin::PenUp => [
                "підніми_перо()",
                "pen_up()",
                "черепашка йде, не малюючи",
                "the turtle moves without drawing",
            ],
            Builtin::PenDown => [
                "опусти_перо()",
                "pen_down()",
                "черепашка знову малює",
                "the turtle draws again",
            ],
            Builtin::BeginFill => [
                "почни_заливку()",
                "begin_fill()",
                "починає запам'ятовувати фігуру для заливки",
                "starts recording a shape to fill",
            ],
            Builtin::EndFill => [
                "заверши_заливку()",
                "end_fill()",
                "зафарбовує фігуру, яку обійшла черепашка",
                "fills the shape the turtle walked",
            ],
        }
    }

    /// Fewest and most arguments.
    fn arity(self) -> (usize, usize) {
        match self {
            Builtin::Say => (0, usize::MAX),
            Builtin::Ask => (0, 1),
            Builtin::Random | Builtin::Append | Builtin::Canvas => (2, 2),
            Builtin::Circle | Builtin::Label => (3, 3),
            Builtin::Rect | Builtin::Line => (4, 4),
            Builtin::PenUp | Builtin::PenDown | Builtin::BeginFill | Builtin::EndFill => (0, 0),
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
    drawing: &mut Drawing,
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
        _ => {
            draw_call(b, &args, span, lang, rng, drawing)?;
            Value::Nothing
        }
    })
}

/// The drawing built-ins; all of them return `нічого`.
fn draw_call(
    b: Builtin,
    args: &[Value],
    span: Span,
    lang: Lang,
    rng: &mut u64,
    d: &mut Drawing,
) -> Result<(), Diagnostic> {
    let err = |kind: ErrorKind| Diagnostic::new(kind, span);
    let n = |i: usize| number_arg(&args[i], span);
    match b {
        Builtin::Canvas => d.canvas(n(0)?, n(1)?).map_err(err),
        Builtin::Background => {
            let color = color_arg(&args[0], span, d.color, rng)?;
            d.background(color).map_err(err)
        }
        Builtin::Color => {
            d.color = color_arg(&args[0], span, d.color, rng)?;
            Ok(())
        }
        Builtin::Thickness => d.set_thickness(n(0)?).map_err(err),
        Builtin::Circle => d.circle(n(0)?, n(1)?, n(2)?).map_err(err),
        Builtin::Rect => d.rect(n(0)?, n(1)?, n(2)?, n(3)?).map_err(err),
        Builtin::Line => d.line(n(0)?, n(1)?, n(2)?, n(3)?).map_err(err),
        Builtin::Label => d.label(args[0].display(lang), n(1)?, n(2)?).map_err(err),
        Builtin::Forward => d.forward(n(0)?).map_err(err),
        Builtin::Back => d.forward(-n(0)?).map_err(err),
        Builtin::Right => {
            d.turn(n(0)?);
            Ok(())
        }
        Builtin::Left => {
            d.turn(-n(0)?);
            Ok(())
        }
        Builtin::PenUp => {
            d.pen(false);
            Ok(())
        }
        Builtin::PenDown => {
            d.pen(true);
            Ok(())
        }
        Builtin::BeginFill => {
            d.begin_fill();
            Ok(())
        }
        Builtin::EndFill => d.end_fill().map_err(err),
        _ => unreachable!("{b:?} is not a drawing built-in"),
    }
}

/// A colour argument: text that names a colour.
fn color_arg(v: &Value, span: Span, current: Rgb, rng: &mut u64) -> Result<Rgb, Diagnostic> {
    match v {
        Value::Text(s) => parse_color(s)
            .map(|c| c.pick(current, rng))
            .map_err(|kind| Diagnostic::new(kind, span)),
        other => Err(Diagnostic::new(ErrorKind::ExpectedText(other.ty()), span)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_explains_itself_in_both_languages() {
        for (b, uk, en) in NAMES {
            assert!(b.signature(Lang::Uk).starts_with(&format!("{uk}(")), "{uk}");
            assert!(b.signature(Lang::En).starts_with(&format!("{en}(")), "{en}");
            assert!(!b.doc(Lang::Uk).is_empty() && !b.doc(Lang::En).is_empty());
        }
        assert_eq!(Builtin::Circle.signature(Lang::Uk), "коло(x, y, радіус)");
        assert_eq!(Builtin::Circle.doc(Lang::En), "a filled circle");
    }
}
