//! What the user has to type to confirm a command.

use crate::safety::RiskLevel;

/// The prompt to show for a level, and the answers that accept it.
///
/// Three different confirmations, matched to how much a mistake costs. A single
/// `y` everywhere would make a capitalised confirmation meaningless, and
/// demanding capitals for a package removal would train the user to type them
/// for everything.
///
/// # Examples
///
/// ```
/// use shx::safety::{RiskLevel, confirmation_policy::prompt_for};
///
/// assert!(prompt_for(RiskLevel::High).contains("YES"));
/// assert!(prompt_for(RiskLevel::Critical).contains("BLOCKED"));
/// ```
#[must_use]
pub fn prompt_for(level: RiskLevel) -> &'static str {
    match level {
        RiskLevel::Safe => "Run this command? [y/N]",
        RiskLevel::Low => "Are you sure? [y/N]",
        RiskLevel::Medium => "This changes your system. Type 'yes' to continue:",
        RiskLevel::High => "This can lose a lot of data. Type 'YES' in capitals to continue:",
        RiskLevel::Critical => "BLOCKED: this command is too dangerous to run.",
    }
}

/// Whether an answer confirms the command at this level.
///
/// The answer is compared exactly, and the comparison is case-sensitive on
/// purpose. `yes` must not satisfy a demand for `YES`: the capitals are the
/// entire point of the high tier, and accepting both would make a stray Enter
/// equivalent to a considered decision.
///
/// # Examples
///
/// ```
/// use shx::safety::{RiskLevel, confirmation_policy::confirms};
///
/// assert!(confirms("yes", RiskLevel::Medium));
/// assert!(!confirms("YES", RiskLevel::Medium), "case matters at the high tier");
/// assert!(confirms("YES", RiskLevel::High));
/// assert!(!confirms("yes", RiskLevel::High));
/// assert!(confirms("y", RiskLevel::Safe));
/// assert!(!confirms("n", RiskLevel::Safe));
/// assert!(!confirms("YES", RiskLevel::Critical), "critical is never confirmable");
/// ```
#[must_use]
pub fn confirms(answer: &str, level: RiskLevel) -> bool {
    let answer = answer.trim();

    match level {
        RiskLevel::Critical => false,
        RiskLevel::High => answer == "YES",
        RiskLevel::Medium => answer == "yes",
        RiskLevel::Low | RiskLevel::Safe => {
            answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes")
        }
    }
}

/// Alternatives to offer for a risky command.
///
/// # Examples
///
/// ```
/// use shx::safety::{RiskLevel, confirmation_policy::suggestions_for};
///
/// let for_delete = suggestions_for(RiskLevel::High, "rm -rf ./build");
/// assert!(for_delete.iter().any(|s| s.contains("trash")));
///
/// assert!(suggestions_for(RiskLevel::Safe, "ls").is_empty());
/// ```
#[must_use]
pub fn suggestions_for(level: RiskLevel, command: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    if level.at_least(RiskLevel::Medium) && command.contains("rm ") {
        suggestions.push("Move it somewhere safe first, or use `trash` if you have it.".to_owned());
    }

    if level.at_least(RiskLevel::Medium) && command.contains("git reset") {
        suggestions.push("`git stash` keeps your changes recoverable.".to_owned());
    }

    if level.at_least(RiskLevel::Low) && command.contains("sudo") {
        suggestions.push("Double-check you actually need elevated privileges.".to_owned());
    }

    if level.at_least(RiskLevel::Low) && command.contains("/tmp/") {
        suggestions.push("/tmp is usually cleared on reboot.".to_owned());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use crate::safety::RiskLevel;
    use crate::safety::confirmation_policy::{confirms, prompt_for, suggestions_for};

    #[test]
    fn each_level_has_its_own_prompt() {
        let mut prompts: Vec<&str> = [
            RiskLevel::Safe,
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ]
        .iter()
        .map(|level| prompt_for(*level))
        .collect();

        let count = prompts.len();
        prompts.sort_unstable();
        prompts.dedup();
        assert_eq!(prompts.len(), count, "two levels share a prompt");
    }

    #[test]
    fn the_high_prompt_shows_the_word_it_wants() {
        assert!(prompt_for(RiskLevel::High).contains("YES"));
    }

    #[test]
    fn the_medium_prompt_shows_the_word_it_wants() {
        assert!(prompt_for(RiskLevel::Medium).contains("yes"));
    }

    #[test]
    fn the_critical_prompt_says_blocked() {
        assert!(prompt_for(RiskLevel::Critical).contains("BLOCKED"));
    }

    #[test]
    fn the_safe_prompt_offers_a_default_of_no() {
        assert!(prompt_for(RiskLevel::Safe).contains("[y/N]"));
    }

    #[test]
    fn the_safe_and_low_tiers_accept_anything_that_means_yes() {
        for level in [RiskLevel::Safe, RiskLevel::Low] {
            for answer in ["y", "Y", "yes", "YES", "Yes", " y "] {
                assert!(confirms(answer, level), "{answer:?} should confirm {level}");
            }
            for answer in ["", "n", "no", "yep", "sure"] {
                assert!(
                    !confirms(answer, level),
                    "{answer:?} should not confirm {level}"
                );
            }
        }
    }

    #[test]
    fn the_medium_tier_wants_exactly_lowercase_yes() {
        assert!(confirms("yes", RiskLevel::Medium));
        assert!(confirms(" yes ", RiskLevel::Medium));
        assert!(!confirms("YES", RiskLevel::Medium));
        assert!(!confirms("y", RiskLevel::Medium));
        assert!(!confirms("", RiskLevel::Medium));
    }

    #[test]
    fn the_high_tier_wants_exactly_capital_yes() {
        assert!(confirms("YES", RiskLevel::High));
        assert!(confirms(" YES ", RiskLevel::High));
        assert!(!confirms("yes", RiskLevel::High));
        assert!(!confirms("Y", RiskLevel::High));
        assert!(!confirms("", RiskLevel::High));
    }

    #[test]
    fn nothing_confirms_a_critical_command() {
        for answer in ["", "y", "yes", "YES", "YES PLEASE", "sudo"] {
            assert!(
                !confirms(answer, RiskLevel::Critical),
                "{answer:?} must not confirm a critical command"
            );
        }
    }

    #[test]
    fn a_recursive_delete_suggests_a_reversible_alternative() {
        let suggestions = suggestions_for(RiskLevel::High, "rm -rf ./build");
        assert!(
            suggestions.iter().any(|s| s.contains("trash")),
            "got {suggestions:?}"
        );
    }

    #[test]
    fn a_git_reset_suggests_stashing() {
        let suggestions = suggestions_for(RiskLevel::High, "git reset --hard HEAD~1");
        assert!(suggestions.iter().any(|s| s.contains("stash")));
    }

    #[test]
    fn a_safe_command_gets_no_suggestions() {
        assert!(suggestions_for(RiskLevel::Safe, "ls -la").is_empty());
        assert!(suggestions_for(RiskLevel::Safe, "rm -rf ./build").is_empty());
    }

    #[test]
    fn sudo_is_suggested_against_only_once() {
        let suggestions = suggestions_for(RiskLevel::Medium, "sudo rm -rf ./x");
        let count = suggestions
            .iter()
            .filter(|s| s.contains("elevated"))
            .count();
        assert_eq!(count, 1, "got {suggestions:?}");
    }

    #[test]
    fn no_suggestion_appears_twice() {
        for command in [
            "sudo rm -rf /tmp/x",
            "rm -rf ./build",
            "sudo apt remove ripgrep",
        ] {
            let suggestions = suggestions_for(RiskLevel::High, command);
            let mut sorted = suggestions.clone();
            sorted.sort();
            let before = sorted.len();
            sorted.dedup();
            assert_eq!(sorted.len(), before, "{command:?} produced a duplicate");
        }
    }
}
