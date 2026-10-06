//! Audio stream parameters and streaming metadata.

/// Format metadata for an active audio stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamMetadata {
    /// Sample rate in Hertz (e.g. 44100, 48000).
    pub sample_rate: u32,
    /// Channel count (e.g. 1 for mono, 2 for stereo).
    pub channels: u16,
    /// Format container or codec name (e.g. "flac", "mp3", "opus").
    pub codec: String,
}

impl Default for StreamMetadata {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            codec: "unknown".to_string(),
        }
    }
}
