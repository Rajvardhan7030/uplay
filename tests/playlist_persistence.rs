use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use uplay::core::{MediaItem, Playlist, PlaylistManager, Source};
use uplay::sources::{LocalResolver, resolve_targets};

fn setup_temp_playlist_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uplay_test_persistence_{test_name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create temp playlist dir");
    dir
}

#[test]
fn test_save_and_load_playlist_persistence() {
    let temp_dir = setup_temp_playlist_dir("save_and_load");
    let manager = PlaylistManager::with_storage_dir(&temp_dir);

    let mut playlist = Playlist::new("coding_vibes");
    let item1 = MediaItem {
        id: "tests/fixtures/sine_440hz.wav".into(),
        source: Source::LocalFile(PathBuf::from("tests/fixtures/sine_440hz.wav")),
        title: "Sine Wave 440Hz".into(),
        artist: Some("Test Generator".into()),
        duration: Some(Duration::from_secs(5)),
    };
    let item2 = MediaItem {
        id: "tests/fixtures/album/01_toneA.wav".into(),
        source: Source::LocalFile(PathBuf::from("tests/fixtures/album/01_toneA.wav")),
        title: "Tone A".into(),
        artist: None,
        duration: None,
    };
    let item3 = MediaItem {
        id: "https://example.com/radio.mp3".into(),
        source: Source::HttpStream("https://example.com/radio.mp3".into()),
        title: "Streaming Radio".into(),
        artist: Some("Internet Broadcaster".into()),
        duration: None,
    };

    playlist.add(item1);
    playlist.add(item2);
    playlist.add(item3);

    // Save playlist to disk
    manager
        .save_to_disk(&playlist)
        .expect("should save playlist to disk");

    // Verify .m3u8 file was written
    let expected_file = temp_dir.join("coding_vibes.m3u8");
    assert!(expected_file.exists());
    assert!(expected_file.is_file());

    // Reopen in a completely new PlaylistManager instance (simulating app restart)
    let fresh_manager = PlaylistManager::with_storage_dir(&temp_dir);
    let loaded = fresh_manager
        .load_from_disk("coding_vibes")
        .expect("should load playlist from disk");

    assert_eq!(loaded.name, "coding_vibes");
    assert_eq!(loaded.len(), 3);

    assert_eq!(loaded.items[0].title, "Sine Wave 440Hz");
    assert_eq!(loaded.items[0].artist.as_deref(), Some("Test Generator"));
    assert_eq!(loaded.items[0].duration, Some(Duration::from_secs(5)));
    assert_eq!(
        loaded.items[0].source,
        Source::LocalFile(PathBuf::from("tests/fixtures/sine_440hz.wav"))
    );

    assert_eq!(loaded.items[1].title, "Tone A");
    assert_eq!(
        loaded.items[1].source,
        Source::LocalFile(PathBuf::from("tests/fixtures/album/01_toneA.wav"))
    );

    assert_eq!(loaded.items[2].title, "Streaming Radio");
    assert_eq!(
        loaded.items[2].source,
        Source::HttpStream("https://example.com/radio.mp3".into())
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_list_and_delete_saved_playlists() {
    let temp_dir = setup_temp_playlist_dir("list_and_delete");
    let manager = PlaylistManager::with_storage_dir(&temp_dir);

    // Initially empty
    assert!(manager.list_saved().unwrap().is_empty());

    let pl_workout = Playlist::new("workout");
    let pl_coding = Playlist::new("coding");
    let pl_ambient = Playlist::new("ambient");

    manager.save_to_disk(&pl_workout).unwrap();
    manager.save_to_disk(&pl_coding).unwrap();
    manager.save_to_disk(&pl_ambient).unwrap();

    let list = manager.list_saved().unwrap();
    assert_eq!(list, vec!["ambient", "coding", "workout"]);

    // Delete one
    manager.delete_saved("coding").unwrap();
    let updated_list = manager.list_saved().unwrap();
    assert_eq!(updated_list, vec!["ambient", "workout"]);
    assert!(manager.load_from_disk("coding").is_err());

    // Deleting nonexistent playlist returns error
    assert!(manager.delete_saved("nonexistent").is_err());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_m3u_format_with_comments_and_whitespace() {
    let temp_dir = setup_temp_playlist_dir("m3u_parsing");
    let m3u_path = temp_dir.join("custom.m3u");

    let m3u_content = r#"#EXTM3U
# This is a comment at the top
#EXTINF:180,Artist Name - Track Alpha
/var/music/alpha.mp3

# An inline comment followed by blank line

#EXTINF:-1,Track Beta Without Artist
/var/music/beta.flac

#EXTINF:240,Gamma
https://example.com/stream.ogg

# Simple plain path with no EXTINF
/var/music/plain.wav
"#;

    fs::write(&m3u_path, m3u_content).expect("failed to write m3u");

    let playlist = Playlist::from_m3u_file(&m3u_path).expect("failed to parse m3u file");
    assert_eq!(playlist.name, "custom");
    assert_eq!(playlist.len(), 4);

    assert_eq!(playlist.items[0].title, "Track Alpha");
    assert_eq!(playlist.items[0].artist.as_deref(), Some("Artist Name"));
    assert_eq!(playlist.items[0].duration, Some(Duration::from_secs(180)));
    assert_eq!(
        playlist.items[0].source,
        Source::LocalFile(PathBuf::from("/var/music/alpha.mp3"))
    );

    assert_eq!(playlist.items[1].title, "Track Beta Without Artist");
    assert!(playlist.items[1].artist.is_none());
    assert_eq!(
        playlist.items[1].source,
        Source::LocalFile(PathBuf::from("/var/music/beta.flac"))
    );

    assert_eq!(playlist.items[2].title, "Gamma");
    assert_eq!(
        playlist.items[2].source,
        Source::HttpStream("https://example.com/stream.ogg".into())
    );

    assert_eq!(playlist.items[3].title, "plain.wav");
    assert_eq!(
        playlist.items[3].source,
        Source::LocalFile(PathBuf::from("/var/music/plain.wav"))
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_local_resolver_resolves_playlist_file() {
    let temp_dir = setup_temp_playlist_dir("resolver_m3u");
    let playlist_path = temp_dir.join("test_mix.m3u8");

    let mut pl = Playlist::new("test_mix");
    pl.add(MediaItem::from_local_path("tests/fixtures/sine_440hz.wav"));
    pl.add(MediaItem::from_local_path(
        "tests/fixtures/album/01_toneA.wav",
    ));
    pl.save_m3u(&playlist_path).unwrap();

    let resolver = LocalResolver::new();
    assert!(LocalResolver::is_playlist_file(&playlist_path));

    let items = resolver
        .resolve_path(&playlist_path)
        .expect("LocalResolver should resolve m3u8 file");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].title, "sine_440hz.wav");
    assert_eq!(items[1].title, "01_toneA.wav");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_playlist_edit_and_target_resolution() {
    let temp_dir = setup_temp_playlist_dir("edits");
    let manager = PlaylistManager::with_storage_dir(&temp_dir);

    // Resolve real fixture files using resolve_targets
    let targets = vec![
        "tests/fixtures/sine_440hz.wav".to_string(),
        "tests/fixtures/album".to_string(),
    ];
    let items = resolve_targets(&targets).expect("should resolve targets");
    // sine_440hz + album (01_toneA and sub/02_toneB) = 3 items
    assert_eq!(items.len(), 3);

    let mut playlist = Playlist::new("mixed_album");
    for item in items {
        playlist.add(item);
    }
    manager.save_to_disk(&playlist).unwrap();

    // Reload and verify
    let mut loaded = manager.load_from_disk("mixed_album").unwrap();
    assert_eq!(loaded.len(), 3);

    // Remove track 1 (sine_440hz.wav)
    let removed = loaded.remove_by_target("1").expect("should remove track 1");
    assert_eq!(removed.title, "sine_440hz.wav");
    assert_eq!(loaded.len(), 2);

    // Save updated playlist
    manager.save_to_disk(&loaded).unwrap();

    // Reload again and verify only 2 tracks remain
    let final_loaded = manager.load_from_disk("mixed_album").unwrap();
    assert_eq!(final_loaded.len(), 2);
    assert_eq!(final_loaded.items[0].title, "01_toneA.wav");
    assert_eq!(final_loaded.items[1].title, "02_toneB.wav");

    let _ = fs::remove_dir_all(&temp_dir);
}
