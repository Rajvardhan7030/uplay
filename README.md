# UPlay

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85.0%2B-orange.svg)](https://www.rust-lang.org)

**UPlay(unix-player)** is a lightweight, Linux-first music player and engine designed around the Unix philosophy:

> *Play music quickly, use very few resources, stay keyboard/terminal friendly, and get out of the user's way.*

---

## Features (Roadmap)

- **Local Music Playback**: Play files and directories recursively with low CPU/RAM footprint.
- **Terminal UI & Scriptable CLI**: Interactive TUI (ratatui) alongside scriptable CLI commands.
- **Queue & Playlists**: Standard queue operations and local playlist management (M3U/M3U8).
- **Playback Modes**: Sequential, shuffle, repeat one, repeat playlist.
- **Session Persistence**: Resume previous tracks, playlists, positions, and queue seamlessly.
- **Modular Sources**: Local files, HTTP streams, and optional YouTube streaming via resolver plugins.
- **Optional Background Daemon**: Decoupled engine running over Unix domain sockets for media key integrations and multi-client workflows.

---

## Minimum Supported Systems & Toolchain

- **Operating System**: Linux (Fedora, Arch Linux, Ubuntu 22.04+, Debian 12+)
- **Kernel**: Linux 5.10 or newer
- **Audio Subsystem**: PipeWire (recommended), PulseAudio, or ALSA
- **Rust Toolchain**: `rustc` / `cargo` **1.85.0+** (Rust Edition 2024)

---

## Getting Started

### Building from Source

```bash
# Clone the repository
git clone https://github.com/uplay/uplay.git
cd uplay

# Build the project
cargo build --release

#Add the release version to .bashrc(depends on your shell)
copy the release version folder or its file location and then create a aliase to use it.
for ex:aliase uplay="path/to/your/realese/folder/destination"

# Run tests
cargo test

# Run linter and formatting checks
cargo clippy
cargo fmt --check
```

### Basic Usage

```bash
# Show help and CLI options
uplay --help

# Play a local audio file (coming in Goal 1)
uplay /path/to/song.flac
```

---

## Project Structure

```text
uplay/
├── Cargo.toml          # Rust package manifest & dependencies
├── README.md           # Project documentation and quickstart
├── LICENSE             # MIT License
├── CONTRIBUTING.md      # Contribution guidelines
├── docs/               # Architecture, roadmap, and summary docs
│   ├── summary.md
│   ├── architecture.md
│   └── roadmap.md
├── src/                # Source code
│   ├── lib.rs          # Core library interface
│   ├── main.rs         # Binary entrypoint
│   ├── error.rs        # Error domain models and Result types
│   ├── cli/            # Command-line interface definitions and parser
│   ├── core/           # Player engine, queue, playlist, session, and state
│   ├── audio/          # Audio backend abstractions and streaming traits
│   ├── sources/        # Source resolvers (local files, network streams)
│   ├── tui/            # Interactive terminal user interface
│   ├── daemon/         # Background service and IPC socket protocol
│   ├── config/         # Configuration file handling and defaults
│   └── filesystem/     # Filesystem watchers and notifications
└── tests/              # Integration and end-to-end test suites
```

---

## Documentation

Full architectural specifications and development roadmaps are available in the [docs/](docs/) directory:
- [Architecture](docs/architecture.md)
- [Summary](docs/summary.md)

---

## License

This project is licensed under the [MIT License](LICENSE).
