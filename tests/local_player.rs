use std::path::PathBuf;
use std::time::Duration;
use uplay::audio::{AudioBackend, RodioBackend};
use uplay::core::state::Source;
use uplay::core::{MediaItem, PlayerEngine, PlayerStatus};
use uplay::sources::LocalResolver;

#[test]
fn test_local_resolver_validation() {
    let resolver = LocalResolver::new();

    // Nonexistent file
    let res = resolver.resolve_file(PathBuf::from("does_not_exist.flac"));
    assert!(res.is_err());

    // Non-audio file
    let res = resolver.resolve_file(PathBuf::from("Cargo.toml"));
    assert!(res.is_err());

    // Valid audio file
    let test_wav = PathBuf::from("tests/fixtures/sine_440hz.wav");
    if test_wav.exists() {
        let res = resolver.resolve_file(&test_wav);
        assert!(res.is_ok());
        let item = res.unwrap();
        assert_eq!(item.title, "sine_440hz.wav");
        assert_eq!(item.source, Source::LocalFile(test_wav));
    }
}

#[test]
fn test_rodio_backend_local_file_lifecycle() {
    let test_wav = PathBuf::from("tests/fixtures/sine_440hz.wav");
    if !test_wav.exists() {
        return;
    }

    let mut backend = RodioBackend::new();
    if !backend.is_available() {
        // In headless environment without audio output device
        return;
    }

    let source = Source::LocalFile(test_wav);
    assert!(backend.load(&source).is_ok());
    assert!(backend.duration().is_some());

    assert!(backend.play().is_ok());
    assert!(!backend.is_paused());

    assert!(backend.pause().is_ok());
    assert!(backend.is_paused());

    assert!(backend.set_volume(50).is_ok());
    assert!(backend.seek(Duration::from_millis(500)).is_ok());

    assert!(backend.stop().is_ok());
}

#[test]
fn test_player_engine_with_real_audio() {
    let test_wav = PathBuf::from("tests/fixtures/sine_440hz.wav");
    if !test_wav.exists() {
        return;
    }

    let mut engine = PlayerEngine::with_rodio();
    let item = MediaItem::from_local_path(&test_wav);

    // If audio hardware is available, test load and controls
    if let Ok(()) = engine.load(item) {
        assert_eq!(engine.state().status, PlayerStatus::Stopped);

        assert!(engine.play().is_ok());
        assert_eq!(engine.state().status, PlayerStatus::Playing);

        assert!(engine.pause().is_ok());
        assert_eq!(engine.state().status, PlayerStatus::Paused);

        assert!(engine.resume().is_ok());
        assert_eq!(engine.state().status, PlayerStatus::Playing);

        assert!(engine.set_volume(70).is_ok());
        assert_eq!(engine.state().volume, 70);

        assert!(engine.seek_relative(1).is_ok());
        assert!(engine.stop().is_ok());
        assert_eq!(engine.state().status, PlayerStatus::Stopped);
    }
}
