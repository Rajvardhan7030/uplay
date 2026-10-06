//! Terminal User Interface (TUI) client implementation.

pub mod app;
pub mod input;
pub mod ui;

pub use app::TuiApp;
pub use input::map_key_to_command;
pub use ui::format_now_playing;
