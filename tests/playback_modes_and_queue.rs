use uplay::core::state::{MediaItem, PlaybackMode};
use uplay::core::{PlayerEngine, PlayerStatus};

fn make_tracks(count: usize) -> Vec<MediaItem> {
    (0..count)
        .map(|i| MediaItem::from_local_path(format!("song_{i:02}.flac")))
        .collect()
}

#[test]
fn test_deterministic_shuffle_reproducibility() {
    let mut engine1 = PlayerEngine::new();
    let mut engine2 = PlayerEngine::new();

    let tracks = make_tracks(6);
    engine1.load_playlist(tracks.clone());
    engine2.load_playlist(tracks);

    // Set same deterministic seed
    engine1.set_mode(PlaybackMode::Shuffle);
    engine1.set_shuffle_seed(9999);

    engine2.set_mode(PlaybackMode::Shuffle);
    engine2.set_shuffle_seed(9999);

    assert_eq!(engine1.shuffle_order(), engine2.shuffle_order());

    let mut seq1 = Vec::new();
    let mut seq2 = Vec::new();

    while let Ok(Some(track)) = engine1.next() {
        seq1.push(track.title);
    }

    while let Ok(Some(track)) = engine2.next() {
        seq2.push(track.title);
    }

    assert_eq!(seq1.len(), 6);
    assert_eq!(seq1, seq2);
}

#[test]
fn test_shuffle_with_priority_queue_non_corruption() {
    let mut engine = PlayerEngine::new();
    let tracks = make_tracks(5);
    engine.load_playlist(tracks);

    engine.set_mode(PlaybackMode::Shuffle);
    engine.set_shuffle_seed(42);

    let initial_shuffle_order = engine.shuffle_order().to_vec();

    // Play first shuffled track
    let first_track = engine.next().unwrap().unwrap();
    assert_eq!(
        first_track.title,
        format!("song_{:02}.flac", initial_shuffle_order[0])
    );

    // Add normal item and add_next item to priority queue
    let normal_queue = MediaItem::from_local_path("normal_queued.mp3");
    let priority_queue = MediaItem::from_local_path("priority_next.mp3");

    engine.queue_add(normal_queue);
    engine.queue_add_next(priority_queue);

    assert_eq!(engine.queue_len(), 2);
    assert_eq!(
        engine.queue_peek().map(|t| t.title.as_str()),
        Some("priority_next.mp3")
    );

    // Priority next item must play next
    let q1 = engine.next().unwrap().unwrap();
    assert_eq!(q1.title, "priority_next.mp3");
    assert_eq!(engine.queue_len(), 1);

    // Then normal queue item must play
    let q2 = engine.next().unwrap().unwrap();
    assert_eq!(q2.title, "normal_queued.mp3");
    assert_eq!(engine.queue_len(), 0);

    // Now playback MUST resume from the second track of the shuffle order!
    let second_shuffled = engine.next().unwrap().unwrap();
    assert_eq!(
        second_shuffled.title,
        format!("song_{:02}.flac", initial_shuffle_order[1])
    );

    // Remainder of shuffle order plays completely without loss
    let third_shuffled = engine.next().unwrap().unwrap();
    assert_eq!(
        third_shuffled.title,
        format!("song_{:02}.flac", initial_shuffle_order[2])
    );
}

#[test]
fn test_queue_manipulation_operations() {
    let mut engine = PlayerEngine::new();
    assert!(engine.queue_is_empty());

    let q1 = MediaItem::from_local_path("q1.mp3");
    let q2 = MediaItem::from_local_path("q2.mp3");
    let q3 = MediaItem::from_local_path("q3.mp3");

    engine.queue_add(q1);
    engine.queue_add(q2);
    engine.queue_add(q3);
    assert_eq!(engine.queue_len(), 3);

    // Inspect queue items
    let titles: Vec<_> = engine
        .queue_items()
        .iter()
        .map(|i| i.title.as_str())
        .collect();
    assert_eq!(titles, vec!["q1.mp3", "q2.mp3", "q3.mp3"]);

    // Move q3 to index 0
    assert!(engine.queue_move(2, 0));
    assert_eq!(
        engine.queue_peek().map(|i| i.title.as_str()),
        Some("q3.mp3")
    );

    // Remove index 1 (which is now q1)
    let removed = engine.queue_remove(1);
    assert_eq!(removed.map(|i| i.title), Some("q1.mp3".into()));
    assert_eq!(engine.queue_len(), 2);

    // Clear queue
    engine.queue_clear();
    assert!(engine.queue_is_empty());
    assert_eq!(engine.queue_len(), 0);
}

#[test]
fn test_repeat_one_and_playlist_modes() {
    let mut engine = PlayerEngine::new();
    let tracks = make_tracks(2);
    engine.load_playlist(tracks);

    // Start with RepeatOne
    engine.set_mode(PlaybackMode::RepeatOne);
    let t0 = engine.next().unwrap().unwrap();
    assert_eq!(t0.title, "song_00.flac");

    // Next must repeat the exact same track
    let t0_rep = engine.next().unwrap().unwrap();
    assert_eq!(t0_rep.title, "song_00.flac");

    // Switch to RepeatPlaylist
    engine.set_mode(PlaybackMode::RepeatPlaylist);
    let t1 = engine.next().unwrap().unwrap();
    assert_eq!(t1.title, "song_01.flac");

    // Next at end of playlist wraps around to first track
    let t0_wrapped = engine.next().unwrap().unwrap();
    assert_eq!(t0_wrapped.title, "song_00.flac");

    // Switch to Sequential
    engine.set_mode(PlaybackMode::Sequential);
    let t1_seq = engine.next().unwrap().unwrap();
    assert_eq!(t1_seq.title, "song_01.flac");

    // Sequential ends at the last track
    let end = engine.next().unwrap();
    assert_eq!(end, None);
    assert_eq!(engine.state().status, PlayerStatus::Stopped);
}
