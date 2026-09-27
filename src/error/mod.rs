//! The error type every module returns.
//!
//! See [`shx_error`] for the enum and its semantics.

mod shx_error;

pub use shx_error::ShxError;

/// A `Result` whose error is always [`ShxError`].
pub type Result<T, E = ShxError> = std::result::Result<T, E>;
