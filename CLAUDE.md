# Agent Context

## Stack
- **Backend** : Rust + Tauri v2 + sqlx (PostgreSQL)
- **Frontend** : Leptos (CSR) + Tailwind CSS + DaisyUI
- **Toolchain** : mise (rust, cargo, node LTS)
- **Restore** : pg_restore (CLI subprocess)

## Architecture
- Le backend expose des Tauri commands que le frontend appelle via `invoke`
- sqlx pour toutes les interactions DB (connection, queries)
- pg_restore en subprocess pour les restores de backup
- Le frontend est une SPA Leptos compilée en WASM (CSR)

## Règles
- Une seule tâche par itération de la boucle
- Toujours lancer `cargo check && cargo test` après chaque modification
- Chercher le code existant avant d'implémenter (éviter les doublons)
- Committer après chaque feature complétée et testée
- Documenter les décisions et apprentissages ci-dessous
- Ne jamais bundler pg_restore — il doit être installé sur le système
- Utiliser Tauri v2 APIs exclusivement (pas v1)

## Apprentissages
- Tauri v2 `app` section n'a pas de champ `title` — le titre est dans `app.windows[].title`
- `tauri::generate_context!()` exige un fichier `icons/icon.png` dans `src-tauri/`
- Leptos 0.8 CSR : `leptos::mount::mount_to_body(App)` fonctionne directement
- Frontend = crate racine (target wasm32), Backend = `src-tauri/` (target native)
- Trunk hook `pre_build` pour générer le CSS Tailwind avant le build WASM
