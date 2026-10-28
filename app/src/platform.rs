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
    // updateViaCache "none": when the browser checks for a new version, it
    // fetches sw.js and the file list it imports from the server, never from
    // its ordinary web cache (GitHub Pages lets that keep files 10 minutes),
    // so a new upload is noticed on the next visit.
    let options = web_sys::RegistrationOptions::new();
    options.set_update_via_cache(web_sys::ServiceWorkerUpdateViaCache::None);
    let _ = navigator.service_worker().register_with_options("sw.js", &options);
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
    /// Registered, but nothing downloaded and nothing reported yet.
    Preparing,
    /// Downloading the offline copy: `done` of `total` files saved. With
    /// `update`, an older version is already ready offline meanwhile.
    Downloading { done: u32, total: u32, update: bool },
    /// The download stopped (the connection dropped). It goes on from where
    /// it stopped the next time the game is opened online.
    Interrupted { done: u32, total: u32 },
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

/// Calls `f` whenever the offline state changes: the service worker reports
/// download progress (see public/sw.js, `report`), or a new version takes
/// over the page.
pub fn watch_offline(f: impl Fn(Offline) + Clone + 'static) {
    if offline_status() == Offline::DevBuild || !has_property(&window().navigator(), "serviceWorker") {
        return;
    }
    let container = window().navigator().service_worker();
    let has_controller = {
        let container = container.clone();
        move || container.controller().is_some()
    };

    let on_message = {
        let (f, has_controller) = (f.clone(), has_controller.clone());
        Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
            let data = ev.data();
            let field = |name: &str| js_sys::Reflect::get(&data, &JsValue::from_str(name)).ok();
            if field("type").and_then(|t| t.as_string()).as_deref() != Some("stopa-offline") {
                return;
            }
            let number = |name: &str| field(name).and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
            let (done, total) = (number("done"), number("total"));
            let update = has_controller();
            match field("state").and_then(|s| s.as_string()).as_deref() {
                Some("progress") => f(Offline::Downloading { done, total, update }),
                // An interrupted update leaves the older version working offline.
                Some("interrupted") if update => f(Offline::Ready),
                Some("interrupted") => f(Offline::Interrupted { done, total }),
                _ => {}
            }
        })
    };
    let _ = container.add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref());
    on_message.forget(); // lives as long as the app

    let on_controller = Closure::<dyn Fn()>::new(move || {
        if has_controller() {
            f(Offline::Ready)
        }
    });
    let _ = container
        .add_event_listener_with_callback("controllerchange", on_controller.as_ref().unchecked_ref());
    on_controller.forget();
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
