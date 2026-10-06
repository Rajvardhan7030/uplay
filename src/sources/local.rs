//! Local filesystem media resolver.

use crate::core::state::{MediaItem, Source};
use crate::error::{Result, UPlayError};
use std::path::{Path, PathBuf};

/// Supported audio file extensions.
pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "opus", "aac"];

/// Resolver for local audio files and directories.
#[derive(Debug, Default)]
pub struct LocalResolver;

impl LocalResolver {
    /// Create a new `LocalResolver`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Check if a path corresponds to a supported audio extension.
    #[must_use]
    pub fn is_audio_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                let ext_lower = ext.to_ascii_lowercase();
                SUPPORTED_EXTENSIONS.iter().any(|&sup| sup == ext_lower)
            })
            .unwrap_or(false)
    }

    /// Resolve a local path into a `MediaItem`.
    pub fn resolve_file(&self, path: impl Into<PathBuf>) -> Result<MediaItem> {
        let path = path.into();
        if !path.exists() {
            return Err(UPlayError::File {
                path: path.clone(),
                reason: "file does not exist".into(),
            });
        }
        Ok(MediaItem {
            id: path.to_string_lossy().to_string(),
            source: Source::LocalFile(path.clone()),
            title: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown Track")
                .to_string(),
            artist: None,
            duration: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_audio_file() {
        assert!(LocalResolver::is_audio_file(Path::new("song.mp3")));
        assert!(LocalResolver::is_audio_file(Path::new("song.FLAC")));
        assert!(!LocalResolver::is_audio_file(Path::new("text.txt")));
        assert!(!LocalResolver::is_audio_file(Path::new("binary")));
    }
}
