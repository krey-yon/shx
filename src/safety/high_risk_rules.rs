//! Commands that lose a lot of data if run by mistake.
//!
//! Confirmable, but only by typing a word in capitals. A stray Enter cannot
//! produce it.

use std::sync::LazyLock;

use crate::safety::{RiskLevel, RuleSet};

static RULES: LazyLock<RuleSet> = LazyLock::new(|| {
    RuleSet::new(
        RiskLevel::High,
        [
            (
                r"\brm\s+-[rf]{1,2}\s+~/?\s*$",
                "This deletes your home directory.",
            ),
            (
                r"\brm\s+-[rf]{1,2}\s+(?:\*|\.|\.\.)\s*$",
                "This deletes everything in the current directory.",
            ),
            (
                r"\brm\s+-[rf]{1,2}\s+/\*",
                "This deletes everything under the filesystem root.",
            ),
            (
                r"\bgit\s+reset\s+--hard\b",
                "This discards uncommitted changes.",
            ),
            (r"\bgit\s+clean\s+-[a-z]*f", "This deletes untracked files."),
            (
                r"\bchmod\s+-R\s+0?777\b",
                "This makes a tree world-writable.",
            ),
            (r"\bkill\s+-9\s+-1\b", "This kills every process you own."),
        ],
    )
});

/// The high-risk rules.
///
/// # Examples
///
/// ```
/// use shx::safety::high_risk_rules::rules;
///
/// assert!(rules().matches("rm -rf ."));
/// assert!(!rules().matches("ls -la"));
/// ```
#[must_use]
pub fn rules() -> &'static RuleSet {
    &RULES
}

#[cfg(test)]
mod tests {
    use crate::safety::RiskLevel;
    use crate::safety::high_risk_rules::rules;

    #[test]
    fn the_tier_is_high() {
        assert_eq!(rules().level, RiskLevel::High);
    }

    #[test]
    fn deleting_the_current_directory_is_high() {
        for command in ["rm -rf .", "rm -rf *", "rm -fr ..", "rm -rf ~/"] {
            assert!(rules().matches(command), "{command:?} should be high");
        }
    }

    #[test]
    fn a_named_target_is_not_this_tier() {
        assert!(!rules().matches("rm -rf ./build"));
        assert!(!rules().matches("rm -rf src/main.rs"));
    }

    #[test]
    fn git_destructive_operations_are_high() {
        assert!(rules().matches("git reset --hard HEAD~1"));
        assert!(rules().matches("git clean -fd"));
        assert!(rules().matches("git clean -fdx"));
    }

    #[test]
    fn a_read_only_git_command_is_not_flagged() {
        for command in ["git status", "git log", "git diff", "git clean -n"] {
            assert!(!rules().matches(command), "{command:?} should be safe");
        }
    }

    #[test]
    fn a_recursive_world_writable_chmod_is_high() {
        assert!(rules().matches("chmod -R 777 /var/www"));
    }

    #[test]
    fn killing_everything_is_high() {
        assert!(rules().matches("kill -9 -1"));
    }
}
