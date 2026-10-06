//! Filesystem operations, directory scanning, and event notifications.

pub mod watcher;

pub use watcher::{FsEvent, Watcher};
