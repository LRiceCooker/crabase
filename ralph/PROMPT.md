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
- Use Tauri v2 APIs (not v1)
- sqlx for DB, pg_restore CLI for restores
- Frontend Leptos CSR only
- **Follow ralph/specs/design.md strictly** for all UI work — colors, spacing, typography, component patterns
- **One component per file** — split code into small focused files per the architecture in ralph/specs/project.md
- All UI text in English
- Use Lucide icons where appropriate
