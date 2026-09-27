//! Turning a line of input into a decision.
//!
//! Nothing here executes, prompts or prints, which is the whole point: the
//! routing table is the part of the REPL most likely to be wrong and the part
//! most expensive to test by hand. Given a line and a session it returns what
//! should happen, and [`ShxApp`](crate::app::shx_app::ShxApp) is the only thing
//! that acts on it.

use crate::app::session_state::SessionState;
use crate::app::slash_commands::SlashCommand;
use crate::shell::{
    InputKind, cd_target, classify_input, is_directory_change, is_screen_clear, is_session_exit,
};

/// What a line of input should cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Routed {
    /// Nothing to do: the line was empty or only whitespace.
    Empty,
    /// Clear the screen.
    ScreenClear,
    /// End the session.
    SessionExit,
    /// Move to this directory. An empty target means the home directory.
    ChangeDirectory(String),
    /// Run a built-in command.
    Slash(SlashCommand),
    /// Run this as a shell command.
    RunCommand(String),
    /// Ask a model this.
    AskModel(String),
}

impl Routed {
    /// Whether this decision ends the session.
    ///
    /// Covers `/exit` as well as the bare word, so a caller that checks this
    /// first does not have to know that exit has two spellings.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::input_router::Routed;
    /// use shx::app::slash_commands::SlashCommand;
    ///
    /// assert!(Routed::SessionExit.ends_session());
    /// assert!(Routed::Slash(SlashCommand::Exit).ends_session());
    /// assert!(!Routed::Empty.ends_session());
    /// ```
    #[must_use]
    pub const fn ends_session(&self) -> bool {
        matches!(self, Self::SessionExit | Self::Slash(SlashCommand::Exit))
    }
}

/// Decide what a line of input means.
///
/// The session is accepted so that this is the one function a caller can hand
/// any line to without first knowing whether the line is special. No rule
/// consults it: every decision below is a property of the line alone, and the
/// things a session would change — where `cd` goes, whether a provider is
/// configured, what the history holds — are answered by the app that owns it.
///
/// The order of the rules is the design, and each step is there because of the
/// one below it:
///
/// 1. **Empty first.** Whitespace is discarded rather than classified, or it
///    would be sent to a model as a question with no words in it.
/// 2. **Exit before everything else.** Ending the session is the only action
///    that cannot be taken back, so it is settled before any line that merely
///    starts with the same letters. `is_session_exit` matches the whole line,
///    so `exit code 0` still gets past this and reaches the model as the
///    question it is.
/// 3. **Clear before `cd`,** because `clear` is both a real command and a
///    control word, and a user typing it wants their screen back.
/// 4. **`cd` before the slash check and before the classifier.** `cd` mutates
///    the session rather than the filesystem, and routing it as a command would
///    run it in a child process that exits immediately, losing the change.
/// 5. **Slashes before the classifier.** `/help` would otherwise be read as an
///    absolute filesystem path, and `/clear` as an argument to nothing.
/// 6. **Classify last**, for everything that is genuinely either a command or a
///    question.
///
/// # Examples
///
/// ```
/// use shx::app::input_router::{Routed, route};
/// use shx::app::session_state::SessionState;
/// use shx::config::ResolvedConfig;
/// use shx::shell::WorkingDirectory;
///
/// let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
///     .expect("valid settings");
/// let state = SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config);
///
/// assert_eq!(route("cd src", &state), Routed::ChangeDirectory("src".to_owned()));
/// assert_eq!(route("ls", &state), Routed::RunCommand("ls".to_owned()));
/// assert_eq!(route("what is a monad", &state), Routed::AskModel("what is a monad".to_owned()));
/// ```
#[must_use]
pub fn route(input: &str, _state: &SessionState) -> Routed {
    let line = input.trim();

    if line.is_empty() {
        return Routed::Empty;
    }

    if is_session_exit(line) {
        return Routed::SessionExit;
    }

    if is_screen_clear(line) {
        return Routed::ScreenClear;
    }

    if is_directory_change(line) {
        return Routed::ChangeDirectory(cd_target(line).unwrap_or_default());
    }

    if let Some(command) = SlashCommand::parse(line) {
        return Routed::Slash(command);
    }

    if is_verb_with_arguments(line) {
        return Routed::AskModel(line.to_owned());
    }

    match classify_input(line) {
        InputKind::Command => Routed::RunCommand(line.to_owned()),
        InputKind::Request => Routed::AskModel(line.to_owned()),
    }
}

/// Whether the line starts with a word that is both a session verb and a
/// command name, and then says something else.
///
/// `exit` and `clear` are in the command catalogue, so `classify_input` reads
/// "exit code 0" as a command. It is not one: neither verb takes an argument, so
/// a second word means the user is asking about the word rather than typing it.
fn is_verb_with_arguments(line: &str) -> bool {
    const VERBS: &[&str] = &["exit", "quit", "clear", "cls"];

    let mut words = line.split_whitespace();
    let Some(first) = words.next() else {
        return false;
    };

    VERBS.iter().any(|verb| first.eq_ignore_ascii_case(verb)) && words.next().is_some()
}

#[cfg(test)]
mod tests {
    use super::{Routed, route};
    use crate::app::session_state::SessionState;
    use crate::app::slash_commands::SlashCommand;
    use crate::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    use crate::shell::WorkingDirectory;

    fn state() -> SessionState {
        let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
            .expect("valid settings");
        SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config)
    }

    fn routed(line: &str) -> Routed {
        route(line, &state())
    }

    #[test]
    fn an_empty_line_is_discarded() {
        assert_eq!(routed(""), Routed::Empty);
        assert_eq!(routed("   "), Routed::Empty);
        assert_eq!(routed("\t\n "), Routed::Empty);
    }

    #[test]
    fn an_empty_line_is_discarded_rather_than_sent_to_a_model() {
        assert_ne!(routed("    "), Routed::AskModel(String::new()));
    }

    #[test]
    fn exit_and_quit_end_the_session() {
        assert_eq!(routed("exit"), Routed::SessionExit);
        assert_eq!(routed("quit"), Routed::SessionExit);
        assert_eq!(routed("  EXIT  "), Routed::SessionExit);
    }

    #[test]
    fn a_sentence_starting_with_exit_is_a_question_not_a_quit() {
        assert_eq!(
            routed("exit code 0"),
            Routed::AskModel("exit code 0".to_owned())
        );
        assert!(!routed("exit code 0").ends_session());
    }

    #[test]
    fn a_sentence_starting_with_clear_is_a_question_not_a_clear() {
        assert_eq!(
            routed("clear the cache"),
            Routed::AskModel("clear the cache".to_owned())
        );
    }

    #[test]
    fn clear_and_cls_clear_the_screen() {
        assert_eq!(routed("clear"), Routed::ScreenClear);
        assert_eq!(routed("cls"), Routed::ScreenClear);
        assert_eq!(routed("  Clear  "), Routed::ScreenClear);
    }

    #[test]
    fn cd_carries_its_target() {
        assert_eq!(routed("cd src"), Routed::ChangeDirectory("src".to_owned()));
        assert_eq!(
            routed("cd /tmp"),
            Routed::ChangeDirectory("/tmp".to_owned())
        );
        assert_eq!(routed("cd ~"), Routed::ChangeDirectory("~".to_owned()));
    }

    #[test]
    fn a_bare_cd_carries_an_empty_target_meaning_home() {
        assert_eq!(routed("cd"), Routed::ChangeDirectory(String::new()));
        assert_eq!(routed("cd    "), Routed::ChangeDirectory(String::new()));
    }

    #[test]
    fn cd_is_not_confused_with_a_word_that_merely_starts_with_it() {
        assert!(!matches!(routed("cdrom"), Routed::ChangeDirectory(_)));
        assert_eq!(routed("cd my cd"), Routed::ChangeDirectory("my".to_owned()));
        assert_eq!(routed("ls cd"), Routed::RunCommand("ls cd".to_owned()));
    }

    #[test]
    fn cd_is_not_run_as_a_child_process() {
        assert!(
            !matches!(routed("cd src"), Routed::RunCommand(_)),
            "a cd that ran in a child process would lose the change"
        );
    }

    #[test]
    fn a_slash_command_is_routed_to_the_slash_parser() {
        assert_eq!(routed("/help"), Routed::Slash(SlashCommand::Help));
        assert_eq!(
            routed("/model gpt-5"),
            Routed::Slash(SlashCommand::Model(Some("gpt-5".to_owned())))
        );
    }

    #[test]
    fn a_slash_command_is_never_read_as_a_path() {
        assert!(!matches!(routed("/help"), Routed::RunCommand(_)));
        assert!(!matches!(
            routed("/usr/local/bin/tool"),
            Routed::RunCommand(_)
        ));
    }

    #[test]
    fn an_unknown_slash_command_still_routes_to_the_slash_parser() {
        assert_eq!(
            routed("/nope"),
            Routed::Slash(SlashCommand::Unknown {
                name: "nope".to_owned(),
                argument: None,
            })
        );
    }

    #[test]
    fn a_known_command_is_run() {
        for line in ["ls", "git status", "cargo build", "./build.sh", "a | b"] {
            assert_eq!(
                routed(line),
                Routed::RunCommand(line.to_owned()),
                "{line:?}"
            );
        }
    }

    #[test]
    fn a_sentence_is_asked_of_the_model() {
        for line in [
            "what is a monad",
            "install ripgrep for me",
            "explain this code",
        ] {
            assert_eq!(routed(line), Routed::AskModel(line.to_owned()), "{line:?}");
        }
    }

    #[test]
    fn the_truth_table_holds_for_the_three_shapes() {
        assert!(matches!(routed("cd src"), Routed::ChangeDirectory(_)));
        assert!(matches!(routed("ls"), Routed::RunCommand(_)));
        assert!(matches!(routed("what is a monad"), Routed::AskModel(_)));
    }

    #[test]
    fn routing_trims_the_line_it_returns() {
        assert_eq!(
            routed("   ls -la   "),
            Routed::RunCommand("ls -la".to_owned())
        );
        assert_eq!(
            routed("  what is this  "),
            Routed::AskModel("what is this".to_owned())
        );
    }

    #[test]
    fn routing_does_not_disturb_the_session() {
        let state = state();
        let before = state.working_directory().display();
        let before_requests = state.request_count();

        let _ = route("rm -rf /", &state);
        let _ = route("exit", &state);
        let _ = route("cd /", &state);

        assert_eq!(state.working_directory().display(), before);
        assert_eq!(state.request_count(), before_requests);
        assert!(state.history_is_empty());
    }

    #[test]
    fn a_command_the_safety_tier_would_refuse_is_still_routed() {
        assert_eq!(
            routed("rm -rf /"),
            Routed::RunCommand("rm -rf /".to_owned()),
            "refusing is the app's job, not the router's"
        );
    }

    #[test]
    fn only_the_exit_routes_end_the_session() {
        for line in ["", "quit", "cls", "/exit", "ls", "exit code 0", "/nope"] {
            assert_eq!(
                routed(line).ends_session(),
                line == "quit" || line == "/exit",
                "{line:?}"
            );
        }
    }
}
