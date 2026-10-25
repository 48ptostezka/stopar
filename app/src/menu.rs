//! The keys on the home screen, top to bottom.
//! To add, remove or reorder a key, edit `entries()`.

use crate::{i18n::Ui, nav::Screen, ui::icons::IconName};

/// The page colours a skin defines. Each page keeps its colour on its
/// header mark, side rails and highlights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accent {
    A1,
    A2,
    A3,
    /// Spare colour for a fourth menu key.
    #[allow(dead_code)]
    A4,
    /// For pages without a colour of their own, such as Settings.
    Muted,
}

impl Accent {
    pub fn css(self) -> &'static str {
        match self {
            Accent::A1 => "var(--accent-1)",
            Accent::A2 => "var(--accent-2)",
            Accent::A3 => "var(--accent-3)",
            Accent::A4 => "var(--accent-4)",
            Accent::Muted => "var(--muted)",
        }
    }
}

#[derive(Clone)]
pub struct MenuEntry {
    pub screen: Screen,
    pub icon: IconName,
    pub label: fn(&Ui) -> &'static str,
    pub accent: Accent,
}

pub fn entries() -> Vec<MenuEntry> {
    vec![
        MenuEntry { screen: Screen::Scan, icon: IconName::Scan, label: |t| t.scan, accent: Accent::A1 },
        MenuEntry {
            screen: Screen::Inventory,
            icon: IconName::Bag,
            label: |t| t.inventory,
            accent: Accent::A2,
        },
        MenuEntry { screen: Screen::Logbook, icon: IconName::Book, label: |t| t.logbook, accent: Accent::A3 },
    ]
}

/// The colour of a page: its menu entry's colour, or a fixed one for pages
/// that aren't in the menu.
pub fn accent_of(screen: &Screen) -> Accent {
    match screen {
        Screen::Location(..) => Accent::A1,
        Screen::Settings | Screen::Home => Accent::Muted,
        other => {
            entries().into_iter().find(|e| &e.screen == other).map(|e| e.accent).unwrap_or(Accent::Muted)
        }
    }
}
