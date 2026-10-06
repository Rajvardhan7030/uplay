use uplay::core::{MediaItem, QueueManager};

#[test]
fn test_queue_integration() {
    let mut queue = QueueManager::new();

    let song_a = MediaItem::from_local_path("a.flac");
    let song_b = MediaItem::from_local_path("b.flac");
    let song_priority = MediaItem::from_local_path("urgent.mp3");

    queue.add(song_a);
    queue.add(song_b);
    assert_eq!(queue.len(), 2);

    queue.add_next(song_priority);
    assert_eq!(queue.len(), 3);

    let next_item = queue.next().expect("should have item");
    assert_eq!(next_item.title, "urgent.mp3");
}
