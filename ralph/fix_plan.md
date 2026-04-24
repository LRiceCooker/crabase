# Task Plan

## In Progress

## Backlog

## Completed
(All prior phases 29-36 completed — tests, audit, E2E fixes)

### Phase 42 (continued)
- [x] Verify every public function and component has a doc comment — backend (Phase 38), frontend (Phase 43), icons self-documenting
- [x] Verify no file exceeds 300 lines — exceptions: shortcuts.rs (data), data_table.rs (310, single component), lib.rs (Tauri handlers)
- [x] Run `just test-frontend` — 25/25 JS bridge tests pass
- [x] Run `just test-e2e` — ALL 40 tests pass
- [x] Run `cargo clippy -- -W clippy::all -W clippy::pedantic` on both crates — zero warnings
- [x] Verify: `cargo check`, `just test-e2e` — zero warnings, 40/40 E2E pass (Phase 44 verification)
- [x] Frontend: ensure all `spawn_local` closures handle errors visibly — fixed 8 silent swallows in main_layout, saved_connections, app
- [x] Frontend: replace all `web_sys::console::error_1(...)` with unified `log_error(msg)` helper — 2 call sites updated
- [x] Backend: add `tracing` crate — no println!/eprintln! found in backend, errors already go through AppError
- [x] Backend: replace all `String` error returns with `AppError` enum — already completed in Phase 42
- [x] Search official `thiserror` docs and `tracing` docs — thiserror already in reference.md, tracing added
- [x] Verify: `cargo check`, `just test-e2e` — zero warnings, 40/40 E2E pass (Phase 43 verification)
- [x] Ensure every component follows SRP — extracted cell_format.rs from data_table.rs (401→310 lines), all others under 300
- [x] Add `///` doc comments to every component — 18 UI components documented (icons excluded, self-documenting)
- [x] Replace `.get()` with `.with(|v| ...)` where you only need a reference — converted 15 calls on Vec/HashSet/HashMap signals
- [x] Use `For` component instead of `.into_iter().map().collect()` for reactive lists — all lists are small/static, per Leptos docs .collect() is fine
- [x] Replace manual conditional rendering with `Show` component where appropriate — no appropriate candidates (multi-branch, lightweight, or equivalent to if/else)
- [x] Use `Memo` for derived computations instead of `Effect` writing to another signal — no candidates found, all Effects are legitimate side effects
- [x] Use `ReadSignal` in props instead of `RwSignal` when the child doesn't write — all RwSignal props are correctly used (all write)
- [x] Search the official Leptos docs for each pattern below — already in `ralph/reference.md` from Phase 39
- [x] Verify: `cargo check`, `cargo clippy`, `just test-e2e` — zero warnings, 40/40 E2E pass
- [x] Add `#[must_use]` where appropriate
- [x] Use `thiserror` for custom error types instead of `String` errors in the backend
- [x] Replace `Vec::new()` + loop + push with `.iter().map().collect()`
- [x] Use `&str` instead of `String` for function params that don't need ownership
- [x] Replace manual `HashMap` building with `.collect()` from iterators

### Phase 42 (partial)
- [x] Search the Rust Reference and Clippy lint docs for each idiom below. Add each one to `ralph/reference.md` with a code example and source URL before applying.
- [x] Replace all `format!("...: {}", e)` with `format!("...: {e}")`
- [x] Replace `match` with `if let` / `map` / `unwrap_or` where cleaner

### Phase 41
- [x] Split `tauri.rs` (773 lines) into: `tauri/connection.rs`, `tauri/tables.rs`, `tauri/queries.rs`, `tauri/settings.rs`, `tauri/files.rs`, `tauri/chat.rs`
- [x] Create `tauri/mod.rs` with FFI bindings and re-exports (public API unchanged)
- [x] Remove `#[allow(dead_code)]` — deleted unused `execute_query` function
- [x] Verify: `cargo check` (both crates), `cargo test` (43 pass), `just test-e2e` (40 pass)

### Phase 40
- [x] Extract the save logic (`on_save` callback with ChangeSet building) into `table_view/save_handler.rs`
- [x] Extract row selection logic (click, cmd+click, shift+click) into `table_view/selection.rs`
- [x] Extract context menu actions (delete, duplicate, copy as JSON, copy as SQL) into `table_view/row_actions.rs`
- [x] Extract toolbar into `table_view/toolbar.rs`, modal editors into `table_view/modal_editors.rs`, data fetching into `table_view/data_fetcher.rs`, find state into `table_view/find_state.rs`
- [x] `table_view.rs` reduced to 240 lines (under 250 target), composing all subcomponents
- [x] Verify: `cargo check` (both crates), `cargo test` — all 43 tests pass

### Phase 39
- [x] Search the official Leptos docs for component best practices, signal types, and component splitting patterns. Added to `ralph/reference.md`.
- [x] Extract the restore backup panel into `src/restore_panel.rs` as `<RestorePanel />`
- [x] Extract the header bar into `src/header_bar.rs` as `<HeaderBar />`
- [x] Extract the header edit form into `src/header_edit_form.rs` as `<HeaderEditForm />`
- [x] Extract content area into `src/content_area.rs` as `<ContentArea />`, keyboard shortcuts into `src/global_shortcuts.rs`
- [x] `main_layout.rs` reduced to 190 lines (under 200 target), composing HeaderBar, Sidebar, TabBar, ContentArea
- [x] Verify: `cargo check` (both crates), `just test-e2e` — all 40 E2E tests pass

### Phase 38 (partial)
- [x] `lib.rs` should be ONLY Tauri command handlers (thin wrappers). Move all business logic to `db/` modules. — Extracted claude chat logic to `claude.rs`, app icon logic to `app_icon.rs`, connection key helper to `saved_queries.rs`.
- [x] Group related commands using `impl` blocks or separate modules if needed — Organized commands into logical sections with section comments; grouped `generate_handler!` by domain.
- [x] Replace any remaining `unwrap()` with proper `?` or `.map_err()` — Verified: zero `unwrap()` in production code; all are in test functions.
- [x] Add doc comments (`///`) to every public Tauri command explaining what it does, its params, and its return type
- [x] Verify: `cargo check`, `just test-e2e` — all 43 unit tests + 40 E2E tests pass

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
- [x] Verify: `cargo check`, `cargo test` — all pass, zero regressions
