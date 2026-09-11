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
}
