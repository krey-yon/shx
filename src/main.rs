//! Entry point for the `shx` binary.

use std::io::Write;
use std::process::ExitCode;

use shx::app::cli_args::{CliArgs, Commands, ConfigAction};
use shx::app::repl_loop::{StdinLines, run_loop};
use shx::app::{SessionState, ShxApp};
use shx::config::prompter::TerminalPrompter;
use shx::config::{
    ResolvedConfig, Settings, config_path, credentials_file, load_config, write_config,
};
use shx::error::{Result, ShxError};
use shx::shell::WorkingDirectory;

fn main() -> ExitCode {
    let arguments = match CliArgs::parse_process_args() {
        Ok(arguments) => arguments,
        Err(message) => {
            eprintln!("shx: {message}");
            return ExitCode::FAILURE;
        }
    };

    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shx: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &CliArgs) -> Result<()> {
    if let Some(Commands::Config { action }) = &arguments.command {
        return handle_config(action);
    }

    let state = build_state(arguments)?;
    let mut app = ShxApp::new(state).with_dry_run(arguments.is_dry_run());

    let mut prompter = TerminalPrompter::default();

    if let Some(request) = &arguments.request {
        let mut out = std::io::stdout();
        app.handle_line(request, &mut prompter, &mut out)?;
        let _ = out.flush();
        return Ok(());
    }

    let mut source = StdinLines;
    let mut out = std::io::stdout();
    run_loop(&mut app, &mut source, &mut prompter, &mut out, true)?;
    let _ = out.flush();
    Ok(())
}

fn build_state(arguments: &CliArgs) -> Result<SessionState> {
    let settings = match &arguments.config {
        Some(path) => shx::config::load_config_from(std::path::Path::new(path))?,
        None => load_config()?,
    };

    let mut config = ResolvedConfig::from_parts(settings, credentials_file::load_credentials()?)?;
    config.apply_overrides(
        arguments.provider.as_deref(),
        arguments.model.as_deref(),
        arguments.verbose,
    )?;

    Ok(SessionState::new(WorkingDirectory::from_env()?, config))
}

fn handle_config(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Path => println!("{}", config_path()?.display()),

        ConfigAction::Show => {
            let settings = load_config()?;
            println!("provider: {}", settings.provider);
            println!(
                "model: {}",
                settings.model_name().unwrap_or("(provider default)")
            );
            println!("max_tokens: {}", settings.max_tokens);
            println!("temperature: {}", settings.temperature);
            println!(
                "command_timeout_seconds: {}",
                settings.command_timeout_seconds
            );
            println!("render_markdown: {}", settings.render_markdown);
        }

        ConfigAction::Set { key, value } => {
            let mut settings = load_config()?;
            apply_setting(&mut settings, key, value)?;
            write_config(&settings)?;
            println!("{key} = {value}");
        }

        ConfigAction::Forget { provider } => {
            if credentials_file::forget_api_key(provider)? {
                println!("forgot the {provider} key");
            } else {
                println!("no stored key for {provider}");
            }
        }
    }

    Ok(())
}

fn apply_setting(settings: &mut Settings, key: &str, value: &str) -> Result<()> {
    let invalid = |expectation: &str| ShxError::MissingSetting {
        setting: format!("{key}={value:?} is not {expectation}"),
    };

    match key {
        "provider" => value
            .to_ascii_lowercase()
            .clone_into(&mut settings.provider),
        "model" => value.to_owned().clone_into(&mut settings.model),
        "max_tokens" => {
            settings.max_tokens = value
                .parse()
                .map_err(|_| invalid("a whole number of tokens"))?;
        }
        "temperature" => {
            settings.temperature = value
                .parse()
                .map_err(|_| invalid("a number between 0.0 and 2.0"))?;
        }
        "command_timeout_seconds" => {
            settings.command_timeout_seconds = value
                .parse()
                .map_err(|_| invalid("a whole number of seconds"))?;
        }
        "render_markdown" => {
            settings.render_markdown = parse_bool(value).ok_or_else(|| invalid("true or false"))?;
        }
        "remember_conversation" => {
            settings.remember_conversation =
                parse_bool(value).ok_or_else(|| invalid("true or false"))?;
        }
        "conversation_history_turns" => {
            settings.conversation_history_turns = value
                .parse()
                .map_err(|_| invalid("a whole number of turns"))?;
        }
        other => {
            return Err(ShxError::MissingSetting {
                setting: format!("unknown setting '{other}'; try: shx config show"),
            });
        }
    }

    Ok(())
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
