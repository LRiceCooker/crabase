use leptos::prelude::*;
use std::collections::HashSet;

use crate::icons::{IconTable, IconTerminal, IconX};
use crate::tabs::tab_title::TabTitle;

// ── Tab types ───────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum TabKind {
    TableView(String), // table name
    SqlEditor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub id: usize,
    pub kind: TabKind,
    pub title: String,
}

// ── Tab state ───────────────────────────────────────────

/// Shared tab state. Clone this and pass it to components that need tab access.
#[derive(Clone)]
pub struct TabState {
    next_id: RwSignal<usize>,
    next_untitled: RwSignal<usize>,
    pub tabs: RwSignal<Vec<Tab>>,
    pub active_id: RwSignal<Option<usize>>,
    dirty_tabs: RwSignal<HashSet<usize>>,
}

impl TabState {
    pub fn new() -> Self {
        Self {
            next_id: RwSignal::new(1),
            next_untitled: RwSignal::new(1),
            tabs: RwSignal::new(Vec::new()),
            active_id: RwSignal::new(None),
            dirty_tabs: RwSignal::new(HashSet::new()),
        }
    }

    /// Open a new tab (or focus an existing one for the same table).
    /// Returns the tab id.
    pub fn open(&self, kind: TabKind) -> usize {
        // For table views, reuse existing tab for the same table
        if let TabKind::TableView(ref table_name) = kind {
            let existing = self.tabs.get().iter().find(|t| {
                matches!(&t.kind, TabKind::TableView(name) if name == table_name)
            }).map(|t| t.id);

            if let Some(id) = existing {
                self.active_id.set(Some(id));
                return id;
            }
        }

        let id = self.next_id.get();
        self.next_id.set(id + 1);

        let title = match &kind {
            TabKind::TableView(name) => name.clone(),
            TabKind::SqlEditor => {
                let n = self.next_untitled.get();
                self.next_untitled.set(n + 1);
                format!("Untitled-{n}")
            }
        };

        let tab = Tab { id, kind, title };
        self.tabs.update(|tabs| tabs.push(tab));
        self.active_id.set(Some(id));
        id
    }

    /// Switch to a tab by id.
    pub fn switch(&self, id: usize) {
        if self.tabs.get().iter().any(|t| t.id == id) {
            self.active_id.set(Some(id));
        }
    }

    /// Mark a tab as dirty or clean.
    pub fn set_dirty(&self, id: usize, dirty: bool) {
        self.dirty_tabs.update(|set| {
            if dirty {
                set.insert(id);
            } else {
                set.remove(&id);
            }
        });
    }

    /// Check if a tab is dirty.
    #[allow(dead_code)]
    pub fn is_dirty(&self, id: usize) -> bool {
        self.dirty_tabs.with(|s| s.contains(&id))
    }

    /// Close a tab by id. If the closed tab was active, activate an adjacent tab.
    pub fn close(&self, id: usize) {
        let tabs = self.tabs.get();
        let idx = tabs.iter().position(|t| t.id == id);

        if let Some(idx) = idx {
            // If closing the active tab, pick a neighbor
            if self.active_id.get() == Some(id) {
                let new_active = if tabs.len() <= 1 {
                    None
                } else if idx + 1 < tabs.len() {
                    Some(tabs[idx + 1].id)
                } else {
                    Some(tabs[idx - 1].id)
                };
                self.active_id.set(new_active);
            }

            self.tabs.update(|tabs| {
                tabs.retain(|t| t.id != id);
            });
            self.dirty_tabs.update(|set| {
                set.remove(&id);
            });
        }
    }

    /// Rename a tab's title.
    pub fn rename_tab(&self, id: usize, new_title: String) {
        self.tabs.update(|tabs| {
            if let Some(tab) = tabs.iter_mut().find(|t| t.id == id) {
                tab.title = new_title;
            }
        });
    }
}

// ── TabBar component ────────────────────────────────────

#[component]
pub fn TabBar(
    state: TabState,
    /// Called when a SQL editor tab is renamed: (tab_id, old_name, new_name).
    #[prop(optional)]
    on_tab_rename: Option<Callback<(usize, String, String)>>,
) -> impl IntoView {
    let tabs = state.tabs;
    let active_id = state.active_id;
    let dirty_tabs = state.dirty_tabs;
    let state_rename = state.clone();
    let state_switch = state.clone();
    let state_close = state;

    view! {
        <div class="flex items-center h-10 border-b border-gray-200 dark:border-zinc-800 bg-white dark:bg-neutral-950 px-2 gap-0.5 overflow-x-auto shrink-0">
            {move || {
                let current_tabs = tabs.get();
                let current_active = active_id.get();

                if current_tabs.is_empty() {
                    return Vec::new();
                }

                current_tabs.into_iter().map(|tab| {
                    let tab_id = tab.id;
                    let is_active = current_active == Some(tab_id);

                    let base_class = if is_active {
                        "group flex items-center gap-1.5 px-3 py-1.5 text-[13px] text-gray-900 dark:text-neutral-50 bg-white dark:bg-neutral-950 border-b-2 border-indigo-500 cursor-pointer transition-colors duration-100 shrink-0"
                    } else {
                        "group flex items-center gap-1.5 px-3 py-1.5 text-[13px] text-gray-500 dark:text-zinc-400 rounded-t-md hover:text-gray-900 dark:hover:text-neutral-50 hover:bg-gray-50 dark:hover:bg-white/[0.03] cursor-pointer transition-colors duration-100 shrink-0"
                    };

                    let is_table = matches!(tab.kind, TabKind::TableView(_));
                    let is_sql = matches!(tab.kind, TabKind::SqlEditor);
                    let tab_title = tab.title.clone();

                    let switch = state_switch.clone();
                    let close = state_close.clone();
                    let rename_state = state_rename.clone();

                    let on_rename = if is_sql {
                        let old_name = tab_title.clone();
                        Some(Callback::new(move |new_name: String| {
                            rename_state.rename_tab(tab_id, new_name.clone());
                            if let Some(cb) = on_tab_rename {
                                cb.run((tab_id, old_name.clone(), new_name));
                            }
                        }))
                    } else {
                        None
                    };

                    view! {
                        <div
                            class=base_class
                            on:click=move |_| switch.switch(tab_id)
                        >
                            {if is_table {
                                view! { <IconTable class="w-3.5 h-3.5 text-gray-400 dark:text-zinc-500 shrink-0" /> }.into_any()
                            } else {
                                view! { <IconTerminal class="w-3.5 h-3.5 text-gray-400 dark:text-zinc-500 shrink-0" /> }.into_any()
                            }}
                            {if let Some(rename_cb) = on_rename {
                                view! { <TabTitle title=tab_title editable=is_sql on_rename=rename_cb /> }.into_any()
                            } else {
                                view! { <TabTitle title=tab_title editable=is_sql /> }.into_any()
                            }}
                            // Dirty indicator dot
                            {move || {
                                if dirty_tabs.with(|s| s.contains(&tab_id)) {
                                    Some(view! {
                                        <span class="w-1.5 h-1.5 rounded-full bg-gray-400 dark:bg-zinc-500 shrink-0" />
                                    })
                                } else {
                                    None
                                }
                            }}
                            <button
                                class="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-gray-200 dark:hover:bg-zinc-700 transition-opacity duration-100"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    close.close(tab_id);
                                }
                            >
                                <IconX class="w-3 h-3 text-gray-400 dark:text-zinc-500" />
                            </button>
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
