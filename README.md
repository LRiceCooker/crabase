<p align="center">
  <img src="src-tauri/crabase-alpha.png" width="180" alt="crabase mascot" />
</p>

<h1 align="center">crabase</h1>

<p align="center">
  A minimal, fast PostgreSQL desktop client built with Rust.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-blue?style=flat-square" />
  <img src="https://img.shields.io/badge/built_with-Rust-orange?style=flat-square" />
  <img src="https://img.shields.io/badge/ui-Leptos_%2B_Tailwind-06b6d4?style=flat-square" />
  <img src="https://img.shields.io/badge/license-Beerware-yellow?style=flat-square" />
</p>

---

<p align="center">
  <img src="./screens/screen.png" width="100%" alt="crabase screenshot" />
</p>

## What is crabase?

crabase is a lightweight PostgreSQL client that gets out of your way. Connect to any Postgres database, browse your tables, edit data inline, write SQL with a real code editor, and restore backups. Light and dark themes included.

Built with [Tauri v2](https://tauri.app), [Leptos](https://leptos.dev), and [CodeMirror 6](https://codemirror.net).

## Features

**Connect**

- Paste a connection string or fill in the details manually
- Save connections for quick access
- Schema selector with live switching
- SSL support

**Explore**

- Browse tables with paginated data and column types
- Filter rows with rich operators (=, !=, LIKE, IN, IS NULL, ...) and AND/OR/XOR logic
- Sort by any column, smart default ordering
- Search across all visible cells with Cmd+F
- Right-click rows to delete, duplicate, or copy as JSON/SQL

**Edit**

- Click any cell to edit inline
- Specialized editors for every Postgres type: date pickers, enum selects, boolean checkboxes, JSON modal with syntax highlighting, UUID generator, and more
- Add or delete rows, track all changes visually (green/orange/red), then save as a single transaction
- Cmd+S to save

**SQL Editor**

- Full CodeMirror 6 editor with SQL syntax highlighting
- Autocompletion for SQL keywords, table names, and column names (schema-aware)
- All VS Code keybindings: undo, redo, find, replace, multi-cursor, move lines, toggle comments, and more
- Execute multi-statement scripts with per-statement results
- Save and organize query files per connection
- Rename files inline by clicking the tab title

**AI Assistant**

- Press Cmd+I to open the chat panel
- Powered by your local [Claude Code](https://claude.ai/code) installation
- Full database schema injected as context
- Claude generates SQL, you click "Apply to Editor" to insert it

**Backup & Restore**

- Restore `.tar.gz` PostgreSQL backups via pg_restore
- Live streaming logs during restore
- Idempotent restores with `--clean --if-exists`

**Table Management**

- Right-click any table in the sidebar to drop, truncate, or export
- Export as JSON or SQL (INSERT statements) to a file

**Theming**

- Light and dark themes
- App icon adapts to the current theme
- Custom CodeMirror theme that matches the app palette
- Toggle via Settings or command palette

## Keyboard Shortcuts

### General

| Shortcut      | Action                                                |
| ------------- | ----------------------------------------------------- |
| `Cmd+Shift+P` | Command palette                                       |
| `Cmd+P`       | Search tables and saved queries                       |
| `Cmd+S`       | Save (context-dependent: query file or table changes) |
| `Cmd+Shift+N` | Open new window                                       |
| `Escape`      | Close any open overlay                                |

### SQL Editor

| Shortcut            | Action                 |
| ------------------- | ---------------------- |
| `Cmd+Z`             | Undo                   |
| `Cmd+Shift+Z`       | Redo                   |
| `Cmd+F`             | Find                   |
| `Cmd+Alt+F`         | Find & replace         |
| `Cmd+D`             | Select next occurrence |
| `Cmd+Shift+L`       | Select all occurrences |
| `Cmd+/`             | Toggle line comment    |
| `Cmd+Shift+A`       | Toggle block comment   |
| `Cmd+Shift+D`       | Copy line down         |
| `Alt+Up/Down`       | Move line up/down      |
| `Cmd+Shift+K`       | Delete line            |
| `Cmd+G`             | Go to line             |
| `Tab` / `Shift+Tab` | Indent / outdent       |
| `Cmd+I`             | Open AI chat panel     |

### Table View

| Shortcut      | Action                                 |
| ------------- | -------------------------------------- |
| `Cmd+F`       | Search in visible cells                |
| Click index   | Select row                             |
| `Cmd+Click`   | Multi-select rows                      |
| `Shift+Click` | Range select rows                      |
| Right-click   | Context menu (delete, duplicate, copy) |

All shortcuts are customizable in **Settings**.

## Tech Stack

| Layer    | Technology                                        |
| -------- | ------------------------------------------------- |
| Runtime  | [Tauri v2](https://tauri.app)                     |
| Backend  | Rust, [sqlx](https://github.com/launchbadge/sqlx) |
| Frontend | [Leptos](https://leptos.dev) (CSR/WASM)           |
| Editor   | [CodeMirror 6](https://codemirror.net)            |
| Styling  | [Tailwind CSS](https://tailwindcss.com)           |
| Icons    | [Lucide](https://lucide.dev)                      |
| Fonts    | Inter, JetBrains Mono                             |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) (latest stable)
- [Node.js](https://nodejs.org) (LTS)
- [Trunk](https://trunkrs.dev) (`cargo install trunk`)
- [Tauri CLI](https://tauri.app) (`cargo install tauri-cli --version "^2"`)
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- PostgreSQL client tools (`brew install libpq`) for backup restore

### Development

```bash
npm install
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

The `.app` and `.dmg` are generated in `src-tauri/target/release/bundle/`.

## License

[Beerware](https://en.wikipedia.org/wiki/Beerware)
