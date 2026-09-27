//! Where the config file lives.

use std::path::PathBuf;

use crate::error::{Result, ShxError};

/// The file name used for both the settings file and the credentials file's
/// sibling directory.
const DIRECTORY_NAME: &str = ".shx";
const SETTINGS_FILE_NAME: &str = "config.json";
const ENV_OVERRIDE: &str = "SHX_CONFIG";

/// The path the settings are read from and written to.
///
/// Resolution order, first match wins:
///
/// 1. `$SHX_CONFIG`, if set. Useful for tests and for running two profiles side
///    by side without touching your real config.
/// 2. `~/.shx/config.json`.
/// 3. A path under the system temporary directory, if the home directory cannot
///    be determined. Losing settings is better than refusing to start.
///
/// # Errors
///
/// Returns [`MissingSetting`](crate::error::ShxError::MissingSetting) if
/// `$SHX_CONFIG` is set to an empty value.
///
/// # Examples
///
/// ```
/// use shx::config::config_path;
///
/// // Does not require the file to exist.
/// let path = config_path().expect("a home directory or a temporary fallback");
/// assert!(path.ends_with("config.json"));
/// ```
pub fn config_path() -> Result<PathBuf> {
    if let Some(raw) = std::env::var_os(ENV_OVERRIDE) {
        if raw.is_empty() {
            return Err(ShxError::MissingSetting {
                setting: format!("{ENV_OVERRIDE} is set but empty"),
            });
        }
        return Ok(PathBuf::from(raw));
    }

    Ok(default_directory().join(SETTINGS_FILE_NAME))
}

/// The directory holding `shx` state, always parented on the config file.
///
/// Exposed because the history file and the credential file live beside it.
///
/// Falls back to the temporary directory when there is no home directory, so it
/// never fails.
///
/// # Examples
///
/// ```
/// use shx::config::config_directory;
///
/// let directory = config_directory();
/// assert!(directory.ends_with(".shx"));
/// ```
#[must_use]
pub fn config_directory() -> PathBuf {
    default_directory()
}

/// The directory name `shx` stores its state in, without the leading dot.
#[must_use]
pub const fn directory_name() -> &'static str {
    DIRECTORY_NAME
}

/// Losing settings is better than refusing to start, so a missing home
/// directory falls back to the temporary directory rather than erroring. The
/// `Result` on the public function exists for the `SHX_CONFIG` validation in
/// [`config_path`].
fn default_directory() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home.join(DIRECTORY_NAME),
        None => std::env::temp_dir().join(DIRECTORY_NAME),
    }
}

#[cfg(test)]
mod tests {
    use super::{config_path, directory_name};

    #[test]
    fn directory_name_is_dot_shx() {
        assert_eq!(directory_name(), ".shx");
    }

    #[test]
    fn path_ends_in_config_json() {
        let path = config_path().expect("a config path should resolve");
        assert!(
            path.ends_with("config.json"),
            "expected a config.json, got {}",
            path.display()
        );
    }
}
