//! HTTP network stream resolver.

use crate::core::state::{MediaItem, Source};
use crate::error::{Result, UPlayError};

/// Resolver for direct HTTP/HTTPS audio streams.
#[derive(Debug, Default)]
pub struct HttpResolver;

impl HttpResolver {
    /// Create a new `HttpResolver`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Resolve an HTTP/HTTPS stream URL into a `MediaItem`.
    pub fn resolve(&self, url: &str) -> Result<MediaItem> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(UPlayError::Network(format!(
                "Invalid HTTP stream URL scheme: {url}"
            )));
        }

        Ok(MediaItem {
            id: url.to_string(),
            source: Source::HttpStream(url.to_string()),
            title: url.to_string(),
            artist: None,
            duration: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_resolver() {
        let resolver = HttpResolver::new();
        let res = resolver.resolve("https://example.com/radio.mp3");
        assert!(res.is_ok());
        let item = res.unwrap();
        assert_eq!(item.id, "https://example.com/radio.mp3");

        let bad = resolver.resolve("ftp://example.com/stream");
        assert!(bad.is_err());
    }
}
