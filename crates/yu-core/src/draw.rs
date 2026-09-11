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
    Named {
        stem,
        soft,
        en,
        rgb,
    }
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

/// At most this many shapes in one drawing, so an endless loop can't eat all memory.
pub const MAX_SHAPES: usize = 50_000;

/// One thing on the canvas. Each carries its own colour, so a renderer keeps no state.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCmd {
    Background(Rgb),
    Circle {
        x: f64,
        y: f64,
        r: f64,
        color: Rgb,
    },
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        color: Rgb,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Rgb,
        width: f64,
    },
    Label {
        text: String,
        x: f64,
        y: f64,
        color: Rgb,
    },
    /// A turtle fill; `slot` is its place in the animation.
    Fill {
        points: Vec<(f64, f64)>,
        color: Rgb,
        slot: usize,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgb = Rgb(0xef, 0x44, 0x44);

    #[test]
    fn colours_in_every_form_and_both_languages() {
        for word in [
            "червоний",
            "червона",
            "червоне",
            "червоні",
            "red",
            " Червона ",
            "RED",
        ] {
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
        let mut d = Drawing {
            color: RED,
            ..Drawing::default()
        };
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
        assert!(matches!(&d.cmds[1], DrawCmd::Fill { points, slot: 4, .. } if points.len() == 4));
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
        assert_eq!(
            d.rect(0.0, 0.0, 5.0, -2.0),
            Err(ErrorKind::NegativeSize(-2.0))
        );
        assert_eq!(d.set_thickness(-3.0), Err(ErrorKind::NegativeSize(-3.0)));
        let mut d = Drawing::default();
        for _ in 0..MAX_SHAPES {
            d.line(0.0, 0.0, 1.0, 1.0).unwrap();
        }
        assert_eq!(d.line(0.0, 0.0, 1.0, 1.0), Err(ErrorKind::TooManyShapes));
    }
}
