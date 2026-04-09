mod app;
mod command_palette;
mod connection;
pub mod icons;
mod main_layout;
mod sidebar;
mod table_view;
mod tabs;
mod tauri;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
