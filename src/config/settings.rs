//! Static configuration settings and default values.

use std::path::PathBuf;

/// Application configuration settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Default music directory to scan or browse.
    pub music_directory: PathBuf,
    /// Default volume level (0..=100).
    pub volume: u8,
    /// Whether shuffle is enabled by default.
    pub shuffle: bool,
    /// Repeat mode name ("none", "one", "playlist").
    pub repeat: String,
    /// Whether to automatically resume previous session on startup.
    pub resume: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let music_dir = dirs_fallback_music_dir();
        Self {
            music_directory: music_dir,
            volume: 80,
            shuffle: false,
            repeat: "none".to_string(),
            resume: false,
        }
    }
}

fn dirs_fallback_music_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join("Music")
    } else {
        PathBuf::from("~/Music")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.volume, 80);
        assert!(!settings.shuffle);
        assert_eq!(settings.repeat, "none");
        assert!(!settings.resume);
    }
}
