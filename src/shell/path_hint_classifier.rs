//! Whether the first word of a line is a path to something runnable.
//!
//! `./build.sh` and `/usr/bin/env python` are commands. `shx` cannot know
//! whether the file exists, and does not need to: a leading `./` or an absolute
//! path is a strong enough signal, and treating it as a command is safe because
//! the user is shown it before it runs.

/// Whether the first word of the line looks like a path to an executable.
///
/// # Examples
///
/// ```
/// use shx::shell::looks_like_path;
///
/// assert!(looks_like_path("./build.sh"));
/// assert!(looks_like_path("/usr/local/bin/thing"));
/// assert!(looks_like_path("../sibling/run"));
/// assert!(!looks_like_path("git status"));
/// ```
#[must_use]
pub fn looks_like_path(input: &str) -> bool {
    let Some(first) = input.split_whitespace().next() else {
        return false;
    };

    if first.starts_with('~') {
        return true;
    }

    first.starts_with("./")
        || first.starts_with("../")
        || first.starts_with('/')
        || first == "."
        || first == ".."
}

/// Whether the first word ends in a file extension that suggests a script.
///
/// A second, weaker signal: `run.py` with no path prefix is probably still a
/// command, and `README.md` is very probably not.
///
/// # Examples
///
/// ```
/// use shx::shell::looks_like_script;
///
/// assert!(looks_like_script("script.py"));
/// assert!(!looks_like_script("README.md"));
/// ```
#[must_use]
pub fn looks_like_script(input: &str) -> bool {
    const SCRIPT_EXTENSIONS: &[&str] = &[
        "sh", "bash", "zsh", "fish", "py", "rb", "pl", "js", "ts", "lua", "php",
    ];

    let Some(first) = input.split_whitespace().next() else {
        return false;
    };

    let Some((_, extension)) = first.rsplit_once('.') else {
        return false;
    };

    SCRIPT_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::{looks_like_path, looks_like_script};

    #[test]
    fn relative_and_absolute_paths_count() {
        for input in [
            "./build.sh",
            "../up/run",
            "/usr/bin/env",
            ".",
            "..",
            "~/bin/thing",
        ] {
            assert!(looks_like_path(input), "{input:?} should look like a path");
        }
    }

    #[test]
    fn bare_names_do_not() {
        for input in ["git status", "ls", "", "   ", "cargo build"] {
            assert!(
                !looks_like_path(input),
                "{input:?} should not look like a path"
            );
        }
    }

    #[test]
    fn a_bare_path_with_a_separator_is_left_to_the_model() {
        assert!(!looks_like_path("src/main.rs"));
        assert!(!looks_like_path("usr/local/bin/thing"));
    }

    #[test]
    fn a_leading_dot_that_is_not_a_path_prefix_is_not_a_path() {
        assert!(!looks_like_path(".gitignore"));
        assert!(!looks_like_path(".editorconfig"));
    }

    #[test]
    fn script_extensions_are_recognised() {
        for input in ["build.sh", "run.py", "script.rb", "main.TS"] {
            assert!(
                looks_like_script(input),
                "{input:?} should look like a script"
            );
        }
    }

    #[test]
    fn document_extensions_are_not_scripts() {
        for input in ["README.md", "notes.txt", "config.yaml", "Makefile"] {
            assert!(
                !looks_like_script(input),
                "{input:?} should not look like a script"
            );
        }
    }

    #[test]
    fn only_the_first_word_is_examined() {
        assert!(!looks_like_path("cat README.md"));
        assert!(!looks_like_script("cat run.py"));
    }

    #[test]
    fn a_dotted_filename_is_not_treated_as_a_script() {
        assert!(!looks_like_script(".gitignore"));
    }
}
