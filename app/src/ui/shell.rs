//! The page frame: a central panel between two decorative side rails.

use leptos::prelude::*;

use crate::{menu::Accent, skin};

/// Enough markers to fill a tall screen; the rest is clipped.
const MARKERS: usize = 40;

/// Wraps a screen. `accents` colour the rail markers in turn (the home screen
/// passes every menu colour; a page passes its own).
#[component]
pub fn Shell(accents: Vec<Accent>, children: Children) -> impl IntoView {
    let page = accents.first().copied().unwrap_or(Accent::Muted);
    view! {
        <div class="shell" style=format!("--page-accent: {}", page.css())>
            <Rail side="left" accents=accents.clone() offset=0/>
            <main class="panel">{children()}</main>
            <Rail side="right" accents=accents offset=2/>
        </div>
    }
}

#[component]
fn Rail(side: &'static str, accents: Vec<Accent>, offset: usize) -> impl IntoView {
    let href = skin::marker_href();
    let count = accents.len().max(1);
    let markers = (0..MARKERS)
        .map(|i| {
            let accent = accents.get((i + offset) % count).copied().unwrap_or(Accent::Muted);
            let (dx, dy) = scatter(i * 7 + offset * 13);
            let style = format!("color: {}; --dx: {dx}px; --dy: {dy}px", accent.css());
            let inner = format!(r#"<use href="{href}"></use>"#);
            view! { <svg class="rail__marker" style=style inner_html=inner></svg> }
        })
        .collect_view();
    view! {
        <aside class=format!("rail rail--{side}") aria-hidden="true">
            <div class="rail__art">{markers}</div>
        </aside>
    }
}

/// Repeatable offsets that loosen the marker grid on wide rails.
fn scatter(seed: usize) -> (i32, i32) {
    let h = (seed as u32).wrapping_mul(2_654_435_761) >> 8;
    ((h % 25) as i32 - 12, ((h / 25) % 90) as i32)
}
