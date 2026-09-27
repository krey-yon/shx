//! The settings `shx` reads at startup and writes back on `config set`.

use serde::{Deserialize, Serialize};

/// Everything the user can configure.
///
/// Every field falls back to a default, so a config file with one key in it is
/// valid. The API key is deliberately not a field here: it lives in its own
/// file with owner-only permissions, separate from the settings you might sync
/// or paste into a bug report.
///
/// # Examples
///
/// ```
/// use shx::config::Settings;
///
/// let settings = Settings::default();
/// assert_eq!(settings.temperature, 0.7);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// Which model provider to talk to, e.g. `gemini` or `ollama`.
    pub provider: String,

    /// The model to ask for, e.g. `gemini-2.5-flash`. Empty means "use the
    /// provider's own default", which avoids pinning us to model names that go
    /// out of date.
    pub model: String,

    /// Upper bound on tokens in a single response.
    pub max_tokens: u32,

    /// Sampling temperature, from 0.0 (deterministic) to 2.0 (unreliable).
    pub temperature: f32,

    /// How long a single command may run before it is killed.
    pub command_timeout_seconds: u64,

    /// Whether to render model replies as markdown rather than plain text.
    pub render_markdown: bool,

    /// Whether `shx` remembers earlier turns of the conversation and sends them
    /// to the provider.
    pub remember_conversation: bool,

    /// How many past turns to include when [`remember_conversation`] is on.
    ///
    /// [`remember_conversation`]: Settings::remember_conversation
    pub conversation_history_turns: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "gemini".to_owned(),
            model: String::new(),
            max_tokens: 8_192,
            temperature: 0.7,
            command_timeout_seconds: 120,
            render_markdown: true,
            remember_conversation: true,
            conversation_history_turns: 10,
        }
    }
}

impl Settings {
    /// A model name to send to the provider, or `None` to let the provider
    /// choose its own default.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::config::Settings;
    ///
    /// assert!(Settings::default().model_name().is_none());
    /// ```
    #[must_use]
    pub fn model_name(&self) -> Option<&str> {
        if self.model.trim().is_empty() {
            None
        } else {
            Some(self.model.as_str())
        }
    }

    /// Whether the temperature is inside the range providers accept.
    ///
    /// Out-of-range values do not fail loudly at the API boundary, they just get
    /// rejected with an opaque message, so it is worth catching here.
    ///
    /// # Examples
    ///
    /// ```
    /// use shx::config::Settings;
    ///
    /// let settings = Settings { temperature: 3.0, ..Settings::default() };
    /// assert!(!settings.has_valid_temperature());
    /// ```
    #[must_use]
    pub fn has_valid_temperature(&self) -> bool {
        (0.0..=2.0).contains(&self.temperature)
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn empty_model_means_provider_default() {
        let settings = Settings {
            model: String::new(),
            ..Settings::default()
        };
        assert_eq!(settings.model_name(), None);
    }

    #[test]
    fn whitespace_only_model_also_means_provider_default() {
        let settings = Settings {
            model: "   ".to_owned(),
            ..Settings::default()
        };
        assert_eq!(settings.model_name(), None);
    }

    #[test]
    fn explicit_model_is_returned() {
        let settings = Settings {
            model: "gemini-2.5-flash".to_owned(),
            ..Settings::default()
        };
        assert_eq!(settings.model_name(), Some("gemini-2.5-flash"));
    }

    #[test]
    fn temperature_bounds_are_checked() {
        assert!(Settings::default().has_valid_temperature());
        assert!(
            !Settings {
                temperature: 2.5,
                ..Settings::default()
            }
            .has_valid_temperature()
        );
        assert!(
            !Settings {
                temperature: -0.1,
                ..Settings::default()
            }
            .has_valid_temperature()
        );
    }

    #[test]
    fn a_partial_config_file_fills_in_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"provider":"ollama"}"#).expect("partial config should parse");
        assert_eq!(settings.provider, "ollama");
        assert_eq!(settings.temperature, Settings::default().temperature);
    }

    #[test]
    fn an_unknown_key_is_rejected_rather_than_silently_ignored() {
        // A typo in a config file should be loud: silently using the default is
        // how you end up debugging why your model override did nothing.
        let result = serde_json::from_str::<Settings>(r#"{"providerr":"ollama"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn round_trips_through_json() {
        let original = Settings {
            provider: "anthropic".to_owned(),
            model: "claude-sonnet-4-5".to_owned(),
            max_tokens: 4_096,
            temperature: 0.2,
            command_timeout_seconds: 30,
            render_markdown: false,
            remember_conversation: false,
            conversation_history_turns: 3,
        };
        let encoded = serde_json::to_string(&original).expect("settings should encode");
        let decoded: Settings = serde_json::from_str(&encoded).expect("settings should decode");
        assert_eq!(original, decoded);
    }
}
