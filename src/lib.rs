//! # UPlay
//!
//! A lightweight, Linux-first music player and engine designed around the Unix philosophy.

pub mod audio;
pub mod cli;
pub mod config;
pub mod core;
pub mod daemon;
pub mod error;
pub mod filesystem;
pub mod sources;
pub mod tui;

pub use error::{Result, UPlayError};
