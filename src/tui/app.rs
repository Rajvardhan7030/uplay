//! Terminal UI application state container.

use crate::core::state::PlayerState;
use std::time::Instant;

/// Active focus view in the TUI interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    /// Focus on playlist track listing.
    #[default]
    Playlist,
    /// Focus on upcoming priority queue tracks.
    Queue,
}

/// TUI client state container.
#[derive(Debug, Default)]
pub struct TuiApp {
    /// Authoritative player state received from the engine or daemon.
    pub player_state: PlayerState,
    /// Whether the TUI application is requested to exit.
    pub should_quit: bool,
    /// Selected index in active list view (e.g. playlist or queue).
    pub selected_index: usize,
    /// Currently focused panel (Playlist or Queue).
    pub active_view: ActiveView,
    /// Whether real-time search input mode is active.
    pub is_searching: bool,
    /// Current search query string.
    pub search_query: String,
    /// Whether the add target input mode is active.
    pub is_adding: bool,
    /// User input buffer for adding tracks/folders/URLs.
    pub add_input: String,
    /// Informational or error status message.
    pub status_message: Option<String>,
    /// Timestamp when status message was set.
    pub status_time: Option<Instant>,
    /// Scroll offset for playlist view.
    pub playlist_scroll: usize,
    /// Scroll offset for queue view.
    pub queue_scroll: usize,
}

impl TuiApp {
    /// Create a new `TuiApp` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the TUI application to quit.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Update with newly received player state and clamp selection.
    pub fn update_state(&mut self, state: PlayerState) {
        self.player_state = state;
        self.clamp_selection();
    }

    /// Set an ephemeral status message.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.status_time = Some(Instant::now());
    }

    /// Clear the current status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_time = None;
    }

    /// Toggle focus between Playlist view and Queue view.
    pub fn toggle_active_view(&mut self) {
        self.active_view = match self.active_view {
            ActiveView::Playlist => ActiveView::Queue,
            ActiveView::Queue => ActiveView::Playlist,
        };
        self.selected_index = 0;
        self.clamp_selection();
    }

    /// Get list of indices of playlist tracks matching current search query.
    #[must_use]
    pub fn filtered_playlist_indices(&self) -> Vec<usize> {
        if self.search_query.trim().is_empty() {
            return (0..self.player_state.playlist.len()).collect();
        }

        let query = self.search_query.trim().to_ascii_lowercase();
        self.player_state
            .playlist
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let title_matches = item.title.to_ascii_lowercase().contains(&query);
                let artist_matches = item
                    .artist
                    .as_ref()
                    .is_some_and(|a| a.to_ascii_lowercase().contains(&query));
                if title_matches || artist_matches {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Total number of selectable items in the currently active panel.
    #[must_use]
    pub fn active_items_count(&self) -> usize {
        match self.active_view {
            ActiveView::Playlist => self.filtered_playlist_indices().len(),
            ActiveView::Queue => self.player_state.queue.len(),
        }
    }

    /// Move the cursor selection up one position.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.adjust_scroll();
    }

    /// Move the cursor selection down one position.
    pub fn move_down(&mut self) {
        let count = self.active_items_count();
        if count > 0 && self.selected_index + 1 < count {
            self.selected_index += 1;
        }
        self.adjust_scroll();
    }

    /// Ensure `selected_index` stays within valid item bounds.
    pub fn clamp_selection(&mut self) {
        let count = self.active_items_count();
        if count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= count {
            self.selected_index = count.saturating_sub(1);
        }
        self.adjust_scroll();
    }

    /// Enter interactive search input mode.
    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.is_adding = false;
        self.search_query.clear();
    }

    /// Exit search input mode while retaining query.
    pub fn finish_search(&mut self) {
        self.is_searching = false;
        self.clamp_selection();
    }

    /// Cancel search and reset search filter.
    pub fn cancel_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        self.clamp_selection();
    }

    /// Enter interactive add target input mode.
    pub fn start_add(&mut self) {
        self.is_adding = true;
        self.is_searching = false;
        self.add_input.clear();
    }

    /// Cancel add target input mode.
    pub fn cancel_add(&mut self) {
        self.is_adding = false;
        self.add_input.clear();
    }

    fn adjust_scroll(&mut self) {
        let visible_lines = 10; // Nominal scroll window threshold
        match self.active_view {
            ActiveView::Playlist => {
                if self.selected_index < self.playlist_scroll {
                    self.playlist_scroll = self.selected_index;
                } else if self.selected_index >= self.playlist_scroll + visible_lines {
                    self.playlist_scroll = self.selected_index - visible_lines + 1;
                }
            }
            ActiveView::Queue => {
                if self.selected_index < self.queue_scroll {
                    self.queue_scroll = self.selected_index;
                } else if self.selected_index >= self.queue_scroll + visible_lines {
                    self.queue_scroll = self.selected_index - visible_lines + 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::MediaItem;

    #[test]
    fn test_tui_app_lifecycle() {
        let mut app = TuiApp::new();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }

    #[test]
    fn test_tui_app_navigation_and_search() {
        let mut app = TuiApp::new();
        let item1 = MediaItem::from_local_path("ambient.mp3");
        let item2 = MediaItem::from_local_path("rock.flac");
        let item3 = MediaItem::from_local_path("ambient_vibe.wav");

        app.player_state.playlist = vec![item1, item2, item3];
        assert_eq!(app.active_items_count(), 3);

        app.move_down();
        assert_eq!(app.selected_index, 1);
        app.move_down();
        assert_eq!(app.selected_index, 2);
        // Clamped at bottom
        app.move_down();
        assert_eq!(app.selected_index, 2);

        app.move_up();
        assert_eq!(app.selected_index, 1);

        // Search filtering
        app.search_query = "ambient".to_string();
        let matches = app.filtered_playlist_indices();
        assert_eq!(matches, vec![0, 2]);
        assert_eq!(app.active_items_count(), 2);

        // View toggle
        app.toggle_active_view();
        assert_eq!(app.active_view, ActiveView::Queue);
        assert_eq!(app.active_items_count(), 0);
    }
}
