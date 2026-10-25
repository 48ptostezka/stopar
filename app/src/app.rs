use leptos::prelude::*;

use crate::{
    nav::Screen,
    platform,
    screens::{
        home::Home, inventory::Inventory, location::Location, logbook::Logbook, scan::Scan,
        settings::Settings,
    },
    skin,
    state::AppState,
};

#[component]
pub fn App() -> impl IntoView {
    let app = AppState::new();
    provide_context(app);
    let skin_ready = skin::load();
    platform::register_service_worker();

    // Each screen starts at the top.
    Effect::new(move |_| {
        app.nav.screen.track();
        platform::scroll_to_top();
    });

    view! {
        <Show when=move || skin_ready.get()>
            {move || match app.nav.screen.get() {
                Screen::Home => view! { <Home/> }.into_any(),
                Screen::Scan => view! { <Scan/> }.into_any(),
                Screen::Inventory => view! { <Inventory/> }.into_any(),
                Screen::Logbook => view! { <Logbook/> }.into_any(),
                Screen::Settings => view! { <Settings/> }.into_any(),
                // Locations open only by scanning, and only scenes already reached.
                // A typed-in address for anything else shows home.
                Screen::Location(id, scene) => {
                    let scene = if scene.is_empty() {
                        app.progress.with_untracked(|p| p.places.get(&id).map(|pl| pl.scene.clone()).unwrap_or_default())
                    } else {
                        scene
                    };
                    if app.progress.with_untracked(|p| p.has_seen(&id, &scene)) {
                        view! { <Location location=id scene=scene/> }.into_any()
                    } else {
                        view! { <Home/> }.into_any()
                    }
                }
            }}
        </Show>
    }
}
