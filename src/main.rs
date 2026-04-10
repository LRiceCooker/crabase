mod app;
mod command_palette;
mod connection;
pub mod icons;
mod main_layout;
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
