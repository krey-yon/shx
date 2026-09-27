//! Tracking the current working directory across `cd`.
//!
//! `shx` is not a shell, so nothing else changes the directory. When the user
//! types `cd`, this value is updated and every later command runs there. Kept as
//! one small type so that "where am I" has exactly one answer.

use std::path::{Path, PathBuf};

use crate::error::{Result, ShxError};

/// The directory commands run in, and what the prompt shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingDirectory {
    current: PathBuf,
}

impl WorkingDirectory {
    /// Start from a directory, canonicalised so that the prompt shows a real
    /// path rather than whatever symlinks the user came in through.
    ///
    /// # Errors
    ///
    /// Returns [`Io`](crate::error::ShxError::Io) if the starting directory
    /// does not exist or cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::shell::working_directory::WorkingDirectory;
    ///
    /// let directory = WorkingDirectory::from_env().expect("there is a current directory");
    /// println!("{}", directory.display());
    /// ```
    pub fn from_env() -> Result<Self> {
        let starting = std::env::current_dir().map_err(|source| ShxError::io(".", source))?;
        Ok(Self::new(&starting))
    }

    /// Start from an explicit path.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::working_directory::WorkingDirectory;
    ///
    /// let directory = WorkingDirectory::new(std::path::Path::new("/tmp"));
    /// assert_eq!(directory.display(), "/tmp");
    /// ```
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            current: path.to_path_buf(),
        }
    }

    /// The directory as a path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.current
    }

    /// The directory as a string, for handing to a child process.
    #[must_use]
    pub fn display(&self) -> String {
        self.current.to_string_lossy().into_owned()
    }

    /// The last component, for a compact prompt.
    ///
    /// Falls back to the whole path at the filesystem root, where there is no
    /// last component worth showing.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::shell::working_directory::WorkingDirectory;
    ///
    /// let directory = WorkingDirectory::new(std::path::Path::new("/home/user/project"));
    /// assert_eq!(directory.short(), "project");
    /// ```
    #[must_use]
    pub fn short(&self) -> String {
        self.current.file_name().map_or_else(
            || self.display(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    /// Move to a new directory, given as the user typed it.
    ///
    /// Handles `~`, absolute paths and paths relative to the current one, and
    /// rejects anything that is not a directory. A path is resolved and
    /// canonicalised before being stored, so `cd ..` twice from `/a/b` is
    /// `/` and not `/a/b/../..`.
    ///
    /// # Errors
    ///
    /// Returns [`Io`](crate::error::ShxError::Io) if the target does not exist
    /// or is not a directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::shell::working_directory::WorkingDirectory;
    ///
    /// let mut directory = WorkingDirectory::from_env().expect("a current directory");
    /// let before = directory.display();
    /// directory.change_to("..").expect(".. exists");
    /// assert_ne!(directory.display(), before);
    /// ```
    pub fn change_to(&mut self, target: &str) -> Result<&Path> {
        let trimmed = target.trim();

        let requested = if trimmed.is_empty() {
            dirs::home_dir()
        } else if let Some(rest) = trimmed.strip_prefix('~') {
            dirs::home_dir().map(|home| {
                if rest.is_empty() {
                    home
                } else {
                    home.join(rest.trim_start_matches(['/', '\\']))
                }
            })
        } else {
            let path = Path::new(trimmed);
            Some(if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.current.join(path)
            })
        };

        let Some(requested) = requested else {
            return Err(ShxError::io(trimmed, no_home_directory()));
        };

        let resolved =
            std::fs::canonicalize(&requested).map_err(|source| ShxError::io(&requested, source))?;

        if !resolved.is_dir() {
            return Err(ShxError::io(
                &resolved,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "not a directory".to_owned(),
                ),
            ));
        }

        self.current = resolved;
        Ok(&self.current)
    }
}

fn no_home_directory() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::NotFound, "no home directory")
}

/// The target of a `cd` line: the argument, or home when there is none.
///
/// Split out from [`WorkingDirectory::change_to`] so the parsing can be tested
/// without a filesystem, and so the REPL can show what it is about to do before
/// it does it.
///
/// # Examples
///
/// ```
/// use shx::shell::working_directory::cd_target;
///
/// assert_eq!(cd_target("cd src").as_deref(), Some("src"));
/// assert_eq!(cd_target("cd /tmp").as_deref(), Some("/tmp"));
/// assert_eq!(cd_target("cd").as_deref(), None, "no argument means home");
/// assert_eq!(cd_target("cd a b").as_deref(), Some("a"), "extra words are ignored");
/// ```
#[must_use]
pub fn cd_target(line: &str) -> Option<String> {
    let mut words = line.split_whitespace();
    if words.next()? != "cd" {
        return None;
    }
    words.next().map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{WorkingDirectory, cd_target};
    use crate::error::ShxError;

    #[test]
    fn starting_from_the_environment_works() {
        let directory = WorkingDirectory::from_env().expect("there is a current directory");
        assert!(directory.path().is_absolute());
    }

    #[test]
    fn the_short_name_is_the_last_component() {
        let directory = WorkingDirectory::new(std::path::Path::new("/home/user/project"));
        assert_eq!(directory.short(), "project");
    }

    #[test]
    fn the_root_falls_back_to_the_whole_path() {
        let directory = WorkingDirectory::new(std::path::Path::new("/"));
        assert_eq!(directory.short(), "/");
    }

    #[test]
    fn moving_to_a_relative_path_works() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let inner = temporary.path().join("inner");
        std::fs::create_dir(&inner).expect("mkdir should succeed");

        let mut directory = WorkingDirectory::new(temporary.path());
        directory.change_to("inner").expect("inner exists");

        let resolved = std::fs::canonicalize(&inner).expect("canonicalize");
        assert_eq!(directory.path(), resolved);
    }

    #[test]
    fn moving_to_the_parent_resolves_rather_than_appending() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let deep = temporary.path().join("a").join("b");
        std::fs::create_dir_all(&deep).expect("mkdir should succeed");

        let mut directory = WorkingDirectory::new(&deep);
        directory.change_to("..").expect("the parent exists");
        directory.change_to("..").expect("the grandparent exists");

        let resolved = std::fs::canonicalize(temporary.path()).expect("canonicalize");
        assert_eq!(
            directory.path(),
            resolved,
            "the path should be resolved, not accumulated"
        );
    }

    #[test]
    fn an_absolute_path_ignores_the_current_one() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let other = tempfile::tempdir().expect("another temp dir");

        let mut directory = WorkingDirectory::new(temporary.path());
        directory
            .change_to(&other.path().to_string_lossy())
            .expect("the target exists");

        let resolved = std::fs::canonicalize(other.path()).expect("canonicalize");
        assert_eq!(directory.path(), resolved);
    }

    #[test]
    fn a_missing_directory_is_an_error_naming_it() {
        let mut directory = WorkingDirectory::new(std::path::Path::new("/tmp"));
        let error = directory
            .change_to("definitely-not-a-real-dir-xyz")
            .expect_err("should not exist");

        match &error {
            ShxError::Io { path, .. } => {
                assert!(
                    path.to_string_lossy()
                        .contains("definitely-not-a-real-dir-xyz")
                );
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn a_file_is_not_a_directory() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let file = temporary.path().join("a-file");
        std::fs::write(&file, "x").expect("write should succeed");

        let mut directory = WorkingDirectory::new(temporary.path());
        let error = directory
            .change_to(&file.to_string_lossy())
            .expect_err("a file is not a directory");

        assert!(matches!(error, ShxError::Io { .. }));
    }

    #[test]
    fn the_current_directory_is_unchanged_after_a_failed_change() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let mut directory = WorkingDirectory::new(temporary.path());
        let before = directory.display();

        let _ = directory.change_to("definitely-not-a-real-dir-xyz");
        assert_eq!(directory.display(), before, "a failed cd must not move us");
    }

    #[test]
    fn whitespace_around_the_target_is_ignored() {
        let temporary = tempfile::tempdir().expect("a temp dir");
        let mut directory = WorkingDirectory::new(temporary.path());
        directory
            .change_to("   ")
            .expect("an empty target goes home");

        let home =
            std::fs::canonicalize(dirs::home_dir().expect("a home dir")).expect("canonicalize");
        assert_eq!(directory.path(), home);
    }

    #[test]
    fn cd_target_extracts_the_argument() {
        assert_eq!(cd_target("cd src").as_deref(), Some("src"));
        assert_eq!(cd_target("  cd   /tmp  ").as_deref(), Some("/tmp"));
    }

    #[test]
    fn cd_target_returns_none_for_no_argument_and_for_other_commands() {
        assert_eq!(cd_target("cd"), None);
        assert_eq!(cd_target("cd   "), None);
        assert_eq!(cd_target("ls"), None);
        assert_eq!(cd_target("cdrom"), None);
    }

    #[test]
    fn cd_target_ignores_extra_words() {
        assert_eq!(cd_target("cd a b").as_deref(), Some("a"));
    }
}
