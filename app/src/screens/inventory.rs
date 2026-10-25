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

#[component]
pub fn Inventory() -> impl IntoView {
    let app = use_app();
    let g = game();
    // Which row is open. Only one at a time, like an accordion.
    let open = RwSignal::new(None::<String>);

    let rows = move || {
        let lang = app.lang();
        app.progress.with(|p| {
            g.items
                .iter()
                .filter_map(|(id, item)| p.items.get(id).filter(|q| **q > 0).map(|q| (id.clone(), item, *q)))
                .map(|(id, item, qty)| {
                    let places = p
                        .where_item_found(g, &id)
                        .iter()
                        .map(|l| l.name.get(lang))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let is_open = {
                        let id = id.clone();
                        move || open.with(|o| o.as_deref() == Some(id.as_str()))
                    };
                    let toggle = {
                        let id = id.clone();
                        move |_| {
                            open.update(|o| {
                                *o = if o.as_deref() == Some(id.as_str()) { None } else { Some(id.clone()) }
                            })
                        }
                    };
                    let row_class = {
                        let is_open = is_open.clone();
                        move || if is_open() { "row row--open" } else { "row" }
                    };
                    let expanded = {
                        let is_open = is_open.clone();
                        move || is_open().to_string()
                    };
                    let description = item.description.get(lang).to_owned();
                    let found = (!places.is_empty()).then(|| i18n::found_at(lang, &places));
                    view! {
                        <li class=row_class>
                            <button class="row__head" aria-expanded=expanded on:click=toggle>
                                <span class="row__icon"><Icon name=IconName::Item(&item.icon)/></span>
                                <span class="row__name">{item.name.get(lang).to_owned()}</span>
                                <span class="row__qty">{format!("×{qty}")}</span>
                                <span class="row__chevron"><Icon name=IconName::Chevron/></span>
                            </button>
                            <Show when=is_open.clone()>
                                <div class="row__detail">
                                    <p>{description.clone()}</p>
                                    {found.clone().map(|f| view! { <p class="meta">{f}</p> })}
                                </div>
                            </Show>
                        </li>
                    }
                })
                .collect_view()
        })
    };
    let is_empty = move || app.progress.with(|p| p.items.values().all(|q| *q == 0));

    view! {
        <Shell accents=vec![accent_of(&Screen::Inventory)]>
            <PageHeader title=move || app.ui().inventory.to_owned()/>
            <Show
                when=move || !is_empty()
                fallback=move || view! { <p class="empty">{move || app.ui().inventory_empty}</p> }
            >
                <div class="table">
                    <div class="table__head">
                        <span>{move || app.ui().column_item}</span>
                        <span>{move || app.ui().column_quantity}</span>
                    </div>
                    <ul class="rows">{rows}</ul>
                </div>
            </Show>
        </Shell>
    }
}
