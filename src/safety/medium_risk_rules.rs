//! Commands that change the system, or force something to stop.
//!
//! Confirmable by typing a lowercase word. The cost of a mistake here is an
//! hour of fixing a running system, not a lost afternoon of edits.

use std::sync::LazyLock;

use crate::safety::{RiskLevel, RuleSet};

static RULES: LazyLock<RuleSet> = LazyLock::new(|| {
    RuleSet::new(
        RiskLevel::Medium,
        [
            (r"\bsudo\b", "This runs as root."),
            (r"\bchown\s+-R\b", "This changes ownership recursively."),
            (r"\bchmod\s+-R\b", "This changes permissions recursively."),
            (
                r"\bkill\s+-9\b|\bkillall\s+-9\b",
                "This force-kills processes.",
            ),
            (r"\biptables\s+-F\b", "This flushes all firewall rules."),
            (r"\bcrontab\s+-r\b", "This removes every cron job."),
            (
                r"\bumount\b[^\n]*(?:force|-f)",
                "This force-unmounts a filesystem.",
            ),
            (
                r"\bdocker\s+system\s+prune\b[^\n]*-a",
                "This removes all unused Docker data.",
            ),
        ],
    )
});

/// The medium-risk rules.
///
/// # Examples
///
/// ```
/// use shx::safety::medium_risk_rules::rules;
///
/// assert!(rules().matches("sudo apt install ripgrep"));
/// assert!(!rules().matches("apt install ripgrep"));
/// ```
#[must_use]
pub fn rules() -> &'static RuleSet {
    &RULES
}

#[cfg(test)]
mod tests {
    use crate::safety::RiskLevel;
    use crate::safety::medium_risk_rules::rules;

    #[test]
    fn the_tier_is_medium() {
        assert_eq!(rules().level, RiskLevel::Medium);
    }

    #[test]
    fn sudo_is_flagged() {
        assert!(rules().matches("sudo systemctl restart nginx"));
    }

    #[test]
    fn a_forced_kill_is_flagged() {
        assert!(rules().matches("kill -9 1234"));
        assert!(rules().matches("killall -9 node"));
    }

    #[test]
    fn a_graceful_stop_is_not_flagged() {
        for command in ["kill 1234", "killall node", "systemctl stop nginx"] {
            assert!(!rules().matches(command), "{command:?} should be safe");
        }
    }

    #[test]
    fn flushing_firewall_rules_is_flagged() {
        assert!(rules().matches("sudo iptables -F"));
    }

    #[test]
    fn recursive_permission_changes_are_flagged() {
        assert!(rules().matches("chmod -R 755 ."));
        assert!(rules().matches("chown -R me:me /srv"));
    }

    #[test]
    fn pruning_all_docker_data_is_flagged() {
        assert!(rules().matches("docker system prune -a"));
    }
}
