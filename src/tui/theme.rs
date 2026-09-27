//! The colours everything the terminal draws uses.
//!
//! Colour is opt-out, and the decision lives in exactly one place: a
//! [`Theme`] decides once whether this process may write escape codes, and
//! every other module paints through it. A pipe, a redirected file, or
//! `NO_COLOR` in the environment turns a style into plain text rather than
//! into a second code path.

use std::ffi::OsStr;
use std::io::IsTerminal;

use owo_colors::{OwoColorize, Style};

/// The variable a user sets to ask for plain text, per no-color.org.
pub const NO_COLOR_VARIABLE: &str = "NO_COLOR";

/// The `>` in the prompt.
pub const PROMPT: Style = Style::new().bright_cyan();

/// The working directory in the prompt.
pub const DIRECTORY: Style = Style::new().blue();

/// A shell command, and code quoted from a model answer.
pub const COMMAND: Style = Style::new().cyan();

/// Prose from a model answer, left in the terminal's own colour so that it
/// does not fight whatever background the user has chosen.
pub const EXPLANATION: Style = Style::new();

/// Something the user should read before typing yes.
pub const WARNING: Style = Style::new().yellow();

/// A failure.
pub const ERROR: Style = Style::new().bright_red().bold();

/// A command that worked.
pub const SUCCESS: Style = Style::new().green();

/// Context rather than content: hints, timings, counts.
pub const MUTED: Style = Style::new().bright_black();

/// Whether this process may write colour to standard output.
#[must_use]
pub fn should_colour() -> bool {
    let no_color = std::env::var_os(NO_COLOR_VARIABLE);
    colour_allowed(no_color.as_deref(), std::io::stdout().is_terminal())
}

fn colour_allowed(no_color: Option<&OsStr>, is_terminal: bool) -> bool {
    no_color.is_none_or(OsStr::is_empty) && is_terminal
}

/// The palette one session draws with.
///
/// # Examples
///
/// ```
/// use shx::tui::theme::{Theme, WARNING};
///
/// let plain = Theme::new(false);
/// assert_eq!(plain.paint(WARNING, "careful"), "careful");
///
/// let coloured = Theme::new(true);
/// assert!(coloured.paint(WARNING, "careful").contains('\u{1b}'));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// The `>` in the prompt.
    pub prompt: Style,
    /// The working directory in the prompt.
    pub directory: Style,
    /// A shell command.
    pub command: Style,
    /// Prose from a model answer.
    pub explanation: Style,
    /// Something the user should read before typing yes.
    pub warning: Style,
    /// A failure.
    pub error: Style,
    /// A command that worked.
    pub success: Style,
    /// Context rather than content.
    pub muted: Style,
    colour: bool,
}

impl Theme {
    /// The standard palette, with styling either on or off.
    ///
    /// `colour` is a parameter rather than a lookup so that tests, pipes, and
    /// the `--no-colour` path can all be explicit about what they expect.
    #[must_use]
    pub const fn new(colour: bool) -> Self {
        Self {
            prompt: PROMPT,
            directory: DIRECTORY,
            command: COMMAND,
            explanation: EXPLANATION,
            warning: WARNING,
            error: ERROR,
            success: SUCCESS,
            muted: MUTED,
            colour,
        }
    }

    /// The palette for this process, with styling off when
    /// [`should_colour`] says so.
    #[must_use]
    pub fn detect() -> Self {
        Self::new(should_colour())
    }

    /// Whether this theme emits escape codes.
    #[must_use]
    pub const fn colour(&self) -> bool {
        self.colour
    }

    /// `text` in `style`, or `text` unchanged when this theme has no colour.
    #[must_use]
    pub fn paint(&self, style: Style, text: &str) -> String {
        if self.colour {
            format!("{}", text.style(style))
        } else {
            text.to_owned()
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::{
        COMMAND, ERROR, EXPLANATION, MUTED, PROMPT, SUCCESS, Theme, WARNING, colour_allowed,
        should_colour,
    };

    const ESCAPE: char = '\u{1b}';

    #[test]
    fn no_color_switches_styling_off() {
        for value in ["1", "0", "yes", "true"] {
            assert!(
                !colour_allowed(Some(OsStr::new(value)), true),
                "NO_COLOR={value} should switch colour off"
            );
        }
    }

    #[test]
    fn colour_needs_a_terminal() {
        assert!(!colour_allowed(None, false));
        assert!(colour_allowed(None, true));
    }

    #[test]
    fn an_empty_no_color_does_not_suppress_colour() {
        assert!(colour_allowed(Some(OsStr::new("")), true));
    }

    #[test]
    fn a_painted_style_carries_escape_codes() {
        let theme = Theme::new(true);
        for style in [PROMPT, COMMAND, WARNING, ERROR, SUCCESS, MUTED] {
            let painted = theme.paint(style, "text");
            assert!(painted.contains(ESCAPE), "{painted:?} should be styled");
            assert!(painted.contains("text"), "{painted:?} should keep its text");
        }
    }

    #[test]
    fn no_color_paints_text_untouched() {
        let theme = Theme::new(false);
        for style in [PROMPT, COMMAND, WARNING, ERROR, SUCCESS, MUTED] {
            assert_eq!(theme.paint(style, "text"), "text");
        }
        assert!(!theme.colour());
    }

    #[test]
    fn a_plain_style_adds_nothing_even_with_colour_on() {
        assert_eq!(Theme::new(true).paint(EXPLANATION, "prose"), "prose");
    }

    #[test]
    fn paint_agrees_with_should_colour() {
        let painted = Theme::detect().paint(ERROR, "boom");
        if should_colour() {
            assert!(painted.contains(ESCAPE));
        } else {
            assert_eq!(painted, "boom");
        }
    }

    #[test]
    fn the_palette_covers_every_role_the_prompt_draws() {
        let theme = Theme::new(true);
        assert_eq!(theme.prompt, PROMPT);
        assert_eq!(theme.command, COMMAND);
        assert_eq!(theme.warning, WARNING);
        assert_eq!(theme.error, ERROR);
        assert_eq!(theme.success, SUCCESS);
        assert_eq!(theme.muted, MUTED);
    }

    #[test]
    fn the_default_palette_is_the_detected_one() {
        assert_eq!(Theme::default().colour(), should_colour());
    }

    #[test]
    fn a_painted_field_writes_the_same_bytes_as_the_constant() {
        let theme = Theme::new(true);
        assert_eq!(theme.paint(theme.error, "x"), theme.paint(ERROR, "x"));
    }
}
