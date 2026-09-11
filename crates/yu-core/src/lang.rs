/// Language of error messages and of printed `так`/`ні`/`нічого`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Uk,
    En,
}

impl Lang {
    /// The Ukrainian or the English text.
    pub fn pick<'a>(self, uk: &'a str, en: &'a str) -> &'a str {
        match self {
            Lang::Uk => uk,
            Lang::En => en,
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        match code {
            "uk" | "ua" => Some(Lang::Uk),
            "en" => Some(Lang::En),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_chooses_by_language() {
        assert_eq!(Lang::Uk.pick("так", "yes"), "так");
        assert_eq!(Lang::En.pick("так", "yes"), "yes");
    }

    #[test]
    fn codes() {
        assert_eq!(Lang::from_code("uk"), Some(Lang::Uk));
        assert_eq!(Lang::from_code("ua"), Some(Lang::Uk));
        assert_eq!(Lang::from_code("en"), Some(Lang::En));
        assert_eq!(Lang::from_code("de"), None);
        assert_eq!(Lang::default(), Lang::Uk);
    }
}
