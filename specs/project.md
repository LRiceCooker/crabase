# Spécifications du projet

## Description
**crabase** est un client desktop PostgreSQL minimaliste.
Il permet de se connecter à une base Postgres via une connection string,
d'explorer les tables, et de restaurer des backups (.tar.gz contenant un .pgsql).

## Stack technique
- **Backend** : Rust, Tauri v2, sqlx (PostgreSQL)
- **Frontend** : Leptos (CSR), Tailwind CSS + DaisyUI
- **Toolchain** : mise (rust, cargo, node LTS)
- **Restore** : pg_restore en subprocess CLI

## Écrans et navigation

### 1. Écran de connexion (vue initiale)
- Champ unique pour la **connection string** PostgreSQL
  - Format : `postgresql://user:password@host:port/dbname`
- Bouton "Se connecter"
- En cas d'erreur de connexion, afficher le message d'erreur sous le champ
- Une fois la connexion validée → navigation vers l'écran principal

### 2. Écran principal (post-connexion)
- **Header** : informations de connexion (host, port, user, dbname) affichées et éditables
  - Permet de modifier et reconnecter sans revenir à l'écran précédent
- **Panneau droit** : liste des tables (noms seulement, liste plate)
  - Récupérées via `information_schema.tables` (schema = 'public')
  - Un clic sur une table ne fait rien pour l'instant (hors scope)
- **Zone centrale** : vide pour l'instant (futur : contenu des tables)

### 3. Command Palette (Cmd+Shift+P)
- **Style VS Code** : input centré en haut de l'écran, overlay sombre, liste de résultats en dessous
- **Fuzzy search** sur les noms de commandes
- Commandes disponibles :
  1. **"Restore Backup"** : ouvre le panneau de restore

### 4. Panneau Restore Backup
- Sélecteur de fichier natif (Tauri dialog) filtré sur `.tar.gz`
- Le `.tar.gz` contient un fichier `.pgsql` **à la racine**
- Bouton "Lancer le restore"
- Restore via `pg_restore` CLI sur la **même DB connectée**
- **Logs en live** : afficher la sortie stdout/stderr de pg_restore en temps réel
- Indicateur succès/échec à la fin

## Contraintes techniques
- Utiliser les API Tauri v2 (pas v1)
- Gérer les erreurs backend avec `thiserror` ou `anyhow`
- Tests unitaires obligatoires pour la logique métier (commandes Tauri, parsing, etc.)
- Le frontend utilise les Tauri commands (invoke) pour communiquer avec le backend
- pg_restore doit être présent sur le système (ne pas le bundler)

## Hors scope (pour l'instant)
- Affichage du contenu des tables
- Support d'autres SGBD que PostgreSQL
- Édition/exécution de requêtes SQL
- Export de données
