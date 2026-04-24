# Specifications — crabase

## Description
**crabase** is a minimal, beautiful PostgreSQL desktop client.
Connect to a Postgres database, explore schemas and tables, edit data inline, run SQL queries with a real code editor, save query files per connection, filter/sort/search through table data, and restore backups. Light and dark themes supported.

## Stack
- **Backend**: Rust, Tauri v2, sqlx (PostgreSQL)
- **Frontend**: Leptos (CSR), Tailwind CSS (with `dark:` class strategy), DaisyUI (sparingly)
- **SQL Editor**: CodeMirror 6 (via JS interop) — handles syntax highlighting, undo/redo, find/replace, autocomplete, all VS Code-like keyboard shortcuts
- **Toolchain**: mise (rust, cargo, node LTS)
- **Icons**: Lucide
- **Fonts**: Inter (UI), JetBrains Mono (code/data)
- **Restore**: pg_restore (CLI subprocess)

## UI Language
All UI text must be in **English**.

## Design
Follow `specs/design.md` strictly for all colors, typography, spacing, components, and states.
Light theme only. Minimal, airy, no visual clutter.

## Architecture — Component Structure
The frontend must be split into small, focused components. One component per file. No god-files.

```
src/
  main.rs                — entry point
  app.rs                 — root component, routing (connection vs main)
  theme.rs               — theme provider (light/dark) reading from settings
  shortcuts.rs           — keyboard shortcuts manager (registry, dispatcher, customization)
  connection/
    mod.rs
    connection_screen.rs   — connection string input step
    connection_form.rs     — parsed details form
    saved_connections.rs   — list of saved connections
  main_layout.rs         — main layout: sidebar + tabs + content
  header_bar.rs          — top header bar (logo, connection info, schema, disconnect)
  sidebar/
    mod.rs
    sidebar.rs           — wraps queries section + tables section
    saved_queries_list.rs — saved query files for current connection (hidden if empty)
    tables_list.rs       — list of tables in selected schema
  tabs/
    mod.rs
    tab_bar.rs           — tab bar
    tab_title.rs         — single tab title (with rename input for queries)
    tab_content.rs       — router: table view, sql editor, settings
  table_view/
    mod.rs
    table_view.rs        — full table viewer
    table_toolbar.rs     — table name, count, refresh, add row
    filter_bar.rs        — inline filter+sort bar
    filter_chip.rs       — single filter chip
    find_overlay.rs      — Cmd+F find bar overlay
    data_table.rs        — the <table> itself (headers, rows, cells, index column, sticky)
    table_row.rs         — single row with selection + state styling
    index_cell.rs        — sticky leftmost index cell
    context_menu.rs      — right-click context menu (Delete, Duplicate, Copy as JSON, Copy as SQL)
    cell_editor.rs       — inline cell editor router (per column type)
    cell_editors/        — specialized editors
      text_editor.rs
      number_editor.rs
      boolean_editor.rs
      enum_editor.rs
      date_editor.rs
      json_editor_modal.rs
    pagination.rs        — pagination controls
    dirty_bar.rs         — floating "save changes" bar
  sql_editor/
    mod.rs
    sql_editor_tab.rs    — full SQL editor tab (toolbar + editor + results + chat panel)
    codemirror.rs        — CodeMirror 6 wrapper component (JS interop) with custom theme
    codemirror_theme.rs  — custom CodeMirror theme matching design.md
    sql_toolbar.rs       — Save + Run buttons + dirty indicator + file name input
    sql_results.rs       — multi-statement result navigator (selector + result viewer)
    sql_result_table.rs  — read-only data table for SELECT results (with click-to-view)
    sql_result_console.rs — console for affected/notice/error results
    chat_panel.rs        — Cmd+I chat panel (subprocess to claude -p)
    chat_messages.rs     — message bubble list
  settings/
    mod.rs
    settings_view.rs     — full settings panel
    theme_setting.rs     — theme toggle
    shortcuts_settings.rs — keyboard shortcut customization
    shortcut_input.rs    — single shortcut recorder input
  command_palette.rs     — Cmd+Shift+P command palette
  table_finder.rs        — Cmd+P table+query fuzzy finder
  tauri.rs               — all Tauri invoke bindings
```

Same one-file-per-component rule applies on the backend in `src-tauri/src/`:
```
src-tauri/src/
  lib.rs                 — Tauri command handlers
  db.rs                  — connection state, basic queries
  table_query.rs         — table data with filters, sort, pagination
  saved_queries.rs       — saved query CRUD (per connection_hash)
  settings.rs            — settings load/save
  restore.rs             — pg_restore subprocess
  saved_connections.rs   — saved connections CRUD
```

## Screens & Features

### 1. Connection Screen
- **Saved connections list**: shows previously saved connections (name, host, db). Click to auto-fill. Delete on hover.
- **Connection string input**: paste a full postgresql:// URL, click "Next"
- **Connection form**: editable fields: host, port, user, password, database, schema (select), SSL toggle
- **Save connection**: checkbox + name field to save the connection for next time
- **Connect button**: validates and connects

### 2. Main Layout (post-connection)
- **Header bar** (h-10): logo "crabase" left, "+" button to open new SQL editor tab, connection info badges (user@host, :port, db) + schema select + "Edit" button + "Disconnect" button right
- **Sidebar** (w-56, left), composed of two stacked sections each independently scrollable:
  1. **Saved Queries section** (top, only shown if there are saved queries for the current connection): list of saved query names. Click opens the query in a new tab (or focuses the existing tab for that query).
  2. **Tables section** (bottom, always shown): list of tables in selected schema. Click opens a tab. Scrollable.
- **Tab bar** (h-10): horizontal tabs. Each tab = a table view, SQL editor, or settings view. Close button on hover. Active tab has indigo bottom border.
- **Content area**: displays active tab content. Scrollable independently.

### App-level shortcuts and commands
- **Cmd+Shift+N**: opens a new application window (a fresh independent instance, starts at the connection screen). Both windows share the same config files (saved connections, settings, queries) but no real-time sync — changes are visible after reload.
- **Disconnect**: button in the header. Closes the DB connection and returns to the connection screen.
- **Cmd+Shift+P**: command palette (existing). Now also includes "Settings" command.
- **Cmd+P**: opens the table/query finder (fuzzy search across both tables AND saved queries of the current connection).

### 3. Table View (tab content)
- **Toolbar**: table name, row count, refresh button, "Add row" button
- **Filter & Sort bar** (inline, always visible below toolbar):
  - List of active filter chips
  - Each filter: column select + operator select + value input
  - Operators: `=`, `!=`, `<`, `>`, `<=`, `>=`, `LIKE`, `NOT LIKE`, `IN`, `NOT IN`, `IS NULL`, `IS NOT NULL`, `contains`, `starts with`, `ends with`
  - Logical combinator between filters: `AND` / `OR` / `XOR` (selectable per filter)
  - "+" button to add a filter
  - Sort: column + direction (asc/desc), can chain multiple sort columns
  - Default sort order:
    1. If table has a `created_at` column → sort by it desc
    2. Else if table has a primary key → sort by PK asc
    3. Else → sort by first column with smart tie-breaking
- **Find bar overlay** (Cmd+F): floating bar at the top of the table view (browser-style), fuzzy search across all visible cell values, highlights matches, navigation with N/Prev buttons or Enter / Shift+Enter
- **Data table**:
  - **Index column** (leftmost): displays the global row index (1, 2, 3...N across pages, e.g. page 2 with 50 rows/page starts at 51). No header label. **Sticky on horizontal scroll** (stays glued to the left when scrolling horizontally)
  - **Row selection**:
    - Click index → select single row
    - Cmd+click index → toggle add to selection (multi-select)
    - Shift+click index → range select between current and clicked row (inclusive)
    - Selected rows: `bg-indigo-50 dark:bg-indigo-500/25`
  - **Right-click** on a row (or any selected row): context menu appears with options:
    - **Delete** — marks selected row(s) for deletion (highlighted red, persists on save)
    - **Duplicate** — duplicates selected row(s) as new rows (highlighted green, persists on save)
    - **Copy as JSON** — copies selected row(s) to clipboard as JSON
    - **Copy as SQL INSERT** — copies selected row(s) to clipboard as SQL INSERT statements
  - **Headers**: sticky on vertical scroll, sortable (click to cycle: asc → desc → none), column type shown subtly
  - **Cells**: monospace, truncated, click to edit
  - NULL displayed as gray italic "NULL"
  - **All Postgres types must be supported** for both display AND editing — see "Postgres Type Support" section below for the full mapping.
  - Row states with left border + background:
    - Added: emerald (emerald-50 / dark:emerald-950/60 + border-emerald-500)
    - Modified: amber (amber-50 / dark:amber-950/60 + border-amber-500)
    - Deleted: red (red-50 / dark:red-950/60 + border-red-500 + line-through + opacity-60)
  - Modified cells: `bg-amber-100/50 dark:bg-amber-900/40`
- **Pagination bar**: page X of Y, rows per page, prev/next
- **Dirty state bar** (floating bottom): appears when there are unsaved changes. Shows change count + "Discard" + "Save changes" buttons. `Cmd+S` also triggers save when active tab is a table in dirty state. Saving persists all changes (inserts, updates, deletes) to the database in a single transaction.

### 4. SQL Editor (tab content)
- **Auto-focus**: when a SQL editor tab is opened or activated, the CodeMirror instance is **immediately focused** so the user can start typing without clicking. This includes when switching back to a SQL tab. Verify this works in practice — past iteration didn't fully fix it.
- **Click-to-focus**: clicking ANYWHERE in the editor area (not just the first line) focuses the CodeMirror instance and places the cursor at the nearest valid position. The current bug where only the first line is clickable must be fixed.
- **Full-height editor**: the CodeMirror instance fills 100% of the available editor area (toolbar height excluded) until the results pane below. The current bug where it stops mid-screen must be fixed. The editor area and results pane are split with a draggable resize handle (cursor-row-resize).
- **Editor**: CodeMirror 6 instance integrated via JS interop. Provides:
  - SQL syntax highlighting (`@codemirror/lang-sql`)
  - Line numbers in gutter
  - **Full VS Code keybindings** — must include AT LEAST: undo (`Cmd+Z`), redo (`Cmd+Shift+Z`), find (`Cmd+F`), find & replace (`Cmd+Alt+F`), select next occurrence (`Cmd+D`), select all occurrences (`Cmd+Shift+L`), toggle line comment (`Cmd+/`), toggle block comment (`Cmd+Shift+A`), select all (`Cmd+A`), copy line down (`Cmd+Shift+D`), move line up/down (`Alt+Up`/`Alt+Down`), delete line (`Cmd+Shift+K`), indent/outdent (`Tab`/`Shift+Tab`), go to line (`Cmd+G`), expand selection (`Cmd+Shift+Right`), shrink selection (`Cmd+Shift+Left`), word forward/back (`Alt+Right`/`Alt+Left`), home/end (`Cmd+Left`/`Cmd+Right`), document start/end (`Cmd+Up`/`Cmd+Down`)
  - All these shortcuts must be **registered in the shortcuts.rs registry** so they appear in Settings → Keyboard Shortcuts and are user-customizable
  - **Theme**: must use a dark theme that matches the app dark palette EXACTLY. CodeMirror background = `#0A0A0A` (same as app `dark:bg-neutral-950`). Only the focused/active line gets a subtle highlight (e.g. `bg-white/[0.03]`). Do not use the default `one-dark` theme — build a custom theme matching specs/design.md.
- **Autocompletion**: SQL keywords + tables of current schema + columns of those tables (fetched at editor mount via Tauri command). When NOT on the `public` schema, suggested table names must be prefixed with the schema (e.g. `myschema.users`). When on `public`, no prefix.
- **Toolbar** (h-10):
  - **File name** with click-to-rename input (see Tab title editing below) — currently broken, must be fixed
  - **Dirty indicator**: small dot/circle near the file name. Filled = unsaved, hollow = saved/synced
  - **Save** button (left of Run, with `Cmd+S` shortcut). Currently does nothing on click — must be fixed
  - **Run** button (emerald, play icon) at the top-right — must run the **entire content** of the editor, not just one line. See Multi-statement Execution below
- **Tab title editing**: clicking the file name in the tab transforms it into an inline input prefilled with the current name. Save on blur or Enter. Escape reverts. The new name is the file's persistent name. **Currently broken — fix it.**
- **Default name**: `Untitled-1`, `Untitled-2`, etc. (incremented globally per app instance)
- **Save behavior**:
  - `Cmd+S` saves the query to a JSON file scoped per connection (see Saved Queries below). Currently broken — fix it.
  - Save button does the same
  - Name conflict: if a saved query with the same name exists, show an error to the user

### Multi-statement SQL Execution
The current SQL editor only handles single-line / single-statement queries. **This must support arbitrary multi-statement scripts** (SELECTs, INSERTs, BEGIN/COMMIT, transactions, DDL, etc).

Approach: use sqlx's multi-statement support (`fetch_many` / `execute_many` or raw `simple_query` via the underlying driver). Send the entire editor content to Postgres in one call. Collect each statement's result.

- **Backend command**: `execute_query_multi(sql) → Vec<StatementResult>` where:
  ```rust
  enum StatementResult {
      Rows { columns: Vec<ColumnInfo>, rows: Vec<Vec<TaggedValue>> },  // SELECT
      Affected { command: String, rows_affected: u64 },                // INSERT/UPDATE/DELETE/etc
      Notice { message: String },                                       // NOTICE/RAISE
      Error { statement_index: usize, message: String },                // failure
  }
  ```
- **Frontend results pane**:
  - Below the editor
  - The result table (or console) is shown at the top of the pane
  - **Statement selector** (tabs or dropdown) is placed **BELOW the result table** (not above). The user explicitly said "en dessous du tableau, il y a un petit sélecteur".
  - Each selector entry shows the statement index + a short SQL preview
  - Clicking an entry switches the result shown in the table above
  - SELECT result → read-only data table with full type support (same as table view, but read-only). Cells of complex types (JSON, arrays) must remain clickable to open a read-only viewer modal.
  - INSERT/UPDATE/DELETE result → console-style output (`bg-gray-900 dark:bg-[#0D0D0F] text-zinc-200 font-mono`) showing `INSERT 0 5 — 5 rows affected`
  - Error → console-style output with red error message and the failing statement index highlighted
  - The selector must scroll horizontally if there are many statements

### 4b. Inline AI Chat Panel (Cmd+I)
A chat panel inside the SQL editor that talks to a local Claude Code installation via subprocess.

- **Trigger**: `Cmd+I` while a SQL editor tab is focused opens the chat panel. Same shortcut closes it.
- **UI**: a side panel (e.g. `w-96`) that slides in from the right of the editor area. Composed of:
  - Header: "AI Assistant" + close button
  - Message list (scrollable, alternating user/assistant bubbles)
  - Input textarea at the bottom + Send button (or Enter to send, Shift+Enter for newline)
- **Backend implementation**: spawn `claude -p "<prompt>" --output-format stream-json --dangerously-skip-permissions` as a subprocess for each message. Stream the JSON output back to the frontend via Tauri events. Parse `assistant` text events and append to the message bubbles.
- **Auto-injected context**: every message sent to Claude is prefixed with:
  - The full schema info of **ALL postgres schemas** of the connected database (not just the active schema). For each schema: list of tables, and for each table: columns + types + primary keys. The user explicitly said "il va avoir la connaissance de tous mes schémas" — Claude must have full database context to generate working SQL.
  - The current SQL editor content
  - The user's actual message
  Example prompt structure:
  ```
  You are a SQL assistant for a PostgreSQL database. Here is the schema of the current connection:
  
  Table users:
    id uuid PRIMARY KEY
    email text NOT NULL
    created_at timestamp
  
  Table orders:
    ...
  
  The user is currently editing this SQL:
  
  ```sql
  SELECT * FROM users
  ```
  
  User message: <user message>
  ```
- **Detection**: on app startup, check if `claude` is in PATH (`which claude`). If not, the chat panel opens but shows a message: "Claude Code is not installed. Install it from claude.com/code to use the AI assistant." with a clickable link. The input is disabled.
- **Conversation persistence**: not required for this iteration. Each opening starts a fresh conversation.

### Saved Queries
- Saved per connection, identified by a hash of `host:port:dbname:user`
- Stored as JSON files in `app_data_dir/queries/{connection_hash}/{query_name}.json`
- Each file contains: `{ name, content, created_at, updated_at }`
- Backend commands:
  - `save_query(connection_hash, name, content)` — error if name already exists
  - `update_query(connection_hash, name, content)` — overwrite existing
  - `rename_query(connection_hash, old_name, new_name)` — error if new_name already exists
  - `delete_query(connection_hash, name)`
  - `list_queries(connection_hash)` — returns Vec<{ name, updated_at }>
  - `load_query(connection_hash, name)` — returns the content

### Overlay Mutual Exclusion
There can only be **one overlay open at a time** (Command Palette, Table Finder, Find Bar, Restore Backup panel, Settings, Chat panel). Opening one overlay must close any currently-open overlay first. Specifically:
- Cmd+Shift+P opens Command Palette → if Table Finder is open, close it first
- Cmd+P opens Table Finder → if Command Palette is open, close it first
- Escape always closes the currently active overlay
- The state must NEVER reach a "stuck" state where the user can't close an overlay. **There is currently a bug**: opening Cmd+Shift+P then Cmd+P leaves both stacked and the user can't escape. This must be fixed by enforcing the mutual exclusion rule.

### 5. Command Palette (Cmd+Shift+P)
- VS Code style: centered overlay, search input on top, results below
- Fuzzy search on command names
- **Keyboard navigation**: arrow up/down to move, Enter to select, Escape to close
- Commands:
  - "Restore Backup" → opens restore panel
  - "New SQL Editor" → opens new SQL tab
  - (extensible)

### 6. Table & Query Finder (Cmd+P)
- Same visual style as command palette
- Lists **both** tables of current schema AND saved queries of current connection
- Fuzzy search by name across both
- Items grouped by type with subtle headers ("Tables", "Queries")
- Arrow keys + Enter to select → opens table or query in a new tab (or focuses existing tab)
- Escape to close

### 8. Settings (opens like the Restore Backup panel — as a special view in the content area, accessed via Cmd+Shift+P → "Settings")
- **Theme**: toggle between Light / Dark / System
- **Keyboard shortcuts**: list of all configurable shortcuts grouped by category. Each row shows the action name and the current shortcut on the right as a clickable indicator. Clicking the indicator clears it and listens for new key combinations to bind. Clicking outside without recording reverts to the previous binding. A "Reset to defaults" button at the bottom restores all shortcuts.
- Settings persisted to `app_data_dir/settings.json`
- Settings reload triggers UI update (theme switches immediately, shortcuts rebind immediately)

### 7. Restore Backup
- Panel in content area (or modal)
- Native file picker for .tar.gz
- .tar.gz contains a .pgsql file at root
- Restore via pg_restore CLI on the connected DB
- Live streaming logs
- Non-fatal errors (exit code 1 with warnings) treated as success with warnings

## Postgres Type Support

The data table must correctly **display** and **edit** all common Postgres types. Currently, many types fall back to NULL in the UI because the backend serialization is incomplete and the frontend lacks specialized editors. This is a foundational issue to fix.

### Backend (sqlx → JSON serialization)
The backend `pg_value_to_json` (or equivalent) must inspect each column's actual Postgres type via sqlx column metadata and serialize correctly. Wrap each value in a tagged JSON shape so the frontend knows the type:

```json
{ "type": "timestamp", "value": "2026-04-10T14:30:00" }
{ "type": "uuid", "value": "..." }
{ "type": "enum", "value": "ACTIVE", "enum_name": "user_status" }
{ "type": "jsonb", "value": { ... } }
{ "type": "null", "value": null }
```

For enum columns, the backend must also return the list of allowed values (queried from `pg_enum` joined with `pg_type`) so the select dropdown can be populated. Cache enum values per (schema, enum_name) per connection.

`get_column_info` must return the resolved type info: base type, is_array, is_enum, enum_values (if applicable), is_nullable, is_primary_key, max_length (for varchar), precision/scale (for numeric).

### Type → Editor Mapping

| Postgres type | Display | Edit component |
|---|---|---|
| `smallint`, `int2` | number | number input (range -32768 to 32767) |
| `integer`, `int`, `int4` | number | number input |
| `bigint`, `int8` | number (string-safe for large) | number input |
| `decimal`, `numeric` | number | number input with precision/scale |
| `real`, `float4` | number | number input |
| `double precision`, `float8` | number | number input |
| `smallserial`, `serial`, `bigserial` | number | number input (typically read-only when PK) |
| `money` | currency | number input with currency format |
| `character`, `char(n)` | text | text input with maxLength |
| `varchar(n)`, `character varying(n)` | text | text input with maxLength |
| `text` | text (truncated) | textarea (auto-grow) or modal for long values |
| `bytea` | hex preview (e.g. `\x4e2d6f...`) | hex viewer + file upload to set |
| `date` | `YYYY-MM-DD` | **native date picker** (`<input type="date">`) |
| `time`, `time without time zone` | `HH:MM:SS` | **native time picker** (`<input type="time">`) |
| `timetz`, `time with time zone` | `HH:MM:SS+TZ` | time picker + timezone select |
| `timestamp`, `timestamp without time zone` | `YYYY-MM-DD HH:MM:SS` | **native datetime picker** (`<input type="datetime-local">`) |
| `timestamptz`, `timestamp with time zone` | `YYYY-MM-DD HH:MM:SS±TZ` | datetime picker + timezone display |
| `interval` | `1 day 02:00:00` | text input with parser (validate Postgres interval syntax) |
| `boolean`, `bool` | checkmark icon | checkbox |
| `uuid` | uuid string | text input with UUID validation + button to generate new UUID |
| `json` | truncated JSON preview | **modal with CodeMirror + lang-json** (syntax highlighting + format + scroll). Also clickable in read-only mode (read-only modal) |
| `jsonb` | truncated JSON preview | **modal with CodeMirror + lang-json**. Same as json. Currently broken: no scroll + no syntax highlighting — must be fixed by using the same CodeMirror setup as the SQL editor |
| `xml` | truncated text | modal with CodeMirror + lang-xml |
| `inet` | IP/CIDR string | text input with IP validation |
| `cidr` | CIDR string | text input with CIDR validation |
| `macaddr`, `macaddr8` | MAC string | text input with MAC validation |
| `bit`, `bit varying` | binary string | text input restricted to 0/1 |
| `tsvector`, `tsquery` | preview | read-only text |
| `point`, `line`, `lseg`, `box`, `path`, `polygon`, `circle` | text repr | text input (advanced/rare) |
| `int4range`, `int8range`, `numrange`, `tsrange`, `tstzrange`, `daterange` | range repr | range editor (two inputs: lower/upper + bounds inclusivity toggles) |
| **Enum** (`USER-DEFINED` with typcategory='E') | label | **select dropdown with allowed values** |
| **Array** (any `_xxx` or `xxx[]`) | comma-separated preview, max 3 items + "..." | array editor modal: list of items with add/remove + per-item editor of the element type |
| **Composite** (custom row type) | text repr | text input (advanced) |
| **Domain** types | inherit base type | inherit base type editor |

### Rules
- **Never silently render a non-NULL value as NULL** because the type is unsupported. If the backend doesn't know how to serialize a type, return `{ "type": "unknown", "raw": "<text repr>" }` and the frontend renders it as a read-only text cell with a tooltip explaining the type isn't fully supported.
- **Never display the raw tagged-JSON object** (`{"raw":"TIMESTAMP","type":"unknown"}`) in the cell. The frontend MUST always extract `value` (or `raw`) and render it appropriately. **There is currently a bug**: when not on the `public` schema, query results show each cell as the raw JSON object. This must be fixed in BOTH the table view AND the SQL editor result table.
- **Schema-prefixed enums must be detected**: if the column type from sqlx is `"schema_name"."enum_name"` (or any other schema), the backend must still detect it as an enum, fetch the values from `pg_enum`, and tag it as `{ type: "enum", value: "ACTIVE", enum_name: "user_status" }`. The frontend must display only the value (e.g. `ACTIVE`), never the schema-qualified type or the JSON wrapper. **Currently broken** for non-public schemas.
- **Timestamp/date columns must show formatted dates**, not the literal type name. Currently `created_at`/`deleted_at` columns sometimes show `Timestamp` as a string — this is wrong. They must show the actual value formatted as `YYYY-MM-DD HH:MM:SS` (or similar) and clicking opens the date/datetime picker. The picker output must be re-formatted to a Postgres-compatible string before sending to the backend.
- **Read-only result tables** (in the SQL editor results pane) must display values **with the same formatting and click-to-view behavior** as the editable table view. Specifically: JSON cells are still clickable to open the JSON modal in **read-only mode**; arrays expand the same way; enums show their value; dates show formatted dates. The only difference is no editing.
- **Auto-increment columns** (serial, bigserial, identity) must be detected and rendered as read-only when displaying existing rows; for new rows, leave empty (DB generates the value).
- **Primary key columns** are read-only when editing existing rows (otherwise we can't generate the WHERE clause for the UPDATE).
- **Nullable columns** must show a clear way to set the value to NULL (small "×" or "Set NULL" option in the editor).
- All editors must respect the **light AND dark themes** per specs/design.md.

## Backend Commands (Tauri)

Existing:
- `parse_connection_string(connection_string) → ConnectionInfo`
- `list_schemas(connection_string) → Vec<String>`
- `connect_db(info: ConnectionInfo) → String`
- `disconnect_db() → String`
- `get_connection_info() → ConnectionInfo`
- `list_tables() → Vec<String>`
- `get_table_data(table_name, page, page_size) → TableData`
- `get_column_info(table_name) → Vec<ColumnInfo>`
- `save_changes(table_name, changes: ChangeSet) → Result`
- `execute_query(sql) → QueryResult | QueryError`
- `save_connection(name, info: ConnectionInfo) → Result`
- `list_saved_connections() → Vec<SavedConnection>`
- `delete_saved_connection(name) → Result`
- `restore_backup(file_path) → String` (with streaming events)

New:
- `get_table_data_filtered(table_name, page, page_size, filters: Vec<Filter>, sort: Vec<SortCol>) → TableData` — extends get_table_data with filters and sort
- `get_columns_for_autocomplete(table_names: Vec<String>) → HashMap<String, Vec<String>>` — for SQL autocomplete (table → columns)
- **Saved queries** (scoped per connection by hash of host:port:dbname:user):
  - `save_query(connection_hash, name, content) → Result`
  - `update_query(connection_hash, name, content) → Result`
  - `rename_query(connection_hash, old_name, new_name) → Result`
  - `delete_query(connection_hash, name) → Result`
  - `list_queries(connection_hash) → Vec<SavedQueryMeta>`
  - `load_query(connection_hash, name) → SavedQuery`
- **Settings**:
  - `load_settings() → Settings` — reads from app_data_dir/settings.json
  - `save_settings(settings: Settings) → Result`
- **Multi-window**:
  - `open_new_window() → Result` — opens a new app window via Tauri WebviewWindowBuilder

New for Phase 19+:
- `execute_query_multi(sql) → Vec<StatementResult>` — multi-statement SQL execution returning a result per statement
- `check_claude_installed() → bool` — checks if `claude` is in PATH
- `chat_with_claude(prompt) → Stream<String>` — spawns `claude -p "<prompt>" --output-format stream-json --dangerously-skip-permissions` and streams parsed assistant text via Tauri events
- `get_full_schema_for_chat() → String` — returns a formatted text representation of the current schema (all tables + columns + types) suitable for injecting as context into a Claude prompt

## Testing Strategy

Playwright/WebdriverIO **cannot drive Tauri's WKWebView on macOS**. Therefore, the testing strategy is:

### 1. Test Infrastructure (`tests/`)
- `docker-compose.yml`: PostgreSQL 16 Alpine container on port 5433 (user: `test`, password: `test`, db: `crabase_test`)
- `seed.sql`: creates test tables covering ALL supported Postgres types (text, integer, boolean, timestamp, timestamptz, date, uuid, json, jsonb, custom enums, arrays, etc.) with sample data
- `setup.sh`: starts Docker container + waits for ready + runs seed
- `teardown.sh`: stops + removes Docker container

### 2. Rust Backend Integration Tests (`src-tauri/tests/`)
- Test every Tauri command against the real Docker Postgres
- Connect to `postgresql://test:test@localhost:5433/crabase_test`
- Cover: connect/disconnect, list_schemas, list_tables, get_column_info, get_table_data (with pagination, filters, sort), save_changes (insert/update/delete), execute_query_multi, drop/truncate/export, saved connections CRUD, saved queries CRUD, settings, enum handling on non-public schemas, timestamp formatting, NULL handling
- Tests must be independent (runnable in any order)

### 3. E2E Tests (Playwright + real DB)
True end-to-end tests. Playwright drives headless Chrome against the app running in dev mode (Trunk at localhost:8080). A `window.__TAURI__` shim injected via `page.addInitScript()` routes `invoke()` calls to a test HTTP server (Rust/axum on port 3001) that wraps the real `db.rs` functions against Docker Postgres. **Zero changes to app code** — the WASM binary is identical to production.

```
Playwright (headless Chrome)
  → localhost:8080 (Trunk, serves app WASM)
    → window.__TAURI__.core.invoke(cmd, args)  [injected shim]
      → fetch("http://localhost:3001/invoke/{cmd}", args)
        → Test HTTP server (Rust/axum, imports crabase::db)
          → Docker Postgres (localhost:5433)
```

- Test files in `tests/e2e/*.spec.ts`, split by feature area
- Covers: connection flow, table browsing, inline editing + save, filters/sort, SQL editor + multi-statement, command palette, table finder, schema switching, theme toggle, context menus, tabs

### 4. Frontend JS Bridge Tests (Vitest)
- Test `codemirror-bridge.js` and `markdown-bridge.js`
- Run with `just test-frontend`

### Justfile commands
- `just test` — full run (setup Docker → backend tests → E2E tests → JS tests → teardown)
- `just test-setup` / `just test-teardown` — Docker management
- `just test-e2e` — Playwright E2E tests only (requires Docker + Trunk + test server running)
- `just test-frontend` — Vitest JS bridge tests only

## Constraints
- Tauri v2 APIs only (not v1)
- Backend errors with `thiserror` or `anyhow`
- Unit tests for backend logic
- Frontend communicates via Tauri `invoke`
- pg_restore must be installed on the system
- All save_changes operations must be wrapped in a database transaction
