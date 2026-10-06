use uplay::core::{MediaItem, Playlist, PlaylistManager};

#[test]
fn test_playlist_integration() {
    let mut manager = PlaylistManager::new();

    let mut workout = Playlist::new("workout");
    workout.add(MediaItem::from_local_path("/music/track1.mp3"));
    workout.add(MediaItem::from_local_path("/music/track2.ogg"));

    assert_eq!(workout.len(), 2);
    manager.save(workout);

    let retrieved = manager.get("workout").expect("playlist should be saved");
    assert_eq!(retrieved.name, "workout");
    assert_eq!(retrieved.items.len(), 2);
    assert_eq!(retrieved.items[0].title, "track1.mp3");
}
