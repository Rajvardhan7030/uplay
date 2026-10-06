//! Error handling types and error reporting for UPlay.

use std::path::PathBuf;
use thiserror::Error;

/// The primary error type for UPlay operations.
#[derive(Debug, Error)]
pub enum UPlayError {
    /// Invalid command-line input or arguments.
    #[error("Input error: {0}")]
    Input(String),

    /// Errors related to configuration loading or parsing.
    #[error("Configuration error: {0}")]
    Config(String),

    /// File system errors (missing file, permission denied, invalid path).
    #[error("File error: {path}: {reason}")]
    File {
        /// File path that caused the error.
        path: PathBuf,
        /// Description of the failure.
        reason: String,
    },

    /// Network connectivity or streaming errors.
    #[error("Network error: {0}")]
    Network(String),

    /// Source resolution errors (resolving a URL or audio source).
    #[error("Resolver error: {0}")]
    Resolver(String),

    /// Audio backend errors (playback, hardware device, format decode).
    #[error("Audio backend error: {0}")]
    Audio(String),

    /// Inter-process communication errors (daemon, socket communication).
    #[error("IPC error: {0}")]
    Ipc(String),
}

/// A specialized [`std::result::Result`] type for UPlay operations.
pub type Result<T, E = UPlayError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = UPlayError::Input("invalid flag".into());
        assert_eq!(err.to_string(), "Input error: invalid flag");

        let err = UPlayError::File {
            path: PathBuf::from("test.mp3"),
            reason: "file not found".into(),
        };
        assert_eq!(err.to_string(), "File error: test.mp3: file not found");
    }
}
