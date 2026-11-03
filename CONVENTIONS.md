# Conventions

Volontairement minces. Ce dépôt est d'abord un référentiel documentaire, lu sur un écran
d'ordinateur, un téléphone et une tablette à encre. Tout cérémonial ajouté ici doit se
justifier par un incident réel, pas par une bonne pratique générale.

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
  réattribué ; le [glossaire](docs/80-glossaire.md#le-projet) les recense.
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

- [ ] Rien de personnel n'entre dans le dépôt (paroles réelles, audio, e-mail, prénom, clé, chemin de machine).
- [ ] La documentation parle du persona, jamais d'une personne réelle ; les exemples sont inventés.
- [ ] Si une décision est prise ici, elle est écrite en ADR, pas seulement dans la PR.

## Portes de CI

**Sept, aujourd'hui.** Chacune marque la PR en échec. Celles que la règle de protection exige
sont dites à un seul endroit, plus bas, à « Protection de `main` ». Requise ou non, une porte
rouge arrête la fusion : on ne fusionne pas une PR rouge.

| Porte | Ce qu'elle refuse | Script | Pourquoi elle existe |
|---|---|---|---|
| `socle / secrets` | Un secret dans un commit ou dans l'historique | Fournie par le modèle (gitleaks) | Un secret poussé est un secret à changer |
| `socle / confidentialite` | Une donnée d'identité dans le diff, les messages de commit ou le texte de la PR | [`scripts/modele/confidentialite.py`](scripts/modele/confidentialite.py) | Le modèle protège tous ses dépôts de la même façon |
| `contenu` | Une donnée personnelle dans les fichiers du dépôt : e-mail, secret, export de mémoire, fichier audio ou MIDI, chemin absolu de machine | [`verif-confidentialite.sh`](scripts/verif-confidentialite.sh) | Le dépôt ne doit rien dire de personne. Nommée `contenu` pour ne pas se confondre avec `socle / confidentialite` |
| `documentation` | Un diagramme qui ne se rend pas ; un lien interne ou une ancre qui ne mène nulle part | [`verif-diagrammes.mjs`](scripts/verif-diagrammes.mjs), [`verif-liens.mjs`](scripts/verif-liens.mjs) | Pendant la refonte d'octobre 2026, des diagrammes se sont cassés sur leur syntaxe ; sur GitHub, un diagramme cassé s'affiche comme un message d'erreur |
| `verifier` | Un code Rust mal formaté, un avertissement de `clippy`, un test rouge (le noyau, le cahier, l'interface, dont `clippy` sur son code de test et pour WebAssembly) | `cargo fmt`, `cargo clippy`, `cargo test` | Le noyau calcule la théorie et dessine les pages : une erreur y fausse tous les rendus. La chaîne est épinglée par `rust-toolchain.toml`. Son `timeout-minutes` est provisoire, à ramener à deux fois un passage mesuré à froid (LE-29 de `modele-projet`) |
| `windows` | Un établi qui ne se construit pas en installateur Windows (NSIS), un avertissement de `clippy` sur l'établi, ou un cahier des tables qui ne s'engendre pas ; il dépose en artefacts l'installateur, `etabli-installateur`, et le cahier des tables daté du commit construit (sur une PR, le commit de fusion d'essai de GitHub, donc du jour du passage), `cahier-des-tables` | `cargo tauri build`, `cargo clippy -p etabli`, `cargo run -p cahier --release` | H10 : l'application doit s'ouvrir sur un PC Windows, et seule une construction sur Windows le prouve. Le cahier des tables (REN-01, REN-07) se juge sur la tablette et sur papier : l'artefact le met entre les mains de la maîtrise d'ouvrage. Son `timeout-minutes` vaut deux fois la durée du premier passage mesuré (LE-29 de `modele-projet`) |
| `mutants` | Un mutant du noyau (la théorie, la palette, le clavier et la guitare) qu'aucun test de ces modules ne tue, dans les lignes que la PR modifie ; seulement sur une PR. Le passage complet a lieu chaque lundi et à la demande, par `mutants-complet.yml` | `cargo mutants -p noyau --in-diff` (`.cargo/mutants.toml`) | La famille 3 des garde-fous de la méthode : la mutation des tests du domaine. Un test qui reste vert quand le code est faussé ne protège rien |

`contenu` et `documentation` ignorent les chemins gérés par le modèle (`.githooks/`,
`scripts/modele/`, `docs/modele/`, `.github/modele/`, `.github/workflows/modele-*.yml`,
`.claude/skills/consigner/`) : ils sont l'affaire de `socle`.

Pas de matrice, pas de release. On ajoute une porte à la fois, quand il y a du code à protéger
([§ 70 · 8](docs/70-qualite-et-verification.md#8-les-portes-de-ci)).

Les lancer avant de pousser :

```bash
bash scripts/verif-confidentialite.sh
node scripts/verif-liens.mjs
npm install --no-save --no-package-lock @mermaid-js/mermaid-cli@12.0.0   # une fois
npx --no-install puppeteer browsers install chrome    # une fois, ou CHROME_PATH=…
node scripts/verif-diagrammes.mjs
cargo fmt --all --check
cargo clippy -p noyau -p cahier --all-targets -- -D warnings
cargo clippy -p interface --all-targets -- -D warnings
cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings
cargo test -p noyau -p cahier -p interface
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

La règle du modèle ne requiert que les deux contrôles du socle, `socle / secrets` et
`socle / confidentialite`. Celle du dépôt, `branche-principale`, requiert aussi `verifier`,
`windows` et `mutants` depuis le 2026-11-03, après INC-0. Elle ne vit que sur GitHub : c'est
ce paragraphe qui la dit dans le dépôt. `contenu` et `documentation` n'y sont pas requis.

S'y ajoutent les deux garde-fous du modèle
([`docs/modele/methode.md`](docs/modele/methode.md), « Les garde-fous ») :

- le hook `pre-push` (`.githooks/pre-push`), qui refuse **tout envoi** vers `main` :
  création, mise à jour, force-push, suppression. Il est actif une fois le dépôt cloné et
  `git config core.hooksPath .githooks` posé ;
- le garde de Claude, branché dans `.claude/settings.json`, qui refuse aussi un envoi vers
  `main`, `--no-verify` et toute modification de `core.hooksPath`.

## Ce qui n'entre jamais dans ce dépôt

Paroles réelles, audio, MIDI, exports de mémoire, adresses e-mail, prénoms de proches,
noms d'artistes de référence, jetons et clés, chemins absolus d'une machine — et le nom
d'une personne réelle, nulle part dans le dépôt.

Les règles musicales et la doctrine esthétique entrent ici **sous forme de règles**, avec
des exemples inventés, jamais sous forme d'extraits de chansons réelles.
