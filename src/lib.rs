//! `shx` — an open-source, minimal Warp alternative for developers.
//! `shx` is a line-oriented read-eval-print loop that classifies what you type,
//! decides whether it is a shell command or a natural-language request, and for
//! the latter asks a large language model what to do next.
//! The crate is split into modules that each own one concern:
//! - [`config`](mod@config) — settings, their file location, and how environment variables
//!   layer on top of them.
//! - [`platform`](mod@platform) — which operating system and distribution we are on, and what
//!   its package manager commands look like.
//! - `shell` — the catalogue of known commands, the classifier that decides
//!   whether input is a command, and the runner that executes it.
//! - `safety` — risk classification and confirmation policy for commands that
//!   could destroy data.
//! - `placeholder` — detection of unfilled template placeholders in a command.
//! - `response` — the typed shapes an agent can return.
//! - `llm` — provider abstraction, the three specialised agents, and their
//!   prompts.
//! - `tui` — terminal rendering: theme, spinner, markdown, line editing.
//! - `app` — orchestration: the REPL loop and the flows it drives.
//! - `error` — the single error type shared by the library.

#![warn(rustdoc::broken_intra_doc_links)]
// Tests are allowed to panic; library code is not. See the `[lints.clippy]`
// section of the manifest for the reasoning.
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::expect_fun_call)
)]

pub mod config;
pub mod error;
pub mod platform;

/// The version of the `shx` binary, taken from the crate manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
