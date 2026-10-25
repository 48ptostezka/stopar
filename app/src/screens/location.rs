use leptos::{ev::SubmitEvent, prelude::*};
use stopa_core::{Choice, ChoiceStatus, Found, Password, Pick, Scene};

use crate::{
    game::game,
    menu::accent_of,
    nav::Screen,
    state::use_app,
    ui::{
        icons::{Icon, IconName},
        keys::PageHeader,
        shell::Shell,
    },
};

/// One scene of a location's story: picture, text, what was just found,
/// then the way on (Continue, choices, a password, or Done at the end).
#[component]
pub fn Location(location: String, scene: String) -> impl IntoView {
    let app = use_app();
    let g = game();
    let (Some(loc), Some(sc)) = (g.locations.get(&location), g.scene(&location, &scene)) else {
        return ().into_any();
    };

    let figure = sc.image.as_ref().map(|file| {
        let alt = move || sc.alt.as_ref().map(|a| a.get(app.lang()).to_owned()).unwrap_or_default();
        view! {
            <figure class="scene__figure">
                <img class="scene__image" src=format!("images/{file}") alt=alt/>
            </figure>
        }
    });

    let found = {
        let (location, scene) = (location.clone(), scene.clone());
        move || {
            app.arrival.with(|a| {
                a.as_ref().filter(|a| a.location == location && a.scene == scene).map(|a| a.found.clone())
            })
        }
    };

    // The way on is offered only while the scene's `requires` holds; until
    // then the player gets the `waiting` text and Done. The memo re-draws
    // this part only when that changes, so typing in a password field isn't
    // reset by unrelated progress updates.
    let ready = Memo::new(move |_| app.progress.with(|p| p.is_ready(sc)));
    let way_on = {
        let (location, scene) = (location.clone(), scene.clone());
        move || {
            if ready.get() {
                way_on(location.clone(), scene.clone(), sc)
            } else {
                let waiting = move || {
                    sc.waiting.as_ref().map(|t| t.get(app.lang())).unwrap_or(app.ui().waiting).to_owned()
                };
                view! {
                    <p class="notice notice--waiting" role="status"><Icon name=IconName::Lock/><span>{waiting}</span></p>
                    <DoneKey/>
                }
                .into_any()
            }
        }
    };

    view! {
        <Shell accents=vec![accent_of(&Screen::Location(String::new(), String::new()))]>
            <PageHeader title=move || loc.name.get(app.lang()).to_owned()/>
            {figure}
            <article class="story"><SceneText scene=sc/></article>
            {move || found().filter(|f| !f.is_empty()).map(|f| view! { <FoundBox found=f/> })}
            {way_on}
        </Shell>
    }
    .into_any()
}

/// Continue, the options, the password field, or Done at the end.
fn way_on(location: String, scene: String, sc: &'static Scene) -> AnyView {
    let app = use_app();
    if let Some(password) = &sc.password {
        view! { <PasswordField location scene password/> }.into_any()
    } else if !sc.choices.is_empty() {
        view! { <Choices location scene scene_data=sc/> }.into_any()
    } else if sc.next.is_some() {
        view! { <ContinueKey on_click=move || app.continue_on(&location, &scene)/> }.into_any()
    } else {
        view! { <DoneKey/> }.into_any()
    }
}

/// `text`, then every paragraph whose condition holds right now. A blank
/// line inside a text starts a new paragraph.
#[component]
fn SceneText(scene: &'static Scene) -> impl IntoView {
    let app = use_app();
    move || {
        let lang = app.lang();
        app.progress.with(|p| {
            let mut blocks: Vec<&str> = scene.text.iter().map(|t| t.get(lang)).collect();
            blocks.extend(
                scene.paragraphs.iter().filter(|para| p.meets(&para.when)).map(|para| para.text.get(lang)),
            );
            blocks
                .into_iter()
                .flat_map(|b| b.split("\n\n"))
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(|t| view! { <p>{t.to_owned()}</p> })
                .collect_view()
        })
    }
}

/// What the step that led here gave the player, and what it used up.
#[component]
fn FoundBox(found: Found) -> impl IntoView {
    let app = use_app();
    let g = game();
    let lang = app.lang();
    let item_row = move |(id, qty): &(String, u32)| {
        g.items.get(id).map(|item| {
            view! {
                <li class="found__item">
                    <span class="row__icon"><Icon name=IconName::Item(&item.icon)/></span>
                    <span class="row__name">{item.name.get(lang).to_owned()}</span>
                    <span class="row__qty">{format!("×{qty}")}</span>
                </li>
            }
        })
    };
    let gained = found.items.iter().filter_map(item_row).collect_view();
    let clues = found
        .clues
        .iter()
        .filter_map(|id| g.clues.get(id))
        .map(|clue| {
            view! {
                <li class="found__item">
                    <span class="row__icon"><Icon name=IconName::Clue/></span>
                    <span class="row__name">{clue.title.get(lang).to_owned()}</span>
                </li>
            }
        })
        .collect_view();
    let used = found.used.iter().filter_map(item_row).collect_view();
    let has_gains = !found.items.is_empty() || !found.clues.is_empty();
    let has_used = !found.used.is_empty();
    view! {
        <section class="found" aria-live="polite">
            {has_gains.then(|| view! {
                <h2 class="section-title">{app.ui().you_found}</h2>
                <ul class="found__list">{gained}{clues}</ul>
            })}
            {has_used.then(|| view! {
                <h2 class="section-title found__used">{app.ui().you_used}</h2>
                <ul class="found__list found__list--used">{used}</ul>
            })}
        </section>
    }
}

/// The options of a decision. Chosen options stay pressed; picking one again
/// follows it without new rewards. Unavailable options are hidden, or shown
/// disabled with their `locked` text.
#[component]
fn Choices(location: String, scene: String, scene_data: &'static Scene) -> impl IntoView {
    let app = use_app();
    let options = scene_data
        .choices
        .iter()
        .map(|choice| {
            let status = {
                let (location, scene) = (location.clone(), scene.clone());
                move || app.progress.with(|p| p.choice_status(&location, &scene, scene_data, choice))
            };
            let (location, scene) = (location.clone(), scene.clone());
            move || option_key(status(), choice, location.clone(), scene.clone())
        })
        .collect_view();

    // Done under the options: always in a menu (pick = "any"), so the player
    // can leave whenever they like; in a final decision, once a chosen
    // option with nowhere to go has ended the story on this scene.
    let menu = scene_data.pick == Pick::Any;
    let show_done = {
        let (location, scene) = (location.clone(), scene.clone());
        move || {
            menu || scene_data.choices.iter().any(|c| {
                c.to.is_none()
                    && app.progress.with(|p| p.choice_status(&location, &scene, scene_data, c))
                        == ChoiceStatus::Chosen
            })
        }
    };
    view! {
        <div class="options" role="group">{options}</div>
        <Show when=show_done><DoneKey/></Show>
    }
}

fn option_key(status: ChoiceStatus, choice: &'static Choice, location: String, scene: String) -> AnyView {
    let app = use_app();
    match status {
        ChoiceStatus::Hidden => ().into_any(),
        ChoiceStatus::Locked => view! {
            <button class="key key--option key--locked" disabled=true>
                <Icon name=IconName::Lock/>
                <span>{move || choice.locked.as_ref().map(|t| t.get(app.lang()).to_owned())}</span>
            </button>
        }
        .into_any(),
        _ => {
            let chosen = status == ChoiceStatus::Chosen;
            view! {
                <button
                    class="key key--option"
                    aria-pressed=chosen.to_string()
                    disabled=status == ChoiceStatus::Closed
                    on:click=move |_| app.choose(&location, &scene, &choice.id)
                >
                    {chosen.then(|| view! { <Icon name=IconName::Check/> })}
                    <span>{move || choice.label.get(app.lang()).to_owned()}</span>
                </button>
            }
            .into_any()
        }
    }
}

/// A field for a password. After a correct answer the field is replaced by
/// "Correct!" and the way on.
#[component]
fn PasswordField(location: String, scene: String, password: &'static Password) -> impl IntoView {
    let app = use_app();
    let value = RwSignal::new(String::new());
    let wrong = RwSignal::new(false);
    let solved = {
        let (location, scene) = (location.clone(), scene.clone());
        move || app.progress.with(|p| p.is_solved(&location, &scene))
    };
    let submit = {
        let (location, scene) = (location.clone(), scene.clone());
        move |ev: SubmitEvent| {
            ev.prevent_default();
            let ok = app.answer(&location, &scene, &value.get_untracked());
            wrong.set(!ok);
        }
    };
    let label =
        move || password.prompt.as_ref().map(|t| t.get(app.lang())).unwrap_or(app.ui().password).to_owned();
    let hint = move || password.hint.as_ref().map(|t| t.get(app.lang()).to_owned());
    let wrong_text = move || {
        password.wrong.as_ref().map(|t| t.get(app.lang())).unwrap_or(app.ui().wrong_answer).to_owned()
    };

    let after = {
        let (location, scene) = (location.clone(), scene.clone());
        move || match &password.to {
            Some(_) => {
                let (location, scene) = (location.clone(), scene.clone());
                view! { <ContinueKey on_click=move || app.continue_on(&location, &scene)/> }.into_any()
            }
            None => view! { <DoneKey/> }.into_any(),
        }
    };

    view! {
        <Show
            when=move || !solved()
            fallback=move || view! {
                <p class="solved"><Icon name=IconName::Check/>{move || app.ui().correct}</p>
                {after.clone()}
            }
        >
            <form class="password" on:submit=submit.clone()>
                <p class="setting__label">{label}</p>
                {move || hint().map(|h| view! { <p class="hint">{h}</p> })}
                <div class="password__row">
                    <input
                        class="field"
                        type="text"
                        aria-label=label
                        autocomplete="off"
                        autocapitalize="none"
                        spellcheck="false"
                        enterkeyhint="go"
                        prop:value=move || value.get()
                        on:input=move |ev| {
                            value.set(event_target_value(&ev));
                            wrong.set(false);
                        }
                    />
                    <button type="submit" class="key key--submit">{move || app.ui().check}</button>
                </div>
                <Show when=move || wrong.get()>
                    <p class="notice notice--wrong" role="alert">{wrong_text}</p>
                </Show>
            </form>
        </Show>
    }
}

/// Goes on to the next scene of the story.
#[component]
fn ContinueKey(on_click: impl Fn() + 'static) -> impl IntoView {
    let app = use_app();
    view! {
        <button class="key key--wide" on:click=move |_| on_click()>
            {move || app.ui().next}
            <Icon name=IconName::Next/>
        </button>
    }
}

/// Ends the location's story and returns to the home screen.
#[component]
fn DoneKey() -> impl IntoView {
    let app = use_app();
    view! { <button class="key key--wide" on:click=move |_| app.nav.home()>{move || app.ui().done}</button> }
}
