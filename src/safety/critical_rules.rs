//! Commands that are refused outright, whatever the user types.
//!
//! Nothing here is confirmable. Each of these either destroys data that cannot
//! be recovered or takes the machine down, and there is no typing sequence that
//! distinguishes intent from a slip of the finger.

use std::sync::LazyLock;

use crate::safety::{RiskLevel, RuleSet};

static RULES: LazyLock<RuleSet> = LazyLock::new(|| {
    RuleSet::new(
        RiskLevel::Critical,
        [
            (
                r"\brm\s+-[rf]{1,2}\s+/(?:\s|$)",
                "Refusing: this deletes a filesystem root.",
            ),
            (
                r"\bmkfs(?:\.\w+)?\s",
                "Refusing: this creates a filesystem.",
            ),
            (
                r"\bdd\b[^\n]*\bof=/dev/",
                "Refusing: this writes directly to a device.",
            ),
            (
                r":\(\)\s*\{\s*:\|:&\s*\}\s*;?\s*:",
                "Refusing: this is a fork bomb.",
            ),
            (
                r"\b(?:shutdown|reboot|halt|poweroff)\b",
                "Refusing: this powers off the machine.",
            ),
            (
                r">\s*/dev/(?:sd[a-z]|nvme\d|disk\d|hd[a-z])",
                "Refusing: this overwrites a disk.",
            ),
        ],
    )
});

/// The critical rules.
///
/// # Examples
///
/// ```
/// use shx::safety::critical_rules::rules;
///
/// assert!(rules().matches("rm -rf /"));
/// assert!(!rules().matches("rm -rf ./build"));
/// ```
#[must_use]
pub fn rules() -> &'static RuleSet {
    &RULES
}

#[cfg(test)]
mod tests {
    use crate::safety::RiskLevel;
    use crate::safety::critical_rules::rules;

    #[test]
    fn destroying_a_root_is_refused() {
        for command in [
            "rm -rf /",
            "rm -fr /",
            "sudo rm -rf /",
            "rm -rf / --no-preserve-root",
        ] {
            assert!(rules().matches(command), "{command:?} should be refused");
        }
    }

    #[test]
    fn a_root_that_is_only_a_prefix_is_not_refused() {
        for command in ["rm -rf /home", "rm -rf ./build", "rm -rf /tmp/x"] {
            assert!(
                !rules().matches(command),
                "{command:?} should not be refused"
            );
        }
    }

    #[test]
    fn filesystem_creation_is_refused() {
        assert!(rules().matches("mkfs.ext4 /dev/sda1"));
        assert!(rules().matches("sudo mkfs -t ext4 /dev/sdb"));
    }

    #[test]
    fn writing_to_a_device_is_refused() {
        assert!(rules().matches("dd if=/dev/zero of=/dev/sda bs=1M"));
    }

    #[test]
    fn a_fork_bomb_is_refused() {
        assert!(rules().matches(":(){ :|:& };:"));
    }

    #[test]
    fn powering_off_is_refused() {
        for command in ["shutdown -h now", "reboot", "sudo poweroff"] {
            assert!(rules().matches(command), "{command:?} should be refused");
        }
    }

    #[test]
    fn overwriting_a_disk_is_refused() {
        assert!(rules().matches("cat image.iso > /dev/sda"));
    }

    #[test]
    fn the_tier_is_critical() {
        assert_eq!(rules().level, RiskLevel::Critical);
    }

    #[test]
    fn every_hit_produces_a_warning() {
        assert_eq!(rules().warnings_for("rm -rf /").len(), 1);
        assert!(rules().warnings_for("ls").is_empty());
    }
}
