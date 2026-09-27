//! Fuzzy matching, the scoring half of shell completion.
//!
//! Typing `gst` should offer `git status`, even though those letters are not
//! adjacent. The rule is a subsequence match — every character of the needle
//! appears in the haystack, in order — and the score says how good a match it
//! is. A match scores better when its characters are adjacent, when they land
//! on word boundaries, when they start early, when the case is what was typed,
//! and when the haystack is short.
//!
//! The match taken is always the leftmost one, so the score is a function of
//! the two strings alone: no map iteration, no floating point, same answer
//! every time.

/// What a match at a word boundary is worth.
const BOUNDARY: i32 = 12;
/// What a character adjacent to the previous match is worth.
const CONSECUTIVE: i32 = 16;
/// What typing the character's own case is worth.
const EXACT_CASE: i32 = 2;
/// The most a single gap between matches can cost.
const GAP_PENALTY: i32 = 8;
/// The most a late first match can cost.
const POSITION_PENALTY: i32 = 12;

/// Scores a needle against a haystack, for completion.
#[derive(Debug, Clone, Copy)]
pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// How well `needle` matches `haystack`, or `None` when it does not.
    ///
    /// Matching ignores case: the needle must be a subsequence of the
    /// haystack, with no character repeated out of order. An empty needle
    /// matches everything and scores `0`, below every real match.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::tui::fuzzy_matcher::FuzzyMatcher;
    ///
    /// assert!(FuzzyMatcher::score("gst", "git status").is_some());
    /// assert_eq!(FuzzyMatcher::score("xyz", "git status"), None);
    /// assert_eq!(FuzzyMatcher::score("", "git"), Some(0));
    ///
    /// let prefix = FuzzyMatcher::score("git", "git");
    /// let middle = FuzzyMatcher::score("git", "a git repo");
    /// assert!(prefix > middle);
    /// ```
    #[must_use]
    pub fn score(needle: &str, haystack: &str) -> Option<i32> {
        if needle.is_empty() {
            return Some(0);
        }
        let haystack: Vec<char> = haystack.chars().collect();
        let mut total = 0;
        let mut from = 0;
        let mut previous: Option<usize> = None;
        for wanted in needle.chars() {
            let found = haystack[from..]
                .iter()
                .position(|candidate| same_letter(*candidate, wanted))
                .map(|offset| offset + from)?;
            total -= score_size(found).min(POSITION_PENALTY);
            if is_boundary(&haystack, found) {
                total += BOUNDARY;
            }
            if haystack[found] == wanted {
                total += EXACT_CASE;
            }
            if let Some(before) = previous {
                total += if found == before + 1 {
                    CONSECUTIVE
                } else {
                    -score_size(found - before - 1).min(GAP_PENALTY)
                };
            }
            previous = Some(found);
            from = found + 1;
        }
        Some(total - score_size(haystack.len()))
    }
}

fn score_size(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn same_letter(candidate: char, wanted: char) -> bool {
    candidate.to_lowercase().eq(wanted.to_lowercase())
}

fn is_boundary(haystack: &[char], at: usize) -> bool {
    at == 0 || !is_word_char(haystack[at - 1])
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::FuzzyMatcher;

    fn score(needle: &str, haystack: &str) -> i32 {
        FuzzyMatcher::score(needle, haystack)
            .unwrap_or_else(|| panic!("{needle:?} should match {haystack:?}"))
    }

    #[test]
    fn adjacent_characters_beat_scattered_ones() {
        assert!(score("gs", "gs----") > score("gs", "g----s"));
    }

    #[test]
    fn a_prefix_beats_a_mid_string_match() {
        assert!(score("git", "git") > score("git", "a git repo"));
        assert!(score("g", "git") > score("g", "legit"));
    }

    #[test]
    fn a_word_boundary_beats_the_middle_of_a_word() {
        assert!(score("s", "cargo status") > score("s", "cargostatus"));
    }

    #[test]
    fn a_shorter_haystack_wins_at_equal_quality() {
        assert!(score("git", "git") > score("git", "git with a long tail"));
    }

    #[test]
    fn an_earlier_match_beats_a_later_one() {
        assert!(score("git", "git") > score("git", "cargo test git"));
    }

    #[test]
    fn matching_ignores_case() {
        assert!(FuzzyMatcher::score("GIT", "git status").is_some());
        assert!(FuzzyMatcher::score("git", "GIT STATUS").is_some());
        assert!(FuzzyMatcher::score("RUST", "rust").is_some());
    }

    #[test]
    fn the_case_that_was_typed_scores_higher() {
        assert!(score("git", "git") > score("Git", "git"));
        assert!(score("Git", "git") > score("GIT", "git"));
    }

    #[test]
    fn an_empty_needle_matches_everything() {
        for haystack in ["", "git", "a long one"] {
            assert_eq!(FuzzyMatcher::score("", haystack), Some(0));
        }
    }

    #[test]
    fn characters_out_of_order_do_not_match() {
        assert_eq!(FuzzyMatcher::score("tg", "git"), None);
        assert_eq!(FuzzyMatcher::score("xyz", "git status"), None);
    }

    #[test]
    fn a_needle_longer_than_the_haystack_does_not_match() {
        assert_eq!(FuzzyMatcher::score("git status", "git"), None);
    }

    #[test]
    fn the_same_input_always_scores_the_same() {
        for (needle, haystack) in [("gs", "git status"), ("rst", "cargo rst")] {
            let first = FuzzyMatcher::score(needle, haystack);
            for _ in 0..8 {
                assert_eq!(FuzzyMatcher::score(needle, haystack), first);
            }
        }
    }

    #[test]
    fn an_exact_match_outscores_every_near_miss() {
        let exact = score("git", "git");
        for haystack in ["gitk", "a git", "git status", "legit"] {
            assert!(
                exact > score("git", haystack),
                "git should beat {haystack:?}"
            );
        }
    }

    #[test]
    fn a_wide_gap_costs_more_than_a_narrow_one() {
        assert!(score("gs", "git status") > score("gs", "git --show-status"));
    }

    #[test]
    fn every_letter_of_the_needle_is_matched() {
        assert!(FuzzyMatcher::score("gt", "git").is_some());
        assert!(FuzzyMatcher::score("gt", "gate").is_some());
        assert_eq!(FuzzyMatcher::score("gt", "title"), None);
        assert_eq!(FuzzyMatcher::score("gtt", "title"), None);
    }

    #[test]
    fn multibyte_haystacks_are_matched_by_character() {
        assert!(FuzzyMatcher::score("é", "café").is_some());
        assert!(FuzzyMatcher::score("caf", "café").is_some());
        assert_eq!(FuzzyMatcher::score("é", "cafe"), None);
    }

    #[test]
    fn an_underscore_does_not_start_a_word() {
        assert!(score("b", "foo bar") > score("b", "foo_bar"));
    }

    #[test]
    fn the_command_itself_wins_in_a_real_command_list() {
        let candidates = ["cargotest", "cargo fmt", "cargo test", "git status"];
        let best = candidates
            .iter()
            .filter_map(|candidate| {
                FuzzyMatcher::score("cgt", candidate).map(|score| (score, *candidate))
            })
            .max_by_key(|(score, _)| *score)
            .map(|(_, candidate)| candidate);
        assert_eq!(best, Some("cargo test"));
    }
}
