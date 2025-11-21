//! Custom error types for the genrex library.

use thiserror::Error;

/// Errors that can occur during regex string generation.
#[derive(Debug, Error)]
pub enum GenrexError {
    #[error("invalid regex pattern: {0}")]
    InvalidRegex(String),

    #[error("no match found within constraints")]
    NoMatch,

    #[error("timeout reached during generation")]
    Timeout,

    #[error("backreference or group error: {0}")]
    BackreferenceError(String),

    #[error("unsupported regex feature: {0}")]
    UnsupportedFeature(String),

    #[error("internal error: {0}")]
    Internal(String),
}
