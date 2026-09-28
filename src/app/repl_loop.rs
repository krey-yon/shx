//! The read-evaluate-print loop, and nothing else.
//!
//! Print the banner, then read a line, hand it to the application, and stop when
//! the application says so. All the decisions live in
//! [`crate::app::input_router`] and all the actions live in
//! [`ShxApp`]; this file owns neither.

use std::io::{BufRead, Write};

use crate::app::shx_app::{Outcome, ShxApp, prompt_for};
use crate::config::prompter::Prompter;
use crate::error::Result;
use crate::tui::theme::Theme;

/// Where the loop reads lines from.
///
/// A trait so that a test can supply a fixed list and the real loop can supply
/// stdin, without either knowing about the other.
pub trait LineSource {
    /// The next line, or `None` at end of input.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the source fails.
    fn next_line(&mut self) -> Result<Option<String>>;
}

/// Lines from standard input.
#[derive(Debug, Default)]
pub struct StdinLines;

impl LineSource for StdinLines {
    fn next_line(&mut self) -> Result<Option<String>> {
        let mut buffer = String::new();
        let read = std::io::stdin()
            .lock()
            .read_line(&mut buffer)
            .map_err(|error| crate::error::ShxError::Terminal(error.to_string()))?;

        Ok((read > 0).then(|| buffer.trim_end().to_owned()))
    }
}

/// A source that replays a fixed list of lines, for tests.
#[derive(Debug, Default)]
pub struct ScriptedLines {
    lines: std::collections::VecDeque<String>,
}

impl ScriptedLines {
    /// A source returning each of `lines` in turn.
    #[must_use]
    pub fn new<I, S>(lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            lines: lines.into_iter().map(Into::into).collect(),
        }
    }
}

impl LineSource for ScriptedLines {
    fn next_line(&mut self) -> Result<Option<String>> {
        Ok(self.lines.pop_front())
    }
}

/// The banner shown when a session starts.
#[must_use]
pub fn banner() -> String {
    let theme = Theme::default();
    let name = theme.paint(theme.success, "shx");
    let hint = theme.paint(
        theme.muted,
        "one prompt for commands and questions — /help for built-ins, exit to quit",
    );
    format!("{name} {}\n{hint}\n", crate::VERSION)
}

/// Run the loop until the input ends or the application asks to stop.
///
/// # Errors
///
/// Returns [`ShxError::Interrupted`](crate::error::ShxError::Interrupted) when
/// the user aborts with Ctrl-C, and
/// [`ShxError::Terminal`](crate::error::ShxError::Terminal) for I/O failures.
///
/// # Examples
///
/// ```
/// use shx::app::repl_loop::{ScriptedLines, run_loop};
/// use shx::app::{Outcome, SessionState, ShxApp};
/// use shx::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
/// use shx::config::prompter::FixedPrompter;
/// use shx::shell::WorkingDirectory;
///
/// let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
///     .expect("valid settings");
/// let state = SessionState::new(WorkingDirectory::new(std::path::Path::new("/tmp")), config);
/// let mut app = ShxApp::new(state);
///
/// let mut input = ScriptedLines::new(["exit"]);
/// let mut out = Vec::new();
/// let outcome = run_loop(&mut app, &mut input, &mut FixedPrompter::new(Vec::<String>::new()), &mut out, false)
///     .expect("the loop should end cleanly");
/// assert_eq!(outcome, Outcome::Quit);
/// ```
pub fn run_loop(
    app: &mut ShxApp,
    source: &mut dyn LineSource,
    prompter: &mut dyn Prompter,
    out: &mut dyn Write,
    show_banner: bool,
) -> Result<Outcome> {
    let theme = Theme::default();

    if show_banner {
        let _ = writeln!(out, "{}", banner());
    }

    loop {
        let _ = write!(
            out,
            "{}",
            theme.paint(theme.prompt, &prompt_for(app.state().working_directory()))
        );
        let _ = out.flush();

        let Some(line) = source.next_line()? else {
            let _ = writeln!(out);
            return Ok(Outcome::Quit);
        };

        match app.handle_line(&line, prompter, out) {
            Ok(Outcome::Quit) => return Ok(Outcome::Quit),
            Ok(Outcome::Continue) => {}
            Err(error) if error.is_recoverable() => {
                let _ = writeln!(out, "{}", theme.paint(theme.error, &error.to_string()));
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Outcome, ScriptedLines, banner, run_loop};
    use crate::app::session_state::SessionState;
    use crate::app::shx_app::ShxApp;
    use crate::config::prompter::FixedPrompter;
    use crate::config::{ResolvedConfig, Settings, credentials_file::StoredApiKeys};
    use crate::shell::WorkingDirectory;

    fn app() -> ShxApp {
        let config = ResolvedConfig::from_parts(Settings::default(), StoredApiKeys::new())
            .expect("valid settings");
        let directory = WorkingDirectory::from_env().expect("a current directory");
        ShxApp::new(SessionState::new(directory, config))
    }

    fn drive(app: &mut ShxApp, lines: &[&str], show_banner: bool) -> String {
        let mut input = ScriptedLines::new(lines.iter().copied());
        let mut out = Vec::new();
        let mut prompter = FixedPrompter::new(Vec::<String>::new());
        run_loop(app, &mut input, &mut prompter, &mut out, show_banner)
            .expect("the loop should end cleanly");
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn end_of_input_ends_the_loop_cleanly() {
        let output = drive(&mut app(), &[], false);
        assert!(output.contains("> "), "got {output:?}");
        assert!(!output.to_lowercase().contains("error"), "got {output:?}");
    }

    #[test]
    fn exit_stops_before_the_rest_is_read() {
        let output = drive(&mut app(), &["exit", "echo unreachable"], false);
        assert!(!output.contains("unreachable"), "got {output:?}");
    }

    #[test]
    fn a_command_runs_and_the_loop_continues() {
        let output = drive(&mut app(), &["echo from-the-loop"], false);
        assert!(output.contains("from-the-loop"), "got {output:?}");
    }

    #[test]
    fn the_prompt_is_shown_before_each_line() {
        let output = drive(&mut app(), &["", ""], false);
        assert!(output.contains("> "), "got {output:?}");
    }

    #[test]
    fn the_banner_appears_when_asked_for_and_not_otherwise() {
        let with = drive(&mut app(), &[], true);
        let without = drive(&mut app(), &[], false);
        assert!(with.contains("one prompt for commands"), "got {with:?}");
        assert!(
            !without.contains("one prompt for commands"),
            "got {without:?}"
        );
    }

    #[test]
    fn the_banner_names_the_version_and_the_help_command() {
        let text = banner();
        assert!(text.contains(crate::VERSION), "got {text:?}");
        assert!(text.contains("/help"), "got {text:?}");
    }

    #[test]
    fn a_recoverable_error_does_not_stop_the_loop() {
        let output = drive(&mut app(), &["cd /nope/nope", "echo still-here"], false);
        assert!(output.contains("still-here"), "got {output:?}");
    }

    #[test]
    fn a_line_that_is_only_whitespace_just_prompts_again() {
        let output = drive(&mut app(), &["   "], false);
        assert!(output.contains("> "), "got {output:?}");
    }

    #[test]
    fn quit_is_the_same_as_exit() {
        assert!(!drive(&mut app(), &["quit", "echo unreachable"], false).contains("unreachable"));
    }

    #[test]
    fn the_loop_returns_quit_when_the_input_ends() {
        let mut application = app();
        let mut input = ScriptedLines::new(["echo x"]);
        let mut out = Vec::new();
        let mut prompter = FixedPrompter::new(Vec::<String>::new());
        let outcome = run_loop(&mut application, &mut input, &mut prompter, &mut out, false)
            .expect("clean end");
        assert_eq!(outcome, Outcome::Quit);
    }

    #[test]
    fn history_survives_across_the_loop() {
        let mut application = app();
        drive(&mut application, &["echo remembered", "/history"], false);
        let output = drive(&mut application, &["/history"], false);
        assert!(output.contains("remembered"), "got {output:?}");
    }
}
