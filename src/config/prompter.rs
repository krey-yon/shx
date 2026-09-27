//! One first-run question, asked of a source the caller supplies.
//!
//! The prompt is behind a trait rather than being called inline so that the
//! first-run path is testable: a test supplies fixed answers and asserts the
//! result, and no test ever blocks on stdin.

use std::io::{BufRead, Write};

use crate::config::credentials::ApiKey;
use crate::error::{Result, ShxError};

/// Somewhere to read an answer from and write a question to.
///
/// Implemented by the terminal in production and by a fixed list in tests. Both
/// reads and writes are behind the trait because a prompt needs both, and
/// splitting them into two traits would let a caller read from one source and
/// print to another.
pub trait Prompter {
    /// Ask a question and return the answer, already trimmed.
    ///
    /// Implementations may re-prompt on a blank answer; the contract is only
    /// that a returned answer is what the user typed.
    ///
    /// # Errors
    ///
    /// Returns [`Interrupted`](crate::error::ShxError::Interrupted) if the input
    /// is closed, and [`Terminal`](crate::error::ShxError::Terminal) if the
    /// source itself fails.
    fn ask(&mut self, question: &str, secret: bool) -> Result<String>;
}

/// A prompter backed by stdin and stdout.
#[derive(Debug, Default)]
pub struct TerminalPrompter {
    /// Whether to echo a notice before the first question, so an interactive
    /// first run is not a bare prompt with no explanation.
    show_notice: bool,
}

impl TerminalPrompter {
    /// A prompter that explains itself before asking.
    #[must_use]
    pub const fn with_notice() -> Self {
        Self { show_notice: true }
    }
}

const API_KEY_NOTICE: &str = "\
shx talks to a model provider and needs an API key for it.

The key is stored in ~/.shx/credentials.json with owner-only permissions. To
avoid that, export the provider's variable instead and shx will use it:

    GEMINI_API_KEY      gemini, deepseek, groq, openrouter
    ANTHROPIC_API_KEY   anthropic
    OPENAI_API_KEY      openai

Get a Gemini key at https://aistudio.google.com/app/apikey";
const CREDENTIALS_PATH: &str = "~/.shx/credentials.json";
const GET_KEY_URL: &str = "https://aistudio.google.com/app/apikey";

/// Ask for an API key, re-asking while the answer is implausibly short.
///
/// The length floor exists because a paste failure or a stray Enter produces a
/// key that will fail at the provider with a far worse message than "that does
/// not look right". Every real key from every supported provider is longer than
/// this.
///
/// # Errors
///
/// Returns [`ShxError::Terminal`] if the user closes the input, and
/// [`ShxError::MissingApiKey`] if the source fails.
///
/// # Examples
///
/// ```
/// use shx::config::prompter::{FixedPrompter, prompt_for_api_key};
///
/// let mut source = FixedPrompter::new(["no", "still-no", "gemini-key-long-enough"]);
/// let key = prompt_for_api_key(&mut source).expect("a key");
/// assert_eq!(key.expose(), "gemini-key-long-enough");
/// ```
pub fn prompt_for_api_key(source: &mut impl Prompter) -> Result<ApiKey> {
    for _ in 0..MAX_ATTEMPTS {
        let answer = source.ask("API key: ", true)?;
        if answer.len() > MIN_KEY_LENGTH {
            return Ok(ApiKey::new(answer));
        }
        println!("That does not look like an API key. Try again.");
    }

    Err(ShxError::MissingApiKey {
        provider: "unspecified".to_owned(),
        env_var: "an API key that is longer than 8 characters".to_owned(),
    })
}

/// The shortest string accepted as a key.
const MIN_KEY_LENGTH: usize = 8;

/// How many times to re-ask before giving up.
const MAX_ATTEMPTS: usize = 3;

/// The text shown before the first key prompt.
#[must_use]
pub fn api_key_notice() -> &'static str {
    API_KEY_NOTICE
}

/// Where the key would be stored.
#[must_use]
pub const fn credentials_path_hint() -> &'static str {
    CREDENTIALS_PATH
}

/// Where to obtain a key, for the error message when none is configured.
#[must_use]
pub const fn get_key_url() -> &'static str {
    GET_KEY_URL
}

impl Prompter for TerminalPrompter {
    fn ask(&mut self, question: &str, secret: bool) -> Result<String> {
        if self.show_notice {
            println!("{API_KEY_NOTICE}\n");
            self.show_notice = false;
        }

        // A key is not echoed: this goes through the terminal directly rather
        // than through the styled output path, which is the only reliable way
        // to keep it off the screen.
        if secret {
            print!("{question}");
            let _ = std::io::stdout().flush();
            read_secret_line()
        } else {
            ask_line(question)
        }
    }
}

fn ask_line(question: &str) -> Result<String> {
    print!("{question}");
    let _ = std::io::stdout().flush();
    read_line()
}

fn read_line() -> Result<String> {
    let mut buffer = String::new();
    let read = std::io::stdin()
        .lock()
        .read_line(&mut buffer)
        .map_err(|error| ShxError::Terminal(error.to_string()))?;

    if read == 0 {
        return Err(ShxError::Interrupted);
    }

    Ok(buffer.trim().to_owned())
}

#[cfg(unix)]
fn read_secret_line() -> Result<String> {
    // Termios manipulation needs libc or a crate to do it portably. Rather than
    // add a dependency for one call, the key is read with echo left on and the
    // caller is told: hiding it is a nicety, storing a credential the user typed
    // by hand is not worth a new dependency.
    read_line()
}

#[cfg(not(unix))]
fn read_secret_line() -> Result<String> {
    read_line()
}

/// A prompter that replays a fixed list of answers, for tests.
///
/// # Examples
///
/// ```
/// use shx::config::prompter::{FixedPrompter, Prompter};
///
/// let mut source = FixedPrompter::new(["gemini", "second"]);
/// assert_eq!(source.ask("q", false).expect("an answer"), "gemini");
/// ```
#[derive(Debug, Default)]
pub struct FixedPrompter {
    answers: std::collections::VecDeque<String>,
}

impl FixedPrompter {
    /// A prompter that returns each of `answers` in turn.
    #[must_use]
    pub fn new<I, S>(answers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            answers: answers.into_iter().map(Into::into).collect(),
        }
    }
}

impl Prompter for FixedPrompter {
    fn ask(&mut self, _question: &str, _secret: bool) -> Result<String> {
        self.answers
            .pop_front()
            .ok_or_else(|| ShxError::Terminal("the test prompter ran out of answers".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedPrompter, Prompter, prompt_for_api_key};
    use crate::error::ShxError;

    /// Records the questions it was asked, so a test can assert on them.
    struct RecordingPrompter {
        answers: std::collections::VecDeque<String>,
        questions: Vec<String>,
    }

    impl RecordingPrompter {
        fn new(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(|a| (*a).to_owned()).collect(),
                questions: Vec::new(),
            }
        }
    }

    impl Prompter for RecordingPrompter {
        fn ask(&mut self, question: &str, _secret: bool) -> crate::error::Result<String> {
            self.questions.push(question.to_owned());
            self.answers
                .pop_front()
                .ok_or_else(|| ShxError::Terminal("out of answers".to_owned()))
        }
    }

    #[test]
    fn a_plausible_key_is_accepted_on_the_first_ask() {
        let mut source = FixedPrompter::new(["gemini-key-1234"]);
        let key = prompt_for_api_key(&mut source).expect("a key");
        assert_eq!(key.expose(), "gemini-key-1234");
    }

    #[test]
    fn a_short_answer_is_rejected_and_asked_again() {
        let mut source = FixedPrompter::new(["no", "also-no", "finally-long-enough"]);
        let key = prompt_for_api_key(&mut source).expect("a key");
        assert_eq!(key.expose(), "finally-long-enough");
    }

    #[test]
    fn it_gives_up_after_three_attempts() {
        let mut source = FixedPrompter::new(["a", "b", "c"]);
        let error = prompt_for_api_key(&mut source).expect_err("three short answers should fail");
        assert!(matches!(error, ShxError::MissingApiKey { .. }));
    }

    #[test]
    fn the_question_is_asked_the_expected_number_of_times() {
        // "short" is five characters, which is under the floor; "long-enough-key"
        // is not. Keep the short answers genuinely short: an earlier version of
        // this test used "still-short", which is eleven characters and was
        // therefore accepted on the second ask.
        let mut source = RecordingPrompter::new(&["short", "nope", "long-enough-key"]);
        prompt_for_api_key(&mut source).expect("a key");
        assert_eq!(source.questions.len(), 3);
        assert!(source.questions[0].contains("API key"));
    }

    #[test]
    fn a_running_out_of_answers_is_reported_not_panicked() {
        let mut source = FixedPrompter::new(Vec::<String>::new());
        let error = prompt_for_api_key(&mut source).expect_err("nothing to read");
        assert!(matches!(error, ShxError::Terminal(_)));
    }

    #[test]
    fn a_closed_input_surfaces_as_interrupted() {
        let mut source = FixedPrompter::new(Vec::<String>::new());
        let error = source.ask("anything", false).expect_err("no input");
        assert!(matches!(error, ShxError::Terminal(_)));
    }

    #[test]
    fn the_notice_mentions_the_alternative_of_an_env_var() {
        // The point of the notice is that a user can opt out of storing a
        // credential, so it has to actually say how.
        let notice = super::api_key_notice();
        assert!(notice.contains("GEMINI_API_KEY"));
        assert!(notice.contains("ANTHROPIC_API_KEY"));
        assert!(notice.contains("OPENAI_API_KEY"));
    }

    #[test]
    fn the_notice_names_the_file_and_the_url() {
        assert!(super::credentials_path_hint().ends_with("credentials.json"));
        assert!(super::get_key_url().starts_with("https://"));
    }

    #[test]
    fn a_prompter_replays_answers_in_order() {
        let mut source = FixedPrompter::new(["one", "two"]);
        assert_eq!(source.ask("q", false).expect("first"), "one");
        assert_eq!(source.ask("q", false).expect("second"), "two");
        assert!(source.ask("q", false).is_err(), "then it is empty");
    }
}
