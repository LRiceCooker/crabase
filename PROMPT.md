# Instructions

Tu es un agent de développement travaillant sur **crabase**, un client desktop PostgreSQL en Rust + Tauri v2 + Leptos.

## Contexte — lis ces fichiers en premier
- `CLAUDE.md` : contexte agent, stack, règles, apprentissages
- `fix_plan.md` : liste des tâches prioritisées
- `specs/project.md` : spécifications complètes du projet (écrans, UX, contraintes)

## Stack
- Backend : Rust, Tauri v2, sqlx (PostgreSQL)
- Frontend : Leptos (CSR, WASM), Tailwind CSS, DaisyUI
- Restore : pg_restore (subprocess CLI)
- Toolchain : mise (rust, cargo, node)

## Workflow par itération
1. Lis `fix_plan.md` et prends la **première tâche `[ ]`** (non complétée)
2. Cherche dans le code existant pour éviter les doublons
3. Implémente la tâche
4. Lance `cargo check && cargo test` pour valider
5. Si les tests passent → coche la tâche `[x]` dans `fix_plan.md`, commit
6. Si les tests échouent → corrige et relance (ne pas abandonner)
7. Note les décisions/apprentissages dans `CLAUDE.md`

## Règles strictes
- **UNE SEULE tâche par itération** — ne jamais en prendre plusieurs
- Ne jamais ignorer les erreurs de compilation
- Ne jamais skip les tests
- Toujours committer le code fonctionnel avant de terminer
- Utiliser Tauri v2 APIs (pas v1)
- sqlx pour la DB, pg_restore CLI pour les restores
- Frontend Leptos en CSR uniquement
