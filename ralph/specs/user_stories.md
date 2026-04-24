# User Stories — crabase

## 1. Connexion a une base de donnees

### 1.1 Se connecter via une connection string
**En tant qu'** utilisateur,
**je veux** coller une URL PostgreSQL complète (`postgresql://user:pass@host:port/db`),
**afin de** me connecter rapidement sans remplir chaque champ manuellement.

- Le champ accepte une connection string PostgreSQL standard
- Cliquer "Next" parse la string et pré-remplit le formulaire de connexion
- Les champs extraits : host, port, user, password, database

### 1.2 Se connecter via le formulaire manuel
**En tant qu'** utilisateur,
**je veux** saisir manuellement les informations de connexion (host, port, user, password, database),
**afin de** me connecter quand je n'ai pas de connection string sous la main.

- Champs : host, port, username, password, database name
- Toggle SSL/TLS (require/disable)
- Sélecteur de schéma (liste récupérée du serveur après parsing)
- Bouton "Connect" qui valide et établit la connexion

### 1.3 Sauvegarder une connexion
**En tant qu'** utilisateur,
**je veux** sauvegarder mes informations de connexion avec un nom,
**afin de** pouvoir me reconnecter rapidement la prochaine fois.

- Checkbox "Save connection" + champ nom
- Comportement upsert : si le nom existe déjà, il est écrasé
- Les connexions sont persistées dans `~/.crabase/saved_connections.json`

### 1.4 Utiliser une connexion sauvegardée
**En tant qu'** utilisateur,
**je veux** voir la liste de mes connexions sauvegardées et cliquer dessus pour pré-remplir le formulaire,
**afin de** ne pas avoir à retaper mes informations à chaque fois.

- Liste affichée sur l'écran de connexion (nom, host:port/dbname)
- Cliquer sur une connexion auto-remplit le formulaire
- Bouton de suppression visible au survol

### 1.5 Supprimer une connexion sauvegardée
**En tant qu'** utilisateur,
**je veux** supprimer une connexion sauvegardée,
**afin de** nettoyer ma liste de connexions obsolètes.

- Icône de suppression visible au survol de la connexion
- Suppression immédiate sans confirmation

### 1.6 Se déconnecter
**En tant qu'** utilisateur,
**je veux** me déconnecter de la base de données via le bouton "Disconnect" dans le header,
**afin de** retourner à l'écran de connexion et éventuellement me connecter à une autre base.

- Le bouton "Disconnect" ferme la connexion DB
- L'app retourne à l'écran de connexion

### 1.7 Changer de schéma
**En tant qu'** utilisateur,
**je veux** changer de schéma PostgreSQL via le sélecteur dans le header,
**afin de** explorer les tables d'un autre schéma sans me déconnecter.

- Liste des schémas récupérée dynamiquement depuis le serveur
- Le changement est instantané (pas de reconnexion)
- La liste des tables se met à jour en conséquence
- L'autocomplétion SQL s'adapte au schéma actif

---

## 2. Navigation et exploration

### 2.1 Voir la liste des tables
**En tant qu'** utilisateur,
**je veux** voir toutes les tables du schéma sélectionné dans la sidebar,
**afin de** savoir quelles tables existent et pouvoir les ouvrir.

- Section "Tables" dans la sidebar (toujours visible)
- Liste scrollable indépendamment
- Cliquer sur une table ouvre un nouvel onglet

### 2.2 Ouvrir une table dans un onglet
**En tant qu'** utilisateur,
**je veux** cliquer sur une table dans la sidebar pour l'ouvrir dans un onglet,
**afin de** visualiser et manipuler ses données.

- Chaque table s'ouvre dans son propre onglet
- L'onglet affiche le nom de la table
- Si la table est déjà ouverte, focus l'onglet existant

### 2.3 Naviguer entre les onglets
**En tant qu'** utilisateur,
**je veux** switcher entre mes onglets ouverts (tables, éditeurs SQL, settings),
**afin de** travailler sur plusieurs tables ou requêtes en parallèle.

- Barre d'onglets horizontale
- Cliquer sur un onglet l'active
- Onglet actif : bordure inférieure indigo
- Bouton de fermeture visible au survol

### 2.4 Fermer un onglet
**En tant qu'** utilisateur,
**je veux** fermer un onglet via le bouton close,
**afin de** garder mon workspace propre.

- Bouton close (×) visible au survol de l'onglet
- La fermeture ne demande pas confirmation (même avec des changements non sauvegardés dans le table view)

### 2.5 Rechercher une table ou une query (Cmd+P)
**En tant qu'** utilisateur,
**je veux** rechercher rapidement une table ou une query sauvegardée par nom,
**afin de** naviguer plus vite qu'en scrollant la sidebar.

- Raccourci : `Cmd+P`
- Recherche fuzzy sur les noms de tables ET les queries sauvegardées
- Résultats groupés par type ("Tables", "Queries")
- Navigation au clavier (flèches + Enter)
- Sélectionner ouvre dans un nouvel onglet (ou focus l'existant)

### 2.6 Ouvrir la palette de commandes (Cmd+Shift+P)
**En tant qu'** utilisateur,
**je veux** accéder rapidement aux actions de l'app via une palette de commandes,
**afin de** ne pas avoir à chercher dans les menus.

- Raccourci : `Cmd+Shift+P`
- Recherche fuzzy sur les commandes
- Commandes disponibles : "New SQL Editor", "Restore Backup", "Settings"
- Navigation au clavier (flèches + Enter + Escape)

### 2.7 Ouvrir une nouvelle fenêtre (Cmd+Shift+N)
**En tant qu'** utilisateur,
**je veux** ouvrir une nouvelle fenêtre indépendante de l'app,
**afin de** travailler sur plusieurs connexions en parallèle.

- Raccourci : `Cmd+Shift+N`
- Nouvelle fenêtre = instance indépendante, démarre à l'écran de connexion
- Partage les fichiers de configuration (connexions, settings, queries)
- Pas de synchronisation temps réel entre fenêtres

---

## 3. Visualisation des données

### 3.1 Voir les données d'une table
**En tant qu'** utilisateur,
**je veux** voir les données d'une table sous forme de grille avec colonnes et lignes,
**afin de** explorer le contenu de ma base de données.

- Affichage en grille avec headers de colonnes (nom + type)
- Headers sticky au scroll vertical
- Colonne index (numéro de ligne) sticky au scroll horizontal
- Cellules en police monospace, tronquées si trop longues
- Valeurs NULL affichées en gris italique "NULL"

### 3.2 Paginer les données
**En tant qu'** utilisateur,
**je veux** naviguer entre les pages de résultats d'une table,
**afin de** parcourir des tables avec beaucoup de lignes sans tout charger.

- Contrôles : boutons "Previous" / "Next"
- Choix du nombre de lignes par page : 25, 50, 100
- Affichage : "Page X of Y", "N rows total"
- Numérotation globale des lignes (page 2 avec 50/page commence à 51)

### 3.3 Trier les données
**En tant qu'** utilisateur,
**je veux** trier les données par une ou plusieurs colonnes,
**afin de** trouver rapidement les données qui m'intéressent.

- Cliquer sur un header de colonne cycle : asc -> desc -> aucun
- Chips de tri dans la barre de filtre (colonne + direction)
- Tri multi-colonnes possible (chaînage)
- Tri par défaut intelligent :
  1. `created_at` desc si la colonne existe
  2. Sinon clé primaire asc
  3. Sinon première colonne

### 3.4 Filtrer les données
**En tant qu'** utilisateur,
**je veux** ajouter des filtres sur les colonnes,
**afin de** restreindre les données affichées à ce qui m'intéresse.

- Bouton "+ Filter" pour ajouter un filtre
- Chaque filtre : sélection de colonne + opérateur + valeur
- Opérateurs : `=`, `!=`, `<`, `>`, `<=`, `>=`, `LIKE`, `NOT LIKE`, `IN`, `NOT IN`, `IS NULL`, `IS NOT NULL`, `contains`, `starts with`, `ends with`
- Combinateur logique entre filtres : `AND`, `OR`, `XOR`
- Suppression individuelle de chaque filtre

### 3.5 Rechercher dans les données visibles (Cmd+F)
**En tant qu'** utilisateur,
**je veux** rechercher un texte dans les cellules visibles de la table,
**afin de** localiser rapidement une valeur spécifique.

- Raccourci : `Cmd+F`
- Barre de recherche flottante en haut à droite de la table
- Recherche fuzzy sur toutes les cellules visibles
- Compteur de correspondances (X/Y)
- Navigation : boutons Prev/Next ou Enter/Shift+Enter
- Surlignage des correspondances en temps réel

### 3.6 Rafraîchir les données
**En tant qu'** utilisateur,
**je veux** rafraîchir les données de la table,
**afin de** voir les changements effectués par d'autres processus ou utilisateurs.

- Bouton "Refresh" dans la toolbar de la table
- Recharge les données depuis la base

### 3.7 Affichage correct de tous les types PostgreSQL
**En tant qu'** utilisateur,
**je veux** que chaque type PostgreSQL soit affiché correctement dans la grille,
**afin de** ne jamais voir de données corrompues, de JSON brut, ou de NULL pour des valeurs existantes.

- Timestamps formatés en `YYYY-MM-DD HH:MM:SS`
- UUID affichés comme strings
- JSON/JSONB : aperçu tronqué
- Arrays : aperçu avec 3 premiers éléments + "..."
- Enums : valeur label (pas le nom qualifié du schéma)
- Booléens : icône checkmark
- Bytea : aperçu hex
- Types inconnus : texte brut en lecture seule avec tooltip

---

## 4. Edition des données

### 4.1 Modifier une cellule
**En tant qu'** utilisateur,
**je veux** cliquer sur une cellule pour éditer sa valeur avec un éditeur adapté au type,
**afin de** modifier les données directement dans la grille.

- Cliquer sur une cellule active l'éditeur inline
- L'éditeur est spécialisé par type PostgreSQL (voir 4.2-4.13)
- Enter pour valider, Escape pour annuler
- La cellule modifiée est surlignée en ambre
- La ligne modifiée a une bordure gauche ambre

### 4.2 Editer du texte (text, varchar, char)
**En tant qu'** utilisateur,
**je veux** un champ de saisie texte pour les colonnes textuelles,
**afin de** modifier les valeurs string.

- Input texte simple
- Respect de maxLength pour varchar(n)
- Auto-focus et sélection du contenu à l'ouverture

### 4.3 Editer un nombre (integer, bigint, numeric, real, etc.)
**En tant qu'** utilisateur,
**je veux** un champ numérique avec validation pour les colonnes numériques,
**afin de** ne saisir que des valeurs numériques valides.

- Input numérique avec validation
- Respect de la précision/échelle pour numeric

### 4.4 Editer un booléen
**En tant qu'** utilisateur,
**je veux** un sélecteur tri-état pour les colonnes boolean,
**afin de** choisir entre TRUE, FALSE, ou NULL.

- Select avec 3 options : NULL, TRUE, FALSE

### 4.5 Editer un enum
**En tant qu'** utilisateur,
**je veux** un menu déroulant avec les valeurs autorisées pour les colonnes enum,
**afin de** ne choisir que des valeurs valides.

- Select dropdown avec les valeurs de l'enum
- Option NULL si la colonne est nullable
- Fonctionne avec les enums de tous les schémas (pas seulement public)

### 4.6 Editer une date / timestamp
**En tant qu'** utilisateur,
**je veux** un date picker natif pour les colonnes date et timestamp,
**afin de** saisir des dates facilement sans risque de format invalide.

- `<input type="date">` pour les colonnes date
- `<input type="time">` pour les colonnes time
- `<input type="datetime-local">` pour les colonnes timestamp
- Le format de sortie est compatible PostgreSQL

### 4.7 Editer un UUID
**En tant qu'** utilisateur,
**je veux** un champ texte avec validation UUID et un bouton pour générer un nouvel UUID,
**afin de** saisir ou générer des identifiants uniques facilement.

- Input texte avec validation UUID
- Bouton "Generate" pour créer un UUID aléatoire

### 4.8 Editer du JSON/JSONB
**En tant qu'** utilisateur,
**je veux** un éditeur modal avec coloration syntaxique pour les colonnes JSON,
**afin de** modifier du JSON structuré confortablement.

- Modale avec éditeur CodeMirror (coloration JSON)
- Scrollable pour les grands documents
- Aussi accessible en lecture seule dans les résultats SQL (clic pour ouvrir)

### 4.9 Editer un tableau (array)
**En tant qu'** utilisateur,
**je veux** un éditeur modal pour les colonnes de type array,
**afin de** ajouter, supprimer et modifier les éléments du tableau.

- Modale avec liste d'éléments
- Bouton ajouter / supprimer par élément
- Editeur par élément adapté au type de base

### 4.10 Editer du XML
**En tant qu'** utilisateur,
**je veux** un éditeur modal avec coloration syntaxique pour les colonnes XML,
**afin de** modifier du XML confortablement.

- Modale avec éditeur CodeMirror (coloration XML)

### 4.11 Editer des types spéciaux (interval, inet, bit, bytea, range)
**En tant qu'** utilisateur,
**je veux** des éditeurs adaptés pour les types PostgreSQL spéciaux,
**afin de** pouvoir modifier ces valeurs avec validation.

- **Interval** : input texte avec validation syntaxe Postgres
- **Inet/CIDR** : input texte avec validation IP
- **Bit** : input texte restreint aux caractères 0/1
- **Bytea** : input hex
- **Range** (int4range, numrange, etc.) : deux inputs (borne inférieure/supérieure) + toggle inclusivité

### 4.12 Mettre une valeur a NULL
**En tant qu'** utilisateur,
**je veux** pouvoir mettre une cellule nullable à NULL,
**afin de** effacer une valeur.

- Bouton "×" (ou "Set NULL") dans chaque éditeur pour les colonnes nullables
- La cellule affiche ensuite "NULL" en gris italique

### 4.13 Colonnes en lecture seule
**En tant qu'** utilisateur,
**je veux** que les clés primaires et les colonnes auto-increment soient en lecture seule sur les lignes existantes,
**afin de** ne pas corrompre les identifiants.

- Les colonnes PK sont non-éditables sur les lignes existantes
- Les colonnes serial/bigserial/identity sont non-éditables sur les lignes existantes
- Sur les nouvelles lignes, les champs auto-increment sont laissés vides (valeur générée par la DB)

---

## 5. Manipulation des lignes

### 5.1 Ajouter une nouvelle ligne
**En tant qu'** utilisateur,
**je veux** ajouter une nouvelle ligne vide à la table,
**afin d'** insérer de nouvelles données.

- Bouton "Add row" dans la toolbar
- La nouvelle ligne apparaît avec un fond vert (emerald)
- Les valeurs par défaut de la DB sont pré-remplies
- Les champs auto-increment sont laissés vides

### 5.2 Dupliquer des lignes
**En tant qu'** utilisateur,
**je veux** dupliquer une ou plusieurs lignes sélectionnées,
**afin de** créer rapidement des entrées similaires.

- Clic droit > "Duplicate" sur les lignes sélectionnées
- Les copies apparaissent comme de nouvelles lignes (fond vert)
- Les valeurs PK/auto-increment sont retirées

### 5.3 Supprimer des lignes
**En tant qu'** utilisateur,
**je veux** marquer des lignes pour suppression,
**afin de** les retirer de la table au prochain save.

- Clic droit > "Delete" sur les lignes sélectionnées
- Les lignes marquées apparaissent en rouge avec texte barré et opacité réduite
- La suppression n'est effective qu'au moment du save

### 5.4 Sélectionner des lignes
**En tant qu'** utilisateur,
**je veux** sélectionner une ou plusieurs lignes,
**afin de** pouvoir effectuer des actions groupées (supprimer, dupliquer, copier).

- Clic sur l'index de ligne : sélection simple
- `Cmd+Clic` : ajouter/retirer de la sélection (multi-select)
- `Shift+Clic` : sélection de plage (range select)
- Lignes sélectionnées : fond indigo léger

### 5.5 Copier des lignes en JSON
**En tant qu'** utilisateur,
**je veux** copier les lignes sélectionnées au format JSON dans le presse-papier,
**afin de** les utiliser dans d'autres outils ou scripts.

- Clic droit > "Copy as JSON"
- JSON formaté (pretty-printed) copié dans le clipboard

### 5.6 Copier des lignes en SQL INSERT
**En tant qu'** utilisateur,
**je veux** copier les lignes sélectionnées sous forme d'instructions SQL INSERT,
**afin de** pouvoir les rejouer ailleurs ou les partager.

- Clic droit > "Copy as SQL INSERT"
- Instructions INSERT complètes copiées dans le clipboard

---

## 6. Sauvegarde des modifications

### 6.1 Voir les changements en attente
**En tant qu'** utilisateur,
**je veux** voir combien de modifications sont en attente,
**afin de** savoir ce qui sera persisté au prochain save.

- Barre flottante en bas : "X changes pending"
- Compteur agrégé : lignes modifiées + lignes ajoutées + lignes supprimées
- La barre n'apparaît que quand il y a des changements

### 6.2 Sauvegarder les modifications (Cmd+S)
**En tant qu'** utilisateur,
**je veux** sauvegarder toutes les modifications en attente en un seul clic,
**afin de** persister mes changements dans la base de données.

- Raccourci : `Cmd+S` (quand l'onglet actif est une table en dirty state)
- Bouton "Save changes" dans la barre flottante
- Toutes les modifications (inserts, updates, deletes) exécutées dans une seule transaction
- Tout ou rien : si une opération échoue, aucune n'est appliquée

### 6.3 Annuler les modifications
**En tant qu'** utilisateur,
**je veux** annuler toutes les modifications en attente,
**afin de** revenir à l'état original des données.

- Bouton "Discard" dans la barre flottante
- Remet toutes les cellules, lignes ajoutées et lignes supprimées à leur état initial

---

## 7. Editeur SQL

### 7.1 Ouvrir un nouvel éditeur SQL
**En tant qu'** utilisateur,
**je veux** créer un nouvel onglet d'éditeur SQL,
**afin d'** écrire et exécuter des requêtes.

- Bouton "+" dans le header
- Commande palette > "New SQL Editor"
- Nom par défaut : "Untitled-1", "Untitled-2", etc.
- Auto-focus sur l'éditeur à l'ouverture

### 7.2 Ecrire du SQL avec coloration syntaxique
**En tant qu'** utilisateur,
**je veux** un éditeur SQL avec coloration syntaxique, numéros de ligne et raccourcis VS Code,
**afin d'** écrire des requêtes efficacement dans un environnement familier.

- Coloration syntaxique SQL (CodeMirror 6)
- Numéros de ligne
- Raccourcis VS Code complets :
  - Undo/Redo, Find/Replace, Select next/all occurrences
  - Toggle comment (ligne/bloc), Copy line down, Move line up/down
  - Delete line, Go to line, Indent/Outdent
  - Et plus

### 7.3 Autocomplétion SQL
**En tant qu'** utilisateur,
**je veux** l'autocomplétion sur les mots-clés SQL, noms de tables et noms de colonnes,
**afin d'** écrire des requêtes plus vite et sans erreurs de frappe.

- Mots-clés SQL standards
- Tables du schéma courant
- Colonnes des tables référencées
- Préfixe de schéma automatique quand on n'est pas sur `public`

### 7.4 Exécuter des requêtes SQL
**En tant qu'** utilisateur,
**je veux** exécuter le contenu entier de l'éditeur (une ou plusieurs requêtes),
**afin de** voir les résultats ou appliquer des modifications à la base.

- Bouton "Run" (vert, icône play)
- Exécute tout le contenu de l'éditeur (multi-statement)
- Support : SELECT, INSERT, UPDATE, DELETE, DDL, transactions, BEGIN/COMMIT

### 7.5 Voir les résultats de requêtes
**En tant qu'** utilisateur,
**je veux** voir les résultats de chaque requête exécutée,
**afin de** vérifier le retour de mes requêtes.

- Panneau résultats sous l'éditeur (split redimensionnable avec handle draggable)
- SELECT : table de données en lecture seule (même style que table view)
- INSERT/UPDATE/DELETE : console avec nombre de lignes affectées
- Erreur : message rouge avec index du statement en erreur
- Sélecteur de statement sous le tableau pour naviguer entre les résultats

### 7.6 Voir les données complexes dans les résultats SQL
**En tant qu'** utilisateur,
**je veux** pouvoir cliquer sur des cellules JSON, array ou XML dans les résultats SQL,
**afin de** voir le contenu complet dans un viewer modal en lecture seule.

- Les cellules JSON, array, XML sont cliquables dans les résultats
- Clic ouvre une modale en lecture seule avec le contenu formaté
- Même formatage et comportement que dans le table view

### 7.7 Redimensionner l'éditeur et le panneau résultats
**En tant qu'** utilisateur,
**je veux** ajuster la taille de l'éditeur et du panneau de résultats,
**afin d'** optimiser mon espace de travail selon ce que je fais.

- Handle de resize draggable entre l'éditeur et les résultats
- Curseur `cursor-row-resize` sur le handle

---

## 8. Gestion des queries sauvegardées

### 8.1 Sauvegarder une query (Cmd+S)
**En tant qu'** utilisateur,
**je veux** sauvegarder ma requête SQL courante,
**afin de** la retrouver la prochaine fois que je me connecte.

- Raccourci : `Cmd+S`
- Bouton "Save" dans la toolbar de l'éditeur SQL
- Sauvegardé par connexion (hash de host:port:dbname:user)
- Indicateur de dirty state (point rempli = non sauvegardé, creux = sauvegardé)
- Erreur si le nom existe déjà (sauf update)

### 8.2 Ouvrir une query sauvegardée
**En tant qu'** utilisateur,
**je veux** ouvrir une query sauvegardée depuis la sidebar,
**afin de** reprendre mon travail sur une requête précédente.

- Section "Saved Queries" dans la sidebar (visible uniquement s'il y a des queries)
- Cliquer ouvre la query dans un nouvel onglet (ou focus l'existant)
- Maximum 20% de la hauteur de la sidebar (scrollable)

### 8.3 Renommer une query
**En tant qu'** utilisateur,
**je veux** renommer une query sauvegardée,
**afin de** mieux organiser mes requêtes.

- Clic droit > "Rename" dans la sidebar
- Cliquer sur le titre de l'onglet pour renommer inline
- Input inline : Enter ou blur pour valider, Escape pour annuler
- Erreur si le nouveau nom existe déjà

### 8.4 Dupliquer une query
**En tant qu'** utilisateur,
**je veux** dupliquer une query sauvegardée,
**afin de** créer une variante sans modifier l'originale.

- Clic droit > "Duplicate" dans la sidebar
- Crée une copie avec le suffixe "(copy)"

### 8.5 Supprimer une query
**En tant qu'** utilisateur,
**je veux** supprimer une query sauvegardée,
**afin de** nettoyer mes requêtes inutiles.

- Clic droit > "Delete" dans la sidebar
- Suppression permanente

---

## 9. Operations sur les tables

### 9.1 Supprimer une table (DROP)
**En tant qu'** utilisateur,
**je veux** supprimer une table entière,
**afin de** retirer une table dont je n'ai plus besoin.

- Clic droit sur la table dans la sidebar > "Drop"
- Dialogue de confirmation avant exécution
- `DROP TABLE ... CASCADE`

### 9.2 Vider une table (TRUNCATE)
**En tant qu'** utilisateur,
**je veux** vider toutes les lignes d'une table,
**afin de** repartir de zéro sans supprimer la structure.

- Clic droit sur la table dans la sidebar > "Truncate"
- Dialogue de confirmation avant exécution
- `TRUNCATE TABLE ... CASCADE`

### 9.3 Exporter une table en JSON
**En tant qu'** utilisateur,
**je veux** exporter toutes les données d'une table en fichier JSON,
**afin de** les utiliser dans d'autres outils ou les archiver.

- Clic droit sur la table dans la sidebar > "Export as JSON"
- Dialogue de sélection de fichier (file picker natif)
- Export de toutes les lignes en tableau JSON

### 9.4 Exporter une table en SQL
**En tant qu'** utilisateur,
**je veux** exporter toutes les données d'une table sous forme d'instructions INSERT SQL,
**afin de** pouvoir les rejouer sur une autre base ou les versionner.

- Clic droit sur la table dans la sidebar > "Export as SQL"
- Dialogue de sélection de fichier (file picker natif)
- Export en instructions `INSERT INTO ...`

---

## 10. Restauration de backup

### 10.1 Restaurer un backup PostgreSQL
**En tant qu'** utilisateur,
**je veux** restaurer un fichier `.tar.gz` de backup PostgreSQL sur ma base connectée,
**afin de** importer des données depuis un dump.

- Accessible via Commande Palette > "Restore Backup"
- File picker natif pour sélectionner le fichier `.tar.gz`
- Exécution via `pg_restore --clean --if-exists` en subprocess
- Logs en temps réel pendant la restauration
- Indicateur de statut : vert (succès) / rouge (échec avec message)
- Erreurs non-fatales (exit code 1 avec warnings) traitées comme succès avec warnings
- Nécessite `pg_restore` installé sur le système

---

## 11. Assistant IA (Claude)

### 11.1 Ouvrir le chat IA (Cmd+I)
**En tant qu'** utilisateur,
**je veux** ouvrir un panneau de chat IA dans l'éditeur SQL,
**afin de** demander de l'aide pour écrire des requêtes SQL.

- Raccourci : `Cmd+I` (toggle ouverture/fermeture)
- Panneau latéral (`w-96`) qui glisse depuis la droite
- Composé de : header, liste de messages (scrollable), textarea d'input + bouton Send

### 11.2 Générer du SQL avec l'IA
**En tant qu'** utilisateur,
**je veux** décrire en langage naturel ce que je veux et recevoir du SQL,
**afin de** ne pas avoir à écrire des requêtes complexes moi-même.

- Chaque message est envoyé avec le contexte complet :
  - Schéma de TOUS les schémas PostgreSQL de la connexion (tables, colonnes, types, PK)
  - Contenu actuel de l'éditeur SQL
- Réponses en streaming (affichage progressif)
- Historique de la conversation affiché (bulles user/assistant)
- Conversation non persistée (reset à chaque ouverture)

### 11.3 Appliquer le SQL suggéré à l'éditeur
**En tant qu'** utilisateur,
**je veux** insérer le SQL suggéré par l'IA directement dans mon éditeur,
**afin de** ne pas avoir à le copier/coller manuellement.

- Bouton "Apply to Editor" sur les blocs de code SQL dans les réponses
- Insert le SQL dans l'éditeur CodeMirror actif

### 11.4 Détection de l'installation de Claude
**En tant qu'** utilisateur,
**je veux** être informé si Claude Code n'est pas installé,
**afin de** savoir pourquoi le chat ne fonctionne pas et comment l'installer.

- Vérification au lancement : `which claude`
- Si absent : message dans le panneau "Claude Code is not installed. Install it from claude.com/code"
- Le lien est cliquable
- L'input est désactivé

---

## 12. Personnalisation

### 12.1 Changer le thème (Light/Dark)
**En tant qu'** utilisateur,
**je veux** basculer entre le thème clair et le thème sombre,
**afin d'** adapter l'interface à mes préférences visuelles.

- Accessible via Settings (Cmd+Shift+P > "Settings")
- Toggle Light / Dark
- Changement immédiat de toute l'interface
- Préférence persistée dans les settings

### 12.2 Personnaliser les raccourcis clavier
**En tant qu'** utilisateur,
**je veux** voir et modifier tous les raccourcis clavier de l'app,
**afin de** les adapter à mes habitudes.

- Accessible via Settings > section "Keyboard Shortcuts"
- Liste de tous les raccourcis groupés par catégorie
- Cliquer sur un raccourci active l'enregistrement d'une nouvelle combinaison
- Affichage avec symboles : macOS (Cmd, Shift, Option, Ctrl)
- Bouton "Reset to defaults" pour restaurer les raccourcis par défaut
- Changements appliqués immédiatement

---

## 13. Exclusion mutuelle des overlays

### 13.1 Un seul overlay à la fois
**En tant qu'** utilisateur,
**je veux** qu'un seul overlay soit ouvert à la fois (palette de commandes, finder, find bar, settings, restore, chat),
**afin de** ne jamais me retrouver bloqué avec des overlays empilés.

- Ouvrir un overlay ferme automatiquement tout overlay déjà ouvert
- Escape ferme toujours l'overlay actif
- Pas d'état "bloqué" possible
