//! Playback session state and snapshot management.

use super::state::{MediaItem, PlaybackMode};
use std::time::Duration;

/// Serializable snapshot representing the user's active playback session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Active playlist name, if any.
    pub playlist_name: Option<String>,
    /// Index of the current playing track.
    pub current_index: Option<usize>,
    /// Elapsed track position.
    pub position: Duration,
    /// Volume level (0..=100).
    pub volume: u8,
    /// Sequencing mode.
    pub playback_mode: PlaybackMode,
    /// Active playlist items.
    pub items: Vec<MediaItem>,
    /// Queued items.
    pub queue: Vec<MediaItem>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            playlist_name: None,
            current_index: None,
            position: Duration::ZERO,
            volume: 80,
            playback_mode: PlaybackMode::default(),
            items: Vec::new(),
            queue: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_default() {
        let session = Session::default();
        assert_eq!(session.volume, 80);
        assert_eq!(session.position, Duration::ZERO);
        assert!(session.items.is_empty());
        assert!(session.queue.is_empty());
    }
}
