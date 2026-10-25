//! Keys (buttons) and page headers.

use leptos::prelude::*;

use crate::{
    menu::MenuEntry,
    skin,
    state::use_app,
    ui::icons::{Icon, IconName},
};

/// A large home-screen key: badge with icon, then the page name.
#[component]
pub fn NavKey(entry: MenuEntry) -> impl IntoView {
    let app = use_app();
    let MenuEntry { screen, icon, label, accent } = entry;
    view! {
        <button
            class="key key--nav"
            style=format!("--key-accent: {}", accent.css())
            on:click=move |_| app.nav.go(screen.clone())
        >
            <span class="badge"><Icon name=icon/></span>
            <span class="key__label">{move || label(app.ui())}</span>
        </button>
    }
}

/// A small square key with only an icon, such as Back or Settings.
#[component]
pub fn IconKey(
    icon: IconName,
    #[prop(into)] label: Signal<&'static str>,
    #[prop(into)] on_press: Callback<()>,
) -> impl IntoView {
    view! {
        <button class="key key--icon" aria-label=move || label.get() on:click=move |_| on_press.run(())>
            <Icon name=icon/>
        </button>
    }
}

/// Back key, page title and the page's marker in its colour.
#[component]
pub fn PageHeader(#[prop(into)] title: Signal<String>, #[prop(default = true)] mark: bool) -> impl IntoView {
    let app = use_app();
    view! {
        <header class="page-header">
            <IconKey icon=IconName::Back label=Signal::derive(move || app.ui().back) on_press=move |_| app.nav.back()/>
            <h1 class="page-title">{move || title.get()}</h1>
            {mark.then(|| view! { <PageMark/> })}
        </header>
    }
}

#[component]
fn PageMark() -> impl IntoView {
    let inner = format!(r#"<use href="{}"></use>"#, skin::marker_href());
    view! { <svg class="page-mark" aria-hidden="true" inner_html=inner></svg> }
}
