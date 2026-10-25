//! App-wide state: the player's progress and settings as signals,
//! loaded from and saved to localStorage.

use leptos::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use stopa_core::{
    payload::{self, Scanned},
    Found, Id, Lang, Outcome, Progress,
};

use crate::{
    game::game,
    i18n::{self, Ui},
    nav::{Nav, Screen},
    platform,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub lang: Lang,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub progress: RwSignal<Progress>,
    pub settings: RwSignal<Settings>,
    pub nav: Nav,
    /// What the player just received, shown on the scene they arrived at.
    pub arrival: RwSignal<Option<Arrival>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arrival {
    pub location: Id,
    pub scene: Id,
    pub found: Found,
}

pub enum ScanOutcome {
    /// A location and the scene to open.
    Location(Id, Id),
    OtherGame,
    Unknown,
}

/// Bump when the saved format changes in a way old data can't be read as.
const PROGRESS_VERSION: u32 = 2; // 2: location stories made of scenes
const SETTINGS_VERSION: u32 = 1;
const SETTINGS_KEY: &str = "stopa/settings";

fn progress_key() -> String {
    format!("stopa/{}/progress", game().game.id)
}

#[derive(Serialize, Deserialize)]
struct Saved<T> {
    version: u32,
    data: T,
}

fn load<T: DeserializeOwned>(key: &str, version: u32) -> Option<T> {
    let text = platform::storage_get(key)?;
    let saved: Saved<T> = serde_json::from_str(&text).ok()?;
    (saved.version == version).then_some(saved.data)
}

fn save<T: Serialize>(key: &str, version: u32, data: &T) {
    if let Ok(text) = serde_json::to_string(&Saved { version, data }) {
        platform::storage_set(key, &text);
    }
}

impl AppState {
    pub fn new() -> AppState {
        let g = game();

        let mut progress: Progress = load(&progress_key(), PROGRESS_VERSION).unwrap_or_default();
        progress.retain_known(g);
        let lang = platform::browser_language()
            .as_deref()
            .and_then(Lang::from_code)
            .unwrap_or(g.game.default_language);
        let settings = load(SETTINGS_KEY, SETTINGS_VERSION).unwrap_or(Settings { lang });

        let state = AppState {
            progress: RwSignal::new(progress),
            settings: RwSignal::new(settings),
            nav: Nav::new(),
            arrival: RwSignal::new(None),
        };

        let (progress, nav) = (state.progress, state.nav);
        Effect::new(move |_| progress.with(|p| save(&progress_key(), PROGRESS_VERSION, p)));
        // Another tab or window of the game saved progress (or deleted it):
        // take it over, so a stale copy here can't overwrite it or offer
        // options that were already closed there.
        platform::on_storage_change(move |key| {
            if key.as_deref().is_none_or(|k| k == progress_key()) {
                let mut fresh: Progress = load(&progress_key(), PROGRESS_VERSION).unwrap_or_default();
                fresh.retain_known(g);
                if progress.with_untracked(|p| p != &fresh) {
                    progress.set(fresh);
                    // A story scene this copy no longer has reached: leave it.
                    if let Screen::Location(id, scene) = nav.screen.get_untracked() {
                        if !scene.is_empty() && !progress.with_untracked(|p| p.has_seen(&id, &scene)) {
                            nav.replace(Screen::Home);
                        }
                    }
                }
            }
        });
        let settings = state.settings;
        Effect::new(move |_| {
            let s = settings.get();
            save(SETTINGS_KEY, SETTINGS_VERSION, &s);
            platform::set_document_language(s.lang.code());
        });

        state
    }

    pub fn lang(&self) -> Lang {
        self.settings.with(|s| s.lang)
    }

    pub fn ui(&self) -> &'static Ui {
        i18n::ui(self.lang())
    }

    /// Handles the text of a scanned QR code. A code for one of this game's
    /// locations records the visit and opens its story where the player left off.
    pub fn scan(&self, text: &str) -> ScanOutcome {
        let g = game();
        match payload::decode(text, &g.game.id) {
            Scanned::Ours(code) => match g.location_by_code(code) {
                Some((id, _)) => {
                    let mut outcome = None;
                    self.progress.update(|p| outcome = p.scan(g, id));
                    match outcome {
                        Some(Outcome { next: Some(scene), found }) => {
                            self.arrival.set(Some(Arrival {
                                location: id.clone(),
                                scene: scene.clone(),
                                found,
                            }));
                            ScanOutcome::Location(id.clone(), scene)
                        }
                        _ => ScanOutcome::Unknown,
                    }
                }
                None => ScanOutcome::Unknown,
            },
            Scanned::OtherGame(_) => ScanOutcome::OtherGame,
            Scanned::Foreign => ScanOutcome::Unknown,
        }
    }

    // The three ways on from a scene. The rules (is this step allowed from
    // here, what does it give) live in stopa_core::Progress; these only show
    // the result.

    /// The Continue key.
    pub fn continue_on(&self, location: &str, scene: &str) {
        let mut outcome = None;
        self.progress.update(|p| outcome = p.continue_on(game(), location, scene));
        if let Some(outcome) = outcome {
            self.show(location, scene, outcome);
        }
    }

    pub fn choose(&self, location: &str, scene: &str, choice: &str) {
        let mut outcome = None;
        self.progress.update(|p| outcome = p.choose(game(), location, scene, choice));
        if let Some(outcome) = outcome {
            self.show(location, scene, outcome);
        }
    }

    /// Checks a password; false if the answer is wrong.
    pub fn answer(&self, location: &str, scene: &str, input: &str) -> bool {
        let mut outcome = None;
        self.progress.update(|p| outcome = p.answer(game(), location, scene, input));
        match outcome {
            Some(outcome) => {
                self.show(location, scene, outcome);
                true
            }
            None => false,
        }
    }

    /// Opens the scene a step led to, with what the step gave; or, when the
    /// story ends on this scene, shows what it gave here.
    fn show(&self, location: &str, scene: &str, outcome: Outcome) {
        let Outcome { next, found } = outcome;
        match next {
            Some(next) => {
                self.arrival.set(Some(Arrival { location: location.to_owned(), scene: next.clone(), found }));
                self.nav.go(Screen::Location(location.to_owned(), next));
            }
            None if !found.is_empty() => self.arrival.set(Some(Arrival {
                location: location.to_owned(),
                scene: scene.to_owned(),
                found,
            })),
            None => {}
        }
    }

    pub fn reset_progress(&self) {
        platform::storage_remove(&progress_key());
        self.progress.set(Progress::default());
        self.arrival.set(None);
        self.nav.replace(Screen::Home);
    }
}

pub fn use_app() -> AppState {
    expect_context::<AppState>()
}
