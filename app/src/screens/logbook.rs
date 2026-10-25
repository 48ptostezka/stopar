use leptos::prelude::*;

use crate::{
    game::game,
    i18n,
    menu::accent_of,
    nav::Screen,
    state::use_app,
    ui::{
        icons::{Icon, IconName},
        keys::PageHeader,
        shell::Shell,
    },
};

/// Found clues as cards, newest first. The order means nothing to the story;
/// players combine clues however they like. A tapped card spreads across
/// both columns and shows its whole text.
#[component]
pub fn Logbook() -> impl IntoView {
    let app = use_app();
    let g = game();
    let open = RwSignal::new(None::<String>);
    // Only the list of found clues decides which cards exist, so marking a
    // clue as read doesn't rebuild the grid (and keeps keyboard focus).
    let found = Memo::new(move |_| app.progress.with(|p| p.clues.clone()));

    let cards = move || {
        let lang = app.lang();
        found
            .get()
            .into_iter()
            .rev()
            .filter_map(|id| g.clues.get(&id).map(|clue| (id, clue)))
            .map(|(id, clue)| {
                let is_open = {
                    let id = id.clone();
                    move || open.with(|o| o.as_deref() == Some(id.as_str()))
                };
                let unread = {
                    let id = id.clone();
                    move || app.progress.with(|p| p.is_unread(&id))
                };
                let toggle = {
                    let id = id.clone();
                    move |_| {
                        open.update(|o| {
                            *o = if o.as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }
                        });
                        if app.progress.with_untracked(|p| p.is_unread(&id)) {
                            app.progress.update(|p| p.mark_read(&id));
                        }
                    }
                };
                let places = app.progress.with_untracked(|p| {
                    p.where_clue_found(g, &id).iter().map(|l| l.name.get(lang)).collect::<Vec<_>>().join(", ")
                });
                let class = {
                    let is_open = is_open.clone();
                    move || if is_open() { "clue clue--open" } else { "clue" }
                };
                let expanded = {
                    let is_open = is_open.clone();
                    move || is_open().to_string()
                };
                view! {
                    <button class=class aria-expanded=expanded on:click=toggle>
                        <Show when=unread>
                            <span class="tag">{move || app.ui().clue_new}</span>
                        </Show>
                        <span class="clue__title">{clue.title.get(lang).to_owned()}</span>
                        <span class="clue__text">{clue.text.get(lang).to_owned()}</span>
                        <Show when=is_open.clone()>
                            <span class="clue__meta">{i18n::found_at(lang, &places)}</span>
                            <span class="clue__close"><Icon name=IconName::Chevron/></span>
                        </Show>
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <Shell accents=vec![accent_of(&Screen::Logbook)]>
            <PageHeader title=move || app.ui().logbook.to_owned()/>
            <Show
                when=move || !found.with(|f| f.is_empty())
                fallback=move || view! { <p class="empty">{move || app.ui().logbook_empty}</p> }
            >
                <div class="clues">{cards}</div>
            </Show>
        </Shell>
    }
}
