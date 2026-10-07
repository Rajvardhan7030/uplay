//! Interactive terminal player for local audio playback (files and folders).

use crate::core::PlayerEngine;
use crate::core::state::{PlaybackMode, PlayerStatus};
use crate::error::{Result, UPlayError};
use crate::sources::LocalResolver;
use crossterm::QueueableCommand;
use crossterm::cursor::{Hide, MoveToColumn, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use std::io::{IsTerminal, Write, stdout};
use std::path::Path;
use std::time::Duration;

/// RAII guard to safely restore terminal mode when exiting playback.
struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    fn enter() -> Result<Self> {
        if stdout().is_terminal() {
            enable_raw_mode()
                .map_err(|e| UPlayError::Input(format!("failed to enable raw mode: {e}")))?;
            let mut out = stdout();
            let _ = out.queue(Hide);
            let _ = out.flush();
            Ok(Self { active: true })
        } else {
            Ok(Self { active: false })
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active {
            let mut out = stdout();
            let _ = out.queue(Show);
            let _ = out.flush();
            let _ = disable_raw_mode();
            println!();
        }
    }
}

/// Format duration into MM:SS or HH:MM:SS.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours:02}:{mins:02}:{secs:02}")
    } else {
        format!("{mins:02}:{secs:02}")
    }
}

/// Create a text progress bar with given width.
#[must_use]
pub fn format_progress_bar(pos: Duration, total: Option<Duration>, width: usize) -> String {
    let width = width.max(5);
    match total {
        Some(tot) if tot.as_secs() > 0 => {
            let ratio = (pos.as_secs_f64() / tot.as_secs_f64()).clamp(0.0, 1.0);
            let filled = (ratio * width as f64).round() as usize;
            let arrow = if filled < width && filled > 0 {
                ">"
            } else {
                "="
            };
            let head = filled.saturating_sub(1);
            let empty = width.saturating_sub(filled);
            format!(
                "[{}{}{}]",
                "=".repeat(head),
                if filled > 0 { arrow } else { "" },
                " ".repeat(empty)
            )
        }
        _ => format!("[{}]", "-".repeat(width)),
    }
}

/// Play a local target with optional shuffle and repeat modes.
pub fn play_path_with_options(path: &Path, shuffle: bool, repeat: Option<&str>) -> Result<()> {
    let resolver = LocalResolver::new();
    let items = resolver.resolve_path(path)?;

    if items.len() == 1 {
        println!("Playing: {}", items[0].title);
        println!(
            "Controls: [Space] Pause/Play  [s] Shuffle  [r] Repeat  [←/→] Seek 5s  [+/-] Vol  [q] Quit\n"
        );
    } else {
        println!("Loaded {} tracks from: {}", items.len(), path.display());
        println!(
            "Controls: [Space] Pause/Play  [n] Next  [p] Prev  [s] Shuffle  [r] Repeat  [←/→] Seek 5s  [+/-] Vol  [q] Quit\n"
        );
    }

    let mut engine = PlayerEngine::with_rodio();
    let total_tracks = items.len();
    engine.load_playlist(items);

    if shuffle {
        engine.set_mode(PlaybackMode::Shuffle);
    } else if let Some(rep) = repeat {
        match rep.to_ascii_lowercase().as_str() {
            "one" | "track" => engine.set_mode(PlaybackMode::RepeatOne),
            "playlist" | "all" => engine.set_mode(PlaybackMode::RepeatPlaylist),
            _ => {}
        }
    }

    engine.next()?;

    let is_interactive = stdout().is_terminal() && std::io::stdin().is_terminal();

    if is_interactive {
        let _guard = TerminalGuard::enter()?;
        let mut stdout_handle = stdout();
        let mut last_displayed_index = None;

        loop {
            // Automatic advancement when current track finishes
            if engine.is_finished() {
                match engine.next()? {
                    Some(_) => {
                        continue;
                    }
                    None => {
                        // End of playlist reached
                        break;
                    }
                }
            }

            let state = engine.state();
            if state.status == PlayerStatus::Stopped {
                break;
            }

            let current_idx = engine.current_track_index().unwrap_or(0);
            if last_displayed_index != Some(current_idx) {
                last_displayed_index = Some(current_idx);
                let current_title = state
                    .current_item
                    .as_ref()
                    .map_or("Unknown Track", |i| i.title.as_str());

                let _ = stdout_handle.queue(MoveToColumn(0));
                let _ = stdout_handle.queue(Clear(ClearType::CurrentLine));
                if total_tracks > 1 {
                    let _ = writeln!(
                        stdout_handle,
                        "\r▶ [Track {}/{}] {}",
                        current_idx + 1,
                        total_tracks,
                        current_title
                    );
                } else {
                    let _ = writeln!(stdout_handle, "\r▶ {current_title}");
                }
            }

            let pos = engine.position();
            let total = engine.duration();

            let status_str = match state.status {
                PlayerStatus::Playing => "Playing",
                PlayerStatus::Paused => "Paused ",
                PlayerStatus::Stopped => "Stopped",
            };

            let mode_tag = match state.playback_mode {
                PlaybackMode::Sequential => "",
                PlaybackMode::Shuffle => " [🔀 Shuffle]",
                PlaybackMode::RepeatOne => " [🔂 Repeat 1]",
                PlaybackMode::RepeatPlaylist => " [🔁 Repeat All]",
            };

            let queue_tag = if engine.queue_len() > 0 {
                format!(" [Queue: {}]", engine.queue_len())
            } else {
                String::new()
            };

            let progress = format_progress_bar(pos, total, 20);
            let pos_str = format_duration(pos);
            let total_str = total.map_or("--:--".to_string(), format_duration);

            let status_line = format!(
                "\r{} {} / {} [{}]{}{} Vol: {:3}%",
                progress, pos_str, total_str, status_str, mode_tag, queue_tag, state.volume
            );

            let _ = stdout_handle.queue(MoveToColumn(0));
            let _ = write!(stdout_handle, "{status_line}");
            let _ = stdout_handle.flush();

            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('c')
                        {
                            let _ = engine.stop();
                            break;
                        }

                        match key.code {
                            KeyCode::Char(' ') => {
                                let _ = engine.toggle_play();
                            }
                            KeyCode::Char('n') => {
                                if engine.next()?.is_none() {
                                    break;
                                }
                            }
                            KeyCode::Char('p') => {
                                let _ = engine.previous();
                            }
                            KeyCode::Char('s') => {
                                engine.toggle_shuffle();
                            }
                            KeyCode::Char('r') => {
                                engine.cycle_repeat();
                            }
                            KeyCode::Left => {
                                let _ = engine.seek_relative(-5);
                            }
                            KeyCode::Right => {
                                let _ = engine.seek_relative(5);
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                let _ = engine.volume_up(5);
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                let _ = engine.volume_down(5);
                            }
                            KeyCode::Char('q') | KeyCode::Esc => {
                                let _ = engine.stop();
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    } else {
        // Non-interactive fallback (e.g. piped input or CI test)
        while engine.state().status == PlayerStatus::Playing {
            if engine.is_finished() && engine.next()?.is_none() {
                break;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    Ok(())
}

/// Play a local target (audio file or entire directory) interactively.
pub fn play_path(path: &Path) -> Result<()> {
    play_path_with_options(path, false, None)
}

/// Backwards-compatible alias for single file playback.
pub fn play_local_file(path: &Path) -> Result<()> {
    play_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(45)), "00:45");
        assert_eq!(format_duration(Duration::from_secs(125)), "02:05");
        assert_eq!(format_duration(Duration::from_secs(3665)), "01:01:05");
    }

    #[test]
    fn test_format_progress_bar() {
        let bar = format_progress_bar(Duration::from_secs(50), Some(Duration::from_secs(100)), 10);
        assert_eq!(bar.len(), 12);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
    }
}
