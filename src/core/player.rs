//! Player Engine: Authoritative controller and coordinator for audio playback.

use super::queue::QueueManager;
use super::state::{MediaItem, PlaybackMode, PlayerState, PlayerStatus};
use crate::audio::{AudioBackend, NullAudioBackend, RodioBackend};
use crate::error::Result;
use std::fmt;
use std::time::Duration;

/// The central player coordinator that holds state, queue, and tracks.
pub struct PlayerEngine {
    state: PlayerState,
    queue: QueueManager,
    current_index: Option<usize>,
    backend: Box<dyn AudioBackend>,
}

impl fmt::Debug for PlayerEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerEngine")
            .field("state", &self.state)
            .field("queue", &self.queue)
            .field("current_index", &self.current_index)
            .finish()
    }
}

impl Default for PlayerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerEngine {
    /// Create a new instance of `PlayerEngine` with a null audio backend.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: PlayerState::default(),
            queue: QueueManager::new(),
            current_index: None,
            backend: Box::new(NullAudioBackend::new()),
        }
    }

    /// Create a new instance of `PlayerEngine` with a custom audio backend.
    #[must_use]
    pub fn with_backend(backend: Box<dyn AudioBackend>) -> Self {
        Self {
            state: PlayerState::default(),
            queue: QueueManager::new(),
            current_index: None,
            backend,
        }
    }

    /// Create a new instance of `PlayerEngine` configured with the Rodio backend.
    #[must_use]
    pub fn with_rodio() -> Self {
        Self::with_backend(Box::new(RodioBackend::new()))
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

    /// Load and prepare a track for playback without starting.
    pub fn load(&mut self, item: MediaItem) -> Result<()> {
        self.backend.load(&item.source)?;
        self.state.current_item = Some(item);
        self.state.position = Duration::ZERO;
        self.state.status = PlayerStatus::Stopped;
        Ok(())
    }

    /// Load and immediately begin playing a track.
    pub fn load_and_play(&mut self, item: MediaItem) -> Result<()> {
        self.load(item)?;
        self.play()
    }

    /// Start or resume playback.
    pub fn play(&mut self) -> Result<()> {
        if self.state.current_item.is_none() {
            self.next()?;
        } else {
            self.backend.play()?;
            self.state.status = PlayerStatus::Playing;
        }
        Ok(())
    }

    /// Pause active playback.
    pub fn pause(&mut self) -> Result<()> {
        if self.state.status == PlayerStatus::Playing {
            self.backend.pause()?;
            self.state.status = PlayerStatus::Paused;
        }
        Ok(())
    }

    /// Resume paused playback.
    pub fn resume(&mut self) -> Result<()> {
        self.play()
    }

    /// Toggle play / pause state.
    pub fn toggle_play(&mut self) -> Result<()> {
        if self.state.status == PlayerStatus::Playing {
            self.pause()
        } else {
            self.play()
        }
    }

    /// Stop playback and reset track position.
    pub fn stop(&mut self) -> Result<()> {
        self.backend.stop()?;
        self.state.status = PlayerStatus::Stopped;
        self.state.position = Duration::ZERO;
        Ok(())
    }

    /// Seek to an absolute position.
    pub fn seek(&mut self, position: Duration) -> Result<()> {
        self.backend.seek(position)?;
        self.state.position = position;
        Ok(())
    }

    /// Seek relative to current playback position.
    pub fn seek_relative(&mut self, offset_secs: i64) -> Result<()> {
        let current = self.position();
        let target = if offset_secs >= 0 {
            current.saturating_add(Duration::from_secs(offset_secs as u64))
        } else {
            let sub = Duration::from_secs((-offset_secs) as u64);
            current.checked_sub(sub).unwrap_or(Duration::ZERO)
        };
        self.seek(target)
    }

    /// Advance to the next track (checking priority queue first, then playlist).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<MediaItem>> {
        // Priority queue item takes precedence
        if let Some(item) = self.queue.next() {
            self.sync_queue_state();
            self.load_and_play(item.clone())?;
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
        self.load_and_play(item.clone())?;
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
        self.load_and_play(item.clone())?;
        Ok(Some(item))
    }

    /// Adjust volume level (0..=100).
    pub fn set_volume(&mut self, volume: u8) -> Result<()> {
        let vol = volume.min(100);
        self.backend.set_volume(vol)?;
        self.state.volume = vol;
        Ok(())
    }

    /// Increase volume by given percentage step.
    pub fn volume_up(&mut self, step: u8) -> Result<()> {
        self.set_volume(self.state.volume.saturating_add(step))
    }

    /// Decrease volume by given percentage step.
    pub fn volume_down(&mut self, step: u8) -> Result<()> {
        self.set_volume(self.state.volume.saturating_sub(step))
    }

    /// Get current playback elapsed position.
    #[must_use]
    pub fn position(&self) -> Duration {
        self.backend.position()
    }

    /// Get total audio duration if known.
    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.backend
            .duration()
            .or_else(|| self.state.current_item.as_ref().and_then(|i| i.duration))
    }

    /// Check if audio playback has completed.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.backend.is_finished()
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
        engine.set_volume(90).unwrap();
        assert_eq!(engine.state().volume, 90);

        // Volume up & down
        engine.volume_up(5).unwrap();
        assert_eq!(engine.state().volume, 95);
        engine.volume_down(10).unwrap();
        assert_eq!(engine.state().volume, 85);

        // Relative seek
        assert!(engine.seek_relative(10).is_ok());
    }
}
