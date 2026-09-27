//! Detecting commands a model wrote as a template rather than as a command.
//!
//! A model asked to "install ripgrep on macOS" may answer `brew install
//! <package>`, forgetting the placeholder. Such a command is forced open for
//! editing instead of being run.

use regex::Regex;
use std::sync::LazyLock;

static ANGLE_PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<([A-Za-z_][A-Za-z0-9_ -]{0,40})>")
        .unwrap_or_else(|error| unreachable!("built-in placeholder regex must compile: {error}"))
});

static TEMPLATE_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:your|my|the)\b[\s_-]*(?:\w+[\s_-]+)?(?:file|filename|path|dir",
        r"|directory|branch|repo|repository|url|name|username|email|host|server|port",
        r"|token|key|secret|password|project|folder|database|db|org|group|container",
        r"|image|package|module)\b",
    ))
    .unwrap_or_else(|error| unreachable!("built-in template-word regex must compile: {error}"))
});

/// Why a command was held back, or `None` if it was not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderReason {
    /// A `<placeholder>` is present.
    AngleBrackets,
    /// A word like `your-path` or `my-branch` is present.
    TemplateWord,
}

impl PlaceholderReason {
    /// A message to show above the editor.
    #[must_use]
    pub fn message(self) -> &'static str {
        match self {
            Self::AngleBrackets => {
                "This command still has placeholder text. Edit it before running."
            }
            Self::TemplateWord => "This command looks like a template. Edit it before running.",
        }
    }
}

/// Whether the command still contains an unfilled placeholder.
///
/// # Examples
///
/// ```
/// use shx::shell::has_placeholder;
///
/// assert!(has_placeholder("brew install <package>"));
/// assert!(has_placeholder("git clone <your-repo-url>"));
/// assert!(!has_placeholder("brew install ripgrep"));
/// assert!(!has_placeholder("echo 'a < b'"));
/// ```
#[must_use]
pub fn has_placeholder(command: &str) -> bool {
    placeholder_reason(command).is_some()
}

/// Why the command was held back, or `None` if it was not.
///
/// # Examples
///
/// ```
/// use shx::shell::placeholder_scanner::{PlaceholderReason, placeholder_reason};
///
/// assert_eq!(
///     placeholder_reason("brew install <package>"),
///     Some(PlaceholderReason::AngleBrackets)
/// );
/// assert_eq!(placeholder_reason("brew install ripgrep"), None);
/// ```
#[must_use]
pub fn placeholder_reason(command: &str) -> Option<PlaceholderReason> {
    if ANGLE_PLACEHOLDER.is_match(command) {
        return Some(PlaceholderReason::AngleBrackets);
    }
    if TEMPLATE_WORDS.is_match(command) {
        return Some(PlaceholderReason::TemplateWord);
    }
    None
}

/// Each placeholder found, in order, for a message that lists them.
///
/// # Examples
///
/// ```
/// use shx::shell::placeholder_scanner::find_placeholders;
///
/// let found = find_placeholders("cp <source-file> <destination-file>");
/// assert_eq!(found, vec!["source-file", "destination-file"]);
/// ```
#[must_use]
pub fn find_placeholders(command: &str) -> Vec<String> {
    ANGLE_PLACEHOLDER
        .captures_iter(command)
        .map(|capture| capture[1].to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PlaceholderReason, find_placeholders, has_placeholder, placeholder_reason};

    #[test]
    fn an_angle_placeholder_is_detected() {
        for command in [
            "brew install <package>",
            "git clone <your-repo-url>",
            "cp <source> <destination>",
            "curl -H 'Authorization: <token>' https://api.example",
        ] {
            assert!(has_placeholder(command), "{command:?} should be held back");
        }
    }

    #[test]
    fn a_finished_command_is_not_held_back() {
        for command in [
            "brew install ripgrep",
            "git clone https://github.com/BurntSushi/ripgrep",
            "cargo build --release",
            "ls -la /tmp",
        ] {
            assert!(!has_placeholder(command), "{command:?} should run as is");
        }
    }

    #[test]
    fn a_bare_angle_bracket_is_a_redirect_not_a_placeholder() {
        for command in [
            "sort < input.txt",
            "echo hi > out.txt",
            "grep pattern < in > out",
            "diff <(ls a) <(ls b)",
        ] {
            assert!(!has_placeholder(command), "{command:?} should run as is");
        }
    }

    #[test]
    fn a_digit_after_the_bracket_is_not_a_placeholder() {
        assert!(!has_placeholder("printf '%s' arr<0"));
    }

    #[test]
    fn an_empty_bracket_is_not_a_placeholder() {
        assert!(!has_placeholder("echo <>"));
    }

    #[test]
    fn template_words_are_detected() {
        for command in [
            "cd your-project",
            "git checkout my-branch",
            "ssh into your-server",
            "edit the config file",
        ] {
            assert!(has_placeholder(command), "{command:?} should be held back");
        }
    }

    #[test]
    fn a_template_lookalike_is_not_held_back() {
        for command in [
            "ls the-reports",
            "cat my-notes.txt",
            "echo the quick brown fox",
        ] {
            assert!(!has_placeholder(command), "{command:?} should run as is");
        }
    }

    #[test]
    fn the_placeholders_are_listed_in_order() {
        let found = find_placeholders("cp <source-file> <destination-file>");
        assert_eq!(found, vec!["source-file", "destination-file"]);
    }

    #[test]
    fn listing_returns_nothing_for_a_clean_command() {
        assert!(find_placeholders("brew install ripgrep").is_empty());
    }

    #[test]
    fn the_reason_distinguishes_the_two_kinds() {
        assert_eq!(
            placeholder_reason("brew install <package>"),
            Some(PlaceholderReason::AngleBrackets)
        );
        assert_eq!(
            placeholder_reason("cd your-project"),
            Some(PlaceholderReason::TemplateWord)
        );
        assert_eq!(placeholder_reason("brew install ripgrep"), None);
    }

    #[test]
    fn angle_brackets_win_when_both_are_present() {
        assert_eq!(
            placeholder_reason("cd your-project/<dir>"),
            Some(PlaceholderReason::AngleBrackets)
        );
    }

    #[test]
    fn both_messages_tell_the_user_to_edit() {
        for reason in [
            PlaceholderReason::AngleBrackets,
            PlaceholderReason::TemplateWord,
        ] {
            assert!(reason.message().contains("Edit"));
        }
    }
}
