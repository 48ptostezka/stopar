//! What a location's QR code contains: `STOPA:<game id>:<location code>`.
//!
//! The prefix and game id let the app tell "not a game code" apart from
//! "a code from a different game" and give the player a useful message.

pub const PREFIX: &str = "STOPA";

/// The text to put in a location's QR code.
pub fn encode(game_id: &str, code: &str) -> String {
    format!("{PREFIX}:{game_id}:{code}")
}

#[derive(Debug, PartialEq, Eq)]
pub enum Scanned<'a> {
    /// A code for this game; the value is the location code.
    Ours(&'a str),
    /// A Stopa code, but for another game.
    OtherGame(&'a str),
    /// Any other QR code.
    Foreign,
}

pub fn decode<'a>(text: &'a str, game_id: &str) -> Scanned<'a> {
    let mut parts = text.trim().splitn(3, ':');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(prefix), Some(game), Some(code)) if prefix.eq_ignore_ascii_case(PREFIX) && !code.is_empty() => {
            if game == game_id {
                Scanned::Ours(code)
            } else {
                Scanned::OtherGame(game)
            }
        }
        _ => Scanned::Foreign,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let text = encode("galaxy", "7Q4M-2KD9");
        assert_eq!(text, "STOPA:galaxy:7Q4M-2KD9");
        assert_eq!(decode(&text, "galaxy"), Scanned::Ours("7Q4M-2KD9"));
    }

    #[test]
    fn other_games_and_foreign_codes() {
        assert_eq!(decode("STOPA:trail:ABC123", "galaxy"), Scanned::OtherGame("trail"));
        assert_eq!(decode("https://example.com", "galaxy"), Scanned::Foreign);
        assert_eq!(decode("STOPA:galaxy:", "galaxy"), Scanned::Foreign);
    }
}
