//! Splitting a command line into a program and its arguments.

/// Split a command line into a program and its arguments.
///
/// A hand-rolled splitter rather than a shell, deliberately. Going through
/// `sh -c` would make every command a shell script, which means quoting rules
/// the safety analyser would then have to reason about, and it would let a
/// suggested command run as arbitrary shell. Splitting here means what the user
/// sees is what runs.
///
/// Handles single quotes, double quotes and backslash escapes, which covers
/// every command in the catalogue and most of what a model suggests.
///
/// # Examples
///
/// ```
/// use shx::shell::command_splitter::split_command;
///
/// assert_eq!(split_command("ls -la"), vec!["ls", "-la"]);
/// assert_eq!(split_command("grep 'a b' file"), vec!["grep", "a b", "file"]);
/// assert_eq!(split_command(r#"echo "hi there""#), vec!["echo", "hi there"]);
/// ```
#[must_use]
pub fn split_command(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut has_word = false;
    let mut characters = command.chars();

    while let Some(character) = characters.next() {
        match character {
            '\\' => {
                has_word = true;
                if let Some(escaped) = characters.next() {
                    current.push(escaped);
                }
            }

            '\'' => {
                has_word = true;
                for next in characters.by_ref() {
                    if next == '\'' {
                        break;
                    }
                    current.push(next);
                }
            }

            '"' => {
                has_word = true;
                while let Some(next) = characters.next() {
                    match next {
                        '"' => break,
                        '\\' => {
                            if let Some(escaped) = characters.next() {
                                current.push(escaped);
                            }
                        }
                        other => current.push(other),
                    }
                }
            }

            character if character.is_whitespace() => {
                if has_word {
                    words.push(std::mem::take(&mut current));
                    has_word = false;
                }
            }

            other => {
                has_word = true;
                current.push(other);
            }
        }
    }

    if has_word {
        words.push(current);
    }

    words
}

#[cfg(test)]
mod tests {
    use super::split_command;

    #[test]
    fn plain_words_split_on_whitespace() {
        assert_eq!(split_command("ls -la /tmp"), vec!["ls", "-la", "/tmp"]);
    }

    #[test]
    fn runs_of_whitespace_collapse() {
        assert_eq!(split_command("  ls   -la  "), vec!["ls", "-la"]);
        assert_eq!(split_command("ls\t-la"), vec!["ls", "-la"]);
    }

    #[test]
    fn single_quotes_group_words() {
        assert_eq!(
            split_command("grep 'a b' file"),
            vec!["grep", "a b", "file"]
        );
    }

    #[test]
    fn double_quotes_group_words() {
        assert_eq!(
            split_command(r#"echo "hi there""#),
            vec!["echo", "hi there"]
        );
    }

    #[test]
    fn backslash_escapes_the_next_character() {
        assert_eq!(split_command(r"echo a\ b"), vec!["echo", "a b"]);
    }

    #[test]
    fn an_empty_quoted_string_is_still_a_word() {
        assert_eq!(split_command(r#"echo """#), vec!["echo", ""]);
        assert_eq!(split_command("echo ''"), vec!["echo", ""]);
    }

    #[test]
    fn an_unterminated_quote_does_not_lose_the_word() {
        let words = split_command(r#"echo "unterminated"#);
        assert_eq!(words, vec!["echo", "unterminated"]);
    }

    #[test]
    fn empty_input_splits_to_nothing() {
        assert!(split_command("").is_empty());
        assert!(split_command("    ").is_empty());
    }
}
