//! What one session accumulates while it runs.
//!
//! Everything here is either cheap to copy or shared by reference, so a caller
//! can hold a clone for a status line without the history coming along. The
//! session is the only owner of mutable state in the REPL, which is what makes
//! [`route`](crate::app::input_router::route) testable: it reads a session and
//! returns a decision without touching it.

use crate::config::ResolvedConfig;
use crate::shell::{ExecutionResult, WorkingDirectory};

/// The state a running session carries between lines.
#[derive(Debug, Clone)]
pub struct SessionState {
    directory: WorkingDirectory,
    history: Vec<ExecutionResult>,
    config: ResolvedConfig,
    requests: usize,
}

impl SessionState {
    /// A session starting in `directory` with `config` resolved.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::session_state::SessionState;
    /// use shx::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    /// use shx::shell::WorkingDirectory;
    ///
    /// let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
    ///     .expect("valid settings");
    /// let state = SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config);
    /// assert_eq!(state.working_directory().short(), "tmp");
    /// assert!(state.history().is_empty());
    /// ```
    #[must_use]
    pub const fn new(directory: WorkingDirectory, config: ResolvedConfig) -> Self {
        Self {
            directory,
            history: Vec::new(),
            config,
            requests: 0,
        }
    }

    /// The directory commands run in.
    #[must_use]
    pub const fn working_directory(&self) -> &WorkingDirectory {
        &self.directory
    }

    /// The directory, mutably, for a `cd`.
    pub const fn working_directory_mut(&mut self) -> &mut WorkingDirectory {
        &mut self.directory
    }

    /// The commands run this session, oldest first.
    #[must_use]
    pub fn history(&self) -> &[ExecutionResult] {
        &self.history
    }

    /// The resolved configuration.
    #[must_use]
    pub const fn config(&self) -> &ResolvedConfig {
        &self.config
    }

    /// The configuration, mutably, for `config set`.
    pub const fn config_mut(&mut self) -> &mut ResolvedConfig {
        &mut self.config
    }

    /// How many requests have been sent to a model this session.
    #[must_use]
    pub const fn request_count(&self) -> usize {
        self.requests
    }

    /// Whether this session has no history.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::session_state::SessionState;
    /// use shx::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    /// use shx::shell::WorkingDirectory;
    ///
    /// let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
    ///     .expect("valid settings");
    /// let state = SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config);
    /// assert!(state.history_is_empty());
    /// ```
    #[must_use]
    pub fn history_is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Add a completed command to the history.
    ///
    /// Only completed commands are recorded. A command that was refused or
    /// cancelled did not happen, and putting it in the history would make
    /// `/history` a list of things the user considered rather than things they
    /// ran.
    pub fn record_command(&mut self, result: ExecutionResult) {
        self.history.push(result);
    }

    /// Count one request sent to a model.
    pub const fn record_request(&mut self) {
        self.requests += 1;
    }

    /// Drop the most recent `count` commands, or all of them when `count` is
    /// `None`.
    ///
    /// Clamped rather than panicking: `/forget 99` on a short session is a
    /// request to forget everything, not a bug.
    pub fn forget(&mut self, count: Option<usize>) {
        let Some(count) = count else {
            self.history.clear();
            return;
        };
        self.history
            .truncate(self.history.len().saturating_sub(count));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::SessionState;
    use crate::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    use crate::shell::{ExecutionResult, WorkingDirectory};

    fn state() -> SessionState {
        let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
            .expect("valid settings");
        SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config)
    }

    fn ran(command: &str, exit_code: i32) -> ExecutionResult {
        ExecutionResult {
            command: command.to_owned(),
            stdout: format!("{command} output"),
            stderr: String::new(),
            exit_code: Some(exit_code),
            duration: Duration::from_millis(7),
        }
    }

    #[test]
    fn a_new_session_is_empty() {
        let state = state();
        assert!(state.history_is_empty());
        assert_eq!(state.request_count(), 0);
        assert_eq!(state.working_directory().display(), "/tmp");
        assert_eq!(state.config().provider_name(), "gemini");
    }

    #[test]
    fn recording_a_command_keeps_it_in_order() {
        let mut state = state();
        state.record_command(ran("ls", 0));
        state.record_command(ran("git status", 0));

        let commands: Vec<&str> = state.history().iter().map(|r| r.command.as_str()).collect();
        assert_eq!(commands, vec!["ls", "git status"]);
        assert!(!state.history_is_empty());
    }

    #[test]
    fn a_recorded_command_keeps_its_output_and_status() {
        let mut state = state();
        state.record_command(ran("ls", 3));

        let recorded = &state.history()[0];
        assert_eq!(recorded.stdout, "ls output");
        assert_eq!(recorded.exit_code, Some(3));
        assert!(
            !recorded.succeeded(),
            "the status is not flattened to a bool"
        );
    }

    #[test]
    fn the_history_grows_one_entry_per_command() {
        let mut state = state();
        for index in 0..5 {
            state.record_command(ran(&format!("cmd{index}"), 0));
        }
        assert_eq!(state.history().len(), 5);
    }

    #[test]
    fn a_clone_does_not_share_the_history() {
        let mut state = state();
        state.record_command(ran("ls", 0));

        let snapshot = state.clone();
        state.record_command(ran("pwd", 0));

        assert_eq!(snapshot.history().len(), 1, "the clone kept its own");
        assert_eq!(state.history().len(), 2);
    }

    #[test]
    fn a_clone_does_not_share_the_working_directory() {
        let mut state = state();
        let snapshot = state.clone();

        state
            .working_directory_mut()
            .change_to("/")
            .expect("the root exists");

        assert_eq!(snapshot.working_directory().display(), "/tmp");
    }

    #[test]
    fn requests_are_counted_independently_of_the_history() {
        let mut state = state();
        state.record_command(ran("ls", 0));
        state.record_request();
        state.record_request();

        assert_eq!(state.request_count(), 2);
        assert_eq!(state.history().len(), 1, "a request is not a command");
    }

    #[test]
    fn forgetting_without_a_count_clears_everything() {
        let mut state = state();
        state.record_command(ran("ls", 0));
        state.record_command(ran("pwd", 0));

        state.forget(None);
        assert!(state.history_is_empty());
    }

    #[test]
    fn forgetting_a_count_drops_from_the_end() {
        let mut state = state();
        state.record_command(ran("ls", 0));
        state.record_command(ran("pwd", 0));
        state.record_command(ran("id", 0));

        state.forget(Some(2));
        let commands: Vec<&str> = state.history().iter().map(|r| r.command.as_str()).collect();
        assert_eq!(commands, vec!["ls"]);
    }

    #[test]
    fn forgetting_more_than_there_is_keeps_what_is_left() {
        let mut state = state();
        state.record_command(ran("ls", 0));

        state.forget(Some(99));
        assert!(state.history_is_empty());
    }

    #[test]
    fn forgetting_on_an_empty_session_is_not_an_error() {
        let mut state = state();
        state.forget(None);
        state.forget(Some(1));
        assert!(state.history_is_empty());
    }

    #[test]
    fn the_config_is_reachable_and_mutable() {
        let mut state = state();
        state
            .config_mut()
            .apply_overrides(Some("ollama"), Some("llama3"), false)
            .expect("valid overrides");

        assert_eq!(state.config().provider_name(), "ollama");
    }
}
