//! Priority queue management for upcoming media items.

use super::state::MediaItem;

/// Priority queue that allows users to queue media items without altering the active playlist.
#[derive(Debug, Default, Clone)]
pub struct QueueManager {
    items: Vec<MediaItem>,
}

impl QueueManager {
    /// Creates a new empty `QueueManager`.
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an item to the end of the queue.
    pub fn add(&mut self, item: MediaItem) {
        self.items.push(item);
    }

    /// Add an item to the front of the queue so it plays next.
    pub fn add_next(&mut self, item: MediaItem) {
        self.items.insert(0, item);
    }

    /// Remove an item at the specified index.
    pub fn remove(&mut self, index: usize) -> Option<MediaItem> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Move an item from `from_index` to `to_index`.
    pub fn move_item(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.items.len() || to_index >= self.items.len() {
            return false;
        }
        let item = self.items.remove(from_index);
        self.items.insert(to_index, item);
        true
    }

    /// Clear all items from the queue.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Peek at the next item in the queue without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<&MediaItem> {
        self.items.first()
    }

    /// Pop the next item from the front of the queue.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<MediaItem> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    /// Return a slice of all items currently in the queue.
    #[must_use]
    pub fn items(&self) -> &[MediaItem] {
        &self.items
    }

    /// Return the number of items in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_operations() {
        let mut q = QueueManager::new();
        assert!(q.is_empty());

        let song1 = MediaItem::from_local_path("song1.mp3");
        let song2 = MediaItem::from_local_path("song2.mp3");
        let song3 = MediaItem::from_local_path("song3.mp3");

        q.add(song1.clone());
        q.add(song2.clone());
        assert_eq!(q.len(), 2);
        assert_eq!(q.peek().map(|s| s.title.as_str()), Some("song1.mp3"));

        // add_next puts song3 at the front
        q.add_next(song3.clone());
        assert_eq!(q.len(), 3);
        assert_eq!(q.peek().map(|s| s.title.as_str()), Some("song3.mp3"));

        // pop next
        let next_item = q.next();
        assert_eq!(next_item.map(|s| s.title), Some("song3.mp3".to_string()));
        assert_eq!(q.len(), 2);

        // move item
        assert!(q.move_item(0, 1));
        assert_eq!(q.peek().map(|s| s.title.as_str()), Some("song2.mp3"));

        // remove
        let removed = q.remove(0);
        assert_eq!(removed.map(|s| s.title), Some("song2.mp3".to_string()));

        q.clear();
        assert!(q.is_empty());
    }
}
