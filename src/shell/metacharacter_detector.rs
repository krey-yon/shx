//! Whether a line contains shell syntax that settles the question.
//!
//! A pipe, a redirect or a subshell means the user is writing a command even if
//! every word in it is a word and not a command name. Checking for these is
//! cheaper and more reliable than guessing from the words.

/// Characters that mean the line is a command line rather than a sentence.
const METACHARACTERS: &[char] = &['|', '&', ';', '<', '>', '`', '$', '(', ')'];

/// Whether the line contains any shell metacharacter.
///
/// Note that `&&` and `||` both trip on `&` and `|`, which is what we want: they
/// are commands. A redirection to a file is a command. A `$(...)` substitution
/// is a command.
///
/// # Examples
///
/// ```
/// use shx::shell::contains_metacharacter;
///
/// assert!(contains_metacharacter("ls | grep foo"));
/// assert!(contains_metacharacter("echo hi > out.txt"));
/// assert!(!contains_metacharacter("what is a monad"));
/// ```
#[must_use]
pub fn contains_metacharacter(input: &str) -> bool {
    input
        .chars()
        .any(|character| METACHARACTERS.contains(&character))
}

/// Which metacharacter was found first, for a message that can point at it.
///
/// # Examples
///
/// ```
/// use shx::shell::metacharacter_detector::first_metacharacter;
///
/// assert_eq!(first_metacharacter("ls | grep x"), Some('|'));
/// assert_eq!(first_metacharacter("hello"), None);
/// ```
#[must_use]
pub fn first_metacharacter(input: &str) -> Option<char> {
    input
        .chars()
        .find(|character| METACHARACTERS.contains(character))
}

#[cfg(test)]
mod tests {
    use super::{contains_metacharacter, first_metacharacter};

    #[test]
    fn pipes_and_redirects_count() {
        for input in [
            "ls | wc -l",
            "ls > out.txt",
            "cat < in.txt",
            "ls >> out.txt",
            "ls; pwd",
            "a && b",
            "a || b",
        ] {
            assert!(contains_metacharacter(input), "{input:?} should count");
        }
    }

    #[test]
    fn substitutions_count() {
        for input in ["echo $(pwd)", "echo `date`", "ls $HOME", "(cd /tmp && ls)"] {
            assert!(contains_metacharacter(input), "{input:?} should count");
        }
    }

    #[test]
    fn prose_does_not_count() {
        for input in [
            "what is a monad",
            "explain the difference between map and filter",
            "how do I read a file backwards",
            "write a hello world script in python",
            "why is my disk full",
        ] {
            assert!(!contains_metacharacter(input), "{input:?} should not count");
        }
    }

    #[test]
    fn the_first_one_is_reported() {
        assert_eq!(first_metacharacter("ls > out | wc"), Some('>'));
        assert_eq!(first_metacharacter("plain text"), None);
    }

    #[test]
    fn an_empty_line_has_nothing() {
        assert!(!contains_metacharacter(""));
        assert_eq!(first_metacharacter(""), None);
    }
}
