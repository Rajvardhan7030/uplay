//! Media source resolvers for local files, HTTP streams, and YouTube.

pub mod http;
pub mod local;
pub mod youtube;

pub use http::HttpResolver;
pub use local::LocalResolver;
pub use youtube::YouTubeResolver;

use crate::core::state::MediaItem;
use crate::error::Result;
use std::path::Path;

/// Resolve a single target (URL, local audio file, directory, or playlist file) into playable `MediaItem`s.
pub fn resolve_target(target: &str) -> Result<Vec<MediaItem>> {
    if YouTubeResolver::is_youtube_url(target) {
        let item = YouTubeResolver::new().resolve(target)?;
        Ok(vec![item])
    } else if target.starts_with("http://") || target.starts_with("https://") {
        let item = HttpResolver::new().resolve(target)?;
        Ok(vec![item])
    } else {
        LocalResolver::new().resolve_path(Path::new(target))
    }
}

/// Resolve multiple targets into a combined list of playable `MediaItem`s.
pub fn resolve_targets(targets: &[String]) -> Result<Vec<MediaItem>> {
    let mut all_items = Vec::new();
    for target in targets {
        let items = resolve_target(target)?;
        all_items.extend(items);
    }
    Ok(all_items)
}
