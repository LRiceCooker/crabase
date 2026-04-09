# Task Plan

## In Progress

## Backlog

### Phase 5 — Inline Editing & Dirty State

### Phase 6 — SQL Editor

### Phase 7 — Table Finder & Command Palette Improvements
- [ ] Command palette: add keyboard shortcut hints right-aligned

## Completed
- [x] Tab close button appears on hover, active tab has indigo bottom border
- [x] Backend: `get_column_info(table_name)` command — returns column names, types, nullable, primary key
- [x] Backend: `get_table_data(table_name, page, page_size)` command — returns paginated rows + total count
- [x] UI: data_table.rs component — sticky headers, monospace cells, truncated, NULL as gray italic
- [x] UI: pagination.rs component — page X of Y, rows per page select, prev/next buttons
- [x] UI: toolbar with table name, row count, refresh button
- [x] UI: cell_editor.rs — click cell to edit, specialized editor per column type (text, number, boolean checkbox, enum select, date input)
- [x] UI: JSON/JSONB editor — modal with syntax-highlighted JSON editor on click
- [x] Track change state: map of modified cells, added rows, deleted rows
- [x] Row highlighting: added=emerald, modified=amber, deleted=red (per design.md)
- [x] Modified cells: amber-100/50 background
- [x] UI: dirty_bar.rs — floating bar at bottom with change count + Discard + Save buttons
- [x] Backend: `save_changes(table_name, changes)` command — applies inserts/updates/deletes in a single transaction
- [x] Add row button in toolbar (adds empty row, highlighted green)
- [x] Delete row button per row (marks for deletion, highlighted red)
- [x] UI: sql_editor.rs — text area with SQL syntax highlighting, line numbers, monospace
- [x] UI: sql_toolbar.rs — Run button (emerald, play icon)
- [x] Cmd+/ to toggle comment on selected lines
- [x] Backend: `execute_query(sql)` command — returns columns+rows on success, error message on failure
- [x] UI: sql_results.rs — success: read-only data table with results, error: dark console with error message
- [x] "+" button in header to open new SQL editor tab
- [x] Command palette: add "New SQL Editor" command
- [x] UI: table_finder.rs (Cmd+P) — fuzzy search tables, arrow keys + Enter to select, opens tab
- [x] Command palette: arrow up/down keyboard navigation + Enter to select + Escape to close
- [x] Clicking a table in sidebar opens a new tab (or focuses existing tab for that table)
- [x] Create main_layout.rs: sidebar + tab bar + content area (each zone scrolls independently, no full-page scroll)
- [x] Create tab_bar.rs component with tab state management (open, close, switch)
- [x] UI: "Save connection" checkbox + name field on connection form
- [x] UI: saved connections list on connection screen (click to fill, delete on hover)
- [x] Backend: `save_connection`, `list_saved_connections`, `delete_saved_connection` commands (store as JSON file in app data dir)
- [x] Apply light theme from specs/design.md to all existing components (connection screen, header, sidebar)
- [x] All UI text must be in English
- [x] Refactor frontend into component file structure per specs/project.md architecture
- [x] Import Lucide icons (lucide-leptos or SVG sprites)
- [x] Initialize Tauri v2 + Leptos (CSR) + Tailwind + DaisyUI
- [x] Configure sqlx with PostgreSQL connection pool
- [x] Create Tauri commands: connect_db, get_connection_info, list_tables, disconnect_db
- [x] Connection screen UI + integration
- [x] Main screen layout: header + sidebar + central area
- [x] Command palette (Cmd+Shift+P) with fuzzy search
- [x] Restore backup: UI + backend pg_restore + streaming logs
- [x] Schema selector (connection form + header)
- [x] Connection form with parsed details (host, port, user, pwd, db, schema, ssl)
- [x] Non-fatal pg_restore errors treated as success with warnings
- [x] Import Inter + JetBrains Mono fonts, configure Tailwind theme with design.md palette
