//! Command-line interface definitions, parsing, and execution.

pub mod commands;
pub mod player;

pub use commands::Command;
pub use player::{play_local_file, play_path, play_path_with_options};

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
    /// Start the background playback daemon.
    Daemon,
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
}
