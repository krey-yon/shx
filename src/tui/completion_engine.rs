//! Turning what the user typed into the things they might have meant.
//!
//! The engine owns a list of candidates and nothing else — no terminal, no
//! session, no terminal-width maths. That is what makes it testable, and it
//! is what lets the same engine serve history completion and command-name
//! completion later.
//!
//! Results are ranked by [`FuzzyMatcher`], and equal scores are broken
//! alphabetically so that the list never reorders between two runs.

use crate::tui::fuzzy_matcher::FuzzyMatcher;

/// A list of candidates, ranked against a query.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompletionEngine {
    candidates: Vec<String>,
}

impl CompletionEngine {
    /// An engine over `candidates`, in the order given.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::tui::completion_engine::CompletionEngine;
    ///
    /// let candidates = vec!["git status".to_owned(), "cargo test".to_owned()];
    /// let engine = CompletionEngine::new(&candidates);
    /// assert_eq!(engine.best("cgt", 5), vec!["cargo test"]);
    /// ```
    #[must_use]
    pub fn new(candidates: &[String]) -> Self {
        Self {
            candidates: candidates.to_vec(),
        }
    }

    /// Every candidate this engine can offer.
    #[must_use]
    pub fn candidates(&self) -> &[String] {
        &self.candidates
    }

    /// The candidates that match `query`, best first.
    ///
    /// An empty query matches everything, in the order the candidates were
    /// given, because there is nothing to rank by.
    #[must_use]
    pub fn filter(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            return self.candidates.clone();
        }
        let mut scored: Vec<(i32, &String)> = self
            .candidates
            .iter()
            .filter_map(|candidate| {
                FuzzyMatcher::score(query, candidate).map(|score| (score, candidate))
            })
            .collect();
        scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(right.1)));
        scored
            .into_iter()
            .map(|(_, candidate)| candidate.clone())
            .collect()
    }

    /// The best `limit` matches for `query`.
    #[must_use]
    pub fn best(&self, query: &str, limit: usize) -> Vec<String> {
        self.filter(query).into_iter().take(limit).collect()
    }

    /// Every candidate that starts with `prefix`, in the order given.
    ///
    /// This is the exact half of completion: the first tab of `git` lists
    /// what `git` starts, not everything that vaguely contains those letters.
    #[must_use]
    pub fn by_prefix(&self, prefix: &str) -> Vec<String> {
        let wanted = prefix.to_lowercase();
        self.candidates
            .iter()
            .filter(|candidate| candidate.to_lowercase().starts_with(&wanted))
            .cloned()
            .collect()
    }

    /// The longest prefix every one of `results` starts with.
    ///
    /// Tab-completion of a query that several candidates match extends the
    /// word to this and no further, so an ambiguous list leaves the user
    /// where they were instead of guessing.
    #[must_use]
    pub fn common_prefix(&self, results: &[String]) -> String {
        let Some(first) = results.first() else {
            return String::new();
        };
        let mut shared = first.chars().count();
        for candidate in &results[1..] {
            let agreement = first
                .chars()
                .zip(candidate.chars())
                .take_while(|(left, right)| left == right)
                .count();
            shared = shared.min(agreement);
        }
        first.chars().take(shared).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::CompletionEngine;

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn engine(candidates: &[&str]) -> CompletionEngine {
        CompletionEngine::new(&owned(candidates))
    }

    #[test]
    fn the_candidates_come_back_as_given() {
        let engine = engine(&["git status", "cargo test"]);
        assert_eq!(engine.candidates(), ["git status", "cargo test"]);
    }

    #[test]
    fn filtering_keeps_only_what_matches() {
        let engine = engine(&["git status", "cargo test", "brew install ripgrep"]);
        assert_eq!(engine.filter("cgt"), vec!["cargo test"]);
        assert_eq!(engine.filter("xyzz"), Vec::<String>::new());
    }

    #[test]
    fn filtering_orders_by_score() {
        let engine = engine(&["cargotest", "cargo fmt", "cargo test"]);
        assert_eq!(
            engine.filter("cgt"),
            vec!["cargo test", "cargotest", "cargo fmt"]
        );
    }

    #[test]
    fn equal_scores_come_back_alphabetically() {
        let engine = engine(&["cargo", "cacao"]);
        assert_eq!(engine.filter("c"), vec!["cacao", "cargo"]);
    }

    #[test]
    fn an_empty_query_returns_everything_as_given() {
        let engine = engine(&["cargo test", "git status", "brew install ripgrep"]);
        assert_eq!(
            engine.filter(""),
            vec!["cargo test", "git status", "brew install ripgrep"]
        );
    }

    #[test]
    fn no_matches_returns_nothing() {
        let engine = engine(&["git status", "cargo test"]);
        assert!(engine.filter("zzzz").is_empty());
    }

    #[test]
    fn best_respects_the_limit() {
        let engine = engine(&["git status", "git stash", "git switch", "cargo test"]);
        let best = engine.best("git", 2);
        assert_eq!(best.len(), 2);
        assert!(best.iter().all(|candidate| candidate.starts_with("git")));
    }

    #[test]
    fn best_of_nothing_is_nothing() {
        let engine = engine(&["git status"]);
        assert!(engine.best("zzzz", 5).is_empty());
    }

    #[test]
    fn a_limit_of_zero_returns_nothing() {
        let engine = engine(&["git status"]);
        assert!(engine.best("git", 0).is_empty());
    }

    #[test]
    fn a_limit_larger_than_the_list_returns_all_of_it() {
        let engine = engine(&["git status", "git stash"]);
        assert_eq!(engine.best("git", 99).len(), 2);
    }

    #[test]
    fn prefix_matching_is_exact_and_ordered() {
        let engine = engine(&["git status", "cargo test", "git stash", "regit"]);
        assert_eq!(engine.by_prefix("git"), vec!["git status", "git stash"]);
    }

    #[test]
    fn prefix_matching_ignores_case() {
        let engine = engine(&["Git status", "cargo test"]);
        assert_eq!(engine.by_prefix("gi"), vec!["Git status"]);
    }

    #[test]
    fn an_empty_prefix_matches_everything() {
        let engine = engine(&["git status", "cargo test"]);
        assert_eq!(engine.by_prefix("").len(), 2);
    }

    #[test]
    fn a_prefix_that_matches_nothing_returns_nothing() {
        assert!(engine(&["git status"]).by_prefix("zzz").is_empty());
    }

    #[test]
    fn the_common_prefix_runs_to_the_first_difference() {
        let engine = engine(&["git status", "git stash"]);
        assert_eq!(
            engine.common_prefix(&owned(&["git status", "git stash"])),
            "git sta"
        );
    }

    #[test]
    fn one_result_is_its_own_common_prefix() {
        let engine = engine(&["git status"]);
        assert_eq!(engine.common_prefix(&owned(&["git status"])), "git status");
    }

    #[test]
    fn unrelated_results_have_no_common_prefix() {
        let engine = engine(&["git status", "cargo test"]);
        assert_eq!(
            engine.common_prefix(&owned(&["git status", "cargo test"])),
            ""
        );
    }

    #[test]
    fn no_results_have_no_common_prefix() {
        assert_eq!(engine(&["git"]).common_prefix(&[]), "");
    }

    #[test]
    fn a_shared_prefix_can_be_completed_into_a_whole_candidate() {
        let engine = engine(&["git status", "git stash", "git switch"]);
        let query = "git s";
        let top = engine.best(query, 3);
        let shared = engine.common_prefix(&top);
        assert!(top.iter().all(|candidate| candidate.starts_with(&shared)));
        assert_eq!(shared, "git s");
    }

    #[test]
    fn an_unambiguous_query_extends_to_one_candidate() {
        let engine = engine(&["git status", "git stash", "cargo test"]);
        let top = engine.best("cat", 3);
        assert_eq!(top, vec!["cargo test"]);
        assert_eq!(engine.common_prefix(&top), "cargo test");
    }

    #[test]
    fn the_same_query_always_gives_the_same_list() {
        let engine = engine(&[
            "git status",
            "git stash",
            "cargo test",
            "brew install ripgrep",
        ]);
        let first = engine.filter("gt");
        for _ in 0..8 {
            assert_eq!(engine.filter("gt"), first);
        }
    }

    #[test]
    fn an_engine_with_no_candidates_offers_nothing() {
        let engine = engine(&[]);
        assert!(engine.filter("git").is_empty());
        assert!(engine.best("git", 5).is_empty());
        assert!(engine.by_prefix("git").is_empty());
    }
}
