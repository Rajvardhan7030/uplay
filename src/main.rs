//! UPlay binary entry point.

use clap::Parser;
use std::path::Path;
use std::process::ExitCode;
use tracing::{Level, debug, info};
use tracing_subscriber::FmtSubscriber;
use uplay::cli::{Cli, Command, play_path};
use uplay::error::Result;

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
                play_path(path)?;
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
