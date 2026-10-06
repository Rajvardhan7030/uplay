use std::fs::{File, create_dir_all};
use std::io::Write;
use uplay::core::{PlayerEngine, PlayerStatus};
use uplay::sources::LocalResolver;

fn create_dummy_wav(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }
    // Write a valid minimal 44-byte WAV header
    let header: [u8; 44] = [
        b'R', b'I', b'F', b'F', 36, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0,
        0, 0, 1, 0, 1, 0, 0x44, 0xac, 0, 0, 0x88, 0x58, 1, 0, 2, 0, 16, 0, b'd', b'a', b't', b'a',
        0, 0, 0, 0,
    ];
    let mut file = File::create(path).expect("create dummy wav");
    file.write_all(&header).expect("write header");
}

#[test]
fn test_folder_scanning_recursive_and_deterministic() {
    let base_dir = std::env::temp_dir().join("uplay_test_album");
    let _ = std::fs::remove_dir_all(&base_dir);

    let t1 = base_dir.join("01_intro.wav");
    let t2 = base_dir.join("cd1").join("02_song.mp3");
    let t3 = base_dir.join("cd2").join("03_outro.flac");
    let non_audio = base_dir.join("lyrics.txt");
    let hidden = base_dir.join(".hidden").join("secret.wav");

    create_dummy_wav(&t1);
    create_dummy_wav(&t2);
    create_dummy_wav(&t3);
    create_dummy_wav(&hidden);

    let mut txt_file = File::create(&non_audio).expect("create text file");
    writeln!(txt_file, "lyrics").expect("write text");

    let resolver = LocalResolver::new();
    let items = resolver.scan_directory(&base_dir).expect("scan directory");

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].title, "01_intro.wav");
    assert_eq!(items[1].title, "02_song.mp3");
    assert_eq!(items[2].title, "03_outro.flac");

    // Clean up
    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_player_engine_folder_sequential_playback() {
    let base_dir = std::env::temp_dir().join("uplay_test_album_seq");
    let _ = std::fs::remove_dir_all(&base_dir);

    let t1 = base_dir.join("track1.wav");
    let t2 = base_dir.join("track2.wav");
    create_dummy_wav(&t1);
    create_dummy_wav(&t2);

    let resolver = LocalResolver::new();
    let items = resolver.scan_directory(&base_dir).expect("scan directory");
    assert_eq!(items.len(), 2);

    let mut engine = PlayerEngine::new();
    engine.load_playlist(items);
    assert_eq!(engine.playlist_len(), 2);
    assert_eq!(engine.state().status, PlayerStatus::Stopped);

    // Track 1 starts
    let current = engine.next().unwrap();
    assert_eq!(current.map(|i| i.title), Some("track1.wav".into()));
    assert_eq!(engine.current_track_index(), Some(0));
    assert_eq!(engine.state().status, PlayerStatus::Playing);

    // Track 2 advances
    let current2 = engine.next().unwrap();
    assert_eq!(current2.map(|i| i.title), Some("track2.wav".into()));
    assert_eq!(engine.current_track_index(), Some(1));

    // Previous returns to Track 1
    let prev = engine.previous().unwrap();
    assert_eq!(prev.map(|i| i.title), Some("track1.wav".into()));
    assert_eq!(engine.current_track_index(), Some(0));

    // Advance to Track 2, then end of playlist
    engine.next().unwrap();
    let finished = engine.next().unwrap();
    assert_eq!(finished, None);
    assert_eq!(engine.state().status, PlayerStatus::Stopped);

    let _ = std::fs::remove_dir_all(&base_dir);
}
