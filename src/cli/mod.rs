//! Command-line interface definitions, parsing, and execution.

pub mod commands;
pub mod player;

pub use commands::Command;
pub use player::{
    play_items_with_options, play_local_file, play_path, play_path_with_options, play_playlist,
};

use clap::{Parser, Subcommand};

/// UPlay: A lightweight, Linux-first music player and engine.
#[derive(Debug, Parser)]
#[command(name = "uplay", version, about, long_about = None)]
pub struct Cli {
    /// Audio file path, directory, or URL to play.
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Enable shuffle playback mode.
    #[arg(short, long)]
    pub shuffle: bool,

    /// Set repeat mode: off, one, playlist.
    #[arg(short, long, value_name = "MODE")]
    pub repeat: Option<String>,

    /// Resume playback from the last saved session.
    #[arg(long)]
    pub resume: bool,

    /// Read track list or paths from standard input.
    #[arg(long)]
    pub stdin: bool,

    /// Enable verbose / debug logging output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Optional control subcommand.
    #[command(subcommand)]
    pub command: Option<Subcommands>,
}

/// Available subcommands for CLI control.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum Subcommands {
    /// Play a file, directory, URL, or resume playback.
    Play {
        /// Optional audio target.
        target: Option<String>,
    },
    /// Pause active playback.
    Pause,
    /// Resume paused playback.
    Resume,
    /// Stop playback.
    Stop,
    /// Skip to the next track.
    Next,
    /// Return to the previous track.
    Previous,
    /// Set volume level (0..=100).
    Volume {
        /// Volume level percentage.
        level: u8,
    },
    /// Toggle shuffle mode.
    Shuffle,
    /// Set repeat mode (off, one, playlist).
    Repeat {
        /// Repeat mode name.
        mode: String,
    },
    /// Add an item to the playback queue or inspect queue.
    Queue {
        /// Audio path or stream URL to queue.
        target: Option<String>,

        /// Add track to play immediately next.
        #[arg(long)]
        next: bool,

        /// Clear all tracks from queue.
        #[arg(long)]
        clear: bool,

        /// List tracks currently in queue.
        #[arg(long)]
        list: bool,
    },
    /// Manage persistent playlists (save, load, list, delete, add, remove).
    Playlist {
        #[command(subcommand)]
        action: PlaylistAction,
    },
    /// Open the interactive Terminal User Interface (TUI).
    Tui,
    /// Start the background playback daemon.
    Daemon,
}

/// Available actions for playlist management.
#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum PlaylistAction {
    /// Save active or specified tracks to a named persistent playlist.
    Save {
        /// Playlist name.
        name: String,
        /// Optional audio tracks, folders, or URLs to save.
        #[arg(value_name = "TARGETS")]
        targets: Vec<String>,
    },
    /// Load and play a persistent playlist by name.
    Load {
        /// Playlist name.
        name: String,
    },
    /// List all saved persistent playlists.
    List,
    /// Delete a saved persistent playlist by name.
    Delete {
        /// Playlist name.
        name: String,
    },
    /// Create a new persistent playlist.
    Create {
        /// Playlist name.
        name: String,
        /// Audio tracks, folders, or URLs to initialize with.
        #[arg(value_name = "TARGETS")]
        targets: Vec<String>,
    },
    /// Add audio tracks, folders, or URLs to an existing playlist.
    Add {
        /// Playlist name.
        name: String,
        /// Audio targets to append.
        #[arg(value_name = "TARGETS", required = true)]
        targets: Vec<String>,
    },
    /// Remove an audio track from a playlist by 1-based index or title/target.
    Remove {
        /// Playlist name.
        name: String,
        /// Target title, path, or 1-based index.
        target: String,
    },
    /// Inspect / show tracks within a saved playlist.
    Show {
        /// Playlist name.
        name: String,
    },
}

impl Cli {
    /// Convert parsed CLI arguments into the unified internal `Command`.
    #[must_use]
    pub fn to_command(&self) -> Option<Command> {
        if let Some(sub) = &self.command {
            match sub {
                Subcommands::Play { target } => Some(Command::Play {
                    target: target.clone(),
                }),
                Subcommands::Pause => Some(Command::Pause),
                Subcommands::Resume => Some(Command::Resume),
                Subcommands::Stop => Some(Command::Stop),
                Subcommands::Next => Some(Command::Next),
                Subcommands::Previous => Some(Command::Previous),
                Subcommands::Volume { level } => Some(Command::Volume(*level)),
                Subcommands::Shuffle => Some(Command::Shuffle),
                Subcommands::Repeat { mode } => Some(Command::Repeat(mode.clone())),
                Subcommands::Queue {
                    target,
                    next,
                    clear,
                    list,
                } => {
                    if *clear {
                        Some(Command::QueueClear)
                    } else if *list {
                        Some(Command::QueueList)
                    } else if let Some(target) = target {
                        if *next {
                            Some(Command::QueueNext(target.clone()))
                        } else {
                            Some(Command::Queue(target.clone()))
                        }
                    } else {
                        Some(Command::QueueList)
                    }
                }
                Subcommands::Playlist { action } => match action {
                    PlaylistAction::Save { name, targets } => Some(Command::PlaylistSave {
                        name: name.clone(),
                        targets: targets.clone(),
                    }),
                    PlaylistAction::Load { name } => {
                        Some(Command::PlaylistLoad { name: name.clone() })
                    }
                    PlaylistAction::List => Some(Command::PlaylistList),
                    PlaylistAction::Delete { name } => {
                        Some(Command::PlaylistDelete { name: name.clone() })
                    }
                    PlaylistAction::Create { name, targets } => Some(Command::PlaylistCreate {
                        name: name.clone(),
                        targets: targets.clone(),
                    }),
                    PlaylistAction::Add { name, targets } => Some(Command::PlaylistAdd {
                        name: name.clone(),
                        targets: targets.clone(),
                    }),
                    PlaylistAction::Remove { name, target } => Some(Command::PlaylistRemove {
                        name: name.clone(),
                        target: target.clone(),
                    }),
                    PlaylistAction::Show { name } => {
                        Some(Command::PlaylistShow { name: name.clone() })
                    }
                },
                Subcommands::Tui => Some(Command::Tui),
                Subcommands::Daemon => None,
            }
        } else {
            self.target.as_ref().map(|target| Command::Play {
                target: Some(target.clone()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_positional_target() {
        let cli = Cli::parse_from(["uplay", "song.flac"]);
        assert_eq!(cli.target.as_deref(), Some("song.flac"));
        assert_eq!(
            cli.to_command(),
            Some(Command::Play {
                target: Some("song.flac".into())
            })
        );
    }

    #[test]
    fn test_cli_subcommands() {
        let cli = Cli::parse_from(["uplay", "next"]);
        assert_eq!(cli.to_command(), Some(Command::Next));

        let cli = Cli::parse_from(["uplay", "volume", "75"]);
        assert_eq!(cli.to_command(), Some(Command::Volume(75)));

        let cli = Cli::parse_from(["uplay", "queue", "urgent.flac", "--next"]);
        assert_eq!(
            cli.to_command(),
            Some(Command::QueueNext("urgent.flac".into()))
        );

        let cli = Cli::parse_from(["uplay", "queue", "--clear"]);
        assert_eq!(cli.to_command(), Some(Command::QueueClear));
    }

    #[test]
    fn test_cli_playlist_subcommands() {
        let cli = Cli::parse_from([
            "uplay",
            "playlist",
            "save",
            "coding",
            "song1.mp3",
            "song2.mp3",
        ]);
        assert_eq!(
            cli.to_command(),
            Some(Command::PlaylistSave {
                name: "coding".into(),
                targets: vec!["song1.mp3".into(), "song2.mp3".into()],
            })
        );

        let cli = Cli::parse_from(["uplay", "playlist", "load", "coding"]);
        assert_eq!(
            cli.to_command(),
            Some(Command::PlaylistLoad {
                name: "coding".into(),
            })
        );

        let cli = Cli::parse_from(["uplay", "playlist", "list"]);
        assert_eq!(cli.to_command(), Some(Command::PlaylistList));

        let cli = Cli::parse_from(["uplay", "playlist", "delete", "coding"]);
        assert_eq!(
            cli.to_command(),
            Some(Command::PlaylistDelete {
                name: "coding".into(),
            })
        );

        let cli = Cli::parse_from(["uplay", "playlist", "add", "coding", "song3.flac"]);
        assert_eq!(
            cli.to_command(),
            Some(Command::PlaylistAdd {
                name: "coding".into(),
                targets: vec!["song3.flac".into()],
            })
        );

        let cli = Cli::parse_from(["uplay", "playlist", "remove", "coding", "1"]);
        assert_eq!(
            cli.to_command(),
            Some(Command::PlaylistRemove {
                name: "coding".into(),
                target: "1".into(),
            })
        );
    }
}
