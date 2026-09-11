/// Every keyword, whichever language it was written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    If,
    Else,
    While,
    Repeat,
    Times,
    For,
    From,
    To,
    Step,
    In,
    Function,
    Return,
    Break,
    Continue,
    And,
    Or,
    Not,
    True,
    False,
    Nothing,
}

/// Every spelling of every keyword; the lexer and the Studio's highlighter both read it.
pub const SPELLINGS: [(&str, Keyword); 43] = [
    ("якщо", Keyword::If),
    ("if", Keyword::If),
    ("інакше", Keyword::Else),
    ("else", Keyword::Else),
    ("поки", Keyword::While),
    ("while", Keyword::While),
    ("повтори", Keyword::Repeat),
    ("repeat", Keyword::Repeat),
    ("раз", Keyword::Times),
    ("рази", Keyword::Times),
    ("разів", Keyword::Times),
    ("times", Keyword::Times),
    ("для", Keyword::For),
    ("for", Keyword::For),
    ("від", Keyword::From),
    ("from", Keyword::From),
    ("до", Keyword::To),
    ("to", Keyword::To),
    ("крок", Keyword::Step),
    ("step", Keyword::Step),
    ("у", Keyword::In),
    ("в", Keyword::In),
    ("in", Keyword::In),
    ("функція", Keyword::Function),
    ("function", Keyword::Function),
    ("поверни", Keyword::Return),
    ("return", Keyword::Return),
    ("стоп", Keyword::Break),
    ("break", Keyword::Break),
    ("далі", Keyword::Continue),
    ("continue", Keyword::Continue),
    ("і", Keyword::And),
    ("and", Keyword::And),
    ("або", Keyword::Or),
    ("or", Keyword::Or),
    ("не", Keyword::Not),
    ("not", Keyword::Not),
    ("так", Keyword::True),
    ("true", Keyword::True),
    ("ні", Keyword::False),
    ("false", Keyword::False),
    ("нічого", Keyword::Nothing),
    ("nothing", Keyword::Nothing),
];

/// Ukrainian and English spellings of each keyword.
pub fn keyword(word: &str) -> Option<Keyword> {
    SPELLINGS.iter().find(|(w, _)| *w == word).map(|(_, k)| *k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_languages_give_the_same_keyword() {
        for (uk, en) in [
            ("якщо", "if"),
            ("поки", "while"),
            ("функція", "function"),
            ("поверни", "return"),
            ("нічого", "nothing"),
        ] {
            assert_eq!(keyword(uk), keyword(en));
            assert!(keyword(uk).is_some());
        }
        assert_eq!(keyword("разів"), Some(Keyword::Times));
        assert_eq!(keyword("в"), Some(Keyword::In));
        assert_eq!(keyword("скажи"), None);
    }

    #[test]
    fn every_spelling_is_listed_once() {
        let mut words: Vec<&str> = SPELLINGS.iter().map(|(w, _)| *w).collect();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), SPELLINGS.len());
        assert_eq!(SPELLINGS.len(), 43);
        for (word, kind) in SPELLINGS {
            assert_eq!(keyword(word), Some(kind), "{word}");
        }
    }
}
