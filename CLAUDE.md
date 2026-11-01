@docs/modele/methode.md

# pupitre

Atelier d'écriture musicale, du motif à l'album.

Ce dépôt porte la conception d'une application d'aide à la composition, écrite pour le persona Sacha, qui écrit seul ses chansons, et son code à partir de l'incrément INC-0. Il est public : rien de personnel n'y entre.

## État

- Le dépôt est né le 01/11/2026, d'un seul commit, de la scission de pupitre (M-0006 § 7). L'historique de la conception jusque-là, ses PR #1 à #15 et l'étiquette `v0.1.0` restent dans le dépôt privé `pupitre-donnees`.
- La conception est écrite : `docs/10-vision.md` à `docs/90-questions-ouvertes.md`, et 21 ADR dans `docs/adr/` (10 acceptées, 8 proposées, 3 remplacées). La pile est Rust (ADR-019 à 021) ; la table de la tonalité est conçue (MIC-12, MIC-13, MOR-14, REN-08, ADR-018).
- Le dépôt est né de `modele-projet` v0.7.1 : nature `code`, cible `aucune`, fusion `session`.
- Le lot C est en cours : après la naissance du dépôt, l'issue d'INC-0 et la PR de son plan. Le plan s'écrit d'après la spec d'INC-0 du 01/11, que la session reçoit d'Edouard ; il dit lui-même ce qui doit survivre.
- Aucune ligne de code encore : INC-0 s'ouvre par les essais H09 et H10 (§ 40 · 11).

## Commandes

Les trois vérifications du projet, que la CI rejoue (`contenu` et `documentation`) :

- `bash scripts/verif-confidentialite.sh` : refuse tout motif interdit dans les fichiers suivis (e-mail, secret, audio, chemin de machine).
- `node scripts/verif-liens.mjs` : chaque lien interne et chaque ancre mènent quelque part.
- `node scripts/verif-diagrammes.mjs` : chaque diagramme Mermaid se rend. Il demande `npm install --no-save --no-package-lock --no-audit --no-fund @mermaid-js/mermaid-cli@12.0.0` avant.

Les commandes de Rust (`cargo fmt`, `cargo clippy`, `cargo test`, `cargo mutants`) s'ajoutent avec le premier code d'INC-0.

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
