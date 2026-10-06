//! Audio backend traits and abstraction layer.

use crate::core::state::Source;
use crate::error::Result;
use std::time::Duration;

/// Trait defining the playback interface for audio backends.
pub trait AudioBackend: Send + Sync {
    /// Load an audio source ready for playback.
    fn load(&mut self, source: &Source) -> Result<()>;

    /// Begin or resume playback.
    fn play(&mut self) -> Result<()>;

    /// Pause audio playback.
    fn pause(&mut self) -> Result<()>;

    /// Stop playback and release or rewind buffers.
    fn stop(&mut self) -> Result<()>;

    /// Seek to a specified playback position.
    fn seek(&mut self, position: Duration) -> Result<()>;

    /// Set playback volume level (0 to 100).
    fn set_volume(&mut self, volume: u8) -> Result<()>;

    /// Get the current playback elapsed position.
    fn position(&self) -> Duration;

    /// Get total audio duration if known.
    fn duration(&self) -> Option<Duration>;
}

/// A null/mock audio backend for headless testing and initial scaffolding.
#[derive(Debug, Default)]
pub struct NullAudioBackend {
    volume: u8,
    position: Duration,
    duration: Option<Duration>,
    is_playing: bool,
}

impl NullAudioBackend {
    /// Create a new `NullAudioBackend`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            volume: 80,
            position: Duration::ZERO,
            duration: Some(Duration::from_secs(180)),
            is_playing: false,
        }
    }
}

impl AudioBackend for NullAudioBackend {
    fn load(&mut self, _source: &Source) -> Result<()> {
        self.position = Duration::ZERO;
        self.is_playing = false;
        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        self.is_playing = true;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.is_playing = false;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.is_playing = false;
        self.position = Duration::ZERO;
        Ok(())
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        self.position = position;
        Ok(())
    }

    fn set_volume(&mut self, volume: u8) -> Result<()> {
        self.volume = volume.min(100);
        Ok(())
    }

    fn position(&self) -> Duration {
        self.position
    }

    fn duration(&self) -> Option<Duration> {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_backend_lifecycle() {
        let mut backend = NullAudioBackend::new();
        let src = Source::LocalFile("test.mp3".into());
        assert!(backend.load(&src).is_ok());
        assert!(backend.play().is_ok());
        assert!(backend.is_playing);
        assert!(backend.pause().is_ok());
        assert!(!backend.is_playing);
        assert!(backend.seek(Duration::from_secs(30)).is_ok());
        assert_eq!(backend.position(), Duration::from_secs(30));
    }
}
