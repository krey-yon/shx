//! The application: what happens once a line has been routed.

use std::io::Write;

use crate::app::input_router::{Routed, route};
use crate::app::session_state::SessionState;
use crate::app::slash_commands::{SlashCommand, help_text};
use crate::config::prompter::Prompter;
use crate::error::{Result, ShxError};
use crate::safety::{analyze_command, confirmation_policy};
use crate::shell::placeholder_scanner;
use crate::shell::{RunRequest, WorkingDirectory, run_command};
use crate::tui::theme::Theme;

/// What the REPL loop should do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Keep reading lines.
    Continue,
    /// Stop.
    Quit,
}

/// The application, holding the state one session accumulates.
///
/// Every path that touches stdin goes through a [`Prompter`] and every path that
/// writes goes through a `Write`, which is what makes the whole thing testable
/// without a terminal and without an API key.
#[derive(Debug, Clone)]
pub struct ShxApp {
    state: SessionState,
    dry_run: bool,
}

impl ShxApp {
    /// An application over an existing session.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::{NoPrompter, Outcome, SessionState, ShxApp};
    /// use shx::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    /// use shx::shell::WorkingDirectory;
    ///
    /// let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
    ///     .expect("valid settings");
    /// let state = SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config);
    /// let mut app = ShxApp::new(state);
    /// assert_eq!(app.handle_line("", &mut NoPrompter, &mut Vec::new()).expect("handled"),
    ///            Outcome::Continue);
    /// ```
    #[must_use]
    pub const fn new(state: SessionState) -> Self {
        Self {
            state,
            dry_run: false,
        }
    }

    /// Explain commands instead of running them.
    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// The session this application is driving.
    #[must_use]
    pub const fn state(&self) -> &SessionState {
        &self.state
    }

    /// Route a line and carry it out.
    ///
    /// # Errors
    ///
    /// Returns [`ShxError::Interrupted`] when the user aborts a prompt. Anything
    /// a command can do is reported to the user rather than returned, because a
    /// failed command is not a failed session.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::app::{NoPrompter, Outcome, SessionState, ShxApp};
    /// use shx::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    /// use shx::shell::WorkingDirectory;
    ///
    /// let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
    ///     .expect("valid settings");
    /// let state = SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config);
    /// let mut app = ShxApp::new(state);
    ///
    /// let mut out = Vec::new();
    /// let outcome = app.handle_line("exit", &mut NoPrompter, &mut out).expect("handled");
    /// assert_eq!(outcome, Outcome::Quit);
    /// ```
    pub fn handle_line(
        &mut self,
        line: &str,
        prompter: &mut dyn Prompter,
        out: &mut dyn Write,
    ) -> Result<Outcome> {
        match route(line, &self.state) {
            Routed::Empty => Ok(Outcome::Continue),
            Routed::SessionExit => Ok(Outcome::Quit),
            Routed::ScreenClear => Ok(Self::screen_clear(out)),
            Routed::ChangeDirectory(target) => Ok(self.change_directory(&target, out)),
            Routed::Slash(command) => Ok(self.run_slash_command(command, out)),
            Routed::RunCommand(command) => self.run_command(&command, prompter, out),
            Routed::AskModel(request) => Ok(self.ask_model(&request, out)),
        }
    }

    fn screen_clear(out: &mut dyn Write) -> Outcome {
        let theme = Theme::default();
        let _ = writeln!(out, "{}", theme.paint(theme.muted, "\x1b[2J\x1b[H"));
        Outcome::Continue
    }

    fn change_directory(&mut self, target: &str, out: &mut dyn Write) -> Outcome {
        match self.state.working_directory_mut().change_to(target) {
            Ok(_) => {
                let theme = Theme::default();
                let _ = writeln!(
                    out,
                    "{}",
                    theme.paint(theme.muted, &self.state.working_directory().display())
                );
            }
            Err(error) => {
                let theme = Theme::default();
                let _ = writeln!(out, "{}", theme.paint(theme.error, &error.to_string()));
            }
        }
        Outcome::Continue
    }

    fn run_command(
        &mut self,
        command: &str,
        prompter: &mut dyn Prompter,
        out: &mut dyn Write,
    ) -> Result<Outcome> {
        let theme = Theme::default();

        if let Some(reason) = placeholder_scanner::placeholder_reason(command) {
            let _ = writeln!(out, "{}", theme.paint(theme.warning, reason.message()));
            let placeholders = placeholder_scanner::find_placeholders(command);
            if !placeholders.is_empty() {
                let _ = writeln!(
                    out,
                    "{}",
                    theme.paint(
                        theme.muted,
                        &format!("Placeholders: {}", placeholders.join(", "))
                    )
                );
            }
            let _ = writeln!(
                out,
                "{}",
                theme.paint(theme.muted, "Edit it in the prompt above before running.")
            );
            return Ok(Outcome::Continue);
        }

        let safety = analyze_command(command);

        if safety.blocked {
            let _ = writeln!(out, "{}", theme.paint(theme.error, &safety.warning_block()));
            return Ok(Outcome::Continue);
        }

        if safety.needs_confirmation() {
            let _ = writeln!(
                out,
                "{}",
                theme.paint(theme.warning, &safety.warning_block())
            );
            let question = confirmation_policy::prompt_for(safety.risk_level);
            let answer = prompter.ask(question, false)?;
            if !confirmation_policy::confirms(&answer, safety.risk_level) {
                let _ = writeln!(out, "{}", theme.paint(theme.muted, "Cancelled."));
                return Ok(Outcome::Continue);
            }
        }

        if self.dry_run {
            let _ = writeln!(
                out,
                "{}",
                theme.paint(theme.muted, &format!("[dry run] would run: {command}"))
            );
            return Ok(Outcome::Continue);
        }

        let request = RunRequest::new(
            command,
            std::time::Duration::from_secs(self.state.config().settings.command_timeout_seconds),
        )
        .in_directory(self.state.working_directory().display());

        match run_command(&request) {
            Ok(result) => {
                let output = result.combined_output();
                if output.is_empty() {
                    let _ = writeln!(out, "{}", theme.paint(theme.muted, "(no output)"));
                } else {
                    let _ = writeln!(out, "{}", theme.paint(theme.command, &output));
                }
                if result.succeeded() {
                    self.state.record_command(result);
                } else {
                    let _ = writeln!(out, "{}", theme.paint(theme.muted, &result.summary()));
                }
            }
            Err(error) => {
                let _ = writeln!(out, "{}", theme.paint(theme.error, &error.to_string()));
            }
        }

        Ok(Outcome::Continue)
    }

    fn ask_model(&mut self, request: &str, out: &mut dyn Write) -> Outcome {
        let theme = Theme::default();
        let config = self.state.config();
        let provider = config.provider_name();

        if config.api_key(&provider).is_none() {
            let _ = writeln!(
                out,
                "{}",
                theme.paint(
                    theme.warning,
                    &format!(
                        "No API key for '{provider}'. Set {} or run: shx config forget {provider}",
                        crate::config::credentials::api_key_variable(&provider)
                            .unwrap_or("the provider's key variable")
                    )
                )
            );
            return Outcome::Continue;
        }

        self.state.record_request();
        let _ = writeln!(
            out,
            "{}",
            theme.paint(theme.muted, &format!("Asking {provider} about: {request}"))
        );
        Outcome::Continue
    }

    fn run_slash_command(&mut self, command: SlashCommand, out: &mut dyn Write) -> Outcome {
        let theme = Theme::default();

        match command {
            SlashCommand::Exit => return Outcome::Quit,
            SlashCommand::Help => {
                let _ = writeln!(out, "{}", theme.paint(theme.muted, &help_text()));
            }
            SlashCommand::Clear => return Self::screen_clear(out),
            SlashCommand::Model(argument) => {
                let settings = &mut self.state.config_mut().settings;
                if let Some(name) = argument {
                    let _ = writeln!(
                        out,
                        "{}",
                        theme.paint(theme.muted, &format!("Model set to {name}"))
                    );
                    settings.model = name;
                } else {
                    let current = settings.model_name().unwrap_or("provider default");
                    let _ = writeln!(
                        out,
                        "{}",
                        theme.paint(theme.muted, &format!("Model: {current}"))
                    );
                }
            }
            SlashCommand::Provider(argument) => {
                let config = self.state.config_mut();
                if let Some(name) = argument {
                    let normalised = name.to_ascii_lowercase();
                    let _ = writeln!(
                        out,
                        "{}",
                        theme.paint(theme.muted, &format!("Provider set to {normalised}"))
                    );
                    config.settings.provider = normalised;
                } else {
                    let current = config.provider_name();
                    let _ = writeln!(
                        out,
                        "{}",
                        theme.paint(theme.muted, &format!("Provider: {current}"))
                    );
                }
            }
            SlashCommand::History => {
                let history = self.state.history();
                if history.is_empty() {
                    let _ = writeln!(
                        out,
                        "{}",
                        theme.paint(theme.muted, "Nothing run yet this session.")
                    );
                } else {
                    for (index, result) in history.iter().enumerate() {
                        let _ = writeln!(
                            out,
                            "{}",
                            theme.paint(
                                theme.muted,
                                &format!("{}. {}", index + 1, result.summary())
                            )
                        );
                    }
                }
            }
            SlashCommand::Forget(count) => {
                self.state.forget(count);
                let _ = writeln!(
                    out,
                    "{}",
                    theme.paint(theme.muted, "Forgot this session's history.")
                );
            }
            SlashCommand::Unknown { name, .. } => {
                let _ = writeln!(
                    out,
                    "{}",
                    theme.paint(theme.error, &format!("No such command: /{name}"))
                );
            }
        }

        Outcome::Continue
    }
}

/// A prompter that answers nothing, for one-shot and dry-run use.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoPrompter;

impl Prompter for NoPrompter {
    fn ask(&mut self, _question: &str, _secret: bool) -> Result<String> {
        Err(ShxError::Terminal("no input available".to_owned()))
    }
}

/// The prompt to show, built from the session's current directory.
#[must_use]
pub fn prompt_for(directory: &WorkingDirectory) -> String {
    format!("{} > ", directory.short())
}

#[cfg(test)]
mod tests {
    use super::{NoPrompter, Outcome, ShxApp, prompt_for};
    use crate::app::session_state::SessionState;
    use crate::config::prompter::FixedPrompter;
    use crate::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    use crate::shell::WorkingDirectory;

    fn app() -> ShxApp {
        let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
            .expect("valid settings");
        let directory = WorkingDirectory::from_env().expect("a current directory");
        ShxApp::new(SessionState::new(directory, config))
    }

    fn run(app: &mut ShxApp, line: &str, answers: &[&str]) -> (Outcome, String) {
        let mut out = Vec::new();
        let mut prompter = FixedPrompter::new(answers.iter().copied());
        let outcome = app
            .handle_line(line, &mut prompter, &mut out)
            .expect("handling should not error");
        (outcome, String::from_utf8_lossy(&out).into_owned())
    }

    #[test]
    fn an_empty_line_keeps_going_and_says_nothing() {
        let (_, output) = run(&mut app(), "   ", &[]);
        assert_eq!(output, "");
    }

    #[test]
    fn exit_and_quit_end_the_session() {
        assert_eq!(run(&mut app(), "exit", &[]).0, Outcome::Quit);
        assert_eq!(run(&mut app(), "quit", &[]).0, Outcome::Quit);
        assert_eq!(run(&mut app(), "/exit", &[]).0, Outcome::Quit);
    }

    #[test]
    fn a_working_command_runs_and_its_output_appears() {
        let (outcome, output) = run(&mut app(), "echo hello-shx", &[]);
        assert_eq!(outcome, Outcome::Continue);
        assert!(output.contains("hello-shx"), "got {output:?}");
    }

    #[test]
    fn a_successful_command_is_recorded_for_history() {
        let mut application = app();
        run(&mut application, "echo recorded", &[]);
        assert_eq!(application.state().history().len(), 1);
    }

    #[test]
    fn a_failing_command_is_reported_and_not_recorded() {
        let mut application = app();
        let (_, output) = run(&mut application, "cat /nope/nope/nope", &[]);
        assert!(
            !output.is_empty(),
            "a failure should say something: {output:?}"
        );
        assert!(
            application.state().history().is_empty(),
            "a failed command should not pollute history"
        );
    }

    #[test]
    fn a_command_with_a_placeholder_is_held_back() {
        let mut application = app();
        let (_, output) = run(&mut application, "brew install <package>", &[]);
        assert!(output.contains("placeholder"), "got {output:?}");
        assert!(
            application.state().history().is_empty(),
            "a held-back command must not run"
        );
    }

    #[test]
    fn a_critical_command_is_refused_even_when_confirmed() {
        let mut application = app();
        let (_, output) = run(&mut application, "rm -rf /", &["YES", "yes", "y"]);
        assert!(
            output.to_lowercase().contains("refus"),
            "expected a refusal, got {output:?}"
        );
        assert!(application.state().history().is_empty());
    }

    #[test]
    fn a_high_risk_command_needs_capitals_exactly() {
        let mut refused = app();
        let (_, output) = run(&mut refused, "rm -rf .", &["yes"]);
        assert!(
            output.contains("Cancelled"),
            "lowercase should be refused: {output:?}"
        );
        assert!(refused.state().history().is_empty());

        let mut allowed = app();
        let (_, output) = run(&mut allowed, "rm -rf .", &["YES"]);
        assert!(!output.contains("Cancelled"), "capitals should be accepted");
    }

    #[test]
    fn a_medium_risk_command_rejects_capitals() {
        let (_, output) = run(&mut app(), "sudo systemctl restart nginx", &["YES"]);
        assert!(
            output.contains("Cancelled"),
            "the medium tier wants lowercase, got {output:?}"
        );
    }

    #[test]
    fn a_low_risk_command_accepts_a_plain_yes() {
        let (_, output) = run(&mut app(), "apt remove ripgrep", &["y"]);
        assert!(!output.contains("Cancelled"), "got {output:?}");
    }

    #[test]
    fn a_safe_command_asks_nothing() {
        let (_, output) = run(&mut app(), "echo no-question", &[]);
        assert!(!output.contains("Type"), "got {output:?}");
    }

    #[test]
    fn dry_run_explains_instead_of_running() {
        let mut application = app().with_dry_run(true);
        let (_, output) = run(&mut application, "echo SHX_DRY_RUN_MARKER", &[]);
        assert!(output.contains("dry run"), "got {output:?}");
        assert!(
            !output
                .lines()
                .any(|line| line.trim() == "SHX_DRY_RUN_MARKER"),
            "the command must not have run: {output:?}"
        );
        assert!(application.state().history().is_empty());
    }

    #[test]
    fn a_model_request_without_a_key_explains_how_to_fix_it() {
        let (_, output) = run(&mut app(), "what is a monad", &[]);
        assert!(output.contains("No API key"), "got {output:?}");
        assert!(
            output.contains("gemini"),
            "should name the provider: {output:?}"
        );
    }

    #[test]
    fn help_lists_the_built_in_commands() {
        let (_, output) = run(&mut app(), "/help", &[]);
        for expected in ["/help", "/model", "/provider", "/history"] {
            assert!(
                output.contains(expected),
                "{expected} missing from {output:?}"
            );
        }
    }

    #[test]
    fn an_unknown_slash_command_says_so() {
        let (_, output) = run(&mut app(), "/nope", &[]);
        assert!(output.contains("nope"), "got {output:?}");
    }

    #[test]
    fn history_is_empty_at_first_and_lists_what_ran() {
        let mut application = app();
        let (_, output) = run(&mut application, "/history", &[]);
        assert!(output.contains("Nothing"), "got {output:?}");

        run(&mut application, "echo listed", &[]);
        let (_, output) = run(&mut application, "/history", &[]);
        assert!(output.contains("listed"), "got {output:?}");
    }

    #[test]
    fn forget_clears_the_session_history() {
        let mut application = app();
        run(&mut application, "echo forgotten", &[]);
        run(&mut application, "/forget", &[]);
        assert!(application.state().history().is_empty());
    }

    #[test]
    fn model_and_provider_report_and_change() {
        let mut application = app();
        let (_, output) = run(&mut application, "/provider", &[]);
        assert!(output.contains("gemini"), "got {output:?}");

        let (_, output) = run(&mut application, "/provider ollama", &[]);
        assert!(output.contains("ollama"), "got {output:?}");
        assert_eq!(application.state().config().provider_name(), "ollama");

        let (_, output) = run(&mut application, "/model llama3", &[]);
        assert!(output.contains("llama3"), "got {output:?}");
    }

    #[test]
    fn cd_to_a_real_directory_updates_the_prompt() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let mut application = app();
        let target = temporary.path().to_string_lossy().into_owned();
        run(&mut application, &format!("cd {target}"), &[]);
        let resolved = std::fs::canonicalize(temporary.path()).expect("canonicalize");
        assert_eq!(application.state().working_directory().path(), resolved);
    }

    #[test]
    fn cd_to_nowhere_reports_and_does_not_move() {
        let mut application = app();
        let before = application.state().working_directory().display();
        let (_, output) = run(&mut application, "cd /nope/nope/nope", &[]);
        assert!(!output.is_empty(), "got {output:?}");
        assert_eq!(application.state().working_directory().display(), before);
    }

    #[test]
    fn clear_keeps_the_session_alive() {
        assert_eq!(run(&mut app(), "clear", &[]).0, Outcome::Continue);
    }

    #[test]
    fn the_prompt_shows_the_directory_name() {
        let directory = WorkingDirectory::new(std::path::Path::new("/home/user/project"));
        assert_eq!(prompt_for(&directory), "project > ");
    }

    #[test]
    fn a_session_with_no_input_available_reports_rather_than_panicking() {
        let mut out = Vec::new();
        let mut application = app();
        let error = application
            .handle_line("sudo ls", &mut NoPrompter, &mut out)
            .expect_err("no prompter should be an error");
        assert!(matches!(error, crate::error::ShxError::Terminal(_)));
    }
}
