//! Terminal UI application state container.

use crate::core::state::PlayerState;

/// TUI client state container.
#[derive(Debug, Default)]
pub struct TuiApp {
    /// Authoritative player state received from the engine or daemon.
    pub player_state: PlayerState,
    /// Whether the TUI application is requested to exit.
    pub should_quit: bool,
    /// Selected index in active list view (e.g. playlist or queue).
    pub selected_index: usize,
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

    /// Update with newly received player state.
    pub fn update_state(&mut self, state: PlayerState) {
        self.player_state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_app_lifecycle() {
        let mut app = TuiApp::new();
        assert!(!app.should_quit);
        app.quit();
        assert!(app.should_quit);
    }
}
