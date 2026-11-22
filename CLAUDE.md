@docs/modele/methode.md

# pupitre

Atelier d'écriture musicale, du motif à l'album.

Ce dépôt porte la conception d'une application d'aide à la composition, écrite pour le persona Sacha, qui écrit seul ses chansons, et son code à partir de l'incrément INC-0. Il est public : rien de personnel n'y entre.

## État

- Le dépôt est né le 01/11/2026, d'un seul commit, de la scission de pupitre (M-0006 § 7). L'historique de la conception jusque-là, ses PR #1 à #15 et l'étiquette `v0.1.0` restent dans le dépôt privé `pupitre-donnees`.
- La conception est écrite : `docs/10-vision.md` à `docs/90-questions-ouvertes.md`, et 24 ADR dans `docs/adr/` (15 acceptées, 6 proposées, 3 remplacées). La pile est Rust (ADR-019 à 021) ; les documents et leur synchronisation vivent côté natif (ADR-022), et leur stockage est écrit (ADR-023) ; l'import d'un fonds existant est conçu (ADR-024, Q23 tranchée le 2026-11-09) ; la table de la tonalité est conçue (MIC-12, MIC-13, MOR-14, REN-08, ADR-018).
- Le dépôt est né de `modele-projet` v0.7.1 : nature `code`, cible `aucune`, fusion `session`.
- INC-0 est livré, sous l'étiquette `v0.2.0` du 03/11/2026 : l'issue #1, sous la milestone « INC-0 · la table », close, et ses PR #2 à #9. La prochaine étape est INC-1, construit par lots (§ 40 · 9).
- H07 est passé le 2026-11-05 (PR #18, issue #16) : sous Linux et sous Windows, deux répliques du fonds modifiées hors ligne convergent par le protocole d'Automerge, avant et après chaque réouverture, et une copie restaurée de la racine des données relit chaque captation contre son empreinte. Le plan du lot 1 part de là, et lève les simplifications de l'essai (AM-4).
- Les crates :
  - `noyau` (`crates/noyau`) : la théorie, les pages SVG et les règles des données, sans entrée-sortie, dont le repli des textes (`repli`), les requêtes, leurs jours civils et les requêtes enregistrées (`requete`), l'index des idées interrogées et la réponse d'une requête (`index`), le journal rangé et son empreinte (`journal`), et les règles de l'import : les signatures et les extensions (`signature`), les en-têtes d'un WAV et d'un MIDI, le texte lisible et son décodage (`entete`, dont `decoder_le_texte`), le manifeste, le rapport d'import, ses comptes et la place libre (`rapport`), l'origine déclarée, le premier nom vu et la portée (`origine`), les stems, leur type proposé, le MIDI d'un stem et le jeu proposé (`stems`), et les opérations nommées de l'import (`import`) ;
  - `cahier` (`crates/cahier`) : le PDF et sa commande en ligne ;
  - `stockage` (`crates/stockage`) : la racine des données, les documents et les captations sur fichiers, côté natif, dont la copie en flux d'un fichier versé (`Captations::verser_en_flux`, sous `import-<ULID>-<rang>.<processus>.partiel`), et la source à verser sur fichiers (`stockage::source::SourceSurFichiers`), qui parcourt le dossier choisi sans suivre un lien ; sa fonction de Cargo `pannes`, que seuls ses tests activent, coupe une écriture à l'étape voulue, ouvre le fonds selon les migrations qu'un test lui donne (`fonds_selon`, `ouvrir_selon`), et garde pour les tests l'écriture et la naissance par une fermeture (`Document::ecrire`, `faire_naitre_le_fonds`, `Naissance`) ;
  - `synchro` (`crates/synchro`) : le cœur de la couche de synchronisation, sans réseau ;
  - `commandes` (`crates/commandes`) : les types des commandes, dont la requête que porte la commande `boite`, les requêtes enregistrées et ceux de l'import (`Inspection`, `Declaration`, `RapportDImport`…), le nom de chaque commande de l'application en constante (`NOTER`, …, `GENERATEURS`, `EXPORTER_CAHIER`), et la liste de ces noms (`NOMS`), que l'interface et l'établi partagent ;
  - `services` (`crates/services`) : les opérations nommées et les lectures du fonds, conduites sur les trois contrats des adaptateurs, `Documents`, `Captations` et `Source`, la source à verser (§ 40 · 3) ; l'import d'un fonds, de l'inspection à la clôture, et son état, gardé entre deux commandes (`etat_de_l_import`) ; l'index des idées interrogées, qu'ils tiennent et rebâtissent à la lecture qui suit un changement des têtes du fonds (§ 31 · 7) ; et le générateur du fonds de mesure (`mesure`, § 31 · 10, test 13) et la source à verser en mémoire qu'il emploie (`source_en_memoire::SourceEnMemoire`), compilés hors d'`en-memoire` ; sa fonction de Cargo `en-memoire`, que seuls les tests activent, compile la paire d'adaptateurs en mémoire ;
  - `interface` (`app/ui`) : l'interface Leptos ; l'établi, qui s'ouvre sur le fonds (`app`), et l'en-tête de l'établi (`entete`), qui porte la recherche, l'état de synchronisation et « Afficher », le fonds ou la table ; la table, la palette et l'export du cahier ; la requête de l'écran et ses touches, `/` et Échap, en fonctions pures (`recherche`) ; la boîte d'idées (`boite`), ses ordres, sa liste, réponse de la requête en cours, et les critères de celle-ci, son compte, ses gestes de tri, la page d'un fonds vide, les requêtes enregistrées et le volet du journal du fonds ; le bandeau (`bandeau`), qui dit l'état du fonds, ses avertissements et le refus d'un geste (I29), et la page d'ouverture ; le pont des commandes de l'établi (`natif`), qui lit aussi le décalage local d'un geste et d'une date ;
  - `etabli` (`app/src-tauri`) : l'application Tauri ; son module `systeme`, seul à lire le dossier de l'application, l'heure, l'aléa et la place libre du disque (I30) : `place_libre`, par `place_offerte`, qui appelle `GetDiskFreeSpaceExW`, déclaré dans le seul bloc `unsafe extern "system"` du dépôt, et `admettre_le_dossier`, qui refuse le dossier de l'application, un dossier qui le contient ou qui s'y trouve ; le fil du fonds (`fil`), qui possède la racine des données et les services, et sert une à une les commandes que l'interface envoie par l'IPC de Tauri (`commandes_de_l_etabli`), dont les dix de l'import, d'`inspecter_un_dossier`, qui ouvre la boîte « Importer un dossier », à `generateurs` ; le dossier choisi, parcouru, que se partagent les commandes de l'import (`DossierChoisi`) ; le manifeste de l'application, que `build.rs` tire de `commandes::NOMS`, `exporter_cahier` compris, et les droits nommés de Windows (`capabilities/windows.json` : `allow-<nom>` pour chaque commande, aucun droit du noyau de Tauri ni d'un greffon), que les tests d'I31 comparent à `commandes::NOMS` (§ 40 · 7.5) ; le protocole `captation`, enregistré par `run()`, qui refuse toute requête jusqu'à la tranche 5.
- La CI (`.github/workflows/ci.yml`) a pour jobs `socle`, `contenu`, `documentation`, `verifier`, `windows` et `mutants`. Le passage complet de mutation a lieu chaque semaine, par `mutants-complet.yml`. Les portes et ce qu'elles protègent sont dans `CONVENTIONS.md` (« Portes de CI »).

## Commandes

Les vérifications du projet, que la CI rejoue (`contenu` et `documentation`) :

- `bash scripts/verif-confidentialite.sh` : refuse tout motif interdit dans les fichiers suivis ; ce qu'il refuse est dit dans `CONVENTIONS.md` (« Portes de CI », ligne `contenu`).
- `node scripts/verif-liens.mjs` : chaque lien interne et chaque ancre mènent quelque part.
- `node scripts/verif-faits.mjs` : chaque fait recopié dit la même chose que sa source, chaque critère inscrit au § 70 · 4.1 a un test Rust qui le nomme, et la CSP de `app/src-tauri/tauri.conf.json` égale, octet pour octet, le bloc de la CSP exacte du § 40 · 7.5, sans fichier propre à Windows (`tauri.windows.conf.json`, `tauri.windows.conf.json5` ou `Tauri.windows.toml`) que Tauri fusionnerait par-dessus ; ce qu'il lit est dit dans `CONVENTIONS.md` (« Portes de CI », ligne `documentation`). Ses tests : `node --test scripts/verif-faits.test.mjs`. Hors CI, il compare les contrôles requis à GitHub s'il le peut, et sinon il avertit.
- `node scripts/verif-diagrammes.mjs` : chaque diagramme Mermaid se rend. Il lui faut d'abord `npm install --no-save --no-package-lock --no-audit --no-fund @mermaid-js/mermaid-cli@12.0.0`, puis `npx --no-install puppeteer browsers install chrome`, qui télécharge le navigateur de Puppeteer ; `CHROME_PATH` peut en désigner un autre, au lieu de cette seconde commande.

Les commandes de Rust, telles que la CI les lance. Deux usages du poste : chaque commande qui construit se lance avec `-p <crate>`, jamais sur tout l'espace de travail ; les tests du cahier sont lents en debug, de l'ordre d'une minute et demie.

- `cargo update --workspace`, puis `git diff --exit-code Cargo.lock` : le `Cargo.lock` suit les `Cargo.toml` ; ces commandes ne construisent rien. L'étape « Cargo.lock à jour » de `verifier` les lance ; en échec, elle dépose le `Cargo.lock` qu'elle a résolu en artefact `cargo-lock`, à relire puis à committer : le tenir à jour ne demande aucun cargo local.
- `cargo tree -p stockage -e no-dev --prefix none`, puis `cargo tree -p interface -e no-dev --prefix none` et `cargo tree -p interface -e no-dev --prefix none --target wasm32-unknown-unknown` : l'interface ne tire ni `automerge`, ni les services, ni le stockage (§ 40 · 3 ; ADR-022), sur la cible de l'hôte comme sur celle du navigateur. L'essai sur le stockage doit trouver une ligne qui commence par `automerge v` et une qui commence par `services v` ; dans les deux graphes de l'interface, aucune ligne ne doit commencer par `automerge v`, `services v` ni `stockage v`. L'étape « Le graphe de l'interface » de `verifier` les lance ; ces commandes ne construisent rien.
- `cargo tree -p stockage -e features --prefix none` et `cargo tree -p stockage -e no-dev,features --prefix none`, puis `cargo tree -p etabli -e no-dev,features --prefix none --target x86_64-pc-windows-msvc` : l'établi tel qu'il se livre, pour Windows, sa seule cible, ne tire ni la fonction `pannes` du stockage, ni `en-memoire` des services, ni `protocol-asset` de Tauri. L'essai sur le stockage doit trouver une ligne qui commence par `stockage feature "pannes"` dans le premier graphe, où ses dépendances de test l'activent, et aucune dans le second ; dans le graphe de l'établi, une ligne doit commencer par `tauri feature "`, et aucune ne doit nommer `feature "en-memoire"`, `feature "pannes"` ni `feature "protocol-asset"`. L'étape « Le graphe de l'établi » de `verifier` les lance ; ces commandes ne construisent rien.
- `cargo fmt --all --check` : le format du code.
- `cargo clippy -p noyau -p cahier -p stockage -p synchro -p commandes -p services --all-targets -- -D warnings` : les avertissements du noyau, du cahier, du stockage, de la synchronisation, des commandes et des services valent des erreurs. Le job `windows` passe aussi `stockage` et `synchro` à `clippy` et à leurs tests, sous Windows (H07).
- `cargo clippy -p stockage --lib -- -D warnings` : compile le stockage sans `pannes`, tel que l'établi le recevra, ses avertissements valant des erreurs ; `verifier` et `windows` le lancent.
- `cargo clippy -p services --lib -- -D warnings` : compile les services sans `en-memoire`, tels que l'établi les recevra, leurs avertissements valant des erreurs ; `verifier` le lance.
- `cargo clippy -p interface --all-targets -- -D warnings`, puis `cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings` : de même pour l'interface, sur la cible de l'hôte puis sur celle du navigateur.
- `cargo clippy -p etabli --all-targets -- -D warnings` : de même pour l'application ; elle demande `app/ui/dist`, donc une construction de l'interface avant.
- `cargo test -p noyau -p cahier -p interface -p stockage -p synchro -p commandes -p services` : les tests, dont ceux par propriétés ; `cargo test -p etabli` pour l'application. L'étape « La sentinelle d'I30 » de `windows` relance ceux de l'application par `cargo test -p etabli -- --include-ignored`, garde-fou d'I30 compris, sous un dossier de l'application redirigé vers une sentinelle vide : elle échoue si Tauri résout ce dossier hors de la sentinelle, ou si un fichier y paraît.
- `cargo test -p stockage --release --test mesure -- --ignored --nocapture` : le test 13 du § 31 · 10, entier. Le fonds de mesure, engendré par les opérations nommées, celles de l'import comprises, puis rouvert sur fichiers, pèse moins de 4 500 000 octets, et une compaction de 101 fichiers d'ajout passe en 2 s au plus ; la commande imprime le poids, les acteurs et les durées. `cargo test` saute ce test, ignoré ; l'étape « Le fonds de mesure (test 13) » de `verifier` le lance, en `--release`.
- `cargo mutants -p noyau` : la mutation des tests du noyau ; `--in-diff <fichier>` la limite aux lignes d'un diff, comme sur une PR.
- `trunk build` dans `app/ui` : construit l'interface en WebAssembly, dont `etabli` a besoin.
- `cargo tauri build` dans `app/src-tauri` : construit l'application et son installateur.
- `cargo run -p cahier -- --date AAAA-MM-JJ [--instrument guitare|clavier|les-deux] [--couche masquee|affichee] --sortie <fichier>` : engendre le cahier des tables en PDF ; la date est une entrée, jamais l'horloge.

## Ce qui fait foi

*En cas de désaccord, c'est l'endroit nommé qui a raison. Un fichier de la racine s'y cite par un lien.*

- Le comportement attendu, c'est [la SFD](docs/20-sfd/README.md) et [la qualité et la vérification](docs/70-qualite-et-verification.md) ; une fois écrits, le code et ses tests.
- Le besoin de départ, c'est [le brief](docs/00-brief-initial.md), qui ne se réécrit pas, et les choix de départ, [le cadrage](docs/cadrage.md), qui renvoie aux documents de la conception.
- Les décisions, ce sont [celles du projet](docs/adr/) (série `adr-NNN`) et [celles du modèle](docs/modele/decisions/) (préfixe `M-`).
- La feuille de route, ce sont les issues, [les pistes](docs/pistes.md) et [l'architecture](docs/40-architecture.md) (§ 9, la trajectoire de construction).
- Le vocabulaire, c'est [le glossaire](docs/80-glossaire.md).
- Les constats, c'est [le registre](docs/registre.md).

## Invariants — ne jamais violer

Les invariants communs sont dans `docs/modele/methode.md`. Ceux-ci sont propres au projet.

- Le persona est Sacha, jamais une personne réelle : la conception décrit un profil, pas quelqu'un.
- Rien de personnel dans le dépôt : ni paroles, ni audio, ni prénom, ni chemin de machine ; les fichiers gérés par le modèle et le nom du compte `EdouardZemb` en sont exceptés. Le dépôt est public : une fuite y serait lue de tous, et l'historique la garderait.
- Les exemples sont inventés, jamais tirés de chansons réelles.
- Un mot pour une chose : le glossaire tranche, et un second mot sème le doute.
- Un identifiant retiré n'est jamais réattribué : un numéro qui change de sens casse tous les renvois.
- Une ADR n'est jamais renumérotée ni supprimée. Une ADR remplacée garde son fichier.
- Aucune information n'est portée par la seule couleur, dans les maquettes, dans la conception et dans le code : le mode encre n'a pas de couleur.
