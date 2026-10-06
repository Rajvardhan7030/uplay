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

    /// Resolve a local path into a `MediaItem`, validating existence and format.
    pub fn resolve_file(&self, path: impl Into<PathBuf>) -> Result<MediaItem> {
        let path = path.into();
        if !path.exists() {
            return Err(UPlayError::File {
                path: path.clone(),
                reason: "file does not exist".into(),
            });
        }
        if !path.is_file() {
            return Err(UPlayError::File {
                path: path.clone(),
                reason: "path is not a file".into(),
            });
        }
        if !Self::is_audio_file(&path) {
            return Err(UPlayError::File {
                path: path.clone(),
                reason: "unsupported audio format".into(),
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

    /// Recursively scan a directory for supported audio files, returning them sorted deterministically.
    pub fn scan_directory(&self, dir_path: impl AsRef<Path>) -> Result<Vec<MediaItem>> {
        let dir = dir_path.as_ref();
        if !dir.exists() {
            return Err(UPlayError::File {
                path: dir.to_path_buf(),
                reason: "directory does not exist".into(),
            });
        }
        if !dir.is_dir() {
            return Err(UPlayError::File {
                path: dir.to_path_buf(),
                reason: "path is not a directory".into(),
            });
        }

        let mut collected_paths = Vec::new();
        Self::collect_audio_files(dir, &mut collected_paths)?;

        if collected_paths.is_empty() {
            return Err(UPlayError::File {
                path: dir.to_path_buf(),
                reason: "no supported audio files found".into(),
            });
        }

        // Deterministic sorting (lexicographical order)
        collected_paths.sort();

        let items = collected_paths
            .into_iter()
            .map(|path| {
                let title = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown Track")
                    .to_string();
                MediaItem {
                    id: path.to_string_lossy().to_string(),
                    source: Source::LocalFile(path),
                    title,
                    artist: None,
                    duration: None,
                }
            })
            .collect();

        Ok(items)
    }

    /// Resolve a path (file or directory) into a list of playable `MediaItem`s.
    pub fn resolve_path(&self, path: impl AsRef<Path>) -> Result<Vec<MediaItem>> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(UPlayError::File {
                path: path.to_path_buf(),
                reason: "file or directory does not exist".into(),
            });
        }

        if path.is_dir() {
            self.scan_directory(path)
        } else {
            let item = self.resolve_file(path.to_path_buf())?;
            Ok(vec![item])
        }
    }

    fn collect_audio_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let entries = std::fs::read_dir(dir).map_err(|e| UPlayError::File {
            path: dir.to_path_buf(),
            reason: format!("failed to read directory: {e}"),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| UPlayError::File {
                path: dir.to_path_buf(),
                reason: format!("failed to read directory entry: {e}"),
            })?;
            let path = entry.path();

            // Skip hidden files/directories (starting with dot)
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') {
                    continue;
                }
            }

            if path.is_dir() {
                Self::collect_audio_files(&path, out)?;
            } else if path.is_file() && Self::is_audio_file(&path) {
                out.push(path);
            }
        }

        Ok(())
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

    #[test]
    fn test_resolve_file_validation() {
        let resolver = LocalResolver::new();
        let err = resolver.resolve_file(PathBuf::from("/nonexistent/song.mp3"));
        assert!(err.is_err());
    }

    #[test]
    fn test_scan_directory_nonexistent_or_empty() {
        let resolver = LocalResolver::new();
        let err = resolver.scan_directory("/nonexistent_dir");
        assert!(err.is_err());

        // Target directory without audio files
        let temp_dir = std::env::temp_dir().join("uplay_test_empty_dir");
        let _ = std::fs::create_dir_all(&temp_dir);
        let err = resolver.scan_directory(&temp_dir);
        assert!(err.is_err());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
