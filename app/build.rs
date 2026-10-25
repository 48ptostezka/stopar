//! Picks the game to build, checks it, and hands it to the app as JSON:
//! `game.json` (the game) and `icons.json` (the markup of its own item icons).
//!
//! `STOPA_GAME` names a folder in `games/` (default `galaxy`).
//! `STOPA_SKIN` overrides the game's skin, for previewing another look.

use std::{collections::BTreeMap, env, fs, path::PathBuf, process};

use stopa_core::{check::load_icon, Game};

fn fail(message: String) -> ! {
    eprintln!("\n  {message}\n");
    process::exit(1);
}

fn main() {
    let app_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let game_name = env::var("STOPA_GAME").unwrap_or_else(|_| "galaxy".into());
    let path = app_dir.join("../games").join(&game_name).join("game.toml");

    println!("cargo:rerun-if-env-changed=STOPA_GAME");
    println!("cargo:rerun-if-env-changed=STOPA_SKIN");
    println!("cargo:rerun-if-changed={}", path.display());

    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| fail(format!("Can't read the game file {}: {e}", path.display())));
    let mut game = Game::from_toml(&text)
        .unwrap_or_else(|e| fail(format!("{} is not a valid game file:\n{e}", path.display())));

    if let Ok(skin) = env::var("STOPA_SKIN") {
        game.game.skin = skin;
    }

    let images = app_dir.join("../games").join(&game_name).join("images");
    let icons = app_dir.join("../games").join(&game_name).join("icons");
    let mut report = game.check();
    report.errors.extend(game.check_images(&images));
    report.errors.extend(game.check_icons(&icons));
    for warning in &report.warnings {
        println!("cargo:warning={}: {warning}", path.display());
    }
    if !report.is_ok() {
        let list = report.errors.iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n");
        fail(format!("{} has {} problem(s):\n{list}", path.display(), report.errors.len()));
    }

    // New or changed pictures and icons should rebuild (and re-check) the game.
    println!("cargo:rerun-if-changed={}", images.display());
    println!("cargo:rerun-if-changed={}", icons.display());

    let skin_css = app_dir.join("skins").join(&game.game.skin).join("skin.css");
    if !skin_css.exists() {
        fail(format!("The skin \"{}\" does not exist: {} is missing", game.game.skin, skin_css.display()));
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out.join("game.json"), serde_json::to_string(&game).unwrap()).unwrap();

    // The game's own icons go into the app as markup, so they take the
    // skin's colour like the built-in ones and work offline. Checked above.
    let markup: BTreeMap<&str, String> = game
        .icon_files()
        .into_iter()
        .map(|f| (f, load_icon(&icons.join(f)).unwrap_or_else(|e| fail(e))))
        .collect();
    fs::write(out.join("icons.json"), serde_json::to_string(&markup).unwrap()).unwrap();
}
