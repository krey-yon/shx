//! How dangerous a command looks.

/// How much damage a command could do.
///
/// Ordered from safe to critical. The order is meaningful:
/// [`RiskLevel::at_least`](RiskLevel::at_least) compares with it, so adding a
/// variant in the middle is a breaking change to the ordering.
///
/// # Examples
///
/// ```
/// use shx::safety::RiskLevel;
///
/// assert!(RiskLevel::Critical.at_least(RiskLevel::High));
/// assert!(!RiskLevel::Low.at_least(RiskLevel::Medium));
/// assert_eq!(RiskLevel::Medium.as_str(), "medium");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RiskLevel {
    /// Nothing here can lose data. Runs without asking.
    Safe,
    /// A package removal or a local edit. Asks once.
    Low,
    /// Changes to the system, or a force kill. Asks with a typed word.
    Medium,
    /// Recursive deletion, or anything that could lose a lot. Asks with a
    /// capitalised word.
    High,
    /// Irreversible damage to the machine. Refused outright.
    Critical,
}

impl RiskLevel {
    /// The name used in output and in the config file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Whether this level is at least as severe as `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::RiskLevel;
    ///
    /// assert!(RiskLevel::High.at_least(RiskLevel::Medium));
    /// assert!(RiskLevel::Medium.at_least(RiskLevel::Medium));
    /// assert!(!RiskLevel::Safe.at_least(RiskLevel::Low));
    /// ```
    #[must_use]
    pub const fn at_least(self, other: Self) -> bool {
        (self as u8) >= (other as u8)
    }

    /// Whether the command is refused regardless of what the user types.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::safety::RiskLevel;
    ///
    /// assert!(RiskLevel::Critical.is_refused());
    /// assert!(!RiskLevel::High.is_refused());
    /// ```
    #[must_use]
    pub const fn is_refused(self) -> bool {
        matches!(self, Self::Critical)
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::RiskLevel;

    #[test]
    fn severity_orders_correctly() {
        assert!(RiskLevel::Safe < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn at_least_is_inclusive() {
        assert!(RiskLevel::Medium.at_least(RiskLevel::Medium));
        assert!(!RiskLevel::Low.at_least(RiskLevel::Medium));
    }

    #[test]
    fn only_critical_is_refused() {
        assert!(RiskLevel::Critical.is_refused());
        for level in [
            RiskLevel::Safe,
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
        ] {
            assert!(!level.is_refused(), "{level} should be confirmable");
        }
    }

    #[test]
    fn names_match_the_serialised_forms() {
        assert_eq!(RiskLevel::Safe.as_str(), "safe");
        assert_eq!(RiskLevel::Critical.to_string(), "critical");
    }
}
