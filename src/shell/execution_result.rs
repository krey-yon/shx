//! What a command did: its output, its exit status, and how long it took.

use std::fmt;
use std::time::Duration;

/// The result of running a command.
///
/// Not a `Result`, because a command that exits non-zero did not fail to run —
/// `grep` finding nothing is a successful execution. The caller decides what a
/// particular status means for a particular command.
///
/// # Examples
///
/// ```
/// use shx::shell::execution_result::ExecutionResult;
/// use std::time::Duration;
///
/// let result = ExecutionResult {
///     command: "ls".to_owned(),
///     stdout: "Cargo.toml".to_owned(),
///     stderr: String::new(),
///     exit_code: Some(0),
///     duration: Duration::from_millis(4),
/// };
/// assert!(result.succeeded());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    /// The command line that was run, for echoing in the history.
    pub command: String,
    /// Everything written to standard output.
    pub stdout: String,
    /// Everything written to standard error.
    pub stderr: String,
    /// The exit status, or `None` if the process was killed by a signal.
    pub exit_code: Option<i32>,
    /// How long the process ran.
    pub duration: Duration,
}

impl ExecutionResult {
    /// Whether the command exited zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::execution_result::ExecutionResult;
    /// use std::time::Duration;
    ///
    /// let failed = ExecutionResult {
    ///     command: "false".to_owned(),
    ///     stdout: String::new(),
    ///     stderr: String::new(),
    ///     exit_code: Some(1),
    ///     duration: Duration::ZERO,
    /// };
    /// assert!(!failed.succeeded());
    /// ```
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }

    /// Everything the command printed, standard error included.
    ///
    /// Merged in that order because that is the order a terminal shows them in,
    /// and a caller rendering one block should not have to interleave by
    /// timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::execution_result::ExecutionResult;
    /// use std::time::Duration;
    ///
    /// let result = ExecutionResult {
    ///     command: "ls".to_owned(),
    ///     stdout: "out".to_owned(),
    ///     stderr: "err".to_owned(),
    ///     exit_code: Some(1),
    ///     duration: Duration::ZERO,
    /// };
    /// assert_eq!(result.combined_output(), "out\nerr");
    /// ```
    #[must_use]
    pub fn combined_output(&self) -> String {
        match (self.stdout.trim_end(), self.stderr.trim_end()) {
            ("", "") => String::new(),
            (out, "") => out.to_owned(),
            ("", err) => err.to_owned(),
            (out, err) => format!("{out}\n{err}"),
        }
    }

    /// The exit status as it would be shown to a user.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::execution_result::ExecutionResult;
    /// use std::time::Duration;
    ///
    /// let killed = ExecutionResult {
    ///     command: "sleep 100".to_owned(),
    ///     stdout: String::new(),
    ///     stderr: String::new(),
    ///     exit_code: None,
    ///     duration: Duration::from_secs(30),
    /// };
    /// assert_eq!(killed.status_label(), "killed");
    /// ```
    #[must_use]
    pub const fn status_label(&self) -> &'static str {
        match self.exit_code {
            Some(0) => "success",
            Some(_) => "failed",
            None => "killed",
        }
    }

    /// A one-line summary for the history file and for `--verbose`.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::execution_result::ExecutionResult;
    /// use std::time::Duration;
    ///
    /// let result = ExecutionResult {
    ///     command: "git status".to_owned(),
    ///     stdout: String::new(),
    ///     stderr: String::new(),
    ///     exit_code: Some(0),
    ///     duration: Duration::from_millis(12),
    /// };
    /// assert_eq!(result.summary(), "git status  [success in 12ms]");
    /// ```
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}  [{} in {}ms]",
            self.command,
            self.status_label(),
            self.duration.as_millis()
        )
    }
}

impl fmt::Display for ExecutionResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.combined_output())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::ExecutionResult;

    fn result(exit_code: Option<i32>, stdout: &str, stderr: &str) -> ExecutionResult {
        ExecutionResult {
            command: "test".to_owned(),
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
            exit_code,
            duration: Duration::from_millis(5),
        }
    }

    #[test]
    fn zero_is_success_and_anything_else_is_not() {
        assert!(result(Some(0), "", "").succeeded());
        assert!(!result(Some(1), "", "").succeeded());
        assert!(!result(Some(127), "", "").succeeded());
    }

    #[test]
    fn a_signalled_process_is_not_a_success() {
        // exit_code is None, not Some(0). Reporting a killed process as
        // successful is how a timeout silently looks like a passing test.
        assert!(!result(None, "", "").succeeded());
    }

    #[test]
    fn combined_output_merges_in_terminal_order() {
        assert_eq!(result(Some(0), "out", "err").combined_output(), "out\nerr");
    }

    #[test]
    fn combined_output_omits_an_empty_stream() {
        assert_eq!(result(Some(0), "out", "").combined_output(), "out");
        assert_eq!(result(Some(0), "", "err").combined_output(), "err");
    }

    #[test]
    fn combined_output_of_nothing_is_empty_not_a_newline() {
        assert_eq!(result(Some(0), "", "").combined_output(), "");
        assert_eq!(result(Some(0), "  \n", "\n").combined_output(), "");
    }

    #[test]
    fn trailing_newlines_are_trimmed_so_blocks_do_not_drift() {
        // A trailing newline on both streams would otherwise print a blank line
        // between every command's output and the next prompt.
        assert_eq!(
            result(Some(0), "out\n\n", "err\n").combined_output(),
            "out\nerr"
        );
    }

    #[test]
    fn status_labels_distinguish_the_three_outcomes() {
        assert_eq!(result(Some(0), "", "").status_label(), "success");
        assert_eq!(result(Some(2), "", "").status_label(), "failed");
        assert_eq!(result(None, "", "").status_label(), "killed");
    }

    #[test]
    fn the_summary_names_the_command_and_the_outcome() {
        let summary = result(Some(0), "", "").summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("success"));
    }

    #[test]
    fn display_shows_the_output_not_the_metadata() {
        assert_eq!(result(Some(0), "hello", "").to_string(), "hello");
    }
}
