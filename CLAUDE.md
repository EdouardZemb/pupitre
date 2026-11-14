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
  - `noyau` (`crates/noyau`) : la théorie, les pages SVG et les règles des données, sans entrée-sortie ;
  - `cahier` (`crates/cahier`) : le PDF et sa commande en ligne ;
  - `stockage` (`crates/stockage`) : la racine des données, les documents et les captations sur fichiers, côté natif ; sa fonction de Cargo `pannes`, que seuls ses tests activent, coupe une écriture à l'étape voulue, et ouvre le fonds selon les migrations qu'un test lui donne (`fonds_selon`) ;
  - `synchro` (`crates/synchro`) : le cœur de la couche de synchronisation, sans réseau ;
  - `commandes` (`crates/commandes`) : les types des commandes et la liste de leurs noms, que l'interface et l'établi partagent ;
  - `interface` (`app/ui`) : l'interface Leptos ;
  - `etabli` (`app/src-tauri`) : l'application Tauri.
- La CI (`.github/workflows/ci.yml`) a pour jobs `socle`, `contenu`, `documentation`, `verifier`, `windows` et `mutants`. Le passage complet de mutation a lieu chaque semaine, par `mutants-complet.yml`. Les portes et ce qu'elles protègent sont dans `CONVENTIONS.md` (« Portes de CI »).

## Commandes

Les vérifications du projet, que la CI rejoue (`contenu` et `documentation`) :

- `bash scripts/verif-confidentialite.sh` : refuse tout motif interdit dans les fichiers suivis ; ce qu'il refuse est dit dans `CONVENTIONS.md` (« Portes de CI », ligne `contenu`).
- `node scripts/verif-liens.mjs` : chaque lien interne et chaque ancre mènent quelque part.
- `node scripts/verif-faits.mjs` : chaque fait recopié dit la même chose que sa source, et chaque critère inscrit au § 70 · 4.1 a un test Rust qui le nomme ; ce qu'il lit est dit dans `CONVENTIONS.md` (« Portes de CI », ligne `documentation`). Ses tests : `node --test scripts/verif-faits.test.mjs`. Hors CI, il compare les contrôles requis à GitHub s'il le peut, et sinon il avertit.
- `node scripts/verif-diagrammes.mjs` : chaque diagramme Mermaid se rend. Il lui faut d'abord `npm install --no-save --no-package-lock --no-audit --no-fund @mermaid-js/mermaid-cli@12.0.0`, puis `npx --no-install puppeteer browsers install chrome`, qui télécharge le navigateur de Puppeteer ; `CHROME_PATH` peut en désigner un autre, au lieu de cette seconde commande.

Les commandes de Rust, telles que la CI les lance. Deux usages du poste : chaque commande qui construit se lance avec `-p <crate>`, jamais sur tout l'espace de travail ; les tests du cahier sont lents en debug, de l'ordre d'une minute et demie.

- `cargo update --workspace`, puis `git diff --exit-code Cargo.lock` : le `Cargo.lock` suit les `Cargo.toml` ; ces commandes ne construisent rien. L'étape « Cargo.lock à jour » de `verifier` les lance ; en échec, elle dépose le `Cargo.lock` qu'elle a résolu en artefact `cargo-lock`, à relire puis à committer : le tenir à jour ne demande aucun cargo local.
- `cargo fmt --all --check` : le format du code.
- `cargo clippy -p noyau -p cahier -p stockage -p synchro --all-targets -- -D warnings` : les avertissements du noyau, du cahier, du stockage et de la synchronisation valent des erreurs. Le job `windows` passe aussi `stockage` et `synchro` à `clippy` et à leurs tests, sous Windows (H07).
- `cargo clippy -p stockage --lib -- -D warnings` : compile le stockage sans `pannes`, tel que l'établi le recevra, ses avertissements valant des erreurs ; `verifier` et `windows` le lancent.
- `cargo clippy -p interface --all-targets -- -D warnings`, puis `cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings` : de même pour l'interface, sur la cible de l'hôte puis sur celle du navigateur.
- `cargo clippy -p etabli --all-targets -- -D warnings` : de même pour l'application ; elle demande `app/ui/dist`, donc une construction de l'interface avant.
- `cargo test -p noyau -p cahier -p interface -p stockage -p synchro` : les tests, dont ceux par propriétés ; `cargo test -p etabli` pour l'application.
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
