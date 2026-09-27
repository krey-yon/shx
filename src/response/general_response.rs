//! An answer that is not a command: prose, markdown, or code.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::response_type::ResponseType;

/// What an agent says when it is not handing over a command to run.
///
/// `content` and `response_type` are required, because both change how the
/// answer is rendered and guessing either would mislabel it. The other two
/// default: a model that omits them is describing an answer to read, not one to
/// act on.
///
/// # Examples
///
/// ```
/// use shx::response::general_response::GeneralResponse;
/// use shx::response::response_type::ResponseType;
///
/// let answer = GeneralResponse::new("A `let` binding is immutable.", ResponseType::GeneralQuery);
/// assert_eq!(answer.markdown_content(), "A `let` binding is immutable.");
/// assert!(!answer.can_execute());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralResponse {
    /// The answer itself, as markdown or plain text.
    pub content: String,

    /// What kind of answer this is, which decides how it is rendered.
    pub response_type: ResponseType,

    /// Whether the user should be offered a follow-up command.
    #[serde(default)]
    pub action_required: bool,

    /// The command to offer. Only meaningful alongside `action_required`.
    #[serde(default)]
    pub suggested_command: Option<String>,
}

impl GeneralResponse {
    /// An answer to read, with nothing to run.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::general_response::GeneralResponse;
    /// use shx::response::response_type::ResponseType;
    ///
    /// let answer = GeneralResponse::new("```rust\nfn main() {}\n```", ResponseType::CodeGeneration);
    /// assert_eq!(answer.response_type, ResponseType::CodeGeneration);
    /// assert_eq!(answer.suggested_command, None);
    /// ```
    #[must_use]
    pub fn new(content: impl Into<String>, response_type: ResponseType) -> Self {
        Self {
            content: content.into(),
            response_type,
            action_required: false,
            suggested_command: None,
        }
    }

    /// The same answer, offering a command to run.
    ///
    /// Sets [`action_required`](GeneralResponse::action_required) too, because
    /// a suggested command the user is never asked about is the one case where
    /// the two fields can disagree without the model being the reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::general_response::GeneralResponse;
    /// use shx::response::response_type::ResponseType;
    ///
    /// let answer =
    ///     GeneralResponse::new("That needs a rebuild.", ResponseType::GeneralQuery)
    ///         .with_suggested_command("cargo build");
    /// assert!(answer.action_required);
    /// assert!(answer.can_execute());
    /// ```
    #[must_use]
    pub fn with_suggested_command(mut self, command: impl Into<String>) -> Self {
        self.suggested_command = Some(command.into());
        self.action_required = true;
        self
    }

    /// Whether there is a real command to hand to the runner.
    ///
    /// All three conditions are needed: an answer that did not ask for action,
    /// one that suggested nothing, and one whose suggestion is blank are all
    /// reasons to print and stop.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::general_response::GeneralResponse;
    /// use shx::response::response_type::ResponseType;
    ///
    /// let answer = GeneralResponse::new("Rebuild it.", ResponseType::GeneralQuery)
    ///     .with_suggested_command("cargo build");
    /// assert!(answer.can_execute());
    /// ```
    #[must_use]
    pub fn can_execute(&self) -> bool {
        self.action_required
            && self
                .suggested_command
                .as_deref()
                .is_some_and(|command| !command.trim().is_empty())
    }

    /// The content as the renderer should see it.
    ///
    /// Borrowed rather than copied, and untrimmed: whether the answer is
    /// rendered as markdown is the renderer's call, not this type's.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::general_response::GeneralResponse;
    /// use shx::response::response_type::ResponseType;
    ///
    /// let answer = GeneralResponse::new("**done**", ResponseType::GeneralQuery);
    /// assert_eq!(answer.markdown_content(), "**done**");
    /// ```
    #[must_use]
    pub fn markdown_content(&self) -> &str {
        &self.content
    }
}

impl fmt::Display for GeneralResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::GeneralResponse;
    use super::ResponseType;

    fn with_suggestion(action_required: bool, suggested_command: Option<&str>) -> GeneralResponse {
        GeneralResponse {
            content: "Run the tests to check.".to_owned(),
            response_type: ResponseType::GeneralQuery,
            action_required,
            suggested_command: suggested_command.map(str::to_owned),
        }
    }

    #[test]
    fn new_produces_an_answer_with_nothing_to_run() {
        let answer = GeneralResponse::new("Port 8080 is free.", ResponseType::GeneralQuery);
        assert_eq!(answer.content, "Port 8080 is free.");
        assert_eq!(answer.response_type, ResponseType::GeneralQuery);
        assert!(!answer.action_required);
        assert_eq!(answer.suggested_command, None);
        assert!(!answer.can_execute());
    }

    #[test]
    fn can_execute_is_the_conjunction_of_three_conditions() {
        assert!(!with_suggestion(false, None).can_execute());
        assert!(!with_suggestion(false, Some("cargo test")).can_execute());
        assert!(!with_suggestion(true, None).can_execute());
        assert!(with_suggestion(true, Some("cargo test")).can_execute());
    }

    #[test]
    fn a_blank_suggested_command_is_not_offered_to_the_runner() {
        assert!(!with_suggestion(true, Some("")).can_execute());
        assert!(!with_suggestion(true, Some("  \n\t")).can_execute());
    }

    #[test]
    fn a_padded_suggested_command_is_still_offered() {
        assert!(with_suggestion(true, Some("  cargo test  ")).can_execute());
    }

    #[test]
    fn the_builder_marks_the_answer_actionable() {
        let answer = GeneralResponse::new("That needs a rebuild.", ResponseType::GeneralQuery)
            .with_suggested_command("cargo build");
        assert_eq!(answer.suggested_command.as_deref(), Some("cargo build"));
        assert!(answer.action_required);
        assert!(answer.can_execute());
    }

    #[test]
    fn markdown_content_is_the_content_verbatim() {
        let content = "```rust\nfn main() {}\n```\n";
        let answer = GeneralResponse::new(content, ResponseType::CodeGeneration);
        assert_eq!(answer.markdown_content(), content);
    }

    #[test]
    fn markdown_content_of_an_empty_answer_is_empty() {
        let answer = GeneralResponse::new("", ResponseType::GeneralQuery);
        assert_eq!(answer.markdown_content(), "");
    }

    #[test]
    fn round_trips_through_json_with_a_suggestion() {
        let answer = GeneralResponse::new("Try this first.", ResponseType::GeneralQuery)
            .with_suggested_command("cargo test");
        let json = serde_json::to_string(&answer).expect("serialise");
        let parsed: GeneralResponse = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(parsed, answer);
    }

    #[test]
    fn round_trips_through_json_without_a_suggestion() {
        let answer = GeneralResponse::new("Nothing to run.", ResponseType::CodeGeneration);
        let json = serde_json::to_string(&answer).expect("serialise");
        assert!(json.contains(r#""suggested_command":null"#));
        let parsed: GeneralResponse = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(parsed, answer);
    }

    #[test]
    fn deserialises_the_shape_the_code_prompt_asks_for() {
        let json = r#"{
            "content": "Here is the loop.",
            "response_type": "code_generation",
            "action_required": false,
            "suggested_command": null
        }"#;
        let answer: GeneralResponse = serde_json::from_str(json).expect("deserialise");
        assert_eq!(answer.response_type, ResponseType::CodeGeneration);
        assert_eq!(answer.content, "Here is the loop.");
        assert!(!answer.can_execute());
    }

    #[test]
    fn deserialises_a_model_that_suggests_a_command() {
        let json = r#"{
            "content": "Install it first.",
            "response_type": "general_query",
            "action_required": true,
            "suggested_command": "sudo apt install docker"
        }"#;
        let answer: GeneralResponse = serde_json::from_str(json).expect("deserialise");
        assert!(answer.can_execute());
        assert_eq!(
            answer.suggested_command.as_deref(),
            Some("sudo apt install docker")
        );
    }

    #[test]
    fn a_model_that_omits_the_optional_fields_still_parses() {
        let json = r#"{"content":"hi","response_type":"general_query"}"#;
        let answer: GeneralResponse = serde_json::from_str(json).expect("deserialise");
        assert!(!answer.action_required);
        assert_eq!(answer.suggested_command, None);
    }

    #[test]
    fn a_missing_content_or_type_is_rejected() {
        assert!(
            serde_json::from_str::<GeneralResponse>(r#"{"response_type":"general_query"}"#)
                .is_err()
        );
        assert!(serde_json::from_str::<GeneralResponse>(r#"{"content":"hi"}"#).is_err());
    }

    #[test]
    fn an_unknown_response_type_is_rejected() {
        let json = r#"{"content":"hi","response_type":"poetry"}"#;
        assert!(serde_json::from_str::<GeneralResponse>(json).is_err());
    }

    #[test]
    fn display_shows_the_content() {
        let answer = GeneralResponse::new("**bold** answer", ResponseType::GeneralQuery);
        assert_eq!(answer.to_string(), "**bold** answer");
    }
}
