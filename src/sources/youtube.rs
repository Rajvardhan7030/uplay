//! YouTube stream resolver interface.

use crate::core::state::{MediaItem, Source};
use crate::error::{Result, UPlayError};

/// Resolver interface for YouTube URLs.
#[derive(Debug, Default)]
pub struct YouTubeResolver;

impl YouTubeResolver {
    /// Create a new `YouTubeResolver`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Check if a given URL or query targets YouTube.
    #[must_use]
    pub fn is_youtube_url(url: &str) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }

    /// Resolve a YouTube URL into a `MediaItem`.
    pub fn resolve(&self, url: &str) -> Result<MediaItem> {
        if !Self::is_youtube_url(url) {
            return Err(UPlayError::Resolver(format!(
                "Not a valid YouTube URL: {url}"
            )));
        }

        Ok(MediaItem {
            id: url.to_string(),
            source: Source::YouTube(url.to_string()),
            title: format!("YouTube: {url}"),
            artist: None,
            duration: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_resolver_detection() {
        assert!(YouTubeResolver::is_youtube_url(
            "https://www.youtube.com/watch?v=123"
        ));
        assert!(YouTubeResolver::is_youtube_url("https://youtu.be/123"));
        assert!(!YouTubeResolver::is_youtube_url(
            "https://example.com/audio.mp3"
        ));

        let resolver = YouTubeResolver::new();
        assert!(resolver.resolve("https://youtu.be/abc").is_ok());
        assert!(resolver.resolve("https://example.com").is_err());
    }
}
