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
    /// Enqueue a file or URL to the end of the queue.
    Queue(String),
    /// Enqueue a file or URL to play immediately next.
    QueueNext(String),
    /// Clear all items from the queue.
    QueueClear,
    /// Inspect / list all queued tracks.
    QueueList,
    /// Save active or specified tracks to a named persistent playlist.
    PlaylistSave {
        /// Playlist name.
        name: String,
        /// Optional audio tracks, folders, or URLs to save.
        targets: Vec<String>,
    },
    /// Load and play a persistent playlist by name.
    PlaylistLoad {
        /// Playlist name.
        name: String,
    },
    /// List all saved persistent playlists.
    PlaylistList,
    /// Delete a saved persistent playlist by name.
    PlaylistDelete {
        /// Playlist name.
        name: String,
    },
    /// Create a new persistent playlist.
    PlaylistCreate {
        /// Playlist name.
        name: String,
        /// Audio tracks, folders, or URLs to initialize with.
        targets: Vec<String>,
    },
    /// Add audio tracks, folders, or URLs to an existing playlist.
    PlaylistAdd {
        /// Playlist name.
        name: String,
        /// Audio targets to append.
        targets: Vec<String>,
    },
    /// Remove an audio track from a playlist by 1-based index or title/target.
    PlaylistRemove {
        /// Playlist name.
        name: String,
        /// Target title, path, or 1-based index.
        target: String,
    },
    /// Inspect / show tracks within a saved playlist.
    PlaylistShow {
        /// Playlist name.
        name: String,
    },
    /// Launch the interactive Terminal User Interface (TUI).
    Tui,
}
