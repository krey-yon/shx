//! Reading and writing `~/.shx/credentials.json`.
//!
//! Separate from the settings file on purpose. Settings describe how the tool
//! behaves and are the kind of thing you would paste into a bug report;
//! credentials are a secret. Keeping them apart means sharing your config does
//! not share your key, and it means the settings file can be world-readable
//! without thinking about it.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::config_directory;
use crate::error::{Result, ShxError};

/// The file name of the credentials file inside the `shx` directory.
const CREDENTIALS_FILE_NAME: &str = "credentials.json";

/// A flat map of provider name to API key.
///
/// # Examples
///
/// ```
/// use shx::config::credentials_file::StoredApiKeys;
///
/// let mut keys = StoredApiKeys::default();
/// keys.insert("gemini".to_owned(), "key".to_owned());
/// assert!(keys.contains_key("gemini"));
/// ```
pub type StoredApiKeys = HashMap<String, String>;

/// Where the credentials file lives.
///
/// # Examples
///
/// ```
/// use shx::config::credentials_file::credentials_path;
///
/// let path = credentials_path();
/// assert!(path.ends_with("credentials.json"));
/// ```
#[must_use]
pub fn credentials_path() -> PathBuf {
    config_directory().join(CREDENTIALS_FILE_NAME)
}

/// Read the credentials file.
///
/// A missing or unreadable file is an empty map, not an error: the caller
/// resolves keys from the environment first, and a user with no credentials
/// file is a normal state, not a broken one.
///
/// # Errors
///
/// Never in practice. The signature keeps the `Result` so that a future
/// validating reader does not change every caller.
///
/// # Examples
///
/// ```no_run
/// use shx::config::credentials_file::load_credentials;
///
/// let stored = load_credentials().expect("credentials should load");
/// println!("{} providers", stored.len());
/// ```
pub fn load_credentials() -> Result<StoredApiKeys> {
    let path = credentials_path();
    load_credentials_from(&path)
}

/// Read credentials from an explicit path.
///
/// # Errors
///
/// As [`load_credentials`].
pub fn load_credentials_from(path: &Path) -> Result<StoredApiKeys> {
    if !path.exists() {
        return Ok(StoredApiKeys::default());
    }

    let contents = fs::read_to_string(path).map_err(|source| ShxError::io(path, source))?;

    if contents.trim().is_empty() {
        return Ok(StoredApiKeys::default());
    }

    Ok(serde_json::from_str(&contents).unwrap_or_default())
}

/// Write credentials, creating the directory if needed, with owner-only
/// permissions.
///
/// # Errors
///
/// Returns [`Io`](crate::error::ShxError::Io) if the directory cannot be
/// created or the file cannot be written.
///
/// # Examples
///
/// ```no_run
/// use shx::config::credentials_file::{StoredApiKeys, write_credentials};
///
/// let mut keys = StoredApiKeys::default();
/// keys.insert("gemini".to_owned(), "the-key".to_owned());
/// write_credentials(&keys).expect("credentials should save");
/// ```
pub fn write_credentials(keys: &StoredApiKeys) -> Result<()> {
    let path = credentials_path();
    write_credentials_to(keys, &path)
}

/// Write credentials to an explicit path.
///
/// # Errors
///
/// As [`write_credentials`].
pub fn write_credentials_to(keys: &StoredApiKeys, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|source| ShxError::io(parent, source))?;
    }

    let encoded = serde_json::to_string_pretty(keys).map_err(|error| ShxError::ConfigParse {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;

    fs::write(path, encoded).map_err(|source| ShxError::io(path, source))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|source| ShxError::io(path, source))?;
    }

    Ok(())
}

/// Store one provider's key, leaving the others alone.
///
/// # Errors
///
/// As [`write_credentials`], plus [`Io`](crate::error::ShxError::Io) if reading
/// the existing file fails.
///
/// # Examples
///
/// ```no_run
/// use shx::config::credentials_file::store_api_key;
///
/// store_api_key("gemini", "the-key").expect("the key should be stored");
/// ```
pub fn store_api_key(provider: &str, key: &str) -> Result<()> {
    let path = credentials_path();
    let mut keys = load_credentials_from(&path)?;
    keys.insert(provider.to_owned(), key.to_owned());
    write_credentials_to(&keys, &path)
}

/// Remove one provider's key.
///
/// A key that is not there is not an error, since the caller's intent is
/// "this should not exist afterwards".
///
/// # Errors
///
/// As [`write_credentials`].
pub fn forget_api_key(provider: &str) -> Result<bool> {
    let path = credentials_path();
    let mut keys = load_credentials_from(&path)?;
    let removed = keys.remove(provider).is_some();
    if removed {
        write_credentials_to(&keys, &path)?;
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{StoredApiKeys, load_credentials_from, write_credentials_to};
    use crate::config::credentials_file;

    #[test]
    fn a_missing_file_is_an_empty_map() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let keys = load_credentials_from(&directory.path().join("absent.json")).expect("reads");
        assert!(keys.is_empty());
    }

    #[test]
    fn an_empty_file_is_an_empty_map() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("credentials.json");
        fs::write(&path, "").expect("write should succeed");

        assert!(load_credentials_from(&path).expect("reads").is_empty());
    }

    #[test]
    fn a_corrupt_file_is_empty_rather_than_fatal() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("credentials.json");
        fs::write(&path, "{ broken").expect("write should succeed");

        assert!(load_credentials_from(&path).expect("reads").is_empty());
    }

    #[test]
    fn keys_round_trip() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("credentials.json");

        let keys = StoredApiKeys::from([("gemini".to_owned(), "k1".to_owned())]);
        write_credentials_to(&keys, &path).expect("write should succeed");

        assert_eq!(load_credentials_from(&path).expect("reads"), keys);
    }

    #[test]
    fn writing_creates_the_directory() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("nested").join("credentials.json");

        write_credentials_to(&StoredApiKeys::default(), &path).expect("write should succeed");
        assert!(path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn the_file_is_not_world_readable() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("credentials.json");

        write_credentials_to(&StoredApiKeys::default(), &path).expect("write should succeed");

        let mode = fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o077, 0, "expected no group or other access");
    }

    #[test]
    fn the_path_ends_in_credentials_json() {
        assert!(credentials_file::credentials_path().ends_with("credentials.json"));
    }

    #[test]
    fn a_second_provider_does_not_clobber_the_first() {
        let directory = tempfile::tempdir().expect("a temp dir");
        let path = directory.path().join("credentials.json");

        let mut keys = load_credentials_from(&path).expect("reads");
        keys.insert("gemini".to_owned(), "k1".to_owned());
        write_credentials_to(&keys, &path).expect("write should succeed");

        let mut keys = load_credentials_from(&path).expect("reads");
        keys.insert("anthropic".to_owned(), "k2".to_owned());
        write_credentials_to(&keys, &path).expect("write should succeed");

        let reloaded = load_credentials_from(&path).expect("reads");
        assert_eq!(reloaded.get("gemini").map(String::as_str), Some("k1"));
        assert_eq!(reloaded.get("anthropic").map(String::as_str), Some("k2"));
    }
}
