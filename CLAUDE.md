@docs/modele/methode.md

# pupitre

Atelier d'écriture musicale, du motif à l'album.

Ce dépôt porte la conception d'une application d'aide à la composition, écrite pour le persona Sacha, qui écrit seul ses chansons, et son code à partir de l'incrément INC-0. Il est public : rien de personnel n'y entre.

## État

- Le dépôt est né le 01/11/2026, d'un seul commit, de la scission de pupitre (M-0006 § 7). L'historique de la conception jusque-là, ses PR #1 à #15 et l'étiquette `v0.1.0` restent dans le dépôt privé `pupitre-donnees`.
- La conception est écrite : `docs/10-vision.md` à `docs/90-questions-ouvertes.md`, et 21 ADR dans `docs/adr/` (10 acceptées, 8 proposées, 3 remplacées). La pile est Rust (ADR-019 à 021) ; la table de la tonalité est conçue (MIC-12, MIC-13, MOR-14, REN-08, ADR-018).
- Le dépôt est né de `modele-projet` v0.7.1 : nature `code`, cible `aucune`, fusion `session`.
- INC-0 est livré : l'issue #1, sous la milestone « INC-0 · la table », et ses PR #2 à #9. Il reste l'étiquette `v0.2.0`, à poser par la maîtrise d'ouvrage, puis la prochaine étape, INC-1 (§ 40 · 9).
- Les crates :
  - `noyau` (`crates/noyau`) : la théorie et les pages SVG, sans entrée-sortie ;
  - `cahier` (`crates/cahier`) : le PDF et sa commande ;
  - `interface` (`app/ui`) : l'interface Leptos ;
  - `etabli` (`app/src-tauri`) : l'application Tauri.
- La CI (`.github/workflows/ci.yml`) a pour jobs `socle`, `contenu`, `documentation`, `verifier`, `windows` et `mutants`. Le passage complet de mutation a lieu chaque semaine, par `mutants-complet.yml`. Les portes et ce qu'elles protègent sont dans `CONVENTIONS.md` (« Portes de CI »).

## Commandes

Les trois vérifications du projet, que la CI rejoue (`contenu` et `documentation`) :

- `bash scripts/verif-confidentialite.sh` : refuse tout motif interdit dans les fichiers suivis (e-mail, secret, audio, chemin de machine).
- `node scripts/verif-liens.mjs` : chaque lien interne et chaque ancre mènent quelque part.
- `node scripts/verif-diagrammes.mjs` : chaque diagramme Mermaid se rend. Il demande `npm install --no-save --no-package-lock --no-audit --no-fund @mermaid-js/mermaid-cli@12.0.0` avant.

Les commandes de Rust, telles que la CI les lance. Deux usages du poste : chaque commande se lance avec `-p <crate>`, jamais sur tout l'espace de travail ; les tests du cahier sont lents en debug, de l'ordre d'une minute et demie.

- `cargo fmt --all --check` : le format du code.
- `cargo clippy -p noyau -p cahier --all-targets -- -D warnings` : les avertissements du noyau et du cahier valent des erreurs.
- `cargo clippy -p interface --all-targets -- -D warnings`, puis `cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings` : de même pour l'interface, sur la cible de l'hôte puis sur celle du navigateur.
- `cargo clippy -p etabli --all-targets -- -D warnings` : de même pour l'application ; elle demande `app/ui/dist`, donc une construction de l'interface avant.
- `cargo test -p noyau -p cahier -p interface` : les tests, dont ceux par propriétés ; `cargo test -p etabli` pour l'application.
- `cargo mutants -p noyau` : la mutation des tests du noyau ; `--in-diff <fichier>` la limite aux lignes d'un diff, comme sur une PR.
- `trunk build` dans `app/ui` : construit l'interface en WebAssembly, dont `etabli` a besoin.
- `cargo tauri build` dans `app/src-tauri` : construit l'application et son installateur.
- `cargo run -p cahier -- --date AAAA-MM-JJ [--instrument guitare|clavier|les-deux] [--couche masquee|affichee] --sortie <fichier>` : engendre le cahier des tables en PDF ; la date est une entrée, jamais l'horloge.

## Ce qui fait foi

*En cas de désaccord, c'est l'endroit nommé qui a raison.*

- Le comportement attendu, c'est la SFD (`docs/20-sfd/`) et `docs/70-qualite-et-verification.md` ; une fois écrits, le code et ses tests.
- Les décisions, ce sont `docs/adr/` (série `adr-NNN`) et `docs/modele/decisions/` (préfixe `M-`).
- La feuille de route, ce sont les issues, `docs/pistes.md` et `docs/40-architecture.md` (§ 9, la trajectoire de construction).
- Le vocabulaire, c'est `docs/80-glossaire.md`.
- Les constats, c'est `docs/registre.md`.

## Invariants — ne jamais violer

Les invariants communs sont dans `docs/modele/methode.md`. Ceux-ci sont propres au projet.

- Le persona est Sacha, jamais une personne réelle : la conception décrit un profil, pas quelqu'un.
- Rien de personnel dans le dépôt : ni paroles, ni audio, ni prénom, ni chemin de machine. Le dépôt est public : une fuite y serait lue de tous, et l'historique la garderait.
- Les exemples sont inventés, jamais tirés de chansons réelles.
- Un mot pour une chose : le glossaire tranche, et un second mot sème le doute.
- Un identifiant retiré n'est jamais réattribué : un numéro qui change de sens casse tous les renvois.
- Une ADR n'est jamais renumérotée ni supprimée. Une ADR remplacée garde son fichier.
- Aucune information n'est portée par la seule couleur, dans les maquettes, dans la conception et dans le code : le mode encre n'a pas de couleur.
