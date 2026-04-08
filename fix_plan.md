# Plan de tâches

## En cours
- [x] Initialiser le projet Tauri v2 avec Leptos (CSR) + Tailwind + DaisyUI

## Backlog

### Infrastructure
- [x] Configurer sqlx avec le pool de connexion PostgreSQL
- [x] Créer la Tauri command `connect_db(connection_string)` qui valide et ouvre la connexion
- [ ] Créer la Tauri command `get_connection_info()` qui retourne host, port, user, dbname
- [ ] Créer la Tauri command `list_tables()` qui retourne les noms des tables (public schema)
- [ ] Créer la Tauri command `disconnect_db()` pour fermer la connexion

### Écran de connexion
- [ ] UI : champ connection string + bouton "Se connecter"
- [ ] Intégration : appel à `connect_db`, gestion erreur, navigation vers écran principal

### Écran principal
- [ ] Layout : header (infos connexion éditables) + panneau droit (tables) + zone centrale vide
- [ ] Header : afficher les infos de connexion, permettre l'édition et la reconnexion
- [ ] Panneau droit : afficher la liste des tables via `list_tables()`

### Command Palette
- [ ] UI : overlay style VS Code (Cmd+Shift+P), input + liste de résultats
- [ ] Fuzzy search sur les noms de commandes
- [ ] Commande "Restore Backup" : ouvre le panneau restore

### Restore Backup
- [ ] UI : panneau avec sélecteur de fichier (.tar.gz) + bouton "Lancer le restore"
- [ ] Backend : Tauri command `restore_backup(file_path)` — décompresse .tar.gz, trouve le .pgsql, lance pg_restore
- [ ] Streaming des logs pg_restore en temps réel vers le frontend (Tauri events)
- [ ] Afficher succès/échec à la fin du restore

## Terminé
<!-- Les tâches complétées sont déplacées ici -->
