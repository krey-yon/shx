//! Parsing of `/etc/os-release`.
//!
//! The format is `KEY=VALUE`, one per line, where a value may be quoted and a
//! few shell expansions are legal. This parses the subset that distributions
//! actually emit: comments, blank lines, `KEY=VALUE`, optional single or double
//! quotes around the value, and an escaped quote inside a quoted value.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, ShxError};

/// The parsed contents of an `os-release` file.
///
/// # Examples
///
/// ```
/// use shx::platform::os_release::OsRelease;
///
/// let contents = "ID=ubuntu\nNAME=\"Ubuntu\"\nVERSION_ID=\"22.04\"\n";
/// let parsed = OsRelease::parse(contents).expect("valid os-release");
/// assert_eq!(parsed.get("id"), Some("ubuntu"));
/// assert_eq!(parsed.get("name"), Some("Ubuntu"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OsRelease {
    entries: HashMap<String, String>,
}

impl OsRelease {
    /// Parse the file's contents.
    ///
    /// Keys are lowercased so callers do not have to remember that `ID` and
    /// `id` are the same key.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedPlatform`](crate::error::ShxError::UnsupportedPlatform)
    /// if a line is neither blank, a comment, nor a `KEY=VALUE` pair. A file we
    /// cannot parse is a file we cannot reason about, and silently skipping the
    /// line would mean guessing at the distribution later.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::os_release::OsRelease;
    ///
    /// let parsed = OsRelease::parse("ID=debian\n").expect("valid");
    /// assert_eq!(parsed.get("ID"), Some("debian"));
    /// ```
    pub fn parse(contents: &str) -> Result<Self> {
        let mut entries = HashMap::new();

        for (index, line) in contents.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                return Err(ShxError::UnsupportedPlatform(format!(
                    "/etc/os-release line {} is not KEY=VALUE: {line:?}",
                    index + 1
                )));
            };

            entries.insert(key.trim().to_ascii_lowercase(), unquote(value.trim()));
        }

        Ok(Self { entries })
    }

    /// Read and parse `/etc/os-release`.
    ///
    /// # Errors
    ///
    /// Returns [`Io`](crate::error::ShxError::Io) if the file is missing or
    /// unreadable, and [`UnsupportedPlatform`](crate::error::ShxError::UnsupportedPlatform)
    /// if it cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::platform::os_release::OsRelease;
    ///
    /// if let Ok(release) = OsRelease::read_system() {
    ///     println!("id = {:?}", release.get("id"));
    /// }
    /// ```
    pub fn read_system() -> Result<Self> {
        Self::read(Path::new(OS_RELEASE_PATH))
    }

    /// Read and parse a specific `os-release` path.
    ///
    /// # Errors
    ///
    /// As [`read_system`](Self::read_system).
    pub fn read(path: &std::path::Path) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).map_err(|source| ShxError::io(path, source))?;
        Self::parse(&contents)
    }

    /// The value for a key, matched case-insensitively.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::os_release::OsRelease;
    ///
    /// let parsed = OsRelease::parse("ID=arch\n").expect("valid");
    /// assert_eq!(parsed.get("ID"), Some("arch"));
    /// assert_eq!(parsed.get("missing"), None);
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .get(&key.to_ascii_lowercase())
            .map(String::as_str)
    }

    /// The distribution identifier, which is what we branch on.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::platform::os_release::OsRelease;
    ///
    /// let parsed = OsRelease::parse("ID=ubuntu\nNAME=Ubuntu\n").expect("valid");
    /// assert_eq!(parsed.id(), Some("ubuntu"));
    /// ```
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.get("id")
    }

    /// The human-readable distribution name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.get("name")
    }

    /// Every key present, for diagnostics.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

/// Where every Linux distribution puts its `os-release`.
const OS_RELEASE_PATH: &str = "/etc/os-release";

/// Strips one layer of matching quotes and resolves the escapes inside.
fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        return value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }
    if bytes.len() >= 2 && bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'' {
        return value[1..value.len() - 1].to_owned();
    }
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::OsRelease;
    use crate::error::ShxError;

    #[test]
    fn a_typical_file_parses() {
        let contents = "\
PRETTY_NAME=\"Ubuntu 22.04.3 LTS\"
NAME=\"Ubuntu\"
VERSION_ID=\"22.04\"
ID=ubuntu
ID_LIKE=debian
";
        let parsed = OsRelease::parse(contents).expect("should parse");

        assert_eq!(parsed.id(), Some("ubuntu"));
        assert_eq!(parsed.name(), Some("Ubuntu"));
        assert_eq!(parsed.get("version_id"), Some("22.04"));
        assert_eq!(parsed.get("id_like"), Some("debian"));
    }

    #[test]
    fn keys_are_matched_case_insensitively() {
        let parsed = OsRelease::parse("ID=arch\n").expect("should parse");
        for key in ["id", "ID", "Id", "iD"] {
            assert_eq!(parsed.get(key), Some("arch"), "failed for {key}");
        }
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let contents = "\n# a comment\n\nID=fedora\n   # indented comment\nNAME=Fedora\n";
        let parsed = OsRelease::parse(contents).expect("should parse");

        assert_eq!(parsed.id(), Some("fedora"));
        assert_eq!(parsed.get("name"), Some("Fedora"));
    }

    #[test]
    fn unquoted_values_keep_their_content() {
        let parsed = OsRelease::parse("ID=gentoo\n").expect("should parse");
        assert_eq!(parsed.id(), Some("gentoo"));
    }

    #[test]
    fn single_quotes_are_stripped() {
        let parsed = OsRelease::parse("ID='sles'\n").expect("should parse");
        assert_eq!(parsed.id(), Some("sles"));
    }

    #[test]
    fn escaped_quotes_inside_a_value_survive() {
        let parsed =
            OsRelease::parse("NAME=\"Fedora \\\"Workstation\\\"\"\n").expect("should parse");
        assert_eq!(parsed.name(), Some("Fedora \"Workstation\""));
    }

    #[test]
    fn values_containing_equals_signs_are_not_split() {
        let parsed = OsRelease::parse("VARIANT_ID=key=value\n").expect("should parse");
        assert_eq!(parsed.get("variant_id"), Some("key=value"));
    }

    #[test]
    fn whitespace_around_the_separator_is_tolerated() {
        let parsed = OsRelease::parse("  ID  =  arch  \n").expect("should parse");
        assert_eq!(parsed.id(), Some("arch"));
    }

    #[test]
    fn a_line_without_an_equals_sign_is_an_error() {
        let error = OsRelease::parse("ID=arch\nthis is not a pair\n").expect_err("should fail");
        match error {
            ShxError::UnsupportedPlatform(message) => {
                assert!(message.contains("line 2"), "got {message}");
            }
            other => panic!("expected UnsupportedPlatform, got {other:?}"),
        }
    }

    #[test]
    fn an_empty_value_is_kept_as_empty_not_dropped() {
        // Some distributions emit an empty ID. Keeping it means the caller gets
        // an honest "unsupported distribution with no id" instead of a confusing
        // "no such key".
        let parsed = OsRelease::parse("ID=\nNAME=Something\n").expect("should parse");
        assert_eq!(parsed.id(), Some(""));
    }

    #[test]
    fn an_empty_file_parses_to_nothing() {
        let parsed = OsRelease::parse("").expect("should parse");
        assert_eq!(parsed.id(), None);
        assert_eq!(parsed.keys().count(), 0);
    }

    #[test]
    fn reading_a_missing_file_reports_the_path() {
        let error = OsRelease::read(Path::new("/nonexistent/os-release")).expect_err("should fail");
        assert!(matches!(error, ShxError::Io { .. }));
    }

    #[test]
    fn keys_are_iterable_for_diagnostics() {
        let parsed = OsRelease::parse("ID=arch\nNAME=Arch\n").expect("should parse");
        let keys: Vec<&str> = parsed.keys().collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"id"));
    }
}
