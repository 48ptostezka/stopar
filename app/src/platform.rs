//! Small wrappers around browser APIs, so the rest of the app stays readable.
//! Every function here fails quietly: a missing feature must never crash the game.

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub fn window() -> web_sys::Window {
    web_sys::window().expect("running in a browser")
}

pub fn document() -> web_sys::Document {
    window().document().expect("page has a document")
}

/// True if `object` has a property called `name`. Used before touching APIs
/// that some browsers hide, e.g. `navigator.serviceWorker` over plain http.
pub fn has_property(object: &JsValue, name: &str) -> bool {
    js_sys::Reflect::has(object, &JsValue::from_str(name)).unwrap_or(false)
}

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

pub fn storage_get(key: &str) -> Option<String> {
    storage()?.get_item(key).ok().flatten()
}

pub fn storage_set(key: &str, value: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(key, value);
    }
}

pub fn storage_remove(key: &str) {
    if let Some(s) = storage() {
        let _ = s.remove_item(key);
    }
}

/// Calls `f` with the key whenever another tab or window of this site
/// changes localStorage (`None` when it was cleared completely). The browser
/// doesn't tell the tab that made the change.
pub fn on_storage_change(f: impl Fn(Option<String>) + 'static) {
    let listener =
        Closure::<dyn Fn(web_sys::StorageEvent)>::new(move |ev: web_sys::StorageEvent| f(ev.key()));
    let _ = window().add_event_listener_with_callback("storage", listener.as_ref().unchecked_ref());
    listener.forget(); // lives as long as the app
}

/// The phone's preferred language, e.g. `cs-CZ`.
pub fn browser_language() -> Option<String> {
    window().navigator().language()
}

pub fn set_document_language(code: &str) {
    if let Some(root) = document().document_element() {
        let _ = root.set_attribute("lang", code);
    }
}

/// A short buzz when a code is recognised. Not every phone supports it.
pub fn vibrate(ms: u32) {
    let navigator = window().navigator();
    if has_property(&navigator, "vibrate") {
        let _ = navigator.vibrate_with_duration(ms);
    }
}

pub fn scroll_to_top() {
    window().scroll_to_with_x_and_y(0.0, 0.0);
}

/// Copies the skin's `--theme-color` into `<meta name="theme-color">`,
/// which colours the phone's status bar.
pub fn apply_theme_color() {
    let Some(root) = document().document_element() else { return };
    let Ok(Some(style)) = window().get_computed_style(&root) else { return };
    let color = style.get_property_value("--theme-color").unwrap_or_default();
    let color = color.trim();
    if color.is_empty() {
        return;
    }
    if let Ok(Some(meta)) = document().query_selector("meta[name=theme-color]") {
        let _ = meta.set_attribute("content", color);
    }
}

/// Registers the offline cache. Only in release builds, so `trunk serve`
/// always shows fresh files while developing.
pub fn register_service_worker() {
    if cfg!(debug_assertions) {
        return;
    }
    let navigator = window().navigator();
    if !has_property(&navigator, "serviceWorker") {
        web_sys::console::warn_1(
            &"Stopa: no service worker on this page, so the game won't work offline. Serve it over https."
                .into(),
        );
        return;
    }
    let _ = navigator.service_worker().register("sw.js");
}

/// Whether the game can start without a network right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offline {
    /// A debug build (`trunk serve`), which never registers the service worker.
    DevBuild,
    /// The page is plain http. Browsers only allow service workers on https
    /// (and on localhost), so nothing is kept for offline use.
    Insecure,
    /// The browser has no service workers.
    Unsupported,
    /// Registered, but not in control of this page yet (first visit).
    Preparing,
    /// A service worker controls the page: everything is cached.
    Ready,
}

pub fn offline_status() -> Offline {
    if cfg!(debug_assertions) {
        return Offline::DevBuild;
    }
    let window = window();
    if !window.is_secure_context() {
        return Offline::Insecure;
    }
    let navigator = window.navigator();
    if !has_property(&navigator, "serviceWorker") {
        return Offline::Unsupported;
    }
    match navigator.service_worker().controller() {
        Some(_) => Offline::Ready,
        None => Offline::Preparing,
    }
}

/// False when the browser knows it has no network. (True doesn't promise
/// that a server is reachable, only that some network is there.)
pub fn is_online() -> bool {
    window().navigator().on_line()
}

/// Deletes every file the service worker kept for offline play and removes
/// the service worker, then reloads the page from the network. A release
/// build registers a fresh service worker on that load and caches the
/// current files again. Call it only while online: without a network the
/// reload has nothing to load.
pub async fn delete_offline_files() {
    let window = window();
    let navigator = window.navigator();
    if has_property(&navigator, "serviceWorker") {
        if let Ok(list) = JsFuture::from(navigator.service_worker().get_registrations()).await {
            for registration in js_sys::Array::from(&list).iter() {
                let registration: web_sys::ServiceWorkerRegistration = registration.unchecked_into();
                if let Ok(promise) = registration.unregister() {
                    let _ = JsFuture::from(promise).await;
                }
            }
        }
    }
    if has_property(&window, "caches") {
        if let Ok(caches) = window.caches() {
            if let Ok(names) = JsFuture::from(caches.keys()).await {
                for name in js_sys::Array::from(&names).iter() {
                    if let Some(name) = name.as_string() {
                        let _ = JsFuture::from(caches.delete(&name)).await;
                    }
                }
            }
        }
    }
    let _ = window.location().reload();
}

pub fn current_hash() -> String {
    window().location().hash().unwrap_or_default()
}

pub fn url_with_hash(hash: &str) -> String {
    let location = window().location();
    let path = location.pathname().unwrap_or_default();
    let search = location.search().unwrap_or_default();
    format!("{path}{search}{hash}")
}
