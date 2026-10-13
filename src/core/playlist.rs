//! Playlist models and collection management.

use super::state::{MediaItem, Source};
use crate::error::{Result, UPlayError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A named collection of media items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playlist {
    /// Playlist name.
    pub name: String,
    /// List of media items in order.
    pub items: Vec<MediaItem>,
}

impl Playlist {
    /// Create a new empty playlist with a given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            items: Vec::new(),
        }
    }

    /// Add an item to the playlist.
    pub fn add(&mut self, item: MediaItem) {
        self.items.push(item);
    }

    /// Remove an item at the specified index.
    pub fn remove(&mut self, index: usize) -> Option<MediaItem> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Total number of tracks in the playlist.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the playlist contains any tracks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get a slice of all items in the playlist.
    #[must_use]
    pub fn items(&self) -> &[MediaItem] {
        &self.items
    }

    /// Clear all items from the playlist.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Remove a track by 1-based index (if numeric string) or by matching title or ID/path.
    pub fn remove_by_target(&mut self, target: &str) -> Option<MediaItem> {
        if let Ok(idx) = target.parse::<usize>() {
            if idx > 0 && idx <= self.items.len() {
                return Some(self.items.remove(idx - 1));
            }
        }
        if let Some(pos) = self.items.iter().position(|item| {
            item.title == target
                || item.id == target
                || item.title.eq_ignore_ascii_case(target)
                || item
                    .title
                    .to_ascii_lowercase()
                    .contains(&target.to_ascii_lowercase())
        }) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    /// Serialize this playlist to standard extended M3U8 string format.
    #[must_use]
    pub fn to_m3u(&self) -> String {
        let mut out = String::from("#EXTM3U\n");
        for item in &self.items {
            let duration_secs = item.duration.map_or(-1, |d| d.as_secs() as i64);
            let display_title = match &item.artist {
                Some(artist) => format!("{} - {}", artist, item.title),
                None => item.title.clone(),
            };
            out.push_str(&format!("#EXTINF:{duration_secs},{display_title}\n"));
            let uri = match &item.source {
                Source::LocalFile(p) => p.to_string_lossy().to_string(),
                Source::HttpStream(u) => u.clone(),
                Source::YouTube(u) => u.clone(),
            };
            out.push_str(&uri);
            out.push('\n');
        }
        out
    }

    /// Parse a playlist from M3U / M3U8 content.
    #[must_use]
    pub fn from_m3u(name: impl Into<String>, content: &str) -> Self {
        Self::from_m3u_with_base_dir(name, content, None)
    }

    /// Parse a playlist from M3U / M3U8 content with an optional base directory for relative paths.
    #[must_use]
    pub fn from_m3u_with_base_dir(
        name: impl Into<String>,
        content: &str,
        base_dir: Option<&Path>,
    ) -> Self {
        let mut items = Vec::new();
        let mut pending_duration: Option<Duration> = None;
        let mut pending_title: Option<String> = None;
        let mut pending_artist: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix('#') {
                if let Some(extinf) = rest.strip_prefix("EXTINF:") {
                    if let Some((dur_str, meta)) = extinf.split_once(',') {
                        if let Ok(secs) = dur_str.trim().parse::<i64>() {
                            if secs >= 0 {
                                pending_duration = Some(Duration::from_secs(secs as u64));
                            }
                        }
                        let meta = meta.trim();
                        if let Some((artist, title)) = meta.split_once(" - ") {
                            pending_artist = Some(artist.trim().to_string());
                            pending_title = Some(title.trim().to_string());
                        } else if !meta.is_empty() {
                            pending_title = Some(meta.to_string());
                        }
                    }
                }
                continue;
            }

            let entry = trimmed;
            let item = if entry.starts_with("http://") || entry.starts_with("https://") {
                if crate::sources::YouTubeResolver::is_youtube_url(entry) {
                    MediaItem {
                        id: entry.to_string(),
                        source: Source::YouTube(entry.to_string()),
                        title: pending_title
                            .take()
                            .unwrap_or_else(|| format!("YouTube: {entry}")),
                        artist: pending_artist.take(),
                        duration: pending_duration.take(),
                    }
                } else {
                    MediaItem {
                        id: entry.to_string(),
                        source: Source::HttpStream(entry.to_string()),
                        title: pending_title.take().unwrap_or_else(|| entry.to_string()),
                        artist: pending_artist.take(),
                        duration: pending_duration.take(),
                    }
                }
            } else {
                let path = PathBuf::from(entry);
                let resolved_path = if path.is_relative() {
                    if let Some(base) = base_dir {
                        let candidate = base.join(&path);
                        if candidate.exists() { candidate } else { path }
                    } else {
                        path
                    }
                } else {
                    path
                };

                let title = pending_title.take().unwrap_or_else(|| {
                    resolved_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown Track")
                        .to_string()
                });

                MediaItem {
                    id: resolved_path.to_string_lossy().to_string(),
                    source: Source::LocalFile(resolved_path),
                    title,
                    artist: pending_artist.take(),
                    duration: pending_duration.take(),
                }
            };

            pending_duration = None;
            pending_title = None;
            pending_artist = None;

            items.push(item);
        }

        Self {
            name: name.into(),
            items,
        }
    }

    /// Read a playlist from an M3U / M3U8 file on disk.
    pub fn from_m3u_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(UPlayError::File {
                path: path.to_path_buf(),
                reason: "file does not exist".into(),
            });
        }
        if !path.is_file() {
            return Err(UPlayError::File {
                path: path.to_path_buf(),
                reason: "path is not a file".into(),
            });
        }

        let content = std::fs::read_to_string(path).map_err(|e| UPlayError::File {
            path: path.to_path_buf(),
            reason: format!("failed to read playlist file: {e}"),
        })?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("playlist");

        let base_dir = path.parent();
        Ok(Self::from_m3u_with_base_dir(name, &content, base_dir))
    }

    /// Save this playlist to a specified M3U / M3U8 file path on disk.
    pub fn save_m3u(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| UPlayError::File {
                    path: parent.to_path_buf(),
                    reason: format!("failed to create playlist directory: {e}"),
                })?;
            }
        }

        let m3u_data = self.to_m3u();
        std::fs::write(path, m3u_data).map_err(|e| UPlayError::File {
            path: path.to_path_buf(),
            reason: format!("failed to write playlist file: {e}"),
        })?;

        Ok(())
    }
}

/// Default filesystem directory used for persistent playlist storage (`~/.config/uplay/playlists`).
#[must_use]
pub fn default_playlist_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("uplay").join("playlists");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home)
                .join(".config")
                .join("uplay")
                .join("playlists");
        }
    }
    PathBuf::from(".config").join("uplay").join("playlists")
}

/// In-memory and on-disk manager for saved playlists.
#[derive(Debug, Clone)]
pub struct PlaylistManager {
    playlists: HashMap<String, Playlist>,
    storage_dir: PathBuf,
}

impl Default for PlaylistManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistManager {
    /// Create a new `PlaylistManager` with the default persistent storage directory.
    #[must_use]
    pub fn new() -> Self {
        Self {
            playlists: HashMap::new(),
            storage_dir: default_playlist_dir(),
        }
    }

    /// Create a new `PlaylistManager` with a custom storage directory.
    #[must_use]
    pub fn with_storage_dir(storage_dir: impl Into<PathBuf>) -> Self {
        Self {
            playlists: HashMap::new(),
            storage_dir: storage_dir.into(),
        }
    }

    /// Get reference to the playlist storage directory.
    #[must_use]
    pub fn storage_dir(&self) -> &Path {
        &self.storage_dir
    }

    /// Set a new storage directory for playlists.
    pub fn set_storage_dir(&mut self, storage_dir: impl Into<PathBuf>) {
        self.storage_dir = storage_dir.into();
    }

    /// Save or replace a playlist by name in memory.
    pub fn save(&mut self, playlist: Playlist) {
        self.playlists.insert(playlist.name.clone(), playlist);
    }

    /// Retrieve an in-memory playlist by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Playlist> {
        self.playlists.get(name)
    }

    /// Remove an in-memory playlist by name.
    pub fn delete(&mut self, name: &str) -> bool {
        self.playlists.remove(name).is_some()
    }

    /// List all in-memory playlist names.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<String> = self.playlists.keys().cloned().collect();
        names.sort();
        names
    }

    /// Save a playlist to persistent disk storage in M3U8 format.
    pub fn save_to_disk(&self, playlist: &Playlist) -> Result<()> {
        let file_name = if playlist.name.ends_with(".m3u") || playlist.name.ends_with(".m3u8") {
            playlist.name.clone()
        } else {
            format!("{}.m3u8", playlist.name)
        };
        let file_path = self.storage_dir.join(file_name);
        playlist.save_m3u(&file_path)
    }

    /// Load a playlist by name from persistent disk storage.
    pub fn load_from_disk(&self, name: &str) -> Result<Playlist> {
        let target_path = self.resolve_playlist_file_path(name)?;
        let mut playlist = Playlist::from_m3u_file(&target_path)?;
        if let Some(stem) = target_path.file_stem().and_then(|s| s.to_str()) {
            playlist.name = stem.to_string();
        }
        Ok(playlist)
    }

    /// List all saved playlist names found on persistent disk storage.
    pub fn list_saved(&self) -> Result<Vec<String>> {
        if !self.storage_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&self.storage_dir).map_err(|e| UPlayError::File {
            path: self.storage_dir.clone(),
            reason: format!("failed to read playlist storage directory: {e}"),
        })?;

        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| UPlayError::File {
                path: self.storage_dir.clone(),
                reason: format!("failed to read playlist directory entry: {e}"),
            })?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    if ext_lower == "m3u" || ext_lower == "m3u8" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            names.push(stem.to_string());
                        }
                    }
                }
            }
        }

        names.sort();
        names.dedup();
        Ok(names)
    }

    /// Delete a saved playlist file from persistent disk storage.
    pub fn delete_saved(&self, name: &str) -> Result<()> {
        let path = self.resolve_playlist_file_path(name)?;
        std::fs::remove_file(&path).map_err(|e| UPlayError::File {
            path: path.clone(),
            reason: format!("failed to delete playlist file: {e}"),
        })
    }

    fn resolve_playlist_file_path(&self, name: &str) -> Result<PathBuf> {
        if name.ends_with(".m3u") || name.ends_with(".m3u8") {
            let direct = self.storage_dir.join(name);
            if direct.is_file() {
                return Ok(direct);
            }
        }

        let m3u8_path = self.storage_dir.join(format!("{name}.m3u8"));
        if m3u8_path.is_file() {
            return Ok(m3u8_path);
        }

        let m3u_path = self.storage_dir.join(format!("{name}.m3u"));
        if m3u_path.is_file() {
            return Ok(m3u_path);
        }

        let exact_path = self.storage_dir.join(name);
        if exact_path.is_file() {
            return Ok(exact_path);
        }

        Err(UPlayError::File {
            path: self.storage_dir.join(format!("{name}.m3u8")),
            reason: format!("playlist '{name}' not found"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playlist_operations() {
        let mut playlist = Playlist::new("coding");
        assert_eq!(playlist.name, "coding");
        assert!(playlist.is_empty());

        let song = MediaItem::from_local_path("test.mp3");
        playlist.add(song);
        assert_eq!(playlist.len(), 1);

        let mut manager = PlaylistManager::new();
        manager.save(playlist);
        assert_eq!(manager.list(), vec!["coding"]);

        let loaded = manager.get("coding").expect("should find playlist");
        assert_eq!(loaded.len(), 1);

        assert!(manager.delete("coding"));
        assert!(manager.get("coding").is_none());
    }

    #[test]
    fn test_m3u_serialization_and_parsing() {
        let mut playlist = Playlist::new("chill");
        let item1 = MediaItem {
            id: "/music/track1.mp3".into(),
            source: Source::LocalFile(PathBuf::from("/music/track1.mp3")),
            title: "track1".into(),
            artist: Some("Artist1".into()),
            duration: Some(Duration::from_secs(120)),
        };
        let item2 = MediaItem {
            id: "https://example.com/radio.mp3".into(),
            source: Source::HttpStream("https://example.com/radio.mp3".into()),
            title: "radio".into(),
            artist: None,
            duration: None,
        };
        playlist.add(item1);
        playlist.add(item2);

        let m3u = playlist.to_m3u();
        assert!(m3u.starts_with("#EXTM3U\n"));
        assert!(m3u.contains("#EXTINF:120,Artist1 - track1\n/music/track1.mp3\n"));
        assert!(m3u.contains("#EXTINF:-1,radio\nhttps://example.com/radio.mp3\n"));

        let parsed = Playlist::from_m3u("chill", &m3u);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.items[0].title, "track1");
        assert_eq!(parsed.items[0].artist.as_deref(), Some("Artist1"));
        assert_eq!(parsed.items[0].duration, Some(Duration::from_secs(120)));
        assert_eq!(parsed.items[1].title, "radio");
        assert_eq!(
            parsed.items[1].source,
            Source::HttpStream("https://example.com/radio.mp3".into())
        );
    }

    #[test]
    fn test_playlist_manager_disk_lifecycle() {
        let temp_dir = std::env::temp_dir().join("uplay_test_playlist_mgr");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let manager = PlaylistManager::with_storage_dir(&temp_dir);
        assert_eq!(manager.list_saved().unwrap(), Vec::<String>::new());

        let mut playlist = Playlist::new("coding");
        playlist.add(MediaItem::from_local_path("test1.mp3"));
        playlist.add(MediaItem::from_local_path("test2.flac"));

        manager
            .save_to_disk(&playlist)
            .expect("save should succeed");
        assert_eq!(manager.list_saved().unwrap(), vec!["coding"]);

        let loaded = manager
            .load_from_disk("coding")
            .expect("load should succeed");
        assert_eq!(loaded.name, "coding");
        assert_eq!(loaded.len(), 2);

        manager
            .delete_saved("coding")
            .expect("delete should succeed");
        assert_eq!(manager.list_saved().unwrap(), Vec::<String>::new());
        assert!(manager.load_from_disk("coding").is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
