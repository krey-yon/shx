//! The single error type shared by every module in the crate.
//!
//! `shx` is an application, not a library that others depend on, so a single
//! crate-level error enum beats a tree of per-module errors that all get wrapped
//! into `anyhow::Error` at the boundary anyway. Modules that need to report a
//! failure the caller can branch on get a named variant; everything else is
//! carried as context on [`ShxError::Io`] or [`ShxError::Unexpected`].
//!
//! Display output is written for a person reading a terminal, not for a log
//! scraper — `tracing` events carry the structured detail.

use std::path::PathBuf;

use thiserror::Error;

/// Anything that can go wrong inside `shx`.
///
/// The variants are grouped by which subsystem raised them. Use
/// [`ShxError::is_recoverable`] to decide whether to keep the REPL running or
/// bail out entirely.
#[derive(Debug, Error)]
pub enum ShxError {
    /// A file could not be read or written.
    #[error("could not access {path}: {source}")]
    Io {
        /// The path involved.
        path: PathBuf,
        /// The underlying operating system error.
        #[source]
        source: std::io::Error,
    },

    /// The config file exists but is not valid JSON, or does not match the
    /// schema.
    #[error("could not parse {path}: {reason}")]
    ConfigParse {
        /// The config file that failed to parse.
        path: PathBuf,
        /// A description of what was wrong with it. Not an [`std::error::Error`],
        /// because it is ours rather than a library's.
        reason: String,
    },

    /// No credentials were found for the selected provider, and the user did not
    /// supply one interactively.
    #[error("no API key for provider '{provider}'; set {env_var} or run 'shx config set api_key'")]
    MissingApiKey {
        /// The provider that has no credential.
        provider: String,
        /// The environment variable that would supply it.
        env_var: String,
    },

    /// The config file needs a value that was not supplied and cannot be
    /// defaulted.
    #[error("missing required setting '{setting}'")]
    MissingSetting {
        /// The setting that has no value.
        setting: String,
    },

    /// This machine is running an operating system `shx` does not know how to
    /// generate commands for.
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// A Linux distribution was recognised but is not one of the families we
    /// have package-manager rules for.
    #[error("unsupported distribution '{distro}' (id '{id}')")]
    UnsupportedDistribution {
        /// The `ID` field from `/etc/os-release`.
        id: String,
        /// The `NAME` field from `/etc/os-release`, for a friendlier message.
        distro: String,
    },

    /// The child process could not be started at all, as opposed to starting and
    /// then failing.
    #[error("could not start '{command}': {source}")]
    CommandSpawn {
        /// The command that was attempted.
        command: String,
        /// The underlying operating system error.
        #[source]
        source: std::io::Error,
    },

    /// A command ran for longer than the configured timeout and was killed.
    #[error("'{command}' timed out after {seconds}s")]
    CommandTimeout {
        /// The command that was killed.
        command: String,
        /// How long it was allowed to run.
        seconds: u64,
    },

    /// The model provider could not be reached, or returned an error.
    #[error("provider '{provider}' failed: {message}")]
    Provider {
        /// The provider that failed.
        provider: String,
        /// The error the provider reported.
        message: String,
    },

    /// The provider replied, but not with anything we can turn into a response.
    ///
    /// This is the common failure mode when a model wraps its answer in prose
    /// or markdown instead of the requested JSON.
    #[error("could not read a response from the model: {0}")]
    UnreadableResponse(String),

    /// The provider has no model configured, or the name it was given is not one
    /// it knows.
    #[error("unknown model '{0}'")]
    UnknownModel(String),

    /// The terminal could not be put into the state we need.
    #[error("terminal error: {0}")]
    Terminal(String),

    /// The user interrupted the REPL with Ctrl-C or Ctrl-D. Not a failure.
    #[error("interrupted")]
    Interrupted,

    /// A bug. Anything that should not be reachable, caught at a boundary.
    #[error("unexpected: {0}")]
    Unexpected(String),
}

impl ShxError {
    /// Whether the REPL can sensibly continue after this error.
    ///
    /// A failed command, a bad config file or a provider outage are all
    /// recoverable: report and prompt again. A missing key or an unsupported
    /// platform is not, because every subsequent request would fail the same
    /// way.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::error::ShxError;
    ///
    /// assert!(ShxError::Interrupted.is_recoverable());
    /// assert!(!ShxError::MissingApiKey {
    ///     provider: "gemini".into(),
    ///     env_var: "GEMINI_API_KEY".into(),
    /// }
    /// .is_recoverable());
    /// ```
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        !matches!(
            self,
            Self::MissingApiKey { .. } | Self::UnsupportedPlatform(_) | Self::UnknownModel(_)
        )
    }

    /// Wrap an [`std::io::Error`] together with the path it happened on.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::error::ShxError;
    ///
    /// let source = std::io::Error::new(std::io::ErrorKind::NotFound, "boom");
    /// let error = ShxError::io(std::path::Path::new("/tmp/x"), source);
    /// assert!(error.to_string().contains("/tmp/x"));
    /// ```
    #[must_use]
    pub fn io(path: impl AsRef<std::path::Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    /// Whether this error came from the filesystem rather than from a model or
    /// the platform.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::error::ShxError;
    ///
    /// let error = ShxError::io("/tmp/x", std::io::Error::other("nope"));
    /// assert!(error.is_io());
    /// ```
    #[must_use]
    pub const fn is_io(&self) -> bool {
        matches!(self, Self::Io { .. } | Self::ConfigParse { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::ShxError;

    #[test]
    fn missing_api_key_is_not_recoverable() {
        let error = ShxError::MissingApiKey {
            provider: "gemini".to_owned(),
            env_var: "GEMINI_API_KEY".to_owned(),
        };
        assert!(!error.is_recoverable());
    }

    #[test]
    fn provider_failure_is_recoverable() {
        let error = ShxError::Provider {
            provider: "anthropic".to_owned(),
            message: "503".to_owned(),
        };
        assert!(error.is_recoverable());
    }

    #[test]
    fn unsupported_platform_is_not_recoverable() {
        assert!(!ShxError::UnsupportedPlatform("plan9".to_owned()).is_recoverable());
    }

    #[test]
    fn interrupt_is_recoverable_so_the_repl_keeps_going() {
        assert!(ShxError::Interrupted.is_recoverable());
    }

    #[test]
    fn config_parse_counts_as_io() {
        let error = ShxError::ConfigParse {
            path: "/tmp/config.json".into(),
            reason: "missing field `model`".to_owned(),
        };
        assert!(error.is_io());
        assert!(error.to_string().contains("missing field `model`"));
    }

    #[test]
    fn io_error_keeps_the_path_in_its_message() {
        let source = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let error = ShxError::io("/etc/shadow", source);
        assert!(error.is_io());
        assert!(error.to_string().contains("/etc/shadow"));
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn command_timeout_names_the_command() {
        let error = ShxError::CommandTimeout {
            command: "cargo build".to_owned(),
            seconds: 30,
        };
        assert!(error.to_string().contains("cargo build"));
        assert!(error.to_string().contains("30"));
    }

    #[test]
    fn unknown_model_is_not_recoverable() {
        assert!(!ShxError::UnknownModel("gpt-9".to_owned()).is_recoverable());
    }
}
