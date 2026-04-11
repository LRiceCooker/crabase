# Task Plan

## In Progress

## Backlog

### Phase 19 — Dark Theme Fixes (Critical Visual Bugs)

### Phase 20 — Overlay Mutual Exclusion Bug Fix

### Phase 21 — Type Display Fixes
- [ ] **Bug fix**: timestamp/date columns in tables (e.g. `created_at`, `deleted_at`) currently sometimes show the literal type name `Timestamp` as a string. Fix the backend serialization to always return the actual value, formatted as ISO string. Frontend displays as `YYYY-MM-DD HH:MM:SS` and clicking the cell opens a **date picker** to edit it. **The date picker output MUST be re-formatted to a Postgres-compatible string** (e.g. `2026-04-10 14:30:00` for timestamp, `2026-04-10` for date, `2026-04-10T14:30:00Z` for timestamptz) before sending to the backend, otherwise the SQL UPDATE will fail.
- [ ] SQL editor read-only result table: must use the same cell formatting and click-to-view behavior as the editable table view. JSON cells clickable to open the JSON modal in **read-only mode**, arrays expand, enums show their value, dates are formatted.

### Phase 22 — JSON Modal Fixes
- [ ] JSON cell editor modal: replace current implementation with CodeMirror 6 + `@codemirror/lang-json` (same setup as the SQL editor)
- [ ] Ensure the modal content scrolls properly (currently broken — no scroll)
- [ ] Syntax highlighting must work (currently broken)
- [ ] Add a read-only mode for the JSON modal (used in the SQL editor result table)
- [ ] Use the custom dark theme so it matches the app

### Phase 23 — SQL Editor Critical Fixes
- [ ] **Auto-focus** on tab open and tab activation: the CodeMirror instance must be focused immediately when a SQL editor tab is opened or switched to, without the user clicking. Currently broken.
- [ ] **Click-to-focus everywhere**: clicking ANYWHERE in the editor area (not just the first line) focuses CodeMirror at the nearest valid position. Currently only the first line is clickable. Fix the underlying layout/CSS issue.
- [ ] **Full-height editor**: the CodeMirror instance must fill 100% of the available editor area (toolbar excluded) until the results pane. Currently it stops mid-screen when many lines are added. Fix the CSS sizing.
- [ ] Add a draggable resize handle (cursor-row-resize) between the editor area and the results pane

### Phase 24 — SQL Editor: Save & Rename Fixes
- [ ] **Bug fix**: clicking the file name in the SQL editor tab title currently does NOT open an inline rename input. Fix it. The input must be prefilled with the current name, save on Enter or blur, revert on Escape, and call `rename_query`.
- [ ] **Bug fix**: clicking the Save button currently does nothing. Wire it up to actually save the query.
- [ ] **Bug fix**: `Cmd+S` currently does nothing in the SQL editor. Wire it up via the shortcuts.rs registry.
- [ ] Verify the dirty indicator (filled vs hollow dot) updates correctly on edit and save

### Phase 25 — Full VS Code Keybindings + Settings Integration
- [ ] Implement the full set of VS Code editing shortcuts in CodeMirror, going beyond Cmd+Z/Cmd+Shift+Z. AT MINIMUM:
  - find (`Cmd+F`), find & replace (`Cmd+Alt+F`)
  - select next occurrence (`Cmd+D`), select all occurrences (`Cmd+Shift+L`)
  - toggle line comment (`Cmd+/`), toggle block comment (`Cmd+Shift+A`)
  - copy line down (`Cmd+Shift+D`), move line up/down (`Alt+Up`/`Alt+Down`)
  - delete line (`Cmd+Shift+K`)
  - indent/outdent (`Tab`/`Shift+Tab`)
  - go to line (`Cmd+G`)
  - expand/shrink selection (`Cmd+Shift+Right`/`Cmd+Shift+Left`)
  - word forward/back (`Alt+Right`/`Alt+Left`)
  - line home/end (`Cmd+Left`/`Cmd+Right`)
  - document start/end (`Cmd+Up`/`Cmd+Down`)
- [ ] Register ALL these shortcuts in the `shortcuts.rs` registry under a new "Editor" category
- [ ] Verify they appear in Settings → Keyboard Shortcuts and are user-customizable
- [ ] When the user customizes a shortcut, it must update the CodeMirror keybinding at runtime

### Phase 26 — SQL Autocomplete: Schema-Aware
- [ ] When the active schema is NOT `public`, all suggested table names in autocomplete must be prefixed with the schema (e.g. `myschema.users` instead of just `users`)
- [ ] When the active schema IS `public`, do not prefix
- [ ] Verify autocomplete also returns columns for the correct tables across schemas

### Phase 27 — Multi-Statement SQL Execution
- [ ] Backend: rename `execute_query` → `execute_query_multi` (or add new command). Use sqlx multi-statement support (`fetch_many` / `simple_query`) to execute the entire editor content as a single multi-statement script.
- [ ] Backend: return `Vec<StatementResult>` where each statement is one of:
  - `Rows { columns, rows }` for SELECTs
  - `Affected { command, rows_affected }` for INSERT/UPDATE/DELETE/etc.
  - `Notice { message }` for NOTICE/RAISE
  - `Error { statement_index, message }` for failures
- [ ] Frontend: replace single-result results pane with a multi-statement result navigator
  - Statement selector (tabs or dropdown) **BELOW the result table** (at the bottom of the results pane), not above. The user explicitly said "en dessous du tableau".
  - The selector is scrollable horizontally if there are many statements
  - Each entry shows the statement index + a short preview of the SQL
  - Clicking an entry switches the result shown in the table above
- [ ] sql_result_table.rs: read-only data table for SELECT results. Same type display as the table view (clickable JSON, formatted dates, enums, etc.). Currently broken — fix.
- [ ] sql_result_console.rs: console-style output for Affected/Notice/Error results. Same dark style as the existing error console.
- [ ] Run button executes the entire editor content (no more single-line limitation)

### Phase 28 — Inline AI Chat Panel (Cmd+I)
- [ ] Backend: `check_claude_installed()` command — checks if `claude` is in PATH via `which claude` and returns bool
- [ ] Backend: `chat_with_claude(prompt)` command — spawns `claude -p "<prompt>" --output-format stream-json --dangerously-skip-permissions` as a subprocess. Streams parsed `assistant` text events back via Tauri events to the frontend
- [ ] Backend: `get_full_schema_for_chat()` command — returns a formatted text representation of **ALL postgres schemas** of the connected database (not just the active schema). For each schema: list of tables, and for each table: columns + types + primary keys. The user explicitly said "il va avoir la connaissance de tous mes schémas" — full database context.
- [ ] sql_editor/chat_panel.rs: side panel (`w-96`) that slides in from the right side of the editor area
- [ ] sql_editor/chat_messages.rs: scrollable list of message bubbles (alternating user/assistant)
- [ ] Cmd+I toggles the chat panel (open if closed, close if open)
- [ ] Register Cmd+I in shortcuts.rs as "Open AI Chat" under a new "AI" category
- [ ] When sending a message, prefix with the auto-injected context: full schema + current SQL editor content + user message
- [ ] If `check_claude_installed` returns false, the chat panel opens but shows a message: "Claude Code is not installed. Install it from claude.com/code to use the AI assistant." Input is disabled.
- [ ] Each new chat panel opening starts a fresh conversation (no persistence required for this iteration)

## Completed
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
