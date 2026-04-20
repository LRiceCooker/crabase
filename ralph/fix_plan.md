# Task Plan

## In Progress

## Backlog

### Phase 29 — Test Infrastructure Setup

### Phase 30 — Rust Backend Integration Tests
Write integration tests in `src-tauri/tests/` that test every Tauri command against the real Docker Postgres. Each test connects to `postgresql://test:test@localhost:5433/crabase_test`.

### Phase 31 — Frontend Tests with Vitest
Set up Vitest for testing the Leptos/WASM frontend with mocked Tauri IPC. Since Playwright cannot drive Tauri's WKWebView on macOS, frontend tests use `@tauri-apps/api/mocks` to simulate the backend.

- [ ] Install Vitest and @tauri-apps/api/mocks: `npm install -D vitest @tauri-apps/api jsdom`
- [ ] Create `vitest.config.ts` with JSDOM environment and the Tauri mock setup
- [ ] Note: since the frontend is compiled to WASM via Leptos (not JS/TS), the Vitest tests focus on testing the JS bridge layer (`js/codemirror-bridge.js`, `js/markdown-bridge.js`) and any pure JS utilities
- [ ] Test `js/codemirror-bridge.js`: CodeMirror create/destroy/getContent/setContent/focus lifecycle
- [ ] Test `js/markdown-bridge.js`: `__markdown.render()` produces correct HTML for markdown input, code blocks have syntax highlighting classes
- [ ] Test markdown render handles edge cases: empty string, pure code block, nested formatting, SQL code block
- [ ] Create a simple smoke test that imports each JS bridge and verifies the global objects exist (`window.__codemirror`, `window.__markdown`)
- [ ] Add `just test-frontend` command to justfile

## Completed
- [x] Create `src-tauri/tests/integration_test.rs` with all Phase 30 tests (connect, disconnect, connection_info, list_schemas, list_tables, get_column_info, get_table_data, pagination, filters, sort, save_changes CRUD, execute_query, execute_query_multi, drop/truncate table, export JSON/SQL, autocomplete, full schema text, enum on non-public schema, timestamp format, NULL handling). Tests for save_connection/save_query/settings lifecycle are already covered by existing unit tests since they require tauri::AppHandle.
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
