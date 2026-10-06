# UPlay --- Architecture

## 1. Architectural Goal

UPlay should be designed as a **small music engine with multiple
interfaces**, not as a TUI application with playback code embedded
inside the UI.

The central architecture is:

``` text
                         ┌─────────────────────┐
                         │      User           │
                         └──────────┬──────────┘
                                    │
                  ┌─────────────────┼─────────────────┐
                  │                 │                 │
                  ▼                 ▼                 ▼
             CLI Client        TUI Client       Media Keys
                  │                 │                 │
                  └─────────────────┼─────────────────┘
                                    ▼
                           ┌────────────────┐
                           │  IPC / Control │
                           └───────┬────────┘
                                   │
                         ┌─────────▼─────────┐
                         │    UPlay Core     │
                         └─────────┬─────────┘
                                   │
          ┌────────────────────────┼────────────────────────┐
          │                        │                        │
          ▼                        ▼                        ▼
    Playlist/Queue           Session/State             Resolver
                                                            │
                                ┌───────────────────────────┼──────────────┐
                                │                           │              │
                                ▼                           ▼              ▼
                           Local Files                 YouTube         HTTP Stream
                                │                           │              │
                                └───────────────────────────┼──────────────┘
                                                            ▼
                                                   ┌─────────────────┐
                                                   │ Audio Backend   │
                                                   └────────┬────────┘
                                                            │
                                                            ▼
                                                   PipeWire / ALSA
```

------------------------------------------------------------------------

# 2. Architectural Principles

## 2.1 Separation of Concerns

The following components should not know unnecessary details about one
another.

For example:

-   TUI should not decode audio.
-   CLI should not manage audio buffers.
-   YouTube resolver should not manage playlists.
-   Playlist code should not know whether an item came from YouTube or a
    local file.
-   Audio backend should not know anything about terminal rendering.

This makes the system easier to test and extend.

------------------------------------------------------------------------

# 3. Core Domain Model

The central object is a media item.

Conceptually:

``` text
MediaItem
├── id
├── source
├── metadata
└── playback information
```

The source may be:

``` text
LocalFile
HttpStream
YouTube
```

Conceptually:

``` rust
enum Source {
    LocalFile(PathBuf),
    HttpStream(Url),
    YouTube(Url),
}
```

The exact implementation may change.

------------------------------------------------------------------------

# 4. Player State

The player should have one authoritative state.

Example:

``` text
PlayerState
├── status
│   ├── Stopped
│   ├── Playing
│   └── Paused
│
├── current_item
├── position
├── volume
├── playback_mode
├── playlist
└── queue
```

Playback mode:

``` text
Sequential
Shuffle
RepeatOne
RepeatPlaylist
```

The UI should read this state rather than maintaining a second copy of
it.

------------------------------------------------------------------------

# 5. Player Engine

The Player Engine is the heart of UPlay.

Responsibilities:

-   Start playback
-   Pause
-   Resume
-   Stop
-   Seek
-   Change volume
-   Move to next item
-   Move to previous item
-   Select random item
-   Apply repeat rules
-   Manage current playback state
-   Report playback events

It should not:

-   Draw the TUI
-   Parse shell arguments
-   Render terminal output
-   Directly implement YouTube extraction

------------------------------------------------------------------------

# 6. Queue Manager

The queue manager handles upcoming media.

Operations:

``` text
add()
add_next()
remove()
move()
clear()
peek()
next()
```

Example:

``` text
Current:
    Song A

Queue:
    Song B
    Song C
    YouTube X
```

The queue should be independent from the persistent playlist.

------------------------------------------------------------------------

# 7. Playlist Manager

The playlist manager handles saved collections.

Responsibilities:

-   Create playlist
-   Load playlist
-   Save playlist
-   Delete playlist
-   Rename playlist
-   Add item
-   Remove item

Storage should initially use simple playlist files.

Example:

``` text
~/.config/uplay/playlists/
├── coding.m3u8
├── workout.m3u8
└── chill.m3u8
```

------------------------------------------------------------------------

# 8. Source/Resolver Layer

A major architectural requirement is separating **where media comes
from** from **how media is played**.

The resolver converts:

``` text
Input
```

into:

``` text
Playable Audio Source
```

Example:

``` text
/home/user/Music/song.flac
        ↓
Local Resolver
        ↓
Playable Stream


https://youtube.com/watch?v=123
        ↓
YouTube Resolver
        ↓
Audio Stream


https://example.com/radio.mp3
        ↓
HTTP Resolver
        ↓
Audio Stream
```

The player engine should not care which resolver produced the stream.

------------------------------------------------------------------------

# 9. Audio Backend

The audio backend is responsible for actual audio playback.

Interface concept:

``` text
AudioBackend
├── load()
├── play()
├── pause()
├── stop()
├── seek()
├── set_volume()
├── position()
└── duration()
```

The implementation can use a mature audio/media engine rather than
reinventing codecs.

This is important because implementing a full media decoder stack would
dramatically increase project complexity.

------------------------------------------------------------------------

# 10. TUI Architecture

The TUI should behave like a client.

``` text
User Input
    ↓
Input Handler
    ↓
Command
    ↓
UPlay Controller
    ↓
Core State
    ↓
UI Renderer
```

The TUI should never directly modify low-level audio state.

Example:

``` text
User presses Space
        ↓
TUI input handler
        ↓
Pause command
        ↓
Player Engine
        ↓
State becomes Paused
        ↓
TUI receives updated state
        ↓
Screen redraw
```

------------------------------------------------------------------------

# 11. CLI Architecture

CLI commands should map to the same internal command system used by the
TUI.

Example:

``` bash
uplay next
```

should conceptually become:

``` text
Command::Next
```

The TUI pressing `n` should also become:

``` text
Command::Next
```

This prevents duplicate business logic.

------------------------------------------------------------------------

# 12. Daemon Architecture

The daemon is optional.

When enabled:

``` text
                  ┌─────────────┐
                  │ uplay daemon│
                  └──────┬──────┘
                         │
                    Unix Socket
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
             CLI         TUI      Scripts
```

The daemon owns:

-   Player Engine
-   Queue
-   Current state
-   Audio backend
-   Session state

Clients send commands.

Example:

``` text
CLI:
    NEXT

Daemon:
    receives NEXT

Core:
    advances queue

Audio:
    loads next track

Daemon:
    broadcasts state update

TUI:
    redraws
```

------------------------------------------------------------------------

# 13. IPC Protocol

A simple structured protocol should be used.

Possible format:

``` json
{
  "command": "next"
}
```

Response:

``` json
{
  "status": "ok"
}
```

For state:

``` json
{
  "event": "state_changed",
  "track": "Song.mp3",
  "position": 83,
  "duration": 241,
  "playing": true
}
```

The protocol should be versioned if it becomes public.

------------------------------------------------------------------------

# 14. Configuration Architecture

Configuration:

``` text
~/.config/uplay/config.toml
```

Possible settings:

``` toml
music_directory = "~/Music"
volume = 80
shuffle = false
repeat = "playlist"
resume = true
```

The configuration layer should:

1.  Load defaults
2.  Load user configuration
3.  Validate values
4.  Apply runtime configuration

------------------------------------------------------------------------

# 15. Session Architecture

Session state should be separate from static configuration.

Configuration answers:

> How does the user want UPlay configured?

Session answers:

> What was UPlay doing when the user last closed it?

Example session:

``` text
playlist = coding
current_item = 17
position = 142
volume = 65
shuffle = true
repeat = playlist
```

------------------------------------------------------------------------

# 16. Filesystem Watcher

The watcher should be optional.

``` text
Filesystem
    ↓
notify
    ↓
Watcher
    ↓
Library/Playlist update
```

Avoid:

``` text
while true:
    scan entire ~/Music
    sleep(1)
```

Prefer OS filesystem notifications.

------------------------------------------------------------------------

# 17. Error Handling

Errors should be categorized.

``` text
InputError
ConfigError
FileError
NetworkError
ResolverError
AudioError
IpcError
```

User-facing output should be concise:

``` text
Error: unable to open song.flac
Reason: unsupported format
```

Debug mode can expose more detail:

``` bash
uplay --verbose
```

------------------------------------------------------------------------

# 18. Logging

Use structured logging rather than printing debug messages everywhere.

Normal mode:

``` text
quiet
```

Verbose mode:

``` bash
uplay --verbose
```

Debug mode:

``` bash
RUST_LOG=debug uplay
```

Logging should never continuously consume resources during normal
playback.

------------------------------------------------------------------------

# 19. Concurrency Model

Potential runtime responsibilities:

``` text
Main Thread
    └── command/control

Audio Thread
    └── playback

IPC
    └── socket communication

Resolver Tasks
    └── network resolution

Filesystem Watcher
    └── file events
```

Avoid unnecessary threads.

Use asynchronous tasks only where they solve a real problem,
particularly network operations and IPC.

------------------------------------------------------------------------

# 20. Security and Safety Boundaries

Network URLs should be treated as untrusted input.

Important rules:

-   Validate URLs
-   Avoid shelling out with unescaped user input
-   Never construct shell commands by string concatenation
-   Restrict filesystem operations to intended paths where possible
-   Do not execute arbitrary downloaded content
-   Keep external resolver tooling isolated

------------------------------------------------------------------------

# 21. Testing Architecture

Testing should exist at multiple levels.

### Unit Tests

Test:

-   Queue
-   Playlist
-   Shuffle
-   Repeat logic
-   Session serialization
-   Command parsing
-   Configuration

### Integration Tests

Test:

``` text
CLI → Core
TUI → Core
IPC → Core
Resolver → Playback
```

### Manual Linux Tests

Test on:

-   Fedora
-   Arch
-   Ubuntu/Debian

Test:

-   PipeWire
-   Different terminal emulators
-   Media keys
-   Network streams
-   Large playlists

------------------------------------------------------------------------

# 22. Architectural Success Criteria

The architecture is successful if:

1.  A new UI can be added without rewriting playback.
2.  A new media source can be added without rewriting the queue.
3.  A new audio backend can be added without rewriting the CLI.
4.  The daemon can run without the TUI.
5.  The CLI can operate without the daemon for simple use cases.
6.  UPlay remains useful with no network connection.
7.  Core playback remains lightweight.
