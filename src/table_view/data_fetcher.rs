use std::collections::HashSet;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::table_view::change_tracker::ChangeTracker;
use crate::tauri;

/// Manages data fetching, pagination, and loading state for the table view.
#[derive(Clone, Copy)]
pub struct DataFetcher {
    pub columns: ReadSignal<Vec<tauri::ColumnInfo>>,
    set_columns: WriteSignal<Vec<tauri::ColumnInfo>>,
    pub rows: RwSignal<Vec<Vec<serde_json::Value>>>,
    pub loading: ReadSignal<bool>,
    set_loading: WriteSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    set_error: WriteSignal<Option<String>>,
    pub loaded_table: ReadSignal<Option<String>>,
    set_loaded_table: WriteSignal<Option<String>>,
    pub page: ReadSignal<u32>,
    set_page: WriteSignal<u32>,
    pub page_size: ReadSignal<u32>,
    set_page_size: WriteSignal<u32>,
    pub total_count: ReadSignal<u64>,
    set_total_count: WriteSignal<u64>,
    pub has_data: ReadSignal<bool>,
    set_has_data: WriteSignal<bool>,
    pub active_filters: RwSignal<Vec<tauri::Filter>>,
    pub active_sort: RwSignal<Vec<tauri::SortCol>>,
    changes: ChangeTracker,
    selected_rows: RwSignal<HashSet<usize>>,
}

impl DataFetcher {
    /// Create a new data fetcher with its own reactive state.
    pub fn new(changes: ChangeTracker, selected_rows: RwSignal<HashSet<usize>>) -> Self {
        let (columns, set_columns) = signal(Vec::<tauri::ColumnInfo>::new());
        let rows = RwSignal::new(Vec::<Vec<serde_json::Value>>::new());
        let (loading, set_loading) = signal(false);
        let (error, set_error) = signal(Option::<String>::None);
        let (loaded_table, set_loaded_table) = signal(Option::<String>::None);
        let (page, set_page) = signal(1u32);
        let (page_size, set_page_size) = signal(50u32);
        let (total_count, set_total_count) = signal(0u64);
        let (has_data, set_has_data) = signal(false);
        let active_filters = RwSignal::new(Vec::<tauri::Filter>::new());
        let active_sort = RwSignal::new(Vec::<tauri::SortCol>::new());

        Self {
            columns,
            set_columns,
            rows,
            loading,
            set_loading,
            error,
            set_error,
            loaded_table,
            set_loaded_table,
            page,
            set_page,
            page_size,
            set_page_size,
            total_count,
            set_total_count,
            has_data,
            set_has_data,
            active_filters,
            active_sort,
            changes,
            selected_rows,
        }
    }

    /// Fetch table data for the given name, page, and page size.
    pub fn fetch_data(&self, name: String, pg: u32, ps: u32) {
        let set_loading = self.set_loading;
        let set_error = self.set_error;
        let set_total_count = self.set_total_count;
        let set_columns = self.set_columns;
        let rows = self.rows;
        let set_has_data = self.set_has_data;
        let filters = self.active_filters.get();
        let sort_cols = self.active_sort.get();

        self.changes.discard();
        self.selected_rows.set(HashSet::new());
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let result = if filters.is_empty() && sort_cols.is_empty() {
                tauri::get_table_data(&name, pg, ps).await
            } else {
                tauri::get_table_data_filtered(&name, pg, ps, filters, sort_cols).await
            };
            match result {
                Ok(td) => {
                    set_total_count.set(td.total_count);
                    set_columns.set(td.columns);
                    rows.set(td.rows);
                    set_has_data.set(true);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_has_data.set(false);
                }
            }
            set_loading.set(false);
        });
    }

    /// Set up the reactive effect that watches `table_name` and fetches data when it changes.
    pub fn watch_table_name(&self, table_name: Memo<Option<String>>) {
        let this = *self;
        Effect::new(move |_| {
            let name = table_name.get();
            let current = this.loaded_table.get();

            if name == current {
                return;
            }

            this.set_loaded_table.set(name.clone());

            if let Some(name) = name {
                this.set_page.set(1);
                this.active_filters.set(Vec::new());
                this.active_sort.set(Vec::new());
                this.fetch_data(name, 1, this.page_size.get());
            } else {
                this.set_has_data.set(false);
                this.set_total_count.set(0);
                this.set_loading.set(false);
            }
        });
    }

    /// Callback for page changes.
    pub fn on_page_change(&self) -> Callback<u32> {
        let this = *self;
        Callback::new(move |new_page: u32| {
            this.set_page.set(new_page);
            if let Some(name) = this.loaded_table.get() {
                this.fetch_data(name, new_page, this.page_size.get());
            }
        })
    }

    /// Callback for page size changes.
    pub fn on_page_size_change(&self) -> Callback<u32> {
        let this = *self;
        Callback::new(move |new_size: u32| {
            this.set_page_size.set(new_size);
            this.set_page.set(1);
            if let Some(name) = this.loaded_table.get() {
                this.fetch_data(name, 1, new_size);
            }
        })
    }

    /// Callback to refresh data for the current table.
    pub fn on_refresh(&self) -> Callback<()> {
        let this = *self;
        Callback::new(move |_: ()| {
            if let Some(name) = this.loaded_table.get() {
                this.fetch_data(name, this.page.get(), this.page_size.get());
            }
        })
    }

    /// Callback for filter/sort changes — re-fetches data.
    pub fn on_filter_change(&self) -> Callback<()> {
        let this = *self;
        Callback::new(move |_| {
            if let Some(name) = this.loaded_table.get() {
                this.fetch_data(name, this.page.get(), this.page_size.get());
            }
        })
    }
}
