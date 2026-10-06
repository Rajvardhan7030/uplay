//! Filesystem watching and live library notification interfaces.

use std::path::{Path, PathBuf};

/// Event representing a detected filesystem change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsEvent {
    /// A new audio file was created.
    Created(PathBuf),
    /// An existing file was modified.
    Modified(PathBuf),
    /// A file was deleted or moved away.
    Deleted(PathBuf),
}

/// Interface for watching directories for audio file events.
pub trait Watcher: Send + Sync {
    /// Begin watching a directory path.
    fn watch(&mut self, path: &Path) -> crate::error::Result<()>;
    /// Stop watching a directory path.
    fn unwatch(&mut self, path: &Path) -> crate::error::Result<()>;
}
