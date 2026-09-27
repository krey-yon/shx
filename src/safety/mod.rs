//! Risk classification for commands that could destroy data.
//!
//! The tiers are separate modules so that adding a rule is a one-line change
//! in the right file, and so a reviewer can see the whole of "what we refuse"
//! without scrolling past everything we merely warn about.
//!
//! - [`rule_set`] — the shape every tier shares.
//! - [`safety_analyzer`] — matches a command against every tier in order.
//! - [`confirmation_policy`] — what the user has to type for each level.

pub mod confirmation_policy;
pub mod critical_rules;
pub mod high_risk_rules;
pub mod low_risk_rules;
pub mod medium_risk_rules;
pub mod risk_level;
pub mod rule_set;
pub mod safety_analyzer;
pub mod safety_result;

pub use risk_level::RiskLevel;
pub use rule_set::{Rule, RuleSet};
pub use safety_analyzer::analyze_command;
pub use safety_result::SafetyResult;
