# Conventions

Volontairement minces. Ce dépôt porte la conception de Pupitre, et son code à partir d'INC-0.
La conception se lit sur un écran d'ordinateur, un téléphone et une tablette à encre. Tout
cérémonial ajouté ici doit se justifier par un incident réel, pas par une bonne pratique
générale.

## Écrire la documentation

- **Un profil, pas une personne.** La conception parle du persona Sacha
  ([§ 11](docs/11-persona-et-situations.md)), au genre neutre ; jamais d'une personne
  réelle, ni par son nom ni par son histoire.
- **Des exemples inventés, et qui le disent.** Titres, grilles, paroles et tempos des
  exemples sont fictifs. Aucun n'est tiré d'une chanson réelle.
- **Un mot pour une chose.** Le [glossaire](docs/80-glossaire.md) fait foi ; un mot
  nouveau y entre avant d'être employé ailleurs.
- **Des identifiants stables.** Besoins `B01`, douleurs `D1`, anti-besoins `N1`,
  situations `S1`, principes `P1`, risques `R1`, indicateurs `IR-1`, fonctions `CAP-01`,
  diagnostics `DG-P1`, invariants `I01`, hypothèses `H01`, incréments `INC-1`, questions
  `Q01`, arbitrages `A01`, ADR `adr-NNN-sujet.md`. Un identifiant retiré n'est jamais
  réattribué ; le [glossaire](docs/80-glossaire.md) les recense, sauf les fonctions, à
  l'[index de la SFD](docs/20-sfd/README.md#les-fonctions), les diagnostics, au
  [§ 60 · 3](docs/60-moteur-musical.md#3-le-catalogue-des-diagnostics), et les ADR, à
  [leur index](docs/adr/README.md).
- **Un fait à un seul endroit.** Un compte, une plage ou une liste qui décrivent le dépôt
  s'écrivent là où ils sont définis ; ailleurs, on y renvoie. Ceux qui restent recopiés, la
  porte `documentation` les compare à leur source (« Portes de CI », plus bas). Elle impose
  trois graphies. Une plage qui part du premier numéro (`Q01–Qnn`) couvre toute la famille. Un
  compte partiel ne s'écrit pas en chiffres devant « ADR » (« N ADR » est le total), ni, dans le
  README, `CLAUDE.md` et ce fichier, en lettres ou en chiffres devant « ADR », « décisions
  d'architecture », « portes », « jobs », « modules », « crates » ou « situations ». Les
  contrôles requis ne se disent qu'à « Protection de `main` ».
- **Visuel d'abord.** Un schéma Mermaid quand une structure, un flux ou un cycle se
  dessine ; une maquette SVG dans `docs/assets/maquettes/` pour un écran, lisible en noir
  et blanc pur.
- **Les diagrammes, ce qui casse.** Types reconnus par GitHub seulement (pas de
  `sankey`) ; jamais de point-virgule dans le texte d'un `sequenceDiagram`, il coupe la
  ligne.
- **Une décision s'écrit en ADR**, pas seulement dans une PR ni dans un document de
  synthèse.

## Commits

Le format est celui du modèle de projet, **Conventional Commits** : un préfixe, deux points,
une phrase à l'infinitif, en français, en-tête de 100 caractères au plus. Le hook
`commit-msg` l'impose, et la CI le rejoue sur la PR. Le cycle d'un lot est dans
[`docs/modele/methode.md`](docs/modele/methode.md).

```
docs: ajouter les cas limites de la transposition a la SFD
docs(adr): adr-018 la table montre tout sans choisir
fix: corriger l'assiette de DG-A1 dans la maquette d'album
ci: verifier aussi les liens des balises img
chore: aligner le depot sur le socle du modele
```

Les anciens préfixes se traduisent ainsi :

| Avant | Maintenant |
|---|---|
| `doc:` | `docs:` |
| `adr:` | `docs(adr):` |
| `corr:` | `fix:` |
| `ci:` | `ci:` (inchangé) |
| `meta:` | `chore:` |

Le corps est optionnel. Quand il existe, il dit **pourquoi**, pas quoi — le diff dit déjà
quoi.

## Branches

```
docs/<sujet-court>
docs/adr-<numero>-<sujet-court>
fix/<sujet-court>
chore/<sujet-court>
```

Exemples : `docs/sfd-album`, `docs/adr-018-la-table-montre-tout`, `fix/assiette-dg-a1`.

Jamais de travail direct sur `main`, même pour une ligne : le hook `pre-push` et le garde de
Claude refusent tout envoi vers la branche principale (plus bas, « Protection de `main` »).
Toute modification passe par une branche et une PR, pour que les portes de CI tournent avant
que `main` ne bouge ; la session la fusionne en squash quand tout est vert (`fusion: session`
dans `.copier-answers.yml`).

## Pull requests

Le gabarit, `.github/pull_request_template.md`, est **géré par le modèle** : il ne se modifie
pas ici ([`docs/modele/methode.md`](docs/modele/methode.md), « Les fichiers gérés par le
modèle »). Pas de revue requise par GitHub.

Sous « Vérifications », chaque PR de pupitre reprend en plus trois contrôles propres au dépôt,
dont les règles sont plus haut (« Écrire la documentation ») et plus bas (« Ce qui n'entre
jamais dans ce dépôt ») :

- [ ] Rien de personnel n'entre dans le dépôt (paroles réelles, audio, e-mail, prénom, clé, chemin de machine),
  hors des fichiers gérés par le modèle et du nom du compte `EdouardZemb`.
- [ ] La documentation parle du persona, jamais d'une personne réelle ; les exemples sont inventés.
- [ ] Si une décision est prise ici, elle est écrite en ADR, pas seulement dans la PR.

## Portes de CI

**Huit, aujourd'hui.** Chacune marque la PR en échec. Celles que la règle de protection exige
sont dites à un seul endroit, plus bas, à « Protection de `main` ». Requise ou non, une porte
rouge arrête la fusion : on ne fusionne pas une PR rouge.

| Porte | Ce qu'elle refuse | Script | Pourquoi elle existe |
|---|---|---|---|
| `socle / secrets` | Un secret dans un commit ou dans l'historique | Fournie par le modèle (gitleaks) | Un secret poussé est un secret à changer |
| `socle / confidentialite` | Une donnée d'identité dans le diff ou les messages de commit ; un commit hors du format Conventional Commits ; un plan de `.claude/plans/` qu'une PR ajoute ou modifie sans ses trois sections ; un `.claude/settings.json` qui ne branche plus le garde de Claude ou `signaux.py` ; un `LICENSE` absent, vide ou non suivi ; un `CLAUDE.md` que le README ne lie pas, ou un autre `.md` de la racine que ni le README ni « Ce qui fait foi » de `CLAUDE.md` ne lient | [`scripts/modele/confidentialite.py`](scripts/modele/confidentialite.py), [`scripts/modele/commits.py`](scripts/modele/commits.py), [`scripts/modele/documentation.py`](scripts/modele/documentation.py), [`scripts/modele/plan.py`](scripts/modele/plan.py) et une étape de [`modele-socle.yml`](.github/workflows/modele-socle.yml) | Le modèle protège tous ses dépôts de la même façon |
| `texte-pr` | Une donnée d'identité dans le titre ou le corps de la PR ; un titre hors du format Conventional Commits ; un corps sans la section « Écarts à la spec », ou mal formée ; une PR qui ferme une issue sans la ligne `Tête relue : <SHA>` égale à sa tête | [`scripts/modele/texte_pr.py`](scripts/modele/texte_pr.py), [`scripts/modele/confidentialite.py`](scripts/modele/confidentialite.py), [`scripts/modele/commits.py`](scripts/modele/commits.py) et [`modele-texte-pr.yml`](.github/workflows/modele-texte-pr.yml) | Seul workflow qui écoute `edited` : une retouche du titre ou du corps ne relance que lui ([`docs/modele/methode.md`](docs/modele/methode.md), « Le texte de la PR ») |
| `contenu` | Une donnée personnelle dans les fichiers du dépôt : e-mail, secret, export de mémoire, fichier audio, MIDI ou partition, chemin absolu de machine ; un document (`*.automerge`) ou une archive (`pupitre-archive-*/`) d'un fonds, suivis, sans exemption | [`verif-confidentialite.sh`](scripts/verif-confidentialite.sh) | Le dépôt ne doit rien dire de personne. Nommée `contenu` pour ne pas se confondre avec `socle / confidentialite`. Un fonds ne se versionne jamais ([§ 40 · 9](docs/40-architecture.md#9-trajectoire-de-construction)) : aucun dossier n'en est exempté |
| `documentation` | Un diagramme qui ne se rend pas ; un lien interne ou une ancre qui ne mène nulle part ; un fait recopié qui diverge de sa source : les plages des familles d'identifiants du glossaire, les comptes des ADR et leur ventilation par statut, leur index et leur graphe, les fonctions de la SFD (leur compte, leur index, la matrice de traçabilité), les portes et les jobs, la règle de protection (dite à un seul endroit, et comparée en CI à celle de GitHub), les crates, et les comptes écrits en lettres dans le README, `CLAUDE.md` et ce fichier ; les critères livrés ([§ 70 · 4.1](docs/70-qualite-et-verification.md#41-les-critères-livrés)) : des critères d'acceptation de la SFD numérotés avec un trou ou un doublon, une fonction de la SFD dont aucun critère ne se lit, un critère inscrit que la SFD n'a pas, qu'elle a retiré ou qu'aucun test qui tourne ne nomme, une fonction Rust nommée d'après un critère que la SFD n'a pas ou qu'elle a retiré, ou nommée `mic_12_1_…` | [`verif-diagrammes.mjs`](scripts/verif-diagrammes.mjs), [`verif-liens.mjs`](scripts/verif-liens.mjs), [`verif-faits.mjs`](scripts/verif-faits.mjs) et ses tests | Pendant la refonte d'octobre 2026, des diagrammes se sont cassés sur leur syntaxe ; sur GitHub, un diagramme cassé s'affiche comme un message d'erreur. Les faits recopiés : trois fois, une PR a changé un fait à un endroit et l'a oublié ailleurs (AM-1, LE-8, LE-9). Les critères livrés : trois fois, une spec s'est contredite (AM-2, LE-6, LE-11) ; un test qui calcule l'exemple de chaque critère inscrit fait voir une règle que son exemple dément (LE-6), et, pour le reste, le passage croisé d'AM-2 demeure |
| `verifier` | Un `Cargo.lock` en retard sur les `Cargo.toml`, dont il dépose alors la résolution en artefact, `cargo-lock` ; un code Rust mal formaté, un avertissement de `clippy`, un test rouge (le noyau, le cahier, le stockage, la synchronisation, les commandes et les services, et l'interface, dont `clippy` sur son code de test et pour WebAssembly) | `cargo update --workspace`, `cargo fmt`, `cargo clippy`, `cargo test` ; `cargo clippy -p stockage --lib -- -D warnings`, qui compile le stockage sans `pannes`, tel que l'établi le recevra ; `cargo clippy -p services --lib -- -D warnings`, qui compile les services sans `en-memoire`, tels que l'établi les recevra | Le noyau calcule la théorie et dessine les pages : une erreur y fausse tous les rendus. Le stockage et la synchronisation gardent le fonds (H07). Le `Cargo.lock` se résout en CI, et se tient ainsi à jour sans cargo local. La chaîne est épinglée par `rust-toolchain.toml`. Son `timeout-minutes` est provisoire, à ramener à deux fois un passage mesuré à froid (LE-29 de `modele-projet`) |
| `windows` | Un avertissement de `clippy` ou un test rouge sur le stockage et la synchronisation, sous Windows (H07) ; un établi qui ne se construit pas en installateur Windows (NSIS), un avertissement de `clippy` ou un test rouge sur l'établi, ou un cahier des tables qui ne s'engendre pas ; il dépose en artefacts l'installateur, `etabli-installateur`, et le cahier des tables daté du commit construit (sur une PR, le commit de fusion d'essai de GitHub, donc du jour du passage), `cahier-des-tables` | `cargo clippy -p stockage -p synchro`, `cargo test -p stockage -p synchro`, `cargo tauri build`, `cargo clippy -p etabli`, `cargo test -p etabli`, `cargo run -p cahier --release` ; `cargo clippy -p stockage --lib -- -D warnings`, qui compile le stockage sans `pannes`, tel que l'établi le recevra | H07 : le stockage et la synchronisation doivent tenir sous Windows, où le renommage suit sa voie propre ([§ 31 · 2.1](docs/31-persistance-et-synchronisation.md#21-sur-le-disque)), et `clippy` n'y voit qu'ici le code propre à Windows. H10 : l'application doit s'ouvrir sur un PC Windows, et seule une construction sur Windows le prouve. Le cahier des tables (REN-01, REN-07) se juge sur la tablette et sur papier : l'artefact le met entre les mains de la maîtrise d'ouvrage. Son `timeout-minutes` vaut deux fois la durée du premier passage mesuré (LE-29 de `modele-projet`) |
| `mutants` | Un mutant du noyau (la théorie, la palette, le clavier et la guitare, et les règles des données : la compaction, l'accès d'un document, le démarrage, la version de schéma, la teneur, la comparaison des copies et des lignées, les identifiants, les gestes et les règles d'une idée) qu'aucun test de ces modules ne tue, dans les lignes que la PR modifie ; seulement sur une PR. Le passage complet a lieu chaque lundi et à la demande, par `mutants-complet.yml` | `cargo mutants -p noyau --in-diff` (`.cargo/mutants.toml`) | La famille 3 des garde-fous de la méthode : la mutation des tests du domaine. Un test qui reste vert quand le code est faussé ne protège rien |

`contenu`, sauf ses contrôles sur les noms de fichiers (fichiers audio, MIDI ou partitions ;
documents et archives d'un fonds), et `documentation` ignorent les chemins gérés par le modèle
(`.githooks/`, `scripts/modele/`, `docs/modele/`, `.github/modele/`, `.github/workflows/modele-*.yml`,
`.claude/skills/consigner/`) : ils sont l'affaire de `socle`.

Pas de matrice, pas de release. On ajoute une porte à la fois, quand il y a du code à protéger
([§ 70 · 8](docs/70-qualite-et-verification.md#8-les-portes-de-ci)).

Les lancer avant de pousser :

```bash
bash scripts/verif-confidentialite.sh
node scripts/verif-liens.mjs
node --test scripts/verif-faits.test.mjs
node scripts/verif-faits.mjs
npm install --no-save --no-package-lock @mermaid-js/mermaid-cli@12.0.0   # une fois
npx --no-install puppeteer browsers install chrome    # une fois, ou CHROME_PATH=…
node scripts/verif-diagrammes.mjs
cargo fmt --all --check
cargo clippy -p noyau -p cahier -p stockage -p synchro -p commandes -p services --all-targets -- -D warnings
cargo clippy -p stockage --lib -- -D warnings
cargo clippy -p services --lib -- -D warnings
cargo clippy -p interface --all-targets -- -D warnings
cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings
cargo test -p noyau -p cahier -p interface -p stockage -p synchro -p commandes -p services
```

**Une porte se corrige par le document qu'elle refuse**, pas en l'affaiblissant à son
premier faux positif.

## Protection de `main`

Visée : force-push refusé, suppression refusée, revue **non** requise.

Le dépôt est public depuis sa naissance, le 2026-11-01 : la règle de protection du modèle
(`.github/modele/ruleset-main.json`) y est posée. Tout commit qui arrive sur `main` est la
tête d'une PR dont les contrôles requis sont verts. Jusqu'au 2026-11-01, la conception vivait
dans un dépôt privé, aujourd'hui `pupitre-donnees`, où GitHub refusait cette règle sans
abonnement payant.

La règle du dépôt, `branche-principale`, ne vit que sur GitHub : ce paragraphe la dit dans le
dépôt, et la porte `documentation` l'y compare. Depuis le 2026-11-12, après la mise à jour
vers la v0.10.0 du modèle, elle requiert six contrôles : `socle / secrets`,
`socle / confidentialite`, `texte-pr`, `verifier`, `windows` et `mutants`. `contenu` et
`documentation` n'y sont pas requis. La règle du modèle, d'où elle vient, requiert les deux
contrôles du socle et `texte-pr`.

S'y ajoutent les deux garde-fous du modèle
([`docs/modele/methode.md`](docs/modele/methode.md), « Les garde-fous ») :

- le hook `pre-push` (`.githooks/pre-push`), qui refuse **tout envoi** vers `main` :
  création, mise à jour, force-push, suppression. Il est actif une fois le dépôt cloné et
  `git config core.hooksPath .githooks` posé ;
- le garde de Claude, branché dans `.claude/settings.json`, qui refuse aussi, entre autres, un
  envoi vers `main`, `--no-verify` et toute modification de `core.hooksPath` autre que sa
  réparation, `git config core.hooksPath .githooks`.

## Ce qui n'entre jamais dans ce dépôt

Paroles réelles, audio, MIDI, exports de mémoire, adresses e-mail, prénoms de proches,
noms d'artistes de référence, jetons et clés, chemins absolus d'une machine — et le nom
d'une personne réelle, nulle part dans le dépôt. Deux exceptions, que le dépôt ne choisit pas : les
fichiers gérés par le modèle (`docs/modele/methode.md`, « Les fichiers gérés par le modèle »), qui
nomment la maîtrise d'ouvrage et que chaque mise à jour réécrit, et le nom du compte `EdouardZemb`,
que portent `LICENSE` et chaque adresse du dépôt.

Les règles musicales et la doctrine esthétique entrent ici **sous forme de règles**, avec
des exemples inventés, jamais sous forme d'extraits de chansons réelles.
