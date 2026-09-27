//! Environment variables that override the settings file.
//!
//! Precedence, lowest to highest: defaults, settings file, environment
//! variables, command-line flags. The environment sits above the file so that
//! `SHX_MODEL=... shx` works for a one-off without editing anything, and below
//! flags so an explicit flag still wins.
//!
//! Variable names are `SHX_`-prefixed except the provider API keys, which keep
//! their conventional names (`GEMINI_API_KEY` and friends) so that credentials
//! work the same in `shx` as they do in every other tool for that provider.

use std::collections::HashMap;

use crate::config::Settings;

/// The variables `shx` reads, mapped to the setting each one overrides.
const SETTING_VARIABLES: &[(&str, &str)] = &[
    ("SHX_PROVIDER", "provider"),
    ("SHX_MODEL", "model"),
    ("SHX_MAX_TOKENS", "max_tokens"),
    ("SHX_TEMPERATURE", "temperature"),
    ("SHX_COMMAND_TIMEOUT", "command_timeout_seconds"),
    ("SHX_RENDER_MARKDOWN", "render_markdown"),
    ("SHX_REMEMBER", "remember_conversation"),
    ("SHX_HISTORY_TURNS", "conversation_history_turns"),
];

/// Apply environment variables on top of loaded settings.
///
/// Values that fail to parse are reported rather than ignored: a typo in
/// `SHX_MAX_TOKENS` that is silently discarded is worse than a startup message.
///
/// One asymmetry is intentional. `SHX_MODEL` may be set to an empty string,
/// which means "no model, use the provider's default" and is how a user undoes
/// a model set in the file. Every other variable rejects an empty value, since
/// an empty number would otherwise default to zero meaning "no tokens".
///
/// # Errors
///
/// Returns [`MissingSetting`](crate::error::ShxError::MissingSetting) for a variable
/// that is set but whose
/// value cannot be interpreted, naming both the variable and what it expected.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use shx::config::{Settings, apply_env_overrides};
///
/// let variables = HashMap::from([("SHX_TEMPERATURE".to_owned(), "0.0".to_owned())]);
/// let settings = apply_env_overrides(Settings::default(), &variables).expect("valid input");
/// assert_eq!(settings.temperature, 0.0);
/// ```
pub fn apply_env_overrides(
    mut settings: Settings,
    variables: &HashMap<String, String, std::collections::hash_map::RandomState>,
) -> crate::error::Result<Settings> {
    for (variable, key) in SETTING_VARIABLES {
        let Some(raw) = variables.get(*variable) else {
            continue;
        };
        let raw = raw.trim();

        match *key {
            "provider" => raw.clone_into(&mut settings.provider),
            "model" => raw.clone_into(&mut settings.model),
            "max_tokens" => {
                settings.max_tokens = parse_number(variable, raw)?;
            }
            "temperature" => {
                settings.temperature = parse_number(variable, raw)?;
            }
            "command_timeout_seconds" => {
                settings.command_timeout_seconds = parse_number(variable, raw)?;
            }
            "render_markdown" => settings.render_markdown = parse_bool(variable, raw)?,
            "remember_conversation" => settings.remember_conversation = parse_bool(variable, raw)?,
            "conversation_history_turns" => {
                settings.conversation_history_turns = parse_number(variable, raw)?;
            }
            _ => tracing::warn!(
                setting = key,
                "no environment variable maps to this setting"
            ),
        }
    }

    Ok(settings)
}

/// Read the real process environment and apply it.
///
/// # Errors
///
/// As [`apply_env_overrides`].
pub fn apply_process_env(settings: Settings) -> crate::error::Result<Settings> {
    let variables: HashMap<String, String> = std::env::vars().collect();
    apply_env_overrides(settings, &variables)
}

/// The provider API key environment variables, in the order they are tried.
///
/// A provider that is not in this list (a self-hosted OpenAI-compatible
/// endpoint, say) has no conventional variable, so it must come from the
/// config file.
///
/// # Examples
///
/// ```
/// use shx::config::api_key_variable_names;
///
/// let names = api_key_variable_names();
/// assert!(names.contains(&"GEMINI_API_KEY"));
/// ```
#[must_use]
pub fn api_key_variable_names() -> &'static [&'static str] {
    &[
        "GEMINI_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "DEEPSEEK_API_KEY",
        "GROQ_API_KEY",
        "OPENROUTER_API_KEY",
    ]
}

fn parse_number<T>(variable: &str, raw: &str) -> crate::error::Result<T>
where
    T: std::str::FromStr,
{
    raw.parse::<T>()
        .map_err(|_| crate::error::ShxError::MissingSetting {
            setting: format!("{variable}={raw:?} is not a valid number"),
        })
}

fn parse_bool(variable: &str, raw: &str) -> crate::error::Result<bool> {
    match raw.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(crate::error::ShxError::MissingSetting {
            setting: format!(
                "{variable}={raw:?} is not a boolean; use one of \
                 1/0, true/false, yes/no, on/off"
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::Settings;
    use crate::error::ShxError;

    use super::{api_key_variable_names, apply_env_overrides};

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn an_empty_environment_changes_nothing() {
        let original = Settings::default();
        let result =
            apply_env_overrides(original.clone(), &env(&[])).expect("an empty env is valid");
        assert_eq!(result, original);
    }

    #[test]
    fn the_environment_wins_over_the_file() {
        let from_file = Settings {
            provider: "gemini".to_owned(),
            temperature: 0.9,
            ..Settings::default()
        };
        let result = apply_env_overrides(
            from_file,
            &env(&[("SHX_PROVIDER", "ollama"), ("SHX_TEMPERATURE", "0.1")]),
        )
        .expect("valid overrides");

        assert_eq!(result.provider, "ollama");
        assert!((result.temperature - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn unset_variables_leave_their_setting_alone() {
        let from_file = Settings {
            provider: "anthropic".to_owned(),
            max_tokens: 111,
            ..Settings::default()
        };
        let result = apply_env_overrides(from_file.clone(), &env(&[("SHX_MODEL", "x")]))
            .expect("valid override");

        assert_eq!(result.provider, "anthropic");
        assert_eq!(result.max_tokens, 111);
    }

    #[test]
    fn numbers_are_trimmed() {
        let result = apply_env_overrides(Settings::default(), &env(&[("SHX_MAX_TOKENS", " 512 ")]))
            .expect("whitespace is trimmed");
        assert_eq!(result.max_tokens, 512);
    }

    #[test]
    fn a_bad_number_is_reported_with_the_variable_name() {
        let error = apply_env_overrides(Settings::default(), &env(&[("SHX_MAX_TOKENS", "lots")]))
            .expect_err("non-numeric should fail");
        match error {
            ShxError::MissingSetting { setting } => {
                assert!(setting.contains("SHX_MAX_TOKENS"), "got {setting}");
            }
            other => panic!("expected MissingSetting, got {other:?}"),
        }
    }

    #[test]
    fn booleans_accept_the_usual_spellings() {
        for truthy in ["1", "true", "TRUE", "yes", "on"] {
            let result = apply_env_overrides(
                Settings::default(),
                &env(&[("SHX_RENDER_MARKDOWN", truthy)]),
            )
            .expect("a valid boolean");
            assert!(result.render_markdown, "{truthy} should be true");
        }

        for falsy in ["0", "false", "no", "off"] {
            let result =
                apply_env_overrides(Settings::default(), &env(&[("SHX_RENDER_MARKDOWN", falsy)]))
                    .expect("a valid boolean");
            assert!(!result.render_markdown, "{falsy} should be false");
        }
    }

    #[test]
    fn a_bad_boolean_explains_the_accepted_values() {
        let error =
            apply_env_overrides(Settings::default(), &env(&[("SHX_RENDER_MARKDOWN", "yep")]))
                .expect_err("a non-boolean should fail");
        let message = error.to_string();
        assert!(message.contains("yes/no"), "got {message}");
    }

    #[test]
    fn an_empty_model_deliberately_clears_back_to_the_provider_default() {
        // This one is intentional rather than an oversight: "no model" is a
        // meaningful state, and it is how a user undoes a model set in the file.
        let from_file = Settings {
            model: "gemini-2.5-flash".to_owned(),
            ..Settings::default()
        };
        let result = apply_env_overrides(from_file, &env(&[("SHX_MODEL", "  ")]))
            .expect("clearing is legal");
        assert_eq!(result.model_name(), None);
    }

    #[test]
    fn an_empty_number_is_an_error_not_a_silent_zero() {
        // The opposite of the model case: an empty numeric value is always a
        // mistake, and defaulting it to 0 would mean "no tokens" or "no timeout".
        for variable in [
            "SHX_MAX_TOKENS",
            "SHX_TEMPERATURE",
            "SHX_COMMAND_TIMEOUT",
            "SHX_HISTORY_TURNS",
        ] {
            let error = apply_env_overrides(Settings::default(), &env(&[(variable, "  ")]))
                .expect_err("an empty number should fail");
            assert!(
                matches!(error, ShxError::MissingSetting { .. }),
                "{variable} should be rejected"
            );
        }
    }

    #[test]
    fn api_key_variables_cover_the_documented_providers() {
        let names = api_key_variable_names();
        assert!(names.contains(&"GEMINI_API_KEY"));
        assert!(names.contains(&"ANTHROPIC_API_KEY"));
        assert!(names.contains(&"OPENAI_API_KEY"));
    }
}
