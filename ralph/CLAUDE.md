# Agent Context

## Stack
- **Backend**: Rust + Tauri v2 + sqlx (PostgreSQL)
- **Frontend**: Leptos (CSR) + Tailwind CSS (with `dark:` class strategy) + DaisyUI (sparingly)
- **SQL Editor**: CodeMirror 6 via JS interop (handles syntax highlighting, undo/redo, find, autocomplete, all VS Code-like shortcuts natively)
- **Icons**: Lucide
- **Fonts**: Inter (UI), JetBrains Mono (code/data)
- **Toolchain**: mise (rust, cargo, node LTS)
- **Restore**: pg_restore (CLI subprocess)
- **Themes**: Light (default) + Dark, both defined in specs/design.md

## Architecture
- Backend exposes Tauri commands, frontend calls via `invoke`
- sqlx for all DB interactions (connection, queries, schema introspection)
- pg_restore subprocess for backup restores
- Frontend is a Leptos SPA compiled to WASM (CSR)
- **One component per file** — follow the file structure in specs/project.md
- **Design system**: follow specs/design.md strictly for all UI

## Rules
- One task per iteration
- Always run `cargo check && cargo test` after every change (both root and src-tauri/)
- Search existing code before implementing (avoid duplicates)
- Commit after each completed task
- Document decisions and learnings below
- Never bundle pg_restore — must be installed on system
- Use Tauri v2 APIs exclusively (not v1)
- All UI text in English
- **Every UI element must support both light AND dark themes** (use Tailwind `dark:` variants from specs/design.md)
- Follow specs/design.md for colors, spacing, typography, components
- Use Lucide icons where appropriate
- Don't reinvent the wheel for editors/highlighting — use CodeMirror 6 via JS interop
- One component per file (split aggressively, follow specs/project.md architecture)
- **Never silently render a non-NULL value as NULL** because the type isn't supported. Backend must return the value tagged with its Postgres type, frontend must dispatch to the correct specialized editor. See "Postgres Type Support" section in specs/project.md for the full type mapping.

## Learnings
- Tauri v2 `app` section has no `title` field — title goes in `app.windows[].title`
- `tauri::generate_context!()` requires `icons/icon.png` in `src-tauri/`
- Leptos 0.8 CSR: `leptos::mount::mount_to_body(App)` works directly
- Frontend = root crate (target wasm32), Backend = `src-tauri/` (target native)
- Trunk hook `pre_build` for generating Tailwind CSS before WASM build
- `url` crate for parsing PostgreSQL connection strings (host, port, user, dbname)
- pg_restore exit code 1 with warnings = non-fatal (treat as success with warnings)
- Tauri command args must be wrapped: frontend sends `{ info: {...} }` not just `{...}` for a param named `info`
