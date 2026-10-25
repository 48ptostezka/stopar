//! The game content, embedded at build time by `build.rs`.

use std::{collections::HashMap, sync::OnceLock};

use stopa_core::Game;

static GAME_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/game.json"));
static GAME: OnceLock<Game> = OnceLock::new();

/// The game being played. Content never changes while the app runs.
pub fn game() -> &'static Game {
    GAME.get_or_init(|| serde_json::from_str(GAME_JSON).expect("game.json is checked by build.rs"))
}

static ICONS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/icons.json"));
static ICONS: OnceLock<HashMap<String, String>> = OnceLock::new();

/// The SVG markup of one of the game's own item icons (`icons/<file>`).
pub fn icon_markup(file: &str) -> Option<&'static str> {
    ICONS
        .get_or_init(|| serde_json::from_str(ICONS_JSON).expect("icons.json is written by build.rs"))
        .get(file)
        .map(String::as_str)
}
