# Task Plan

## In Progress

## Backlog

### Phase 29 — Test Infrastructure Setup
(Docker Postgres, seed SQL, setup/teardown scripts — already done, kept as-is)

### Phase 30 — Test HTTP Server for E2E
(First 3 tasks already completed — test server exists with all routes, CORS, port 3001)

### Phase 31 — E2E Tests (Playwright + real DB)
Real E2E tests using Playwright driving a real Chrome browser against the app in dev mode. The Tauri `invoke` bridge is replaced by a `window.__TAURI__` shim (injected via `page.addInitScript()`) that routes calls to a test HTTP server wrapping the real `db.rs` functions. **Zero changes to app code.**

Architecture:
```
Playwright (headless Chrome)
  → localhost:8080 (Trunk dev server, serves the app WASM — identical to prod)
    → window.__TAURI__.core.invoke("cmd", args)
      → fetch("http://localhost:3001/invoke/cmd", args)  [injected shim]
        → Test HTTP server (Rust/axum, imports crabase::db)
          → Docker Postgres (localhost:5433, crabase_test)
```

Prerequisites: Docker, Node.js, `npx playwright install chromium`.

**Infrastructure:**
- [ ] Create `tests/test_server/` as a Rust binary (workspace member or standalone). Uses `axum` to expose each `db.rs` function as `POST /invoke/{command}`. Connects to `postgresql://test:test@localhost:5433/crabase_test`. Handles JSON serialization matching what the frontend expects from `invoke`. Add CORS headers for `localhost:8080`. Serve on port 3001.
- [ ] Create `tests/e2e/tauri-shim.js` — the `addInitScript` content that defines `window.__TAURI__` with `core.invoke` routed to fetch, `event.listen` as no-op returning unlisten function, `dialog.open/save` returning null.
- [ ] Create `tests/e2e/playwright.config.ts` — configure Playwright: baseURL `http://localhost:8080`, use Chromium, set up `addInitScript` with the tauri shim.
- [ ] Create `tests/e2e/global-setup.ts` — starts Docker Postgres (`just test-setup`), starts the test HTTP server, starts Trunk dev server (`trunk serve --port 8080`), waits for all to be ready.
- [ ] Create `tests/e2e/global-teardown.ts` — stops Trunk, test server, Docker (`just test-teardown`).
- [ ] Install Playwright: `npm install -D @playwright/test`
- [ ] Add `just test-e2e` command to justfile
- [ ] Update `just test` to include `just test-e2e`

**E2E test files** (split by feature area, each file = one `test.describe` block):

`tests/e2e/connection.spec.ts`:
- [ ] Type the test connection string, click "Next", verify connection form appears with parsed fields (host=localhost, port=5433, user=test, db=crabase_test)
- [ ] On the connection form, click "Connect", verify the main layout appears with the sidebar showing real tables from the DB
- [ ] Verify the header shows connection info (test@localhost, :5433, crabase_test)
- [ ] Save a connection with a name, disconnect, verify the saved connection appears on the connection screen, click it, verify it fills the form

`tests/e2e/table-browsing.spec.ts`:
- [ ] Click "users" in sidebar, verify a tab opens, data table renders with real rows
- [ ] Verify column headers match the users table schema (id, username, email, role, ...)
- [ ] Verify pagination shows correct total count (12 rows)
- [ ] Click page 2, verify different rows appear
- [ ] Verify enum values display correctly (not raw JSON or type names)
- [ ] Verify timestamps display as formatted dates (not "Timestamp" literal)
- [ ] Verify NULL values display as "NULL" in gray italic

`tests/e2e/inline-editing.spec.ts`:
- [ ] Click on a cell (user's bio), verify it enters edit mode (input appears)
- [ ] Type a new value, press Enter, verify the cell shows the new value and the dirty bar appears
- [ ] Click "Discard", verify original value is restored and dirty bar disappears
- [ ] Edit a cell again, click "Save changes", verify dirty bar disappears (save persisted to real DB)
- [ ] Refresh the page, navigate back to the same table, verify the saved value persists

`tests/e2e/filters-sort.spec.ts`:
- [ ] Click "+" on filter bar, select column "role", operator "=", type "admin", verify table shows only 3 rows
- [ ] Add a second filter with OR combinator, verify results update
- [ ] Remove a filter, verify results update
- [ ] Click a column header to sort ascending, verify row order changes
- [ ] Click again for descending, verify order reverses

`tests/e2e/sql-editor.spec.ts`:
- [ ] Click "+" to open a new SQL editor tab, verify editor area is visible
- [ ] Type `SELECT * FROM users WHERE role = 'admin'`, click Run, verify results table shows 3 rows
- [ ] Type a multi-statement script (SELECT + INSERT), click Run, verify statement selector appears below with 2 entries
- [ ] Verify INSERT result shows affected rows in console style
- [ ] Save the query (Cmd+S or Save button), verify it appears in the sidebar "Saved Queries" section
- [ ] Rename the query by clicking on the tab title, verify the name updates

`tests/e2e/command-palette.spec.ts`:
- [ ] Press Cmd+Shift+P, verify command palette opens and input is focused
- [ ] Type "Restore", verify "Restore Backup" appears in filtered results
- [ ] Press Escape, verify palette closes
- [ ] Press Cmd+P, verify table finder opens with test tables listed
- [ ] Type "user", verify "users" is filtered, press Enter, verify users table tab opens
- [ ] Open Cmd+Shift+P then Cmd+P, verify first overlay closes and second opens (no stuck state)

`tests/e2e/schema-switching.spec.ts`:
- [ ] Change schema select in header to "test_schema", verify sidebar updates with test_schema tables
- [ ] Click a table, verify data loads from test_schema
- [ ] Switch back to "public", verify public tables return

`tests/e2e/theme.spec.ts`:
- [ ] Open command palette, select "Settings", verify settings view opens
- [ ] Toggle theme to dark, verify `<html>` element has "dark" class
- [ ] Toggle back to light, verify "dark" class is removed

`tests/e2e/context-menus.spec.ts`:
- [ ] Right-click on a table in sidebar, verify context menu appears with Export JSON, Export SQL, Truncate, Drop
- [ ] Right-click on a saved query in sidebar, verify context menu with Rename, Duplicate, Delete
- [ ] Right-click on a row in the data table, verify context menu with Delete, Duplicate, Copy as JSON, Copy as SQL INSERT

`tests/e2e/tabs.spec.ts`:
- [ ] Open multiple table tabs, switch between them, verify correct data each time
- [ ] Close a tab, verify it's removed and adjacent tab becomes active
- [ ] Open a SQL editor tab and a table tab, verify switching works correctly

### Phase 32 — Code Audit & Refactor: Backend (src-tauri/src/)
Audit the entire Rust backend for bad practices, memory issues, and code quality. For EACH file, read the latest Rust/sqlx/Tauri v2 docs if needed to verify correct usage. **Do NOT break any existing feature or test.** Run `just test` after every refactor to confirm nothing is broken.

- [ ] Audit `db.rs`: fix any memory leaks from `Mutex<Option<PgPool>>` (consider using `tokio::sync::RwLock` instead of `std::sync::Mutex` for async code). Remove `.clone()` on `PgPool` if unnecessary (PgPool is already `Arc` internally). Eliminate redundant `.lock()` calls. Extract duplicated pool+schema access into a helper method.
- [ ] Audit `db.rs` query builders: review `build_where_clause`, `build_set_clause`, `build_filter_where_clause`, `build_select_columns` for SQL injection edge cases. Ensure all user-provided values are parameterized (never interpolated into SQL strings).
- [ ] Audit `pg_value_to_json`: remove dead branches, simplify the match, ensure every type is handled without panic. Check for unwrap() calls that should be handled gracefully.
- [ ] Audit `restore.rs`: check for resource leaks (temp dirs, child processes). Ensure child processes are always waited on. Fix any unused functions (dead code warnings).
- [ ] Audit `saved_connections.rs` and `saved_queries.rs`: check for file system race conditions, proper error handling on read/write, validate file paths (path traversal prevention).
- [ ] Audit `settings.rs`: same as above — safe file I/O, proper defaults if file doesn't exist.
- [ ] Audit `lib.rs`: review every `#[tauri::command]` for proper error handling. Remove any `unwrap()` that could panic in production. Ensure all async commands use `spawn_blocking` for blocking I/O (file reads, subprocess spawns).
- [ ] Remove ALL dead code: unused functions, unused imports, commented-out code, stale TODOs. Address every compiler warning.
- [ ] Run `cargo clippy -- -W clippy::all -W clippy::pedantic` and fix all warnings that are reasonable to fix (skip false positives). Document any intentional suppressions with `#[allow(...)]` + a comment explaining why.

### Phase 33 — Code Audit & Refactor: Frontend (src/)
Audit the entire Leptos frontend for bad practices, memory leaks, and code quality. Read the Leptos 0.7+ docs to verify correct usage of signals, effects, and component lifecycle. **Do NOT break any existing feature.** Run `cargo check` after every change.

- [ ] Audit ALL event listeners registered with `closure.forget()`: these are memory leaks. For listeners that should live as long as the component, use `on_cleanup` properly. For global listeners (app lifetime), document why `forget()` is acceptable. Remove any duplicate listeners that get re-registered on re-render.
- [ ] Audit ALL `Effect::new(...)` usages: ensure no Effect writes to a signal that another Effect reads in a way that causes re-entrant borrows (the `RefCell::borrow_mut` crash). Use `set_untracked`, `get_untracked`, or `setTimeout(0)` deferral where needed.
- [ ] Audit ALL `spawn_local(async move { ... })` usages: ensure captured signals are valid for the async lifetime. Check for stale signal reads after async operations. Ensure errors are always handled (no silent swallows).
- [ ] Audit `main_layout.rs`: this file is too large. Extract the restore panel into its own component (`restore_panel.rs`). Extract header editing into its own component (`header_bar.rs` if not already done). Reduce the number of signals defined at the top level.
- [ ] Audit `table_view.rs`: also too large. Ensure the `on_save` callback properly handles all edge cases (empty PK, mixed operations). Check that `unwrap_tagged_owned` is called consistently everywhere values are sent to the backend.
- [ ] Audit `data_table.rs`: review the rendering closure for performance. Avoid re-creating DOM elements unnecessarily. Check that `selected_idx.get()` inside for loops doesn't cause excessive re-renders.
- [ ] Audit `cell_editor.rs` and all cell editors: ensure blur handlers don't fire after the component is unmounted. Check that on_commit is only called once per edit (not duplicated on blur+enter).
- [ ] Audit `sql_tab.rs`: check that CodeMirror instances are properly destroyed on unmount. Verify the save trigger Effect doesn't fire on initial mount.
- [ ] Audit `chat_panel.rs`: check that event listeners (`listen_chat_response`, `listen_chat_done`) are properly cleaned up after each message. Avoid accumulating listeners.
- [ ] Audit `overlay.rs`, `shortcuts.rs`, `theme.rs`: verify context providers are set up correctly and there are no stale context reads.
- [ ] Audit ALL components for proper `Clone` vs `Copy` usage on signals and callbacks. Remove unnecessary `.clone()` calls.
- [ ] Remove ALL dead code, unused imports, commented-out code, stale TODOs across the entire frontend.
- [ ] Run `cargo clippy -- -W clippy::all` on the frontend crate and fix all reasonable warnings.

### Phase 34 — Code Audit & Refactor: JS Bridge Layer
- [ ] Audit `js/codemirror-bridge.js`: review for memory leaks (stored editor instances that are never cleaned up). Ensure `destroy()` properly removes all event listeners and DOM nodes. Check that the `editors` map doesn't grow unbounded.
- [ ] Audit `js/markdown-bridge.js`: review `marked` configuration for XSS safety (ensure `sanitize` or DOMPurify is used since we render with `inner_html`). Add DOMPurify if not present.
- [ ] Review both JS bundles for unnecessary dependencies that could be tree-shaken.

### Phase 35 — Final Verification
- [ ] Run `just test` — ALL tests must pass
- [ ] Run `cargo clippy -- -W clippy::all` on both crates — zero warnings (or all intentionally suppressed with comments)
- [ ] Run `cargo check` on both crates — zero errors
- [ ] Run `just dev` and manually verify: connection flow, table browsing, inline editing + save, SQL editor + run, command palette, table finder, theme toggle, restore backup, AI chat (if Claude installed). Nothing should be broken.
- [ ] Commit all changes with a clear message

## Completed
- [x] `tests/e2e/command-palette.spec.ts`: open/close palette, filter commands, table finder, overlay switching
- [x] `tests/e2e/sql-editor.spec.ts`: open editor, run query, multi-statement, save
- [x] `tests/e2e/filters-sort.spec.ts`: add filter, sort asc/desc
- [x] `tests/e2e/inline-editing.spec.ts`: edit cell, dirty bar, discard, save, persist after reload
- [x] `tests/e2e/table-browsing.spec.ts`: click table, column headers, pagination, enum/NULL display
- [x] `tests/e2e/connection.spec.ts`: parse connection string, connect, verify header, save/load connection
- [x] Update `just test` to run test-frontend + test-e2e
- [x] Add `just test-e2e` command to justfile (`npx playwright test --config tests/e2e/playwright.config.ts`)
- [x] Install Playwright (`@playwright/test` as devDependency)
- [x] Create `tests/e2e/global-teardown.ts` — kills spawned processes via saved PIDs, runs just test-teardown
- [x] Create `tests/e2e/global-setup.ts` — starts Docker Postgres, test HTTP server, Trunk dev server, waits for readiness
- [x] Create `tests/e2e/playwright.config.ts` — Playwright config with baseURL localhost:8080, Chromium, tauri-shim.js injection, global setup/teardown
- [x] Create `tests/e2e/tauri-shim.js` — defines `window.__TAURI__` shim routing invoke to test HTTP server, event.listen as no-op, dialog.open/save as null
- [x] Add `just test-server` command to start the test server (`cargo run --manifest-path tests/test_server/Cargo.toml`)
- [x] Implement `POST /invoke/{command}` route for all commands, CORS for localhost:8080, port 3001
- [x] Create `tests/test_server/` as a standalone Rust binary (Cargo.toml with crabase path dep, axum, tokio, serde_json, tower-http). Implements `POST /invoke/{command}` for all commands, CORS for localhost:8080, serves on port 3001. File-based commands (settings, connections, queries) use in-memory state.
- [x] Frontend Vitest tests: codemirror-bridge (11 tests: create/destroy/getContent/setContent/isDirty/markClean/onChange/readOnly/json/multi-editor), markdown-bridge (14 tests: render, headings, code blocks, links, lists, empty string, nested formatting, SQL/JSON code, GFM tables, line breaks). `just test-frontend` command added.
- [x] Create `vitest.config.ts` with JSDOM environment
- [x] Install Vitest and jsdom as dev dependencies
- [x] (Removed) Backend integration tests replaced by Playwright E2E tests
- [x] Add `just test`, `just test-setup`, and `just test-teardown` commands to justfile
- [x] Create `tests/seed.sql` with comprehensive test schema (3 tables in public + 1 in test_schema, all Postgres types, 12 rows each, custom enums, arrays)
- [x] Create `tests/teardown.sh` script (stops and removes Docker container)
- [x] Create `tests/setup.sh` script (starts Docker, waits for Postgres, runs seed SQL)
- [x] Create `tests/docker-compose.yml` for a test Postgres container
- [x] Create a `tests/` directory at project root for the test infrastructure
- [x] Inline AI Chat Panel (Cmd+I): backend check_claude_installed, chat_with_claude (streaming), get_full_schema_for_chat; frontend chat_panel.rs side panel with message bubbles, auto-injected DB context, Claude not-installed message, fresh conversation per open
- [x] Multi-statement SQL execution: backend execute_query_multi returns Vec<StatementResult> (Rows/Affected/Error), frontend shows multi-statement navigator below results, statement selector with previews
- [x] Schema-aware SQL autocomplete: table names prefixed with schema when not on public, columns returned for correct tables
- [x] Full VS Code keybindings in CodeMirror (toggle comment, block comment, copy line, move line, delete line, find, find & replace, select next occurrence, go to line, indent/outdent) + registered in shortcuts.rs under Editor category
- [x] SQL Editor: tab title rename changed from double-click to single-click; Save button, Cmd+S, and dirty indicator verified working
- [x] SQL Editor: auto-focus on tab open/activation, click-to-focus everywhere (full-height editor), full-height CodeMirror via CSS height:100%, draggable resize handle between editor and results
- [x] JSON cell editor modal: CodeMirror 6 with @codemirror/lang-json (already implemented), scroll fixed, syntax highlighting working, read-only mode added, custom dark theme applied
- [x] SQL editor read-only result table: same cell formatting as editable table (boolean checkmarks, JSON clickable to read-only modal, arrays expanded, dates formatted)
- [x] **Bug fix**: timestamp/date columns now properly serialized via chrono with Postgres-compatible formatting; date picker output reformatted
- [x] **Bug fix**: schema-prefixed enums — fetch udt_schema for correct enum value lookup across schemas
- [x] **Bug fix**: when not on `public` schema, query results currently show each cell as the raw tagged-JSON object — frontend now always extracts the inner value via unwrap_tagged
- [x] Refactor overlay state management so only ONE overlay can be open at a time (Command Palette, Table Finder, Find Bar, Restore, Settings, Chat)
- [x] Opening any overlay must close any currently-open overlay first
- [x] Cmd+Shift+P → Command Palette closes Table Finder if open
- [x] Cmd+P → Table Finder closes Command Palette if open
- [x] Escape always closes the active overlay
- [x] Verify no "stuck" state where the user can't escape an overlay
- [x] Verify dark mode contrast across the entire app — find any remaining unreadable text (sql results table, JSON modal, settings inputs, etc.) and fix
- [x] Build a custom CodeMirror theme matching design.md exactly: editor background `#0A0A0A`, gutter background `#0A0A0A`, gutter text `text-zinc-600`, active line highlight `bg-white/[0.03]`, selection `bg-indigo-500/25`, cursor `text-neutral-50`. Replace the default `one-dark` theme with this custom theme.
- [x] Audit ALL table cell text styles: ensure every `text-gray-*` has a `dark:text-zinc-*` (target `dark:text-zinc-200`) so cell text is readable in dark mode
- [x] Tauri window background: set `backgroundColor` in `tauri.conf.json` to dark color (`#0A0A0A`) and ensure `<html>`/`<body>` use `bg-white dark:bg-neutral-950` so the white window edges no longer bleed through in dark mode
- [x] Verify that both windows share the same config files (settings, saved connections, queries)
- [x] Cmd+Shift+N opens a new app window (independent instance, starts at connection screen)
- [x] Backend: `open_new_window` command using Tauri WebviewWindowBuilder
- [x] Escape closes the overlay
- [x] Navigation with Enter (next) / Shift+Enter (prev), or N/Prev buttons
- [x] Highlights matching cells
- [x] Fuzzy search across all visible cell values
- [x] Cmd+F triggers the find overlay when a table tab is active
- [x] table_view/find_overlay.rs: floating bar at top of table view (browser-style)
- [x] Click table header to cycle sort direction (asc → desc → none)
- [x] Default sort behavior: created_at desc if exists, else PK asc, else first column with smart fallback
- [x] Sort: column + direction, can chain multiple sort columns
- [x] "+" button to add a new filter
- [x] table_view/filter_chip.rs: column select + operator select + value input + delete button + combinator selector
- [x] table_view/filter_bar.rs: inline bar below toolbar, always visible
- [x] SortCol struct: column, direction (asc/desc)
- [x] Filter struct: column, operator (=, !=, <, >, <=, >=, LIKE, NOT LIKE, IN, NOT IN, IS NULL, IS NOT NULL, contains, starts with, ends with), value, combinator (AND/OR/XOR for the previous filter)
- [x] Backend: `get_table_data_filtered(table_name, page, page_size, filters, sort)` extending get_table_data
- [x] Option: **Copy as SQL INSERT** — copies row(s) to clipboard as SQL INSERT statements
- [x] Option: **Copy as JSON** — copies row(s) to clipboard as JSON
- [x] Option: **Duplicate** — duplicates row(s) as new rows (green highlight, persists on save)
- [x] Option: **Delete** — marks row(s) for deletion (red highlight, persists on save)
- [x] Right-click on a row opens menu; right-click on a row that is part of a multi-selection keeps the selection and shows the same menu
- [x] context_menu.rs: right-click context menu component with options
- [x] Remove the inline delete button column from each row (deletion now via context menu)
- [x] Selected row visual: `bg-indigo-50 dark:bg-indigo-500/25`
- [x] Shift+click on index → range select (inclusive) between current and clicked row
- [x] Cmd+click on index → toggle row in selection (multi-select)
- [x] Click on index → select single row
- [x] Index column is sticky on horizontal scroll (CSS `sticky left-0`)
- [x] data_table.rs: add leftmost index column (no header label) showing global row index across pages (page 2 with 50 rows starts at 51)
- [x] Frontend: read-only mode for primary keys and auto-increment columns when editing existing rows
- [x] Frontend: NULL handling in editors — show a clear "Set NULL" / "×" affordance for nullable columns
- [x] Frontend: implement specialized editors per type (number, text, boolean, date, time, datetime, interval, uuid, enum, array modal, inet/cidr/macaddr, bit, range, bytea, xml modal, unknown)
- [x] Frontend: cell display formatting matches the type (boolean as checkmark icon, JSON/array truncated, array as `[a, b, c, ...]`)
- [x] All editors support both light and dark themes per specs/design.md
- [x] Frontend: refactor cell_editor.rs to dispatch to the correct specialized editor based on the column type from the new tagged value
- [x] Backend: extend `get_column_info` to return resolved type info: base_type, is_array, is_enum, enum_values (if applicable), is_nullable, is_primary_key, is_auto_increment, max_length, precision, scale.
- [x] Backend: for enum columns (USER-DEFINED with typcategory='E'), query `pg_enum` joined with `pg_type` to fetch allowed values. Cache per (schema, enum_name).
- [x] Backend: never return a non-NULL value as NULL because the type is unknown. Fall back to `{ "type": "unknown", "raw": "<text repr>" }`.
- [x] Backend: extend `pg_value_to_json` (or rewrite as a tagged serializer) to handle ALL common Postgres types per the mapping table. Output values as `{ "type": "<pg_type>", "value": ..., extras }` so the frontend knows the type.
- [x] Update table_finder.rs to fuzzy search across BOTH tables AND saved queries of current connection
- [x] Group results by type with subtle headers ("Tables", "Queries")
- [x] Selecting a query opens it in a new tab (same as sidebar click)
- [x] Click on saved query in sidebar opens it in a new tab (or focuses existing tab)
- [x] sidebar/saved_queries_list.rs: scrollable section above tables list, shows saved queries for current connection. Hidden if empty.
- [x] Default name: `Untitled-1`, `Untitled-2`, etc. (incremented globally per app instance)
- [x] tab_title.rs: clicking the file name on a SQL editor tab transforms it into an inline rename input. Save on blur or Enter, revert on Escape, calls `rename_query`
- [x] Dirty indicator (filled vs hollow dot) near the file name in tab title and toolbar
- [x] Cmd+S contextual: save SQL query in SQL editor tabs, save table changes in dirty table tabs
- [x] sql_editor/sql_toolbar.rs: add Save button (left of Run) with dirty state tracking and disabled state
- [x] Save query name conflict returns an error to display to the user
- [x] Backend commands: `save_query`, `update_query`, `rename_query`, `delete_query`, `list_queries`, `load_query`
- [x] Backend: saved_queries.rs module with CRUD scoped per connection hash (host:port:dbname:user)
- [x] JSON cell editor modal: use CodeMirror with @codemirror/lang-json for syntax-highlighted editing
- [x] SQL autocomplete: register a custom completion source with SQL keywords + tables of current schema + columns of those tables (fetched on editor mount)
- [x] Backend: `get_columns_for_autocomplete(table_names)` command — returns table → columns map
- [x] CodeMirror theme: switch between light theme and dark theme (one-dark) based on app theme
- [x] Auto-focus the editor when SQL editor tab is opened or activated
- [x] Replace existing SQL editor with CodeMirror integration. Verify VS Code shortcuts work natively (Cmd+Z, Cmd+Shift+Z, Cmd+F, Cmd+D, Cmd+/, Cmd+A)
- [x] Initialize Tauri v2 + Leptos (CSR) + Tailwind + DaisyUI
- [x] Configure sqlx with PostgreSQL connection pool
- [x] Tauri commands: connect_db, get_connection_info, list_tables, disconnect_db
- [x] Connection screen + form (host, port, user, pwd, db, schema, ssl)
- [x] Schema selector (connection form + header)
- [x] Saved connections (save, list, delete + UI)
- [x] Main layout: header + sidebar + tab bar + content area
- [x] Sidebar: list of tables, scrollable independently
- [x] Tab system (open, close, switch)
- [x] Table data viewer: columns, rows, pagination, refresh
- [x] Inline cell editing with specialized editors per type
- [x] Dirty state bar with save/discard
- [x] Backend save_changes (transactional inserts/updates/deletes)
- [x] SQL editor (basic textarea, run button, results display)
- [x] Command palette (Cmd+Shift+P) with fuzzy search
- [x] Table finder (Cmd+P) with fuzzy search on tables
- [x] Restore backup with streaming logs
- [x] Light theme applied per design.md
- [x] Component file structure refactor
- [x] Lucide icons imported
- [x] Inter + JetBrains Mono fonts
- [x] All UI text in English
- [x] Non-fatal pg_restore errors treated as success
- [x] pg_restore --clean --if-exists for idempotent restores
- [x] Add `dark` class strategy to Tailwind config and update Trunk build
- [x] Apply dark theme palette from specs/design.md to ALL existing components (use `dark:` variants)
- [x] Backend: `load_settings` and `save_settings` commands (read/write `app_data_dir/settings.json`)
- [x] Frontend: theme.rs provider that reads settings, applies `dark` class to `<html>`, exposes a toggle
- [x] Add "Disconnect" button in header that closes connection and returns to connection screen
- [x] Create settings/settings_view.rs that opens like the Restore Backup panel (special view in content area)
- [x] Add "Settings" command in the command palette (Cmd+Shift+P)
- [x] settings/theme_setting.rs: Light / Dark / System toggle that persists to settings.json and applies immediately
- [x] shortcuts.rs: keyboard shortcuts registry (default bindings + dispatcher)
- [x] settings/shortcut_input.rs: clickable shortcut input that listens for key combinations to bind
- [x] settings/shortcuts_settings.rs: list of all configurable shortcuts grouped by category, with click-to-rebind, "Reset to defaults" button
- [x] All existing shortcuts (Cmd+Shift+P, Cmd+P, Cmd+S, Cmd+/, Cmd+Z, Cmd+F, etc.) registered through shortcuts.rs and customizable
- [x] Add CodeMirror 6 dependencies via npm (@codemirror/state, @codemirror/view, @codemirror/lang-sql, @codemirror/lang-json, @codemirror/commands, @codemirror/autocomplete, @codemirror/search, @codemirror/theme-one-dark)
- [x] Create sql_editor/codemirror.rs: Leptos wrapper around CodeMirror 6 instance via JS interop (mount, unmount, get/set content, dirty tracking)
