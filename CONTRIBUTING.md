# Contributing to UPlay

Thank you for your interest in contributing to UPlay!

## Core Principles

- **Linux-first & Terminal-first**: Keep things keyboard friendly and composable.
- **Low Resource Usage**: Every feature must justify its CPU, memory, complexity, and maintenance cost.
- **Modularity**: Separate CLI/TUI interfaces, playback engine, sources, and audio backends.

## Development Setup

### Prerequisites

- **Linux OS**: Modern Linux distribution (Fedora, Arch, Ubuntu/Debian).
- **Rust Toolchain**: Rust 1.85.0 or newer (Edition 2024).
- **Cargo Tools**: `cargo clippy`, `cargo fmt`.

### Engineering Standards & Checks

Before submitting any changes, make sure all checks pass:

```bash
# Build
cargo build

# Tests
cargo test

# Linter
cargo clippy --all-targets --all-features -- -D warnings

# Code formatting
cargo fmt --check
```

## Project Structure

Refer to [Architecture Documentation](docs/architecture.md) and [Project Summary](docs/summary.md) for details on code organization and architectural boundaries.
