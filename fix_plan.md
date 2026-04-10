# Task Plan

## In Progress

## Backlog

### Phase 12 — Table Finder Enhancement (Cmd+P)
- [ ] Update table_finder.rs to fuzzy search across BOTH tables AND saved queries of current connection
- [ ] Group results by type with subtle headers ("Tables", "Queries")
- [ ] Selecting a query opens it in a new tab (same as sidebar click)

### Phase 13 — Comprehensive Postgres Type Support (Foundational Fix)
This is a foundational fix: the current table view incorrectly displays many Postgres types as NULL because the backend doesn't serialize them and the frontend has no specialized editors. See "Postgres Type Support" section in specs/project.md for the full type → editor mapping table.
- [ ] Backend: extend `pg_value_to_json` (or rewrite as a tagged serializer) to handle ALL common Postgres types per the mapping table. Output values as `{ "type": "<pg_type>", "value": ..., extras }` so the frontend knows the type.
- [ ] Backend: never return a non-NULL value as NULL because the type is unknown. Fall back to `{ "type": "unknown", "raw": "<text repr>" }`.
- [ ] Backend: extend `get_column_info` to return resolved type info: base_type, is_array, is_enum, enum_values (if applicable), is_nullable, is_primary_key, is_auto_increment, max_length, precision, scale.
- [ ] Backend: for enum columns (USER-DEFINED with typcategory='E'), query `pg_enum` joined with `pg_type` to fetch allowed values. Cache per (schema, enum_name).
- [ ] Frontend: refactor cell_editor.rs to dispatch to the correct specialized editor based on the column type from the new tagged value
- [ ] Frontend: implement specialized editors per type:
  - [ ] number_editor.rs (smallint, integer, bigint, decimal, real, double precision, money) — number input with proper precision/scale/range
  - [ ] text_editor.rs (char, varchar, text) — text input + textarea for long values
  - [ ] boolean_editor.rs — checkbox
  - [ ] date_editor.rs — native `<input type="date">`
  - [ ] time_editor.rs — native `<input type="time">`
  - [ ] datetime_editor.rs — native `<input type="datetime-local">` for timestamp/timestamptz
  - [ ] interval_editor.rs — text input with Postgres interval syntax validation
  - [ ] uuid_editor.rs — text input with UUID validation + generate button
  - [ ] enum_editor.rs — `<select>` with allowed values from column metadata
  - [ ] json_editor_modal.rs — modal with CodeMirror + lang-json (used for json and jsonb)
  - [ ] array_editor_modal.rs — modal with list editor + per-item editor of the element type
  - [ ] inet_editor.rs / cidr_editor.rs / macaddr_editor.rs — text input with format validation
  - [ ] bit_editor.rs — text input restricted to 0/1
  - [ ] range_editor.rs — two inputs (lower, upper) + bounds inclusivity toggles
  - [ ] bytea_editor.rs — hex preview + file upload
  - [ ] xml_editor_modal.rs — CodeMirror with lang-xml
  - [ ] unknown_editor.rs — read-only text with tooltip explaining the type isn't fully supported
- [ ] Frontend: NULL handling in editors — show a clear "Set NULL" / "×" affordance for nullable columns
- [ ] Frontend: read-only mode for primary keys and auto-increment columns when editing existing rows
- [ ] Frontend: cell display formatting matches the type (e.g., date as `YYYY-MM-DD`, boolean as checkmark icon, JSON truncated, array as `[a, b, c, ...]`)
- [ ] All editors must support both light and dark themes per specs/design.md

### Phase 14 — Table View: Index Column & Selection
- [ ] data_table.rs: add leftmost index column (no header label) showing global row index across pages (page 2 with 50 rows starts at 51)
- [ ] Index column is sticky on horizontal scroll (CSS `sticky left-0`)
- [ ] Click on index → select single row
- [ ] Cmd+click on index → toggle row in selection (multi-select)
- [ ] Shift+click on index → range select (inclusive) between current and clicked row
- [ ] Selected row visual: `bg-indigo-50 dark:bg-indigo-500/25`
- [ ] Remove the inline delete button column from each row (deletion now via context menu)

### Phase 15 — Right-Click Context Menu
- [ ] context_menu.rs: right-click context menu component with options
- [ ] Right-click on a row opens menu; right-click on a row that is part of a multi-selection keeps the selection and shows the same menu
- [ ] Option: **Delete** — marks row(s) for deletion (red highlight, persists on save)
- [ ] Option: **Duplicate** — duplicates row(s) as new rows (green highlight, persists on save)
- [ ] Option: **Copy as JSON** — copies row(s) to clipboard as JSON
- [ ] Option: **Copy as SQL INSERT** — copies row(s) to clipboard as SQL INSERT statements

### Phase 16 — Filter & Sort Bar (Linear-style)
- [ ] Backend: `get_table_data_filtered(table_name, page, page_size, filters, sort)` extending get_table_data
- [ ] Filter struct: column, operator (=, !=, <, >, <=, >=, LIKE, NOT LIKE, IN, NOT IN, IS NULL, IS NOT NULL, contains, starts with, ends with), value, combinator (AND/OR/XOR for the previous filter)
- [ ] SortCol struct: column, direction (asc/desc)
- [ ] table_view/filter_bar.rs: inline bar below toolbar, always visible
- [ ] table_view/filter_chip.rs: column select + operator select + value input + delete button + combinator selector
- [ ] "+" button to add a new filter
- [ ] Sort: column + direction, can chain multiple sort columns
- [ ] Default sort behavior: created_at desc if exists, else PK asc, else first column with smart fallback
- [ ] Click table header to cycle sort direction (asc → desc → none)

### Phase 17 — Find Bar Overlay (Cmd+F)
- [ ] table_view/find_overlay.rs: floating bar at top of table view (browser-style)
- [ ] Cmd+F triggers the find overlay when a table tab is active
- [ ] Fuzzy search across all visible cell values
- [ ] Highlights matching cells
- [ ] Navigation with Enter (next) / Shift+Enter (prev), or N/Prev buttons
- [ ] Escape closes the overlay

### Phase 18 — Multi-Window
- [ ] Backend: `open_new_window` command using Tauri WebviewWindowBuilder
- [ ] Cmd+Shift+N opens a new app window (independent instance, starts at connection screen)
- [ ] Verify that both windows share the same config files (settings, saved connections, queries)

## Completed
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
