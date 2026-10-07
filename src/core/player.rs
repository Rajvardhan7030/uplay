//! Player Engine: Authoritative controller and coordinator for audio playback.

use super::queue::QueueManager;
use super::state::{MediaItem, PlaybackMode, PlayerState, PlayerStatus};
use crate::audio::{AudioBackend, NullAudioBackend, RodioBackend};
use crate::error::Result;
use std::fmt;
use std::time::Duration;

/// A fast, deterministic pseudo-random number generator for shuffle sequencing.
#[derive(Debug, Clone)]
pub struct ShuffleRng {
    state: u64,
}

impl ShuffleRng {
    /// Create a new PRNG with a specific non-zero seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x853c_49e6_748f_ea9b
            } else {
                seed
            },
        }
    }

    /// Create a PRNG seeded from current system time.
    #[must_use]
    pub fn from_time() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(123_456_789, |d| d.as_nanos() as u64);
        Self::with_seed(seed)
    }

    /// Next 64-bit random number using xorshift64star.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// Random index in `0..max`.
    pub fn gen_range(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_u64() % (max as u64)) as usize
        }
    }

    /// In-place Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(i + 1);
            slice.swap(i, j);
        }
    }
}

/// The central player coordinator that holds state, queue, and tracks.
pub struct PlayerEngine {
    state: PlayerState,
    queue: QueueManager,
    current_index: Option<usize>,
    shuffle_order: Vec<usize>,
    shuffle_pos: usize,
    rng: ShuffleRng,
    backend: Box<dyn AudioBackend>,
}

impl fmt::Debug for PlayerEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerEngine")
            .field("state", &self.state)
            .field("queue", &self.queue)
            .field("current_index", &self.current_index)
            .field("shuffle_pos", &self.shuffle_pos)
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
            shuffle_order: Vec::new(),
            shuffle_pos: 0,
            rng: ShuffleRng::from_time(),
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
            shuffle_order: Vec::new(),
            shuffle_pos: 0,
            rng: ShuffleRng::from_time(),
            backend,
        }
    }

    /// Create a new instance of `PlayerEngine` configured with the Rodio backend.
    #[must_use]
    pub fn with_rodio() -> Self {
        Self::with_backend(Box::new(RodioBackend::new()))
    }

    /// Set a deterministic seed for the shuffle PRNG (useful for testing).
    pub fn set_shuffle_seed(&mut self, seed: u64) {
        self.rng = ShuffleRng::with_seed(seed);
        if self.state.playback_mode == PlaybackMode::Shuffle {
            self.rebuild_shuffle_order();
        }
    }

    /// Slice of the current shuffled track indices.
    #[must_use]
    pub fn shuffle_order(&self) -> &[usize] {
        &self.shuffle_order
    }

    /// Retrieve the current authoritative player state snapshot.
    #[must_use]
    pub fn state(&self) -> &PlayerState {
        &self.state
    }

    /// Total number of tracks in the active playlist.
    #[must_use]
    pub fn playlist_len(&self) -> usize {
        self.state.playlist.len()
    }

    /// Get the 0-based index of the currently active track in the playlist.
    #[must_use]
    pub fn current_track_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Load a list of tracks as the active playlist.
    pub fn load_playlist(&mut self, items: Vec<MediaItem>) {
        self.state.playlist = items;
        self.current_index = None;
        self.state.current_item = None;
        self.state.position = Duration::ZERO;
        self.state.status = PlayerStatus::Stopped;
        self.shuffle_pos = 0;
        self.rebuild_shuffle_order();
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

    /// Advance to the next track (checking priority queue first, then playback mode sequencing).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<MediaItem>> {
        // Priority queue item takes precedence without disturbing playlist sequencing
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

        let next_idx = match self.state.playback_mode {
            PlaybackMode::RepeatOne => self.current_index.unwrap_or(0),
            PlaybackMode::Shuffle => {
                if self.shuffle_order.is_empty() {
                    self.rebuild_shuffle_order();
                }

                let next_pos = match self.current_index {
                    Some(_) => {
                        let candidate = self.shuffle_pos + 1;
                        if candidate >= self.shuffle_order.len() {
                            self.stop()?;
                            return Ok(None);
                        }
                        candidate
                    }
                    None => 0,
                };

                self.shuffle_pos = next_pos;
                self.shuffle_order[next_pos]
            }
            PlaybackMode::RepeatPlaylist => match self.current_index {
                Some(idx) => (idx + 1) % self.state.playlist.len(),
                None => 0,
            },
            PlaybackMode::Sequential => match self.current_index {
                Some(idx) => {
                    let candidate = idx + 1;
                    if candidate >= self.state.playlist.len() {
                        self.stop()?;
                        return Ok(None);
                    }
                    candidate
                }
                None => 0,
            },
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

        if self.position() > Duration::from_secs(3) {
            self.seek(Duration::ZERO)?;
            return Ok(self.state.current_item.clone());
        }

        let prev_idx = match self.state.playback_mode {
            PlaybackMode::RepeatOne => {
                self.seek(Duration::ZERO)?;
                return Ok(self.state.current_item.clone());
            }
            PlaybackMode::Shuffle => {
                let prev_pos = self.shuffle_pos.saturating_sub(1);
                self.shuffle_pos = prev_pos;
                self.shuffle_order.get(prev_pos).copied().unwrap_or(0)
            }
            PlaybackMode::RepeatPlaylist => match self.current_index {
                Some(0) | None => self.state.playlist.len().saturating_sub(1),
                Some(idx) => idx - 1,
            },
            PlaybackMode::Sequential => match self.current_index {
                Some(0) | None => 0,
                Some(idx) => idx - 1,
            },
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
        if mode == PlaybackMode::Shuffle {
            self.rebuild_shuffle_order();
        }
    }

    /// Toggle shuffle mode on/off.
    pub fn toggle_shuffle(&mut self) {
        if self.state.playback_mode == PlaybackMode::Shuffle {
            self.set_mode(PlaybackMode::Sequential);
        } else {
            self.set_mode(PlaybackMode::Shuffle);
        }
    }

    /// Cycle through repeat modes (Sequential -> RepeatOne -> RepeatPlaylist -> Sequential).
    pub fn cycle_repeat(&mut self) {
        let next_mode = match self.state.playback_mode {
            PlaybackMode::Sequential | PlaybackMode::Shuffle => PlaybackMode::RepeatOne,
            PlaybackMode::RepeatOne => PlaybackMode::RepeatPlaylist,
            PlaybackMode::RepeatPlaylist => PlaybackMode::Sequential,
        };
        self.set_mode(next_mode);
    }

    // --- Priority Queue Operations ---

    /// Add an item to the end of the priority queue.
    pub fn queue_add(&mut self, item: MediaItem) {
        self.queue.add(item);
        self.sync_queue_state();
    }

    /// Add an item to play immediately next.
    pub fn queue_add_next(&mut self, item: MediaItem) {
        self.queue.add_next(item);
        self.sync_queue_state();
    }

    /// Remove a queue item by index.
    pub fn queue_remove(&mut self, index: usize) -> Option<MediaItem> {
        let removed = self.queue.remove(index);
        self.sync_queue_state();
        removed
    }

    /// Move a queue item from one index to another.
    pub fn queue_move(&mut self, from: usize, to: usize) -> bool {
        let ok = self.queue.move_item(from, to);
        self.sync_queue_state();
        ok
    }

    /// Clear all items from the queue.
    pub fn queue_clear(&mut self) {
        self.queue.clear();
        self.sync_queue_state();
    }

    /// Peek at the next item in the queue.
    #[must_use]
    pub fn queue_peek(&self) -> Option<&MediaItem> {
        self.queue.peek()
    }

    /// Return all current queue items.
    #[must_use]
    pub fn queue_items(&self) -> &[MediaItem] {
        self.queue.items()
    }

    /// Number of items currently queued.
    #[must_use]
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty.
    #[must_use]
    pub fn queue_is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn sync_queue_state(&mut self) {
        self.state.queue = self.queue.items().to_vec();
    }

    fn rebuild_shuffle_order(&mut self) {
        self.shuffle_order = (0..self.state.playlist.len()).collect();
        if self.state.playback_mode == PlaybackMode::Shuffle {
            self.rng.shuffle(&mut self.shuffle_order);
            if let Some(curr) = self.current_index {
                if let Some(pos) = self.shuffle_order.iter().position(|&x| x == curr) {
                    self.shuffle_order.swap(0, pos);
                    self.shuffle_pos = 0;
                }
            }
        }
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
        assert_eq!(engine.playlist_len(), 2);

        // Next starts first track (idx 0)
        let next_track = engine.next().unwrap();
        assert_eq!(next_track, Some(track1.clone()));
        assert_eq!(engine.state().status, PlayerStatus::Playing);
        assert_eq!(engine.current_track_index(), Some(0));

        // Next advances to second track (idx 1)
        let next_track2 = engine.next().unwrap();
        assert_eq!(next_track2, Some(track2.clone()));
        assert_eq!(engine.current_track_index(), Some(1));

        // Previous goes back to first track (idx 0)
        let prev_track = engine.previous().unwrap();
        assert_eq!(prev_track, Some(track1.clone()));
        assert_eq!(engine.current_track_index(), Some(0));

        // End of sequential playlist: advance from 1 -> finishes playlist (returns None)
        engine.next().unwrap();
        let end_of_playlist = engine.next().unwrap();
        assert_eq!(end_of_playlist, None);
        assert_eq!(engine.state().status, PlayerStatus::Stopped);
    }

    #[test]
    fn test_deterministic_shuffle() {
        let mut engine = PlayerEngine::new();
        let tracks: Vec<_> = (0..5)
            .map(|i| MediaItem::from_local_path(format!("song{i}.mp3")))
            .collect();
        engine.load_playlist(tracks);

        engine.set_mode(PlaybackMode::Shuffle);
        engine.set_shuffle_seed(42);

        let order = engine.shuffle_order().to_vec();
        assert_eq!(order.len(), 5);

        // Re-seeding with same seed produces identical order
        engine.set_shuffle_seed(42);
        assert_eq!(engine.shuffle_order(), &order[..]);

        // Sequential play follows this shuffle order
        for expected_idx in order {
            let track = engine.next().unwrap().expect("track");
            assert_eq!(track.title, format!("song{expected_idx}.mp3"));
        }
    }

    #[test]
    fn test_queue_priority_does_not_corrupt_playlist_or_shuffle() {
        let mut engine = PlayerEngine::new();
        let tracks: Vec<_> = (0..4)
            .map(|i| MediaItem::from_local_path(format!("track{i}.mp3")))
            .collect();
        engine.load_playlist(tracks);
        engine.set_mode(PlaybackMode::Sequential);

        // Play track 0
        let t0 = engine.next().unwrap().unwrap();
        assert_eq!(t0.title, "track0.mp3");

        // Queue priority track
        let q_item = MediaItem::from_local_path("urgent.mp3");
        engine.queue_add(q_item.clone());
        assert_eq!(engine.queue_len(), 1);

        // Next MUST play queued item first
        let q_played = engine.next().unwrap().unwrap();
        assert_eq!(q_played.title, "urgent.mp3");
        assert_eq!(engine.queue_len(), 0);

        // Next resumes normal playlist at track 1!
        let t1 = engine.next().unwrap().unwrap();
        assert_eq!(t1.title, "track1.mp3");
    }

    #[test]
    fn test_repeat_modes() {
        let mut engine = PlayerEngine::new();
        let tracks = vec![
            MediaItem::from_local_path("a.mp3"),
            MediaItem::from_local_path("b.mp3"),
        ];
        engine.load_playlist(tracks);

        // Repeat One
        engine.set_mode(PlaybackMode::RepeatOne);
        let t1 = engine.next().unwrap().unwrap();
        assert_eq!(t1.title, "a.mp3");
        let t1_repeat = engine.next().unwrap().unwrap();
        assert_eq!(t1_repeat.title, "a.mp3");

        // Repeat Playlist
        engine.set_mode(PlaybackMode::RepeatPlaylist);
        let tb = engine.next().unwrap().unwrap();
        assert_eq!(tb.title, "b.mp3");
        let ta = engine.next().unwrap().unwrap();
        assert_eq!(ta.title, "a.mp3"); // wrapped around!
    }
}
