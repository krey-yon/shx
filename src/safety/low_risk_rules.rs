//! Commands that remove a package or move a file somewhere temporary.
//!
//! A single yes is enough. These are routine, and asking twice would train the
//! user to answer without reading.

use std::sync::LazyLock;

use crate::safety::{RiskLevel, RuleSet};

static RULES: LazyLock<RuleSet> = LazyLock::new(|| {
    RuleSet::new(
        RiskLevel::Low,
        [
            (
                r"\b(?:apt|dnf|yum|brew|apk)\s+(?:remove|purge|uninstall|del)\b",
                "This removes a package.",
            ),
            (
                r"\b(?:pacman|zypper|apt)\s+-[a-zA-Z]*[RS](?:\s|$)",
                "This removes a package.",
            ),
            (
                r"\b(?:pip|pip3|npm|yarn|pnpm)\s+uninstall\b",
                "This removes a package.",
            ),
            (
                r"\bmv\b[^\n]*\s/tmp/\s*$",
                "This moves things to /tmp, which is often cleared on reboot.",
            ),
            (r"\btruncate\b", "This empties a file."),
        ],
    )
});

/// The low-risk rules.
///
/// # Examples
///
/// ```
/// use shx::safety::low_risk_rules::rules;
///
/// assert!(rules().matches("apt remove ripgrep"));
/// assert!(!rules().matches("apt install ripgrep"));
/// ```
#[must_use]
pub fn rules() -> &'static RuleSet {
    &RULES
}

#[cfg(test)]
mod tests {
    use crate::safety::RiskLevel;
    use crate::safety::low_risk_rules::rules;

    #[test]
    fn the_tier_is_low() {
        assert_eq!(rules().level, RiskLevel::Low);
    }

    #[test]
    fn package_removal_is_flagged_across_package_managers() {
        for command in [
            "apt remove ripgrep",
            "sudo apt purge ripgrep",
            "dnf remove ripgrep",
            "brew uninstall ripgrep",
            "pacman -R ripgrep",
            "apk del ripgrep",
            "pip uninstall requests",
            "npm uninstall lodash",
        ] {
            assert!(rules().matches(command), "{command:?} should be low risk");
        }
    }

    #[test]
    fn installing_a_package_is_not_flagged() {
        for command in [
            "apt install ripgrep",
            "brew install ripgrep",
            "npm install lodash",
            "cargo install ripgrep",
        ] {
            assert!(!rules().matches(command), "{command:?} should be safe");
        }
    }

    #[test]
    fn moving_to_tmp_is_flagged() {
        assert!(rules().matches("mv report.pdf /tmp/"));
    }

    #[test]
    fn emptying_a_file_is_flagged() {
        assert!(rules().matches("truncate -s 0 access.log"));
    }

    #[test]
    fn an_ordinary_listing_is_not_flagged() {
        assert!(!rules().matches("ls -la /tmp"));
    }
}
