//! The kind of answer an agent produced.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The kind of answer an agent produced.
///
/// The strings in [`as_str`](ResponseType::as_str) are a contract with the
/// model: every prompt asks for exactly one of them, and a response naming
/// anything else is not one of ours.
///
/// # Examples
///
/// ```
/// use shx::response::response_type::ResponseType;
///
/// assert_eq!(ResponseType::Command.as_str(), "shell_command");
/// assert_eq!(ResponseType::CodeGeneration.to_string(), "code_generation");
/// assert_eq!(
///     ResponseType::parse("GENERAL_QUERY"),
///     Some(ResponseType::GeneralQuery)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    /// A shell command the user may choose to run.
    #[serde(rename = "shell_command")]
    Command,
    /// An answer to a question, in prose.
    GeneralQuery,
    /// Code, or a technical explanation of it.
    CodeGeneration,
}

impl ResponseType {
    /// The wire form, spelled the way the prompt asks for it.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::response_type::ResponseType;
    ///
    /// assert_eq!(ResponseType::Command.as_str(), "shell_command");
    /// ```
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Command => "shell_command",
            Self::GeneralQuery => "general_query",
            Self::CodeGeneration => "code_generation",
        }
    }

    /// Read a wire form back, ignoring case and surrounding whitespace.
    ///
    /// `None` for anything that is not one of the three, leaving the caller to
    /// decide between asking again and falling back to a plain-text answer.
    /// Models are inconsistent about capitalising these, and a model that
    /// capitalises them still meant one of ours.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::response::response_type::ResponseType;
    ///
    /// assert_eq!(
    ///     ResponseType::parse(" code_generation "),
    ///     Some(ResponseType::CodeGeneration)
    /// );
    /// assert_eq!(ResponseType::parse("shell-command"), None);
    /// ```
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let candidate = text.trim();
        [Self::Command, Self::GeneralQuery, Self::CodeGeneration]
            .into_iter()
            .find(|kind| kind.as_str().eq_ignore_ascii_case(candidate))
    }
}

impl fmt::Display for ResponseType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::ResponseType;

    const ALL: [ResponseType; 3] = [
        ResponseType::Command,
        ResponseType::GeneralQuery,
        ResponseType::CodeGeneration,
    ];

    #[test]
    fn wire_strings_are_the_ones_the_prompts_ask_for() {
        assert_eq!(ResponseType::Command.as_str(), "shell_command");
        assert_eq!(ResponseType::GeneralQuery.as_str(), "general_query");
        assert_eq!(ResponseType::CodeGeneration.as_str(), "code_generation");
    }

    #[test]
    fn display_matches_as_str() {
        for kind in ALL {
            assert_eq!(kind.to_string(), kind.as_str());
        }
    }

    #[test]
    fn parse_reads_every_wire_string_back() {
        for kind in ALL {
            assert_eq!(ResponseType::parse(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn parse_ignores_case() {
        assert_eq!(
            ResponseType::parse("SHELL_COMMAND"),
            Some(ResponseType::Command)
        );
        assert_eq!(
            ResponseType::parse("General_Query"),
            Some(ResponseType::GeneralQuery)
        );
        assert_eq!(
            ResponseType::parse("CoDe_GeNeRaTiOn"),
            Some(ResponseType::CodeGeneration)
        );
    }

    #[test]
    fn parse_ignores_surrounding_whitespace() {
        assert_eq!(
            ResponseType::parse("\n  shell_command \t"),
            Some(ResponseType::Command)
        );
    }

    #[test]
    fn parse_rejects_anything_that_is_not_a_wire_string() {
        for rejected in ["", "   ", "shell-command", "shell command", "poetry"] {
            assert_eq!(ResponseType::parse(rejected), None, "{rejected:?} parsed");
        }
    }

    #[test]
    fn the_rust_variant_names_are_not_wire_strings() {
        assert_eq!(ResponseType::parse("Command"), None);
        assert_eq!(ResponseType::parse("GeneralQuery"), None);
    }

    #[test]
    fn serialising_produces_the_wire_string() {
        assert_eq!(
            serde_json::to_string(&ResponseType::Command).expect("serialise"),
            "\"shell_command\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseType::GeneralQuery).expect("serialise"),
            "\"general_query\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseType::CodeGeneration).expect("serialise"),
            "\"code_generation\""
        );
    }

    #[test]
    fn deserialising_reads_the_wire_string() {
        for kind in ALL {
            let json = format!("\"{}\"", kind.as_str());
            let parsed: ResponseType = serde_json::from_str(&json).expect("deserialise");
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn deserialising_an_unknown_string_fails() {
        assert!(serde_json::from_str::<ResponseType>("\"shell_command \"").is_err());
        assert!(serde_json::from_str::<ResponseType>("\"Command\"").is_err());
    }

    #[test]
    fn as_str_and_parse_are_inverses() {
        for kind in ALL {
            assert_eq!(
                ResponseType::parse(kind.as_str()).expect("round trip"),
                kind
            );
        }
    }
}
