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

## Testing
- **Playwright/WebdriverIO cannot drive Tauri's WKWebView on macOS** — do NOT attempt to use them for E2E testing
- Backend integration tests: Rust `#[tokio::test]` in `src-tauri/tests/` against Docker Postgres on `postgresql://test:test@localhost:5433/crabase_test`
- **E2E tests**: Playwright + headless Chrome + real Docker Postgres. The app runs via Trunk (localhost:8080) with identical WASM to production. Playwright injects a `window.__TAURI__` shim via `addInitScript()` that routes `invoke()` to a test HTTP server (Rust/axum on port 3001). The test server imports `crabase::db` and talks to Docker Postgres. Zero app code changes needed.
- Frontend JS bridge tests: Vitest for `codemirror-bridge.js`, `markdown-bridge.js`
- Docker container must be running before backend tests: `just test-setup` or `just test`
- Tests must be independent — each test must work regardless of execution order
- The `tests/seed.sql` must cover ALL Postgres types the app supports

## Known issues to fix during audit
- `std::sync::Mutex` wrapping async `PgPool` — should be `tokio::sync::RwLock` or leverage PgPool's internal Arc
- `closure.forget()` on event listeners in components that re-render — memory leak + duplicate handlers
- `Effect::new` writing signals that trigger other Effects — causes `RefCell::borrow_mut` panics (Out of bounds memory access)
- `main_layout.rs` and `table_view.rs` are god-files — too many signals and responsibilities
- Multiple `unwrap()` calls in Tauri commands that could panic in production
- `inner_html` rendering of markdown without XSS sanitization (DOMPurify needed)
- Dead code: unused functions in `restore.rs`, unused imports across the codebase
- Redundant `.clone()` on PgPool (it's Arc internally, cloning is cheap but unnecessary)
- No cleanup of Tauri event listeners (`listen_chat_response`, `listen_chat_done`) — accumulate on repeated messages

## Learnings
- Tauri v2 `app` section has no `title` field — title goes in `app.windows[].title`
- `tauri::generate_context!()` requires `icons/icon.png` in `src-tauri/`
- Leptos 0.8 CSR: `leptos::mount::mount_to_body(App)` works directly
- Frontend = root crate (target wasm32), Backend = `src-tauri/` (target native)
- Trunk hook `pre_build` for generating Tailwind CSS before WASM build
- `url` crate for parsing PostgreSQL connection strings (host, port, user, dbname)
- pg_restore exit code 1 with warnings = non-fatal (treat as success with warnings)
- Tauri command args must be wrapped: frontend sends `{ info: {...} }` not just `{...}` for a param named `info`
