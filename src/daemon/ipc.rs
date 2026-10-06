//! IPC protocol types and socket messaging.

/// Client-to-daemon command messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommand {
    /// Start or resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Stop playback.
    Stop,
    /// Advance to the next track.
    Next,
    /// Go back to the previous track.
    Previous,
    /// Adjust volume level.
    SetVolume(u8),
    /// Query current player status.
    GetStatus,
}

/// Daemon-to-client response or broadcast event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcResponse {
    /// Operation succeeded.
    Ok,
    /// Current state update event.
    StateChanged {
        /// Playing status string.
        status: String,
        /// Current track title if any.
        track: Option<String>,
        /// Volume level.
        volume: u8,
    },
    /// An error occurred on the daemon.
    Error(String),
}
