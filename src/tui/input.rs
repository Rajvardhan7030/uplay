//! Terminal keyboard input mapping and event translation.

use crate::cli::commands::Command;

/// Translates raw character inputs into internal UPlay commands.
#[must_use]
pub fn map_key_to_command(key: char) -> Option<Command> {
    match key {
        ' ' => Some(Command::TogglePlay),
        'n' => Some(Command::Next),
        'p' => Some(Command::Previous),
        'q' => Some(Command::Stop),
        '+' | '=' => Some(Command::VolumeUp),
        '-' | '_' => Some(Command::VolumeDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_mapping() {
        assert_eq!(map_key_to_command(' '), Some(Command::TogglePlay));
        assert_eq!(map_key_to_command('n'), Some(Command::Next));
        assert_eq!(map_key_to_command('p'), Some(Command::Previous));
        assert_eq!(map_key_to_command('q'), Some(Command::Stop));
        assert_eq!(map_key_to_command('z'), None);
    }
}
