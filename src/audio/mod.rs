//! Audio backend traits, stream metadata, and output engine interfaces.

pub mod backend;
pub mod rodio_backend;
pub mod stream;

pub use backend::{AudioBackend, NullAudioBackend};
pub use rodio_backend::RodioBackend;
pub use stream::StreamMetadata;
