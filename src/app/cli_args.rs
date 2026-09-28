//! Command-line flags for the `shx` binary.

use clap::{ArgAction, Parser, Subcommand};

/// Open-source, minimal Warp alternative for developers.
#[derive(Debug, Clone, Parser)]
#[command(name = "shx", version, about, long_about = None, subcommand_precedence_over_arg = true)]
pub struct CliArgs {
    /// A provider to use, e.g. gemini, anthropic, openai or ollama.
    #[arg(long, value_name = "NAME", global = true)]
    pub provider: Option<String>,

    /// A model to ask for, overriding the configured one.
    #[arg(long, value_name = "NAME", global = true)]
    pub model: Option<String>,

    /// A config file to use instead of ~/.shx/config.json.
    #[arg(long, value_name = "PATH", global = true)]
    pub config: Option<String>,

    /// Explain what a request would do without running anything.
    #[arg(long, action = ArgAction::SetTrue, global = true)]
    pub dry_run: bool,

    /// Log debug detail to stderr.
    #[arg(long, short, action = ArgAction::SetTrue, global = true)]
    pub verbose: bool,

    /// Inspect or change the configuration.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// One request to run, then exit.
    #[arg(value_name = "REQUEST")]
    pub request: Option<String>,
}

/// The subcommands `shx` accepts.
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Work with the configuration.
    Config {
        /// What to do.
        #[command(subcommand)]
        action: ConfigAction,
    },
}

/// What `shx config` can do.
#[derive(Debug, Clone, Subcommand)]
pub enum ConfigAction {
    /// Print where the config file is.
    Path,
    /// Print the current settings.
    Show,
    /// Change one setting.
    Set {
        /// The setting name, e.g. provider or temperature.
        key: String,
        /// The new value.
        value: String,
    },
    /// Remove a stored API key.
    Forget {
        /// The provider whose key should go.
        provider: String,
    },
}

impl CliArgs {
    /// Parse the process's real arguments.
    ///
    /// # Errors
    ///
    /// Returns a message if the arguments are not valid. `clap` prints its own
    /// help and exits before this can return, so an error here is a
    /// contradictory-arguments case rather than a bad flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use clap::Parser;
    /// use shx::app::cli_args::CliArgs;
    ///
    /// let args = CliArgs::parse_from(["shx", "install ripgrep"]);
    /// assert_eq!(args.request.as_deref(), Some("install ripgrep"));
    /// assert!(!args.is_dry_run());
    /// ```
    pub fn parse_process_args() -> Result<Self, String> {
        Self::try_parse_from(std::env::args_os()).map_err(|error| error.to_string())
    }

    /// Whether commands should be explained rather than run.
    ///
    /// # Examples
    ///
    /// ```
    /// use clap::Parser;
    /// use shx::app::cli_args::CliArgs;
    ///
    /// let args = CliArgs::try_parse_from(["shx", "--dry-run", "ls"]).expect("valid");
    /// assert!(args.is_dry_run());
    /// ```
    #[must_use]
    pub const fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Whether a single request was given on the command line.
    ///
    /// # Examples
    ///
    /// ```
    /// use clap::Parser;
    /// use shx::app::cli_args::CliArgs;
    ///
    /// assert!(CliArgs::try_parse_from(["shx", "ls"]).expect("valid").has_request());
    /// assert!(!CliArgs::try_parse_from(["shx"]).expect("valid").has_request());
    /// ```
    #[must_use]
    pub const fn has_request(&self) -> bool {
        self.request.is_some()
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use clap::error::ErrorKind;

    use super::{CliArgs, Commands, ConfigAction};

    fn parse(args: &[&str]) -> CliArgs {
        let mut full = vec!["shx"];
        full.extend_from_slice(args);
        CliArgs::try_parse_from(full)
            .unwrap_or_else(|error| panic!("{args:?} should parse: {error}"))
    }

    #[test]
    fn no_arguments_is_the_interactive_case() {
        let args = parse(&[]);
        assert!(!args.has_request());
        assert!(args.command.is_none());
        assert_eq!(args.provider, None);
        assert!(!args.is_dry_run());
        assert!(!args.verbose);
    }

    #[test]
    fn a_positional_is_the_one_shot_request() {
        assert_eq!(
            parse(&["install ripgrep"]).request.as_deref(),
            Some("install ripgrep")
        );
    }

    #[test]
    fn every_flag_is_parsed() {
        let args = parse(&[
            "--provider",
            "ollama",
            "--model",
            "llama3",
            "--config",
            "/tmp/x.json",
            "--dry-run",
            "--verbose",
        ]);
        assert_eq!(args.provider.as_deref(), Some("ollama"));
        assert_eq!(args.model.as_deref(), Some("llama3"));
        assert_eq!(args.config.as_deref(), Some("/tmp/x.json"));
        assert!(args.is_dry_run());
        assert!(args.verbose);
    }

    #[test]
    fn verbose_has_a_short_form() {
        assert!(parse(&["-v"]).verbose);
    }

    #[test]
    fn config_path_prints_the_location() {
        let args = parse(&["config", "path"]);
        assert!(matches!(
            args.command,
            Some(Commands::Config {
                action: ConfigAction::Path
            })
        ));
    }

    #[test]
    fn config_show_prints_the_settings() {
        let args = parse(&["config", "show"]);
        assert!(matches!(
            args.command,
            Some(Commands::Config {
                action: ConfigAction::Show
            })
        ));
    }

    #[test]
    fn config_set_takes_a_key_and_a_value() {
        let args = parse(&["config", "set", "temperature", "0.2"]);
        match args.command {
            Some(Commands::Config {
                action: ConfigAction::Set { key, value },
            }) => {
                assert_eq!(key, "temperature");
                assert_eq!(value, "0.2");
            }
            other => panic!("expected Set, got {other:?}"),
        }
    }

    #[test]
    fn config_forget_takes_a_provider() {
        let args = parse(&["config", "forget", "gemini"]);
        match args.command {
            Some(Commands::Config {
                action: ConfigAction::Forget { provider },
            }) => assert_eq!(provider, "gemini"),
            other => panic!("expected Forget, got {other:?}"),
        }
    }

    #[test]
    fn config_set_without_a_value_is_an_error_and_not_an_exit() {
        let error = CliArgs::try_parse_from(["shx", "config", "set", "temperature"])
            .expect_err("a missing value should fail");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn an_unknown_flag_is_an_error_and_not_an_exit() {
        // This is why the tests use try_parse_from: a bad flag must return, not
        // call std::process::exit and take the test runner with it.
        let error = CliArgs::try_parse_from(["shx", "--nope"]).expect_err("unknown flag");
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn a_bare_word_is_a_request_not_an_unknown_subcommand() {
        // Only the four real subcommands are subcommands. Anything else is the
        // one-shot request, so `shx install ripgrep` does what it looks like.
        let args = parse(&["nope"]);
        assert_eq!(args.request.as_deref(), Some("nope"));
        assert!(args.command.is_none());
    }

    #[test]
    fn global_flags_work_before_and_after_the_subcommand() {
        let before = parse(&["--dry-run", "config", "show"]);
        let after = parse(&["config", "show", "--dry-run"]);
        assert!(before.is_dry_run() && after.is_dry_run());
    }

    #[test]
    fn a_subcommand_takes_no_bare_request() {
        // A subcommand is a subcommand: `shx config show` takes no positional,
        // and silently ignoring an extra word would run something the user did
        // not ask for.
        let error = CliArgs::try_parse_from(["shx", "config", "show", "ls"])
            .expect_err("a subcommand takes no positional");
        assert!(error.to_string().contains("ls"));
    }
}
