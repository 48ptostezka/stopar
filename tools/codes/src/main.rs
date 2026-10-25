//! Prints a page with one QR code per location of a game, for testing on a
//! screen or printing and putting up at the real places.
//!
//!     cargo run -p stopa-codes -- games/galaxy/game.toml > codes.html
//!
//! The game file is checked first, with the same checks as the app build.

use std::{env, fs, path::Path, process};

use qrcode::{render::svg, EcLevel, QrCode};
use stopa_core::{payload, Game};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: stopa-codes <path to game.toml> > codes.html");
        process::exit(2);
    };
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("can't read {path}: {e}");
        process::exit(1);
    });
    let game = Game::from_toml(&text).unwrap_or_else(|e| {
        eprintln!("{path} is not a valid game file:\n{e}");
        process::exit(1);
    });
    let mut report = game.check();
    let folder = Path::new(&path).parent().unwrap_or(Path::new("."));
    report.errors.extend(game.check_images(&folder.join("images")));
    report.errors.extend(game.check_icons(&folder.join("icons")));
    for w in &report.warnings {
        eprintln!("warning: {w}");
    }
    if !report.is_ok() {
        for e in &report.errors {
            eprintln!("error: {e}");
        }
        process::exit(1);
    }
    print!("{}", page(&game));
}

fn page(game: &Game) -> String {
    let mut cards = String::new();
    for (_, location) in &game.locations {
        let text = payload::encode(&game.game.id, &location.code);
        // Level M survives some dirt and glare; 260 px prints at about 7 cm.
        let code =
            QrCode::with_error_correction_level(text.as_bytes(), EcLevel::M).expect("code fits in a QR code");
        let image = code.render::<svg::Color>().min_dimensions(260, 260).quiet_zone(true).build();
        cards.push_str(&format!(
            "<figure>{image}<figcaption><strong>{cs}</strong><code>{text}</code><small>Hra pro děti – prosím nesundávejte, po hře vše uklidíme sami. Díky 🖤 48. PTO Stezka | stezka.org</small></figcaption></figure>\n",
            cs = escape(&location.name.cs),
            text = escape(&text),
        ));
    }
    format!(
        r#"<!doctype html>
<html lang="cs">
<head>
<meta charset="utf-8">
<title>{title}: QR codes</title>
<style>
  body {{ font-family: system-ui, sans-serif; margin: 24px; color: #111; }}
  h1 {{ font-size: 20px; }}
  main {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px; }}
  figure {{ margin: 0; padding: 16px; border: 1.5px dashed #999; break-inside: avoid; text-align: center; }}
  figure svg {{ width: 260px; height: auto; }}
  figcaption {{ display: grid; gap: 4px; margin-top: 8px; }}
  code, small {{ color: #555; }}
  @media print {{ body {{ margin: 0; }} h1 {{ display: none; }} }}
</style>
</head>
<body>
<h1>{title} ({id}): {count} locations</h1>
<main>
{cards}</main>
</body>
</html>
"#,
        title = escape(&game.game.title.en),
        id = escape(&game.game.id),
        count = game.locations.len(),
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
