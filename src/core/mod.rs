//! Core player engine, queue, playlist, and state management.

pub mod player;
pub mod playlist;
pub mod queue;
pub mod session;
pub mod state;

pub use player::PlayerEngine;
pub use playlist::{Playlist, PlaylistManager};
pub use queue::QueueManager;
pub use session::Session;
pub use state::{MediaItem, PlaybackMode, PlayerState, PlayerStatus, Source};
