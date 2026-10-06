//! Media source resolvers for local files, HTTP streams, and YouTube.

pub mod http;
pub mod local;
pub mod youtube;

pub use http::HttpResolver;
pub use local::LocalResolver;
pub use youtube::YouTubeResolver;
