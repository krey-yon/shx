//! The verdict of the safety analysis.

use crate::safety::risk_level::RiskLevel;

/// What the analyser found, and what to tell the user about it.
///
/// # Examples
///
/// ```
/// use shx::safety::{RiskLevel, SafetyResult};
///
/// let result = SafetyResult {
///     risk_level: RiskLevel::High,
///     warnings: vec!["recursive delete".to_owned()],
///     blocked: true,
///     suggestions: vec!["move it to a backup first".to_owned()],
/// };
/// assert!(result.blocked);
/// assert_eq!(result.warnings.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyResult {
    /// How severe this command is.
    pub risk_level: RiskLevel,
    /// One line per thing that was found, in the order the rules are checked.
    pub warnings: Vec<String>,
    /// Whether the command may not be run at all, whatever the user types.
    pub blocked: bool,
    /// Alternatives worth offering, mostly for deletions.
    pub suggestions: Vec<String>,
}

impl SafetyResult {
    /// A verdict for a command with nothing wrong with it.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::{RiskLevel, SafetyResult};
    ///
    /// let result = SafetyResult::safe();
    /// assert_eq!(result.risk_level, RiskLevel::Safe);
    /// assert!(!result.blocked);
    /// ```
    #[must_use]
    pub const fn safe() -> Self {
        Self {
            risk_level: RiskLevel::Safe,
            warnings: Vec::new(),
            blocked: false,
            suggestions: Vec::new(),
        }
    }

    /// Whether the user needs to be asked at all.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::{RiskLevel, SafetyResult};
    ///
    /// let result = SafetyResult::safe();
    /// assert!(!result.needs_confirmation());
    ///
    /// let risky = SafetyResult { risk_level: RiskLevel::Medium, ..SafetyResult::safe() };
    /// assert!(risky.needs_confirmation());
    /// ```
    #[must_use]
    pub const fn needs_confirmation(&self) -> bool {
        !matches!(self.risk_level, RiskLevel::Safe)
    }

    /// Everything to show above the confirmation prompt, as one block.
    ///
    /// Empty for a safe command, so a caller can print it unconditionally.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::{RiskLevel, SafetyResult};
    ///
    /// let result = SafetyResult {
    ///     risk_level: RiskLevel::High,
    ///     warnings: vec!["recursive delete".to_owned()],
    ///     blocked: true,
    ///     suggestions: vec!["use trash instead".to_owned()],
    /// };
    ///
    /// let rendered = result.warning_block();
    /// assert!(rendered.contains("recursive delete"));
    /// assert!(rendered.contains("use trash instead"));
    /// ```
    #[must_use]
    pub fn warning_block(&self) -> String {
        let mut block = String::new();

        for warning in &self.warnings {
            block.push_str(warning);
            block.push('\n');
        }

        if !self.suggestions.is_empty() {
            block.push_str("Suggestions:\n");
            for suggestion in &self.suggestions {
                block.push_str("  - ");
                block.push_str(suggestion);
                block.push('\n');
            }
        }

        block.trim_end().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::safety::{RiskLevel, SafetyResult};

    #[test]
    fn a_safe_result_is_empty_and_unblocked() {
        let result = SafetyResult::safe();
        assert_eq!(result.risk_level, RiskLevel::Safe);
        assert!(!result.blocked);
        assert!(result.warnings.is_empty());
        assert!(result.suggestions.is_empty());
        assert!(!result.needs_confirmation());
    }

    #[test]
    fn anything_but_safe_needs_confirmation() {
        for level in [
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ] {
            let result = SafetyResult {
                risk_level: level,
                ..SafetyResult::safe()
            };
            assert!(result.needs_confirmation(), "{level} should ask");
        }
    }

    #[test]
    fn the_warning_block_is_empty_for_a_safe_command() {
        assert!(SafetyResult::safe().warning_block().is_empty());
    }

    #[test]
    fn the_block_lists_warnings_then_suggestions() {
        let result = SafetyResult {
            risk_level: RiskLevel::High,
            warnings: vec!["first".to_owned(), "second".to_owned()],
            blocked: true,
            suggestions: vec!["try this".to_owned()],
        };

        let block = result.warning_block();
        let first = block.find("first").expect("first warning");
        let second = block.find("second").expect("second warning");
        let suggestion = block.find("try this").expect("suggestion");

        assert!(first < second, "warnings should keep their order");
        assert!(second < suggestion, "suggestions come after warnings");
    }

    #[test]
    fn a_block_with_warnings_but_no_suggestions_has_no_suggestion_header() {
        let result = SafetyResult {
            risk_level: RiskLevel::Low,
            warnings: vec!["only a warning".to_owned()],
            ..SafetyResult::safe()
        };
        assert!(!result.warning_block().contains("Suggestions"));
    }

    #[test]
    fn the_block_is_trimmed_rather_than_ending_in_a_newline() {
        let result = SafetyResult {
            risk_level: RiskLevel::Low,
            warnings: vec!["w".to_owned()],
            ..SafetyResult::safe()
        };
        assert_eq!(result.warning_block(), "w");
    }
}
