//! Where a provider's API key comes from, and how it reaches the client.
//!
//! Kept separate from [`crate::config`] because the answer is a security
//! question, not a settings question: a key must never be logged, never written
//! to the settings file, and never included in an error message.

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::fmt;

use crate::config::api_key_variable_names;
use crate::error::{Result, ShxError};

/// A snapshot of the process environment, as `std::env::vars` gives it.
///
/// Taken by reference rather than read from the environment inside the
/// resolution functions, so that resolution is testable without mutating global
/// process state, which would make tests order-dependent.
pub type EnvironmentVariables = HashMap<String, String, RandomState>;

/// Provider name to API key, as read from the credentials file.
pub type StoredApiKeys = HashMap<String, String>;

/// The key for a provider, resolved from the environment or the credentials
/// file.
///
/// Wraps the raw `String` so that a key cannot be printed by accident: the
/// `Display` impl shows only enough to tell two keys apart.
///
/// # Examples
///
/// ```
/// use shx::config::credentials::ApiKey;
///
/// let key = ApiKey::new("sk-ant-0123456789abcdef".to_owned());
/// // Debug and Display both redact; the key itself never reaches a log.
/// assert!(!format!("{key}").contains("0123456789"));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Wrap a key so it is treated as a secret from here on.
    #[must_use]
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// Borrow the key for handing to an HTTP client.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and get the key back.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Whether the key is empty or only whitespace, which the client would then
    /// reject with a much less helpful message.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ApiKey(<redacted, {} chars>)", self.0.len())
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "<redacted>")
    }
}

/// Where a resolved key came from, for the `--verbose` log and for error
/// messages that need to tell the user where to look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// An environment variable such as `GEMINI_API_KEY`.
    Environment(&'static str),
    /// The credentials file next to the config file.
    CredentialsFile,
    /// Typed in at the prompt during this run.
    Prompted,
}

impl fmt::Display for KeySource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment(name) => write!(formatter, "{name}"),
            Self::CredentialsFile => formatter.write_str("the credentials file"),
            Self::Prompted => formatter.write_str("this session"),
        }
    }
}

/// A provider's key together with where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedApiKey {
    /// The key itself.
    pub key: ApiKey,
    /// Where it was found.
    pub source: KeySource,
}

/// Which environment variable holds a given provider's key.
///
/// Returns `None` for providers with no conventional variable, such as a
/// self-hosted OpenAI-compatible endpoint, which must come from the credentials
/// file instead.
///
/// # Examples
///
/// ```
/// use shx::config::credentials::api_key_variable;
///
/// assert_eq!(api_key_variable("gemini"), Some("GEMINI_API_KEY"));
/// assert_eq!(api_key_variable("my-local-llm"), None);
/// ```
#[must_use]
pub fn api_key_variable(provider: &str) -> Option<&'static str> {
    let upper = provider.trim().to_ascii_uppercase().replace('-', "_");
    api_key_variable_names()
        .iter()
        .copied()
        .find(|name| name.strip_suffix("_API_KEY") == Some(upper.as_str()))
}

/// Find a provider's key in the environment.
///
/// The whole conventional variable list is searched, not just the one matching
/// the provider, so a user who exported `ANTHROPIC_API_KEY` and then switched to
/// `--provider anthropic` by a slightly different spelling still works.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use shx::config::credentials::find_key_in_env;
///
/// let env = HashMap::from([("ANTHROPIC_API_KEY".to_owned(), "sk-test".to_owned())]);
/// let found = find_key_in_env("anthropic", &env);
/// assert!(found.is_some());
/// ```
#[must_use]
pub fn find_key_in_env(provider: &str, variables: &EnvironmentVariables) -> Option<ResolvedApiKey> {
    if let Some(name) = api_key_variable(provider)
        && let Some(value) = variables.get(name)
        && !value.trim().is_empty()
    {
        return Some(ResolvedApiKey {
            key: ApiKey::new(value.trim().to_owned()),
            source: KeySource::Environment(name),
        });
    }

    api_key_variable_names().iter().find_map(|name| {
        let value = variables.get(*name)?;
        (!value.trim().is_empty()).then(|| ResolvedApiKey {
            key: ApiKey::new(value.trim().to_owned()),
            source: KeySource::Environment(name),
        })
    })
}

/// Resolve a provider's key, or explain precisely what to do about it.
///
/// The message names the variable to set and the command to run, because "missing
/// API key" on its own is the least helpful error in a terminal tool.
///
/// # Errors
///
/// Returns [`ShxError::MissingApiKey`] if no key is found.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use shx::config::credentials::resolve_api_key;
///
/// let env = HashMap::new();
/// let error = resolve_api_key("gemini", &env, &HashMap::new())
///     .expect_err("no key anywhere");
/// assert!(error.to_string().contains("GEMINI_API_KEY"));
/// ```
pub fn resolve_api_key(
    provider: &str,
    variables: &EnvironmentVariables,
    stored: &StoredApiKeys,
) -> Result<ResolvedApiKey> {
    if let Some(found) = find_key_in_env(provider, variables) {
        return Ok(found);
    }

    if let Some(value) = stored.get(provider)
        && !value.trim().is_empty()
    {
        return Ok(ResolvedApiKey {
            key: ApiKey::new(value.trim().to_owned()),
            source: KeySource::CredentialsFile,
        });
    }

    Err(ShxError::MissingApiKey {
        provider: provider.to_owned(),
        env_var: api_key_variable(provider)
            .unwrap_or("SHX_PROVIDER-specific key in ~/.shx/credentials.json")
            .to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::credentials::{
        ApiKey, EnvironmentVariables, KeySource, StoredApiKeys, find_key_in_env, resolve_api_key,
    };

    fn env(pairs: &[(&str, &str)]) -> EnvironmentVariables {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn display_never_leaks_the_key() {
        let key = ApiKey::new("sk-ant-secret-value".to_owned());
        assert_eq!(key.to_string(), "<redacted>");
        assert!(!key.to_string().contains("secret"));
    }

    #[test]
    fn debug_never_leaks_the_key() {
        let key = ApiKey::new("sk-ant-secret-value".to_owned());
        let rendered = format!("{key:?}");
        assert!(!rendered.contains("secret"), "got {rendered}");
        assert!(rendered.contains("redacted"));
    }

    #[test]
    fn expose_returns_the_original() {
        let key = ApiKey::new("abc".to_owned());
        assert_eq!(key.expose(), "abc");
    }

    #[test]
    fn a_blank_key_is_recognised_before_the_client_rejects_it() {
        assert!(ApiKey::new(String::new()).is_blank());
        assert!(ApiKey::new("   ".to_owned()).is_blank());
        assert!(!ApiKey::new("k".to_owned()).is_blank());
    }

    #[test]
    fn provider_names_normalise() {
        use crate::config::credentials::api_key_variable;
        assert_eq!(api_key_variable("gemini"), Some("GEMINI_API_KEY"));
        assert_eq!(api_key_variable("Gemini"), Some("GEMINI_API_KEY"));
        assert_eq!(api_key_variable("openai"), Some("OPENAI_API_KEY"));
    }

    #[test]
    fn an_unmapped_provider_has_no_conventional_variable() {
        use crate::config::credentials::api_key_variable;
        assert_eq!(api_key_variable("llamacpp"), None);
    }

    #[test]
    fn the_matching_variable_wins() {
        let variables = env(&[
            ("GEMINI_API_KEY", "gemini-key"),
            ("OPENAI_API_KEY", "openai-key"),
        ]);
        let found = find_key_in_env("gemini", &variables).expect("gemini has a key");
        assert_eq!(found.key.expose(), "gemini-key");
        assert_eq!(found.source, KeySource::Environment("GEMINI_API_KEY"));
    }

    #[test]
    fn a_blank_variable_is_treated_as_absent() {
        let variables = env(&[("GEMINI_API_KEY", "   ")]);
        let found = find_key_in_env("gemini", &variables);
        assert!(found.is_none(), "a blank key is not a key");
    }

    #[test]
    fn the_key_is_trimmed() {
        let variables = env(&[("GEMINI_API_KEY", "  spaced-key  ")]);
        let found = find_key_in_env("gemini", &variables).expect("a key");
        assert_eq!(found.key.expose(), "spaced-key");
    }

    #[test]
    fn the_file_is_consulted_only_after_the_environment() {
        let stored = StoredApiKeys::from([("gemini".to_owned(), "from-file".to_owned())]);

        let from_env = resolve_api_key("gemini", &env(&[("GEMINI_API_KEY", "from-env")]), &stored)
            .expect("a key");
        assert_eq!(from_env.key.expose(), "from-env");

        let from_file = resolve_api_key("gemini", &HashMap::new(), &stored).expect("a key");
        assert_eq!(from_file.key.expose(), "from-file");
        assert_eq!(from_file.source, KeySource::CredentialsFile);
    }

    #[test]
    fn the_error_names_the_variable_to_set() {
        let error =
            resolve_api_key("anthropic", &HashMap::new(), &HashMap::new()).expect_err("no key");
        let message = error.to_string();
        assert!(message.contains("ANTHROPIC_API_KEY"), "got {message}");
        assert!(message.contains("anthropic"), "got {message}");
    }

    #[test]
    fn the_error_is_not_recoverable_so_startup_bails_out_early() {
        use crate::error::ShxError;
        let error =
            resolve_api_key("gemini", &HashMap::new(), &HashMap::new()).expect_err("no key");
        assert!(!error.is_recoverable());
        assert!(matches!(error, ShxError::MissingApiKey { .. }));
    }

    #[test]
    fn a_provider_with_no_conventional_variable_still_reports_where_to_put_the_key() {
        let error =
            resolve_api_key("llamacpp", &HashMap::new(), &HashMap::new()).expect_err("no key");
        let message = error.to_string();
        assert!(message.contains("credentials.json"), "got {message}");
    }

    #[test]
    fn key_source_renders_readably() {
        assert_eq!(
            KeySource::Environment("GEMINI_API_KEY").to_string(),
            "GEMINI_API_KEY"
        );
        assert_eq!(
            KeySource::CredentialsFile.to_string(),
            "the credentials file"
        );
    }
}
