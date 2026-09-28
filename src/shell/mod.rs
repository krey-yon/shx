//! Deciding whether a line of input is a shell command.
//!
//! This is the highest-leverage judgement in `shx`. Get it wrong in one
//! direction and `ls` gets sent to a model; get it wrong in the other and
//! "explain this command" gets executed. The heuristic below is deliberately
//! biased towards treating input as a command, because a mistaken command is
//! shown to the user and needs confirmation, while a mistaken question wastes a
//! model call and shows up as an oddly specific shell command.

pub mod command_catalogue;
pub mod command_classifier;
pub mod command_runner;
pub mod command_splitter;
pub mod execution_result;
pub mod metacharacter_detector;
pub mod path_hint_classifier;
pub mod placeholder_scanner;
pub mod working_directory;

pub use command_catalogue::{
    COMMANDS, Category, category_names, command_category, is_known_command,
};
pub use command_classifier::{
    InputKind, classify_input, first_word, is_directory_change, is_screen_clear, is_session_exit,
};
pub use command_runner::{RunRequest, run_command};
pub use execution_result::ExecutionResult;
pub use metacharacter_detector::contains_metacharacter;
pub use path_hint_classifier::{looks_like_path, looks_like_script};
pub use placeholder_scanner::has_placeholder;
pub use working_directory::{WorkingDirectory, cd_target};
