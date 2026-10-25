//! Stopa: a phone web app for games played on foot.

mod app;
mod game;
mod i18n;
mod menu;
mod nav;
mod platform;
mod scanner;
mod screens;
mod skin;
mod state;
mod ui;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
