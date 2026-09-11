# Yu Drawing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Yu programs draw shapes and turtle paths, and the core renders them as an animated SVG that `yu --svg` saves.

**Architecture:** `draw.rs` holds colours, the turtle and the list of `DrawCmd`s; `svg.rs` turns a `Drawing` into SVG text; sixteen built-ins in `builtins.rs` fill the drawing; `Session` keeps the drawing and the random state between runs; the CLI writes the SVG. The `Host` trait does not change.

**Tech Stack:** Rust 1.98 stable, edition 2021, no new dependencies. Python with Playwright (headless Chromium) only for the visual check.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-09-11-yu-drawing-design.md`; where it differs from the v1 design, it wins.
- `yu-core` stays free of dependencies and I/O.
- Every message exists in Ukrainian and English; Ukrainian is the default. Ukrainian quotes are «…», English '…' (`quote`).
- Defaults: canvas 600×400, white; colour black; thickness 2; turtle in the centre facing right, pen down.
- Limits: canvas 1–4000 points per side; at most 50 000 shapes (`MAX_SHAPES`).
- SVG numbers: two decimals, no trailing zeros, `-0` and non-finite as `0`. Animation step = min(0.15 s, 4 s ÷ places).
- cargo lives in `$HOME/.cargo/bin` (GNU toolchain): in Git Bash run `export PATH="$HOME/.cargo/bin:$PATH"` first. Use `set -o pipefail` whenever cargo output is piped.
- Before every commit: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings` and `cargo test --all` pass.
- Commit with `git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "<message>" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"`, staging explicit paths only.
- Branch: `m2-drawing`.

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `crates/yu-core/src/draw.rs` | create | `Rgb`, palette, colour parsing; `DrawCmd`, `Turtle`, `Drawing` |
| `crates/yu-core/src/svg.rs` | create | `render(&Drawing) -> String` and `num` |
| `crates/yu-core/src/diagnostic.rs` | modify | seven new `ErrorKind`s with messages and hints |
| `crates/yu-core/src/builtins.rs` | modify | sixteen drawing built-ins; `call` receives `&mut Drawing` |
| `crates/yu-core/src/interp.rs` | modify | the interpreter owns the run's `Drawing`; `ColorNeedsQuotes` in `lookup` |
| `crates/yu-core/src/lib.rs` | modify | modules, re-exports; `Session` keeps drawing and random state |
| `crates/yu-core/tests/drawing.rs` | create | drawing through the public API |
| `crates/yu-core/tests/golden.rs` | modify | examples checked against `.out`, `.svg` and `.err` through a `Session` |
| `crates/yu-cli/src/main.rs` | modify | `--svg`, saving or a hint, the REPL hint |
| `crates/yu-cli/src/color.rs` | modify | `Paint::hint` |
| `crates/yu-cli/tests/cli.rs` | modify | `--svg` tests |
| `examples/{house,sun,star,spiral,flag}.yu` and `.svg` | create | gallery and golden tests |
| `README.md` | modify | drawing section with the gallery |

---

### Task 1: Colours

**Files:**
- Create: `crates/yu-core/src/draw.rs`
- Modify: `crates/yu-core/src/lib.rs` (module list)
- Modify: `crates/yu-core/src/diagnostic.rs` (two error kinds, their messages, hints and a test)

**Interfaces:**
- Consumes: `builtins::next_random(&mut u64) -> u64`, `suggest::closest`, `Lang::pick`, `quote`.
- Produces:
  - `pub struct Rgb(pub u8, pub u8, pub u8)` with `pub fn hex(self) -> String` (`"#0057b7"`)
  - `pub struct Named` with `pub rgb: Rgb` and `pub fn name(&self, lang: Lang) -> String`
  - `pub const PALETTE: [Named; 12]`; the first eight are the bright colours
  - `pub enum ColorSpec { Exact(Rgb), Random }` with `pub fn pick(self, current: Rgb, rng: &mut u64) -> Rgb`
  - `pub fn parse_color(text: &str) -> Result<ColorSpec, ErrorKind>`
  - `pub fn is_color_word(word: &str) -> bool`
  - `ErrorKind::UnknownColor { name: String, suggestion: Option<String> }`, `ErrorKind::ColorNeedsQuotes(String)`

- [ ] **Step 1: Write the failing tests**

In `crates/yu-core/src/lib.rs` add `pub mod draw;` between `pub mod diagnostic;` and `pub mod env;`.

Create `crates/yu-core/src/draw.rs` with only the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgb = Rgb(0xef, 0x44, 0x44);

    #[test]
    fn colours_in_every_form_and_both_languages() {
        for word in ["червоний", "червона", "червоне", "червоні", "red", " Червона ", "RED"] {
            assert_eq!(parse_color(word), Ok(ColorSpec::Exact(RED)), "{word}");
        }
        let blue = ColorSpec::Exact(Rgb(0x00, 0x57, 0xb7));
        for word in ["синій", "синя", "синє", "сині", "blue"] {
            assert_eq!(parse_color(word), Ok(blue), "{word}");
        }
        assert_eq!(parse_color("grey"), parse_color("сіра"));
        assert_eq!(
            parse_color("#FF8800"),
            Ok(ColorSpec::Exact(Rgb(0xff, 0x88, 0x00)))
        );
        assert_eq!(parse_color("випадкова"), Ok(ColorSpec::Random));
        assert_eq!(parse_color("random"), Ok(ColorSpec::Random));
        assert_eq!(Rgb(0x00, 0x57, 0xb7).hex(), "#0057b7");
    }

    #[test]
    fn unknown_colours_suggest_the_nearest_name() {
        assert_eq!(
            parse_color("червний"),
            Err(ErrorKind::UnknownColor {
                name: "червний".into(),
                suggestion: Some("червоний".into())
            })
        );
        assert_eq!(
            parse_color("#12"),
            Err(ErrorKind::UnknownColor {
                name: "#12".into(),
                suggestion: None
            })
        );
    }

    #[test]
    fn random_picks_a_bright_colour_other_than_the_current_one() {
        let bright: Vec<Rgb> = PALETTE[..8].iter().map(|c| c.rgb).collect();
        let mut rng = 7;
        for _ in 0..100 {
            let c = ColorSpec::Random.pick(RED, &mut rng);
            assert_ne!(c, RED);
            assert!(bright.contains(&c));
        }
    }

    #[test]
    fn colour_words_without_quotes_are_recognised() {
        assert!(is_color_word("червоний"));
        assert!(is_color_word("Blue"));
        assert!(is_color_word("випадковий"));
        assert!(!is_color_word("бал"));
    }
}
```

At the end of the `tests` module in `crates/yu-core/src/diagnostic.rs` add:

```rust
    #[test]
    fn colour_errors_say_what_to_write() {
        let k = ErrorKind::ColorNeedsQuotes("червоний".into());
        assert_eq!(
            k.message(Lang::Uk),
            "«червоний» — це колір, його треба взяти в лапки"
        );
        assert_eq!(k.message(Lang::En), "'червоний' is a colour and needs quotes");
        assert_eq!(k.hint(Lang::Uk).unwrap(), "Напиши так: \"червоний\"");
        let k = ErrorKind::UnknownColor {
            name: "червний".into(),
            suggestion: Some("червоний".into()),
        };
        assert_eq!(k.message(Lang::Uk), "невідомий колір «червний»");
        assert_eq!(k.hint(Lang::Uk).unwrap(), "Можливо, ти мав на увазі «червоний»?");
        let k = ErrorKind::UnknownColor {
            name: "x".into(),
            suggestion: None,
        };
        assert_eq!(
            k.hint(Lang::En).unwrap(),
            "Colours: red, orange, yellow, green, lightblue, blue, purple, pink, white, black, gray, brown or \"#ff8800\""
        );
    }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p yu-core colour`
Expected: compile errors such as ``cannot find type `Rgb` in this scope`` and ``no variant named `ColorNeedsQuotes` ``.

- [ ] **Step 3: Implement colours**

Put this above the tests in `crates/yu-core/src/draw.rs`:

```rust
//! What a program draws: colours, the turtle and the shapes in the order they were drawn.

use crate::builtins::next_random;
use crate::diagnostic::ErrorKind;
use crate::lang::Lang;
use crate::suggest;

/// A colour as red, green and blue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// `#rrggbb`, the way SVG writes it.
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

/// A colour with a name; the Ukrainian stem takes a gender or plural ending.
pub struct Named {
    stem: &'static str,
    /// «синій» takes soft endings (-ій, -я, -є, -і); the rest take hard ones (-ий, -а, -е, -і).
    soft: bool,
    /// English names; the first one is used in messages.
    en: &'static [&'static str],
    pub rgb: Rgb,
}

impl Named {
    /// The masculine Ukrainian form, or the first English name.
    pub fn name(&self, lang: Lang) -> String {
        match lang {
            Lang::Uk => format!("{}{}", self.stem, if self.soft { "ій" } else { "ий" }),
            Lang::En => self.en[0].to_string(),
        }
    }

    fn matches(&self, word: &str) -> bool {
        self.en.contains(&word) || has_ending(word, self.stem, self.soft)
    }
}

/// `word` is `stem` followed by one of its four endings.
fn has_ending(word: &str, stem: &str, soft: bool) -> bool {
    let endings = if soft {
        ["ій", "я", "є", "і"]
    } else {
        ["ий", "а", "е", "і"]
    };
    word.strip_prefix(stem)
        .is_some_and(|end| endings.contains(&end))
}

const fn named(stem: &'static str, soft: bool, en: &'static [&'static str], rgb: Rgb) -> Named {
    Named { stem, soft, en, rgb }
}

/// Every named colour. The first eight are the bright ones `випадковий` chooses from;
/// blue and yellow are the colours of the Ukrainian flag.
pub const PALETTE: [Named; 12] = [
    named("червон", false, &["red"], Rgb(0xef, 0x44, 0x44)),
    named("помаранчев", false, &["orange"], Rgb(0xf9, 0x73, 0x16)),
    named("жовт", false, &["yellow"], Rgb(0xff, 0xd7, 0x00)),
    named("зелен", false, &["green"], Rgb(0x22, 0xc5, 0x5e)),
    named("блакитн", false, &["lightblue"], Rgb(0x38, 0xbd, 0xf8)),
    named("син", true, &["blue"], Rgb(0x00, 0x57, 0xb7)),
    named("фіолетов", false, &["purple"], Rgb(0x8b, 0x5c, 0xf6)),
    named("рожев", false, &["pink"], Rgb(0xec, 0x48, 0x99)),
    named("біл", false, &["white"], Rgb(0xff, 0xff, 0xff)),
    named("чорн", false, &["black"], Rgb(0x00, 0x00, 0x00)),
    named("сір", false, &["gray", "grey"], Rgb(0x9c, 0xa3, 0xaf)),
    named("коричнев", false, &["brown"], Rgb(0x8b, 0x5a, 0x2b)),
];

/// What a colour argument asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpec {
    Exact(Rgb),
    Random,
}

impl ColorSpec {
    /// The colour to use; `Random` picks a bright colour other than `current`.
    pub fn pick(self, current: Rgb, rng: &mut u64) -> Rgb {
        match self {
            ColorSpec::Exact(rgb) => rgb,
            ColorSpec::Random => {
                let bright: Vec<Rgb> = PALETTE[..8]
                    .iter()
                    .map(|c| c.rgb)
                    .filter(|c| *c != current)
                    .collect();
                bright[(next_random(rng) % bright.len() as u64) as usize]
            }
        }
    }
}

fn is_random(word: &str) -> bool {
    word == "random" || has_ending(word, "випадков", false)
}

/// Reads `"червона"`, `"red"`, `"#ff8800"` or `"випадковий"`; case and outer spaces don't matter.
pub fn parse_color(text: &str) -> Result<ColorSpec, ErrorKind> {
    let word = text.trim().to_lowercase();
    if let Some(hex) = word.strip_prefix('#') {
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            return Ok(ColorSpec::Exact(Rgb(byte(0), byte(2), byte(4))));
        }
    }
    if let Some(c) = PALETTE.iter().find(|c| c.matches(&word)) {
        return Ok(ColorSpec::Exact(c.rgb));
    }
    if is_random(&word) {
        return Ok(ColorSpec::Random);
    }
    let names: Vec<String> = PALETTE
        .iter()
        .flat_map(|c| [c.name(Lang::Uk), c.name(Lang::En)])
        .collect();
    let suggestion = suggest::closest(&word, names.iter().map(String::as_str)).map(str::to_string);
    Err(ErrorKind::UnknownColor {
        name: text.trim().to_string(),
        suggestion,
    })
}

/// A colour name written without quotes, as in `колір(червоний)`.
pub fn is_color_word(word: &str) -> bool {
    let word = word.to_lowercase();
    PALETTE.iter().any(|c| c.matches(&word)) || is_random(&word)
}
```

In `crates/yu-core/src/diagnostic.rs`:

1. Add `use crate::draw::PALETTE;` below `use crate::lang::Lang;`.
2. Add two variants at the end of `ErrorKind`, after `TooLong,`:

```rust
    UnknownColor {
        name: String,
        suggestion: Option<String>,
    },
    ColorNeedsQuotes(String),
```

3. In `message`, after the `TooLong` arm:

```rust
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
```

4. In `hint`, replace the arm `UnknownName { suggestion: Some(s), .. } => Some(format!(…))` with one that serves both kinds, and add two arms before `_ => None`:

```rust
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
```

```rust
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
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p yu-core colour`
Expected: 5 passed (four in `draw::tests`, `colour_errors_say_what_to_write` in `diagnostic::tests`).

- [ ] **Step 5: Check everything and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all
git add crates/yu-core/src/draw.rs crates/yu-core/src/diagnostic.rs crates/yu-core/src/lib.rs
git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "Colours: every Ukrainian form, English names, hex and random" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 2: The drawing model

**Files:**
- Modify: `crates/yu-core/src/draw.rs` (shapes, turtle, fills, limits; tests)
- Modify: `crates/yu-core/src/diagnostic.rs` (four error kinds, messages, hints, a test)

**Interfaces:**
- Consumes: `Rgb` from Task 1; `value::format_number(f64) -> String`.
- Produces:
  - `pub const MAX_SHAPES: usize = 50_000`
  - `pub enum DrawCmd { Background(Rgb), Circle { x, y, r, color }, Rect { x, y, w, h, color }, Line { x1, y1, x2, y2, color, width }, Label { text: String, x, y, color }, Fill { points: Vec<(f64, f64)>, color, slot: usize } }` (numbers are `f64`, colours `Rgb`; derives `Debug, Clone, PartialEq`)
  - `pub struct Turtle { pub x: f64, pub y: f64, pub heading: f64, pub pen_down: bool, pub used: bool }`
  - `pub struct Drawing { pub width: u32, pub height: u32, pub cmds: Vec<DrawCmd>, pub turtle: Turtle, pub color: Rgb, pub thickness: f64, /* private fill */ }`, `Default` (600×400, black, thickness 2, turtle at 300, 200)
  - `Drawing` methods: `is_empty(&self) -> bool`; `canvas(&mut self, w: f64, h: f64)`, `background(&mut self, color: Rgb)`, `set_thickness(&mut self, n: f64)`, `circle(&mut self, x, y, r)`, `rect(&mut self, x, y, w, h)`, `line(&mut self, x1, y1, x2, y2)`, `label(&mut self, text: String, x, y)`, `forward(&mut self, n: f64)`, `end_fill(&mut self)`, all returning `Result<(), ErrorKind>`; `turn(&mut self, degrees: f64)`, `pen(&mut self, down: bool)`, `begin_fill(&mut self)` return nothing
  - `ErrorKind::NegativeSize(f64)`, `ErrorKind::BadCanvas`, `ErrorKind::FillNotStarted`, `ErrorKind::TooManyShapes`

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `crates/yu-core/src/draw.rs`:

```rust
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_square_brings_the_turtle_back() {
        let mut d = Drawing::default();
        for _ in 0..4 {
            d.forward(100.0).unwrap();
            d.turn(90.0);
        }
        assert!(close(d.turtle.x, 300.0) && close(d.turtle.y, 200.0));
        assert_eq!(d.cmds.len(), 4);
    }

    #[test]
    fn right_turns_clockwise_on_screen() {
        let mut d = Drawing::default();
        d.turn(90.0);
        d.forward(10.0).unwrap();
        assert!(close(d.turtle.x, 300.0) && close(d.turtle.y, 210.0));
        d.turn(-180.0);
        assert_eq!(d.turtle.heading, 270.0);
    }

    #[test]
    fn pen_up_moves_without_drawing() {
        let mut d = Drawing::default();
        assert!(d.is_empty());
        d.pen(false);
        d.forward(50.0).unwrap();
        assert!(d.cmds.is_empty());
        assert!(!d.is_empty());
    }

    #[test]
    fn shapes_take_the_current_colour_and_thickness() {
        let mut d = Drawing::default();
        d.color = RED;
        d.set_thickness(5.0).unwrap();
        d.line(0.0, 0.0, 10.0, 10.0).unwrap();
        d.circle(1.0, 2.0, 3.0).unwrap();
        assert_eq!(
            d.cmds,
            vec![
                DrawCmd::Line {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 10.0,
                    y2: 10.0,
                    color: RED,
                    width: 5.0
                },
                DrawCmd::Circle {
                    x: 1.0,
                    y: 2.0,
                    r: 3.0,
                    color: RED
                },
            ]
        );
    }

    #[test]
    fn a_new_canvas_recentres_an_unused_turtle_only() {
        let mut d = Drawing::default();
        d.canvas(800.0, 600.4).unwrap();
        assert_eq!((d.width, d.height), (800, 600));
        assert_eq!((d.turtle.x, d.turtle.y), (400.0, 300.0));
        d.forward(10.0).unwrap();
        d.canvas(100.0, 100.0).unwrap();
        assert_eq!((d.turtle.x, d.turtle.y), (410.0, 300.0));
        assert_eq!(d.canvas(0.0, 10.0), Err(ErrorKind::BadCanvas));
        assert_eq!(d.canvas(10.0, 4001.0), Err(ErrorKind::BadCanvas));
    }

    #[test]
    fn a_fill_goes_under_its_outline_and_appears_when_finished() {
        let mut d = Drawing::default();
        d.background(RED).unwrap();
        d.begin_fill();
        for _ in 0..3 {
            d.forward(100.0).unwrap();
            d.turn(120.0);
        }
        d.end_fill().unwrap();
        assert_eq!(d.cmds.len(), 5);
        assert!(
            matches!(&d.cmds[1], DrawCmd::Fill { points, slot: 4, .. } if points.len() == 4)
        );
        assert!(matches!(d.cmds[2], DrawCmd::Line { .. }));
    }

    #[test]
    fn drawing_mistakes_and_limits() {
        let mut d = Drawing::default();
        assert_eq!(d.end_fill(), Err(ErrorKind::FillNotStarted));
        d.begin_fill();
        d.forward(10.0).unwrap();
        d.end_fill().unwrap();
        assert!(!d.cmds.iter().any(|c| matches!(c, DrawCmd::Fill { .. })));
        assert_eq!(d.circle(0.0, 0.0, -1.0), Err(ErrorKind::NegativeSize(-1.0)));
        assert_eq!(d.rect(0.0, 0.0, 5.0, -2.0), Err(ErrorKind::NegativeSize(-2.0)));
        assert_eq!(d.set_thickness(-3.0), Err(ErrorKind::NegativeSize(-3.0)));
        let mut d = Drawing::default();
        for _ in 0..MAX_SHAPES {
            d.line(0.0, 0.0, 1.0, 1.0).unwrap();
        }
        assert_eq!(d.line(0.0, 0.0, 1.0, 1.0), Err(ErrorKind::TooManyShapes));
    }
```

Add to the `tests` module in `crates/yu-core/src/diagnostic.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p yu-core draw`
Expected: compile errors such as ``cannot find struct, variant or union type `Drawing` `` and ``no variant named `NegativeSize` ``.

- [ ] **Step 3: Implement the model**

In `crates/yu-core/src/draw.rs`, below `is_color_word` and above the tests:

```rust
/// At most this many shapes in one drawing, so an endless loop can't eat all memory.
pub const MAX_SHAPES: usize = 50_000;

/// One thing on the canvas. Each carries its own colour, so a renderer keeps no state.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCmd {
    Background(Rgb),
    Circle { x: f64, y: f64, r: f64, color: Rgb },
    Rect { x: f64, y: f64, w: f64, h: f64, color: Rgb },
    Line { x1: f64, y1: f64, x2: f64, y2: f64, color: Rgb, width: f64 },
    Label { text: String, x: f64, y: f64, color: Rgb },
    /// A turtle fill; `slot` is its place in the animation.
    Fill { points: Vec<(f64, f64)>, color: Rgb, slot: usize },
}

/// Where the turtle is, where it faces (degrees, clockwise from the right) and its pen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turtle {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
    pub pen_down: bool,
    /// Set by the first turtle command; only a used turtle appears in the picture.
    pub used: bool,
}

/// Everything a program has drawn so far, and the pen it draws with.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawing {
    pub width: u32,
    pub height: u32,
    pub cmds: Vec<DrawCmd>,
    pub turtle: Turtle,
    pub color: Rgb,
    pub thickness: f64,
    /// An open fill: where its polygon goes in `cmds`, and the points walked so far.
    fill: Option<(usize, Vec<(f64, f64)>)>,
}

impl Default for Drawing {
    fn default() -> Self {
        Drawing {
            width: 600,
            height: 400,
            cmds: Vec::new(),
            turtle: Turtle {
                x: 300.0,
                y: 200.0,
                heading: 0.0,
                pen_down: true,
                used: false,
            },
            color: Rgb(0, 0, 0),
            thickness: 2.0,
            fill: None,
        }
    }
}

/// A size may be zero but not negative.
fn size(n: f64) -> Result<f64, ErrorKind> {
    if n < 0.0 {
        Err(ErrorKind::NegativeSize(n))
    } else {
        Ok(n)
    }
}

impl Drawing {
    /// Nothing drawn and the turtle never used.
    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty() && !self.turtle.used
    }

    fn room(&self) -> Result<(), ErrorKind> {
        if self.cmds.len() >= MAX_SHAPES {
            Err(ErrorKind::TooManyShapes)
        } else {
            Ok(())
        }
    }

    fn push(&mut self, cmd: DrawCmd) -> Result<(), ErrorKind> {
        self.room()?;
        self.cmds.push(cmd);
        Ok(())
    }

    /// Resizes the canvas; a turtle that hasn't been used yet goes to the new centre.
    pub fn canvas(&mut self, w: f64, h: f64) -> Result<(), ErrorKind> {
        let (w, h) = (w.round(), h.round());
        let fits = |n: f64| (1.0..=4000.0).contains(&n);
        if !fits(w) || !fits(h) {
            return Err(ErrorKind::BadCanvas);
        }
        self.width = w as u32;
        self.height = h as u32;
        if !self.turtle.used {
            self.turtle.x = w / 2.0;
            self.turtle.y = h / 2.0;
        }
        Ok(())
    }

    /// Paints the whole canvas, over everything drawn so far.
    pub fn background(&mut self, color: Rgb) -> Result<(), ErrorKind> {
        self.push(DrawCmd::Background(color))
    }

    pub fn set_thickness(&mut self, n: f64) -> Result<(), ErrorKind> {
        self.thickness = size(n)?;
        Ok(())
    }

    pub fn circle(&mut self, x: f64, y: f64, r: f64) -> Result<(), ErrorKind> {
        let r = size(r)?;
        self.push(DrawCmd::Circle {
            x,
            y,
            r,
            color: self.color,
        })
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) -> Result<(), ErrorKind> {
        let (w, h) = (size(w)?, size(h)?);
        self.push(DrawCmd::Rect {
            x,
            y,
            w,
            h,
            color: self.color,
        })
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> Result<(), ErrorKind> {
        self.push(DrawCmd::Line {
            x1,
            y1,
            x2,
            y2,
            color: self.color,
            width: self.thickness,
        })
    }

    pub fn label(&mut self, text: String, x: f64, y: f64) -> Result<(), ErrorKind> {
        self.push(DrawCmd::Label {
            text,
            x,
            y,
            color: self.color,
        })
    }

    /// Moves the turtle `n` points ahead (back when `n` is negative).
    pub fn forward(&mut self, n: f64) -> Result<(), ErrorKind> {
        let t = self.turtle;
        let (sin, cos) = t.heading.to_radians().sin_cos();
        let (x, y) = (t.x + n * cos, t.y + n * sin);
        if t.pen_down {
            self.line(t.x, t.y, x, y)?;
        }
        self.turtle.x = x;
        self.turtle.y = y;
        self.turtle.used = true;
        if let Some((_, points)) = &mut self.fill {
            points.push((x, y));
        }
        Ok(())
    }

    /// Turns the turtle clockwise on screen (anticlockwise for negative degrees).
    pub fn turn(&mut self, degrees: f64) {
        self.turtle.heading = (self.turtle.heading + degrees).rem_euclid(360.0);
        self.turtle.used = true;
    }

    pub fn pen(&mut self, down: bool) {
        self.turtle.pen_down = down;
        self.turtle.used = true;
    }

    /// Starts recording the turtle's path; calling it again starts over.
    pub fn begin_fill(&mut self) {
        self.fill = Some((self.cmds.len(), vec![(self.turtle.x, self.turtle.y)]));
        self.turtle.used = true;
    }

    /// Fills the recorded path, under the lines drawn since `begin_fill`.
    pub fn end_fill(&mut self) -> Result<(), ErrorKind> {
        let (at, points) = self.fill.take().ok_or(ErrorKind::FillNotStarted)?;
        if points.len() >= 3 {
            self.room()?;
            let slot = self.cmds.len();
            self.cmds.insert(
                at,
                DrawCmd::Fill {
                    points,
                    color: self.color,
                    slot,
                },
            );
        }
        Ok(())
    }
}
```

In `crates/yu-core/src/diagnostic.rs`:

1. Add `use crate::value::format_number;` below the other `use` lines.
2. Add four variants at the end of `ErrorKind`, after `ColorNeedsQuotes(String),`:

```rust
    NegativeSize(f64),
    BadCanvas,
    FillNotStarted,
    TooManyShapes,
```

3. In `message`, after the `ColorNeedsQuotes` arm:

```rust
            NegativeSize(n) => format!(
                "{} {}",
                lang.pick("розмір не може бути від'ємним:", "a size can't be negative:"),
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
                .pick("забагато фігур (понад 50 000)", "too many shapes (over 50,000)")
                .into(),
```

4. In `hint`, change the first line of the `TooLong` arm from `TooLong => Some(` to `TooLong | TooManyShapes => Some(`, and add before `_ => None`:

```rust
            FillNotStarted => Some(
                lang.pick("Спершу виклич почни_заливку()", "Call begin_fill() first")
                    .into(),
            ),
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p yu-core draw`
Expected: every test in `draw::tests` and `diagnostic::tests::drawing_errors_in_both_languages` passes, 0 failed.

- [ ] **Step 5: Check everything and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all
git add crates/yu-core/src/draw.rs crates/yu-core/src/diagnostic.rs
git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "Drawing model: shapes, turtle, fills and limits" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 3: The SVG renderer

**Files:**
- Create: `crates/yu-core/src/svg.rs`
- Modify: `crates/yu-core/src/lib.rs` (module list)

**Interfaces:**
- Consumes: `Drawing`, `DrawCmd`, `Turtle` from Task 2; `Rgb::hex`.
- Produces: `pub fn render(d: &Drawing) -> String` and `pub fn num(x: f64) -> String`.

The animation technique was checked in headless Chromium before this plan: with `pathLength="1"` and the `yu-draw` keyframes a line is hidden at 0 ms, half drawn at 500 ms of a 1 s animation and solid afterwards; a delayed circle stays hidden until its turn.

- [ ] **Step 1: Write the failing tests**

In `crates/yu-core/src/lib.rs` add `pub mod svg;` between `pub mod suggest;` and `pub mod token;`.

Create `crates/yu-core/src/svg.rs` with only the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Rgb;

    #[test]
    fn numbers_are_short_and_always_valid() {
        assert_eq!(num(300.0), "300");
        assert_eq!(num(212.1320343), "212.13");
        assert_eq!(num(-12.5), "-12.5");
        assert_eq!(num(-0.001), "0");
        assert_eq!(num(f64::INFINITY), "0");
        assert_eq!(num(f64::NAN), "0");
    }

    #[test]
    fn an_empty_drawing_is_a_white_canvas() {
        let svg = render(&Drawing::default());
        assert!(svg.starts_with(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"600\" height=\"400\" viewBox=\"0 0 600 400\">\n<style>\n"
        ));
        assert!(svg.contains("\n<rect width=\"600\" height=\"400\" fill=\"#ffffff\"/>\n"));
        assert!(svg.ends_with("</g>\n</svg>\n"));
        assert!(!svg.contains("rotate("));
    }

    #[test]
    fn shapes_become_elements_with_growing_delays() {
        let mut d = Drawing::default();
        d.color = Rgb(0xef, 0x44, 0x44);
        d.circle(300.0, 200.0, 50.0).unwrap();
        d.line(0.0, 0.0, 10.0, 20.0).unwrap();
        d.label("<Привіт & пока>".into(), 5.0, 10.0).unwrap();
        let svg = render(&d);
        assert!(svg.contains(
            "<circle cx=\"300\" cy=\"200\" r=\"50\" fill=\"#ef4444\" style=\"animation-delay:0s\"/>"
        ));
        assert!(svg.contains(
            "<line x1=\"0\" y1=\"0\" x2=\"10\" y2=\"20\" stroke=\"#ef4444\" stroke-width=\"2\" pathLength=\"1\" style=\"animation-delay:0.15s\"/>"
        ));
        assert!(svg.contains(
            "<text x=\"5\" y=\"26\" fill=\"#ef4444\" style=\"animation-delay:0.3s\">&lt;Привіт &amp; пока&gt;</text>"
        ));
    }

    #[test]
    fn backgrounds_and_rectangles() {
        let mut d = Drawing::default();
        d.background(Rgb(0x00, 0x57, 0xb7)).unwrap();
        d.rect(10.0, 20.0, 30.0, 40.0).unwrap();
        let svg = render(&d);
        assert!(svg.contains(
            "<rect width=\"600\" height=\"400\" fill=\"#0057b7\" style=\"animation-delay:0s\"/>"
        ));
        assert!(svg.contains(
            "<rect x=\"10\" y=\"20\" width=\"30\" height=\"40\" fill=\"#000000\" style=\"animation-delay:0.15s\"/>"
        ));
    }

    #[test]
    fn long_drawings_finish_in_about_four_seconds() {
        let mut d = Drawing::default();
        for _ in 0..100 {
            d.line(0.0, 0.0, 1.0, 1.0).unwrap();
        }
        let svg = render(&d);
        assert!(svg.contains(".yu>line{animation:yu-draw 0.04s linear backwards}"));
        assert!(svg.contains("animation-delay:3.96s"));
    }

    #[test]
    fn fills_are_polygons_that_appear_at_their_slot() {
        let mut d = Drawing::default();
        d.begin_fill();
        for _ in 0..3 {
            d.forward(100.0).unwrap();
            d.turn(120.0);
        }
        d.end_fill().unwrap();
        assert!(render(&d).contains(
            "<polygon points=\"300,200 400,200 350,286.6 300,200\" fill=\"#000000\" style=\"animation-delay:0.45s\"/>"
        ));
    }

    #[test]
    fn the_turtle_is_drawn_last_where_it_stopped() {
        let mut d = Drawing::default();
        d.turn(90.0);
        d.forward(50.0).unwrap();
        let svg = render(&d);
        let turtle = svg
            .find("<g transform=\"translate(300 250) rotate(90)\" style=\"animation-delay:0.15s\">")
            .expect("the turtle");
        assert!(svg.find("<line").unwrap() < turtle);
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p yu-core svg`
Expected: compile errors ``cannot find function `render` `` and ``cannot find function `num` ``.

- [ ] **Step 3: Implement the renderer**

Put this above the tests in `crates/yu-core/src/svg.rs`:

```rust
//! The one renderer: a drawing as an SVG that draws itself.

use crate::draw::{DrawCmd, Drawing, Turtle};

const FONT: &str = "system-ui, -apple-system, Segoe UI, Roboto, sans-serif";

/// A number rounded to two decimals, without trailing zeros; `-0` and non-finite numbers are `0`.
pub fn num(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0;
    if !r.is_finite() || r == 0.0 {
        return "0".into();
    }
    format!("{r}")
}

/// Seconds for CSS, to the millisecond.
fn secs(t: f64) -> String {
    format!("{}s", (t * 1000.0).round() / 1000.0)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Lines draw themselves in one step; everything else fades in. Before its turn an element is
/// hidden and afterwards it keeps its plain style, so viewers without CSS animation show the
/// finished picture.
fn style(step: f64) -> String {
    let draw = format!(
        ".yu>line{{animation:yu-draw {} linear backwards}}",
        secs(step)
    );
    [
        "<style>",
        ".yu>*{animation:yu-in .3s ease-out backwards}",
        draw.as_str(),
        "@keyframes yu-in{from{opacity:0}}",
        "@keyframes yu-draw{from{stroke-dasharray:1;stroke-dashoffset:1}to{stroke-dasharray:1;stroke-dashoffset:0}}",
        "@media (prefers-reduced-motion:reduce){.yu>*{animation:none}}",
        "</style>",
    ]
    .join("\n")
}

/// A small green turtle where `t` stopped, turned the way it faces.
fn turtle(t: &Turtle, delay: &str) -> String {
    format!(
        "<g transform=\"translate({} {}) rotate({})\"{delay}>\
         <g fill=\"#15803d\"><circle cx=\"6\" cy=\"-8\" r=\"3\"/><circle cx=\"6\" cy=\"8\" r=\"3\"/>\
         <circle cx=\"-6\" cy=\"-8\" r=\"3\"/><circle cx=\"-6\" cy=\"8\" r=\"3\"/>\
         <circle cx=\"12\" cy=\"0\" r=\"4\"/><path d=\"M-9 -3 L-15 0 L-9 3 Z\"/></g>\
         <ellipse rx=\"10\" ry=\"8\" fill=\"#22c55e\" stroke=\"#15803d\" stroke-width=\"1.5\"/>\
         <ellipse rx=\"5\" ry=\"4\" fill=\"#16a34a\"/></g>",
        num(t.x),
        num(t.y),
        num(t.heading)
    )
}

/// The drawing as an animated SVG; the same drawing always gives the same text.
pub fn render(d: &Drawing) -> String {
    let (width, height) = (d.width, d.height);
    let places = d.cmds.len() + usize::from(d.turtle.used);
    let step = (4.0 / places.max(1) as f64).min(0.15);
    let delay = |place: usize| {
        format!(
            " style=\"animation-delay:{}\"",
            secs(place as f64 * step)
        )
    };
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n{}\n\
         <rect width=\"{width}\" height=\"{height}\" fill=\"#ffffff\"/>\n\
         <g class=\"yu\" stroke-linecap=\"round\" stroke-linejoin=\"round\" font-family=\"{FONT}\" font-size=\"20\">\n",
        style(step)
    );
    for (i, cmd) in d.cmds.iter().enumerate() {
        let element = match cmd {
            DrawCmd::Background(color) => format!(
                "<rect width=\"{width}\" height=\"{height}\" fill=\"{}\"{}/>",
                color.hex(),
                delay(i)
            ),
            DrawCmd::Circle { x, y, r, color } => format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"{}/>",
                num(*x),
                num(*y),
                num(*r),
                color.hex(),
                delay(i)
            ),
            DrawCmd::Rect { x, y, w, h, color } => format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"{}/>",
                num(*x),
                num(*y),
                num(*w),
                num(*h),
                color.hex(),
                delay(i)
            ),
            DrawCmd::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\" pathLength=\"1\"{}/>",
                num(*x1),
                num(*y1),
                num(*x2),
                num(*y2),
                color.hex(),
                num(*width),
                delay(i)
            ),
            DrawCmd::Label { text, x, y, color } => format!(
                "<text x=\"{}\" y=\"{}\" fill=\"{}\"{}>{}</text>",
                num(*x),
                num(y + 16.0),
                color.hex(),
                delay(i),
                escape(text)
            ),
            DrawCmd::Fill {
                points,
                color,
                slot,
            } => {
                let points: Vec<String> = points
                    .iter()
                    .map(|(x, y)| format!("{},{}", num(*x), num(*y)))
                    .collect();
                format!(
                    "<polygon points=\"{}\" fill=\"{}\"{}/>",
                    points.join(" "),
                    color.hex(),
                    delay(*slot)
                )
            }
        };
        out.push_str(&element);
        out.push('\n');
    }
    if d.turtle.used {
        out.push_str(&turtle(&d.turtle, &delay(d.cmds.len())));
        out.push('\n');
    }
    out.push_str("</g>\n</svg>\n");
    out
}
```

Note: inside the `Line` arm the field `width` shadows the canvas `width`; the arm only uses the line's width. The label baseline is 16 points (0.8 × 20 px) below `y`, so `y` is the top of the text.

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p yu-core svg`
Expected: 7 passed in `svg::tests`, 0 failed.

- [ ] **Step 5: Check everything and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all
git add crates/yu-core/src/svg.rs crates/yu-core/src/lib.rs
git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "SVG renderer: fading shapes, self-drawing lines and the turtle" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 4: Drawing built-ins, the interpreter and the session

**Files:**
- Modify: `crates/yu-core/src/builtins.rs` (sixteen variants, names, arity, `call`, `draw_call`, `color_arg`)
- Modify: `crates/yu-core/src/interp.rs` (the interpreter owns the drawing; `ColorNeedsQuotes`)
- Modify: `crates/yu-core/src/lib.rs` (`Session` keeps drawing and random state; re-exports)
- Modify: `crates/yu-core/src/diagnostic.rs` (`ExpectedText` and a test)
- Create: `crates/yu-core/tests/drawing.rs`

**Interfaces:**
- Consumes: `parse_color`, `ColorSpec::pick`, `is_color_word`, `Rgb` (Task 1); `Drawing` and its methods (Task 2).
- Produces:
  - `Builtin::{Canvas, Background, Color, Thickness, Circle, Rect, Line, Label, Forward, Back, Right, Left, PenUp, PenDown, BeginFill, EndFill}`, `NAMES: [(Builtin, &str, &str); 24]`
  - `builtins::call(b: Builtin, args: Vec<Value>, span: Span, host: &mut dyn Host, lang: Lang, rng: &mut u64, drawing: &mut Drawing) -> Result<Value, Diagnostic>`
  - `Interp` fields `pub(crate) rng: u64` and `pub(crate) drawing: Drawing`
  - `Session::drawing(&self) -> &Drawing`; `Session::run` continues the random sequence between runs
  - `yu_core::{DrawCmd, Drawing, Rgb}` re-exported
  - `ErrorKind::ExpectedText(Ty)`

- [ ] **Step 1: Write the failing tests**

Create `crates/yu-core/tests/drawing.rs`:

```rust
mod common;

use common::Capture;
use yu_core::diagnostic::Ty;
use yu_core::{DrawCmd, Drawing, ErrorKind, Lang, Options, Rgb, Session};

const RED: Rgb = Rgb(0xef, 0x44, 0x44);

/// Runs `src` in a new session and returns what it drew.
fn draw(src: &str) -> Drawing {
    let mut session = Session::new(Options::default());
    if let Err(e) = session.run(src, &mut Capture::default()) {
        panic!("{}", e.render(src, Lang::Uk));
    }
    session.drawing().clone()
}

fn err(src: &str) -> ErrorKind {
    let mut session = Session::new(Options::default());
    session.run(src, &mut Capture::default()).unwrap_err().kind
}

#[test]
fn shapes_in_both_languages() {
    let d = draw("колір(\"червона\")\nколо(300, 200, 50)\ncolor(\"#0057B7\")\nrect(0, 0, 10, 20)");
    assert_eq!(
        d.cmds,
        vec![
            DrawCmd::Circle {
                x: 300.0,
                y: 200.0,
                r: 50.0,
                color: RED
            },
            DrawCmd::Rect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 20.0,
                color: Rgb(0x00, 0x57, 0xb7)
            },
        ]
    );
}

#[test]
fn labels_show_values_like_say() {
    let d = draw("напис(\"Бал: \" + 5, 10, 20)\nlabel(так, 0, 0)");
    assert!(matches!(&d.cmds[0], DrawCmd::Label { text, .. } if text == "Бал: 5"));
    assert!(matches!(&d.cmds[1], DrawCmd::Label { text, .. } if text == "так"));
}

#[test]
fn the_turtle_draws_a_square() {
    let d = draw("повтори 4 рази:\n    вперед(100)\n    праворуч(90)");
    assert_eq!(d.cmds.len(), 4);
    assert!(d.turtle.used);
    assert!(matches!(d.cmds[1], DrawCmd::Line { x1, y1, .. } if x1 == 400.0 && y1 == 200.0));
}

#[test]
fn a_filled_triangle_in_english() {
    let d = draw(
        "color(\"yellow\")\nbegin_fill()\nrepeat 3 times:\n    forward(100)\n    left(120)\nend_fill()\npen_up()\nback(10)",
    );
    assert!(matches!(&d.cmds[0], DrawCmd::Fill { color, points, .. }
        if *color == Rgb(0xff, 0xd7, 0x00) && points.len() == 4));
    assert_eq!(d.cmds.len(), 4);
}

#[test]
fn a_random_colour_never_repeats_the_current_one() {
    let d = draw("колір(\"червоний\")\nповтори 20 разів:\n    колір(\"випадковий\")\n    коло(0, 0, 1)");
    let colors: Vec<Rgb> = d
        .cmds
        .iter()
        .map(|c| match c {
            DrawCmd::Circle { color, .. } => *color,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_ne!(colors[0], RED);
    assert!(colors.windows(2).all(|w| w[0] != w[1]));
}

#[test]
fn drawing_errors() {
    assert_eq!(
        err("колір(червоний)"),
        ErrorKind::ColorNeedsQuotes("червоний".into())
    );
    assert_eq!(err("фон(5)"), ErrorKind::ExpectedText(Ty::Number));
    assert_eq!(
        err("колір(\"червний\")"),
        ErrorKind::UnknownColor {
            name: "червний".into(),
            suggestion: Some("червоний".into())
        }
    );
    assert_eq!(
        err("коло(1, 2)"),
        ErrorKind::ArgCount {
            name: "коло".into(),
            expected: 3,
            got: 2
        }
    );
    assert_eq!(err("коло(1, 2, -3)"), ErrorKind::NegativeSize(-3.0));
    assert_eq!(err("коло(\"1\", 2, 3)"), ErrorKind::ExpectedNumber(Ty::Text));
    assert_eq!(err("заверши_заливку()"), ErrorKind::FillNotStarted);
    assert_eq!(err("полотно(0, 10)"), ErrorKind::BadCanvas);
    assert_eq!(err("лінія = 5"), ErrorKind::BuiltinAsName("лінія".into()));
}

#[test]
fn a_session_keeps_its_drawing_and_its_random_sequence() {
    let mut session = Session::new(Options::default());
    let mut host = Capture::default();
    session.run("коло(10, 10, 5)", &mut host).unwrap();
    session.run("коло(20, 20, 5)", &mut host).unwrap();
    assert_eq!(session.drawing().cmds.len(), 2);
    let first = session.run("випадкове(1, 1000000)", &mut host).unwrap();
    let second = session.run("випадкове(1, 1000000)", &mut host).unwrap();
    assert_ne!(first, second);
}
```

Add to the `tests` module in `crates/yu-core/src/diagnostic.rs`:

```rust
    #[test]
    fn expected_text_names_what_it_got() {
        assert_eq!(
            ErrorKind::ExpectedText(Ty::Number).message(Lang::Uk),
            "тут потрібен текст, а маємо: число"
        );
        assert_eq!(
            ErrorKind::ExpectedText(Ty::List).message(Lang::En),
            "expected text, got a list"
        );
    }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p yu-core --test drawing`
Expected: compile errors such as ``unresolved imports `yu_core::DrawCmd` `` and ``no method named `drawing` found for struct `Session` ``.

- [ ] **Step 3: Implement**

`crates/yu-core/src/diagnostic.rs`: add the variant `ExpectedText(Ty),` at the end of `ErrorKind` (after `TooManyShapes,`) and, in `message`, after the `TooManyShapes` arm:

```rust
            ExpectedText(t) => {
                if uk {
                    format!("тут потрібен текст, а маємо: {}", t.name(lang))
                } else {
                    format!("expected text, got {}", t.name(lang))
                }
            }
```

`crates/yu-core/src/builtins.rs`:

1. Add `use crate::draw::{parse_color, Drawing, Rgb};` to the imports.
2. Replace the enum and the name table:

```rust
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
```

3. Replace the body of `arity`:

```rust
        match self {
            Builtin::Say => (0, usize::MAX),
            Builtin::Ask => (0, 1),
            Builtin::Random | Builtin::Append | Builtin::Canvas => (2, 2),
            Builtin::Circle | Builtin::Label => (3, 3),
            Builtin::Rect | Builtin::Line => (4, 4),
            Builtin::PenUp | Builtin::PenDown | Builtin::BeginFill | Builtin::EndFill => (0, 0),
            _ => (1, 1),
        }
```

4. Give `call` a last parameter `drawing: &mut Drawing` and add a final arm to its `match b` after the `Builtin::Append` arm:

```rust
        _ => {
            draw_call(b, &args, span, lang, rng, drawing)?;
            Value::Nothing
        }
```

5. Add below `call`:

```rust
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
```

`crates/yu-core/src/interp.rs`:

1. Add `use crate::draw::{self, Drawing};` to the imports.
2. In `struct Interp` change `rng: u64,` to `pub(crate) rng: u64,` and add the field `pub(crate) drawing: Drawing,` after it; in `Interp::new` add `drawing: Drawing::default(),` after `rng,`.
3. In `call`, pass the drawing as the last argument: after `&mut self.rng,` add `&mut self.drawing,`.
4. In `lookup`, right after the `if let Some(b) = Builtin::lookup(name) { … }` block:

```rust
        if draw::is_color_word(name) {
            return Err(Diagnostic::new(
                ErrorKind::ColorNeedsQuotes(name.into()),
                span,
            ));
        }
```

`crates/yu-core/src/lib.rs`:

1. Add `pub use draw::{DrawCmd, Drawing, Rgb};` to the `pub use` lines.
2. Replace `Session` and its `impl`:

```rust
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
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p yu-core`
Expected: all pass, including 7 in `tests/drawing.rs` and `expected_text_names_what_it_got`; the milestone-1 tests still pass (`скаж` still suggests `скажи`).

- [ ] **Step 5: Check everything and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all
git add crates/yu-core/src/builtins.rs crates/yu-core/src/interp.rs crates/yu-core/src/lib.rs crates/yu-core/src/diagnostic.rs crates/yu-core/tests/drawing.rs
git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "Drawing built-ins in both languages; a session keeps its picture and random sequence" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 5: `yu --svg` saves the picture

**Files:**
- Modify: `crates/yu-cli/src/main.rs` (`--svg`, `run_file`, `save_picture`, usage, REPL hint)
- Modify: `crates/yu-cli/src/color.rs` (`Paint::hint`)
- Modify: `crates/yu-cli/tests/cli.rs` (five tests)

**Interfaces:**
- Consumes: `Session::drawing`, `Drawing::is_empty` (Tasks 2 and 4), `yu_core::svg::render` (Task 3).
- Produces: the `--svg <file>` option; `Paint::hint(&self, line: &str) -> String`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/yu-cli/tests/cli.rs`:

```rust
#[test]
fn svg_saves_the_picture() {
    let p = program("circle.yu", "коло(300, 200, 50)\n");
    let out_file = p.with_extension("svg");
    let _ = std::fs::remove_file(&out_file);
    let out = yu().arg(&p).arg("--svg").arg(&out_file).output().unwrap();
    assert!(out.status.success(), "{}", text(out.stderr));
    let svg = std::fs::read_to_string(&out_file).unwrap();
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""), "{svg}");
    assert!(svg.contains("<circle cx=\"300\" cy=\"200\" r=\"50\""), "{svg}");
}

#[test]
fn a_drawing_without_svg_gets_a_hint() {
    let p = program("hint.yu", "лінія(0, 0, 10, 10)\n");
    let out = yu().arg(&p).arg("--no-color").output().unwrap();
    assert!(out.status.success());
    let err = text(out.stderr);
    assert!(err.contains("Щоб зберегти малюнок: yu "), "{err}");
    let svg_arg = format!("--svg {}", p.with_extension("svg").display());
    assert!(err.contains(&svg_arg), "{err}");
    let out = yu().args(["--lang", "en"]).arg(&p).output().unwrap();
    assert!(text(out.stderr).contains("To save the picture: yu "));
}

#[test]
fn programs_that_draw_nothing_get_no_hint() {
    let p = program("quiet.yu", "скажи(1)\n");
    let out = yu().arg(&p).output().unwrap();
    assert_eq!(text(out.stderr), "");
}

#[test]
fn svg_needs_a_file_name_and_a_program() {
    let p = program("plain.yu", "скажи(1)\n");
    assert_eq!(
        yu().arg(&p).arg("--svg").output().unwrap().status.code(),
        Some(2)
    );
    assert_eq!(
        yu().args(["--svg", "x.svg"]).output().unwrap().status.code(),
        Some(2)
    );
}

#[test]
fn the_repl_says_once_where_pictures_go() {
    let mut child = yu()
        .arg("--no-color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("коло(1, 1, 1)\nколо(2, 2, 2)\nвийти\n".as_bytes())
        .unwrap();
    let err = text(child.wait_with_output().unwrap().stderr);
    assert_eq!(err.matches("--svg").count(), 1, "{err}");
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p yu-cli`
Expected: `svg_saves_the_picture` (exit code 2, unknown option), `a_drawing_without_svg_gets_a_hint` and `the_repl_says_once_where_pictures_go` (no hint) FAIL. The other two already pass and guard the behaviour.

- [ ] **Step 3: Implement**

`crates/yu-cli/src/color.rs`, inside `impl Paint` after `diagnostic`:

```rust
    /// A cyan line with a tip.
    pub fn hint(&self, line: &str) -> String {
        self.wrap("36", line)
    }
```

`crates/yu-cli/src/main.rs`:

1. Imports: add `use std::path::Path;` and change the `yu_core` line to `use yu_core::{svg, Host, Lang, Options, Session, Value};`.
2. Add the field `svg: Option<String>,` to `struct Args`, between `color` and `command`.
3. Replace `usage`:

```rust
fn usage(lang: Lang) -> &'static str {
    lang.pick(
        "Yu — мова програмування українською та англійською\n\n\
         Використання:\n  yu                  інтерактивний режим\n  yu файл.yu          запустити програму\n  \
         yu run файл.yu      те саме\n  --svg файл.svg      зберегти малюнок\n  --lang en           помилки англійською\n  \
         --no-color          без кольорів\n  --version           версія",
        "Yu — a programming language in Ukrainian and English\n\n\
         Usage:\n  yu                  interactive mode\n  yu file.yu          run a program\n  \
         yu run file.yu      the same\n  --svg file.svg      save the picture\n  --lang uk           messages in Ukrainian\n  \
         --no-color          no colours\n  --version           version",
    )
}
```

4. In `parse_args`: declare `let mut svg: Option<String> = None;` after the first `let`; add this arm after the `"--no-color"` arm:

```rust
            "--svg" => {
                let file = args.next().filter(|f| !f.starts_with('-'));
                svg = Some(file.ok_or_else(|| {
                    lang.pick("--svg: потрібна назва файлу", "--svg: a file name is needed")
                        .to_string()
                })?);
            }
```

then, after `let command = …;` and before `Ok(Args { … })`:

```rust
    if svg.is_some() && matches!(command, Command::Repl) {
        return Err(lang
            .pick(
                "--svg працює лише з файлом програми",
                "--svg needs a program file",
            )
            .into());
    }
```

and build `Args { lang, color, svg, command }`.

5. In `main`, change the `Run` arm to `Command::Run(path) => run_file(&path, args.svg.as_deref(), opts, paint),`.
6. Replace `run_file` and add `save_picture`:

```rust
fn run_file(path: &str, svg_file: Option<&str>, opts: Options, paint: Paint) -> ExitCode {
    let src = match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(e) => {
            eprintln!(
                "yu: {} {path}: {e}",
                opts.lang.pick("не вдалося прочитати", "can't read")
            );
            return ExitCode::from(2);
        }
    };
    let worker = std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            let mut session = Session::new(opts);
            let result = session.run(&src, &mut Terminal);
            let drew = !session.drawing().is_empty();
            let picture = svg::render(session.drawing());
            result
                .map(|_| (drew, picture))
                .map_err(|e| e.render(&src, opts.lang))
        });
    match worker.expect("start the program thread").join() {
        Ok(Ok((drew, picture))) => save_picture(path, svg_file, drew, &picture, opts.lang, paint),
        Ok(Err(rendered)) => {
            eprintln!("{}", paint.diagnostic(&rendered));
            ExitCode::from(1)
        }
        Err(_) => ExitCode::from(101),
    }
}

/// Writes the picture for `--svg`; without it, says how to save a picture the program drew.
fn save_picture(
    path: &str,
    svg_file: Option<&str>,
    drew: bool,
    picture: &str,
    lang: Lang,
    paint: Paint,
) -> ExitCode {
    match svg_file {
        Some(file) => {
            if let Err(e) = std::fs::write(file, picture) {
                eprintln!(
                    "yu: {} {file}: {e}",
                    lang.pick("не вдалося зберегти", "can't save")
                );
                return ExitCode::from(2);
            }
        }
        None if drew => {
            let suggested = Path::new(path).with_extension("svg");
            let tip = format!(
                "{} yu {path} --svg {}",
                lang.pick("Щоб зберегти малюнок:", "To save the picture:"),
                suggested.display()
            );
            eprintln!("{}", paint.hint(&tip));
        }
        None => {}
    }
    ExitCode::SUCCESS
}
```

7. In `repl`, add `let mut told_about_pictures = false;` after `let mut buffer = String::new();`, and right after the `match session.run(&src, &mut Terminal) { … }` block:

```rust
        if !told_about_pictures && !session.drawing().is_empty() {
            told_about_pictures = true;
            let tip = opts.lang.pick(
                "Малюнок тут не видно. Щоб зберегти його, запусти програму з файлу: yu файл.yu --svg малюнок.svg",
                "Pictures aren't shown here. To save one, run a program file: yu file.yu --svg picture.svg",
            );
            eprintln!("{}", paint.hint(tip));
        }
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p yu-cli`
Expected: 10 passed (5 old, 5 new), 0 failed.

- [ ] **Step 5: Try it by hand**

```bash
printf 'колір("синій")\nколо(300, 200, 80)\n' > target/demo.yu
cargo run -q -p yu-cli -- target/demo.yu
cargo run -q -p yu-cli -- target/demo.yu --svg target/demo.svg && head -c 120 target/demo.svg
```

Expected: the first run prints `Щоб зберегти малюнок: yu target/demo.yu --svg target/demo.svg` to stderr; the second prints nothing and `target/demo.svg` starts with `<svg xmlns="http://www.w3.org/2000/svg" width="600" height="400"`.

- [ ] **Step 6: Check everything and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all
git add crates/yu-cli/src/main.rs crates/yu-cli/src/color.rs crates/yu-cli/tests/cli.rs
git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "yu --svg saves the picture; a hint when a drawing isn't saved" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

### Task 6: Gallery, golden SVGs and the README

**Files:**
- Create: `examples/flag.yu`, `examples/sun.yu`, `examples/star.yu`, `examples/house.yu`, `examples/spiral.yu` and their `.svg`
- Modify: `crates/yu-core/tests/golden.rs`
- Modify: `README.md`
- Create, not committed: `target/preview.py`

**Interfaces:**
- Consumes: `Session::drawing`, `Drawing::is_empty`, `svg::render`, `yu --svg`.
- Produces: five golden drawing examples; the README gallery shows their SVGs.

- [ ] **Step 1: Make the golden test check pictures, and add the programs**

Replace `crates/yu-core/tests/golden.rs`:

```rust
mod common;

use std::fs;
use std::path::Path;

use common::Capture;
use yu_core::{svg, Lang, Options, Session};

#[test]
fn examples_match_their_expected_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut checked = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("yu") {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        let mut host = Capture::default();
        if let Ok(input) = fs::read_to_string(path.with_extension("in")) {
            host.input = input.lines().map(str::to_string).collect();
        }
        let mut session = Session::new(Options::default());
        match session.run(&src, &mut host) {
            Ok(_) => {
                let drawing = session.drawing();
                expect(&path, "out", &host.out.join("\n"), !host.out.is_empty());
                expect(&path, "svg", &svg::render(drawing), !drawing.is_empty());
            }
            Err(e) => expect(&path, "err", &e.render(&src, Lang::Uk), true),
        }
        checked += 1;
    }
    assert!(checked >= 11, "only {checked} examples found");
}

/// Compares `actual` with the file next to `example`; the file must exist when `produced`.
fn expect(example: &Path, ext: &str, actual: &str, produced: bool) {
    let file = example.with_extension(ext);
    match fs::read_to_string(&file) {
        Ok(expected) => assert_eq!(
            actual.trim_end(),
            expected.replace("\r\n", "\n").trim_end(),
            "{}",
            file.display()
        ),
        Err(_) => assert!(!produced, "missing {}", file.display()),
    }
}
```

`examples/flag.yu`:

```
# Прапор України: два прямокутники й напис
колір("синій")
прямокутник(0, 0, 600, 200)
колір("жовтий")
прямокутник(0, 200, 600, 200)
колір("білий")
напис("Україна", 24, 24)
```

`examples/sun.yu`:

```
# Сонце: черепашка малює промені, а коло стає посередині
фон("блакитний")
колір("жовтий")
товщина(8)
повтори 12 разів:
    підніми_перо()
    вперед(75)
    опусти_перо()
    вперед(45)
    підніми_перо()
    назад(120)
    праворуч(30)
коло(300, 200, 60)
```

`examples/star.yu`:

```
# Зірка: черепашка обходить п'ять кутів, а заливка її зафарбовує
фон("синій")
колір("жовтий")

# Відходимо до лівого кута зірки, нічого не малюючи
підніми_перо()
назад(150)
ліворуч(90)
вперед(35)
праворуч(90)
опусти_перо()

почни_заливку()
повтори 5 разів:
    вперед(300)
    праворуч(144)
заверши_заливку()
```

`examples/house.yu`:

```
# Будинок: прямокутники, дах черепашкою із заливкою, сонце й напис
фон("блакитний")
колір("жовтий")
коло(510, 80, 45)
колір("зелений")
прямокутник(0, 330, 600, 70)

колір("білий")
прямокутник(210, 230, 180, 120)
колір("коричневий")
прямокутник(280, 280, 40, 70)
колір("блакитний")
прямокутник(230, 250, 35, 35)
прямокутник(335, 250, 35, 35)

# Дах: черепашка йде до лівого краю стіни й обходить трикутник
колір("червоний")
підніми_перо()
назад(100)
праворуч(90)
вперед(30)
ліворуч(90)
опусти_перо()
почни_заливку()
повтори 3 рази:
    вперед(200)
    ліворуч(120)
заверши_заливку()

колір("білий")
напис("Мій дім", 24, 24)
```

`examples/spiral.yu`:

```
# A colourful spiral, written with English keywords
background("black")
colors = ["red", "orange", "yellow", "green", "lightblue", "purple"]
thickness(3)
for i from 1 to 110:
    color(colors[i % 6 + 1])
    forward(i * 1.6)
    right(59)
```

- [ ] **Step 2: Run the golden test to see it fail**

Run: `cargo test -p yu-core --test golden`
Expected: FAIL with `missing …/examples/<name>.svg` for one of the new examples.

- [ ] **Step 3: Generate the pictures with `yu`**

```bash
for f in flag sun star house spiral; do cargo run -q -p yu-cli -- examples/$f.yu --svg examples/$f.svg; done
```

Expected: no output; five `.svg` files in `examples/`. They are text with LF endings (`.gitattributes` covers `examples/*`).

- [ ] **Step 4: Look at every picture**

Create `target/preview.py` (`target/` is ignored by git):

```python
"""Renders every examples/*.svg in headless Chromium: 40 % into its animation and finished."""
import pathlib
import sys

from playwright.sync_api import sync_playwright

sys.stdout.reconfigure(encoding="utf-8")
EXE = r"C:\Users\home\AppData\Local\ms-playwright\chromium_headless_shell-1208\chrome-headless-shell-win64\chrome-headless-shell.exe"
ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "target" / "preview"
OUT.mkdir(parents=True, exist_ok=True)

with sync_playwright() as p:
    browser = p.chromium.launch(executable_path=EXE)
    page = browser.new_page(viewport={"width": 600, "height": 400})
    for svg in sorted((ROOT / "examples").glob("*.svg")):
        page.goto(svg.as_uri())
        end = page.evaluate(
            "Math.max(0, ...document.getAnimations().map(a => a.effect.getComputedTiming().endTime))"
        )
        for name, t in (("mid", end * 0.4), ("end", end + 100)):
            page.evaluate(
                "t => document.getAnimations().forEach(a => { a.pause(); a.currentTime = t; })", t
            )
            page.screenshot(path=str(OUT / f"{svg.stem}-{name}.png"))
        count = page.evaluate("document.getAnimations().length")
        print(f"{svg.stem}: {count} animations, {end:.0f} ms")
    browser.close()
```

Run: `uv run --no-project --with playwright==1.48.0 python target/preview.py`, then open every `target/preview/*-end.png` and `*-mid.png`.

Expected, finished pictures: flag — blue over yellow with a white «Україна» top left; sun — twelve yellow rays round a yellow circle on light blue, the green turtle in the middle; star — a solid yellow five-pointed star on blue; house — white walls, a red roof, a brown door, two windows, a sun, grass and «Мій дім»; spiral — a colourful spiral on black that stays inside the canvas. The mid pictures are partly drawn. Every animation ends within about 4.3 s.

If a picture is cut off or something overlaps badly, change that example's numbers, regenerate its SVG (Step 3) and look again.

- [ ] **Step 5: Run the golden test to see it pass**

Run: `cargo test -p yu-core --test golden`
Expected: 1 passed.

- [ ] **Step 6: Add the drawing section to the README**

In `README.md`, insert before `## Як це влаштовано`:

````markdown
## Малювання

<p align="center">
  <img src="examples/star.svg" width="32%" alt="Жовта зірка на синьому тлі">
  <img src="examples/house.svg" width="32%" alt="Будинок із червоним дахом під сонцем">
  <img src="examples/spiral.svg" width="32%" alt="Кольорова спіраль на чорному тлі">
</p>

Картинки анімовані: так їх малює `yu`. Зірка — це [`examples/star.yu`](examples/star.yu), а її серце —
кілька рядків:

```
колір("жовтий")
почни_заливку()
повтори 5 разів:
    вперед(300)
    праворуч(144)
заверши_заливку()
```

| Українською | English | |
|---|---|---|
| `полотно(ш, в)` | `canvas(w, h)` | розмір полотна (спершу 600×400) |
| `фон("колір")` | `background("colour")` | зафарбувати все полотно |
| `колір("колір")` | `color("colour")` | колір наступних фігур і ліній |
| `товщина(n)` | `thickness(n)` | товщина ліній |
| `коло(x, y, радіус)` | `circle(x, y, r)` | зафарбоване коло |
| `прямокутник(x, y, ш, в)` | `rect(x, y, w, h)` | зафарбований прямокутник |
| `лінія(x1, y1, x2, y2)` | `line(x1, y1, x2, y2)` | лінія |
| `напис(текст, x, y)` | `label(text, x, y)` | текст |
| `вперед(n)`, `назад(n)` | `forward(n)`, `back(n)` | черепашка йде й малює |
| `праворуч(°)`, `ліворуч(°)` | `right(°)`, `left(°)` | черепашка повертає |
| `підніми_перо()`, `опусти_перо()` | `pen_up()`, `pen_down()` | іти без лінії / знову малювати |
| `почни_заливку()`, `заверши_заливку()` | `begin_fill()`, `end_fill()` | зафарбувати фігуру, яку обійшла черепашка |

Кольори: червоний, помаранчевий, жовтий, зелений, блакитний, синій, фіолетовий, рожевий, білий,
чорний, сірий, коричневий — у будь-якому роді («червона», «червоне»), англійською або `"#ff8800"`.
Синій і жовтий — кольори прапора. `колір("випадковий")` щоразу бере інший яскравий колір.
Координати йдуть від лівого верхнього кута, y росте вниз; черепашка стартує в центрі й дивиться
праворуч.

```bash
cargo run -p yu-cli -- examples/house.yu --svg house.svg   # зберегти малюнок
```

````

and replace the whole `## Як це влаштовано` section (up to `## Ліцензія`) with:

```markdown
## Як це влаштовано

`crates/yu-core` — лексер з відступами, парсер (рекурсивний спуск + Pratt), дерево-інтерпретатор,
малювання (`draw.rs`) з єдиним рендерером SVG (`svg.rs`) і двомовні повідомлення; без сторонніх
залежностей, зовнішній світ — лише через трейт `Host`. `crates/yu-cli` — команда `yu`. Тести:
`cargo test --all` (кожна програма в `examples/` звіряється з очікуваним виводом і малюнком).

Далі: Yu Studio — редактор, вивід і малюнок в одному вікні браузера.

```

- [ ] **Step 7: Check everything and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --all
git add crates/yu-core/tests/golden.rs README.md examples/flag.yu examples/flag.svg examples/sun.yu examples/sun.svg examples/star.yu examples/star.svg examples/house.yu examples/house.svg examples/spiral.yu examples/spiral.svg
git -c user.email=y.dmytrenko14@gmail.com -c user.name="Yurii Dmytrenko" commit -m "Drawing gallery: five examples with golden SVGs and a README section" -m "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
```

---

## After the last task

Run the whole check once more on the finished branch (`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all`), then finish the branch with superpowers:finishing-a-development-branch: push `m2-drawing`, open a pull request, wait for CI on Ubuntu and Windows, and open the README on the branch page on GitHub to see that the three animated pictures show up.
