//! Running the tiers in order and reporting what they found.

use crate::safety::confirmation_policy::suggestions_for;
use crate::safety::rule_set::RuleSet;
use crate::safety::{
    SafetyResult, critical_rules, high_risk_rules, low_risk_rules, medium_risk_rules,
};

/// Classify a command.
///
/// Tiers are checked most severe first and the first one that matches wins, so
/// `sudo rm -rf ~` is reported as high risk rather than as the medium risk that
/// `sudo` alone would give. Suggestions from the winning tier are attached.
///
/// # Examples
///
/// ```
/// use shx::safety::{RiskLevel, analyze_command};
///
/// assert_eq!(analyze_command("rm -rf /").risk_level, RiskLevel::Critical);
/// assert!(analyze_command("rm -rf /").blocked);
///
/// assert_eq!(analyze_command("ls -la").risk_level, RiskLevel::Safe);
/// ```
#[must_use]
pub fn analyze_command(command: &str) -> SafetyResult {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return SafetyResult::safe();
    }

    for set in tiers() {
        let warnings = set.warnings_for(trimmed);
        if warnings.is_empty() {
            continue;
        }

        return SafetyResult {
            risk_level: set.level,
            warnings: warnings.into_iter().map(ToOwned::to_owned).collect(),
            blocked: set.level.is_refused(),
            suggestions: suggestions_for(set.level, trimmed),
        };
    }

    SafetyResult::safe()
}

/// The tiers, most severe first.
///
/// # Examples
///
/// ```
/// use shx::safety::RiskLevel;
/// use shx::safety::safety_analyzer::tiers;
///
/// let levels: Vec<RiskLevel> = tiers().iter().map(|set| set.level).collect();
/// assert_eq!(levels, vec![RiskLevel::Critical, RiskLevel::High, RiskLevel::Medium, RiskLevel::Low]);
/// ```
#[must_use]
pub fn tiers() -> [&'static RuleSet; 4] {
    [
        critical_rules::rules(),
        high_risk_rules::rules(),
        medium_risk_rules::rules(),
        low_risk_rules::rules(),
    ]
}

#[cfg(test)]
mod tests {
    use crate::safety::safety_analyzer::tiers;
    use crate::safety::{RiskLevel, SafetyResult, analyze_command};

    fn level_of(command: &str) -> RiskLevel {
        analyze_command(command).risk_level
    }

    #[test]
    fn an_ordinary_command_is_safe() {
        for command in [
            "ls -la",
            "git status",
            "cargo build --release",
            "echo hello",
            "cat README.md",
        ] {
            assert_eq!(level_of(command), RiskLevel::Safe, "{command:?}");
        }
    }

    #[test]
    fn an_empty_command_is_safe() {
        assert_eq!(level_of(""), RiskLevel::Safe);
        assert_eq!(level_of("   "), RiskLevel::Safe);
    }

    #[test]
    fn each_tier_is_reachable() {
        assert_eq!(level_of("rm -rf /"), RiskLevel::Critical);
        assert_eq!(level_of("rm -rf ."), RiskLevel::High);
        assert_eq!(level_of("sudo systemctl restart nginx"), RiskLevel::Medium);
        assert_eq!(level_of("apt remove ripgrep"), RiskLevel::Low);
    }

    #[test]
    fn only_critical_is_blocked() {
        assert!(analyze_command("rm -rf /").blocked);
        for command in ["rm -rf .", "sudo ls", "apt remove ripgrep"] {
            assert!(
                !analyze_command(command).blocked,
                "{command:?} is confirmable"
            );
        }
    }

    #[test]
    fn the_most_severe_matching_tier_wins() {
        let result = analyze_command("sudo rm -rf .");
        assert_eq!(result.risk_level, RiskLevel::High);
        assert!(!result.blocked, "high is still confirmable");
    }

    #[test]
    fn a_command_that_trips_nothing_has_no_warnings() {
        let result = analyze_command("ls -la");
        assert!(result.warnings.is_empty());
        assert!(result.suggestions.is_empty());
        assert!(!result.needs_confirmation());
    }

    #[test]
    fn every_hit_produces_at_least_one_warning() {
        for command in ["rm -rf /", "rm -rf .", "sudo ls", "apt remove x"] {
            let result = analyze_command(command);
            assert!(!result.warnings.is_empty(), "{command:?} needs a warning");
        }
    }

    #[test]
    fn a_flagged_recursive_delete_gets_a_suggestion() {
        let result = analyze_command("sudo rm -rf ./build");
        assert_eq!(result.risk_level, RiskLevel::Medium);
        assert!(
            result.suggestions.iter().any(|s| s.contains("trash")),
            "got {:?}",
            result.suggestions
        );
    }

    #[test]
    fn an_unflagged_command_gets_no_suggestion_to_act_on() {
        assert!(analyze_command("rm -rf ./build").suggestions.is_empty());
    }

    #[test]
    fn leading_and_trailing_whitespace_does_not_change_the_verdict() {
        assert_eq!(level_of("   rm -rf /   "), RiskLevel::Critical);
        assert_eq!(level_of("\trm -rf ."), RiskLevel::High);
    }

    #[test]
    fn the_tiers_are_ordered_most_severe_first() {
        let levels: Vec<RiskLevel> = tiers().iter().map(|set| set.level).collect();
        assert_eq!(
            levels,
            vec![
                RiskLevel::Critical,
                RiskLevel::High,
                RiskLevel::Medium,
                RiskLevel::Low
            ]
        );
    }

    #[test]
    fn the_tiers_are_ordered_descending() {
        let levels: Vec<RiskLevel> = tiers().iter().map(|set| set.level).collect();
        for pair in levels.windows(2) {
            assert!(
                pair[0] > pair[1],
                "{:?} should come before {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn the_result_is_a_plain_value_that_compares_by_value() {
        assert_eq!(analyze_command("ls"), SafetyResult::safe());
    }
}
