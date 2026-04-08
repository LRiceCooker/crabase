# Plan de tâches

## En cours
- [x] Initialiser le projet Tauri v2 avec Leptos (CSR) + Tailwind + DaisyUI

## Backlog

### Infrastructure
- [x] Configurer sqlx avec le pool de connexion PostgreSQL
- [x] Créer la Tauri command `connect_db(connection_string)` qui valide et ouvre la connexion
- [x] Créer la Tauri command `get_connection_info()` qui retourne host, port, user, dbname
- [x] Créer la Tauri command `list_tables()` qui retourne les noms des tables (public schema)
- [x] Créer la Tauri command `disconnect_db()` pour fermer la connexion

### Écran de connexion
- [x] UI : champ connection string + bouton "Se connecter"
- [x] Intégration : appel à `connect_db`, gestion erreur, navigation vers écran principal

### Écran principal
- [x] Layout : header (infos connexion éditables) + panneau droit (tables) + zone centrale vide
- [x] Header : afficher les infos de connexion, permettre l'édition et la reconnexion
- [x] Panneau droit : afficher la liste des tables via `list_tables()`

### Command Palette
- [x] UI : overlay style VS Code (Cmd+Shift+P), input + liste de résultats
- [x] Fuzzy search sur les noms de commandes
- [x] Commande "Restore Backup" : ouvre le panneau restore

### Restore Backup
- [x] UI : panneau avec sélecteur de fichier (.tar.gz) + bouton "Lancer le restore"
- [x] Backend : Tauri command `restore_backup(file_path)` — décompresse .tar.gz, trouve le .pgsql, lance pg_restore
- [ ] Streaming des logs pg_restore en temps réel vers le frontend (Tauri events)
- [ ] Afficher succès/échec à la fin du restore

## Terminé
<!-- Les tâches complétées sont déplacées ici -->
