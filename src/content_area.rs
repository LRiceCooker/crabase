use leptos::prelude::*;

use crate::overlay::{self, ActiveOverlay};
use crate::restore_panel::RestorePanel;
use crate::settings::settings_view::SettingsView;
use crate::sql_editor::sql_tab::SqlTab;
use crate::table_view::table_view::TableView;
use crate::tabs::tab_bar::TabState;

/// Main content area that switches between overlays (restore, settings)
/// and the active tab content (table view, SQL editor, or empty state).
#[component]
pub fn ContentArea(
    /// The currently active table name (if any).
    active_table: Memo<Option<String>>,
    /// Whether the active tab is an SQL editor.
    is_sql_tab: Memo<bool>,
    /// The active tab ID.
    active_tab_id: Memo<usize>,
    /// Tab state for accessing tab metadata and dirty state.
    tab_state: TabState,
    /// Called when a saved query is saved (to refresh the sidebar list).
    on_query_saved: Callback<()>,
) -> impl IntoView {
    let overlay_ctx = overlay::use_overlay();

    view! {
        <main class="flex-1 overflow-y-auto">
            {move || {
                if overlay_ctx.is_open(ActiveOverlay::Restore) {
                    view! {
                        <RestorePanel on_close=Callback::new(move |_: ()| {
                            overlay_ctx.close();
                        }) />
                    }.into_any()
                } else if overlay_ctx.is_open(ActiveOverlay::Settings) {
                    view! { <SettingsView /> }.into_any()
                } else if active_table.get().is_some() {
                    view! {
                        <div class="h-full">
                            <TableView table_name=active_table />
                        </div>
                    }.into_any()
                } else if is_sql_tab.get() {
                    let ts = tab_state.clone();
                    let tab_id = active_tab_id.get();
                    let on_dirty = Callback::new(move |dirty: bool| {
                        ts.set_dirty(tab_id, dirty);
                    });
                    // Use get_untracked to avoid re-creating the SqlTab on tab rename
                    let query_name = {
                        let tabs = tab_state.tabs.get_untracked();
                        tabs.iter()
                            .find(|t| t.id == tab_id)
                            .map(|t| t.title.clone())
                            .unwrap_or_default()
                    };
                    view! {
                        <div class="h-full">
                            <SqlTab
                                query_name=query_name
                                on_dirty_change=on_dirty
                                on_query_saved=on_query_saved
                            />
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center justify-center h-full text-gray-400 dark:text-zinc-500">
                            <p class="text-[13px]">"Select a table to get started"</p>
                        </div>
                    }.into_any()
                }
            }}
        </main>
    }
}
