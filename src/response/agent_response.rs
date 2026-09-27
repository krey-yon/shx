//! The union the REPL switches on.

use std::fmt;

use thiserror::Error;

use super::command_response::CommandResponse;
use super::general_response::GeneralResponse;

/// Whatever an agent came back with.
///
/// Two cases because there are two things the REPL can do next: run something
/// the user approves, or print an answer. Turning the provider's bytes into one
/// of these is the parser's job, in [`crate::llm::response_parser`].
///
/// # Examples
///
/// ```
/// use shx::response::agent_response::AgentResponse;
/// use shx::response::command_response::CommandResponse;
///
/// let response = AgentResponse::Command(CommandResponse::new("cargo test", "run the suite"));
/// assert_eq!(response.to_string(), "cargo test  run the suite");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentResponse {
    /// A command to offer the user.
    Command(CommandResponse),
    /// An answer to read, possibly offering a command of its own.
    General(GeneralResponse),
}

impl fmt::Display for AgentResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(command) => write!(formatter, "{command}"),
            Self::General(general) => write!(formatter, "{general}"),
        }
    }
}

/// Why a provider's answer could not be turned into an [`AgentResponse`].
///
/// The two cases are worth separating because the REPL treats them differently:
/// an empty body is a provider hiccup worth retrying, and a body we cannot read
/// is a reason to show the user what came back instead.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResponseParseError {
    /// The provider returned nothing at all.
    #[error("the model returned an empty response")]
    Empty,

    /// The body was not the JSON we asked for, or a field of it was wrong.
    #[error("could not read the model's response: {0}")]
    Malformed(String),
}

#[cfg(test)]
mod tests {
    use super::AgentResponse;
    use super::ResponseParseError;
    use crate::response::command_response::CommandResponse;
    use crate::response::general_response::GeneralResponse;
    use crate::response::response_type::ResponseType;

    fn command() -> AgentResponse {
        AgentResponse::Command(CommandResponse::new("git status --short", "what changed"))
    }

    fn general() -> AgentResponse {
        AgentResponse::General(
            GeneralResponse::new("The build is green.", ResponseType::GeneralQuery)
                .with_suggested_command("cargo build"),
        )
    }

    fn assert_std_error<E: std::error::Error>() {}

    #[test]
    fn each_variant_carries_its_response() {
        assert_eq!(
            command(),
            AgentResponse::Command(CommandResponse::new("git status --short", "what changed"))
        );
        let AgentResponse::General(general) = general() else {
            panic!("expected a general response");
        };
        assert_eq!(general.response_type, ResponseType::GeneralQuery);
        assert!(general.can_execute());
    }

    #[test]
    fn the_command_variant_is_what_the_runner_looks_for() {
        let AgentResponse::Command(inner) = command() else {
            panic!("expected a command response");
        };
        assert!(!inner.is_empty());
    }

    #[test]
    fn an_empty_command_response_is_still_representable_so_the_caller_can_reject_it() {
        let response = AgentResponse::Command(CommandResponse::default());
        let AgentResponse::Command(inner) = response else {
            panic!("expected a command response");
        };
        assert!(inner.is_empty());
    }

    #[test]
    fn display_shows_the_command_and_its_reason() {
        assert_eq!(command().to_string(), "git status --short  what changed");
    }

    #[test]
    fn display_shows_the_prose_of_a_general_response() {
        assert_eq!(general().to_string(), "The build is green.");
    }

    #[test]
    fn display_of_an_empty_command_response_is_empty() {
        assert_eq!(
            AgentResponse::Command(CommandResponse::default()).to_string(),
            ""
        );
    }

    #[test]
    fn the_empty_parse_error_says_what_happened() {
        assert_eq!(
            ResponseParseError::Empty.to_string(),
            "the model returned an empty response"
        );
    }

    #[test]
    fn a_malformed_response_keeps_the_reason_it_failed() {
        let error = ResponseParseError::Malformed("missing field `command`".to_owned());
        assert!(error.to_string().contains("missing field `command`"));
    }

    #[test]
    fn the_two_parse_errors_are_distinguishable() {
        assert_ne!(
            ResponseParseError::Empty,
            ResponseParseError::Malformed(String::new())
        );
    }

    #[test]
    fn a_parse_error_is_a_std_error() {
        assert_std_error::<ResponseParseError>();
        let error = ResponseParseError::Malformed("trailing prose".to_owned());
        assert!(std::error::Error::source(&error).is_none());
    }
}
