//! Audio backend traits, stream metadata, and output engine interfaces.

pub mod backend;
pub mod stream;

pub use backend::{AudioBackend, NullAudioBackend};
pub use stream::StreamMetadata;
