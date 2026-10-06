use std::time::Duration;
use uplay::core::{MediaItem, PlaybackMode, Session};

#[test]
fn test_session_state_preservation() {
    let session = Session {
        playlist_name: Some("chill".into()),
        current_index: Some(2),
        position: Duration::from_secs(142),
        volume: 65,
        playback_mode: PlaybackMode::Shuffle,
        items: vec![
            MediaItem::from_local_path("track1.mp3"),
            MediaItem::from_local_path("track2.mp3"),
            MediaItem::from_local_path("track3.mp3"),
        ],
        queue: vec![],
    };

    assert_eq!(session.volume, 65);
    assert_eq!(session.position.as_secs(), 142);
    assert_eq!(session.current_index, Some(2));
    assert_eq!(session.playback_mode, PlaybackMode::Shuffle);
    assert_eq!(session.items.len(), 3);
}
