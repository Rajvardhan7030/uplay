//! Terminal keyboard input mapping and event translation.

use crate::cli::commands::Command;

/// Translates raw character inputs into internal UPlay commands.
#[must_use]
pub fn map_key_to_command(key: char) -> Option<Command> {
    match key {
        ' ' => Some(Command::TogglePlay),
        'n' => Some(Command::Next),
        'p' => Some(Command::Previous),
        'q' => Some(Command::Stop),
        '+' | '=' => Some(Command::VolumeUp),
        '-' | '_' => Some(Command::VolumeDown),
        _ => None,
    }
}

use super::app::{ActiveView, TuiApp};
use crate::core::PlayerEngine;
use crate::error::Result;
use crate::sources::resolve_target;
use crossterm::event::{KeyCode, KeyModifiers};

/// Handle a keyboard event updating the `TuiApp` state and driving `PlayerEngine` actions.
pub fn handle_key_event(
    app: &mut TuiApp,
    engine: &mut PlayerEngine,
    key_code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<()> {
    // Ctrl+C exits immediately
    if modifiers.contains(KeyModifiers::CONTROL) && key_code == KeyCode::Char('c') {
        let _ = engine.stop();
        app.quit();
        return Ok(());
    }

    // 1. Interactive Text Input Modes (Search & Add)
    if app.is_searching {
        match key_code {
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.selected_index = 0;
                app.clamp_selection();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.selected_index = 0;
                app.clamp_selection();
            }
            KeyCode::Enter => {
                app.finish_search();
            }
            KeyCode::Esc => {
                app.cancel_search();
            }
            _ => {}
        }
        return Ok(());
    }

    if app.is_adding {
        match key_code {
            KeyCode::Char(c) => {
                app.add_input.push(c);
            }
            KeyCode::Backspace => {
                app.add_input.pop();
            }
            KeyCode::Enter => {
                let target = app.add_input.trim().to_string();
                if !target.is_empty() {
                    match resolve_target(&target) {
                        Ok(items) => {
                            let count = items.len();
                            for item in items {
                                engine.add_playlist_item(item);
                            }
                            app.update_state(engine.state().clone());
                            app.set_status(format!("Added {count} track(s) from '{target}'"));
                        }
                        Err(e) => {
                            app.set_status(format!("Failed to add: {e}"));
                        }
                    }
                }
                app.cancel_add();
            }
            KeyCode::Esc => {
                app.cancel_add();
            }
            _ => {}
        }
        return Ok(());
    }

    // 2. Normal Navigation & Playback Controls
    match key_code {
        KeyCode::Char(' ') => {
            let _ = engine.toggle_play();
            app.update_state(engine.state().clone());
        }
        KeyCode::Enter => match app.active_view {
            ActiveView::Playlist => {
                let indices = app.filtered_playlist_indices();
                if let Some(&actual_idx) = indices.get(app.selected_index) {
                    let _ = engine.play_track_at(actual_idx);
                    app.update_state(engine.state().clone());
                }
            }
            ActiveView::Queue => {
                let _ = engine.next();
                app.update_state(engine.state().clone());
            }
        },
        KeyCode::Char('n') => {
            let _ = engine.next();
            app.update_state(engine.state().clone());
        }
        KeyCode::Char('p') => {
            let _ = engine.previous();
            app.update_state(engine.state().clone());
        }
        KeyCode::Char('s') => {
            engine.toggle_shuffle();
            let mode = engine.state().playback_mode;
            app.update_state(engine.state().clone());
            app.set_status(format!("Mode changed to: {mode:?}"));
        }
        KeyCode::Char('r') => {
            engine.cycle_repeat();
            let mode = engine.state().playback_mode;
            app.update_state(engine.state().clone());
            app.set_status(format!("Repeat mode: {mode:?}"));
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            let _ = engine.volume_up(5);
            app.update_state(engine.state().clone());
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            let _ = engine.volume_down(5);
            app.update_state(engine.state().clone());
        }
        KeyCode::Left => {
            let _ = engine.seek_relative(-5);
            app.update_state(engine.state().clone());
        }
        KeyCode::Right => {
            let _ = engine.seek_relative(5);
            app.update_state(engine.state().clone());
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_down();
        }
        KeyCode::Tab => {
            app.toggle_active_view();
            let view_name = match app.active_view {
                ActiveView::Playlist => "Playlist",
                ActiveView::Queue => "Queue",
            };
            app.set_status(format!("Switched view to {view_name}"));
        }
        KeyCode::Char('/') => {
            app.start_search();
        }
        KeyCode::Char('a') => {
            app.start_add();
        }
        KeyCode::Char('d') | KeyCode::Delete => match app.active_view {
            ActiveView::Playlist => {
                let indices = app.filtered_playlist_indices();
                if let Some(&actual_idx) = indices.get(app.selected_index) {
                    if let Some(removed) = engine.remove_playlist_item(actual_idx) {
                        app.update_state(engine.state().clone());
                        app.set_status(format!("Removed track '{}'", removed.title));
                    }
                }
            }
            ActiveView::Queue => {
                if let Some(removed) = engine.queue_remove(app.selected_index) {
                    app.update_state(engine.state().clone());
                    app.set_status(format!("Removed queued track '{}'", removed.title));
                }
            }
        },
        KeyCode::Char('e') | KeyCode::Char('Q') => {
            if app.active_view == ActiveView::Playlist {
                let indices = app.filtered_playlist_indices();
                if let Some(&actual_idx) = indices.get(app.selected_index) {
                    if let Some(item) = engine.state().playlist.get(actual_idx).cloned() {
                        let title = item.title.clone();
                        engine.queue_add(item);
                        app.update_state(engine.state().clone());
                        app.set_status(format!("Enqueued '{title}'"));
                    }
                }
            }
        }
        KeyCode::Char('c') => {
            if app.active_view == ActiveView::Queue {
                engine.queue_clear();
                app.update_state(engine.state().clone());
                app.set_status("Cleared priority queue");
            }
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.cancel_search();
                app.set_status("Cleared search filter");
            } else {
                let _ = engine.stop();
                app.quit();
            }
        }
        KeyCode::Char('q') => {
            let _ = engine.stop();
            app.quit();
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_mapping() {
        assert_eq!(map_key_to_command(' '), Some(Command::TogglePlay));
        assert_eq!(map_key_to_command('n'), Some(Command::Next));
        assert_eq!(map_key_to_command('p'), Some(Command::Previous));
        assert_eq!(map_key_to_command('q'), Some(Command::Stop));
        assert_eq!(map_key_to_command('z'), None);
    }
}
