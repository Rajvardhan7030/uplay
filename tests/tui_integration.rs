use clap::Parser;
use crossterm::event::{KeyCode, KeyModifiers};
use std::time::Duration;
use uplay::cli::{Cli, Command};
use uplay::core::state::{PlaybackMode, PlayerStatus};
use uplay::core::{MediaItem, PlayerEngine};
use uplay::tui::{ActiveView, TuiApp, handle_key_event, render_frame};

#[test]
fn test_cli_parsing_tui_command() {
    let cli = Cli::parse_from(["uplay", "tui"]);
    assert_eq!(cli.to_command(), Some(Command::Tui));

    let cli_default = Cli::parse_from(["uplay"]);
    assert_eq!(cli_default.to_command(), None);
}

#[test]
fn test_tui_navigation_and_selection() {
    let mut engine = PlayerEngine::new();
    let track1 = MediaItem::from_local_path("tests/fixtures/sine_440hz.wav");
    let track2 = MediaItem::from_local_path("tests/fixtures/album/01_toneA.wav");
    let track3 = MediaItem::from_local_path("tests/fixtures/album/sub/02_toneB.wav");
    engine.load_playlist(vec![track1, track2, track3]);

    let mut app = TuiApp::new();
    app.update_state(engine.state().clone());

    assert_eq!(app.selected_index, 0);
    assert_eq!(app.active_items_count(), 3);

    // Navigate down with down arrow
    handle_key_event(&mut app, &mut engine, KeyCode::Down, KeyModifiers::NONE).unwrap();
    assert_eq!(app.selected_index, 1);

    // Navigate down with 'j'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('j'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(app.selected_index, 2);

    // Clamped at bottom
    handle_key_event(&mut app, &mut engine, KeyCode::Down, KeyModifiers::NONE).unwrap();
    assert_eq!(app.selected_index, 2);

    // Navigate up with up arrow
    handle_key_event(&mut app, &mut engine, KeyCode::Up, KeyModifiers::NONE).unwrap();
    assert_eq!(app.selected_index, 1);

    // Navigate up with 'k'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('k'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(app.selected_index, 0);

    // Play selected track with Enter
    handle_key_event(&mut app, &mut engine, KeyCode::Enter, KeyModifiers::NONE).unwrap();
    assert_eq!(engine.state().status, PlayerStatus::Playing);
    assert_eq!(
        engine
            .state()
            .current_item
            .as_ref()
            .map(|i| i.title.as_str()),
        Some("sine_440hz.wav")
    );
}

#[test]
fn test_tui_playback_controls() {
    let mut engine = PlayerEngine::new();
    let track = MediaItem::from_local_path("tests/fixtures/sine_440hz.wav");
    engine.load_playlist(vec![track]);

    let mut app = TuiApp::new();
    app.update_state(engine.state().clone());

    // Space starts/toggles playback
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.state().status, PlayerStatus::Playing);

    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.state().status, PlayerStatus::Paused);

    // Volume controls
    let initial_vol = engine.state().volume;
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('+'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.state().volume, (initial_vol + 5).min(100));

    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('-'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.state().volume, initial_vol);

    // Shuffle and Repeat toggles
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('s'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.state().playback_mode, PlaybackMode::Shuffle);

    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('r'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.state().playback_mode, PlaybackMode::RepeatOne);

    // Quit key
    assert!(!app.should_quit);
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('q'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert!(app.should_quit);
}

#[test]
fn test_tui_search_workflow() {
    let mut engine = PlayerEngine::new();
    let track1 = MediaItem::from_local_path("alpha_song.mp3");
    let track2 = MediaItem::from_local_path("beta_tune.flac");
    let track3 = MediaItem::from_local_path("alpha_remix.wav");
    engine.load_playlist(vec![track1, track2, track3]);

    let mut app = TuiApp::new();
    app.update_state(engine.state().clone());

    // Enter search mode with '/'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('/'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert!(app.is_searching);

    // Type query "alpha"
    for c in "alpha".chars() {
        handle_key_event(&mut app, &mut engine, KeyCode::Char(c), KeyModifiers::NONE).unwrap();
    }
    assert_eq!(app.search_query, "alpha");
    assert_eq!(app.active_items_count(), 2);
    assert_eq!(app.filtered_playlist_indices(), vec![0, 2]);

    // Backspace test
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Backspace,
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(app.search_query, "alph");

    // Apply with Enter
    handle_key_event(&mut app, &mut engine, KeyCode::Enter, KeyModifiers::NONE).unwrap();
    assert!(!app.is_searching);
    assert_eq!(app.active_items_count(), 2);

    // Cancel / clear search with Esc
    handle_key_event(&mut app, &mut engine, KeyCode::Esc, KeyModifiers::NONE).unwrap();
    assert!(!app.is_searching);
    assert!(app.search_query.is_empty());
    assert_eq!(app.active_items_count(), 3);
}

#[test]
fn test_tui_add_and_remove_items() {
    let mut engine = PlayerEngine::new();
    let mut app = TuiApp::new();
    app.update_state(engine.state().clone());
    assert_eq!(engine.playlist_len(), 0);

    // Enter add mode with 'a'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('a'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert!(app.is_adding);

    // Type path to real fixture file
    let path_str = "tests/fixtures/sine_440hz.wav";
    for c in path_str.chars() {
        handle_key_event(&mut app, &mut engine, KeyCode::Char(c), KeyModifiers::NONE).unwrap();
    }
    assert_eq!(app.add_input, path_str);

    // Submit with Enter
    handle_key_event(&mut app, &mut engine, KeyCode::Enter, KeyModifiers::NONE).unwrap();
    assert!(!app.is_adding);
    assert_eq!(engine.playlist_len(), 1);
    assert_eq!(app.active_items_count(), 1);
    assert_eq!(engine.state().playlist[0].title, "sine_440hz.wav");

    // Remove item with 'd'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('d'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.playlist_len(), 0);
    assert_eq!(app.active_items_count(), 0);
}

#[test]
fn test_tui_queue_operations() {
    let mut engine = PlayerEngine::new();
    let track1 = MediaItem::from_local_path("song1.mp3");
    let track2 = MediaItem::from_local_path("song2.flac");
    engine.load_playlist(vec![track1, track2]);

    let mut app = TuiApp::new();
    app.update_state(engine.state().clone());

    // Highlight song 2
    app.selected_index = 1;

    // Enqueue song 2 with 'e'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('e'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.queue_len(), 1);
    assert_eq!(engine.queue_items()[0].title, "song2.flac");

    // Switch focus to Queue view with Tab
    handle_key_event(&mut app, &mut engine, KeyCode::Tab, KeyModifiers::NONE).unwrap();
    assert_eq!(app.active_view, ActiveView::Queue);
    assert_eq!(app.active_items_count(), 1);

    // Clear queue with 'c'
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('c'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(engine.queue_len(), 0);
    assert_eq!(app.active_items_count(), 0);

    // Switch back to Playlist with Tab
    handle_key_event(&mut app, &mut engine, KeyCode::Tab, KeyModifiers::NONE).unwrap();
    assert_eq!(app.active_view, ActiveView::Playlist);
    assert_eq!(app.active_items_count(), 2);

    // Test direct '2' key to switch to Queue
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('2'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(app.active_view, ActiveView::Queue);

    // Test direct '1' key to switch back to Playlist
    handle_key_event(
        &mut app,
        &mut engine,
        KeyCode::Char('1'),
        KeyModifiers::NONE,
    )
    .unwrap();
    assert_eq!(app.active_view, ActiveView::Playlist);
}

#[test]
fn test_tui_render_frame_sizing() {
    let mut app = TuiApp::new();
    app.player_state.playlist = vec![
        MediaItem::from_local_path("track1.mp3"),
        MediaItem::from_local_path("track2.wav"),
    ];
    app.player_state.current_item = Some(MediaItem::from_local_path("track1.mp3"));
    app.player_state.status = PlayerStatus::Playing;
    app.player_state.position = Duration::from_secs(45);

    // Standard 80x24 terminal
    let frame80 = render_frame(&app, 80, 24);
    assert!(frame80.contains("UPlay v1.0.1"));
    assert!(frame80.contains("track1.mp3"));
    assert!(frame80.contains("track2.wav"));
    assert!(frame80.contains("00:45"));

    // Larger 120x40 terminal
    let frame120 = render_frame(&app, 120, 40);
    assert!(frame120.contains("UPlay v1.0.1"));
    assert!(frame120.contains("track1.mp3"));

    // Compact 60x16 terminal
    let frame60 = render_frame(&app, 60, 16);
    assert!(frame60.contains("UPlay v1.0.1"));
    assert!(frame60.contains("track1.mp3"));

    // Small split pane 50x12 terminal
    let frame50 = render_frame(&app, 50, 12);
    assert!(frame50.contains("UPlay"));
    assert!(frame50.contains("track1.mp3"));

    // Tiny 40x8 terminal
    let frame40 = render_frame(&app, 40, 8);
    assert!(frame40.contains("UPlay"));
    assert!(frame40.contains("track1.mp3"));
}
