/// Levenshtein distance, counted in characters.
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The nearest candidate that is close enough to be a typo; the first one wins a tie.
pub fn closest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = match word.chars().count() {
        0..=2 => 1,
        3..=5 => 2,
        _ => 3,
    };
    let word = word.to_lowercase();
    candidates
        .into_iter()
        .map(|c| (distance(&word, &c.to_lowercase()), c))
        .filter(|(d, _)| *d > 0 && *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_counts_character_edits() {
        assert_eq!(distance("скаж", "скажи"), 1);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("", "abc"), 3);
    }

    #[test]
    fn closest_picks_a_near_name_or_nothing() {
        assert_eq!(closest("довжна", ["додай", "довжина"]), Some("довжина"));
        assert_eq!(closest("xyz", ["скажи"]), None);
        assert_eq!(closest("скажи", ["скажи"]), None);
    }
}
