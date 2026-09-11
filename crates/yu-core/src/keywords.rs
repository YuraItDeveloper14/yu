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

/// Ukrainian and English spellings of each keyword.
pub fn keyword(word: &str) -> Option<Keyword> {
    use Keyword::*;
    Some(match word {
        "якщо" | "if" => If,
        "інакше" | "else" => Else,
        "поки" | "while" => While,
        "повтори" | "repeat" => Repeat,
        "раз" | "рази" | "разів" | "times" => Times,
        "для" | "for" => For,
        "від" | "from" => From,
        "до" | "to" => To,
        "крок" | "step" => Step,
        "у" | "в" | "in" => In,
        "функція" | "function" => Function,
        "поверни" | "return" => Return,
        "стоп" | "break" => Break,
        "далі" | "continue" => Continue,
        "і" | "and" => And,
        "або" | "or" => Or,
        "не" | "not" => Not,
        "так" | "true" => True,
        "ні" | "false" => False,
        "нічого" | "nothing" => Nothing,
        _ => return None,
    })
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
}
