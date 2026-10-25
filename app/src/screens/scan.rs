use std::time::Duration;

use leptos::{html, prelude::*};

use crate::{
    menu::accent_of,
    nav::Screen,
    platform,
    scanner::{CameraStatus, Scanner},
    state::{use_app, ScanOutcome},
    ui::{keys::PageHeader, shell::Shell},
};

/// A short message shown under the camera after scanning a code that
/// doesn't lead anywhere.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Notice {
    Unknown,
    OtherGame,
}

#[component]
pub fn Scan() -> impl IntoView {
    let app = use_app();
    let status = RwSignal::new(CameraStatus::Starting);
    let notice = RwSignal::new(None::<Notice>);
    let attempt = RwSignal::new(0u32);
    let video = NodeRef::<html::Video>::new();
    // The scanner holds browser objects, which must stay on this thread.
    let scanner = StoredValue::new_local(None::<Scanner>);

    let show_notice = move |n: Notice| {
        notice.set(Some(n));
        set_timeout(move || notice.set(None), Duration::from_millis(3000));
    };

    Effect::new(move |_| {
        attempt.track();
        let Some(element) = video.get() else { return };
        status.set(CameraStatus::Starting);
        let on_code = move |text: &str| match app.scan(text) {
            ScanOutcome::Location(id, scene) => {
                platform::vibrate(60);
                app.nav.replace(Screen::Location(id, scene));
                true
            }
            ScanOutcome::OtherGame => {
                show_notice(Notice::OtherGame);
                false
            }
            ScanOutcome::Unknown => {
                show_notice(Notice::Unknown);
                false
            }
        };
        let started = Scanner::start(element, move |s| status.set(s), on_code);
        // Replacing the old scanner drops it, which turns its camera off.
        scanner.update_value(|slot| *slot = Some(started));
    });
    on_cleanup(move || scanner.update_value(|slot| *slot = None));

    let problem = move || -> Option<(&'static str, &'static str, bool)> {
        let ui = app.ui();
        match status.get() {
            CameraStatus::Starting | CameraStatus::Scanning => None,
            CameraStatus::Denied => Some((ui.camera_denied.0, ui.camera_denied.1, true)),
            CameraStatus::Missing => Some((ui.camera_missing.0, ui.camera_missing.1, false)),
            CameraStatus::Insecure => Some((ui.camera_insecure.0, ui.camera_insecure.1, false)),
            CameraStatus::Failed => Some((ui.camera_failed.0, ui.camera_failed.1, true)),
        }
    };

    view! {
        <Shell accents=vec![accent_of(&Screen::Scan)]>
            <PageHeader title=move || app.ui().scan.to_owned()/>
            <div class=move || if status.get() == CameraStatus::Scanning { "viewport" } else { "viewport viewport--idle" }>
                <video node_ref=video class="viewport__video" autoplay=true playsinline=true prop:muted=true></video>
                <div class="viewport__finder" aria-hidden="true">
                    <span class="viewport__line"></span>
                </div>
            </div>
            <div class="scan-status" role="status" aria-live="polite">
                {move || match problem() {
                    Some((title, body, retry)) => view! {
                        <div class="problem">
                            <h2 class="problem__title">{title}</h2>
                            <p class="hint">{body}</p>
                            {retry.then(|| view! {
                                <button class="key key--wide" on:click=move |_| attempt.update(|n| *n += 1)>
                                    {move || app.ui().try_again}
                                </button>
                            })}
                        </div>
                    }
                    .into_any(),
                    None if status.get() == CameraStatus::Starting => view! {
                        <p class="status-line">{move || app.ui().camera_starting}</p>
                    }
                    .into_any(),
                    None => view! {
                        <p class="status-line"><span class="status-dot"></span>{move || app.ui().looking}</p>
                        {move || match notice.get() {
                            Some(n) => {
                                let ui = app.ui();
                                let text = if n == Notice::OtherGame { ui.code_other_game } else { ui.code_unknown };
                                view! { <p class="notice">{text}</p> }.into_any()
                            }
                            None => view! { <p class="hint">{move || app.ui().scan_hint}</p> }.into_any(),
                        }}
                    }
                    .into_any(),
                }}
            </div>
        </Shell>
    }
}
