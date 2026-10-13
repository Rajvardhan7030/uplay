//! UPlay binary entry point.

use clap::Parser;
use std::path::Path;
use std::process::ExitCode;
use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;
use uplay::cli::{Cli, Command, play_path_with_options, play_playlist};
use uplay::core::{Playlist, PlaylistManager};
use uplay::error::{Result, UPlayError};
use uplay::sources::{resolve_target, resolve_targets};

fn init_logging(verbose: bool) {
    let max_level = if verbose { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(max_level)
        .with_target(false)
        .without_time()
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn run(cli: Cli) -> Result<()> {
    debug!("Parsed CLI options: {:?}", cli);

    match cli.to_command() {
        Some(Command::Play { target }) => {
            if let Some(target) = target {
                let path = Path::new(&target);
                play_path_with_options(path, cli.shuffle, cli.repeat.as_deref())?;
            } else {
                info!("Request to resume playback");
            }
        }
        Some(Command::Pause) => {
            info!("Playback paused");
        }
        Some(Command::Resume) => {
            info!("Playback resumed");
        }
        Some(Command::TogglePlay) => {
            info!("Playback toggled");
        }
        Some(Command::Stop) => {
            info!("Playback stopped");
        }
        Some(Command::Next) => {
            info!("Advanced to next track");
        }
        Some(Command::Previous) => {
            info!("Returned to previous track");
        }
        Some(Command::Volume(level)) => {
            info!("Volume set to {}%", level.min(100));
        }
        Some(Command::VolumeUp) => {
            info!("Volume increased");
        }
        Some(Command::VolumeDown) => {
            info!("Volume decreased");
        }
        Some(Command::Shuffle) => {
            info!("Toggled shuffle mode");
        }
        Some(Command::Repeat(mode)) => {
            info!("Set repeat mode to {}", mode);
        }
        Some(Command::Queue(target)) => {
            info!("Queued target: {}", target);
        }
        Some(Command::QueueNext(target)) => {
            info!("Queued next target: {}", target);
        }
        Some(Command::QueueClear) => {
            info!("Cleared playback queue");
        }
        Some(Command::QueueList) => {
            info!("Playback queue inspected (empty)");
        }
        Some(Command::PlaylistSave { name, targets }) => {
            let items = if targets.is_empty() {
                if let Some(target) = &cli.target {
                    resolve_target(target)?
                } else {
                    Vec::new()
                }
            } else {
                resolve_targets(&targets)?
            };
            let mut playlist = Playlist::new(&name);
            playlist.items = items;
            let manager = PlaylistManager::new();
            manager.save_to_disk(&playlist)?;
            if playlist.is_empty() {
                println!("Saved empty playlist '{name}'.");
            } else {
                println!("Saved playlist '{name}' ({} tracks).", playlist.len());
            }
        }
        Some(Command::PlaylistLoad { name }) => {
            let manager = PlaylistManager::new();
            let playlist = manager.load_from_disk(&name)?;
            if playlist.is_empty() {
                println!("Playlist '{name}' is empty.");
            } else {
                println!("Loaded playlist '{name}' with {} tracks.", playlist.len());
                play_playlist(&playlist, cli.shuffle, cli.repeat.as_deref())?;
            }
        }
        Some(Command::PlaylistList) => {
            let manager = PlaylistManager::new();
            let saved = manager.list_saved()?;
            if saved.is_empty() {
                println!("No saved playlists found.");
            } else {
                println!("Available playlists:");
                for pl_name in saved {
                    if let Ok(pl) = manager.load_from_disk(&pl_name) {
                        println!("  {} ({} tracks)", pl_name, pl.len());
                    } else {
                        println!("  {pl_name}");
                    }
                }
            }
        }
        Some(Command::PlaylistDelete { name }) => {
            let manager = PlaylistManager::new();
            manager.delete_saved(&name)?;
            println!("Deleted playlist '{name}'.");
        }
        Some(Command::PlaylistCreate { name, targets }) => {
            let items = if targets.is_empty() {
                Vec::new()
            } else {
                resolve_targets(&targets)?
            };
            let mut playlist = Playlist::new(&name);
            playlist.items = items;
            let manager = PlaylistManager::new();
            manager.save_to_disk(&playlist)?;
            println!("Created playlist '{name}' ({} tracks).", playlist.len());
        }
        Some(Command::PlaylistAdd { name, targets }) => {
            let manager = PlaylistManager::new();
            let mut playlist = manager
                .load_from_disk(&name)
                .unwrap_or_else(|_| Playlist::new(&name));
            let new_items = resolve_targets(&targets)?;
            let added_count = new_items.len();
            for item in new_items {
                playlist.add(item);
            }
            manager.save_to_disk(&playlist)?;
            println!(
                "Added {added_count} track(s) to playlist '{name}' (total: {}).",
                playlist.len()
            );
        }
        Some(Command::PlaylistRemove { name, target }) => {
            let manager = PlaylistManager::new();
            let mut playlist = manager.load_from_disk(&name)?;
            if let Some(removed) = playlist.remove_by_target(&target) {
                manager.save_to_disk(&playlist)?;
                println!(
                    "Removed track '{}' from playlist '{name}' (remaining: {}).",
                    removed.title,
                    playlist.len()
                );
            } else {
                return Err(UPlayError::Input(format!(
                    "track '{target}' not found in playlist '{name}'"
                )));
            }
        }
        Some(Command::PlaylistShow { name }) => {
            let manager = PlaylistManager::new();
            let playlist = manager.load_from_disk(&name)?;
            println!("Playlist '{}' ({} tracks):", playlist.name, playlist.len());
            for (idx, item) in playlist.items.iter().enumerate() {
                println!("  {}. {}", idx + 1, item.title);
            }
        }
        None => {
            // No command given; in future phases this opens the interactive TUI
            println!("UPlay - A lightweight, Linux-first music player");
            println!("Run 'uplay --help' for available commands.");
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
