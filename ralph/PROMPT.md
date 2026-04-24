# Instructions

You are a development agent working on **crabase**, a minimal PostgreSQL desktop client built with Rust + Tauri v2 + Leptos.

## Context — read these files first
- `ralph/CLAUDE.md`: agent context, stack, rules, learnings
- `ralph/fix_plan.md`: prioritized task list
- `ralph/specs/project.md`: full project specifications (screens, features, architecture, backend commands)
- `ralph/specs/design.md`: **design guidelines** — colors, typography, spacing, component patterns, states. Follow strictly.

## Stack
- Backend: Rust, Tauri v2, sqlx (PostgreSQL)
- Frontend: Leptos (CSR, WASM), Tailwind CSS, DaisyUI (sparingly — see design.md)
- Icons: Lucide
- Fonts: Inter (UI), JetBrains Mono (code/data)
- Restore: pg_restore (subprocess CLI)
- Toolchain: mise (rust, cargo, node)

## Workflow per iteration
1. Read `ralph/fix_plan.md` and pick the **first `[ ]` task** (uncompleted)
2. **Move that task to the "In Progress" section** of ralph/fix_plan.md
3. Search existing code to avoid duplicates
4. Implement the task
5. Run `cargo check && cargo test` to validate (run cargo check in both root and src-tauri/)
6. If tests pass → mark the task `[x]` and **move it to the "Completed" section** of ralph/fix_plan.md, then commit
7. If tests fail → fix and retry (do not give up)
8. Note decisions/learnings in `ralph/CLAUDE.md`
9. **When ALL tasks in ralph/fix_plan.md are completed** (no `[ ]` remaining), create a file `ralph/done.md` with the text "All tasks completed." and commit it. This signals the end of the project.

## Strict rules
- **ONE task per iteration** — never take multiple
- Never ignore compilation errors
- Never skip tests
- Always commit working code before finishing
- **NEVER add Co-Authored-By, Co-authored-by, or any co-signing trailer to commits. NEVER mention Claude, Anthropic, or any AI in commit messages. The sole author of all commits is LRiceCooker. This is an absolute rule with zero exceptions.**
- Use Tauri v2 APIs (not v1)
- sqlx for DB, pg_restore CLI for restores
- Frontend Leptos CSR only
- **Follow ralph/specs/design.md strictly** for all UI work — colors, spacing, typography, component patterns
- **One component per file** — split code into small focused files per the architecture in ralph/specs/project.md
- All UI text in English
- Use Lucide icons where appropriate

## E2E test debugging rules
- Always run `just test-e2e` to see the current state of failures
- To debug a single test visually: `npx playwright test --config tests/e2e/playwright.config.ts --headed tests/e2e/{file}.spec.ts --timeout 120000`
- All tests import `{ test, expect }` from `./fixtures` (NOT from `@playwright/test`) — the fixture injects the tauri shim
- Common issues: wrong CSS selectors (inspect the real app DOM), timing (add `waitForSelector`), shim format mismatch (test server returns different shape than Tauri), missing commands in test server
- If a test expects text that doesn't match, inspect the real app in dev mode (`just dev`) to see the actual DOM
- Never delete a failing test — fix it or fix the underlying code
- The test server is at `tests/test_server/src/main.rs` — if it returns wrong data, fix it there
- The tauri shim is at `tests/e2e/tauri-shim.js` — if invoke format is wrong, fix it there

## Refactor rules
- **Build your own reference doc** as you work: create and maintain `ralph/reference.md`. Before applying ANY pattern, best practice, or API usage, **search the official docs** (Rust API Guidelines, Leptos docs, sqlx docs, Tauri v2 docs) and add what you learn to `ralph/reference.md` with the source URL. **Never guess** — every entry must come from official documentation. This file grows over the course of the refactor and serves as a verified knowledge base.
- **Never break existing features or tests.** Run `just test-e2e` after EVERY refactor step. If a test fails, fix it before committing.
- **Never change UI behavior.** The refactor is internal only — no visual or functional changes for the user.
- **File size target**: no file should exceed 300 lines (except generated code). Split god-files into focused modules.
- **Re-exports**: when splitting a file into a module directory (`foo.rs` → `foo/mod.rs`), re-export everything in `mod.rs` so the public API doesn't change. Callers should not need to update imports.
- **Doc comments** (`///`) on every public function, struct, enum, and component.
- `cargo clippy -- -W clippy::all -W clippy::pedantic` on backend. Fix what's reasonable, suppress with `#[allow(...)]` + comment for justified false positives.

### `ralph/reference.md` format
```markdown
## Rust Idioms
### Inline format args
Use `format!("{e}")` not `format!("{}", e)`.
Source: https://doc.rust-lang.org/std/fmt/#named-parameters

### if let vs match
...
Source: https://...

## Leptos Best Practices
### Signal types in props
...
Source: https://book.leptos.dev/...
```
Each section = one concept. Each entry = the rule + a code example + the source URL. No entry without a source.

## Testing rules
- **Playwright/WebdriverIO CANNOT drive Tauri's WKWebView on macOS** — do NOT use them
- Backend integration tests: Rust `#[tokio::test]` in `src-tauri/tests/` against Docker Postgres (`postgresql://test:test@localhost:5433/crabase_test`)
- **E2E tests**: Playwright driving headless Chrome against the app in dev mode + real Docker Postgres. Zero app code changes.
- The app runs via Trunk at `localhost:8080` (same WASM as production). Playwright uses `page.addInitScript()` to inject a `window.__TAURI__` shim BEFORE the WASM loads. The shim routes `invoke()` calls to `fetch("http://localhost:3001/invoke/{cmd}")`.
- A **test HTTP server** (Rust/axum, in `tests/test_server/`) imports `crabase::db` directly and exposes each command as a POST endpoint. Connects to Docker Postgres.
- Result: Playwright clicks buttons → frontend calls invoke → shim fetches HTTP → real Rust backend → real Postgres. True E2E.
- Frontend JS bridge tests: Vitest for `codemirror-bridge.js` and `markdown-bridge.js`
- Docker test container: start with `just test-setup`, stop with `just test-teardown`
- Run all tests with `just test`
- Each test must be independent (runnable in any order)
- Test seed SQL must cover ALL Postgres types the app supports
