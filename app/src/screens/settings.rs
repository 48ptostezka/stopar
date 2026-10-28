use leptos::prelude::*;
use stopa_core::Lang;

use crate::{
    i18n,
    menu::accent_of,
    nav::Screen,
    platform::{self, Offline},
    state::use_app,
    ui::{keys::PageHeader, shell::Shell},
};

/// Player settings. Add new options as more `<section class="setting">` blocks.
#[component]
pub fn Settings() -> impl IntoView {
    let app = use_app();
    view! {
        <Shell accents=vec![accent_of(&Screen::Settings)]>
            <PageHeader title=move || app.ui().settings.to_owned() mark=false/>
            <section class="setting">
                <h2 class="setting__label">{move || app.ui().language}</h2>
                <div class="segmented">
                    {Lang::ALL
                        .into_iter()
                        .map(|lang| view! {
                            <button
                                class="key key--choice"
                                lang=lang.code()
                                aria-pressed=move || (app.lang() == lang).to_string()
                                on:click=move |_| app.settings.update(|s| s.lang = lang)
                            >
                                {lang.native_name()}
                            </button>
                        })
                        .collect_view()}
                </div>
            </section>
            <OfflineStatus/>
            <DeveloperTools/>
        </Shell>
    }
}

/// Says whether the game will start without a network, and if not, why.
/// Updates live while the offline copy downloads.
#[component]
fn OfflineStatus() -> impl IntoView {
    let app = use_app();
    let text = move || {
        let (ui, lang) = (app.ui(), app.lang());
        match app.offline.get() {
            Offline::Ready => ui.offline_ready.to_owned(),
            Offline::Preparing => ui.offline_preparing.to_owned(),
            Offline::Downloading { done, total, update } => {
                i18n::offline_downloading(lang, done, total, update)
            }
            Offline::Interrupted { done, total } => i18n::offline_interrupted(lang, done, total),
            Offline::Insecure => ui.offline_insecure.to_owned(),
            Offline::Unsupported => ui.offline_unsupported.to_owned(),
            Offline::DevBuild => ui.offline_dev.to_owned(),
        }
    };
    let class = move || match app.offline.get() {
        Offline::Ready | Offline::Downloading { update: true, .. } => "status-line",
        _ => "notice",
    };
    view! {
        <section class="setting">
            <h2 class="setting__label">{move || app.ui().offline}</h2>
            <p class=class role="status">{text}</p>
        </section>
    }
}

/// Tools for testing, in every build. Each acts on the second tap, so a
/// stray tap can't delete anything.
#[component]
fn DeveloperTools() -> impl IntoView {
    let app = use_app();
    view! {
        <section class="setting setting--developer">
            <h2 class="setting__label">{move || app.ui().developer}</h2>
            <TwoTapKey
                label=move || app.ui().reset_progress
                confirm=move || app.ui().reset_confirm
                note=move || app.ui().reset_note
                on_confirm=move || {
                    app.reset_progress();
                    None
                }
            />
            <TwoTapKey
                label=move || app.ui().delete_offline
                confirm=move || app.ui().delete_offline_confirm
                note=move || app.ui().delete_offline_note
                on_confirm=move || {
                    if !platform::is_online() {
                        return Some(app.ui().needs_network);
                    }
                    leptos::task::spawn_local(platform::delete_offline_files());
                    Some(app.ui().delete_offline_working)
                }
            />
        </section>
    }
}

/// A key that asks for a second tap before acting. `on_confirm` may return
/// a message to show under the key instead of the note.
#[component]
fn TwoTapKey(
    label: impl Fn() -> &'static str + Send + Sync + 'static,
    confirm: impl Fn() -> &'static str + Send + Sync + 'static,
    note: impl Fn() -> &'static str + Send + Sync + 'static,
    on_confirm: impl Fn() -> Option<&'static str> + 'static,
) -> impl IntoView {
    let armed = RwSignal::new(false);
    let message = RwSignal::new(None::<&'static str>);
    view! {
        <div class="tool">
            <button
                class="key key--wide"
                aria-pressed=move || armed.get().to_string()
                on:click=move |_| {
                    if armed.get_untracked() {
                        armed.set(false);
                        // try_set: the action may have left this screen already.
                        let _ = message.try_set(on_confirm());
                    } else {
                        armed.set(true);
                        message.set(None);
                    }
                }
            >
                {move || if armed.get() { confirm() } else { label() }}
            </button>
            <p class="hint" role="status">{move || message.get().unwrap_or_else(&note)}</p>
        </div>
    }
}
