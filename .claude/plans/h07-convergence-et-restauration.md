# H07 · deux répliques convergent, la copie restaurée relit ses captations — plan court

> **Pour les agents :** compétence requise : `superpowers:subagent-driven-development` (recommandée) ou
> `superpowers:executing-plans`, tâche par tâche. Les pas se cochent (`- [ ]`).

**But :** éprouver en CI, sous Linux et sous Windows, le format de l'ADR-023 et le cœur de la couche de
synchronisation. Deux répliques du fonds, modifiées hors ligne, se réunissent par le protocole de
synchronisation d'Automerge et convergent ; une copie de la racine des données, restaurée dans un dossier
neuf, relit chaque captation contre son empreinte. Le code reste : le plan du lot 1 part de lui.

**Architecture :** deux crates naissent hors d'`etabli`. `stockage` tient la racine des données et son
verrou, le document du fonds sur fichiers (un fichier complet, des fichiers d'ajout nommés par leur
empreinte, chaque fichier écrit à côté puis renommé, la compaction) et les captations. `synchro` est le
cœur de la couche de synchronisation : il compare deux copies, puis porte les messages du protocole
d'Automerge, sans réseau. Les règles qui décident (quels fichiers une compaction retire, quelles copies
s'échangent) sont des fonctions pures du noyau, que la mutation éprouve. L'essai ne touche ni `etabli`
ni `interface`.

**Pile :** Rust 1.99.1, édition 2024 ; `automerge` 0.12.0, sans fonction ; `sha2` 0.11.1 ; tests :
`proptest` 1.12.0 ; `cargo-mutants` 27.1.0, en CI.

**Spec :** § 40 · 3 (« Hors d'`etabli` »), § 40 · 9 (lot 1, H07) et § 40 · 11 (la ligne H07) ; § 31 · 2.1
à 2.5 (le § 2.4 pour la version de schéma 1 et les conteneurs nés avec leur objet) ; § 30 · 2 (les
identités que l'échange compare) ; § 30 · 9.1 et 9.2 (l'idée et la prise du schéma provisoire) ;
§ 31 · 4.4, § 31 · 6.1, § 31 · 10 (en partie, les tests 3, 4, 8 et 9) ; ADR-003, ADR-022, ADR-023
(décisions 1 à 4 et 8 ; 5 et 6 en partie) ; § 41 · 6 ; § 70 · 1 (« Les règles qui décident des
données »), I14, I28, I30 (le dossier jetable des tests seulement). L'issue : #16.

---

## Les décisions de ce plan

1. **Les crates hors d'`etabli`** (§ 40 · 3). L'essai fait naître les deux premières ; le lot 1, les deux
   autres.

   | Crate | Chemin | Ce qu'elle porte | Dépend de |
   |---|---|---|---|
   | `stockage` | `crates/stockage` | La racine des données et son verrou ; les adaptateurs `Documents` et `Captations` sur fichiers, au format de l'ADR-023 ; côté natif seulement | `noyau`, `synchro`, `automerge`, `sha2` |
   | `synchro` | `crates/synchro` | Le cœur de la couche de synchronisation : comparer deux copies, porter le protocole d'Automerge ; au lot 3, lire les conflits et les relever, selon la règle du noyau (§ 70 · 1). Aucun transport | `noyau`, `automerge` |
   | `services` | `crates/services` | Les services d'application (opérations nommées, requêtes) et les adaptateurs en mémoire, qui se compilent aussi en WebAssembly ; le plan du lot 1 dit comment une application installée les écarte | `noyau`, `commandes`, `automerge` ; pour `wasm32-unknown-unknown`, `automerge` avec sa fonction `wasm` (ci-dessous) |
   | `commandes` | `crates/commandes` | Les types des commandes : arguments, réponses, vues. Ni `automerge`, ni services | `serde` |

   Au lot 1, `stockage` dépend aussi de `services`, dont il réalise les adaptateurs, et `etabli` les
   assemble. `automerge` 0.12.0 tire sans condition `getrandom` 0.4, qui ne se compile pour le
   navigateur qu'avec son moteur `wasm_js` : sans la fonction `wasm` d'`automerge`, qui l'active, rien
   de ce qui tire `automerge` ne se compile en WebAssembly. Le README de `getrandom` déconseille de
   l'activer sans condition dans une bibliothèque : le plan du lot 1 la pose sous
   `[target.'cfg(all(target_arch = "wasm32", target_os = "unknown"))'.dependencies]` de `services`, ou
   dans la construction navigateur, et porte au budget du § 41 · 6, pour cette construction,
   `automerge` et ce que sa fonction `wasm` tire hors de la ligne « Application ». Le transport du
   lot 3, WebSocket en TLS, se branche sur `synchro` : sa PR de conception dit s'il y entre ou s'il vit
   à côté. Si H07 échoue, samod remplace `synchro` et la part des documents de
   `stockage` ; les captations restent.
2. **Les jobs.** L'essai tourne dans `verifier` (Linux) et dans `windows`, sans job de plus. `windows`
   passe `stockage` et `synchro` à `clippy` et à leurs tests avant de construire l'installateur : `clippy`
   y voit le code propre à Windows (`#[cfg(windows)]`), que `verifier` ne compile pas.
3. **Le `Cargo.lock`, résolu par la CI.** Le poste ne lance pas cargo. Une étape de `verifier` lance
   `cargo update --workspace`, échoue si le `Cargo.lock` du dépôt en diffère, et dépose celui qu'elle a
   résolu en artefact `cargo-lock`, qu'on relit puis qu'on committe. Elle reste après l'essai : un
   `Cargo.lock` en retard sur les `Cargo.toml` se voit dès lors en CI.
4. **La mutation.** `crates/noyau/src/donnees/compaction.rs` et `crates/noyau/src/donnees/copie.rs`
   entrent dans `.cargo/mutants.toml`, et `--test donnees` dans les tests que rejoue chaque mutant.
   `noyau::donnees::fonds`, qui ne porte que des constantes, n'y entre pas, ni `stockage` ni `synchro`,
   qui sont des adaptateurs. La tâche 4 remesure le passage complet, à froid, et corrige les deux
   délais.
5. **Le budget de dépendances** (§ 41 · 6). `automerge` et `sha2` y sont déjà, à la ligne « Application
   (côté natif), nœud » ; rien d'autre n'entre. `getrandom` attend le lot 1 : l'essai écrit ses
   identifiants dans les tests, et l'acteur neuf de chaque ouverture vient d'Automerge. `proptest` est une
   dépendance de test. La PR de l'essai nomme ces dépendances dans sa description.
6. **Le schéma du fonds est provisoire.** Ses clés, ses messages et sa genèse en octets vivent dans
   `noyau::donnees::fonds` : le noyau, qui calculera les changements des opérations nommées, et les deux
   adaptateurs `Documents` les y trouvent. Le plan du lot 1 les complète ; les octets de la genèse peuvent
   changer jusqu'à la `v0.3.0`, qui les fige (§ 31 · 2.3), et aucune donnée réelle n'entre avant.
7. **Ce qui est reçu s'écrit.** Ce qu'apporte un message de synchronisation n'est reçu qu'une fois son
   fichier d'ajout en place : `save_after` des *heads* d'avant la réception. Un message qui n'apporte rien
   n'écrit rien. La tâche 10 l'écrit au § 31 · 2.1.
8. **Ce que l'essai simplifie, et que le lot 1 complète.** Une écriture qui échoue laisse le document
   « à relire » : il refuse d'écrire, d'envoyer, de recevoir et de compacter jusqu'à sa réouverture, au
   lieu de se relire du disque. Un fichier d'ajout altéré, ou une modification qui attend ses
   dépendances, fait refuser l'ouverture en nommant la cause, au lieu d'ouvrir en lecture seule. La
   comparaison des copies ne connaît que l'égalité, pas les lignées remplacées. Le démarrage ne traite
   aucun `.partiel` laissé. `voisin` ne prend pas de suffixe, et `ecrire_a_cote` écrase un `.partiel`
   laissé sous le même numéro de processus. Le lot 1, qui fond les voisins laissés, lui donne le
   suffixe `-2`, `-3`… du § 31 · 2.1. Deux comportements sont écrits sans que l'essai les éprouve : une
   compaction qui échoue après une écriture ne la défait pas (tâche 8), et une réception qui change les
   heads s'écrit, qu'elle ait réussi ou non (tâche 10). Le lot 1 les éprouve.

## Ce qui doit survivre au plan

Le plan est retiré à la clôture du lot 1. Ce qu'il décide s'écrit avant, par la tâche qui le livre :

| Quoi | Où | Tâche |
|---|---|---|
| Les crates hors d'`etabli`, leurs noms et leurs dépendances | § 40 · 3 | 1 |
| Les crates nées, leurs commandes, le `Cargo.lock` résolu par la CI | `CLAUDE.md` (« Les crates », « Commandes »), `README.md` (« Code »), `CONVENTIONS.md` (`verifier`, `windows`) | 1 |
| La mutation des règles des données | `.cargo/mutants.toml`, `CONVENTIONS.md` (`mutants`) | 4 |
| Ce qui est reçu s'écrit | § 31 · 2.1 | 10 |
| I14 éprouvé, le Cargo.lock à jour | § 70 · 8 | 12 |
| Le verdict d'H07 | `CLAUDE.md` (« État »), `README.md` (« Bloquant pour démarrer ») ; négatif, par une PR à part, partie de `main`, avec la ligne `EM` du registre (« Si H07 échoue ») | 12 |
| Les simplifications de la décision 8 | Le plan du lot 1, qui les lève | — |

## Contraintes générales

- **En CI seulement.** Personne ne lance cargo, trunk ni npm sur le poste, dont le disque est presque
  plein. Chaque vérification se fait en committant, en poussant la branche et en lisant la CI.
- **Le format, sur le poste, par rustfmt seul.** Avant chaque commit,
  `rustfmt --edition 2024 <fichiers>`, sur les `.rs` que la tâche crée ou modifie. La chaîne 1.99.1 y
  est installée avec rustfmt, qui ne compile rien et n'écrit rien dans `target/` ; ce n'est pas
  `cargo fmt`, qui lance cargo. `--edition 2024` se
  passe, puisque rustfmt seul ne lit pas `Cargo.toml`.
- **La branche** `h07-essai`, dans le worktree `pupitre.worktrees/h07-essai`. La PR de l'essai s'ouvre en
  brouillon au premier envoi, puisque `ci.yml` ne tourne que sur une PR ; sa description dit
  « Refs #16 », puis « Closes #16 » au verdict (tâche 12). Aucune étiquette. L'issue
  #16 est sous la milestone « INC-1 · carnet et fonds » (§ 40 · 9).
- **Lire un passage.** `gh run list --branch h07-essai --workflow ci --limit 1` donne son numéro ;
  `gh run watch <n> --exit-status` attend son verdict ; `gh run view <n> --log-failed` montre ce qui a
  échoué ; `gh run view <n> --json jobs` donne le numéro de chaque job, et
  `gh run view <n> --job <job> --log` en montre les étapes.
- **Le rouge, puis le vert.** Un rouge se lit dans `verifier`, qui suffit : `cannot find`,
  `unresolved import`, `no method named` (E0599) ou `no variant or associated item named` à l'étape
  « Lint », ou le nom du test et son `panicked at` à l'étape « Tests ». Un rouge à « Format », qui
  précède « Lint », n'est pas le rouge attendu : `gh run view <n> --log-failed` en montre le diff ;
  corriger, committer `style: le format de rustfmt`, pousser, puis relire le rouge ou le vert attendu.
  Un vert se lit dans les deux jobs : l'étape « Tests » de `verifier` et l'étape « Tests du stockage et de la
  synchronisation (H07) » de `windows` montrent, pour chaque fichier de tests de la tâche,
  `test result: ok.` avec le nombre de tests qu'elle ajoute. `concurrency` annule le passage en cours à
  chaque envoi : on pousse le vert une fois le rouge lu, et on attend le verdict du vert avant la tâche
  suivante.
- **Les commits** : Conventional Commits en français, en-tête de 100 caractères au plus ; `test:` pour le
  rouge, `feat:` pour le vert, `style:` pour un format corrigé, `ci:`, `chore:` ou `docs:` pour le
  reste. Chaque commit : `git add` des seuls chemins du bloc « Fichiers » de la tâche, puis
  `git commit -F <scratchpad>/msg-<tâche>-<pas>.txt`. Envoi : `git push -u origin h07-essai` la
  première fois, puis `git push origin h07-essai`. Ni `--no-verify`, ni `core.hooksPath` modifié.
- **Les commandes de Rust** prennent `-p <crate>`, jamais tout l'espace de travail.
- **Le noyau** reste sans entrée-sortie et sans `automerge`.
- **Les chemins se reçoivent** (I30). `stockage` ne devine aucun chemin : ni `temp_dir`, ni variable
  d'environnement. Chaque test prend son dossier jetable par une seule fonction,
  `commun::dossier_jetable`, dans le dossier temporaire du système, et l'efface en commençant
  (`std::fs::remove_dir_all`). Le garde-fou du dossier sentinelle et le contrôle statique d'I30
  viennent au lot 1.
- **Aucune donnée réelle.** Les textes sont inventés, les identifiants écrits dans les tests, et les
  captations engendrées par le code (`commun::octets`). Aucun fichier audio, MIDI ni `.automerge` n'est
  suivi.
- **Les mots** sont ceux du glossaire : racine des données, racine du document, fichier complet, fichier
  d'ajout, voisin, genèse, lignée, captation, empreinte, prise, idée, étiquette. « Écart » nomme une chose
  du morceau (§ 30 · 5) : le code de l'essai ne l'emploie pas. À envoyer est l'état d'une captation
  (glossaire) : le code de l'essai ne l'emploie pas.
- **Les erreurs** dérivent `Debug`. Leur texte en français vient avec les commandes, au lot 1.

## Points de vigilance

Cinq cas que la conception implique et qu'un test du chemin nominal n'exercerait pas. Chacun a son test,
dans la tâche qui possède le code.

1. **Un fichier d'ajout altéré.** `load_incremental` s'arrête sans erreur au premier tronçon altéré :
   seul le contrôle de l'empreinte du nom le voit. L'ouverture le nomme, et ne retire ni ne réécrit rien ;
   la compaction ne le retire jamais. Tests : `un_ajout_altere_se_nomme_et_rien_ne_bouge` (tâche 7),
   `a_retirer_garde_un_fichier_non_conforme` (tâche 4), `la_compaction_garde_un_ajout_altere`
   (tâche 8).
2. **Ce qui est reçu, ce qui ne part pas.** Ce qu'une réplique reçoit est sur le disque avant sa
   réponse ; un message qui n'apporte rien n'écrit aucun fichier ; un document à relire n'envoie rien,
   ne reçoit rien et ne se compacte pas. Tests : `ce_qui_est_recu_se_relit_du_disque`,
   `un_message_sans_rien_de_neuf_n_ecrit_rien`, `un_document_a_relire_ne_laisse_rien_partir`
   (tâche 10).
3. **Deux copies étrangères.** D'un autre propriétaire, d'un autre document ou d'une autre lignée, elles
   ne s'échangent rien : l'échange ne s'ouvre pas, et le refus nomme la première différence. Tests :
   `comparer_rend_la_premiere_difference` (tâche 4), `deux_lignees_differentes_n_ouvrent_aucun_echange`
   (tâche 9).
4. **Une captation versée deux fois.** Un seul objet ; l'existant n'est ni réécrit ni rouvert en
   écriture ; aucun nom provisoire ne reste. Test : `verser_deux_fois_ne_fait_qu_un_objet` (tâche 3).
5. **La copie restaurée.** Elle emporte le fichier `verrou`, dont la seule présence ne dit rien : elle
   s'ouvre. Et son contrôle peut échouer : une captation altérée ou absente s'y nomme. Tests du code :
   `la_seule_presence_du_verrou_ne_dit_rien` (tâche 1), `une_captation_alteree_ou_absente_se_nomme`
   (tâche 3) ; test de l'essai : `une_captation_alteree_ou_absente_de_la_copie_se_nomme` (tâche 11).

## Les fichiers

```text
crates/noyau/src/donnees/
  mod.rs                pub mod compaction; pub mod copie; pub mod fonds;
  compaction.rs         a_l_ouverture, apres_une_ecriture, Lu, a_retirer          (muté)
  copie.rs              Copie, Difference, comparer                               (muté)
  fonds.rs              les clés du fonds, ses messages, sa genèse en octets
crates/noyau/tests/donnees.rs
crates/synchro/         Cargo.toml, src/lib.rs (Echange, Erreur), tests/echange.rs
crates/stockage/
  Cargo.toml
  src/lib.rs            Erreur, empreinte, et ce que les modules rendent publics
  src/racine_des_donnees.rs   RacineDesDonnees
  src/fichier.rs        voisin, ecrire_a_cote, rendre_durable
  src/captations.rs     Captations
  src/documents.rs      Naissance, Document
  tests/commun/mod.rs   dossier_jetable, octets, copier_l_arbre, texte_long, les gestes de test, reunir,
                        relire_le_dossier_du_fonds
  tests/racine_des_donnees.rs, fichier.rs, captations.rs, genese.rs, fonds.rs, ajouts.rs,
  tests/compaction.rs, synchronisation.rs, h07.rs
```

## Mécanismes annoncés

- `.github/workflows/ci.yml:4` — `pull_request:` — `ci.yml` tourne sur une PR, et non à l'envoi d'une autre
  branche que `main` : la PR de l'essai s'ouvre en brouillon au premier envoi (« La branche »).
- `.github/workflows/ci.yml:16` — `cancel-in-progress` — sur une PR, chaque envoi annule le passage en cours :
  on pousse le vert une fois le rouge lu, et on attend son verdict avant la tâche suivante (« Le rouge, puis
  le vert »).
- `.github/workflows/ci.yml:83` — `Swatinem/rust-cache` — rust-cache ne sauve pas le cache d'un passage
  rouge : le premier passage vert après l'entrée d'`automerge` donne les durées de `verifier` et de `windows`
  (tâche 1, pas 5 ; tâche 12, pas 1).
- `.github/workflows/ci.yml:153` — `cargo tauri build` — le job `windows` construit l'installateur ;
  `stockage` et `synchro` y passent à `clippy` et à leurs tests avant lui, et `clippy` y voit le code propre
  à Windows, que `verifier` ne compile pas (décision 2).
- `.cargo/mutants.toml:6` — `examine_globs` — la liste des modules que la mutation examine :
  `compaction.rs` et `copie.rs` y entrent (décision 4 ; tâche 4, pas 3).
- `.cargo/mutants.toml:24` — `additional_cargo_test_args` — les tests que rejoue chaque mutant :
  `--test donnees` y entre (décision 4).
- `.github/workflows/mutants-complet.yml:8` — `workflow_dispatch` — le passage complet de la mutation se
  lance à la demande, sur la branche de l'essai : la remesure à froid (tâche 4, pas 5).
- `crates/noyau/tests/commun/mod.rs:5` — `#![allow(dead_code)]` — chaque fichier de tests est une crate à
  part, où une aide inemployée lèverait `dead_code`, que `-D warnings` refuse : le `tests/commun/mod.rs` du
  stockage commence de même (tâche 1).
- `docs/41-pile-technique.md:289` — `Application (côté natif), nœud` — `automerge` est déjà au budget de
  dépendances (§ 41 · 6) : il entre sans ligne nouvelle (décision 5).
- `docs/41-pile-technique.md:296` — `application (côté natif), nœud` — `sha2` est déjà au budget de
  dépendances, au noyau et au côté natif : il entre sans ligne nouvelle (décision 5).
- `crates/stockage/src/racine_des_donnees.rs:64` — `TryLockError::WouldBlock` — `File::try_lock` ne refuse
  que si un autre processus tient le verrou : la seule présence du fichier `verrou`, qu'emporte une copie, ne
  dit rien (tâche 1 ; vigilance 5).
- `crates/stockage/src/fichier.rs:110` — `fs::rename(de, vers)` — `std::fs::rename` remplace au besoin le
  fichier qui porte déjà ce nom, sous Windows comme ailleurs (tâche 2).
- `crates/stockage/tests/commun/mod.rs:54` — `fs::remove_dir_all` — sous Windows, `remove_dir_all` efface
  aussi un fichier en lecture seule, et sous Linux seuls les droits du dossier comptent : le test de la
  tâche 3 ne lève pas la lecture seule de l'objet.
- `crates/stockage/src/documents.rs:270` — `save_after` — `save_after` des heads d'avant rend les seules
  modifications faites depuis : le contenu d'un fichier d'ajout, d'une écriture comme d'une réception
  (tâches 7 et 10 ; décision 7).
- `crates/stockage/src/documents.rs:493` — `load_incremental` — `load_incremental` s'arrête sans erreur au
  premier tronçon altéré : seul le contrôle de l'empreinte du nom voit un fichier d'ajout altéré
  (vigilance 1).
- `crates/stockage/src/documents.rs:401` — `ActorId::random()` — chaque ouverture tire un acteur neuf,
  qu'Automerge tire lui-même : `getrandom` attend le lot 1 (décision 5 ; tâche 6).
- `crates/stockage/tests/commun/mod.rs:125` — `comprime le fichier complet` — `save` comprime le fichier
  complet, `save_after` ne comprime pas les fichiers d'ajout : le fonds des tâches 7 et 8 naît d'un texte
  long (`texte_long`).
- `crates/synchro/src/lib.rs:41` — `generate_sync_message` — il rend `None` quand une copie n'a plus rien à
  dire : `reunir` s'achève quand aucune des deux n'envoie (tâches 9 et 10).
- `crates/synchro/src/lib.rs:48` — `Message::decode` — il refuse des octets qui ne sont pas un message, avant
  que le document ne reçoive quoi que ce soit (tâche 9).

## Outils et cas limites

- `load_incremental` — un fichier d'ajout altéré se charge sans erreur jusqu'à son premier tronçon altéré —
  `un_ajout_altere_se_nomme_et_rien_ne_bouge` (tâche 7), `la_compaction_garde_un_ajout_altere` (tâche 8)
- `File::try_lock` — un fichier `verrou` présent que nul processus ne tient, comme en emporte une copie —
  `la_seule_presence_du_verrou_ne_dit_rien` (tâche 1)
- `std::fs::rename` — sous Windows, un fichier qui porte déjà le nom visé —
  `ecrire_a_cote_remplace_un_fichier_existant` (tâche 2), dans le job `windows`
- `std::fs::rename` — une cible qui est un dossier non vide, que le renommage refuse —
  `un_renommage_refuse_retire_le_voisin` (tâche 2)
- `std::fs::remove_dir_all` — un objet mis en lecture seule, que Windows efface quand même —
  `verser_deux_fois_ne_fait_qu_un_objet` (tâche 3)
- `save` et `save_after` — le fichier complet comprimé, les fichiers d'ajout non —
  `chaque_ecriture_ajoute_un_fichier_nomme_par_son_empreinte` et `chaque_ouverture_tire_un_acteur_neuf`
  (tâche 7), sur un fonds né de `texte_long()`
- `merge` d'Automerge — deux documents sans genèse commune, chacun avec son propre `idees` —
  `sans_genese_les_idees_d_un_cote_sortent_de_la_vue` (tâche 5)
- `generate_sync_message` — deux copies identiques échangent encore un message, qui n'apporte rien —
  `un_message_sans_rien_de_neuf_n_ecrit_rien` (tâche 10)
- `Message::decode` — des octets quelconques — `un_message_illisible_est_refuse_et_le_document_ne_bouge_pas`
  (tâche 9)
- `cargo update --workspace` — un `Cargo.lock` en retard sur les `Cargo.toml`, que la commande réécrit sans
  échouer — l'étape « Cargo.lock à jour » de `verifier`, rouge au pas 2 de la tâche 1, puis verte

## Revues

- la revue de chaque tâche, par une autre session : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11

---

### Tâche 1 : les deux crates, la racine des données et son verrou

**Fichiers :**
- Créer : `crates/stockage/Cargo.toml`, `crates/stockage/src/lib.rs`,
  `crates/stockage/src/racine_des_donnees.rs`, `crates/stockage/tests/racine_des_donnees.rs`,
  `crates/stockage/tests/commun/mod.rs`, `crates/synchro/Cargo.toml`,
  `crates/synchro/src/lib.rs` (son seul commentaire de tête, jusqu'à la tâche 9)
- Modifier : `Cargo.toml` (les deux membres), `Cargo.lock` (par l'artefact), `.github/workflows/ci.yml`
  (`verifier`, `windows`), `CLAUDE.md` (« Les crates », « Commandes »), `README.md` (la ligne « Code »),
  `CONVENTIONS.md` (les lignes `verifier` et `windows`, « Les lancer avant de pousser »),
  `docs/40-architecture.md` § 3 (« Hors d'`etabli` »)

**Interfaces — produit :**

```rust
// stockage
pub struct RacineDesDonnees { /* son chemin ; le fichier `verrou`, ouvert et verrouillé */ }
impl RacineDesDonnees {
    /// § 31 · 2.1 : crée au besoin le dossier, prend le verrou de `verrou`
    /// (`OpenOptions` : `create`, `write`, `truncate(false)` ; puis `File::try_lock`), et seulement
    /// ensuite crée `documents/` et `captations/`.
    pub fn ouvrir(chemin: &Path) -> Result<RacineDesDonnees, Erreur>;
    pub fn chemin(&self) -> &Path;
}
#[derive(Debug)]
pub enum Erreur {
    /// `TryLockError::WouldBlock` : un autre processus tient la racine des données.
    RacineDesDonneesTenue,
    Disque { chemin: PathBuf, erreur: std::io::Error },
}
// tests/commun/mod.rs
pub fn dossier_jetable(nom: &str) -> PathBuf;   // temp_dir()/pupitre-stockage-<nom>-<processus>, effacé
                                                 // par std::fs::remove_dir_all, puis recréé
/// Un générateur congruentiel ou xorshift semé par `graine`, dont on garde les bits hauts : une
/// suite déterministe, sans période courte, sans fichier.
pub fn octets(graine: u8, longueur: usize) -> Vec<u8>;
pub fn copier_l_arbre(de: &Path, vers: &Path);
```

`tests/commun/mod.rs` commence, comme celui du noyau, par `// Chaque fichier de test n'emploie qu'une
partie des aides.` et `#![allow(dead_code)]` : chaque fichier de tests est une crate à part, où une
aide inemployée lèverait `dead_code`, que `-D warnings` refuse.

`crates/stockage/Cargo.toml` porte `automerge = "0.12.0"`, `noyau` et `synchro` par leur chemin, et
`sha2 = { version = "0.11.1", default-features = false }`, comme le noyau ; en test,
`proptest = "1.12.0"`. `crates/synchro/Cargo.toml` porte `automerge = "0.12.0"` et `noyau`. Toutes les
dépendances de l'essai entrent ici : seule cette tâche change le `Cargo.lock`.

`ci.yml`, job `verifier` : après `rust-cache`,

```yaml
      # Un Cargo.lock en retard sur les Cargo.toml échoue ici, et la CI dépose celui qu'elle a
      # résolu en artefact, à relire puis à committer : le tenir à jour ne demande aucun cargo local.
      - name: Cargo.lock à jour
        run: |
          cargo update --workspace
          git diff --exit-code Cargo.lock
      - name: Déposer le Cargo.lock résolu
        if: failure()
        uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
        with:
          name: cargo-lock
          path: Cargo.lock
```

puis `-p stockage -p synchro` ajoutés à l'étape « Lint » (`cargo clippy -p noyau -p cahier -p stockage
-p synchro --all-targets -- -D warnings`) et à l'étape « Tests » (`cargo test -p noyau -p cahier
-p interface -p stockage -p synchro`). Job `windows` : après `rust-cache`, avant « Installer trunk »,

```yaml
      # H07 sous Windows, où le renommage suit sa voie propre (§ 31 · 2.1). clippy y voit le
      # code propre à Windows, que le job verifier ne compile pas.
      - name: Lint du stockage et de la synchronisation
        run: cargo clippy -p stockage -p synchro --all-targets -- -D warnings
      - name: Tests du stockage et de la synchronisation (H07)
        run: cargo test -p stockage -p synchro
```

Au § 40 · 3, la phrase « Le plan court d'H07, qui écrit le premier de ces adaptateurs, fixe les crates et
leurs noms. » devient la décision 1, en prose : les quatre noms, ce que chacune porte, ses dépendances, et
qu'H07 fait naître `stockage` et `synchro`. `CLAUDE.md` ajoute deux puces à « Les crates »
(« - `stockage` (`crates/stockage`) : la racine des données, les documents et les captations sur fichiers,
côté natif ; », « - `synchro` (`crates/synchro`) : le cœur de la couche de synchronisation, sans
réseau ; »), et la ligne « Code » du README les nomme en code. Aucun compte ne s'écrit devant « crates ».
Dans `CONVENTIONS.md`, la ligne `verifier` refuse aussi « un `Cargo.lock` en retard sur les
`Cargo.toml` » et couvre le stockage et la synchronisation ; la ligne `windows`, leur lint et leurs
tests sous Windows (H07). « Les lancer avant de pousser », les « Commandes » de `CLAUDE.md` et le
commentaire du job `verifier` prennent les `-p` nouveaux ; les « Commandes » disent l'artefact
`cargo-lock`.

- [ ] **Pas 1 : écrire les tests** (`tests/racine_des_donnees.rs`), avec les fichiers et les changements
  ci-dessus, `src/lib.rs` ne déclarant encore rien :
  - `ouvrir_une_racine_des_donnees_neuve_cree_ses_dossiers_et_son_verrou` : `documents/`, `captations/`
    et `verrou` existent ;
  - `une_racine_des_donnees_tenue_se_refuse` : une seconde `ouvrir` du même chemin rend
    `Erreur::RacineDesDonneesTenue` ; la première fermée, elle réussit ;
  - `la_seule_presence_du_verrou_ne_dit_rien` : un `verrou` écrit d'avance, comme en emporte une copie,
    n'empêche pas d'ouvrir.
- [ ] **Pas 2 : le Cargo.lock.** Committer (`test: la racine des données et son verrou`), pousser, ouvrir
  la PR en brouillon :
  `gh pr create --draft --base main --head h07-essai --title "feat: l'essai H07, convergence et restauration" --body-file <scratchpad>/h07-pr.md`.
  Ce titre devient l'en-tête du squash (tâche 12). Attendu : `verifier` rouge à
  « Cargo.lock à jour ». `gh run download <n> -n cargo-lock -D <dossier du scratchpad>`, puis le copier
  sur `Cargo.lock`. Relire `git diff --stat Cargo.lock` et le diff : `stockage`, `synchro`, `automerge` et
  ses dépendances y entrent, aucune version déjà épinglée ne change. Committer
  (`chore: le Cargo.lock de stockage et de synchro`), pousser.
- [ ] **Pas 3 : voir échouer.** Attendu : `verifier` rouge à « Lint », `RacineDesDonnees` introuvable.
- [ ] **Pas 4 : écrire `racine_des_donnees.rs`.**
- [ ] **Pas 5 : voir passer.** Committer (`feat: la racine des données et son verrou`), pousser.
  Attendu : `verifier`, `windows` et `documentation` verts ; dans les deux premiers,
  `tests/racine_des_donnees.rs` : `test result: ok. 3 passed`. Relever, pour la tâche 12, les durées
  de `verifier` et de `windows` sur ce passage (`gh run view <n> --json jobs`, `startedAt` et
  `completedAt`) : c'est le premier passage complet après l'entrée d'`automerge`, puisque rust-cache
  ne sauve pas le cache d'un passage rouge.

### Tâche 2 : écrire à côté, puis renommer

**Fichiers :** créer `crates/stockage/src/fichier.rs`, `crates/stockage/tests/fichier.rs` ; modifier
`src/lib.rs`.

**Interfaces — produit :**

```rust
// stockage::fichier (§ 31 · 2.1)
/// `<nom>.<processus>.partiel`, dans le dossier de `chemin`.
pub fn voisin(chemin: &Path) -> PathBuf;
/// Écrit le voisin, le synchronise (`sync_all`), le ferme, le renomme en `chemin`
/// (`std::fs::rename`, qui remplace au besoin), puis rend le renommage durable.
/// En cas d'échec, retire le voisin et rend l'erreur.
pub fn ecrire_a_cote(chemin: &Path, octets: &[u8]) -> std::io::Result<()>;
/// Hors de Windows, synchronise le dossier de `chemin` ; sous Windows, rouvre `chemin` en
/// écriture, sans le tronquer, et le synchronise.
pub fn rendre_durable(chemin: &Path) -> std::io::Result<()>;
```

- [ ] **Pas 1 : écrire les tests** (`tests/fichier.rs`) :
  - `le_voisin_porte_le_numero_du_processus` : `voisin(d/complet.automerge)` vaut
    `d/complet.automerge.<std::process::id()>.partiel` ;
  - `ecrire_a_cote_ecrit_et_ne_laisse_rien_d_autre` : le fichier porte les octets, le dossier n'a que
    lui ;
  - `ecrire_a_cote_remplace_un_fichier_existant` : la voie de Windows, éprouvée dans le job `windows` ;
  - `un_renommage_refuse_retire_le_voisin` : `chemin` est un dossier qui porte un fichier ; l'erreur se
    rend, le dossier et son fichier sont intacts, aucun `.partiel` ne reste.
- [ ] **Pas 2 : voir échouer.** Committer (`test: écrire à côté, puis renommer`), pousser. Attendu :
  `verifier` rouge à « Lint », `stockage::fichier` introuvable.
- [ ] **Pas 3 : écrire `fichier.rs`**, avec `#[cfg(windows)]` et `#[cfg(not(windows))]` pour
  `rendre_durable`.
- [ ] **Pas 4 : voir passer.** Committer (`feat: écrire à côté, puis renommer`), pousser. Attendu :
  `tests/fichier.rs` : `test result: ok. 4 passed`, dans `verifier` et dans `windows`, et « Lint du
  stockage et de la synchronisation » vert dans `windows`.

### Tâche 3 : les captations

**Fichiers :** créer `crates/stockage/src/captations.rs`, `crates/stockage/tests/captations.rs` ;
modifier `src/lib.rs`, `src/racine_des_donnees.rs`.

**Interfaces — consomme :** `fichier::voisin`, `fichier::rendre_durable`. **Produit :**

```rust
// stockage
/// Le SHA-256 de `octets`, en 64 caractères hexadécimaux minuscules (`Sha256::digest`).
pub fn empreinte(octets: &[u8]) -> String;
impl RacineDesDonnees { pub fn captations(&self) -> Captations<'_>; }
pub struct Captations<'r> { /* la racine des données */ }
impl Captations<'_> {
    /// § 31 · 6.1 : range les octets sous `captations/<deux premiers caractères>/<empreinte>`,
    /// et rend l'empreinte.
    pub fn verser(&self, octets: &[u8]) -> Result<String, Erreur>;
    /// Relit une captation contre son empreinte.
    pub fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Erreur>;
    /// Les captations rangées dont le contenu trahit le nom, ou rangées hors du sous-dossier de
    /// leurs deux premiers caractères. Les noms provisoires de `captations/` n'y entrent pas.
    pub fn verifier(&self) -> Result<Vec<PathBuf>, Erreur>;
}
// Erreur += CaptationAlteree { empreinte: String }, CaptationAbsente { empreinte: String }
```

`verser`, dans cet ordre : si `captations/<xx>/<e>` existe, il rend `e` sans rien écrire. Sinon il écrit
le nom provisoire `voisin(captations/<e>)`, le synchronise, le ferme, le relit contre `e` (sinon il le
retire et rend `CaptationAlteree`) ; il crée `<xx>/` au besoin, et, hors de Windows, synchronise alors
`captations/` (`#[cfg(not(windows))]` : `rendre_durable(captations/<xx>)`, qui synchronise le dossier de
son chemin) ; il renomme, puis `rendre_durable`. Ce nom provisoire est celui de l'essai : la PR de
conception du lot 2 fixe celui d'une capture interrompue.

- [ ] **Pas 1 : écrire les tests** (`tests/captations.rs`) :
  - `l_empreinte_est_le_sha_256_en_hexadecimal` : `""` donne
    `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`, `"abc"` donne
    `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` ;
  - `une_captation_se_range_sous_son_empreinte` : pour `octets(1, 4000)` et pour zéro octet, le fichier
    `captations/<e[..2]>/<e>` porte les octets, `captations/` n'a que ses sous-dossiers, `relire(e)` les
    rend ;
  - `verser_deux_fois_ne_fait_qu_un_objet` (vigilance 4) : l'objet mis en lecture seule avant le second
    `verser`, qui réussit et rend la même empreinte ; un seul fichier, inchangé ; aucun `.partiel`. Le
    test ne lève pas la lecture seule : `commun::dossier_jetable` efface par
    `std::fs::remove_dir_all`, qui l'ignore sous Windows, et sous Linux seuls les droits du dossier
    comptent ;
  - `une_captation_alteree_ou_absente_se_nomme` : un octet changé, `relire` rend `CaptationAlteree` et
    `verifier` nomme son chemin ; une empreinte inconnue, `CaptationAbsente` ; un nom provisoire
    déposé dans `captations/`, que `verifier` ne nomme pas ; une captation déposée hors du
    sous-dossier de ses deux premiers caractères, qu'il nomme.
- [ ] **Pas 2 : voir échouer.** Committer (`test: les captations, rangées par empreinte`), pousser.
  Attendu : `verifier` rouge à « Lint », `empreinte` et `Captations` introuvables.
- [ ] **Pas 3 : écrire `captations.rs` et `empreinte`.**
- [ ] **Pas 4 : voir passer.** Committer (`feat: les captations, rangées par empreinte`), pousser.
  Attendu : `tests/captations.rs` : `test result: ok. 4 passed`, dans `verifier` et dans `windows`.

### Tâche 4 : les règles du noyau, et leur mutation

**Fichiers :** créer `crates/noyau/src/donnees/mod.rs`, `crates/noyau/src/donnees/compaction.rs`,
`crates/noyau/src/donnees/copie.rs`, `crates/noyau/tests/donnees.rs` ; modifier `crates/noyau/src/lib.rs`
(`pub mod donnees;`), `.cargo/mutants.toml`, `CONVENTIONS.md` (la ligne `mutants`), puis les délais de
`.github/workflows/ci.yml` (`mutants`) et de `.github/workflows/mutants-complet.yml`.

**Interfaces — produit :**

```rust
// noyau::donnees::compaction (§ 31 · 2.2)
pub const AJOUTS_AU_PLUS: usize = 100;
/// Point 1 : à l'ouverture, s'il y a un fichier d'ajout, ou le voisin entier du fichier complet.
pub fn a_l_ouverture(ajouts: usize, voisin_du_complet: bool) -> bool;
/// Point 1 : après une écriture, dès que les ajouts pèsent ensemble plus que le fichier
/// complet, ou passent AJOUTS_AU_PLUS.
pub fn apres_une_ecriture(octets_du_complet: u64, octets_des_ajouts: u64, ajouts: usize) -> bool;
/// Ce que l'adaptateur a lu d'un fichier d'ajout, ou du voisin du fichier complet.
pub struct Lu { pub nom: String, pub conforme: bool, pub absorbe: bool }
/// Points 3 et 4 : si le fichier complet relu mène au même point que le document en mémoire,
/// les seuls fichiers conformes à leur empreinte et absorbés, dans l'ordre reçu ; sinon aucun.
pub fn a_retirer(meme_point: bool, lus: &[Lu]) -> Vec<String>;

// noyau::donnees::copie (§ 31 · 2.5, § 30 · 2)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copie { pub proprietaire: String, pub document: String, pub lignee: String }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difference { AutreProprietaire, AutreDocument, AutreLignee }
/// Deux copies s'échangent si elles ont le même propriétaire, le même document et la même
/// lignée ; sinon, la première différence, dans cet ordre.
pub fn comparer(locale: &Copie, distante: &Copie) -> Result<(), Difference>;
```

- [ ] **Pas 1 : écrire les tests** (`crates/noyau/tests/donnees.rs`) :
  - `compacter_a_l_ouverture` : (0, faux) non ; (1, faux) oui ; (0, vrai) oui ;
  - `compacter_apres_une_ecriture` : (100, 100, 1) non, l'égalité ne suffit pas ; (100, 101, 1) oui ;
    (10 000, 10, 100) non ; (10 000, 10, 101) oui ;
  - `a_retirer_ne_retire_rien_si_le_complet_relu_mene_ailleurs` : `meme_point` faux, rien ;
  - `a_retirer_garde_un_fichier_non_conforme` (vigilance 1) : sur [a conforme et absorbé, b non conforme
    et absorbé, c conforme et non absorbé, d conforme et absorbé], [a, d] ;
  - `comparer_rend_la_premiere_difference` (vigilance 3) : égales, `Ok` ; chaque champ seul, sa
    différence ; le propriétaire et le document, `AutreProprietaire` ; le document et la lignée,
    `AutreDocument`.
- [ ] **Pas 2 : voir échouer.** Committer (`test: les règles de la compaction et des copies`), pousser.
  Attendu : `verifier` rouge à « Lint », `noyau::donnees` introuvable.
- [ ] **Pas 3 : écrire les deux modules**, et `.cargo/mutants.toml` : les deux fichiers dans
  `examine_globs`, `"--test", "donnees"` dans `additional_cargo_test_args`, ses commentaires à jour (les
  modules examinés, les tests rejoués). `CONVENTIONS.md`, ligne `mutants` : « la théorie, la palette, le
  clavier et la guitare, et les règles des données : la compaction et la comparaison des copies ».
- [ ] **Pas 4 : voir passer.** Committer (`feat: les règles de la compaction et des copies`), pousser.
  Attendu : `tests/donnees.rs` : `test result: ok. 5 passed` dans `verifier` ; `mutants` vert, sans
  `MISSED` dans son journal.
- [ ] **Pas 5 : remesurer la mutation, à froid** (§ 70 · 1).
  `gh workflow run mutants-complet.yml --ref h07-essai`, puis
  `gh run list --workflow mutants-complet --branch h07-essai --limit 1` pour son numéro. Dans le journal de l'étape rust-cache, lire « No cache found. ». Si un cache a été restauré
  (« Restored from cache key … »), retirer ceux que ce dispatch peut lire :
  `gh cache list --key v0-rust-mutants --json key,ref`, puis
  `gh cache delete <clé> --ref refs/heads/main` et `--ref refs/heads/h07-essai`, et relancer. Relever la durée du job
  (`gh run view <n> --json jobs`, `startedAt` et `completedAt`) et la ligne « N mutants tested » du
  journal. Les deux `timeout-minutes` (`mutants` de `ci.yml`, `mutants-complet.yml`) valent deux fois
  cette durée, arrondie à la minute au-dessus ; leurs commentaires disent « à froid », le compte, la
  durée, la date et la PR. Si la mesure n'a pu se prendre que sur un cache, les commentaires le disent,
  et la PR le donne comme un écart au § 70 · 1. Committer
  (`ci: les délais de la mutation, remesurés avec les règles des données`), pousser ; attendu : vert.

### Tâche 5 : la genèse du fonds

**Fichiers :** créer `crates/noyau/src/donnees/fonds.rs`, `crates/stockage/tests/genese.rs` ; modifier
`crates/noyau/src/donnees/mod.rs`.

**Interfaces — produit :**

```rust
// noyau::donnees::fonds : le schéma provisoire du fonds (décision 6)
pub const VERSION_DE_SCHEMA: &str = "version_de_schema"; // Uint, 1 à la genèse
pub const IDEES: &str = "idees";               // objet à clés : identifiant d'idée → idée
pub const PRISES: &str = "prises";             // objet à clés : empreinte → prise
pub const IDENTIFIANT: &str = "identifiant";   // Str, écrit à la naissance
pub const LIGNEE: &str = "lignee";             // Str, écrit à la naissance
pub const PROPRIETAIRE: &str = "proprietaire"; // Str, écrit à la naissance
pub const TEXTE: &str = "texte";               // d'une idée : Str
pub const ETIQUETTES: &str = "etiquettes";     // d'une idée : objet à clés, étiquette → true
pub const IDEE: &str = "idee";                 // d'une prise : Str, l'identifiant de son idée
pub const FORMAT: &str = "format";             // d'une prise : Str
pub const MESSAGE_DE_LA_GENESE: &str = "Genèse du fonds";
pub const MESSAGE_DE_LA_NAISSANCE: &str = "Fonds créé";
pub const ACTEUR_DE_LA_GENESE: [u8; 16] = *b"pupitre-genese-1";
/// La genèse du fonds (§ 31 · 2.3) : une modification Automerge, en octets, sur une ligne,
/// telle que le test l'imprime.
#[rustfmt::skip]
pub const GENESE: &[u8] = &[]; // le pas 3 y recopie la ligne que le test imprime
```

La recette, dans `tests/genese.rs`. Son corps est donné, puisque les octets en dépendent :

```rust
fn genese_par_la_recette() -> Vec<u8> {
    let mut doc = Automerge::new().with_actor(ActorId::from(ACTEUR_DE_LA_GENESE));
    let mut tx = doc.transaction();
    tx.put(ROOT, VERSION_DE_SCHEMA, 1u64).unwrap();
    tx.put_object(ROOT, IDEES, ObjType::Map).unwrap();
    tx.put_object(ROOT, PRISES, ObjType::Map).unwrap();
    let options = CommitOptions::default().with_message(MESSAGE_DE_LA_GENESE).with_time(0);
    let (h, _) = tx.commit_with(options);
    doc.get_change_by_hash(&h.unwrap()).unwrap().raw_bytes().to_vec()
}
```

- [ ] **Pas 1 : écrire `fonds.rs`, puis les tests.** `crates/noyau/src/donnees/fonds.rs` tel que le
  bloc « Interfaces » le donne, `GENESE` vide, et `pub mod fonds;` dans `donnees/mod.rs`, pour que le
  rouge tombe à « Tests », où la recette s'imprime. Puis `tests/genese.rs` :
  - `la_genese_se_reconstruit_a_l_octet` : la recette rend `GENESE` ; sinon le message d'échec imprime
    les octets de la recette, sur une ligne, comme un tableau Rust (`{:?}`) ;
  - `la_genese_pose_la_version_de_schema_1_et_les_conteneurs` : `Change::try_from(GENESE)` appliqué à
    `Automerge::new()` par `apply_changes` ; la racine du document n'a que les trois clés, la version de
    schéma `Uint(1)`, `idees` et `prises` en objets à clés ; la modification a pour acteur
    `ACTEUR_DE_LA_GENESE`, pour heure 0, pour message `MESSAGE_DE_LA_GENESE`, et aucune dépendance ;
  - `avec_la_genese_les_idees_des_deux_cotes_restent` et `sans_genese_les_idees_d_un_cote_sortent_de_la_vue`
    (§ 2.3) : deux documents écrivent chacun une idée (`put_object(idees, "idee-a", ObjType::Map)`
    d'un côté, `"idee-b"` de l'autre, sans `commun::noter`, qui n'arrive qu'à la tâche 6), puis
    `merge` ; partis de `GENESE`, les deux idées sont sous `idees` ; chacun créant son `idees`, l'une
    n'est plus dans l'objet que présente Automerge.
- [ ] **Pas 2 : voir échouer, et lire les octets.** Committer (`test: la genèse du fonds`), pousser, et
  attendre les deux jobs. Attendu, dans `verifier` et dans `windows` : rouges,
  `la_genese_se_reconstruit_a_l_octet`, qui imprime les octets, et les deux tests qui lisent `GENESE` ;
  vert, `sans_genese_…`, qui décrit Automerge et non notre code. Relever la ligne imprimée dans chacun
  (`gh run view <n> --job <job> --log`) : les deux doivent être identiques. Sinon, H07 échoue
  (« Si H07 échoue ») : la recette n'écrit pas les mêmes octets sur les deux plateformes, et la
  décision 4 de l'ADR-023 est à reprendre.
- [ ] **Pas 3 : recopier la ligne imprimée** dans `GENESE`, telle quelle : `#[rustfmt::skip]` la garde
  sur une ligne, et `cargo fmt --check` la laisse.
- [ ] **Pas 4 : voir passer.** Committer (`feat: la genèse du fonds, en octets`), pousser. Attendu :
  `tests/genese.rs` : `test result: ok. 4 passed`, dans `verifier` et dans `windows`.

### Tâche 6 : le fonds naît, et se rouvre

**Fichiers :** créer `crates/stockage/src/documents.rs`, `crates/stockage/tests/fonds.rs` ; modifier
`src/lib.rs`, `src/racine_des_donnees.rs`, `tests/commun/mod.rs` (les gestes de test).

**Interfaces — consomme :** `noyau::donnees::fonds`, `fichier::ecrire_a_cote`. **Produit :**

```rust
// stockage
pub struct Naissance<'a> { pub identifiant: &'a str, pub lignee: &'a str, pub proprietaire: &'a str }
pub struct Document<'r> { /* la racine des données ; documents/fonds/ ; l'Automerge ; à relire */ }
impl RacineDesDonnees {
    /// Le fonds, rouvert s'il est né (son `complet.automerge` existe) ; sinon `None`.
    /// Chaque ouverture tire un acteur neuf (`set_actor(ActorId::random())`).
    pub fn fonds(&self) -> Result<Option<Document<'_>>, Erreur>;
    /// § 31 · 2.3 : `GENESE`, la naissance (`MESSAGE_DE_LA_NAISSANCE`, qui écrit `IDENTIFIANT`,
    /// `LIGNEE` et `PROPRIETAIRE`), puis le premier geste, avec son message ; les trois en un
    /// seul fichier, `documents/fonds/complet.automerge`, écrit à côté puis renommé, et
    /// `documents/fonds/ajouts/` créé. Le geste refusé, rien ne s'écrit.
    pub fn faire_naitre_le_fonds(
        &self, naissance: &Naissance<'_>, message: &str, heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
    ) -> Result<Document<'_>, Erreur>;
}
impl Document<'_> {
    pub fn automerge(&self) -> &Automerge;
}
// Erreur += FondsDejaNe, Genese(String), Automerge(AutomergeError)
// tests/commun/mod.rs : les gestes de test, qu'appellent les fermetures `geste`
pub fn noter(tx: &mut Transaction<'_>, idee: &str, texte: &str) -> Result<(), AutomergeError>;
pub fn changer_le_texte(tx: &mut Transaction<'_>, idee: &str, texte: &str) -> Result<(), AutomergeError>;
pub fn etiqueter(tx: &mut Transaction<'_>, idee: &str, etiquette: &str) -> Result<(), AutomergeError>;
pub fn oter_l_etiquette(tx: &mut Transaction<'_>, idee: &str, etiquette: &str) -> Result<(), AutomergeError>;
pub fn poser_une_prise(tx: &mut Transaction<'_>, empreinte: &str, idee: &str, format: &str) -> Result<(), AutomergeError>;
```

`noter` crée l'idée sous `IDEES`, son `TEXTE` et son objet `ETIQUETTES` dans la même modification : les
conteneurs d'un objet naissent avec lui (§ 31 · 2.4). `oter_l_etiquette` n'écrit rien si l'étiquette manque.
Ces gestes sont ceux des tests : les opérations nommées arrivent au lot 1.

- [ ] **Pas 1 : écrire les tests** (`tests/fonds.rs`) :
  - `une_racine_des_donnees_neuve_n_a_pas_de_fonds` : `fonds()` rend `None`, `documents/` est vide ;
  - `naitre_ecrit_un_seul_fichier_complet` : `documents/fonds/` n'a que `complet.automerge` et un
    `ajouts/` vide ; rouvert, le fonds a trois modifications (`get_changes(&[])`), aux messages « Genèse
    du fonds », « Fonds créé » et celui du geste, la première d'empreinte `Change::try_from(GENESE)` ; la
    racine du document porte l'identifiant, la lignée et le propriétaire donnés, et l'idée notée ;
  - `un_premier_geste_refuse_ne_fait_pas_naitre_le_fonds` : un geste qui rend une erreur ; `fonds()`
    rend encore `None`, ni `complet.automerge` ni `.partiel` ;
  - `le_fonds_ne_nait_qu_une_fois` : une seconde naissance rend `FondsDejaNe`.
- [ ] **Pas 2 : voir échouer.** Committer (`test: le fonds naît, et se rouvre`), pousser. Attendu :
  `verifier` rouge à « Lint », `faire_naitre_le_fonds` introuvable.
- [ ] **Pas 3 : écrire `documents.rs`** (la naissance, la lecture du fichier complet par
  `Automerge::load`) **et les gestes de test.**
- [ ] **Pas 4 : voir passer.** Committer (`feat: le fonds naît, et se rouvre`), pousser. Attendu :
  `tests/fonds.rs` : `test result: ok. 4 passed`, dans `verifier` et dans `windows`.

### Tâche 7 : les fichiers d'ajout

**Fichiers :** créer `crates/stockage/tests/ajouts.rs` ; modifier `src/documents.rs`, `src/lib.rs`,
`tests/commun/mod.rs` (`texte_long`).

**Interfaces — produit :**

```rust
impl Document<'_> {
    /// § 31 · 2.1 : une modification atomique, avec son message et son heure (`commit_with`),
    /// faite une fois son fichier d'ajout en place : `save_after` des heads d'avant, sous
    /// `ajouts/<empreinte>.automerge`, écrit à côté puis renommé. Une écriture sans modification
    /// n'écrit rien. Le geste refusé, la transaction est annulée (`rollback`), `Erreur::Automerge`
    /// se rend, et le document reste ouvert. L'écriture du fichier échoue : l'erreur se rend, et
    /// le document est à relire.
    pub fn ecrire(
        &mut self, message: &str, heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
    ) -> Result<(), Erreur>;
}
// Erreur += AjoutAltere { chemin: PathBuf }, DependancesManquantes, ARelire
// tests/commun/mod.rs
/// 2 000 lettres de `a` à `z`, la n-ième `b'a' + octets(9, 2000)[n] % 26` : un texte que DEFLATE
/// ne ramène pas au poids d'un fichier d'ajout (`save` comprime le fichier complet, `save_after`
/// ne comprime pas les fichiers d'ajout).
pub fn texte_long() -> String;
```

`fonds()` lit ensuite chaque `ajouts/*.automerge`, dans l'ordre que donne le dossier : le contenu doit
donner l'empreinte du nom, sinon `AjoutAltere`, et rien n'est touché ; puis `load_incremental`. Tout lu,
`get_missing_deps(&[])` est vide, sinon `DependancesManquantes`. Un autre nom de `ajouts/`, un `.partiel`
par exemple, est laissé tel quel.

- [ ] **Pas 1 : écrire les tests** (`tests/ajouts.rs`). Le fonds y naît d'une idée au texte long
  (`commun::texte_long()`). Il ne sert qu'à la naissance, jamais à une écriture des tests des tâches 7
  et 8, pour que la compaction de la tâche 8, quand elle viendra, ne retire pas le fichier d'ajout
  d'une écriture :
  - `chaque_ecriture_ajoute_un_fichier_nomme_par_son_empreinte` : une écriture, un fichier de plus, nommé
    par l'empreinte de son contenu, qui pèse moins que la moitié du fichier complet ; le fichier
    complet lu seul, puis ce fichier, mènent aux heads de l'écriture ;
  - `rouvrir_relit_le_complet_puis_les_ajouts` : un `.partiel` déposé dans `ajouts/` ; fermé puis
    rouvert, les mêmes heads et la même valeur (`hydrate(None)`), et le `.partiel` est toujours là ;
  - `une_ecriture_sans_modification_n_ecrit_rien` : aucun fichier de plus ;
  - `un_geste_refuse_n_ecrit_rien_et_le_document_reste_ouvert` : le geste rend une erreur ; `ecrire`
    rend `Erreur::Automerge(_)` ; ni fichier ni `.partiel` de plus ; les heads d'avant ; l'écriture
    suivante réussit ;
  - `un_ajout_manquant_rend_dependances_manquantes` : deux écritures, le fichier d'ajout de la première
    retiré ; `fonds()` rend `DependancesManquantes` ;
  - `chaque_ouverture_tire_un_acteur_neuf` : écrire, rouvrir, écrire ; les deux modifications ont deux
    acteurs différents. Le fichier d'ajout de la seconde écriture est exactement `save_after` des heads
    d'avant elle, lu sur le document rouvert ; il pèse moins du quart du fichier complet, puisque le
    fonds y porte `texte_long()` ;
  - `un_ajout_altere_se_nomme_et_rien_ne_bouge` (vigilance 1) : un octet changé dans un fichier d'ajout ;
    `fonds()` rend `AjoutAltere` avec son chemin, et l'empreinte de chaque fichier de `documents/fonds/`
    est celle d'avant ;
  - `une_ecriture_qui_echoue_laisse_le_document_a_relire` : `ajouts/` remplacé par un fichier ; `ecrire`
    rend `Disque`, puis `ARelire` ; `ajouts/` rendu, le fonds rouvert a les heads d'avant l'écriture
    refusée.
- [ ] **Pas 2 : voir échouer.** Committer (`test: les fichiers d'ajout`), pousser. Attendu : `verifier`
  rouge à « Lint », `Document::ecrire` introuvable.
- [ ] **Pas 3 : écrire `ecrire` et la lecture des fichiers d'ajout.**
- [ ] **Pas 4 : voir passer.** Committer (`feat: les fichiers d'ajout`), pousser. Attendu :
  `tests/ajouts.rs` : `test result: ok. 8 passed`, dans `verifier` et dans `windows`.

### Tâche 8 : la compaction

**Fichiers :** créer `crates/stockage/tests/compaction.rs` ; modifier `src/documents.rs`, `src/lib.rs`.

**Interfaces — consomme :** `noyau::donnees::compaction`. **Produit :**

```rust
impl Document<'_> {
    /// § 31 · 2.2 : `save()` dans un fichier complet neuf, écrit à côté puis renommé ; relu par
    /// `Automerge::load` ; puis les fichiers que désigne `a_retirer` sont retirés. Le fichier
    /// complet relu ne mène pas aux mêmes heads : rien n'est retiré, et `CompactionInfidele`.
    /// Un document à relire : `ARelire`, et rien ne s'écrit.
    pub fn compacter(&mut self) -> Result<(), Erreur>;
}
// Erreur += CompactionInfidele
```

Ce que l'adaptateur lit, pour chaque fichier d'ajout : `conforme`, son contenu donne l'empreinte de son
nom ; `absorbe`, chargé par `load_incremental` dans un `fork()` du document relu, il laisse ses heads
inchangés et `get_missing_deps(&[])` vide. `fonds()` compacte quand `a_l_ouverture(ajouts, false)` le
dit ; `ecrire`, une fois son fichier en place, quand `apres_une_ecriture` le dit. Une compaction qui
échoue après une écriture ne la défait pas : l'écriture est faite, rien n'est retiré, et la compaction se
retente à la suivante ; l'essai ne l'éprouve pas (décision 8).

- [ ] **Pas 1 : écrire les tests** (`tests/compaction.rs`), le fonds né d'une idée au texte long, comme à
  la tâche 7 :
  - `rouvrir_compacte_les_ajouts` : une écriture, puis rouvrir ; `ajouts/` est vide, les heads et la
    valeur sont celles d'avant, et le fichier complet lu seul les donne ;
  - `les_ajouts_ne_pesent_jamais_plus_que_le_complet` : cent vingt écritures ; après chacune, sur le
    disque, les ajouts pèsent au plus le fichier complet et sont au plus cent ; rouvert, le fonds a ses
    cent vingt-trois modifications. Le fichier complet y est remplacé plusieurs fois, la voie de Windows ;
  - `la_compaction_garde_toute_l_histoire` : quelques écritures, puis `compacter()`, que le test appelle
    lui-même ; les messages de `get_changes(&[])` sont ceux de chaque modification, la genèse en tête ;
  - `la_compaction_garde_un_ajout_altere` (vigilance 1) : deux écritures, un octet changé dans le
    fichier d'ajout de la première, `compacter()` ; le fichier altéré est toujours là, octet pour
    octet, celui de la seconde est retiré, et `fonds()` rend ensuite `AjoutAltere` avec ce chemin.
- [ ] **Pas 2 : voir échouer.** Committer (`test: la compaction`), pousser. Attendu : `verifier` rouge à
  « Lint », `Document::compacter` introuvable.
- [ ] **Pas 3 : écrire `compacter`, et ses deux appels.**
- [ ] **Pas 4 : voir passer.** Committer (`feat: la compaction`), pousser. Attendu :
  `tests/compaction.rs` : `test result: ok. 4 passed`, et ceux des tâches 6 et 7 toujours verts, dans
  `verifier` et dans `windows`.

### Tâche 9 : le cœur de la synchronisation

**Fichiers :** modifier `crates/synchro/src/lib.rs` ; créer `crates/synchro/tests/echange.rs`.

**Interfaces — consomme :** `noyau::donnees::copie`. **Produit :**

```rust
// synchro (§ 31 · 4.4)
pub use noyau::donnees::copie::{Copie, Difference};
pub struct Echange { /* automerge::sync::State */ }
#[derive(Debug)]
pub enum Erreur { MessageIllisible(String), Automerge(automerge::AutomergeError) }
impl Echange {
    /// § 31 · 2.5 : compare les deux copies (`comparer`) avant tout échange.
    pub fn ouvrir(locale: &Copie, distante: &Copie) -> Result<Echange, Difference>;
    /// Le prochain message pour l'autre copie (`SyncDoc::generate_sync_message`, puis
    /// `Message::encode`), ou `None` quand il n'y a plus rien à dire.
    pub fn prochain_message(&mut self, doc: &Automerge) -> Option<Vec<u8>>;
    /// `Message::decode`, puis `SyncDoc::receive_sync_message`.
    pub fn recevoir(&mut self, doc: &mut Automerge, message: &[u8]) -> Result<(), Erreur>;
}
```

- [ ] **Pas 1 : écrire les tests** (`tests/echange.rs`), sur des documents en mémoire, dont la racine du
  document porte un propriétaire, un document et une lignée, le second né d'un `fork()` du premier :
  - `deux_copies_d_une_meme_lignee_convergent` : chacune écrit de son côté, dont la même clé ; on échange
    jusqu'à ce qu'aucune n'envoie plus rien ; mêmes heads, même valeur, et les deux valeurs de la clé
    commune dans `get_all` de chacune ;
  - `deux_lignees_differentes_n_ouvrent_aucun_echange` (vigilance 3) : `ouvrir` rend `AutreLignee` ; de
    même `AutreProprietaire` et `AutreDocument` ;
  - `un_message_illisible_est_refuse_et_le_document_ne_bouge_pas` : des octets quelconques ;
    `MessageIllisible`, les heads d'avant.
- [ ] **Pas 2 : voir échouer.** Committer (`test: le cœur de la synchronisation`), pousser. Attendu :
  `verifier` rouge à « Lint », `synchro::Echange` introuvable.
- [ ] **Pas 3 : écrire `Echange`.**
- [ ] **Pas 4 : voir passer.** Committer (`feat: le cœur de la synchronisation`), pousser. Attendu :
  `tests/echange.rs` : `test result: ok. 3 passed`, dans `verifier` et dans `windows`.

### Tâche 10 : ce qui est reçu s'écrit

**Fichiers :** créer `crates/stockage/tests/synchronisation.rs` ; modifier `src/documents.rs`,
`src/lib.rs`, `tests/commun/mod.rs` (`reunir`, `relire_le_dossier_du_fonds`),
`docs/31-persistance-et-synchronisation.md` § 2.1.

**Interfaces — consomme :** `synchro::Echange`. **Produit :**

```rust
impl Document<'_> {
    /// Le propriétaire, l'identifiant et la lignée, lus à la racine du document.
    pub fn copie(&self) -> Result<Copie, Erreur>;
    /// Le prochain message de l'échange ; `None` aussi pour un document à relire.
    pub fn prochain_message(&self, echange: &mut Echange) -> Option<Vec<u8>>;
    /// Applique un message. Si les heads ont changé, que la réception ait réussi ou non (ce que
    /// l'essai n'éprouve pas, décision 8), `save_after` des heads d'avant s'écrit en un fichier
    /// d'ajout avant la réponse, puis la compaction suit `apres_une_ecriture`. Rien de neuf, rien
    /// d'écrit. L'écriture échoue : le document est à relire. Un document à relire : `ARelire`.
    pub fn recevoir(&mut self, echange: &mut Echange, message: &[u8]) -> Result<(), Erreur>;
}
// Erreur += ChampAbsent { champ: &'static str }, Synchro(synchro::Erreur)
// tests/commun/mod.rs
/// Relit `documents/fonds/` à part, sans `RacineDesDonnees` : `Automerge::load` de
/// `complet.automerge`, puis `load_incremental` de chaque `ajouts/*.automerge`. Les noms en
/// `.partiel`, de `documents/fonds/` comme de `ajouts/`, sont ignorés, comme `fonds()` le fait.
pub fn relire_le_dossier_du_fonds(racine_des_donnees: &Path) -> Automerge;
```

`reunir` est le transport de l'essai, et son corps est donné : il dit quand une réunion est finie.

```rust
/// Réunit deux copies sans réseau : chacune envoie tant qu'elle a à dire, en octets, comme le
/// fera le transport du lot 3. Rend le nombre de messages ; au-delà de mille, la réunion ne
/// s'achève pas, et c'est un échec.
pub fn reunir(a: &mut Document<'_>, b: &mut Document<'_>) -> usize {
    let mut ea = Echange::ouvrir(&a.copie().unwrap(), &b.copie().unwrap()).unwrap();
    let mut eb = Echange::ouvrir(&b.copie().unwrap(), &a.copie().unwrap()).unwrap();
    let mut n = 0;
    loop {
        let de_a = a.prochain_message(&mut ea);
        if let Some(m) = &de_a { b.recevoir(&mut eb, m).unwrap(); n += 1; }
        let de_b = b.prochain_message(&mut eb);
        if let Some(m) = &de_b { a.recevoir(&mut ea, m).unwrap(); n += 1; }
        if de_a.is_none() && de_b.is_none() { return n; }
        assert!(n <= 1000, "la réunion ne s'achève pas");
    }
}
```

Au § 31 · 2.1, à la puce « Une opération nommée n'est faite qu'une fois son fichier d'ajout en place »,
une phrase : ce que la synchronisation reçoit ne l'est qu'une fois écrit, en un fichier d'ajout
(`save_after` des heads d'avant la réception), avant que l'échange ne continue ; un message qui n'apporte
rien n'écrit rien.

- [ ] **Pas 1 : écrire les tests** (`tests/synchronisation.rs`). B est une copie de la racine des
  données de A, A fermée, comme le carnet partira de l'établi (§ 31 · 2.3) :
  - `la_copie_se_lit_a_la_racine_du_document` : les trois champs de la naissance ;
  - `ce_qui_est_recu_se_relit_du_disque` (vigilance 2) : B écrit deux fois ; le test déroule à la main
    la boucle de `reunir`. Après chaque `recevoir` d'A, et avant son `prochain_message`,
    `relire_le_dossier_du_fonds` de la racine des données d'A mène à `a.automerge().get_heads()`.
    Au premier `recevoir` d'A qui change ses heads, et le test vérifie qu'il arrive, `ajouts/` d'A a
    un fichier de plus, dont le contenu donne l'empreinte de son nom. Une fois la réunion finie, A
    fermée puis rouverte a les heads de B ;
  - `un_message_sans_rien_de_neuf_n_ecrit_rien` (vigilance 2) : A et B identiques ; `reunir` échange au
    moins un message, et `ajouts/` de chacune est inchangé ;
  - `un_document_a_relire_ne_laisse_rien_partir` (vigilance 2) : `ajouts/` d'A remplacé par un
    fichier, puis une écriture refusée. `prochain_message` d'A rend `None`, `recevoir` et `compacter`
    rendent `ARelire`, et l'empreinte de chaque fichier de `documents/fonds/` d'A est celle d'avant.
    `ajouts/` remis avec ses fichiers d'avant, comme à la tâche 7, A rouverte puis réunie à B : ni A
    ni B ne portent le message de l'écriture refusée.
- [ ] **Pas 2 : voir échouer.** Committer (`test: ce qui est reçu s'écrit`), pousser. Attendu :
  `verifier` rouge à « Lint », `Document::recevoir` introuvable.
- [ ] **Pas 3 : écrire `copie`, `prochain_message`, `recevoir`, `reunir`, `relire_le_dossier_du_fonds`,
  et la phrase du § 31 · 2.1.**
- [ ] **Pas 4 : voir passer.** Committer (`feat: ce qui est reçu s'écrit`), pousser. Attendu :
  `tests/synchronisation.rs` : `test result: ok. 4 passed`, dans `verifier` et dans `windows` ;
  `documentation` vert.

### Tâche 11 : l'essai, convergence et restauration

**Fichiers :** créer `crates/stockage/tests/h07.rs`.

**Interfaces — consomme :** `RacineDesDonnees::{ouvrir, chemin, fonds, faire_naitre_le_fonds,
captations}`, `Naissance`, `Document::{ecrire, automerge, copie}`, `Captations::{verser, relire,
verifier}`, `Erreur::{CaptationAlteree, CaptationAbsente}`, `noyau::donnees::fonds::{IDEES, PRISES,
TEXTE, ETIQUETTES}`, `commun::{dossier_jetable, octets, copier_l_arbre, noter, changer_le_texte,
etiqueter, oter_l_etiquette, poser_une_prise, reunir}` ; ne produit aucun code. Ces tests ne précèdent
pas de code : ils sont l'essai. Un rouge n'y est pas une étape, c'est un verdict à instruire (« Si H07
échoue », plus bas).

- [ ] **Pas 1 : écrire les tests** (`tests/h07.rs`) :
  - `deux_repliques_modifiees_hors_ligne_convergent`. A naît en notant `idee-1` (« une ligne de basse
    qui descend »), puis note `idee-2` (« un refrain en question »), verse `octets(1, 4000)` et pose sa
    prise ; A fermée, B en est la copie. Hors ligne, A change le texte d'`idee-1` (« une ligne de basse
    qui monte »), étiquette `idee-2` « refrain », note `idee-3`, puis se rouvre ; B change le texte
    d'`idee-1` (« une ligne de basse en arpèges »), étiquette `idee-1` « basse », note `idee-4`, verse
    `octets(2, 9000)` et pose sa prise, puis se rouvre. `reunir`. Attendu : mêmes heads et même valeur ;
    `get_all` du texte d'`idee-1` porte les deux textes dans chacune ; les quatre idées, les deux
    étiquettes et les deux prises dans chacune ; la captation de B n'est que dans B, puisque la
    synchronisation ne transporte que les documents. Fermées puis rouvertes, les deux ont les mêmes heads
    et la même valeur qu'avant ;
  - `la_convergence_tient_sur_des_gestes_tires_au_hasard` : `proptest`,
    `proptest::test_runner::Config::with_cases(32)`. La base : A naît avec deux idées communes, B en est
    la copie. Chaque réplique joue de 0 à 12 gestes, tirés parmi : changer le texte d'une idée commune
    (quatre textes inventés), étiqueter une idée commune, ou en ôter l'étiquette (trois étiquettes),
    noter une idée à elle (`<réplique>-<rang>`), se rouvrir. `reunir`. Attendu : mêmes heads et même valeur,
    avant et après la réouverture des deux ;
  - `une_copie_restauree_relit_chaque_captation_contre_son_empreinte`. A verse `octets(3, 0)`,
    `octets(4, 1)` et `octets(5, 70000)`, pose leurs trois prises, relève ses heads et sa valeur, puis se
    ferme ; `copier_l_arbre` vers un dossier neuf, `verrou` compris. La copie s'ouvre ; `verifier` n'y
    nomme rien ; chaque clé de `prises` s'y relit (`relire`) ; le fonds a les heads et la valeur relevées ;
    une écriture y réussit, et `copie()` est celle de A ;
  - `une_captation_alteree_ou_absente_de_la_copie_se_nomme` (vigilance 5) : la même copie, la deuxième
    captation altérée d'un octet, la troisième retirée. `verifier` nomme la deuxième et elle seule ;
    `relire` rend `CaptationAlteree` pour elle, `CaptationAbsente` pour la troisième.
- [ ] **Pas 2 : lancer l'essai.** Committer (`test: l'essai H07, convergence et restauration`), pousser.
  Attendu : `tests/h07.rs` : `test result: ok. 4 passed`, dans l'étape « Tests » de `verifier` et dans
  l'étape « Tests du stockage et de la synchronisation (H07) » de `windows`. Relever le numéro du passage
  et les deux lignes, pour la PR.

### Tâche 12 : le verdict

**Fichiers :** modifier `CLAUDE.md` (« État »), `README.md` (« Bloquant pour démarrer »),
`docs/70-qualite-et-verification.md` (§ 8), au besoin `.github/workflows/ci.yml` (les délais de
`verifier` et de `windows`) et `docs/registre.md`.

- [ ] **Pas 1 : les délais.** Lire les durées de `verifier` et de `windows` relevées au pas 5 de la
  tâche 1 : le premier passage complet après l'entrée d'`automerge`, rust-cache ne sauvant pas le cache
  d'un passage rouge. Les passages rouges ou annulés ne comptent pas. Si deux fois la durée de
  `windows` passe ses 40 minutes, son `timeout-minutes` vaut ce double, et son commentaire le dit
  (LE-29 de `modele-projet`) ; le commentaire de `verifier`, provisoire, prend la mesure. Chaque
  commentaire dit que ce passage restaure le cache de `main`, et n'est froid que pour `automerge` et ses
  dépendances.
- [ ] **Pas 2 : les documents.** `CLAUDE.md`, « État » : une ligne, « H07 est passé le AAAA-MM-JJ (PR #n,
  issue #16) », ce qu'il a montré, sous Linux et sous Windows, et que le plan du lot 1 part de
  là. `README.md`, « Bloquant pour démarrer » : l'essai H07 en sort, Q23 y reste. Au § 70 · 8, à la
  suite du paragraphe « Après INC-0 (2026-11-04) », un paragraphe : « Après H07 (AAAA-MM-JJ) : I14 a
  son test, dans `stockage` (`tests/captations.rs`, `tests/h07.rs`) ; le job `verifier` refuse aussi
  un `Cargo.lock` en retard sur les `Cargo.toml`. » Une leçon de l'essai va au registre par
  `/consigner`.
- [ ] **Pas 3 : la PR.** Son titre reste `feat: l'essai H07, convergence et restauration` : il devient
  l'en-tête du squash. Sa description dit ce que l'essai a éprouvé, les décisions de ce plan, les
  dépendances qui entrent (`automerge` 0.12.0, `sha2` 0.11.1, et `proptest` en test), les numéros des
  passages verts dans les deux jobs, et « Closes #16 ». Committer
  (`docs: le verdict d'H07`), pousser ; `gh pr ready <n>` ; tous les contrôles verts,
  `gh pr merge <n> --squash` (fusion `session`). Aucune étiquette.

---

## Si H07 échoue

**Comment l'échec se constate.** Un test de `tests/h07.rs`, ou, plus tôt, un test des tâches 2, 3 et 5
à 10 qu'on ne sait pas faire passer, dont une genèse aux octets différents sous Linux et sous Windows
(tâche 5, pas 2), est rouge dans `verifier`, dans `windows`, ou dans l'un des deux seulement :
- **divergence** : la réunion finie, les deux répliques n'ont pas les mêmes heads, ou pas la même valeur,
  ou plus après leur réouverture ; ou la réunion ne s'achève pas ; ou un cas tiré au hasard échoue, et
  `proptest` imprime la plus courte suite de gestes qui le fait ;
- **altération** : sans rien d'injecté, la copie restaurée nomme une captation, ou ne relit pas une prise ;
- **fichiers** : un fichier écrit à côté puis renommé ne tient pas, sous Windows surtout (tâches 2
  et 3), ou un document ne se relit pas de ses fichiers (tâches 6 à 8) ;
- **genèse** : la recette n'écrit pas les mêmes octets sur les deux plateformes, ou les idées d'un côté
  sortent de la vue malgré elle (tâche 5).

**Avant de conclure**, `superpowers:systematic-debugging` situe la cause. Un défaut de notre code qui se
corrige sans toucher aux décisions 1 à 4 de l'ADR-023 ni au protocole se corrige dans la PR, qui le dit ;
une leçon en va au registre. H07 échoue quand la cause est dans le format (un dossier par document, le
fichier complet et les fichiers d'ajout, l'écriture à côté puis renommée, la compaction), dans le
protocole tel que la couche le porte, dans la voie de Windows que le format ne sait pas tenir, dans
l'écriture d'une captation, ou dans la genèse.

**Alors**, on s'arrête : la PR de l'essai n'est pas fusionnée, reste en brouillon, et sa description dit
le verdict, les passages rouges et le plus petit cas ; l'issue reste ouverte ; Edouard en est averti.
Une PR à part, `docs: le verdict d'H07, négatif`, partie de `main`, porte trois lignes : la ligne `EM`
du registre, par `/consigner` ; la ligne « État » de `CLAUDE.md`, « H07 a échoué le AAAA-MM-JJ (PR #n
en brouillon, issue #16) : <cause> ; la conception se reprend » ; et la ligne « Bloquant pour
démarrer » du README, où le plan du lot 1 attend la conception reprise. Elle se fusionne ; la PR de
l'essai reste en brouillon. **On revient à la conception**, avant tout plan du lot 1 :
- une cause dans les fichiers des documents (le fichier complet, les fichiers d'ajout, leur écriture à
  côté et leur voie de Windows, la compaction, le protocole : tâches 2 et 6 à 10) suit la branche des
  répliques qui divergent : samod remplace la couche de synchronisation et le rangement des documents
  sur le disque, derrière le même adaptateur, par une ADR nouvelle qui amende les décisions 1 à 3 de
  l'ADR-023 (A21, ADR-003) ;
- une cause dans les captations (tâche 3, l'écriture à côté d'une captation, la restauration de la
  tâche 11) fait reprendre l'adaptateur `Captations` ;
- une défaillance du renommage sous Windows qui touche les deux se nomme dans les deux ;
- une genèse qui ne s'écrit pas à l'octet près sur les deux plateformes, ou qui ne garde pas les idées
  des deux côtés (tâche 5), rouvre le § 31 · 2.3 et la décision 4 de l'ADR-023, sans repli écrit
  d'avance.

Ce plan ne conçoit aucune de ces reprises.
