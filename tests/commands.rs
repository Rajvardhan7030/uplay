use clap::Parser;
use uplay::cli::{Cli, Command};

#[test]
fn test_cli_parsing_commands() {
    let args = ["uplay", "play", "my_song.flac"];
    let cli = Cli::parse_from(args);
    assert_eq!(
        cli.to_command(),
        Some(Command::Play {
            target: Some("my_song.flac".to_string())
        })
    );

    let args = ["uplay", "pause"];
    let cli = Cli::parse_from(args);
    assert_eq!(cli.to_command(), Some(Command::Pause));

    let args = ["uplay", "stop"];
    let cli = Cli::parse_from(args);
    assert_eq!(cli.to_command(), Some(Command::Stop));

    let args = ["uplay", "volume", "65"];
    let cli = Cli::parse_from(args);
    assert_eq!(cli.to_command(), Some(Command::Volume(65)));

    let args = ["uplay", "queue", "https://example.com/stream.mp3"];
    let cli = Cli::parse_from(args);
    assert_eq!(
        cli.to_command(),
        Some(Command::Queue("https://example.com/stream.mp3".into()))
    );
}
