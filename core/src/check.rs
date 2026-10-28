//! Checks run on a game file at build time. Errors stop the build; warnings are printed.

use std::{
    collections::{HashSet, VecDeque},
    path::Path,
};

use indexmap::IndexMap;

use crate::content::{
    normalize_answer, Choice, Condition, Game, Id, ItemIcon, Location, Pick, Rewards, Scene, Text,
};

#[derive(Debug, Default)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Lowercase letters, digits, `_` and `-`.
fn is_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Uppercase letters, digits and `-`, 6 to 32 characters.
fn is_code(s: &str) -> bool {
    (6..=32).contains(&s.len()) && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

const IMAGE_TYPES: [&str; 5] = ["png", "svg", "jpg", "jpeg", "webp"];

/// A plain file name with a picture extension, no folders.
fn is_image_name(s: &str) -> bool {
    let plain = !s.is_empty() && !s.starts_with('.') && !s.contains(['/', '\\']);
    let ext = s.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    plain && ext.is_some_and(|e| IMAGE_TYPES.contains(&e.as_str()))
}

/// Pictures above this size (in bytes) get a warning.
const LARGE_PICTURE: u64 = 500_000;

const BUILTIN_ICONS: &str =
    "generic, map, key, coin, lantern, feather, cell, crystal, decoder, tank, letter, flask";

/// A plain file name ending in .svg, no folders.
fn is_icon_name(s: &str) -> bool {
    is_image_name(s) && s.to_ascii_lowercase().ends_with(".svg")
}

/// Reads an icon file and returns its markup, ready to place in the page:
/// the `<svg>` element and what's inside it (an XML declaration or comments
/// before it are dropped). Refuses files that could run code, since the
/// markup goes straight into the page.
pub fn load_icon(path: &Path) -> Result<String, String> {
    let name = path.display();
    let text =
        std::fs::read_to_string(path).map_err(|_| format!("icon \"{name}\" is missing or can't be read"))?;
    let start =
        text.find("<svg").ok_or_else(|| format!("icon \"{name}\" is not an SVG file (no <svg> element)"))?;
    let markup = text[start..].trim_end().to_owned();
    let lower = markup.to_ascii_lowercase();
    let runs_code = ["<script", "<foreignobject", "javascript:"].iter().any(|bad| lower.contains(bad))
        || has_event_attribute(&lower);
    if runs_code {
        return Err(format!("icon \"{name}\": remove scripts and event attributes (onload=…) from the SVG"));
    }
    Ok(markup)
}

/// True if the markup has an attribute such as `onload=` or `onclick =`.
fn has_event_attribute(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    lower.match_indices("on").any(|(i, _)| {
        let after_space = i > 0 && bytes[i - 1].is_ascii_whitespace();
        let rest = &lower[i + 2..];
        let letters = rest.bytes().take_while(u8::is_ascii_lowercase).count();
        after_space && letters > 0 && rest[letters..].trim_start().starts_with('=')
    })
}

/// Collects what the story sets and tests, so the end of the check can
/// spot typos (a flag tested but never set) and unused content.
#[derive(Default)]
struct Usage<'a> {
    given_items: HashSet<&'a str>,
    given_clues: HashSet<&'a str>,
    set_flags: HashSet<&'a str>,
    tested_flags: HashSet<&'a str>,
}

struct Checker<'a> {
    game: &'a Game,
    report: Report,
    usage: Usage<'a>,
}

impl<'a> Checker<'a> {
    fn error(&mut self, message: String) {
        self.report.errors.push(message);
    }

    fn warn(&mut self, message: String) {
        self.report.warnings.push(message);
    }

    fn text(&mut self, path: &str, t: &Text) {
        for (lang, value) in t.iter() {
            if value.trim().is_empty() {
                self.error(format!("{path}: the {} text is empty", lang.code()));
            }
        }
    }

    fn item(&mut self, path: &str, id: &str) {
        if !self.game.items.contains_key(id) {
            self.error(format!("{path}: there is no item \"{id}\""));
        }
    }

    fn clue(&mut self, path: &str, id: &str) {
        if !self.game.clues.contains_key(id) {
            self.error(format!("{path}: there is no clue \"{id}\""));
        }
    }

    fn condition(&mut self, path: &str, c: &'a Condition) {
        for (id, n) in &c.items {
            self.item(&format!("{path}.items"), id);
            if *n == 0 {
                self.error(format!("{path}.items.{id}: 0 always holds; use at least 1"));
            }
        }
        for id in &c.without_items {
            self.item(&format!("{path}.without_items"), id);
        }
        for id in c.clues.iter().chain(&c.without_clues) {
            self.clue(&format!("{path}.clues"), id);
        }
        for id in &c.visited {
            if !self.game.locations.contains_key(id) {
                self.error(format!("{path}.visited: there is no location \"{id}\""));
            }
        }
        for f in c.flags.iter().chain(&c.without_flags) {
            self.usage.tested_flags.insert(f);
        }
    }

    fn rewards(&mut self, path: &str, r: &'a Rewards) {
        for (id, n) in &r.items {
            self.item(&format!("{path}.items"), id);
            if *n == 0 {
                self.error(format!("{path}.items.{id}: the quantity must be at least 1"));
            }
            self.usage.given_items.insert(id);
        }
        for id in &r.clues {
            self.clue(&format!("{path}.clues"), id);
            self.usage.given_clues.insert(id);
        }
    }

    fn flags(&mut self, path: &str, flags: &'a [String]) {
        for f in flags {
            if !is_slug(f) {
                self.error(format!("{path}: flag \"{f}\": use lowercase letters, digits, _ or -"));
            }
            self.usage.set_flags.insert(f);
        }
    }

    fn take(&mut self, path: &str, take: &IndexMap<Id, u32>) {
        for (id, n) in take {
            self.item(path, id);
            if *n == 0 {
                self.error(format!("{path}.{id}: the quantity must be at least 1"));
            }
        }
    }

    fn target(&mut self, path: &str, loc: &Location, to: &Option<Id>) {
        if let Some(to) = to {
            if !loc.scenes.contains_key(to) {
                self.error(format!("{path}: there is no scene \"{to}\" in this location"));
            }
        }
    }

    fn location(&mut self, id: &str, loc: &'a Location) {
        let base = format!("locations.{id}");
        if loc.scenes.is_empty() {
            self.error(format!("{base}: add at least one scene"));
            return;
        }
        if let Some(start) = &loc.start {
            if !loc.scenes.contains_key(start) {
                self.error(format!("{base}.start: there is no scene \"{start}\""));
            }
        }
        for (sid, scene) in &loc.scenes {
            let path = format!("{base}.scenes.{sid}");
            if !is_slug(sid) {
                self.error(format!("{path}: scene ids use lowercase letters, digits, _ or -"));
            }
            if let Some(t) = &scene.text {
                self.text(&format!("{path}.text"), t);
            }
            if scene.text.is_none() && scene.paragraphs.is_empty() && scene.image.is_none() {
                self.warn(format!("{path}: the scene has no text and no picture"));
            }
            match &scene.image {
                Some(file) if !is_image_name(file) => self.error(format!(
                    "{path}.image \"{file}\": give a file name in the game's images folder, ending in .png, .svg, .jpg or .webp"
                )),
                Some(_) if scene.alt.is_none() => {
                    self.warn(format!("{path}: add `alt`, a short description of the picture for screen readers"))
                }
                _ => {}
            }
            if let Some(alt) = &scene.alt {
                self.text(&format!("{path}.alt"), alt);
            }
            for (i, para) in scene.paragraphs.iter().enumerate() {
                self.condition(&format!("{path}.paragraphs[{i}].when"), &para.when);
                self.text(&format!("{path}.paragraphs[{i}].text"), &para.text);
            }
            self.rewards(&format!("{path}.give"), &scene.give);
            self.flags(&format!("{path}.set"), &scene.set);

            let exits = [scene.next.is_some(), !scene.choices.is_empty(), scene.password.is_some()];
            if exits.iter().filter(|e| **e).count() > 1 {
                self.error(format!("{path}: use only one of next, choices and password"));
            }
            self.target(&format!("{path}.next"), loc, &scene.next);

            let mut ids = HashSet::new();
            for choice in &scene.choices {
                let cpath = format!("{path}.choices.{}", choice.id);
                if !is_slug(&choice.id) {
                    self.error(format!("{cpath}: choice ids use lowercase letters, digits, _ or -"));
                }
                if !ids.insert(choice.id.as_str()) {
                    self.error(format!("{cpath}: another choice in this scene has the same id"));
                }
                self.text(&format!("{cpath}.label"), &choice.label);
                if let Some(locked) = &choice.locked {
                    self.text(&format!("{cpath}.locked"), locked);
                    if choice.when == Condition::default() && choice.take.is_empty() {
                        self.warn(format!(
                            "{cpath}: `locked` is never shown, because the choice has no condition"
                        ));
                    }
                }
                self.condition(&format!("{cpath}.when"), &choice.when);
                self.take(&format!("{cpath}.take"), &choice.take);
                self.rewards(&format!("{cpath}.give"), &choice.give);
                self.flags(&format!("{cpath}.set"), &choice.set);
                self.target(&format!("{cpath}.to"), loc, &choice.to);
            }

            self.condition(&format!("{path}.requires"), &scene.requires);
            if let Some(waiting) = &scene.waiting {
                self.text(&format!("{path}.waiting"), waiting);
                if scene.requires.is_empty() {
                    self.warn(format!(
                        "{path}: `waiting` is never shown, because the scene has no `requires`"
                    ));
                }
            }
            if !scene.requires.is_empty() && !scene.has_exit() {
                self.warn(format!(
                    "{path}: `requires` does nothing here, because the scene ends the story (it has no next, choices or password)"
                ));
            }
            if scene.allow_lockout && (scene.pick == Pick::Any || scene.choices.len() < 2) {
                self.warn(format!(
                    "{path}: `allow_lockout` only matters for a final decision with several options"
                ));
            }

            if let Some(pw) = &scene.password {
                let ppath = format!("{path}.password");
                if pw.answers.is_empty() {
                    self.error(format!("{ppath}.answers: add at least one answer"));
                }
                for answer in &pw.answers {
                    if normalize_answer(answer).is_empty() {
                        self.error(format!("{ppath}.answers: \"{answer}\" has no letters or digits"));
                    }
                }
                for (key, t) in [("prompt", &pw.prompt), ("hint", &pw.hint), ("wrong", &pw.wrong)] {
                    if let Some(t) = t {
                        self.text(&format!("{ppath}.{key}"), t);
                    }
                }
                self.rewards(&format!("{ppath}.give"), &pw.give);
                self.flags(&format!("{ppath}.set"), &pw.set);
                self.target(&format!("{ppath}.to"), loc, &pw.to);
            }
        }

        // Scenes no path from the start reaches are probably a mistake.
        let Some(start) = loc.start_scene() else { return };
        let reached = reachable(loc, start, |_, _| false);
        for sid in loc.scenes.keys().filter(|s| !reached.contains(s.as_str())) {
            self.warn(format!("{base}.scenes.{sid}: no path from the start scene leads here"));
        }

        for (sid, scene) in &loc.scenes {
            if scene.pick == Pick::One && scene.choices.len() >= 2 && reached.contains(sid.as_str()) {
                self.decision(id, loc, start, sid, scene);
            }
        }
    }

    /// Checks a final decision for two ways a player could be surprised:
    /// losing an option because they decided before they had what it needs,
    /// and reaching one option's outcome without picking that option.
    fn decision(&mut self, lid: &str, loc: &'a Location, start: &str, sid: &str, scene: &'a Scene) {
        let path = format!("locations.{lid}.scenes.{sid}");

        if !scene.allow_lockout {
            let have = guaranteed(lid, loc, start, sid, scene);
            for c in &scene.choices {
                let missing: Vec<Need> = needs(c).into_iter().filter(|n| !have.holds(n)).collect();
                for n in missing {
                    let Some(other) =
                        scene.choices.iter().find(|d| d.id != c.id && !needs(d).iter().any(|m| m.covers(&n)))
                    else {
                        continue;
                    };
                    self.warn(format!(
                        "{path}: a player can make this final decision before they have {n}, which option \"{}\" needs. \
                         Picking \"{}\" then closes \"{}\" for good. Add `requires` to the scene so the decision waits, \
                         or `allow_lockout = true` if losing the option is intended",
                        c.id, other.id, c.id
                    ));
                    break;
                }
            }
        }

        let mut warned = HashSet::new();
        for c in &scene.choices {
            let Some(to) = c.to.as_deref() else { continue };
            let Some(target) = loc.scenes.get(to) else { continue };
            let gives = target.give != Rewards::default() || !target.set.is_empty();
            if to == sid || !gives || !warned.insert(to) {
                continue;
            }
            let without = reachable(loc, start, |from, t| from == sid && t == to);
            if without.contains(to) {
                self.warn(format!(
                    "locations.{lid}.scenes.{to}: players can reach this scene without picking \"{}\" at \"{sid}\", \
                     so its rewards don't depend on that decision",
                    c.id
                ));
            }
        }
    }
}

/// Scenes reachable from `start`, not using links for which `blocked(from, to)` is true.
fn reachable<'l>(
    loc: &'l Location,
    start: &'l str,
    blocked: impl Fn(&str, &str) -> bool,
) -> HashSet<&'l str> {
    let mut reached: HashSet<&str> = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(sid) = queue.pop_front() {
        let Some(scene) = loc.scenes.get(sid) else { continue };
        let targets = scene
            .next
            .iter()
            .chain(scene.choices.iter().filter_map(|c| c.to.as_ref()))
            .chain(scene.password.iter().filter_map(|p| p.to.as_ref()));
        for to in targets {
            if loc.scenes.contains_key(to) && !blocked(sid, to) && reached.insert(to) {
                queue.push_back(to);
            }
        }
    }
    reached
}

/// One thing an option needs before it can be picked.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Need<'a> {
    Item(&'a str, u32),
    Clue(&'a str),
    Flag(&'a str),
    Visited(&'a str),
}

impl Need<'_> {
    /// True if needing `self` means also needing `other`.
    fn covers(&self, other: &Need) -> bool {
        match (self, other) {
            (Need::Item(a, n), Need::Item(b, m)) => a == b && n >= m,
            _ => self == other,
        }
    }
}

impl std::fmt::Display for Need<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Need::Item(id, n) => write!(f, "item \"{id}\" ×{n}"),
            Need::Clue(id) => write!(f, "clue \"{id}\""),
            Need::Flag(id) => write!(f, "flag \"{id}\""),
            Need::Visited(id) => write!(f, "a visit to \"{id}\""),
        }
    }
}

fn needs(c: &Choice) -> Vec<Need<'_>> {
    let mut items: IndexMap<&str, u32> = IndexMap::new();
    for (id, n) in c.when.items.iter().chain(&c.take) {
        let e = items.entry(id.as_str()).or_default();
        *e = (*e).max(*n);
    }
    items
        .into_iter()
        .map(|(id, n)| Need::Item(id, n))
        .chain(c.when.clues.iter().map(|id| Need::Clue(id)))
        .chain(c.when.flags.iter().map(|id| Need::Flag(id)))
        .chain(c.when.visited.iter().map(|id| Need::Visited(id)))
        .collect()
}

/// What a player is sure to have when a decision is offered.
#[derive(Default)]
struct Have<'a> {
    items: IndexMap<&'a str, u32>,
    clues: HashSet<&'a str>,
    flags: HashSet<&'a str>,
    visited: HashSet<&'a str>,
}

impl Have<'_> {
    fn holds(&self, need: &Need) -> bool {
        match need {
            Need::Item(id, n) => self.items.get(id).is_some_and(|have| have >= n),
            Need::Clue(id) => self.clues.contains(id),
            Need::Flag(id) => self.flags.contains(id),
            Need::Visited(id) => self.visited.contains(id),
        }
    }
}

/// What a player surely has at decision scene `sid`: what the scene requires,
/// and what every scene on every path from the start to it gives (including
/// itself). An approximation: items used up on the way still count, and links
/// that only some players can take (because of conditions) count as open, so
/// the lockout warning can occasionally miss a case or report one that can't
/// happen. `allow_lockout = true` silences a warning that is wrong.
fn guaranteed<'a>(lid: &'a str, loc: &'a Location, start: &str, sid: &str, scene: &'a Scene) -> Have<'a> {
    let mut have = Have::default();
    have.visited.insert(lid);
    let r = &scene.requires;
    for (id, n) in &r.items {
        have.items.insert(id, *n);
    }
    have.clues.extend(r.clues.iter().map(String::as_str));
    have.flags.extend(r.flags.iter().map(String::as_str));
    have.visited.extend(r.visited.iter().map(String::as_str));

    // Scenes on every path to `sid`: without them, `sid` can't be reached.
    let on_every_path = loc.scenes.iter().filter(|(x, _)| {
        x.as_str() == start
            || x.as_str() == sid
            || !reachable(loc, start, |from, to| from == x.as_str() || to == x.as_str()).contains(sid)
    });
    let mut gained: IndexMap<&str, u32> = IndexMap::new();
    for (_, s) in on_every_path {
        for (id, n) in &s.give.items {
            *gained.entry(id).or_default() += n;
        }
        have.clues.extend(s.give.clues.iter().map(String::as_str));
        have.flags.extend(s.set.iter().map(String::as_str));
    }
    for (id, n) in gained {
        let e = have.items.entry(id).or_default();
        *e = (*e).max(n);
    }
    have
}

impl Game {
    pub fn check(&self) -> Report {
        let mut c = Checker { game: self, report: Report::default(), usage: Usage::default() };

        if !is_slug(&self.game.id) {
            c.error(format!("game.id \"{}\": use lowercase letters, digits, _ or -", self.game.id));
        }
        if !is_slug(&self.game.skin) {
            c.error(format!("game.skin \"{}\": use lowercase letters, digits, _ or -", self.game.skin));
        }
        c.text("game.title", &self.game.title);

        for (id, item) in &self.items {
            if !is_slug(id) {
                c.error(format!("items.{id}: ids use lowercase letters, digits, _ or -"));
            }
            c.text(&format!("items.{id}.name"), &item.name);
            c.text(&format!("items.{id}.description"), &item.description);
            if let ItemIcon::File(file) = &item.icon {
                if !is_icon_name(file) {
                    c.error(format!(
                        "items.{id}.icon \"{file}\": use a built-in icon ({BUILTIN_ICONS}) \
                         or the name of an .svg file in the game's icons folder"
                    ));
                }
            }
        }
        for (id, clue) in &self.clues {
            if !is_slug(id) {
                c.error(format!("clues.{id}: ids use lowercase letters, digits, _ or -"));
            }
            c.text(&format!("clues.{id}.title"), &clue.title);
            c.text(&format!("clues.{id}.text"), &clue.text);
        }

        let mut codes = HashSet::new();
        for (id, loc) in &self.locations {
            if !is_slug(id) {
                c.error(format!("locations.{id}: ids use lowercase letters, digits, _ or -"));
            }
            if !is_code(&loc.code) {
                c.error(format!(
                    "locations.{id}.code \"{}\": use 6 to 32 uppercase letters, digits or -",
                    loc.code
                ));
            }
            if !codes.insert(loc.code.as_str()) {
                c.error(format!("locations.{id}.code \"{}\" is used by another location", loc.code));
            }
            c.text(&format!("locations.{id}.name"), &loc.name);
            c.location(id, loc);
        }

        if self.locations.is_empty() {
            c.warn("the game has no locations".into());
        }
        let Usage { given_items, given_clues, set_flags, tested_flags } = std::mem::take(&mut c.usage);
        for id in self.items.keys().filter(|id| !given_items.contains(id.as_str())) {
            c.warn(format!("item \"{id}\" is never given"));
        }
        for id in self.clues.keys().filter(|id| !given_clues.contains(id.as_str())) {
            c.warn(format!("clue \"{id}\" is never given"));
        }
        let mut never_set: Vec<_> = tested_flags.difference(&set_flags).collect();
        never_set.sort();
        for f in never_set {
            c.warn(format!("flag \"{f}\" is tested but never set (a typo?)"));
        }
        c.report
    }

    /// Errors for icon files missing from `dir` (the game's icons folder) or
    /// not usable as icons.
    pub fn check_icons(&self, dir: &Path) -> Vec<String> {
        self.icon_files()
            .into_iter()
            .filter(|f| is_icon_name(f))
            .filter_map(|f| load_icon(&dir.join(f)).err())
            .collect()
    }

    /// Warnings for large files in `dir` (the game's images folder). Players
    /// download every one of them for offline play, used or not, so big
    /// photos make the first download slow on a weak connection.
    pub fn large_pictures(dir: &Path) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
        let mut warnings: Vec<String> = entries
            .flatten()
            .filter_map(|e| Some((e.file_name().to_string_lossy().into_owned(), e.metadata().ok()?.len())))
            .filter(|(_, size)| *size > LARGE_PICTURE)
            .map(|(name, size)| {
                format!(
                    "picture \"{name}\" is {:.1} MB. Players download every picture for offline play, so on a slow \
                     connection the first download takes long. About 1000 px wide and under 300 KB is plenty on a phone",
                    size as f64 / 1_000_000.0
                )
            })
            .collect();
        warnings.sort();
        warnings
    }

    /// Errors for pictures missing from `dir` (the game's images folder).
    pub fn check_images(&self, dir: &Path) -> Vec<String> {
        self.images()
            .into_iter()
            .filter(|f| !dir.join(f).is_file())
            .map(|f| format!("picture \"{f}\" is not in {}", dir.display()))
            .collect()
    }
}

#[cfg(all(test, feature = "toml"))]
mod tests {
    use super::*;

    const GALAXY: &str = include_str!("../../games/galaxy/game.toml");
    const FIXTURE: &str = include_str!("fixture.toml");

    fn errors(text: &str) -> Vec<String> {
        Game::from_toml(text).unwrap().check().errors
    }

    #[test]
    fn shipped_galaxy_game_is_valid() {
        let game = Game::from_toml(GALAXY).expect("galaxy game parses");
        let report = game.check();
        assert!(report.errors.is_empty(), "{:#?}", report.errors);
        assert!(report.warnings.is_empty(), "{:#?}", report.warnings);
        let images = Path::new(env!("CARGO_MANIFEST_DIR")).join("../games/galaxy/images");
        assert!(game.check_images(&images).is_empty(), "{:#?}", game.check_images(&images));
    }

    #[test]
    fn shipped_testing_game_is_valid() {
        let game =
            Game::from_toml(include_str!("../../games/testing/game.toml")).expect("testing game parses");
        let report = game.check();
        assert!(report.errors.is_empty(), "{:#?}", report.errors);
        assert!(report.warnings.is_empty(), "{:#?}", report.warnings);
        let images = Path::new(env!("CARGO_MANIFEST_DIR")).join("../games/testing/images");
        assert!(game.check_images(&images).is_empty(), "{:#?}", game.check_images(&images));
        let icons = Path::new(env!("CARGO_MANIFEST_DIR")).join("../games/testing/icons");
        assert!(game.check_icons(&icons).is_empty(), "{:#?}", game.check_icons(&icons));
        assert_eq!(game.icon_files(), ["propustka.svg"]);
        // The guard's lockout is intended; without allow_lockout the build would warn.
        let strict = include_str!("../../games/testing/game.toml").replace("allow_lockout = true\n", "");
        let w = Game::from_toml(&strict).unwrap().check().warnings;
        assert!(w.iter().any(|w| w.contains("guard.scenes.gate") && w.contains("closes")), "{w:#?}");
    }

    #[test]
    fn tables_keep_file_order() {
        let game = Game::from_toml(GALAXY).unwrap();
        let locations: Vec<_> = game.locations.keys().map(String::as_str).collect();
        assert_eq!(locations, ["relay_k7", "wreck_vega", "crystal_cave", "radio_mast"]);
        assert_eq!(game.items.keys().next().map(String::as_str), Some("star_chart"));
    }

    #[test]
    fn broken_references_are_errors() {
        let e =
            errors(&FIXTURE.replace("give = { items = { coin = 2 } }", "give = { items = { coins = 2 } }"));
        assert!(e.iter().any(|e| e.contains("there is no item \"coins\"")), "{e:#?}");
        let e = errors(&FIXTURE.replace("to = \"room\"", "to = \"rooom\""));
        assert!(e.iter().any(|e| e.contains("there is no scene \"rooom\"")), "{e:#?}");
    }

    #[test]
    fn a_scene_has_one_way_out() {
        let e = errors(&FIXTURE.replace("pick = \"any\"", "pick = \"any\"\nnext = \"room\""));
        assert!(e.iter().any(|e| e.contains("only one of next, choices and password")), "{e:#?}");
    }

    #[test]
    fn misspelled_flags_are_reported() {
        let report = Game::from_toml(&FIXTURE.replace("flags = [\"friend\"]", "flags = [\"freind\"]"))
            .unwrap()
            .check();
        assert!(
            report.warnings.iter().any(|w| w.contains("\"freind\" is tested but never set")),
            "{:#?}",
            report.warnings
        );
    }

    #[test]
    fn pictures_must_be_plain_file_names() {
        let e = errors(&FIXTURE.replace(
            "text = { cs = \"Zámek.\", en = \"A lock.\" }",
            "text = { cs = \"Z\", en = \"L\" }\nimage = \"../secret.png\"",
        ));
        assert!(e.iter().any(|e| e.contains("../secret.png")), "{e:#?}");
    }

    fn warnings(text: &str) -> Vec<String> {
        let report = Game::from_toml(text).unwrap().check();
        assert!(report.errors.is_empty(), "{:#?}", report.errors);
        report.warnings
    }

    const VAULT_REQUIRES: &str = "requires = { items = { key = 1 } }\n";

    #[test]
    fn the_fixture_has_no_warnings() {
        // In particular: the guard's "pay" needs a coin, but the hall before it
        // always gives two, and the vault's decision waits for the key.
        assert!(warnings(FIXTURE).is_empty(), "{:#?}", warnings(FIXTURE));
    }

    #[test]
    fn a_decision_that_can_lose_an_option_is_reported() {
        let text = FIXTURE.replace(VAULT_REQUIRES, "");
        let w = warnings(&text);
        assert!(
            w.iter().any(|w| w.contains("vault.scenes.safe")
                && w.contains("item \"key\" ×1")
                && w.contains("Picking \"gold\" then closes \"map\"")),
            "{w:#?}"
        );
        // Also: `waiting` without `requires` is never shown.
        assert!(w.iter().any(|w| w.contains("`waiting` is never shown")), "{w:#?}");

        // Intended lockout: silenced.
        let text = FIXTURE.replace(VAULT_REQUIRES, "allow_lockout = true\n");
        assert!(!warnings(&text).iter().any(|w| w.contains("closes")), "{:#?}", warnings(&text));

        // A coin fewer than the hall gives is still covered; more than it gives is not.
        assert!(!warnings(&FIXTURE.replace("take = { coin = 1 }", "take = { coin = 2 }"))
            .iter()
            .any(|w| w.contains("closes")));
        let w = warnings(&FIXTURE.replace("take = { coin = 1 }", "take = { coin = 3 }"));
        assert!(w.iter().any(|w| w.contains("door.scenes.room") && w.contains("item \"coin\" ×3")), "{w:#?}");
    }

    #[test]
    fn an_outcome_reachable_without_its_option_is_reported() {
        let text = FIXTURE.replace(
            "text = { cs = \"Bohatství.\", en = \"Riches.\" }",
            "text = { cs = \"Bohatství.\", en = \"Riches.\" }\nnext = \"wise\"",
        );
        let w = warnings(&text);
        assert!(
            w.iter().any(|w| w.contains("vault.scenes.wise") && w.contains("without picking \"map\"")),
            "{w:#?}"
        );
    }

    #[test]
    fn requires_on_an_ending_scene_is_reported() {
        let text = FIXTURE.replace(
            "text = { cs = \"Konec.\", en = \"The end.\" }",
            "text = { cs = \"Konec.\", en = \"The end.\" }\nrequires = { clues = [\"knock\"] }",
        );
        assert!(warnings(&text)
            .iter()
            .any(|w| w.contains("door.scenes.end") && w.contains("`requires` does nothing")));
        let e = errors(&FIXTURE.replace(VAULT_REQUIRES, "requires = { items = { keys = 1 } }\n"));
        assert!(e.iter().any(|e| e.contains("safe.requires.items") && e.contains("\"keys\"")), "{e:#?}");
    }

    #[test]
    fn item_icons_are_built_in_names_or_svg_files() {
        let with_icon =
            |icon: &str| FIXTURE.replacen("[items.key]\n", &format!("[items.key]\nicon = \"{icon}\"\n"), 1);
        let game = Game::from_toml(&with_icon("key")).unwrap();
        assert_eq!(game.items["key"].icon, ItemIcon::Builtin(crate::content::Icon::Key));
        let game = Game::from_toml(&with_icon("wheel.svg")).unwrap();
        assert_eq!(game.items["key"].icon, ItemIcon::File("wheel.svg".into()));
        assert!(game.check().errors.is_empty());
        // A misspelt built-in name, or a path, is reported with the list of names.
        for bad in ["lanternn", "../wheel.svg", "wheel.png"] {
            let e = errors(&with_icon(bad));
            assert!(e.iter().any(|e| e.contains(bad) && e.contains("generic, map, key")), "{bad}: {e:#?}");
        }
    }

    #[test]
    fn icon_files_must_exist_be_svg_and_run_no_code() {
        let dir = std::env::temp_dir().join(format!("stopa-icons-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, text: &str| std::fs::write(dir.join(name), text).unwrap();
        write(
            "ok.svg",
            "<?xml version=\"1.0\"?>\n<!-- c -->\n<svg viewBox=\"0 0 1 1\"><path d=\"M0 0\"/></svg>\n",
        );
        write("png.svg", "not an svg");
        write("script.svg", "<svg><script>alert(1)</script></svg>");
        write("onload.svg", "<svg onload = \"x()\"><path/></svg>");
        write("fine-words.svg", "<svg><path d=\"M0 0\" class=\"one done\"/></svg>");
        assert_eq!(
            load_icon(&dir.join("ok.svg")).unwrap(),
            "<svg viewBox=\"0 0 1 1\"><path d=\"M0 0\"/></svg>"
        );
        assert!(load_icon(&dir.join("fine-words.svg")).is_ok());
        assert!(load_icon(&dir.join("png.svg")).unwrap_err().contains("not an SVG"));
        assert!(load_icon(&dir.join("script.svg")).unwrap_err().contains("remove scripts"));
        assert!(load_icon(&dir.join("onload.svg")).unwrap_err().contains("remove scripts"));
        assert!(load_icon(&dir.join("missing.svg")).unwrap_err().contains("missing"));
        let game =
            Game::from_toml(&FIXTURE.replacen("[items.key]\n", "[items.key]\nicon = \"missing.svg\"\n", 1))
                .unwrap();
        assert_eq!(game.check_icons(&dir).len(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn large_pictures_are_reported() {
        let dir = std::env::temp_dir().join(format!("stopa-pictures-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("small.svg"), vec![b' '; 1000]).unwrap();
        std::fs::write(dir.join("photo.jpg"), vec![0u8; 2_400_000]).unwrap();
        let w = Game::large_pictures(&dir);
        assert_eq!(w.len(), 1, "{w:#?}");
        assert!(w[0].contains("\"photo.jpg\" is 2.4 MB"), "{w:#?}");
        assert!(Game::large_pictures(&dir.join("missing")).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unknown_fields_are_rejected() {
        assert!(Game::from_toml(&GALAXY.replacen("[game]", "[game]\ncolour = \"red\"", 1)).is_err());
        assert!(Game::from_toml(&FIXTURE.replace("pick = \"any\"", "pik = \"any\"")).is_err());
    }
}
