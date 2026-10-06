# UPlay --- Project Summary

## 1. Project Vision

**UPlay** is a lightweight, Linux-first music player designed around the
Unix philosophy:

> Play music quickly, use very few resources, stay keyboard/terminal
> friendly, and get out of the user's way.

It is not intended to compete with feature-heavy desktop music
applications. Its primary purpose is to provide a fast and composable
music engine with a CLI and lightweight TUI.

### Core principles

-   Linux-first
-   Terminal-first
-   Low CPU and memory usage
-   Fast startup
-   Keyboard-driven operation
-   Local-first
-   Unix-friendly and scriptable
-   Modular playback backends
-   No mandatory account or cloud service
-   No Electron
-   GUI is optional rather than the foundation

------------------------------------------------------------------------

# 2. Feature Set

## 2.1 Local Music Playback --- MVP

-   Play individual audio files
-   Play an entire directory
-   Recursive directory scanning
-   Support common audio formats through the selected audio backend
-   Next / previous
-   Play / pause
-   Stop
-   Seek forward/backward
-   Volume control
-   Playback position
-   Automatic next-track handling

Example:

``` bash
uplay ~/Music
uplay ~/Music/song.flac
```

## 2.2 Playback Modes

-   Sequential playback
-   Shuffle/random playback
-   Repeat one
-   Repeat playlist
-   Repeat off

Example:

``` bash
uplay ~/Music --shuffle
```

## 2.3 Queue

Users can add media without destroying the current playlist.

``` bash
uplay queue ~/Downloads/song.mp3
```

The queue should support:

-   Add
-   Remove
-   Move
-   Clear
-   Add next
-   Inspect queue

## 2.4 Playlists

Persistent playlists stored locally.

Examples:

``` text
coding
workout
chill
gaming
```

Possible commands:

``` bash
uplay playlist save coding
uplay playlist load coding
uplay playlist list
```

## 2.5 Session Persistence

UPlay can remember:

-   Current playlist
-   Current track
-   Playback position
-   Volume
-   Shuffle state
-   Repeat state
-   Queue

Example:

``` bash
uplay --resume
```

## 2.6 YouTube Audio Streaming

UPlay should support playing audio from a YouTube URL without
downloading the complete video file.

Example:

``` bash
uplay "https://youtube.com/watch?v=..."
```

A YouTube URL can also become a playlist item.

Important design principle:

-   YouTube support should be an optional backend/integration.
-   The core player must remain usable without it.
-   External extraction/downloading tooling should be isolated from the
    audio engine.

## 2.7 Generic Network Streams

Future-compatible support for:

-   HTTP audio streams
-   Internet radio
-   Other direct audio URLs

Example:

``` bash
uplay "https://example.com/radio.mp3"
```

## 2.8 Terminal UI

Running:

``` bash
uplay
```

can open a lightweight TUI.

The TUI should display:

-   Current track
-   Artist/title when available
-   Playback position
-   Progress bar
-   Playlist
-   Queue
-   Playback mode
-   Volume
-   Status/errors

Suggested controls:

``` text
Space     Play/Pause
n         Next
p         Previous
s         Shuffle
r         Repeat
a         Add
d         Remove
+/-       Volume
←/→       Seek
q         Quit
/         Search
```

## 2.9 CLI Control

The player should also be controllable without opening the TUI.

Examples:

``` bash
uplay play
uplay pause
uplay next
uplay previous
uplay volume 70
uplay shuffle
uplay repeat one
```

## 2.10 Background Daemon

Optional background mode:

``` bash
uplay daemon
```

The daemon owns the playback engine while CLI/TUI clients communicate
with it.

This enables:

-   Media-key control
-   Persistent playback
-   Multiple clients
-   Shell scripting
-   Waybar/status integrations

## 2.11 Media-Key Support

Linux desktop media keys should be able to control:

-   Play/pause
-   Next
-   Previous
-   Volume

This should be implemented outside the core playback engine where
possible.

## 2.12 Unix/CLI Composition

UPlay should work well with other Linux tools.

Examples:

``` bash
find ~/Music -type f | uplay --stdin
```

``` bash
cat playlist.m3u | uplay --stdin
```

Potential future usage:

``` bash
fzf | uplay
```

The exact stdin format should be defined during implementation.

## 2.13 Filesystem Watching

Optional automatic detection of newly added music.

Example:

``` text
~/Music/
    song1.mp3
    song2.mp3
    NewSong.flac   <- detected automatically
```

This should use efficient filesystem notifications rather than constant
polling.

## 2.14 Sessions

A complete playback state can be saved as a named session.

Example:

``` bash
uplay session save coding
uplay session load coding
```

A session may contain:

-   Playlist
-   Queue
-   Current track
-   Position
-   Volume
-   Shuffle
-   Repeat

------------------------------------------------------------------------

# 3. Features Explicitly Out of Scope for MVP

These should not be allowed to inflate the first version:

-   Album-art downloading
-   Built-in web browser
-   User accounts
-   Cloud synchronization
-   Social features
-   Music recommendations
-   AI recommendations
-   Spotify account integration
-   Podcast management
-   Heavy visualizers
-   Mandatory graphical interface
-   Electron-based application shell

These can be reconsidered later as optional plugins/features.

------------------------------------------------------------------------

# 4. Suggested Technology Stack

## Primary Language: Rust

Rust is the preferred implementation language because the project
emphasizes:

-   Low resource usage
-   Performance
-   Memory safety
-   Good concurrency support
-   Excellent CLI ecosystem
-   Strong Linux support
-   Easy distribution as a single native binary

## CLI

Recommended ecosystem:

-   `clap` for command-line parsing
-   `anyhow` / `thiserror` for error handling
-   `tracing` for structured logging

## TUI

Recommended:

-   `ratatui`
-   `crossterm`

The TUI should remain a client/interface rather than containing playback
logic.

## Audio Engine

A backend abstraction should be created so the player is not permanently
tied to one audio implementation.

Candidate approaches:

-   GStreamer
-   mpv/libmpv
-   Symphonia + a separate audio output layer
-   Another mature Linux audio backend

For the first implementation, prioritize **reliable format support and
streaming** over implementing an audio decoder from scratch.

## Linux Audio

The project should work with the Linux audio stack available on modern
distributions.

Potential output integration:

-   PipeWire
-   PulseAudio compatibility where available
-   ALSA where appropriate

The core should avoid hard-coding itself to one desktop environment.

## Configuration

Recommended:

``` text
TOML
```

Example location:

``` text
~/.config/uplay/config.toml
```

## Persistence

Start simple:

-   M3U/M3U8 for portable playlists
-   TOML/JSON for configuration and session state

Introduce SQLite only when the application genuinely needs structured
history/library metadata.

## IPC

For daemon communication:

-   Unix domain socket

Example:

``` text
/run/user/<uid>/uplay.sock
```

This keeps communication local, fast, and Linux-native.

## Filesystem Events

Potential Rust library:

``` text
notify
```

for efficient filesystem change detection.

## YouTube Integration

Use an external extraction/downloading component rather than
implementing YouTube extraction logic inside the core player.

The integration should expose a simple internal interface:

``` text
URL
 ↓
Resolver
 ↓
Audio Stream
 ↓
Playback Engine
```

This keeps the core independent from external website-specific behavior.

------------------------------------------------------------------------

# 5. Suggested Project Structure

``` text
uplay/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── CONTRIBUTING.md
│
├── docs/
│   ├── summary.md
│   ├── architecture.md
│   └── roadmap.md
│
├── src/
│   ├── main.rs
│   │
│   ├── cli/
│   │   ├── mod.rs
│   │   └── commands.rs
│   │
│   ├── core/
│   │   ├── mod.rs
│   │   ├── player.rs
│   │   ├── queue.rs
│   │   ├── playlist.rs
│   │   ├── session.rs
│   │   └── state.rs
│   │
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── backend.rs
│   │   └── stream.rs
│   │
│   ├── sources/
│   │   ├── mod.rs
│   │   ├── local.rs
│   │   ├── http.rs
│   │   └── youtube.rs
│   │
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── ui.rs
│   │   └── input.rs
│   │
│   ├── daemon/
│   │   ├── mod.rs
│   │   └── ipc.rs
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs
│   │
│   ├── filesystem/
│   │   ├── mod.rs
│   │   └── watcher.rs
│   │
│   └── error.rs
│
├── tests/
│   ├── playlist.rs
│   ├── queue.rs
│   ├── commands.rs
│   └── session.rs
│
└── assets/
    └── man/
```

------------------------------------------------------------------------

# 6. Design Rule

The most important project rule:

> **Every feature must justify its CPU, memory, complexity, and
> maintenance cost.**

If a feature makes UPlay significantly heavier without improving the
core listening experience, it should not belong in the core application.
