//! Rodio audio backend implementation for local playback.

use super::backend::AudioBackend;
use crate::core::state::Source;
use crate::error::{Result, UPlayError};
use rodio::source::Source as RodioSource;
use rodio::stream::{DeviceSinkBuilder, MixerDeviceSink};
use rodio::{Decoder, Player};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

/// An audio backend backed by the `rodio` playback library.
pub struct RodioBackend {
    _sink_device: Option<MixerDeviceSink>,
    player: Option<Player>,
    current_source: Option<Source>,
    duration: Option<Duration>,
    volume: u8,
}

impl Default for RodioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RodioBackend {
    /// Create a new `RodioBackend`.
    #[must_use]
    pub fn new() -> Self {
        let mut sink_device = DeviceSinkBuilder::open_default_sink().ok();
        if let Some(sink) = &mut sink_device {
            sink.log_on_drop(false);
        }

        let player = sink_device
            .as_ref()
            .map(|sink| Player::connect_new(sink.mixer()));

        Self {
            _sink_device: sink_device,
            player,
            current_source: None,
            duration: None,
            volume: 80,
        }
    }

    /// Check if audio device was successfully initialized.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.player.is_some()
    }

    /// Check if audio playback is currently paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.player.as_ref().is_some_and(Player::is_paused)
    }

    /// Check if player queue is empty (finished playing).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.player.as_ref().is_some_and(Player::empty)
    }
}

impl AudioBackend for RodioBackend {
    fn load(&mut self, source: &Source) -> Result<()> {
        let path = match source {
            Source::LocalFile(p) => p,
            _ => {
                return Err(UPlayError::Audio(
                    "RodioBackend currently only supports local audio files".into(),
                ));
            }
        };

        if !path.exists() {
            return Err(UPlayError::File {
                path: path.clone(),
                reason: "audio file does not exist".into(),
            });
        }

        let file = File::open(path).map_err(|e| UPlayError::File {
            path: path.clone(),
            reason: e.to_string(),
        })?;

        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| {
            UPlayError::Audio(format!(
                "failed to decode audio file {}: {e}",
                path.display()
            ))
        })?;

        self.duration = decoder.total_duration();

        // If player is not present, attempt to reinitialize
        if self.player.is_none() {
            if let Some(sink) = &self._sink_device {
                self.player = Some(Player::connect_new(sink.mixer()));
            }
        }

        if let Some(player) = &self.player {
            player.clear();
            player.set_volume(f32::from(self.volume) / 100.0);
            player.append(decoder);
            player.pause();
        } else {
            return Err(UPlayError::Audio("No audio output device available".into()));
        }

        self.current_source = Some(source.clone());
        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        if let Some(player) = &self.player {
            player.play();
            Ok(())
        } else {
            Err(UPlayError::Audio(
                "Audio output device not initialized".into(),
            ))
        }
    }

    fn pause(&mut self) -> Result<()> {
        if let Some(player) = &self.player {
            player.pause();
            Ok(())
        } else {
            Err(UPlayError::Audio(
                "Audio output device not initialized".into(),
            ))
        }
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(player) = &self.player {
            player.clear();
            Ok(())
        } else {
            Err(UPlayError::Audio(
                "Audio output device not initialized".into(),
            ))
        }
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        if let Some(player) = &self.player {
            player
                .try_seek(position)
                .map_err(|e| UPlayError::Audio(format!("seek failed: {e}")))?;
            Ok(())
        } else {
            Err(UPlayError::Audio(
                "Audio output device not initialized".into(),
            ))
        }
    }

    fn set_volume(&mut self, volume: u8) -> Result<()> {
        self.volume = volume.min(100);
        if let Some(player) = &self.player {
            player.set_volume(f32::from(self.volume) / 100.0);
        }
        Ok(())
    }

    fn position(&self) -> Duration {
        self.player.as_ref().map_or(Duration::ZERO, Player::get_pos)
    }

    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn is_finished(&self) -> bool {
        self.is_empty()
    }
}
