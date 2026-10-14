//! Terminal User Interface (TUI) client implementation.

pub mod app;
pub mod input;
pub mod ui;

pub use app::{ActiveView, TuiApp};
pub use input::{handle_key_event, map_key_to_command};
pub use ui::{format_now_playing, render_frame, render_lines};

use crate::config::Settings;
use crate::core::{PlayerEngine, PlaylistManager};
use crate::error::{Result, UPlayError};
use crate::sources::LocalResolver;
use crossterm::QueueableCommand;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
    disable_raw_mode, enable_raw_mode,
};
use std::io::{IsTerminal, Write, stdout};
use std::time::Duration;

/// RAII guard to safely manage alternate screen and terminal raw mode.
struct TuiTerminalGuard;

impl TuiTerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()
            .map_err(|e| UPlayError::Input(format!("failed to enable raw mode: {e}")))?;
        let mut out = stdout();
        let _ = out.queue(EnterAlternateScreen);
        let _ = out.queue(Hide);
        let _ = out.queue(DisableLineWrap);
        let _ = out.queue(Clear(ClearType::All));
        let _ = out.flush();
        Ok(Self)
    }
}

impl Drop for TuiTerminalGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = out.queue(EnableLineWrap);
        let _ = out.queue(Show);
        let _ = out.queue(LeaveAlternateScreen);
        let _ = out.flush();
        let _ = disable_raw_mode();
    }
}

/// Run the interactive Terminal User Interface (TUI).
pub fn run() -> Result<()> {
    run_with_engine(None)
}

/// Run the TUI with an optional pre-configured `PlayerEngine`.
pub fn run_with_engine(engine: Option<PlayerEngine>) -> Result<()> {
    if !stdout().is_terminal() || !std::io::stdin().is_terminal() {
        println!("UPlay - A lightweight, Linux-first music player");
        println!("Run 'uplay --help' for available commands.");
        return Ok(());
    }

    let mut engine = engine.unwrap_or_else(PlayerEngine::with_rodio);

    // Initial track discovery:
    // 1. Try saved playlists first
    let playlist_mgr = PlaylistManager::new();
    if engine.playlist_len() == 0 {
        if let Ok(saved) = playlist_mgr.list_saved() {
            if let Some(first) = saved.first() {
                if let Ok(pl) = playlist_mgr.load_from_disk(first) {
                    if !pl.is_empty() {
                        engine.load_playlist(pl.items);
                    }
                }
            }
        }
    }

    // 2. Fall back to settings music directory if playlist still empty
    if engine.playlist_len() == 0 {
        let music_dir = Settings::default().music_directory;
        if music_dir.is_dir() {
            let resolver = LocalResolver::new();
            if let Ok(items) = resolver.scan_directory(&music_dir) {
                engine.load_playlist(items);
            }
        }
    }

    let mut app = TuiApp::new();
    app.update_state(engine.state().clone());

    let _guard = TuiTerminalGuard::enter()?;
    let mut out = stdout();

    loop {
        // Automatic advancement when track completes
        if engine.is_finished() {
            let _ = engine.next();
        }

        app.update_state(engine.state().clone());

        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let lines = render_lines(&app, cols, rows);

        for (r, line) in lines.iter().take(rows as usize).enumerate() {
            let _ = out.queue(MoveTo(0, r as u16));
            let _ = out.write_all(line.as_bytes());
            let _ = out.queue(Clear(ClearType::UntilNewLine));
        }
        let _ = out.flush();

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(&mut app, &mut engine, key.code, key.modifiers)?;
                    }
                }
                Ok(Event::Resize(_, _)) => {
                    let _ = out.queue(Clear(ClearType::All));
                }
                _ => {}
            }
        }
    }

    Ok(())
}
