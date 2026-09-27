//! A command an agent proposed, and why it proposed it.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A shell command the agent suggests, with the reason behind it.
///
/// Deserialising is lenient because the model is the one writing this: it may
/// add keys the prompt only mentions in passing, and it may drop the
/// `response_type` key it was shown alongside. A missing or blank
/// [`command`](CommandResponse::command) is caught by
/// [`is_empty`](CommandResponse::is_empty) rather than at the JSON layer, so
/// the caller decides whether to retry or to drop the turn.
///
/// # Examples
///
/// ```
/// use shx::response::command_response::CommandResponse;
///
/// let response = CommandResponse::new("cargo test", "run the suite");
/// assert!(!response.is_empty());
/// assert!(response.has_explanation());
/// assert_eq!(response.to_string(), "cargo test  run the suite");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandResponse {
    /// The command line, exactly as the user would type it.
    pub command: String,
    /// Why this is the right command for what the user asked.
    pub explanation: String,
}

impl CommandResponse {
    /// A command with its explanation.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::command_response::CommandResponse;
    ///
    /// let response = CommandResponse::new("ls -la", "list every file");
    /// assert_eq!(response.command, "ls -la");
    /// ```
    #[must_use]
    pub fn new(command: impl Into<String>, explanation: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            explanation: explanation.into(),
        }
    }

    /// Whether there is nothing here worth running.
    ///
    /// A command that is only whitespace is empty: the model returned a
    /// placeholder, and the safety checker would happily confirm a blank line
    /// and the runner would happily "succeed" at it.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::command_response::CommandResponse;
    ///
    /// assert!(CommandResponse::default().is_empty());
    /// assert!(CommandResponse::new("  \n", "could not work it out").is_empty());
    /// assert!(!CommandResponse::new("ls", "").is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.command.trim().is_empty()
    }

    /// Whether the model bothered to say why.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::command_response::CommandResponse;
    ///
    /// assert!(CommandResponse::new("ls", "list files").has_explanation());
    /// assert!(!CommandResponse::new("ls", "  ").has_explanation());
    /// ```
    #[must_use]
    pub fn has_explanation(&self) -> bool {
        !self.explanation.trim().is_empty()
    }
}

impl fmt::Display for CommandResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_explanation() {
            write!(formatter, "{}  {}", self.command, self.explanation.trim())
        } else {
            formatter.write_str(&self.command)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandResponse;

    #[test]
    fn new_keeps_both_fields_verbatim() {
        let response = CommandResponse::new("git status --short", "  what changed  ");
        assert_eq!(response.command, "git status --short");
        assert_eq!(response.explanation, "  what changed  ");
    }

    #[test]
    fn a_default_response_has_nothing_to_run() {
        let response = CommandResponse::default();
        assert!(response.is_empty());
        assert!(!response.has_explanation());
    }

    #[test]
    fn a_blank_command_is_empty_even_with_an_explanation() {
        assert!(CommandResponse::new("", "explains itself").is_empty());
        assert!(CommandResponse::new("   ", "explains itself").is_empty());
        assert!(CommandResponse::new("\t\n", "explains itself").is_empty());
    }

    #[test]
    fn a_command_with_padding_is_not_empty() {
        assert!(!CommandResponse::new("  ls  ", "").is_empty());
    }

    #[test]
    fn has_explanation_is_false_for_blank_prose() {
        assert!(!CommandResponse::new("ls", "").has_explanation());
        assert!(!CommandResponse::new("ls", "  \n ").has_explanation());
    }

    #[test]
    fn serialises_to_the_two_field_shape() {
        let json =
            serde_json::to_string(&CommandResponse::new("ls", "list files")).expect("serialise");
        assert_eq!(json, r#"{"command":"ls","explanation":"list files"}"#);
    }

    #[test]
    fn deserialises_the_shape_the_shell_prompt_asks_for() {
        let json = r#"{"command":"sudo apt install docker","explanation":"install docker",
            "response_type":"shell_command"}"#;
        let response: CommandResponse = serde_json::from_str(json).expect("deserialise");
        assert_eq!(response.command, "sudo apt install docker");
        assert_eq!(response.explanation, "install docker");
    }

    #[test]
    fn a_missing_explanation_deserialises_as_blank_rather_than_failing() {
        let response: CommandResponse =
            serde_json::from_str(r#"{"command":"ls"}"#).expect("deserialise");
        assert!(!response.is_empty());
        assert!(!response.has_explanation());
    }

    #[test]
    fn a_missing_command_deserialises_as_empty_rather_than_failing() {
        let response: CommandResponse =
            serde_json::from_str(r#"{"explanation":"oops"}"#).expect("deserialise");
        assert!(response.is_empty());
    }

    #[test]
    fn round_trips_through_json() {
        let response = CommandResponse::new("cargo build --release", "build optimised");
        let json = serde_json::to_string(&response).expect("serialise");
        let parsed: CommandResponse = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(parsed, response);
    }

    #[test]
    fn a_wrongly_typed_field_is_rejected() {
        assert!(serde_json::from_str::<CommandResponse>(r#"{"command":42}"#).is_err());
        assert!(serde_json::from_str::<CommandResponse>(r#"{"explanation":[]}"#).is_err());
    }

    #[test]
    fn a_positional_array_fills_the_fields_in_order() {
        let response: CommandResponse =
            serde_json::from_str(r#"["ls","list files"]"#).expect("deserialise");
        assert_eq!(response, CommandResponse::new("ls", "list files"));
    }

    #[test]
    fn an_array_with_nothing_in_it_is_empty_rather_than_a_parse_failure() {
        let response: CommandResponse = serde_json::from_str("[]").expect("deserialise");
        assert!(response.is_empty());
    }

    #[test]
    fn display_shows_the_command_and_its_reason() {
        assert_eq!(
            CommandResponse::new("cargo test", "run the suite").to_string(),
            "cargo test  run the suite"
        );
    }

    #[test]
    fn display_omits_a_blank_reason() {
        assert_eq!(
            CommandResponse::new("cargo test", "").to_string(),
            "cargo test"
        );
        assert_eq!(
            CommandResponse::new("cargo test", "  \n").to_string(),
            "cargo test"
        );
    }

    #[test]
    fn display_of_an_empty_response_is_empty() {
        assert_eq!(CommandResponse::default().to_string(), "");
    }
}
