//! How wide the terminal is, for wrapping and for the banner.
//!
//! The environment wins over a syscall, because a test or a pipe can set
//! `COLUMNS` and a terminal ioctl cannot. Anything the variable cannot answer
//! — unset, empty, zero, nonsense — falls back to 80 columns, the width every
//! terminal has been since 1971 and the one every manual page assumes.
//!
//! Reading one variable is the whole reason this is a file rather than a
//! dependency.

use std::ffi::OsStr;

/// The variable a shell exports to tell a program how wide the window is.
pub const COLUMNS_VARIABLE: &str = "COLUMNS";

/// The width assumed when nothing better is known.
pub const DEFAULT_WIDTH: usize = 80;

/// The terminal width in columns, at least [`DEFAULT_WIDTH`].
///
/// # Examples
///
/// ```
/// use shx::tui::terminal_width::{DEFAULT_WIDTH, terminal_width};
///
/// assert!(terminal_width() >= DEFAULT_WIDTH);
/// ```
#[must_use]
pub fn terminal_width() -> usize {
    width_from(std::env::var_os(COLUMNS_VARIABLE).as_deref())
}

fn width_from(columns: Option<&OsStr>) -> usize {
    columns
        .and_then(|raw| raw.to_str())
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|width| *width > 0)
        .unwrap_or(DEFAULT_WIDTH)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::{COLUMNS_VARIABLE, DEFAULT_WIDTH, terminal_width, width_from};

    fn columns(raw: &str) -> usize {
        width_from(Some(OsStr::new(raw)))
    }

    #[test]
    fn the_environment_variable_is_respected() {
        assert_eq!(columns("120"), 120);
        assert_eq!(columns("1"), 1);
    }

    #[test]
    fn an_unset_variable_falls_back() {
        assert_eq!(width_from(None), DEFAULT_WIDTH);
    }

    #[test]
    fn zero_is_not_a_width() {
        assert_eq!(columns("0"), DEFAULT_WIDTH);
        assert_eq!(columns("00"), DEFAULT_WIDTH);
    }

    #[test]
    fn nonsense_falls_back() {
        for raw in ["", "wide", "-1", "12.5", "1_000", "80px", "🙂"] {
            assert_eq!(columns(raw), DEFAULT_WIDTH, "{raw:?} is not a width");
        }
    }

    #[test]
    fn a_very_large_width_is_taken_at_face_value() {
        assert_eq!(columns("100000"), 100_000);
    }

    #[test]
    fn the_public_function_agrees_with_the_environment() {
        let from_env = std::env::var_os(COLUMNS_VARIABLE);
        assert_eq!(terminal_width(), width_from(from_env.as_deref()));
    }

    #[test]
    fn the_public_function_never_returns_zero() {
        assert!(terminal_width() > 0);
    }
}
