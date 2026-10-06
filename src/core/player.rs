//! Player Engine: Authoritative controller and coordinator for audio playback.

use super::queue::QueueManager;
use super::state::{MediaItem, PlaybackMode, PlayerState, PlayerStatus};
use crate::error::Result;
use std::time::Duration;

/// The central player coordinator that holds state, queue, and tracks.
#[derive(Debug, Default)]
pub struct PlayerEngine {
    state: PlayerState,
    queue: QueueManager,
    current_index: Option<usize>,
}

impl PlayerEngine {
    /// Create a new instance of `PlayerEngine`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve the current authoritative player state snapshot.
    #[must_use]
    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    /// Load a list of tracks as the active playlist.
    pub fn load_playlist(&mut self, items: Vec<MediaItem>) {
        self.state.playlist = items;
        self.current_index = None;
        self.state.current_item = None;
        self.state.position = Duration::ZERO;
        self.state.status = PlayerStatus::Stopped;
    }

    /// Start or resume playback.
    pub fn play(&mut self) -> Result<()> {
        if self.state.current_item.is_none() {
            self.next()?;
        } else {
            self.state.status = PlayerStatus::Playing;
        }
        Ok(())
    }

    /// Pause active playback.
    pub fn pause(&mut self) -> Result<()> {
        if self.state.status == PlayerStatus::Playing {
            self.state.status = PlayerStatus::Paused;
        }
        Ok(())
    }

    /// Stop playback and reset track position.
    pub fn stop(&mut self) -> Result<()> {
        self.state.status = PlayerStatus::Stopped;
        self.state.position = Duration::ZERO;
        Ok(())
    }

    /// Advance to the next track (checking priority queue first, then playlist).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<MediaItem>> {
        // Priority queue item takes precedence
        if let Some(item) = self.queue.next() {
            self.state.current_item = Some(item.clone());
            self.state.status = PlayerStatus::Playing;
            self.state.position = Duration::ZERO;
            self.sync_queue_state();
            return Ok(Some(item));
        }

        if self.state.playlist.is_empty() {
            self.state.current_item = None;
            self.state.status = PlayerStatus::Stopped;
            return Ok(None);
        }

        let next_idx = match self.current_index {
            Some(idx) => (idx + 1) % self.state.playlist.len(),
            None => 0,
        };

        self.current_index = Some(next_idx);
        let item = self.state.playlist[next_idx].clone();
        self.state.current_item = Some(item.clone());
        self.state.status = PlayerStatus::Playing;
        self.state.position = Duration::ZERO;
        Ok(Some(item))
    }

    /// Move to the previous track in the playlist.
    pub fn previous(&mut self) -> Result<Option<MediaItem>> {
        if self.state.playlist.is_empty() {
            self.state.current_item = None;
            self.state.status = PlayerStatus::Stopped;
            return Ok(None);
        }

        let prev_idx = match self.current_index {
            Some(0) | None => self.state.playlist.len().saturating_sub(1),
            Some(idx) => idx - 1,
        };

        self.current_index = Some(prev_idx);
        let item = self.state.playlist[prev_idx].clone();
        self.state.current_item = Some(item.clone());
        self.state.status = PlayerStatus::Playing;
        self.state.position = Duration::ZERO;
        Ok(Some(item))
    }

    /// Adjust volume level (0..=100).
    pub fn set_volume(&mut self, volume: u8) {
        self.state.volume = volume.min(100);
    }

    /// Set playback mode (Sequential, Shuffle, RepeatOne, RepeatPlaylist).
    pub fn set_mode(&mut self, mode: PlaybackMode) {
        self.state.playback_mode = mode;
    }

    /// Add an item to the priority queue.
    pub fn queue_add(&mut self, item: MediaItem) {
        self.queue.add(item);
        self.sync_queue_state();
    }

    fn sync_queue_state(&mut self) {
        self.state.queue = self.queue.items().to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_engine_navigation() {
        let mut engine = PlayerEngine::new();
        let track1 = MediaItem::from_local_path("song1.mp3");
        let track2 = MediaItem::from_local_path("song2.mp3");

        engine.load_playlist(vec![track1.clone(), track2.clone()]);
        assert_eq!(engine.state().status, PlayerStatus::Stopped);

        // Next starts first track
        let next_track = engine.next().unwrap();
        assert_eq!(next_track, Some(track1.clone()));
        assert_eq!(engine.state().status, PlayerStatus::Playing);

        // Next advances to second track
        let next_track2 = engine.next().unwrap();
        assert_eq!(next_track2, Some(track2.clone()));

        // Previous goes back to first track
        let prev_track = engine.previous().unwrap();
        assert_eq!(prev_track, Some(track1.clone()));

        // Pause
        engine.pause().unwrap();
        assert_eq!(engine.state().status, PlayerStatus::Paused);

        // Volume
        engine.set_volume(90);
        assert_eq!(engine.state().volume, 90);
    }
}
