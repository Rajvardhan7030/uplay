//! TUI rendering interfaces and layout components.

use super::app::TuiApp;

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
}
