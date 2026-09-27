//! Reading and writing the settings file.

use std::fs;
use std::path::Path;

use crate::config::{Settings, config_path};
use crate::error::{Result, ShxError};

/// Load the settings, applying defaults for anything the file omits.
///
/// A missing file is not an error: first run, and defaults are the answer.
///
/// # Errors
///
/// Returns [`Io`](crate::error::ShxError::Io) if the file exists but cannot be
/// read, and [`ConfigParse`](crate::error::ShxError::ConfigParse) if it is not
/// valid JSON or has an unexpected key.
///
/// # Examples
///
/// ```no_run
/// use shx::config::load_config;
///
/// let settings = load_config().expect("config should load");
/// println!("{}", settings.provider);
/// ```
pub fn load_config() -> Result<Settings> {
    let path = config_path()?;
    load_config_from(&path)
}

/// Read settings from an explicit path, for tests and for `shx --config`.
///
/// # Errors
///
/// As [`load_config`], except that a missing file at `path` also yields defaults
/// rather than an error.
pub fn load_config_from(path: &Path) -> Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }

    let contents = fs::read_to_string(path).map_err(|source| ShxError::io(path, source))?;

    if contents.trim().is_empty() {
        return Ok(Settings::default());
    }

    serde_json::from_str(&contents).map_err(|error| ShxError::ConfigParse {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

/// Write the settings back, creating the directory if needed.
///
/// The file is written with owner-only permissions because it can contain a
/// credential.
///
/// # Errors
///
/// Returns [`ShxError::Io`] if the directory cannot be created or the file
/// cannot be written.
///
/// # Examples
///
/// ```no_run
/// use shx::config::{load_config, write_config};
///
/// let mut settings = load_config().expect("config should load");
/// settings.temperature = 0.2;
/// write_config(&settings).expect("config should save");
/// ```
pub fn write_config(settings: &Settings) -> Result<()> {
    let path = config_path()?;
    write_config_to(settings, &path)
}

/// Write the settings to an explicit path.
///
/// # Errors
///
/// As [`write_config`], plus [`ShxError::ConfigParse`] if the settings cannot be
/// serialised, which would mean a bug rather than bad input.
pub fn write_config_to(settings: &Settings, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|source| ShxError::io(parent, source))?;
    }

    let encoded =
        serde_json::to_string_pretty(settings).map_err(|error| ShxError::ConfigParse {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;

    fs::write(path, encoded).map_err(|source| ShxError::io(path, source))?;

    set_owner_only_permissions(path)?;

    Ok(())
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|source| ShxError::io(path, source))
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) -> Result<()> {
    // Windows has no POSIX mode bits; the file inherits the user's ACL, which
    // already excludes other accounts. Nothing portable to do here.
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::config::Settings;
    use crate::error::ShxError;

    use super::{load_config_from, write_config_to};

    #[test]
    fn a_missing_file_yields_defaults() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("nope.json");

        let settings = load_config_from(&path).expect("a missing file is not an error");
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn an_empty_file_yields_defaults() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("empty.json");
        fs::write(&path, "   \n").expect("write should succeed");

        let settings = load_config_from(&path).expect("an empty file is not an error");
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn malformed_json_reports_the_path() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("broken.json");
        fs::write(&path, "{ not json").expect("write should succeed");

        let error = load_config_from(&path).expect_err("malformed json should fail");
        match &error {
            ShxError::ConfigParse { path: reported, .. } => {
                assert_eq!(reported, &path, "the error should name the bad file");
            }
            other => panic!("expected ConfigParse, got {other:?}"),
        }
    }

    #[test]
    fn writing_creates_missing_directories() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory
            .path()
            .join("nested")
            .join("deeper")
            .join("config.json");

        let settings = Settings::default();
        write_config_to(&settings, &path).expect("writing should create parents");

        assert!(path.exists(), "the file should have been created");
    }

    #[test]
    fn written_settings_read_back_identically() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("config.json");

        let settings = Settings {
            provider: "ollama".to_owned(),
            temperature: 0.1,
            ..Settings::default()
        };
        write_config_to(&settings, &path).expect("write should succeed");

        let reloaded = load_config_from(&path).expect("read should succeed");
        assert_eq!(settings, reloaded);
    }

    #[test]
    fn partial_files_keep_defaults_for_absent_keys() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("config.json");
        fs::write(&path, r#"{"provider":"ollama"}"#).expect("write should succeed");

        let settings = load_config_from(&path).expect("read should succeed");
        assert_eq!(settings.provider, "ollama");
        assert_eq!(settings.max_tokens, Settings::default().max_tokens);
    }

    #[cfg(unix)]
    #[test]
    fn the_file_is_not_world_readable() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("config.json");

        write_config_to(&Settings::default(), &path).expect("write should succeed");

        let mode = fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o077, 0, "expected no group or other access");
    }

    #[test]
    fn overwriting_replaces_rather_than_appends() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("config.json");

        write_config_to(&Settings::default(), &path).expect("first write");
        let short = Settings {
            provider: "ollama".to_owned(),
            ..Settings::default()
        };
        write_config_to(&short, &path).expect("second write");

        let reloaded = load_config_from(&path).expect("read should succeed");
        assert_eq!(short, reloaded, "the second write should win outright");
    }
}
