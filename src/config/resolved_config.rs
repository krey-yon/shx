//! The resolved configuration a running session operates on.
//!
//! One value, built once at startup, passed by reference everywhere. Functions
//! that need a setting take [`Settings`] rather than reading the environment or
//! the file themselves, which is what makes every other module testable without
//! touching a developer's real config.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::credentials_file::{self, StoredApiKeys};
use crate::config::env_overlay::apply_env_overrides;
use crate::config::{Settings, config_path, load_config};
use crate::error::{Result, ShxError};

/// Everything a session needs, resolved and validated.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// The user's settings after the file and the environment have been applied.
    pub settings: Settings,
    /// Where the settings were read from, for `shx config path`.
    pub settings_path: PathBuf,
    /// API keys by provider, from the environment and the credentials file.
    pub api_keys: StoredApiKeys,
}

impl ResolvedConfig {
    /// Load the config the way the binary does: file, then environment, then
    /// validate.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigParse`](crate::error::ShxError::ConfigParse) if the
    /// config file is malformed, and
    /// [`MissingSetting`](crate::error::ShxError::MissingSetting) if an
    /// environment variable cannot be interpreted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use shx::config::ResolvedConfig;
    ///
    /// let config = ResolvedConfig::load().expect("config should resolve");
    /// println!("provider: {}", config.settings.provider);
    /// ```
    pub fn load() -> Result<Self> {
        let settings = apply_env_overrides(load_config()?, &std::env::vars().collect())?;
        let settings_path = config_path()?;
        let api_keys = collect_api_keys()?;

        let resolved = Self {
            settings,
            settings_path,
            api_keys,
        };
        resolved.validate()?;
        Ok(resolved)
    }

    /// Build a config from explicit values, for tests and for `--config` with
    /// inline overrides.
    ///
    /// # Errors
    ///
    /// As [`load`](Self::load), minus the file access.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::config::{ResolvedConfig, Settings};
    ///
    /// let config = ResolvedConfig::from_parts(
    ///     Settings { provider: "ollama".to_owned(), ..Settings::default() },
    ///     Default::default(),
    /// )
    /// .expect("valid settings");
    /// assert_eq!(config.provider_name(), "ollama");
    /// ```
    pub fn from_parts(settings: Settings, api_keys: StoredApiKeys) -> Result<Self> {
        let resolved = Self {
            settings_path: config_path()?,
            settings,
            api_keys,
        };
        resolved.validate()?;
        Ok(resolved)
    }

    /// The provider name, lowercased and trimmed.
    #[must_use]
    pub fn provider_name(&self) -> String {
        self.settings.provider.trim().to_ascii_lowercase()
    }

    /// The key for a provider, if one is available.
    #[must_use]
    pub fn api_key(&self, provider: &str) -> Option<&str> {
        self.api_keys.get(provider).map(String::as_str)
    }

    /// Apply command-line overrides, which outrank the environment.
    ///
    /// Flags are applied last so that `--model` beats `SHX_MODEL`, which is the
    /// order a user expects when they are typing both.
    ///
    /// # Errors
    ///
    /// As [`load`](Self::load), and
    /// [`UnknownModel`](crate::error::ShxError::UnknownModel) if a model is
    /// requested that no provider is configured for.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::config::{ResolvedConfig, Settings};
    ///
    /// let mut config = ResolvedConfig::from_parts(Settings::default(), Default::default())
    ///     .expect("valid");
    /// config.apply_overrides(Some("anthropic"), Some("claude-sonnet-4-5"), true).expect("ok");
    /// assert_eq!(config.provider_name(), "anthropic");
    /// ```
    pub fn apply_overrides(
        &mut self,
        provider: Option<&str>,
        model: Option<&str>,
        verbose: bool,
    ) -> Result<()> {
        if let Some(provider) = provider {
            self.settings.provider = provider.trim().to_ascii_lowercase();
        }
        if let Some(model) = model {
            model.trim().clone_into(&mut self.settings.model);
        }
        if verbose {
            self.settings.verbose_logging = true;
        }
        self.validate()?;
        Ok(())
    }

    /// Check everything that can be checked without a network call.
    ///
    /// # Errors
    ///
    /// Returns [`MissingSetting`](crate::error::ShxError::MissingSetting) if the
    /// provider is blank, or the temperature is out of range, or a token limit
    /// or timeout is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::config::{ResolvedConfig, Settings};
    ///
    /// let config = ResolvedConfig::from_parts(Settings::default(), Default::default());
    /// assert!(config.is_ok());
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.settings.provider.trim().is_empty() {
            return Err(ShxError::MissingSetting {
                setting: "provider is empty; set SHX_PROVIDER or run 'shx config set provider'"
                    .to_owned(),
            });
        }

        if !self.settings.has_valid_temperature() {
            return Err(ShxError::MissingSetting {
                setting: format!(
                    "temperature {} is outside 0.0 to 2.0",
                    self.settings.temperature
                ),
            });
        }

        if self.settings.max_tokens == 0 {
            return Err(ShxError::MissingSetting {
                setting: "max_tokens is zero, which means no response at all".to_owned(),
            });
        }

        if self.settings.command_timeout_seconds == 0 {
            return Err(ShxError::MissingSetting {
                setting: "command_timeout_seconds is zero, which would kill every command"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

fn collect_api_keys() -> Result<StoredApiKeys> {
    let stored = credentials_file::load_credentials()?;

    let mut keys = stored;
    for (provider, key) in environment_api_keys() {
        // The environment wins: exporting a key is the documented way to opt
        // out of storing one.
        keys.insert(provider, key);
    }
    Ok(keys)
}

/// Provider name to key, for every provider with a conventional variable that
/// is currently set.
fn environment_api_keys() -> HashMap<String, String> {
    let mut found = HashMap::new();

    for variable in crate::config::api_key_variable_names() {
        let Ok(value) = std::env::var(variable) else {
            continue;
        };
        if value.trim().is_empty() {
            continue;
        }

        let Some(provider) = variable.strip_suffix("_API_KEY") else {
            continue;
        };

        found.insert(provider.to_ascii_lowercase(), value.trim().to_owned());
    }

    found
}

#[cfg(test)]
mod tests {
    use super::{ResolvedConfig, StoredApiKeys};
    use crate::config::Settings;
    use crate::error::ShxError;

    fn config(settings: Settings) -> ResolvedConfig {
        ResolvedConfig::from_parts(settings, StoredApiKeys::default()).expect("valid settings")
    }

    #[test]
    fn the_provider_name_is_normalised() {
        let config = config(Settings {
            provider: "  Gemini  ".to_owned(),
            ..Settings::default()
        });
        assert_eq!(config.provider_name(), "gemini");
    }

    #[test]
    fn an_empty_provider_is_rejected() {
        let error = ResolvedConfig::from_parts(
            Settings {
                provider: "   ".to_owned(),
                ..Settings::default()
            },
            StoredApiKeys::default(),
        )
        .expect_err("blank provider");

        assert!(matches!(error, ShxError::MissingSetting { .. }));
    }

    #[test]
    fn an_out_of_range_temperature_is_rejected() {
        let error = ResolvedConfig::from_parts(
            Settings {
                temperature: 3.0,
                ..Settings::default()
            },
            StoredApiKeys::default(),
        )
        .expect_err("temperature too high");

        let message = error.to_string();
        assert!(message.contains("temperature"), "got {message}");
    }

    #[test]
    fn zero_tokens_is_rejected() {
        let error = ResolvedConfig::from_parts(
            Settings {
                max_tokens: 0,
                ..Settings::default()
            },
            StoredApiKeys::default(),
        )
        .expect_err("zero tokens");
        assert!(error.to_string().contains("max_tokens"));
    }

    #[test]
    fn a_zero_timeout_is_rejected() {
        // Zero would mean every command is killed instantly, which looks like a
        // hung terminal rather than a misconfiguration.
        let error = ResolvedConfig::from_parts(
            Settings {
                command_timeout_seconds: 0,
                ..Settings::default()
            },
            StoredApiKeys::default(),
        )
        .expect_err("zero timeout");
        assert!(error.to_string().contains("command_timeout_seconds"));
    }

    #[test]
    fn verbose_turns_logging_on_and_never_off() {
        let mut config = config(Settings::default());
        assert!(!config.settings.verbose_logging, "off by default");

        config.apply_overrides(None, None, true).expect("verbose");
        assert!(config.settings.verbose_logging);

        // There is no --no-verbose: a later invocation that forgets the flag
        // must not turn logging back off for a config file that asked for it.
        config.apply_overrides(None, None, false).expect("no flags");
        assert!(
            config.settings.verbose_logging,
            "a later run without --verbose should not disable it"
        );
    }

    #[test]
    fn flags_beat_the_file() {
        let mut config = config(Settings {
            provider: "gemini".to_owned(),
            model: "from-file".to_owned(),
            ..Settings::default()
        });

        config
            .apply_overrides(Some("Anthropic"), Some("claude-sonnet-4-5"), false)
            .expect("overrides should apply");

        assert_eq!(config.provider_name(), "anthropic");
        assert_eq!(config.settings.model_name(), Some("claude-sonnet-4-5"));
    }

    #[test]
    fn an_override_leaves_absent_flags_alone() {
        let mut config = config(Settings {
            provider: "gemini".to_owned(),
            model: "from-file".to_owned(),
            ..Settings::default()
        });

        config.apply_overrides(None, None, false).expect("no flags");

        assert_eq!(config.provider_name(), "gemini");
        assert_eq!(config.settings.model_name(), Some("from-file"));
    }

    #[test]
    fn an_invalid_override_is_caught_at_the_override_not_later() {
        let mut config = config(Settings::default());
        let error = config
            .apply_overrides(Some("   "), None, false)
            .expect_err("a blank provider flag should be rejected");
        assert!(matches!(error, ShxError::MissingSetting { .. }));
    }

    #[test]
    fn a_stored_key_is_found_by_provider_name() {
        let keys = StoredApiKeys::from([("ollama".to_owned(), "k".to_owned())]);
        let config = ResolvedConfig::from_parts(Settings::default(), keys).expect("valid");

        assert_eq!(config.api_key("ollama"), Some("k"));
        assert_eq!(config.api_key("gemini"), None);
    }

    #[test]
    fn the_default_configuration_validates() {
        let config = config(Settings::default());
        assert!(config.validate().is_ok());
        assert_eq!(config.settings.max_tokens, 8_192);
    }
}
