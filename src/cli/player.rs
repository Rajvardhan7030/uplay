//! Interactive terminal player for local audio playback.

use crate::core::PlayerEngine;
use crate::core::state::PlayerStatus;
use crate::error::{Result, UPlayError};
use crate::sources::LocalResolver;
use crossterm::QueueableCommand;
use crossterm::cursor::{Hide, MoveToColumn, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
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

/// Play a local audio file interactively in the terminal.
pub fn play_local_file(path: &Path) -> Result<()> {
    let resolver = LocalResolver::new();
    let item = resolver.resolve_file(path)?;

    println!("Playing: {}", item.title);
    println!("Controls: [Space] Pause/Play  [←/→] Seek 5s  [+/-] Vol  [s] Stop  [q] Quit\n");

    let mut engine = PlayerEngine::with_rodio();
    engine.load_and_play(item)?;

    let is_interactive = stdout().is_terminal() && std::io::stdin().is_terminal();

    if is_interactive {
        let _guard = TerminalGuard::enter()?;
        let mut stdout_handle = stdout();

        loop {
            let pos = engine.position();
            let total = engine.duration();
            let state = engine.state();

            let status_str = match state.status {
                PlayerStatus::Playing => "Playing",
                PlayerStatus::Paused => "Paused ",
                PlayerStatus::Stopped => "Stopped",
            };

            let progress = format_progress_bar(pos, total, 20);
            let pos_str = format_duration(pos);
            let total_str = total.map_or("--:--".to_string(), format_duration);

            let status_line = format!(
                "\r{} {} / {} [{}] Vol: {:3}%",
                progress, pos_str, total_str, status_str, state.volume
            );

            let _ = stdout_handle.queue(MoveToColumn(0));
            let _ = write!(stdout_handle, "{status_line}");
            let _ = stdout_handle.flush();

            if engine.is_finished() || state.status == PlayerStatus::Stopped {
                break;
            }

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
                            KeyCode::Char('s') => {
                                let _ = engine.stop();
                                break;
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
        // Non-interactive fallback (e.g. piped input or CI)
        while !engine.is_finished() && engine.state().status == PlayerStatus::Playing {
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    Ok(())
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
        assert_eq!(bar.len(), 12); // '[' + 10 chars + ']'
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
    }
}
