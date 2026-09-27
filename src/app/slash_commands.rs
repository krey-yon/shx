//! The built-in commands, which never reach a model.
//!
//! Parsing lives here and behaviour lives in
//! [`ShxApp`](crate::app::shx_app::ShxApp), so a new command is one variant and
//! one match arm rather than a new parsing rule as well. An unrecognised name
//! keeps its own variant instead of being dropped, because silently ignoring
//! `/hel` would look to the user like the app had crashed.

/// A built-in command, parsed but not yet acted on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    /// `/help` — list the commands.
    Help,
    /// `/clear` — clear the screen.
    Clear,
    /// `/model [name]` — show or set the model.
    Model(Option<String>),
    /// `/provider [name]` — show or set the provider.
    Provider(Option<String>),
    /// `/history` — what has run this session.
    History,
    /// `/forget [count]` — drop the most recent commands from the history.
    Forget(Option<usize>),
    /// `/exit` — end the session.
    Exit,
    /// A name we do not know, kept so the app can say so.
    Unknown {
        /// The name as typed, without the leading slash.
        name: String,
        /// Whatever followed it.
        argument: Option<String>,
    },
}

impl SlashCommand {
    /// Parse a line, or `None` if it is not slash-prefixed.
    ///
    /// A bare `/` parses as [`SlashCommand::Unknown`] with an empty name, which
    /// the app reports, because falling through to the shell classifier would
    /// treat it as a filesystem path.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::slash_commands::SlashCommand;
    ///
    /// assert_eq!(SlashCommand::parse("/help"), Some(SlashCommand::Help));
    /// assert_eq!(
    ///     SlashCommand::parse("/model claude-sonnet-4-5"),
    ///     Some(SlashCommand::Model(Some("claude-sonnet-4-5".to_owned()))),
    /// );
    /// assert_eq!(SlashCommand::parse("ls"), None);
    /// ```
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let rest = line.trim().strip_prefix('/')?;
        let (name, argument) = match rest.split_once(char::is_whitespace) {
            Some((name, argument)) => (name, Some(argument.trim())),
            None => (rest, None),
        };

        let argument = argument
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        Some(match name.to_ascii_lowercase().as_str() {
            "help" | "?" => Self::Help,
            "clear" => Self::Clear,
            "model" => Self::Model(argument),
            "provider" => Self::Provider(argument),
            "history" => Self::History,
            "forget" => Self::Forget(argument.and_then(|value| value.parse().ok())),
            "exit" | "quit" => Self::Exit,
            _ => Self::Unknown {
                name: name.to_owned(),
                argument,
            },
        })
    }

    /// The name as the user typed it, for echoing back.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::slash_commands::SlashCommand;
    ///
    /// assert_eq!(SlashCommand::Help.name(), "help");
    /// assert_eq!(SlashCommand::Exit.name(), "quit");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Help => "help",
            Self::Clear => "clear",
            Self::Model(_) => "model",
            Self::Provider(_) => "provider",
            Self::History => "history",
            Self::Forget(_) => "forget",
            Self::Exit => "exit",
            Self::Unknown { name, .. } => name,
        }
    }
}

/// The text `/help` prints.
#[must_use]
pub fn help_text() -> String {
    let commands = [
        "/help              this list",
        "/clear             clear the screen",
        "/model [name]      show or set the model",
        "/provider [name]   show or set the provider",
        "/history           commands run this session",
        "/forget [count]    drop the last commands from the history",
        "/exit              end the session",
        "",
        "Anything else is run as a shell command, or asked of the model.",
    ];

    let mut text = String::from("shx commands:\n");
    for line in commands {
        text.push_str("  ");
        text.push_str(line);
        text.push('\n');
    }
    text.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::{SlashCommand, help_text};

    #[test]
    fn each_command_without_an_argument_parses() {
        assert_eq!(SlashCommand::parse("/help"), Some(SlashCommand::Help));
        assert_eq!(SlashCommand::parse("/clear"), Some(SlashCommand::Clear));
        assert_eq!(SlashCommand::parse("/history"), Some(SlashCommand::History));
        assert_eq!(SlashCommand::parse("/exit"), Some(SlashCommand::Exit));
        assert_eq!(
            SlashCommand::parse("/model"),
            Some(SlashCommand::Model(None))
        );
        assert_eq!(
            SlashCommand::parse("/provider"),
            Some(SlashCommand::Provider(None))
        );
        assert_eq!(
            SlashCommand::parse("/forget"),
            Some(SlashCommand::Forget(None))
        );
    }

    #[test]
    fn an_argument_is_kept_with_the_command() {
        assert_eq!(
            SlashCommand::parse("/model claude-sonnet-4-5"),
            Some(SlashCommand::Model(Some("claude-sonnet-4-5".to_owned())))
        );
        assert_eq!(
            SlashCommand::parse("/provider Anthropic"),
            Some(SlashCommand::Provider(Some("Anthropic".to_owned())))
        );
        assert_eq!(
            SlashCommand::parse("/forget 3"),
            Some(SlashCommand::Forget(Some(3)))
        );
    }

    #[test]
    fn a_forget_argument_that_is_not_a_number_forgets_nothing_specific() {
        assert_eq!(
            SlashCommand::parse("/forget all"),
            Some(SlashCommand::Forget(None))
        );
    }

    #[test]
    fn names_are_case_insensitive_and_the_argument_is_not() {
        assert_eq!(SlashCommand::parse("/HELP"), Some(SlashCommand::Help));
        assert_eq!(
            SlashCommand::parse("/MoDeL X"),
            Some(SlashCommand::Model(Some("X".to_owned())))
        );
    }

    #[test]
    fn an_unknown_name_is_kept_rather_than_dropped() {
        assert_eq!(
            SlashCommand::parse("/hel"),
            Some(SlashCommand::Unknown {
                name: "hel".to_owned(),
                argument: None,
            })
        );
        assert_eq!(
            SlashCommand::parse("/deploy staging now"),
            Some(SlashCommand::Unknown {
                name: "deploy".to_owned(),
                argument: Some("staging now".to_owned()),
            })
        );
    }

    #[test]
    fn a_bare_slash_is_unknown_rather_than_a_path() {
        assert_eq!(
            SlashCommand::parse("/"),
            Some(SlashCommand::Unknown {
                name: String::new(),
                argument: None,
            })
        );
    }

    #[test]
    fn a_line_that_is_not_slash_prefixed_is_not_a_slash_command() {
        for line in ["ls", "ls /tmp", "what is /tmp", "", "   "] {
            assert_eq!(SlashCommand::parse(line), None, "{line:?}");
        }
    }

    #[test]
    fn trailing_whitespace_does_not_become_an_empty_argument() {
        assert_eq!(
            SlashCommand::parse("/model   "),
            Some(SlashCommand::Model(None))
        );
        assert_eq!(SlashCommand::parse("  /help  "), Some(SlashCommand::Help));
    }

    #[test]
    fn an_inner_slash_is_part_of_the_name() {
        assert_eq!(
            SlashCommand::parse("/model/gpt"),
            Some(SlashCommand::Unknown {
                name: "model/gpt".to_owned(),
                argument: None,
            })
        );
    }

    #[test]
    fn aliases_parse_to_their_canonical_command() {
        assert_eq!(SlashCommand::parse("/?"), Some(SlashCommand::Help));
        assert_eq!(SlashCommand::parse("/quit"), Some(SlashCommand::Exit));
    }

    #[test]
    fn the_name_is_what_the_user_typed_for_an_unknown_command() {
        let Some(SlashCommand::Unknown { name, .. }) = SlashCommand::parse("/Deploy") else {
            panic!("expected unknown");
        };
        assert_eq!(name, "Deploy", "the name is not normalised");
    }

    #[test]
    fn the_name_of_a_known_command_is_its_canonical_spelling() {
        assert_eq!(
            SlashCommand::parse("/quit").map(|command| command.name().to_owned()),
            Some("exit".to_owned())
        );
    }

    #[test]
    fn help_lists_every_command_the_parser_understands() {
        let help = help_text();
        for line in [
            "/help",
            "/clear",
            "/model",
            "/provider",
            "/history",
            "/forget",
            "/exit",
        ] {
            assert!(help.contains(line), "help should mention {line}");
        }
    }

    #[test]
    fn help_does_not_end_in_a_newline() {
        assert!(!help_text().ends_with('\n'));
    }
}
