# Task Plan

## In Progress

## Backlog

### Phase 37 — Deep Refactor: Backend `db.rs` (1380 lines → split into modules)
This file is a god-file. Split it into focused modules. Run `just test-e2e` after each step.
- [ ] Verify: `cargo check`, `cargo test`, `just test-e2e` — all pass, zero regressions

### Phase 38 — Deep Refactor: Backend `lib.rs` (418 lines → thin command layer)
- [ ] `lib.rs` should be ONLY Tauri command handlers (thin wrappers). Move all business logic to `db/` modules.
- [ ] Group related commands using `impl` blocks or separate modules if needed
- [ ] Replace any remaining `unwrap()` with proper `?` or `.map_err()`
- [ ] Add doc comments (`///`) to every public Tauri command explaining what it does, its params, and its return type
- [ ] Verify: `cargo check`, `just test-e2e`

### Phase 39 — Deep Refactor: Frontend `main_layout.rs` (709 lines → split)
- [ ] Search the official Leptos docs (https://book.leptos.dev/) for component best practices, signal types, and component splitting patterns. Add findings to `ralph/reference.md` with source URLs.
- [ ] Extract the restore backup panel into `src/restore_panel.rs` as a standalone `<RestorePanel />` component
- [ ] Extract the header bar (connection info badges, schema select, edit mode, disconnect) into `src/header_bar.rs` as `<HeaderBar />`
- [ ] Extract the header edit form (host/port/user/password/dbname editing) into `src/header_edit_form.rs`
- [ ] `main_layout.rs` should only compose: `<HeaderBar />`, `<Sidebar />`, `<TabBar />`, `<ContentArea />` — under 200 lines
- [ ] Verify: `cargo check`, `just test-e2e`

### Phase 40 — Deep Refactor: Frontend `table_view.rs` (724 lines → split)
- [ ] Extract the save logic (`on_save` callback with ChangeSet building) into `table_view/save_handler.rs`
- [ ] Extract row selection logic (click, cmd+click, shift+click) into `table_view/selection.rs`
- [ ] Extract context menu actions (delete, duplicate, copy as JSON, copy as SQL) into `table_view/row_actions.rs`
- [ ] The remaining `table_view.rs` should be the component shell composing subcomponents — under 250 lines
- [ ] Verify: `cargo check`, `just test-e2e`

### Phase 41 — Deep Refactor: Frontend `tauri.rs` (773 lines → split by domain)
- [ ] Split into: `tauri/connection.rs`, `tauri/tables.rs`, `tauri/queries.rs`, `tauri/settings.rs`, `tauri/files.rs`, `tauri/chat.rs`
- [ ] Create `tauri/mod.rs` that re-exports everything (public API unchanged)
- [ ] Remove all `#[allow(dead_code)]` — delete truly dead code, make used code public
- [ ] Verify: `cargo check`, `just test-e2e`

### Phase 42 — Clean Code Pass: Rust Idioms
- [ ] Search the Rust Reference and Clippy lint docs for each idiom below. Add each one to `ralph/reference.md` with a code example and source URL before applying.
- [ ] Replace all `format!("...: {}", e)` with `format!("...: {e}")`
- [ ] Replace `match` with `if let` / `map` / `unwrap_or` where cleaner
- [ ] Replace manual `HashMap` building with `.collect()` from iterators
- [ ] Use `&str` instead of `String` for function params that don't need ownership
- [ ] Replace `Vec::new()` + loop + push with `.iter().map().collect()`
- [ ] Use `thiserror` for custom error types instead of `String` errors in the backend
- [ ] Add `#[must_use]` where appropriate
- [ ] Verify: `cargo check`, `cargo clippy`, `just test-e2e`

### Phase 43 — Clean Code Pass: Leptos Best Practices
- [ ] Search the official Leptos docs for each pattern below. Add each one to `ralph/reference.md` with a code example and source URL before applying.
- [ ] Use `ReadSignal` in props instead of `RwSignal` when the child doesn't write
- [ ] Use `Memo` for derived computations instead of `Effect` writing to another signal
- [ ] Replace manual conditional rendering with `Show` component where appropriate
- [ ] Use `For` component instead of `.into_iter().map().collect::<Vec<_>>()` for reactive lists
- [ ] Replace `.get()` with `.with(|v| ...)` where you only need a reference
- [ ] Add `///` doc comments to every component
- [ ] Ensure every component follows SRP — one responsibility, under 150 lines ideally
- [ ] Verify: `cargo check`, `just test-e2e`

### Phase 44 — Clean Code Pass: Error Handling & Logging
- [ ] Search official `thiserror` docs and `tracing` docs for correct usage patterns. Add to `ralph/reference.md` with source URLs.
- [ ] Backend: replace all `String` error returns with a proper `AppError` enum using `thiserror`
- [ ] Backend: add `tracing` crate for structured logging, replace `println!`/`eprintln!` with `tracing::info!`/`tracing::error!`
- [ ] Frontend: replace all `web_sys::console::error_1(...)` with a unified `log_error(msg)` helper
- [ ] Frontend: ensure all `spawn_local` closures handle errors visibly, never silently swallow
- [ ] Verify: `cargo check`, `just test-e2e`

### Phase 45 — Final Verification
- [ ] Run `cargo clippy -- -W clippy::all -W clippy::pedantic` on both crates — zero warnings
- [ ] Run `just test-e2e` — ALL 40 tests pass
- [ ] Run `just test-frontend` — all JS bridge tests pass
- [ ] Verify no file exceeds 300 lines (except generated code/icons)
- [ ] Verify every public function and component has a doc comment

## Completed
(All prior phases 29-36 completed — tests, audit, E2E fixes)

### Phase 37
- [x] Search the official Rust API Guidelines for module organization, naming, and re-export conventions. Added to `ralph/reference.md` with source URLs.
- [x] Create `src-tauri/src/db/` directory with `mod.rs` that re-exports everything (public API stays identical)
- [x] Extract `ConnectionInfo`, `DbState`, `connect`, `disconnect`, `get_connection_info`, `get_connection_string` into `db/connection.rs`
- [x] Extract `list_tables`, `get_column_info`, `get_columns_for_autocomplete`, `ColumnInfo` into `db/schema.rs`
- [x] Extract `get_table_data`, `get_table_data_filtered`, `TableData`, `Filter`, `SortCol`, query builders into `db/query.rs`
- [x] Extract `save_changes`, `ChangeSet`, `RowUpdate`, `RowInsert`, `RowDelete`, mutation helpers into `db/mutations.rs`
- [x] Extract `execute_query`, `execute_query_multi`, `QueryResult`, `StatementResult` into `db/execute.rs`
- [x] Extract `drop_table`, `truncate_table`, `export_table_json`, `export_table_sql` into `db/table_ops.rs`
- [x] Extract `get_full_schema_text` into `db/introspection.rs`
- [x] Extract `pg_value_to_json`, `tagged`, `tagged_unknown`, `normalize_pg_type` into `db/types.rs`
- [x] Extract `parse_connection_string`, `build_connection_string`, `list_schemas` into `db/connection.rs` (already done in connection extraction step)
- [x] Move all `#[cfg(test)]` unit tests to their respective module files
