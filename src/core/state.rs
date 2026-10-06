//! Authoritative state representations and domain models for UPlay.

use std::path::PathBuf;
use std::time::Duration;

/// Origin and type of a media item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A local audio file on disk.
    LocalFile(PathBuf),
    /// A direct HTTP/HTTPS audio stream URL.
    HttpStream(String),
    /// A YouTube video/stream URL.
    YouTube(String),
}

/// A playable media track or stream with its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaItem {
    /// Unique identifier for the item.
    pub id: String,
    /// Origin and resolution source.
    pub source: Source,
    /// Human-readable title or filename.
    pub title: String,
    /// Optional artist name.
    pub artist: Option<String>,
    /// Track duration if known.
    pub duration: Option<Duration>,
}

impl MediaItem {
    /// Create a new `MediaItem` from a local file path.
    #[must_use]
    pub fn from_local_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Track")
            .to_string();

        Self {
            id: path.to_string_lossy().to_string(),
            source: Source::LocalFile(path),
            title,
            artist: None,
            duration: None,
        }
    }
}

/// Status of the audio playback engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerStatus {
    /// No media is currently playing and playback is reset.
    #[default]
    Stopped,
    /// Media is actively playing.
    Playing,
    /// Media playback is temporarily paused.
    Paused,
}

/// Mode governing playback sequence and repeat behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    /// Play tracks sequentially in playlist order.
    #[default]
    Sequential,
    /// Play tracks in randomized order.
    Shuffle,
    /// Repeat the current track indefinitely.
    RepeatOne,
    /// Repeat the entire playlist after the last track finishes.
    RepeatPlaylist,
}

/// Authoritative snapshot of the player state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    /// Current status (stopped, playing, paused).
    pub status: PlayerStatus,
    /// Currently loaded media item, if any.
    pub current_item: Option<MediaItem>,
    /// Current playback elapsed position.
    pub position: Duration,
    /// Volume level from 0 to 100.
    pub volume: u8,
    /// Active playback sequencing mode.
    pub playback_mode: PlaybackMode,
    /// Active playlist items.
    pub playlist: Vec<MediaItem>,
    /// High-priority queue items.
    pub queue: Vec<MediaItem>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            status: PlayerStatus::Stopped,
            current_item: None,
            position: Duration::ZERO,
            volume: 80,
            playback_mode: PlaybackMode::default(),
            playlist: Vec::new(),
            queue: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_item_from_local_path() {
        let item = MediaItem::from_local_path("/path/to/song.flac");
        assert_eq!(item.title, "song.flac");
        assert_eq!(
            item.source,
            Source::LocalFile(PathBuf::from("/path/to/song.flac"))
        );
    }

    #[test]
    fn test_default_player_state() {
        let state = PlayerState::default();
        assert_eq!(state.status, PlayerStatus::Stopped);
        assert_eq!(state.volume, 80);
        assert_eq!(state.playback_mode, PlaybackMode::Sequential);
        assert!(state.playlist.is_empty());
        assert!(state.queue.is_empty());
    }
}
