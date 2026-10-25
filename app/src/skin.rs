//! Loads the game's skin: one stylesheet plus the art it points to.
//! See `app/skins/README.md` for what a skin contains.

use leptos::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};

use crate::{game::game, platform};

fn dir() -> String {
    format!("skins/{}", game().game.skin)
}

/// Link to the skin's rail and page marker, for `<use href=…>`.
pub fn marker_href() -> String {
    format!("{}/markers.svg#marker", dir())
}

/// Adds the skin's stylesheet to the page. The signal turns true once it
/// has loaded (or failed), so the app never shows an unstyled flash.
pub fn load() -> ReadSignal<bool> {
    let (ready, set_ready) = signal(false);
    let doc = platform::document();
    let Ok(element) = doc.create_element("link") else {
        set_ready.set(true);
        return ready;
    };
    let link: web_sys::HtmlLinkElement = element.unchecked_into();
    link.set_rel("stylesheet");
    link.set_href(&format!("{}/skin.css", dir()));

    let on_load = Closure::<dyn FnMut()>::new(move || {
        platform::apply_theme_color();
        set_ready.set(true);
    });
    let on_error = Closure::<dyn FnMut()>::new(move || set_ready.set(true));
    link.set_onload(Some(on_load.as_ref().unchecked_ref()));
    link.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_load.forget();
    on_error.forget();

    match doc.head() {
        Some(head) => {
            let _ = head.append_child(&link);
        }
        None => set_ready.set(true),
    }
    ready
}
