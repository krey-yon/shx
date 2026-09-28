//! Orchestration: the REPL loop and the flows it drives.
//!
//! Three files do the work. [`input_router`] decides what a line means and
//! executes nothing, [`shx_app`] carries the decision out, and [`repl_loop`]
//! reads lines until the application asks to stop. Everything that touches stdin
//! goes through a [`Prompter`](crate::config::prompter::Prompter) and everything
//! that prints goes through a `Write`, so all three are testable without a
//! terminal, a network or an API key.

pub mod cli_args;
pub mod input_router;
pub mod repl_loop;
pub mod session_state;
pub mod shx_app;
pub mod slash_commands;

pub use cli_args::{CliArgs, Commands, ConfigAction};
pub use input_router::{Routed, route};
pub use repl_loop::{ScriptedLines, StdinLines, banner, run_loop};
pub use session_state::SessionState;
pub use shx_app::{NoPrompter, Outcome, ShxApp};
pub use slash_commands::SlashCommand;
