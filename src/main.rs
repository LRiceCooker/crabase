// Suppress noisy warnings for a Leptos CSR app
#![allow(
    clippy::module_inception,
    clippy::type_complexity,
    clippy::empty_line_after_outer_attr
)]

mod app;
mod command_palette;
mod connection;
mod content_area;
mod global_shortcuts;
mod header_bar;
mod header_edit_form;
mod log;
pub mod icons;
mod main_layout;
pub mod overlay;
mod restore_panel;
mod settings;
pub mod shortcuts;
mod sidebar;
mod sql_editor;
mod table_finder;
mod table_view;
mod tabs;
mod tauri;
pub mod theme;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
