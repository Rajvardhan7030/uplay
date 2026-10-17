//! TUI rendering interfaces and layout components.

use super::app::{ActiveView, TuiApp};
use crate::core::state::{PlaybackMode, PlayerStatus};
use std::time::Duration;

/// Formats the current track info for terminal display.
#[must_use]
pub fn format_now_playing(app: &TuiApp) -> String {
    match &app.player_state.current_item {
        Some(item) => format!(
            "Now Playing: {} - {:?}",
            item.title, app.player_state.status
        ),
        None => "Playback stopped".to_string(),
    }
}

fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let rem_secs = secs % 60;

    if hours > 0 {
        format!("{hours:02}:{mins:02}:{rem_secs:02}")
    } else {
        format!("{mins:02}:{rem_secs:02}")
    }
}

fn format_bar(pos: Duration, total: Option<Duration>, width: usize) -> String {
    let width = width.max(5);
    match total {
        Some(tot) if tot.as_secs_f64() > 0.0 => {
            let ratio = (pos.as_secs_f64() / tot.as_secs_f64()).clamp(0.0, 1.0);
            let filled = (ratio * width as f64).round() as usize;
            let head = filled.saturating_sub(1);
            let empty = width.saturating_sub(filled);
            let arrow = if filled < width && filled > 0 {
                ">"
            } else if filled == width {
                "="
            } else {
                ""
            };
            format!("[{}{}{}]", "=".repeat(head), arrow, " ".repeat(empty))
        }
        _ => format!("[{}]", "-".repeat(width)),
    }
}

/// Helper to truncate or pad a string to exact visual width.
#[must_use]
pub fn pad_or_truncate(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count > width {
        s.chars().take(width).collect()
    } else {
        format!("{}{}", s, " ".repeat(width - char_count))
    }
}

/// Helper to truncate or pad a string using a custom fill character.
#[must_use]
pub fn pad_or_truncate_fill(s: &str, width: usize, fill: char) -> String {
    let char_count = s.chars().count();
    if char_count > width {
        s.chars().take(width).collect()
    } else {
        let mut res = s.to_string();
        for _ in 0..(width - char_count) {
            res.push(fill);
        }
        res
    }
}

/// Render the TUI layout into individual display rows for the specified dimensions.
///
/// Every line is formatted to exactly `width` characters, and the total lines generated
/// will strictly equal `height` rows (adapting cleanly to viewports as small as 20x6).
#[must_use]
pub fn render_lines(app: &TuiApp, width: u16, height: u16) -> Vec<String> {
    let width = (width as usize).max(20);
    let height = (height as usize).max(6);
    let inner_width = width.saturating_sub(2);

    let mut lines = Vec::with_capacity(height);

    // 1. Top border
    lines.push(format!("┌{}┐", "─".repeat(inner_width)));

    // 2. Title & Status Bar
    let status_str = match app.player_state.status {
        PlayerStatus::Playing => "▶ Playing",
        PlayerStatus::Paused => "⏸ Paused ",
        PlayerStatus::Stopped => "■ Stopped",
    };
    let mode_str = match app.player_state.playback_mode {
        PlaybackMode::Sequential => "Sequential",
        PlaybackMode::Shuffle => "Shuffle",
        PlaybackMode::RepeatOne => "Repeat 1",
        PlaybackMode::RepeatPlaylist => "Repeat All",
    };
    let status_line = format!(
        " UPlay v0.1.0 │ Status: {} │ Mode: {} │ Volume: {:3}%",
        status_str, mode_str, app.player_state.volume
    );
    lines.push(format!("│{}│", pad_or_truncate(&status_line, inner_width)));

    // 3. Now Playing & Progress Bar (Compact single line)
    let pos_str = format_duration(app.player_state.position);
    let total_str = app
        .player_state
        .current_item
        .as_ref()
        .and_then(|i| i.duration)
        .map_or("--:--".to_string(), format_duration);
    let time_str = format!("{pos_str} / {total_str}");

    let bar_width = (inner_width / 4).clamp(6, 20);
    let bar = format_bar(
        app.player_state.position,
        app.player_state
            .current_item
            .as_ref()
            .and_then(|i| i.duration),
        bar_width,
    );
    let right_part = format!("{bar} {time_str} ");
    let right_len = right_part.chars().count();

    let track_info = match &app.player_state.current_item {
        Some(item) => {
            let artist_info = item
                .artist
                .as_deref()
                .map(|a| format!(" ({a})"))
                .unwrap_or_default();
            format!(" Now Playing: {}{}", item.title, artist_info)
        }
        None => " Now Playing: (No track loaded)".to_string(),
    };

    let left_avail = inner_width.saturating_sub(right_len);
    let now_playing_line = if left_avail >= 15 {
        format!("{}{}", pad_or_truncate(&track_info, left_avail), right_part)
    } else {
        pad_or_truncate(&track_info, inner_width)
    };
    lines.push(format!(
        "│{}│",
        pad_or_truncate(&now_playing_line, inner_width)
    ));

    // Handle micro-height terminals (height < 8)
    if height < 8 {
        let filtered_indices = app.filtered_playlist_indices();
        let list_rows = height.saturating_sub(5);
        for row in 0..list_rows {
            let pl_idx_in_filtered = app.playlist_scroll + row;
            let content = if pl_idx_in_filtered < filtered_indices.len() {
                let actual_idx = filtered_indices[pl_idx_in_filtered];
                let item = &app.player_state.playlist[actual_idx];
                format!(" > {}. {}", actual_idx + 1, item.title)
            } else {
                String::new()
            };
            lines.push(format!("│{}│", pad_or_truncate(&content, inner_width)));
        }

        let prompt_line = if app.is_searching {
            format!(" Search: {}█", app.search_query)
        } else if app.is_adding {
            format!(" Add: {}█", app.add_input)
        } else {
            " [Space] Play [q] Quit".to_string()
        };
        lines.push(format!("│{}│", pad_or_truncate(&prompt_line, inner_width)));
        lines.push(format!("└{}┘", "─".repeat(inner_width)));
        return lines;
    }

    // 4. Section Header / Divider
    let show_queue = !app.player_state.queue.is_empty() || app.active_view == ActiveView::Queue;
    let is_dual_column = inner_width >= 56 && show_queue;
    let (left_col_w, right_col_w) = if is_dual_column {
        let l = (inner_width * 6) / 10;
        let r = inner_width.saturating_sub(l + 1);
        (l, r)
    } else {
        (inner_width, 0)
    };

    let pl_active_tag = if app.active_view == ActiveView::Playlist {
        "[*] "
    } else {
        "    "
    };
    let q_active_tag = if app.active_view == ActiveView::Queue {
        "[*] "
    } else {
        "    "
    };

    let pl_header = format!(
        "{}[1] Playlist ({})",
        pl_active_tag,
        app.player_state.playlist.len()
    );
    let q_header = format!(
        "{}[2] Queue ({})",
        q_active_tag,
        app.player_state.queue.len()
    );

    if is_dual_column {
        let left_header_fill = pad_or_truncate_fill(&format!("─ {pl_header} "), left_col_w, '─');
        let right_header_fill = pad_or_truncate_fill(&format!("─ {q_header} "), right_col_w, '─');
        lines.push(format!("├{}┬{}┤", left_header_fill, right_header_fill));
    } else {
        let tab_str = if app.active_view == ActiveView::Playlist {
            format!(
                "─ [*] [1] Playlist ({}) ── [2] Queue ({}) [Tab] ",
                app.player_state.playlist.len(),
                app.player_state.queue.len()
            )
        } else {
            format!(
                "─ [1] Playlist ({}) ── [*] [2] Queue ({}) [Tab] ",
                app.player_state.playlist.len(),
                app.player_state.queue.len()
            )
        };
        lines.push(format!(
            "├{}┤",
            pad_or_truncate_fill(&tab_str, inner_width, '─')
        ));
    }

    // 5. Content Rows
    let is_compact_height = height < 18;
    let static_lines = if is_compact_height { 7 } else { 9 };
    let list_rows = height.saturating_sub(static_lines);

    let filtered_indices = app.filtered_playlist_indices();

    for row in 0..list_rows {
        let pl_idx_in_filtered = app.playlist_scroll + row;
        let left_content = if pl_idx_in_filtered < filtered_indices.len() {
            let actual_idx = filtered_indices[pl_idx_in_filtered];
            let item = &app.player_state.playlist[actual_idx];
            let is_selected =
                app.active_view == ActiveView::Playlist && app.selected_index == pl_idx_in_filtered;
            let is_playing = app
                .player_state
                .current_item
                .as_ref()
                .is_some_and(|cur| cur.id == item.id);

            let prefix = match (is_selected, is_playing) {
                (true, true) => "▶> ",
                (true, false) => " > ",
                (false, true) => "▶  ",
                (false, false) => "   ",
            };

            let artist_info = item
                .artist
                .as_deref()
                .map(|a| format!(" ({a})"))
                .unwrap_or_default();
            format!(
                "{}{}. {}{}",
                prefix,
                actual_idx + 1,
                item.title,
                artist_info
            )
        } else if app.player_state.playlist.is_empty() && row == 0 {
            "   (Playlist is empty - press 'a' to add)".to_string()
        } else if !app.search_query.is_empty() && filtered_indices.is_empty() && row == 0 {
            format!("   (No matches for '{}')", app.search_query)
        } else {
            String::new()
        };

        if is_dual_column {
            let q_idx = app.queue_scroll + row;
            let right_content = if q_idx < app.player_state.queue.len() {
                let item = &app.player_state.queue[q_idx];
                let is_selected =
                    app.active_view == ActiveView::Queue && app.selected_index == q_idx;
                let prefix = if is_selected { " > " } else { "   " };
                let artist_info = item
                    .artist
                    .as_deref()
                    .map(|a| format!(" ({a})"))
                    .unwrap_or_default();
                format!("{}{}. {}{}", prefix, q_idx + 1, item.title, artist_info)
            } else if app.player_state.queue.is_empty() && row == 0 {
                "   (Queue is empty)".to_string()
            } else {
                String::new()
            };

            lines.push(format!(
                "│{}│{}│",
                pad_or_truncate(&left_content, left_col_w),
                pad_or_truncate(&right_content, right_col_w)
            ));
        } else {
            let content = match app.active_view {
                ActiveView::Playlist => left_content,
                ActiveView::Queue => {
                    let q_idx = app.queue_scroll + row;
                    if q_idx < app.player_state.queue.len() {
                        let item = &app.player_state.queue[q_idx];
                        let is_selected = app.selected_index == q_idx;
                        let prefix = if is_selected { " > " } else { "   " };
                        let artist_info = item
                            .artist
                            .as_deref()
                            .map(|a| format!(" ({a})"))
                            .unwrap_or_default();
                        format!("{}{}. {}{}", prefix, q_idx + 1, item.title, artist_info)
                    } else if app.player_state.queue.is_empty() && row == 0 {
                        "   (Queue is empty - press 'e' on a playlist track to enqueue)".to_string()
                    } else {
                        String::new()
                    }
                }
            };
            lines.push(format!("│{}│", pad_or_truncate(&content, inner_width)));
        }
    }

    // 6. Prompt / Search / Info Bar
    lines.push(format!("├{}┤", "─".repeat(inner_width)));
    let prompt_line = if app.is_searching {
        format!(
            " Search: {}█ (Enter to apply, Esc to cancel)",
            app.search_query
        )
    } else if app.is_adding {
        format!(
            " Add Path/URL/Playlist: {}█ (Enter to add, Esc to cancel)",
            app.add_input
        )
    } else if let Some(msg) = &app.status_message {
        format!(" Message: {msg}")
    } else if !app.search_query.is_empty() {
        format!(
            " Filter: '{}' (press '/' to change, Esc to clear)",
            app.search_query
        )
    } else {
        let view_name = match app.active_view {
            ActiveView::Playlist => "Playlist",
            ActiveView::Queue => "Queue",
        };
        if show_queue {
            format!(
                " Focus: {view_name}  [Tab] Switch focus  [/] Search  [a] Add track  [d] Remove  [c] Clear Queue"
            )
        } else {
            " Focus: Playlist  [e] Enqueue  [Tab] View Queue  [/] Search  [a] Add track  [d] Remove"
                .to_string()
        }
    };
    lines.push(format!("│{}│", pad_or_truncate(&prompt_line, inner_width)));

    // 7. Footer / Keybindings / Bottom border
    if is_compact_height {
        let footer_keys = " [Space] Play [Enter] Sel [n/p] Next [+/-] Vol [q] Quit ";
        lines.push(format!(
            "└{}┘",
            pad_or_truncate_fill(&format!("─{footer_keys}─"), inner_width, '─')
        ));
    } else {
        lines.push(format!("├{}┤", "─".repeat(inner_width)));
        let footer = " [Space] Play/Pause [Enter] Play [n/p] Next/Prev [s] Shuffle [r] Repeat [+/-] Vol [q] Quit";
        lines.push(format!("│{}│", pad_or_truncate(footer, inner_width)));
        lines.push(format!("└{}┘", "─".repeat(inner_width)));
    }

    lines
}

/// Render the complete TUI frame for the specified width and height as a newline-delimited string.
#[must_use]
pub fn render_frame(app: &TuiApp, width: u16, height: u16) -> String {
    render_lines(app, width, height).join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::MediaItem;

    #[test]
    fn test_format_now_playing() {
        let mut app = TuiApp::new();
        assert_eq!(format_now_playing(&app), "Playback stopped");

        app.player_state.current_item = Some(MediaItem::from_local_path("test.mp3"));
        assert!(format_now_playing(&app).contains("Now Playing: test.mp3"));
    }

    #[test]
    fn test_render_frame_components() {
        let mut app = TuiApp::new();
        app.player_state.playlist = vec![
            MediaItem::from_local_path("01_intro.mp3"),
            MediaItem::from_local_path("02_chorus.flac"),
        ];
        app.player_state.queue = vec![MediaItem::from_local_path("urgent.wav")];
        app.player_state.current_item = Some(MediaItem::from_local_path("01_intro.mp3"));
        app.player_state.status = PlayerStatus::Playing;
        app.player_state.position = Duration::from_secs(30);

        let frame = render_frame(&app, 80, 24);
        assert!(frame.contains("UPlay v0.1.0"));
        assert!(frame.contains("▶ Playing"));
        assert!(frame.contains("01_intro.mp3"));
        assert!(frame.contains("02_chorus.flac"));
        assert!(frame.contains("Playlist (2)"));
        assert!(frame.contains("Queue (1)"));

        // Test search prompt display
        app.start_search();
        app.search_query = "intro".to_string();
        let search_frame = render_frame(&app, 80, 24);
        assert!(search_frame.contains("Search: intro"));

        // Test add prompt display
        app.start_add();
        app.add_input = "new_song.mp3".to_string();
        let add_frame = render_frame(&app, 80, 24);
        assert!(add_frame.contains("Add Path/URL/Playlist: new_song.mp3"));
    }

    #[test]
    fn test_render_lines_exact_dimensions() {
        let mut app = TuiApp::new();
        app.player_state.playlist = vec![
            MediaItem::from_local_path("01_intro.mp3"),
            MediaItem::from_local_path("02_chorus.flac"),
        ];

        let sizes = [(80, 24), (60, 16), (120, 40), (50, 12), (40, 8), (30, 6)];
        for (w, h) in sizes {
            let lines = render_lines(&app, w, h);
            assert_eq!(lines.len(), h as usize, "Failed height check for {w}x{h}");
            for (idx, line) in lines.iter().enumerate() {
                assert_eq!(
                    line.chars().count(),
                    w as usize,
                    "Line {idx} failed width check for {w}x{h}"
                );
            }
        }
    }

    #[test]
    fn test_queue_conditional_visibility() {
        let mut app = TuiApp::new();
        app.player_state.playlist = vec![
            MediaItem::from_local_path("song1.mp3"),
            MediaItem::from_local_path("song2.flac"),
        ];

        // 1. Initially queue is empty and active view is Playlist:
        // Dual column divider ┬ should NOT be rendered in the header.
        let frame_collapsed = render_frame(&app, 80, 24);
        assert!(!frame_collapsed.contains('┬'));
        assert!(frame_collapsed.contains("Playlist (2)"));
        assert!(frame_collapsed.contains("Queue (0)"));
        assert!(frame_collapsed.contains("song1.mp3"));
        assert!(frame_collapsed.contains("song2.flac"));

        // 2. User enqueues a track: queue is no longer empty!
        // Queue column divider ┬ should now appear!
        app.player_state
            .queue
            .push(MediaItem::from_local_path("enqueued.wav"));
        let frame_with_queue = render_frame(&app, 80, 24);
        assert!(frame_with_queue.contains('┬'));
        assert!(frame_with_queue.contains("enqueued.wav"));

        // 3. Queue is cleared, but user presses Tab to switch to Queue view:
        app.player_state.queue.clear();
        app.active_view = ActiveView::Queue;
        let frame_tab_queue = render_frame(&app, 80, 24);
        assert!(frame_tab_queue.contains('┬'));
        assert!(frame_tab_queue.contains("Queue is empty"));

        // 4. User presses Tab to switch back to Playlist:
        app.active_view = ActiveView::Playlist;
        let frame_collapsed_again = render_frame(&app, 80, 24);
        assert!(!frame_collapsed_again.contains('┬'));
    }

    #[test]
    fn test_format_bar_progression() {
        let total = Some(Duration::from_secs(100));
        let bar0 = format_bar(Duration::ZERO, total, 10);
        assert_eq!(bar0, "[          ]");

        let bar50 = format_bar(Duration::from_secs(50), total, 10);
        assert_eq!(bar50, "[====>     ]");

        let bar100 = format_bar(Duration::from_secs(100), total, 10);
        assert_eq!(bar100, "[==========]");

        let bar_none = format_bar(Duration::from_secs(50), None, 10);
        assert_eq!(bar_none, "[----------]");
    }
}
