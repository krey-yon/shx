//! The decision: is this line a command to run, or a question to ask?
//!
//! The classifier is a small ordered set of rules, most specific first. Order
//! matters more than any single rule: a line that both starts with `./` and
//! contains a question word is a command, because the path is the stronger
//! signal.

use crate::shell::command_catalogue::is_known_command;
use crate::shell::metacharacter_detector::contains_metacharacter;
use crate::shell::path_hint_classifier::{looks_like_path, looks_like_script};

/// What a line of input turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// A shell command to run.
    Command,
    /// A natural-language request to send to a model.
    Request,
}

impl InputKind {
    /// Whether this is a command.
    #[must_use]
    pub const fn is_command(&self) -> bool {
        matches!(self, Self::Command)
    }
}

/// Classify a line of input.
///
/// Rules, in order:
///
/// 1. Empty input is a [`InputKind::Request`], which the caller discards.
///    Returning a value rather than an `Option` keeps the rules flat.
/// 2. A known command name, or a path, or a script, is an
///    [`InputKind::Command`].
/// 3. A line containing shell metacharacters is an [`InputKind::Command`].
/// 4. Anything else is an [`InputKind::Request`].
///
/// Rule 3 comes after rule 2 deliberately: `find . -name '*.rs' | head` starts
/// with a known command but the pipe is what settles it, and either order gives
/// the same answer. It is listed third because a metacharacter inside a word —
/// `what is a|b` — is weaker evidence than a command name at the start.
///
/// # Examples
///
/// ```
/// use shx::shell::classify_input;
///
/// assert!(classify_input("git status").is_command());
/// assert!(classify_input("find . -name '*.rs' | head -5").is_command());
/// assert!(!classify_input("what is a monad").is_command());
/// assert!(!classify_input("install ripgrep for me").is_command());
/// ```
#[must_use]
pub fn classify_input(input: &str) -> InputKind {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return InputKind::Request;
    }

    let first_word = trimmed.split_whitespace().next().unwrap_or(trimmed);

    // An absolute or explicitly relative path is the strongest signal there is:
    // the user wrote where the thing is.
    if looks_like_path(trimmed) {
        return InputKind::Command;
    }

    // A known command name at the start, with no leading word that would make
    // it an argument to something else.
    if is_known_command(first_word) {
        return InputKind::Command;
    }

    if looks_like_script(trimmed) {
        return InputKind::Command;
    }

    if contains_metacharacter(trimmed) {
        return InputKind::Command;
    }

    InputKind::Request
}

/// The first word of the line, which is what command-name matching looks at.
///
/// Exposed because the completion engine wants the same split.
///
/// # Examples
///
/// ```
/// use shx::shell::first_word;
///
/// assert_eq!(first_word("  git status "), Some("git"));
/// assert_eq!(first_word("   "), None);
/// ```
#[must_use]
pub fn first_word(input: &str) -> Option<&str> {
    input.split_whitespace().next()
}

/// Whether the line is a request to change directory.
///
/// Handled locally rather than by the model, because `cd` changes the shell's
/// state and asking a model what `cd src` means would eventually produce a
/// `Command` the app then has to special-case anyway.
///
/// # Examples
///
/// ```
/// use shx::shell::is_directory_change;
///
/// assert!(is_directory_change("cd src"));
/// assert!(is_directory_change("cd"));
/// assert!(!is_directory_change("cdrom"));
/// ```
#[must_use]
pub fn is_directory_change(input: &str) -> bool {
    matches!(first_word(input), Some(word) if word == "cd")
}

/// Whether the line is a request to clear the screen.
///
/// Matches the whole line, not just the first word. `clear the cache` is a
/// request to a model, and `clear` as a command with an argument is rare enough
/// that treating the whole line as the verb is the better default.
///
/// # Examples
///
/// ```
/// use shx::shell::is_screen_clear;
///
/// assert!(is_screen_clear("clear"));
/// assert!(is_screen_clear("  CLS  "));
/// assert!(!is_screen_clear("clear the cache"));
/// ```
#[must_use]
pub fn is_screen_clear(input: &str) -> bool {
    let line = input.trim();
    line.eq_ignore_ascii_case("clear") || line.eq_ignore_ascii_case("cls")
}

/// Whether the line is a request to end the session.
///
/// Matches the whole line. `exit code 0` is a question about a program, not an
/// instruction to quit, and reading it as one loses the user's session.
///
/// # Examples
///
/// ```
/// use shx::shell::is_session_exit;
///
/// assert!(is_session_exit("exit"));
/// assert!(is_session_exit("QUIT"));
/// assert!(!is_session_exit("exit code 0"));
/// ```
#[must_use]
pub fn is_session_exit(input: &str) -> bool {
    let line = input.trim();
    line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit")
}

#[cfg(test)]
mod tests {
    use super::{
        InputKind, classify_input, first_word, is_directory_change, is_screen_clear,
        is_session_exit,
    };

    #[test]
    fn a_known_command_is_a_command() {
        for input in [
            "ls",
            "git status",
            "cargo build --release",
            "docker ps -a",
            "sudo rm x",
        ] {
            assert_eq!(classify_input(input), InputKind::Command, "{input:?}");
        }
    }

    #[test]
    fn a_request_is_a_request() {
        for input in [
            "what is a monad",
            "install ripgrep for me",
            "explain this code",
            "write a hello world script in python",
            "how do I reverse a string in rust",
            "why is my build slow",
        ] {
            assert_eq!(classify_input(input), InputKind::Request, "{input:?}");
        }
    }

    #[test]
    fn a_pipe_makes_it_a_command_even_with_no_known_name() {
        assert_eq!(classify_input("frobnicate | wc"), InputKind::Command);
    }

    #[test]
    fn a_path_makes_it_a_command() {
        for input in ["./build.sh", "/usr/local/bin/thing", "~/bin/tool"] {
            assert_eq!(classify_input(input), InputKind::Command, "{input:?}");
        }
    }

    #[test]
    fn a_path_wins_over_a_request_shaped_word() {
        // "what" is a question word but "./what" is a file.
        assert_eq!(classify_input("./what"), InputKind::Command);
    }

    #[test]
    fn a_sentence_containing_a_word_that_is_a_command_is_still_a_request() {
        // `install` is a real command, but as the first word of this sentence
        // it is an English verb.
        assert_eq!(classify_input("install ripgrep for me"), InputKind::Request);
    }

    #[test]
    fn empty_input_is_discarded_rather_than_classified() {
        assert_eq!(classify_input(""), InputKind::Request);
        assert_eq!(classify_input("    "), InputKind::Request);
    }

    #[test]
    fn leading_whitespace_does_not_change_the_verdict() {
        assert_eq!(classify_input("   git status"), InputKind::Command);
        assert_eq!(classify_input("   what is this"), InputKind::Request);
    }

    #[test]
    fn first_word_handles_spacing() {
        assert_eq!(first_word("  git status "), Some("git"));
        assert_eq!(first_word("git"), Some("git"));
        assert_eq!(first_word("   "), None);
    }

    #[test]
    fn cd_is_recognised_only_as_a_whole_first_word() {
        assert!(is_directory_change("cd src"));
        assert!(is_directory_change("cd"));
        assert!(!is_directory_change("cdrom"));
        assert!(!is_directory_change("ls cd"));
    }

    #[test]
    fn clear_and_cls_are_screen_clears() {
        assert!(is_screen_clear("clear"));
        assert!(is_screen_clear("cls"));
        assert!(is_screen_clear("CLEAR"));
        assert!(!is_screen_clear("clear the cache"));
    }

    #[test]
    fn exit_and_quit_end_the_session() {
        assert!(is_session_exit("exit"));
        assert!(is_session_exit("quit"));
        assert!(is_session_exit("QUIT"));
        assert!(!is_session_exit("exit code 0"));
    }

    #[test]
    fn is_command_matches_the_variant() {
        assert!(InputKind::Command.is_command());
        assert!(!InputKind::Request.is_command());
    }
}
