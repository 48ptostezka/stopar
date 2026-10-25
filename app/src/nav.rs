//! Which screen is showing, kept in step with the browser history so the
//! phone's back gesture moves between screens (and between the scenes of a
//! location's story) instead of leaving the app.

use leptos::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::platform;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    Home,
    Scan,
    Inventory,
    Logbook,
    Settings,
    /// A location and one of its scenes. An empty scene means "where the
    /// player left off", used when an address has no scene in it.
    Location(String, String),
}

impl Screen {
    fn to_hash(&self) -> String {
        match self {
            Screen::Home => String::new(),
            Screen::Scan => "#scan".into(),
            Screen::Inventory => "#inventory".into(),
            Screen::Logbook => "#logbook".into(),
            Screen::Settings => "#settings".into(),
            Screen::Location(id, scene) if scene.is_empty() => format!("#place/{id}"),
            Screen::Location(id, scene) => format!("#place/{id}/{scene}"),
        }
    }

    fn from_hash(hash: &str) -> Screen {
        match hash.trim_start_matches('#') {
            "scan" => Screen::Scan,
            "inventory" => Screen::Inventory,
            "logbook" => Screen::Logbook,
            "settings" => Screen::Settings,
            other => {
                match other.strip_prefix("place/").map(|rest| rest.split_once('/').unwrap_or((rest, ""))) {
                    Some((id, scene)) if !id.is_empty() => Screen::Location(id.to_owned(), scene.to_owned()),
                    _ => Screen::Home,
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Nav {
    pub screen: RwSignal<Screen>,
}

/// How many entries this app has pushed on top of the home screen. Stored
/// in each history entry, so Done can jump straight home and Back knows
/// whether there is anything to go back to.
fn depth() -> i32 {
    platform::window()
        .history()
        .ok()
        .and_then(|h| h.state().ok())
        .and_then(|s| s.as_f64())
        .map(|d| d as i32)
        .unwrap_or(0)
}

impl Nav {
    pub fn new() -> Nav {
        let screen = RwSignal::new(Screen::from_hash(&platform::current_hash()));
        let on_pop =
            Closure::<dyn FnMut()>::new(move || screen.set(Screen::from_hash(&platform::current_hash())));
        let _ =
            platform::window().add_event_listener_with_callback("popstate", on_pop.as_ref().unchecked_ref());
        on_pop.forget(); // lives as long as the app
        Nav { screen }
    }

    /// Opens a screen; the back gesture returns to the current one.
    pub fn go(&self, to: Screen) {
        if self.screen.get_untracked() == to {
            return;
        }
        if let Ok(history) = platform::window().history() {
            let state = JsValue::from_f64((depth() + 1) as f64);
            let _ = history.push_state_with_url(&state, "", Some(&platform::url_with_hash(&to.to_hash())));
        }
        self.screen.set(to);
    }

    /// Swaps the current screen for another, e.g. Scan for the location it found,
    /// so Back from the location goes home rather than to the camera.
    pub fn replace(&self, to: Screen) {
        if let Ok(history) = platform::window().history() {
            let state = JsValue::from_f64(depth() as f64);
            let _ = history.replace_state_with_url(&state, "", Some(&platform::url_with_hash(&to.to_hash())));
        }
        self.screen.set(to);
    }

    /// The on-screen Back key: one step back in history, or home after a
    /// reload, when there is nothing to go back to.
    pub fn back(&self) {
        match platform::window().history() {
            Ok(history) if depth() > 0 => {
                let _ = history.back();
            }
            _ => self.replace(Screen::Home),
        }
    }

    /// Straight home, skipping every screen and scene opened since.
    pub fn home(&self) {
        let steps = depth();
        match platform::window().history() {
            Ok(history) if steps > 0 => {
                let _ = history.go_with_delta(-steps);
            }
            _ => self.replace(Screen::Home),
        }
    }
}
