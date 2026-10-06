//! Internal command representation shared between CLI, TUI, and IPC interfaces.

/// Internal command representation that unifies user intent across all frontends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Play a track, URL, or resume current playback.
    Play {
        /// Optional target path or stream URL.
        target: Option<String>,
    },
    /// Pause active playback.
    Pause,
    /// Resume paused playback.
    Resume,
    /// Toggle play / pause state.
    TogglePlay,
    /// Stop playback completely.
    Stop,
    /// Skip to the next track.
    Next,
    /// Go back to the previous track.
    Previous,
    /// Set explicit volume percentage (0..=100).
    Volume(u8),
    /// Increase volume by standard step.
    VolumeUp,
    /// Decrease volume by standard step.
    VolumeDown,
    /// Toggle shuffle mode.
    Shuffle,
    /// Set repeat mode ("off", "one", "playlist").
    Repeat(String),
    /// Enqueue a file or URL.
    Queue(String),
}
