use leptos::prelude::*;

use crate::{
    game::game,
    menu,
    nav::Screen,
    state::use_app,
    ui::{
        icons::IconName,
        keys::{IconKey, NavKey},
        shell::Shell,
    },
};

#[component]
pub fn Home() -> impl IntoView {
    let app = use_app();
    let entries = menu::entries();
    let accents = entries.iter().map(|e| e.accent).collect();
    view! {
        <Shell accents=accents>
            <div class="top-bar">
                <IconKey
                    icon=IconName::Sliders
                    label=Signal::derive(move || app.ui().settings)
                    on_press=move |_| app.nav.go(Screen::Settings)
                />
            </div>
            <section class="title-block">
                <img class="logo" src="images/logo.svg" alt=""/>
                <h1 class="game-title">{move || game().game.title.get(app.lang()).to_owned()}</h1>
            </section>
            <nav class="menu" aria-label=move || app.ui().menu>
                {entries.into_iter().map(|entry| view! { <NavKey entry=entry/> }).collect_view()}
            </nav>
        </Shell>
    }
}
