//! The shape every tier of rules shares.

use regex::Regex;

use crate::safety::RiskLevel;

/// One pattern that marks a command as belonging to a risk tier.
#[derive(Debug, Clone)]
pub struct Rule {
    /// The pattern to look for, case-insensitively.
    pub pattern: &'static str,
    /// What to tell the user when it matches.
    pub warning: &'static str,
    compiled: Regex,
}

impl Rule {
    /// A rule from a pattern and the warning it produces.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::Rule;
    ///
    /// let rule = Rule::new(r"\brm\s+-rf\s+/", "wipes the filesystem");
    /// assert!(rule.matches("rm -rf /"));
    /// ```
    #[must_use]
    pub fn new(pattern: &'static str, warning: &'static str) -> Self {
        Self {
            pattern,
            warning,
            compiled: Regex::new(pattern)
                .unwrap_or_else(|error| unreachable!("built-in safety pattern: {error}")),
        }
    }

    /// Whether the command trips this rule.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::Rule;
    ///
    /// assert!(Rule::new(r"\brm\s+-rf", "delete").matches("rm -rf x"));
    /// assert!(!Rule::new(r"\brm\s+-rf", "delete").matches("ls"));
    /// ```
    #[must_use]
    pub fn matches(&self, command: &str) -> bool {
        self.compiled.is_match(command)
    }

    /// The warning this rule produces.
    #[must_use]
    pub const fn warning(&self) -> &'static str {
        self.warning
    }
}

/// All the rules for one risk level.
///
/// # Examples
///
/// ```
/// use shx::safety::{RiskLevel, RuleSet};
///
/// let set = RuleSet::new(RiskLevel::Low, [(r"pip uninstall", "removes a package")]);
/// assert!(set.matches("pip uninstall requests"));
/// assert!(!set.matches("pip install requests"));
/// ```
#[derive(Debug)]
pub struct RuleSet {
    /// The level these rules belong to.
    pub level: RiskLevel,
    /// The rules themselves.
    pub rules: Vec<Rule>,
}

impl RuleSet {
    /// Build a set from `(pattern, warning)` pairs.
    ///
    /// Every pattern is compiled here rather than on first use, so a pattern
    /// that does not compile is a test failure rather than a surprise on the
    /// user's first command.
    #[must_use]
    pub fn new<const N: usize>(
        level: RiskLevel,
        patterns: [(&'static str, &'static str); N],
    ) -> Self {
        Self {
            level,
            rules: patterns
                .iter()
                .map(|(pattern, warning)| Rule::new(pattern, warning))
                .collect(),
        }
    }

    /// Every warning produced by the rules this command trips.
    ///
    /// All of them, not just the first: a command that is both a forced kill and
    /// a recursive chmod deserves to hear about both.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::{RiskLevel, RuleSet};
    ///
    /// let set = RuleSet::new(RiskLevel::High, [(r"sudo", "needs root"), (r"rm -rf", "delete")]);
    /// assert_eq!(set.warnings_for("sudo rm -rf x").len(), 2);
    /// assert!(set.warnings_for("ls").is_empty());
    /// ```
    #[must_use]
    pub fn warnings_for(&self, command: &str) -> Vec<&'static str> {
        self.rules
            .iter()
            .filter(|rule| rule.matches(command))
            .map(Rule::warning)
            .collect()
    }

    /// Whether any rule in this set trips.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::{RiskLevel, RuleSet};
    ///
    /// let set = RuleSet::new(RiskLevel::Critical, [(r"rm\s+-rf\s+/", "wipes disk")]);
    /// assert!(set.matches("rm -rf /"));
    /// assert!(!set.matches("rm -rf ./build"));
    /// ```
    #[must_use]
    pub fn matches(&self, command: &str) -> bool {
        self.rules.iter().any(|rule| rule.matches(command))
    }
}
