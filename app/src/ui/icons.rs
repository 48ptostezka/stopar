//! The icon set, shared by all skins. Drawn on a 32 × 32 grid centred on 0,0
//! with strokes only; the colour comes from CSS (`currentColor`).

use leptos::prelude::*;
use stopa_core::{Icon as Builtin, ItemIcon};

use crate::game::icon_markup;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconName {
    Scan,
    Bag,
    Book,
    Sliders,
    Back,
    Chevron,
    Clue,
    Lock,
    Check,
    Next,
    /// An item's picture: built in, or one of the game's own SVG files.
    Item(&'static ItemIcon),
}

fn markup(name: IconName) -> &'static str {
    match name {
        IconName::Scan => "<path d='M-15 -8V-15H-8M8 -15H15V-8M15 8V15H8M-8 15H-15V8M-9 0H9'/>",
        IconName::Bag => "<rect x='-12' y='-8' width='24' height='23' rx='6'/><path d='M-5 -8V-11a5 5 0 0 1 10 0V-8M-12 1H12M-3 1V5H3V1'/>",
        IconName::Book => "<path d='M-11 -14H9a2 2 0 0 1 2 2V14H-9a2 2 0 0 1-2-2ZM-6 -14V14M-1 5H6'/><path d='M2 -14V-3L4.5 -5.5L7 -3V-14' style='fill:currentColor'/>",
        IconName::Sliders => "<path d='M-11 -8H11M-11 0H11M-11 8H11'/><circle cx='4' cy='-8' r='3.2' style='fill:var(--surface)'/><circle cx='-4' cy='0' r='3.2' style='fill:var(--surface)'/><circle cx='2' cy='8' r='3.2' style='fill:var(--surface)'/>",
        IconName::Back => "<path d='M4 -9L-5 0L4 9'/>",
        IconName::Chevron => "<path d='M-6 -3L0 3L6 -3'/>",
        IconName::Clue => "<circle cx='-3' cy='-3' r='8'/><path d='M3 3L11 11'/>",
        IconName::Lock => "<rect x='-9' y='-3' width='18' height='14' rx='3'/><path d='M-5 -3V-7a5 5 0 0 1 10 0V-3M0 3V6'/>",
        IconName::Check => "<path d='M-10 1L-3 8L11 -7'/>",
        IconName::Next => "<path d='M-9 0H9M3 -7L10 0L3 7'/>",
        IconName::Item(ItemIcon::File(_)) => "",
        IconName::Item(ItemIcon::Builtin(icon)) => match icon {
            Builtin::Generic => "<path d='M-11 -6L0 -12L11 -6V7L0 13L-11 7ZM-11 -6L0 0L11 -6M0 0V13'/>",
            Builtin::Map => "<path d='M-11 -8L-4 -10L4 -8L11 -10V8L4 10L-4 8L-11 10ZM-4 -10V8M4 -8V10'/>",
            Builtin::Key => "<circle cx='-6' cy='0' r='5'/><path d='M-1 0H12M8 0V5M11 0V4'/>",
            Builtin::Coin => "<circle r='11'/><circle r='6'/>",
            Builtin::Lantern => "<path d='M-3.5 -12a3.5 3.5 0 0 1 7 0M-9 11H9M0 -3V3'/><rect x='-7' y='-9' width='14' height='18' rx='2'/>",
            Builtin::Feather => "<path d='M-10 11L8 -7M8 -11C-3 -11 -8 -2 -7 7C2 8 10 2 10 -9Z'/>",
            Builtin::Cell => "<rect x='-7' y='-9' width='14' height='21' rx='2'/><path d='M-3 -12H3M1 -4L-2.5 2H2.5L-1 8'/>",
            Builtin::Crystal => "<path d='M0 -12L9 -3L0 12L-9 -3ZM-9 -3H9M0 -12L-3.5 -3L0 12L3.5 -3Z'/>",
            Builtin::Decoder => "<rect x='-9' y='0' width='18' height='11' rx='2'/><path d='M0 0V-6M-5.5 -9a8 8 0 0 1 11 0M-9.5 -13a13 13 0 0 1 19 0M-4 5.5H1'/>",
            Builtin::Tank => "<rect x='-6' y='-8' width='12' height='20' rx='5'/><path d='M-3 -11H3M0 -11V-8'/>",
            Builtin::Letter => "<rect x='-12' y='-8' width='24' height='16' rx='2'/><path d='M-12 -7L0 2L12 -7'/>",
            Builtin::Flask => "<path d='M-4 -12H4M-3 -12V-3L-10 9a2 2 0 0 0 1.7 3H8.3a2 2 0 0 0 1.7 -3L3 -3V-12M-6 4H6'/>",
        },
    }
}

#[component]
pub fn Icon(name: IconName) -> impl IntoView {
    match name {
        // A game's own icon is a whole <svg> file, so it sits in a wrapper
        // that takes the size and colour the built-in icons would get.
        IconName::Item(ItemIcon::File(file)) => view! {
            <span class="icon icon--file" aria-hidden="true" inner_html=icon_markup(file).unwrap_or_default()></span>
        }
        .into_any(),
        _ => view! {
            <svg class="icon" viewBox="-16 -16 32 32" aria-hidden="true" focusable="false" inner_html=markup(name)></svg>
        }
        .into_any(),
    }
}
