//! Playlist models and collection management.

use super::state::MediaItem;
use std::collections::HashMap;

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
}

/// In-memory manager for saved playlists.
#[derive(Debug, Default)]
pub struct PlaylistManager {
    playlists: HashMap<String, Playlist>,
}

impl PlaylistManager {
    /// Create a new empty `PlaylistManager`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            playlists: HashMap::new(),
        }
    }

    /// Save or replace a playlist by name.
    pub fn save(&mut self, playlist: Playlist) {
        self.playlists.insert(playlist.name.clone(), playlist);
    }

    /// Retrieve a playlist by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Playlist> {
        self.playlists.get(name)
    }

    /// Remove a playlist by name.
    pub fn delete(&mut self, name: &str) -> bool {
        self.playlists.remove(name).is_some()
    }

    /// List all playlist names.
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<String> = self.playlists.keys().cloned().collect();
        names.sort();
        names
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
}
