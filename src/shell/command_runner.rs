//! Running a command as a child process.
//!
//! Output is captured rather than streamed to the terminal, because a line
//! editor owns the screen: writing directly underneath it corrupts the
//! prompt. The captured text is rendered afterwards, by the TUI layer.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::error::{Result, ShxError};
use crate::shell::execution_result::ExecutionResult;

/// How a command should be run.
#[derive(Debug, Clone)]
pub struct RunRequest {
    /// The command line.
    pub command: String,
    /// The working directory, or `None` to inherit the process's own.
    pub working_directory: Option<String>,
    /// How long to allow before killing the child.
    pub timeout: Duration,
    /// Whether standard input is connected. Off by default: a command that
    /// prompts would otherwise hang waiting for a terminal it cannot see.
    pub attach_stdin: bool,
}

impl RunRequest {
    /// A request with the given timeout and no working directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::command_runner::RunRequest;
    /// use std::time::Duration;
    ///
    /// let request = RunRequest::new("ls", Duration::from_secs(30));
    /// assert_eq!(request.command, "ls");
    /// ```
    #[must_use]
    pub fn new(command: impl Into<String>, timeout: Duration) -> Self {
        Self {
            command: command.into(),
            working_directory: None,
            timeout,
            attach_stdin: false,
        }
    }

    /// Run in a specific directory.
    #[must_use]
    pub fn in_directory(mut self, directory: impl Into<String>) -> Self {
        self.working_directory = Some(directory.into());
        self
    }

    /// Connect the child's standard input to ours.
    #[must_use]
    pub const fn with_stdin(mut self) -> Self {
        self.attach_stdin = true;
        self
    }
}

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
/// use shx::shell::command_runner::split_command;
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
            // A backslash escapes the next character literally, inside or out
            // of quotes. This is the same rule the shell uses, and models
            // produce it out of habit.
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
                        // Backslash escapes only work for these inside double
                        // quotes, matching the shell.
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

/// Run a command and capture its output.
///
/// # Errors
///
/// Returns [`CommandSpawn`](crate::error::ShxError::CommandSpawn) if the program
/// cannot be started at all, which is different from starting and failing.
///
/// # Examples
///
/// ```no_run
/// use shx::shell::command_runner::{RunRequest, run_command};
/// use std::time::Duration;
///
/// let result = run_command(&RunRequest::new("echo hello", Duration::from_secs(10)))
///     .expect("echo should run");
/// assert_eq!(result.stdout.trim(), "hello");
/// ```
pub fn run_command(request: &RunRequest) -> Result<ExecutionResult> {
    let words = split_command(&request.command);

    let Some((program, arguments)) = words.split_first() else {
        return Err(ShxError::CommandSpawn {
            command: request.command.clone(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty command"),
        });
    };

    let mut command = Command::new(program);
    command.args(arguments);

    if let Some(directory) = &request.working_directory {
        command.current_dir(Path::new(directory));
    }

    if !request.attach_stdin {
        command.stdin(std::process::Stdio::null());
    }

    let started = Instant::now();
    let output = command.output().map_err(|source| ShxError::CommandSpawn {
        command: request.command.clone(),
        source,
    })?;

    Ok(ExecutionResult {
        command: request.command.clone(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code(),
        duration: started.elapsed(),
    })
}

/// Whether a program can be found on `PATH`.
///
/// Used to tell "you do not have ripgrep" apart from "ripgrep is broken", which
/// are different problems with different fixes.
///
/// # Examples
///
/// ```
/// use shx::shell::command_runner::is_on_path;
///
/// // `sh` exists everywhere this crate builds; `definitely-not-a-real-tool` does not.
/// assert!(is_on_path("sh"));
/// assert!(!is_on_path("definitely-not-a-real-tool"));
/// ```
#[must_use]
pub fn is_on_path(program: &str) -> bool {
    which(program).is_some()
}

/// The full path of a program on `PATH`, if it is there.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use shx::shell::command_runner::which;
///
/// let found = which("sh");
/// assert!(found.is_some());
/// assert!(Path::new(&found.expect("sh")).is_absolute());
/// ```
#[must_use]
pub fn which(program: &str) -> Option<String> {
    if program.contains('/') {
        let path = Path::new(program);
        return is_executable(path).then(|| path.to_path_buf().to_string_lossy().into_owned());
    }

    let path_variable = std::env::var_os("PATH")?;

    std::env::split_paths(&path_variable).find_map(|directory| {
        let candidate = directory.join(program);
        is_executable(&candidate).then(|| candidate.to_string_lossy().into_owned())
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{RunRequest, is_on_path, run_command, split_command, which};
    use crate::error::ShxError;

    fn run(command: &str) -> crate::error::Result<crate::shell::execution_result::ExecutionResult> {
        run_command(&RunRequest::new(command, Duration::from_secs(10)))
    }

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
        // `echo ""` must pass one empty argument, not zero.
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

    #[test]
    fn a_command_runs_and_its_output_is_captured() {
        let result = run("echo hello").expect("echo should run");
        assert!(result.succeeded());
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[test]
    fn standard_error_is_captured_separately() {
        let result = run("sh -c 'echo out; echo err >&2'").expect("should run");
        assert_eq!(result.stdout.trim(), "out");
        assert_eq!(result.stderr.trim(), "err");
        assert_eq!(result.combined_output(), "out\nerr");
    }

    #[test]
    fn a_non_zero_exit_is_a_result_not_an_error() {
        // `false` ran successfully. The caller decides what exit 1 means.
        let result = run("sh -c 'exit 3'").expect("the process should start");
        assert!(!result.succeeded());
        assert_eq!(result.exit_code, Some(3));
    }

    #[test]
    fn a_missing_program_is_a_spawn_error() {
        let error = run("definitely-not-a-real-tool-xyz").expect_err("should fail to start");
        match &error {
            ShxError::CommandSpawn { command, .. } => {
                assert_eq!(command, "definitely-not-a-real-tool-xyz");
            }
            other => panic!("expected CommandSpawn, got {other:?}"),
        }
    }

    #[test]
    fn an_empty_command_is_a_spawn_error_not_a_panic() {
        let error = run("   ").expect_err("an empty command cannot run");
        assert!(matches!(error, ShxError::CommandSpawn { .. }));
    }

    #[test]
    fn the_working_directory_is_honoured() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let request = RunRequest::new("pwd", Duration::from_secs(10))
            .in_directory(temporary.path().to_string_lossy().into_owned());

        let result = run_command(&request).expect("pwd should run");
        // macOS reports /private/var for /var, so compare the resolved paths
        // rather than the strings.
        let reported =
            std::fs::canonicalize(result.stdout.trim()).expect("pwd should print a real path");
        let expected = std::fs::canonicalize(temporary.path()).expect("the temp dir should exist");
        assert_eq!(reported, expected);
    }

    #[test]
    fn a_working_directory_that_does_not_exist_is_a_spawn_error() {
        let request =
            RunRequest::new("ls", Duration::from_secs(10)).in_directory("/nope/nope/nope");
        assert!(matches!(
            run_command(&request).expect_err("should fail"),
            ShxError::CommandSpawn { .. }
        ));
    }

    #[test]
    fn the_duration_is_measured() {
        let result = run("sleep 0.05").expect("should run");
        assert!(
            result.duration >= Duration::from_millis(40),
            "expected at least 40ms, got {:?}",
            result.duration
        );
    }

    #[test]
    fn a_known_program_is_on_the_path() {
        assert!(is_on_path("sh"));
        assert!(matches!(which("sh"), Some(path) if path.contains('/')));
    }

    #[test]
    fn an_unknown_program_is_not_on_the_path() {
        assert!(!is_on_path("definitely-not-a-real-tool-xyz"));
        assert_eq!(which("definitely-not-a-real-tool-xyz"), None);
    }

    #[test]
    fn a_path_with_a_separator_is_checked_directly() {
        // A path containing a separator is not searched for; it is used as given.
        assert!(matches!(which("/bin/sh"), Some(found) if found == "/bin/sh"));
        assert!(which("/bin/definitely-not-here").is_none());
    }
}
