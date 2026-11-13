# La tranche 1 du lot 1 d'INC-1 : le stockage durci — plan d'implémentation

> **Pour les agents :** exécuter ce plan avec `superpowers:subagent-driven-development`, tâche par tâche ;
> les étapes sont des cases à cocher (`- [ ]`).

**Objectif :** le stockage tient une panne à chaque étape, s'ouvre en lecture seule au lieu de refuser,
reprend au démarrage ce qu'un arrêt a laissé, et chacune de ses règles décide au noyau.

**Architecture :** `stockage` reçoit un point d'injection, la fonction de Cargo `pannes`, sur chaque étape
d'une écriture ; ses écritures rendent « faite », « faite, avec un avertissement » ou un refus suivi de la
relecture du disque, et son ouverture rend un document entier, en lecture seule nommée, ou plus récent.
Le noyau gagne les règles de l'accès d'un document, du démarrage, de la version de schéma, de la teneur et
de la lignée, sous mutation ; `synchro` gagne deux fonctions, la reprise d'un échange après une réception
refusée et l'envoi sans modification.

**Pile :** Rust 1.99.1, édition 2024 ; `automerge` 0.12.0 ; `sha2` 0.11.1 ; tests : `proptest` 1.12.0 ; en
CI, `cargo-mutants` 27.1.0. Aucune dépendance nouvelle : `pannes` est une fonction de Cargo.

**Spécification :** le plan maître, `.claude/plans/lot1-le-fonds-sur-l-etabli.md`, section « Tranche 1 · le
stockage durci », ses décisions 4, 7, 8, 9, 12, 16 et 20, « AM-4, point par point » ; et ce qu'il cite :
§ 31 · 2.1 à 2.6, 6.1 et 10 (tests 4, 8 à 11) ; § 70 · 1 et 3 (I14, I28, I30) ; ADR-023 ; au registre, AM-4
et LE-10 ; le plan d'H07, décisions 7 et 8.

Refs #24

---

## Contraintes générales

Recopiées du plan maître (« Contraintes générales », « Le déroulé ») ; chaque tâche les suppose.

- « **En CI seulement.** Ni cargo, ni trunk, ni npm sur le poste : chaque vérification de Rust se lit dans le job que nomme le critère, le rouge puis le vert dans `verifier`, le vert aussi dans `windows` si le critère l'y met. »
- « Seul `rustfmt --edition 2024` formate les `.rs` touchés ; les vérifications de documentation tournent sur le poste, après le `git add` (LE-7). »
- « Les commandes de Rust prennent `-p <crate>`, et `ci.yml` nomme chaque crate qu'il passe à `clippy` et aux tests. »
- « **Le noyau** reste sans entrée-sortie, ni `automerge`, ni horloge, ni aléa ; chaque règle qui décide y vit, sous mutation (décision 4). »
- « **Les chemins se reçoivent** (I30) : hors de `systeme`, aucun code ne devine un chemin ; chaque test prend son dossier jetable par une seule fonction de sa crate. »
- « **Aucune donnée réelle** : textes et noms inventés ; identifiants, documents et captations engendrés à l'exécution ; aucun audio, MIDI ni `.automerge` suivi. »
- « **Les mots du glossaire**, dits en entier : la racine des données et la racine du document, la version de schéma, l'origine déclarée, l'identifiant de propriétaire, le manifeste de l'application. »
- « Une PR qui ajoute un mot au glossaire le cherche d'abord (`git grep -n -i -w <mot> -- docs README.md CONVENTIONS.md`), requalifie chaque emploi antérieur et cite la recherche (LE-2) ; une PR qui périme un mot ou un identifiant le cherche dans tout le dépôt, maquettes et commentaires SVG compris (LE-3). »
- « Conventional Commits en français, en-tête de 100 caractères au plus, titre de 93 au plus ; « Refs #24 » » ; `git add` par chemins explicites, jamais `git add .` ; `git commit -F <scratchpad>/msg-<tâche>-<étape>.txt` (un texte accentué passe par un fichier).
- « Chaque PR qui livre un critère de la SFD en entier l'inscrit au tableau du § 70 · 4.1, et un test le nomme (`cap06_7_…`) et en calcule l'exemple. » La tranche 1 n'en livre aucun (ses critères de sortie, au plan maître, n'en nomment aucun ; § 70 · 4.1) : le tableau ne change pas, et la revue de chaque PR le vérifie.
- « fusion par squash, une fois tout vert » ; mode `session` : la PR s'est ouverte en brouillon, donc `gh pr ready <n>`, puis `gh pr merge <n> --squash`, depuis le worktree, sans `cd` ni `-R`.
- « une tranche qui alourdit les tests remesure les délais de `verifier` et de `windows` ; l'installateur NSIS reste en artefact. »
- Chaque PR part de `main` à jour (depuis le clone principal `pupitre`, jamais depuis un worktree, où le chemin relatif mènerait dans `pupitre.worktrees/pupitre.worktrees/` : `git fetch`, puis `git worktree add ../pupitre.worktrees/<branche> -b <branche> origin/main`, en barres obliques, qui valent sous Git Bash comme sous PowerShell ; aucun chemin de machine n'entre dans le dépôt), s'ouvre en brouillon au premier envoi (`ci.yml` ne tourne que sur une PR), et dit dans sa description les tests réécrits et pourquoi.
- **Lire la CI** : `gh run list --branch <branche> --workflow ci --limit 1`, `gh run watch <n> --exit-status`, `gh run view <n> --log-failed`. Un rouge attendu se lit à l'étape « Lint » de `verifier` (`unresolved import`, `cannot find`, E0599) ou à « Tests » (`panicked at`, avec le nom du test) ; un rouge à « Format » se corrige (`style: le format de rustfmt`) avant de relire. Un vert se lit à « Tests » de `verifier` et à « Tests du stockage et de la synchronisation (H07) » de `windows` : `test result: ok.` pour chaque fichier touché. Les tests du noyau ne tournent que dans `verifier` et `mutants` (écart 2 au plan maître, plus bas).
- Chaque test qui prouve un des tests 4, 8, 9, 10 ou 11 du § 31 · 10 porte au-dessus de lui le commentaire `// § 31 · 10, test N : <la clause>` ; les tests d'H07 qui prouvent déjà une clause le prennent aussi, à la tâche 17, qui les liste.

---

## Ce que les passages ont levé

Les deux passages (AM-2, LE-6), faits avant ce plan, ne sont pas dans le dépôt : leur détail a servi à écrire ce
plan ; la description de sa PR (#27) porte la table de couverture. La PR de ce plan consigne (`/consigner`), au
motif « Une spec qui se contredit », une ligne, LE-12, qui nomme C2, C7, C12 et Éc-1 à Éc-7, avec leurs décisions
ci-dessous ; la tâche 17 la clôt.

### Le passage d'AM-2

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| C1. La relecture du disque qui échoue : la mémoire montrerait l'opération refusée | Le document passe en lecture seule `RelectureManquee { chemin }`, sur `fork_at(heads d'avant)` de sa mémoire, jamais la mémoire telle quelle ; il n'envoie ni ne reçoit | 6 |
| C2. « Faite, avec un avertissement » contre le § 31 · 2.1 et l'ADR-023 | « En place » veut dire « renommé ». Avant le renommage : refus, voisin retiré, relecture du disque. Après : un `rendre_durable` refusé, ou une compaction qui échoue, laisse « faite, avec un avertissement ». La règle vaut pour l'opération, la réception, le premier fichier complet et la migration. La PR (b) récrit le § 31 · 2.1, texte et diagramme ; c'est une précision de l'ADR-023, non un amendement (question 4 de la PR de ce plan, tranchée le 2026-11-09) | 6, 8, 13 |
| C3. Le voisin du premier fichier complet, une seconde naissance | Le prédicat `pupitre_neuf` au noyau : ni fichier complet, ni voisin du complet lisible. `faire_naitre_le_fonds` refuse (`FondsDejaNe`) dès qu'il est faux ; les tranches 2 et 6 le reprennent, sans en écrire un autre | 9, 11 |
| C4. Aucune étape « retrait » | Huitième étape `Retrait`. Un voisin qui ne se retire pas se tronque à zéro octet : vide, il ne se lit pas, et le démarrage le retire. La double panne (retrait et troncature) est la seule qui puisse faire paraître l'opération refusée, et le § 31 · 2.1 le dit | 3, 7, 8, 11 |
| C5. « Entiers » : des parts vivent ailleurs | « Entier » : chaque clause des tests 4 et 8 à 11 qui porte sur les chemins d'écriture de `stockage`, la genèse d'aujourd'hui et l'API du stockage. Partent ailleurs : test 4 rejoué sur la copie en flux (tranches 5, 6) et sa règle étendue (tranches 2, 5) ; la réponse de la commande du test 8 (tranche 2) ; le bandeau du test 10 (tranche 3), « l'idée qui la reçoit naît après la mise à jour » (PR de conception du lot 2), son refus par une commande sur les deux paires (tranche 2) ; le manifeste témoin du test 11 (tranche 5), sa porte du gel (tranche 9), ses références changées (tranche 2). Les tests du noyau tournent dans `verifier` seul | — |
| C6. Opération, commande, refus nommé avant les tranches 2 et 3 | En tranche 1 : une opération est un appel d'`ecrire` ; sa réponse, `Result<Option<Avertissement>, Erreur>` ; un refus nommé, une variante d'`Erreur` qui porte la raison. Pour le plan de la tranche 2 : la suite des commandes ouvre un fonds de version N + 1 et voit la commande refusée en le nommant, sur les deux paires ; le doctest « pendant l'écriture » se reprouve sur l'API qui applique les écritures du noyau ; son plan dit qui conduit la migration. Les tests nouveaux de la tranche écrivent par `commun::ecrire` | 1, 6 |
| C7. « `synchro` ne change pas » ; le test N + 1 qui passerait à vide ; la lignée | Les dépendances et le rôle de `synchro` ne changent pas ; deux fonctions s'y ajoutent : `Echange::reprendre` (tâche 6), sans laquelle un échange ne reprend pas après une réception refusée (l'autre copie ne renvoie pas ce qu'elle croit envoyé), et `prochain_message_sans_modification` (tâche 14). Le test N + 1 donne à la copie une modification que l'autre n'a pas. La règle de la lignée reste au noyau ; `comparer`, `Copie` et `Echange::ouvrir` ne changent pas avant le lot 4 ; le cas de `synchro` de la règle de la lignée (`Echange::ouvrir` qui la consulterait) sort de la tranche | 6, 14, 16 |
| C8. La clé des lignées remplacées naît après le gel | La constante `noyau::donnees::fonds::LIGNEES_REMPLACEES`, que rien n'écrit avant le lot 4 ; la teneur l'exclut dès la tranche 1, avec un cas de test ; la genèse ne change pas | 15 |
| C9. La table des versions de Pupitre, livrée et reportée | La table existe : le type `Migration`, et `MIGRATIONS`, vide. Elle arrive en argument de l'ouverture (`fonds_selon`, un point d'entrée des tests, sous `pannes`) ; la migration de test y entre le temps du test, avec la version de Pupitre `0.0.0-essai` ; sa lecture dans le journal reste reportée | 13 |
| C10. Lecture seule, avertissement, voisin, reprise | Deux états dans le type (`LectureSeule`, `Acces`) ; une entrée « Lecture seule (d'un document) » au glossaire, PR (b) ; « faite, avec un avertissement » défini au § 31 · 2.1, sans entrée ; l'entrée « Voisin » prend le suffixe, PR (c) ; le code dit « ranger » un nom provisoire, jamais « reprendre », et la PR (c) récrit en ce sens la clause du test 4 du § 31 · 10 (« prend son nom au démarrage ») | 8, 12 |
| C11. Des documents qu'aucune PR ne nomme | (b) : § 31 · 2.1, § 31 · 2.2, glossaire, § 40 · 3, `CLAUDE.md` ; (c) : § 31 · 2.1, 2.2, 6.1 et 10 (test 4), glossaire ; (d) : § 31 · 2.4 et 2.6, § 40 · 3 et `CLAUDE.md` (`fonds_selon`), le registre (AM-4 et LE-12) ; et, si le repli de la tâche 14 joue, le glossaire (Lecture seule) et, au registre, la ligne de la contradiction | 8, 12, 14, 17 |
| C12. « Sans exemption », quand le § 40 · 9 en prévoit une | Aucun dossier n'est exempté, et la porte n'a aucun mécanisme d'exemption ; la PR qui suivrait un document de test l'ajouterait et le dirait ; la ligne `contenu` le dit | 2 |
| C13. « Compaction retentée » | Le `Document` tient `compaction_due`, levé par toute compaction qui échoue, baissé par une qui réussit ; `apres_une_ecriture` le reçoit et compacte alors à l'écriture suivante | 4, 6, 7 |
| C14. Une seule fonction de dossier jetable par crate | `stockage` : `commun::dossier_jetable` ; `etabli` : `dossier_jetable`, renommée ; `cahier` : `fichier_temporaire` compte comme sa fonction ; `noyau`, `synchro`, `interface` : aucun test n'écrit. L'écriture de la maquette, à la demande et jamais en CI, sort d'I30 | 1 |

### Le passage de LE-6

| Écart ou point | Décision de ce plan | Tâche |
|---|---|---|
| Éc-1. `Lu::conforme` ne vaut jamais pour le voisin du complet | Le champ se renomme `lisible` (tâche 4), le mot que `Partiel::DansLeDossier` emploie : un fichier d'ajout dont le contenu donne l'empreinte de son nom, et (tâche 7) que `load_incremental` charge, puis aussi (tâche 11) un voisin du complet non vide que `load` relit sans erreur ; `compacter` le calcule ainsi. `Lu::nom` devient le chemin relatif au dossier du document (`ajouts/<e>.automerge`, `complet.automerge.<processus>.partiel`) | 4, 7, 11 |
| Éc-2. Les règles de compaction disent « compacter » en lecture seule | `a_l_ouverture` et `apres_une_ecriture` reçoivent l'`Acces` ; la doc d'`AJOUTS_AU_PLUS` se réécrit | 4, 14 |
| Éc-3. Le complet relu après son renommage | `ecrire_a_cote` relit le voisin, synchronisé et fermé, avant de le renommer, pour tout fichier : l'ordre des captations. `a_retirer` perd `meme_point`. Le § 31 · 2.2, point 3, suit | 3, 4, 8 |
| Éc-4. Le voisin du complet vide | « Lisible » : non vide, et `load` sans erreur ; un voisin vide est retiré, et Pupitre reste neuf | 9, 11, 12 |
| Éc-5. « Sous Windows » contre « partout » | Le § 31 · 6.1 dit « partout » | 12 |
| Éc-6. « Sans rien transmettre » est irréalisable | Les messages partent sans modification (`ChunkList::empty()`), et annoncent les heads de l'autre copie (le champ public `State::their_heads`, `Option<Vec<ChangeHash>>` en 0.12.0, vide au premier) : sans cela, `generate_sync_message` ne rend jamais `None` et l'échange ne finit pas. Le test borne les messages ; s'il n'en vient pas à bout, la tâche 14 s'arrête (voir elle) | 14 |
| Éc-7. La forme d'un `F64` n'est pas fixée | La plus courte de `{}` et de `{:e}`, la fixe à égalité ; l'infini `inf`, `-inf` ; NaN refusé ; jamais `{:?}`. Le § 31 · 2.6 le dit | 15, 17 |
| Éc-8. « Décision 12 du plan d'H07 » | La décision 12 est celle du plan maître ; ce plan lit les décisions 7 et 8 d'H07 | — |
| Une compaction d'ouverture refusée, que la décision 12 du plan maître dit « retentée », quand les règles du noyau ne la retentent qu'à l'ouverture suivante | Voir C13 | 4 |
| Le suffixe (§ 31 · 2.1) : un voisin suffixé doit se reconnaître au démarrage, et le test 9, clause 4, passerait sans suffixe s'il ne coupait rien | `fichier_du_voisin` reconnaît `-<n>` ; le test 9, clause 4, coupe le renommage du voisin de la compaction | 9, 11 |
| La garde de la naissance ne regarde que `complet.automerge` : une naissance par-dessus un voisin lisible du complet (§ 31 · 2.1, « à la naissance du document ») en écrirait une seconde | La garde de la naissance voit le voisin lisible du complet | 11 |
| Le rangement des noms provisoires (décision 12 du plan maître ; § 31 · 6.1) : deux noms provisoires du même contenu, une captation rangée altérée, un fonds en lecture seule | La règle se lit sur l'état mis à jour : de deux noms provisoires du même contenu, le premier dans l'ordre des noms (sinon le second écrirait sur une captation rangée, contre I14) ; une captation rangée altérée reste au contrôle des empreintes, le nom provisoire reste ; en lecture seule, « que le fonds nomme » porte sur ce qui se lit | 9, 12 |
| Une version de schéma absente, nulle ou qui n'est pas un `Uint`, dont le § 31 · 2.4 ne dit rien | `Lecture::Illisible`, lecture seule `VersionIllisible`, sans échange | 13 |
| Les lignées remplacées (§ 31 · 2.5) : une lignée qui en remplace une qui en remplaçait une troisième ; deux lignées qui se nomment l'une l'autre | La liste des lignées remplacées se veut complète : la règle ne lit que l'appartenance, et le lot 4 écrit la liste entière ; deux lignées qui se nomment l'une l'autre sont étrangères | 16 |
| L'empreinte de la genèse que le test 11 écrit : celle des 106 octets de `GENESE`, ou celle de sa modification pour Automerge | L'empreinte des 106 octets, lecture littérale du test 11 : `aa03e0cee230ee1aab90c1d53a6a00e51da250b8b5944b67560dd4f9b301c105` | 17 |
| Les seuils de compaction (§ 31 · 2.2, point 1 : « plus que son fichier complet », « passent la centaine ») | Tenus ; « passent la centaine » ne s'éprouve qu'au noyau, ce qui suffit à la mutation | — |
| Le voisin du complet dans `a_retirer` (§ 31 · 2.2, point 4) ; en lecture seule, pas de compaction (§ 31 · 2.2) ; l'ordre de la relecture du complet (§ 31 · 2.2, point 3 ; LE-10) ; ce qu'une copie reçoit et transmet devant une version de schéma plus récente (§ 31 · 2.4) | Éc-1, Éc-2, Éc-3 et Éc-6, ci-dessus | — |
| La teneur (§ 31 · 2.6) : les clés dans l'ordre de leurs octets (« Z » avant « a », « é » après « z »), les textes pris octet pour octet, la forme d'un `F64` | Tenus, sauf la forme d'un `F64` : Éc-7 | 15 |
| Les ordres de grandeur du § 31 · 6.1 (256 sous-dossiers, quelques dizaines de captations chacun au fonds visé, quelques centaines à cent mille) | Tenus | — |
| Les chiffres recopiés : les 39 tests d'H07, les neuf modules de `.cargo/mutants.toml`, les délais de 35 minutes (`mutants`), 40 (`windows`) et 15 (`verifier`) | Tenus ; « décision 12 du plan d'H07 » : Éc-8 | — |
| Les « jamais » et les « toujours » : « la compaction n'écrit jamais son propre voisin par-dessus celui qu'elle fond » (§ 31 · 2.1), « un nom provisoire […] n'est jamais retiré d'office » (§ 31 · 6.1), « une captation se verse toujours » (décision 12 du plan maître), « chaque ouverture tire un acteur neuf » (§ 31 · 2.1) ; le doctest `compile_fail` qui « fige » | Tenus, avec le suffixe de `voisin` et de `verser` ; un doctest `compile_fail` passe sur toute erreur de compilation : il dit la sienne, E0502 | 6, 10 |

Les tensions que le relevé des exigences a trouvées en lisant les sources, et leur sort :

- la « décision 12 du plan d'H07 », qui est celle du plan maître → Éc-8 ;
- « `synchro` ne change pas » (décision 7 du plan maître) contre le test d'une copie N + 1 et la règle de la lignée → C7 ;
- les tests 8 et 9, qui se complètent en (c), et le test 9 en (d), sa colonne « Échoue si » nommant la teneur → en (b),
  le test 9 compare les heads et `hydrate`, et la teneur s'y ajoute en (d) ;
- les aides recopiées dans deux crates (`copie`, `reunir`, `HEURE`), quand une crate ne voit pas le `tests/commun`
  d'une autre → un exemplaire par crate (tâche 1) ;
- la table des versions de Pupitre d'une migration, livrée (plan maître, tranche 1, sixième puce) et reportée
  (« Ce qui est reporté ») → C9 ;
- le `.gitignore` de `main`, sans les deux motifs que le plan maître dit ajouter → la PR (a) les écrit ;
- le mécanisme de `pannes`, qui touche ou non `ci.yml` et les « Commandes » de `CLAUDE.md` → une dépendance de test
  de `stockage` sur elle-même ;
- `CompactionInfidele`, qu'aucun test d'H07 ne provoque et que le plan maître ne nomme pas → elle reste, rendue par
  l'injection sur `Save` ;
- deux lectures seules, devant un fichier illisible et devant une version de schéma plus récente → C10 ;
- le rangement des noms provisoires d'un fonds en lecture seule → « que le fonds nomme » porte sur ce qui se lit ;
- `rendre_durable` refusé après le renommage du premier fichier complet, qui ferait se croire Pupitre neuf (test 8)
  → la naissance faite, avec un avertissement (C2) ;
- le suffixe, que le nom provisoire d'une captation demande aussi (§ 31 · 6.1 ; test 4) → il vaut pour `verser` ;
- « verts dans `verifier` et `windows` », quand `windows` ne lance que `stockage` et `synchro` → les tests du noyau
  dans `verifier` seul (écart 2) ;
- le test 11 et « chaque genèse », quand la seule genèse du lot 1 change encore en tranche 2 → la tranche 2 change
  les références du test 11 et le dit ;
- les tests de part de CAP-01 · 3 et de SYN-01 · 4 (§ 70 · 4.1) → aucun en tranche 1 : l'exemple de CAP-01 · 3
  suppose un enregistrement, qui vient au lot 2 ;
- l'ordre de l'ouverture (décision 12 : les voisins de `documents/`, puis les noms provisoires de `captations/`) → le
  démarrage a lieu à chaque `fonds()`, après le verrou par construction.

### Les écarts au plan maître

Le plan maître le veut (en tête) : un plan de tranche qui le contredit le dit dans sa PR, et la maîtrise
d'ouvrage relit ce point. La description de la PR de ce plan a listé ces trois écarts, puis la question C2 ;
la maîtrise d'ouvrage les a tranchés le 2026-11-09, chacun selon son défaut :

1. **Des obligations passent à la tranche 2** (C5, C6), que la section « Tranche 2 » du plan maître ne porte
   pas : la suite des commandes ouvre un fonds de version de schéma N + 1 et voit la commande refusée en le
   nommant, sur les deux paires ; le doctest « une synchronisation lancée pendant l'écriture ne voit pas
   l'opération en cours » se reprouve sur l'API qui applique les écritures du noyau ; son plan dit qui conduit
   la migration. La PR de ce plan les écrit dans la section « Tranche 2 » du plan maître, liste « Elle livre ».
   Retenu.
2. **Les tests du noyau tournent dans `verifier` seul** (C5 ; `windows` ne lance que `stockage` et `synchro`), quand le critère de sortie
   dit « verts dans `verifier` et `windows` ». Un seul porte un marqueur, `les_noms_provisoires_a_ranger`
   (test 4), dont la clause a aussi son test de `stockage` dans `windows`,
   `une_captation_nommee_prend_son_nom_au_demarrage`. Retenu : l'écart accepté par écrit, les règles du noyau
   étant sans entrée-sortie ; le job `windows` ne change pas.
3. **`synchro` gagne deux fonctions** (C7), quand la décision 7 dit « `synchro` ne change pas ». Retenu : ses
   dépendances et son rôle ne changent pas, ce que la décision 7 garde.

La question C2, qui touche l'ADR-023 et non le plan maître : « faite, avec un avertissement », précision ou
amendement de l'ADR-023 ? Retenu : une précision ; la PR (b) récrit le § 31 · 2.1 en ce sens, et l'ADR-023
ne change pas.

---

## Points de revue

Cinq conditions que la spec implique sans qu'aucune de ses sources ne demande de les éprouver ; chacune a son test,
dans la tâche qui possède le code.

1. **Une ouverture qui échoue rend la garde.** Sinon le fonds ne se rouvre plus sans relancer Pupitre.
   `une_ouverture_qui_echoue_rend_la_garde`, tâche 5.
2. **Après une réception refusée par le disque, l'échange en cours reprend.** La mémoire relue perd ce que
   l'état de synchronisation croit reçu, et l'autre copie, qui range ce qu'elle a envoyé dans `sent_hashes`,
   ne le renverrait jamais : sur les mêmes `Echange`, les deux copies s'enverraient sans fin des messages
   vides. `Echange::reprendre` demande à l'autre copie d'oublier ce qu'elle a envoyé (drapeau `SYNC_RESET`), et
   la réunion converge. `une_reception_refusee_relit_le_document_et_l_echange_reprend`, tâche 6.
3. **Un fichier étranger dans `documents/`** (`desktop.ini`, `Thumbs.db`, que Windows dépose) : ni lu, ni
   retiré, et le document s'ouvre entier. `un_fichier_etranger_est_laisse_tel_quel`, tâche 11, une
   caractérisation : le code d'H07 ne lit déjà que les `.automerge` d'`ajouts/`, et le démarrage doit le garder.
4. **Un nom provisoire dont le sous-dossier de l'empreinte n'existe pas encore** : le rangement au démarrage le
   crée. `une_captation_nommee_prend_son_nom_au_demarrage`, cas du sous-dossier absent, tâche 12.
5. **Un document illisible d'une version de schéma plus ancienne** : il ne migre pas, et rien ne s'écrit.
   `un_document_illisible_ne_migre_pas`, tâche 13.

## Mécanismes annoncés

- `crates/stockage/src/fichier.rs:29` — `std::process::id()` — le voisin d'H07 ne porte que le numéro du
  processus, sans suffixe : un voisin laissé sous ce numéro serait écrasé (décision 8 d'H07 ; tâche 10).
- `crates/stockage/src/fichier.rs:91` — `File::create(chemin)` — `File::create` tronque le fichier qui porte
  déjà ce nom : sans suffixe, un nom provisoire laissé par un arrêt serait tronqué, puis retiré par l'échec
  (tâche 10).
- `crates/stockage/src/documents.rs:557` — `OsStr::new("automerge")` — le code d'H07 ne lit, dans `ajouts/`,
  que les `.automerge` : un fichier étranger n'y est ni lu ni retiré (point de revue 3 ; tâche 11, une
  caractérisation).
- `crates/stockage/src/racine_des_donnees.rs:63` — `verrou.try_lock()` — `ouvrir` prend le verrou avant de
  créer `documents/` et `captations/`, et n'y touche à rien d'autre : `le_demarrage_ne_touche_rien_sans_le_verrou`
  est une caractérisation (tâche 11).
- `crates/stockage/tests/h07.rs:231-232` — `refuser la lecture du fichier` — sous Windows, le verrou que tient
  une racine des données ouverte peut refuser la lecture du fichier `verrou` : le test de la tâche 11 compare
  les empreintes de `documents/` et de `captations/`, jamais celles de la racine des données.
- `crates/stockage/src/documents.rs:192-193` — `&mut self` — `ecrire` prend déjà `&mut self` : aucun message
  ne se prépare pendant une écriture, et les deux doctests sur `Document::ecrire` sont des caractérisations
  (tâche 6).
- `crates/stockage/src/documents.rs:493` — `load_incremental` — `load_incremental` s'arrête sans erreur au
  premier tronçon altéré : seule l'empreinte du nom dit qu'un fichier d'ajout a changé, et un fichier
  illisible n'est jamais retiré (tâches 4 et 7).
- `crates/stockage/src/lib.rs:83` — `DuplicateSeqNumber` — `load_incremental` refuse un fichier d'ajout qui
  donne l'empreinte de son nom, mais porte une modification d'un acteur à un rang déjà pris : `AjoutIllisible`
  (tâche 7).
- `crates/stockage/src/documents.rs:456` — `OnPartialLoad::Ignore` — un fichier complet que `load` refuse se
  relit par `load_with_options`, sans ce qui ne se lit pas : `CompletIllisible` montre ce qui se lit
  (tâche 7).
- `crates/synchro/src/lib.rs:60` — `needs_reset` — `State::needs_reset`, champ public d'automerge 0.12.0 : le
  prochain message demande à l'autre copie d'oublier ce qu'elle croit avoir envoyé (`sent_hashes`), et elle
  le renvoie (point de revue 2 ; tâche 6).
- `crates/synchro/src/lib.rs:41` — `generate_sync_message` — un message dont les modifications sont ôtées ne
  laisse `generate_sync_message` rendre `None` que s'il annonce les heads de l'autre copie (`State::their_heads`,
  champ public en 0.12.0, vide au premier message) ; sans elles, l'échange ne finit pas (Éc-6 ; tâche 14).
- `.cargo/mutants.toml:45` — `sort alors en 3` — sur un TIMEOUT, cargo-mutants sort en 3, ce qui fait
  échouer `mutants` : la recherche de `voisin_libre` est bornée, et chacun de ses mutants panique ou rend un
  autre nom (tâche 9).
- `Cargo.toml:3` — `resolver = "3"` — sans cible de test, `cargo clippy -p stockage --lib` ne tire aucune
  dépendance de test, donc ni la dépendance du stockage sur lui-même ni `pannes` : l'étape « Lint du
  stockage, sans pannes » compile le stockage tel qu'il se livre (tâche 3).
- `crates/stockage/Cargo.toml:21` — `features = ["pannes"]` — cargo admet qu'une crate dépende d'elle-même en
  test, avec une fonction : seuls les tests du stockage activent `pannes` (tâche 3).
- `.github/workflows/ci.yml:146` — `cargo test -p stockage -p synchro` — le job `windows` ne lance que les
  tests de `stockage` et de `synchro` : les tests du noyau ne tournent que dans `verifier` et `mutants`
  (écart 2).
- `.github/workflows/ci.yml:4` — `pull_request:` — `ci.yml` tourne sur une PR, et non à l'envoi d'une autre
  branche que `main` : chaque PR de la tranche s'ouvre en brouillon au premier envoi.

## Outils et cas limites

- `git ls-files` — un fichier indexé par `git add -f`, que le `.gitignore` masque, reste suivi et listé —
  l'essai de la tâche 2 : `essai.automerge` et `pupitre-archive-2026-11-09/LISEZMOI.md`, indexés de force,
  refusés par le contrôle 6
- `git check-ignore` — un fichier indexé échappe à la règle qui le masque — l'essai de la tâche 2, étape 3 :
  les deux fichiers désindexés d'abord, puis `git check-ignore` sans `-v`
- `cargo-mutants` — un mutant qui boucle sans fin, pris par le délai (TIMEOUT), le fait sortir en 3 —
  `le_voisin_prend_le_premier_nom_libre` (tâche 9), sur la recherche bornée de `voisin_libre`, et le job
  `mutants`
- `rustdoc` — un doctest `compile_fail` passe sur toute erreur de compilation, pas seulement la sienne —
  les deux doctests sur `Document::ecrire` (tâche 6), dont le premier dit la sienne, `compile_fail,E0502`
- `load_incremental` — un fichier d'ajout qui donne l'empreinte de son nom, mais porte une modification d'un
  acteur à un rang déjà pris (`DuplicateSeqNumber`) — `un_ajout_qu_automerge_refuse_ouvre_en_lecture_seule`
  (tâche 7)
- `load_incremental` — un fichier d'ajout altéré se charge sans erreur jusqu'à son premier tronçon altéré —
  `un_ajout_altere_ouvre_en_lecture_seule_et_se_nomme` et `la_compaction_garde_un_ajout_altere` (tâche 7)
- `Automerge::load` — un fichier complet altéré, que `load` refuse et que `load_with_options`
  (`OnPartialLoad::Ignore`) relit en partie — `un_complet_illisible_ouvre_en_lecture_seule_et_se_nomme`
  (tâche 7)
- le protocole de synchronisation d'automerge 0.12.0 — après une réception refusée, l'autre copie ne renvoie
  pas ce qu'elle a rangé dans `sent_hashes` — `une_reception_refusee_relit_le_document_et_l_echange_reprend`
  (tâche 6)
- `generate_sync_message` — des messages sans modification qui n'annoncent pas les heads de l'autre copie, et
  l'échange ne finit pas — `une_copie_n_plus_1_recoit_sans_rien_transmettre` (tâche 14), qui borne les
  messages
- `File::create` — un nom provisoire ou un voisin laissé sous le même nom, qu'elle tronquerait —
  `verser_laisse_intact_un_nom_provisoire_laisse` et `un_nom_de_voisin_pris_recoit_un_suffixe` (tâche 10)
- `std::fs::read_dir` — un dossier absent, `ajouts/` à côté d'un seul voisin du complet, dont le `NotFound`
  n'est pas une erreur — `un_voisin_du_complet_vide_laisse_pupitre_neuf` (tâche 11)
- le verrou d'un fichier sous Windows — le fichier `verrou` d'une racine des données ouverte, dont la lecture
  peut être refusée — `le_demarrage_ne_touche_rien_sans_le_verrou` (tâche 11), qui ne lit que `documents/`
  et `captations/`
- `format!("{}")` et `format!("{:e}")` — un `F64` dont l'une ou l'autre forme est la plus courte, `-0`,
  l'infini — `un_f64_s_ecrit_sous_sa_forme_la_plus_courte` (tâche 15)
- `ObjType::Map` et `ObjType::Table` — deux sortes d'objets à clés, qu'Automerge distingue —
  `map_et_table_sont_une_seule_sorte` (tâche 15)
- `get` d'Automerge — deux valeurs concurrentes d'une même clé, dont Automerge ne présente qu'une —
  `la_teneur_prend_la_valeur_que_presente_automerge` (tâche 15)

## Revues

- la revue de chaque tâche : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12, Tâche 13, Tâche 14, Tâche 15, Tâche 16

---

## La carte des fichiers

```text
crates/noyau/src/donnees/
  mod.rs            + pub mod acces; demarrage; teneur; version;
  acces.rs          (b, d) Acces : ce que permet un document ouvert                       (muté)
  compaction.rs     (b, c) a_l_ouverture, apres_une_ecriture avec l'accès et « due » ; Lu::lisible ; a_retirer
  demarrage.rs      (c) voisin_libre, fichier_du_voisin, sort_d_un_partiel, pupitre_neuf, a_ranger (muté)
  version.rs        (d) Migration, Lecture, lecture, connue, a_appliquer, message_de_la_migration (muté)
  teneur.rs         (d) Valeur, ErreurDeTeneur, ecrire_la_teneur                           (muté)
  copie.rs          (d) + Rapport, rapport_des_lignees                                     (muté)
  fonds.rs          (d) + LIGNEES_REMPLACEES, MIGRATIONS
crates/noyau/tests/
  donnees.rs        l'accès, la compaction, la version, la lignée
  demarrage.rs      (c) ; teneur.rs (d)
crates/synchro/src/lib.rs        (b) + Echange::reprendre ; (d) + Echange::prochain_message_sans_modification
crates/synchro/tests/echange.rs  (d) + le test N + 1
crates/stockage/
  Cargo.toml        (b) [features] pannes ; dépendance de test sur elle-même
  src/pannes.rs     (b) Etape, point, relue ; sous la fonction : Panne, couper, alterer_la_relecture
  src/fichier.rs    (b) Echec, ecrire_a_cote relit avant de renommer ; les étapes ; (c) voisin suffixé
  src/lib.rs        Erreur, LectureSeule, Avertissement ; (d) teneur
  src/racine_des_donnees.rs  (b) la garde, la doc de fonds ; (d) fonds_selon, sous pannes
  src/documents.rs  (b) écriture, relecture, lecture seule ; (c) démarrage ; (d) migration, envoi
  src/captations.rs (b) les étapes ; (c) suffixe, ranger_les_noms_provisoires
  tests/commun/mod.rs        (a) les aides ; (d) migration_de_test
  tests/fichier.rs, captations.rs, ajouts.rs, compaction.rs, fonds.rs, synchronisation.rs
  tests/demarrage.rs (c) ; tests/version.rs, teneur.rs, references.rs (d) ; (d) les marqueurs des tests d'H07, h07.rs compris
app/src-tauri/src/lib.rs   (a) dossier_jetable
scripts/verif-confidentialite.sh, .gitignore   (a)
.cargo/mutants.toml, .github/workflows/ci.yml, .github/workflows/mutants-complet.yml   (b, c, d) délais
.github/workflows/ci.yml   (b) « Lint du stockage, sans pannes », dans verifier et windows
docs/31-persistance-et-synchronisation.md   § 2.1, 2.2 (b, c), 6.1 et 10, test 4 (c), 2.4, 2.6 (d)
docs/80-glossaire.md   Lecture seule (b), Voisin (c) ; Lecture seule (d), si le repli de la tâche 14 joue
docs/40-architecture.md § 3, CLAUDE.md   pannes, et la commande sans pannes (b) ; fonds_selon (d)
CONVENTIONS.md   contenu (a), mutants (b, c, d), verifier et windows (b)
docs/registre.md   LE-10 (b), AM-4 et LE-12 (d) ; la contradiction (d), si le repli de la tâche 14 joue
```

---

## PR (a) · les aides de tests et la porte contenu

**Branche :** `t1a-aides-et-contenu`. **Titre :** `test: les aides des tests du stockage en un seul endroit, la
porte contenu refuse un fonds` (90). **Documents :** `CONVENTIONS.md` (ligne `contenu`). **Fin :** `contenu`,
`documentation`, `verifier`, `windows` et `mutants` verts (rien à muter) ; l'essai de la tâche 2 dans la
description ; aucune remesure ; `gh pr ready <n>`, puis fusion par squash.

### Tâche 1 : les aides des tests, une fois par crate

**Fichiers :** modifier `crates/stockage/tests/commun/mod.rs`, `tests/fonds.rs` (:19-45), `tests/ajouts.rs`
(:18-50), `tests/compaction.rs` (:18-60), `tests/synchronisation.rs` (:20-68), `tests/h07.rs` (:24-35, :135-160,
:262), `tests/captations.rs` (:13) ; `app/src-tauri/src/lib.rs` (:165 et ses appels).

**Interfaces — produit** (`crates/stockage/tests/commun/mod.rs`) :

```rust
/// La naissance des tests : des identifiants écrits, jamais tirés.
pub const NAISSANCE: Naissance<'static> =
    Naissance { identifiant: "fonds-1", lignee: "lignee-1", proprietaire: "proprietaire-1" };
/// L'heure de chaque modification des tests : une entrée, jamais l'horloge.
pub const HEURE: i64 = 1_700_000_000;
pub const MESSAGE_DU_GESTE: &str = "Idée notée";
pub fn ouvrir(chemin: &Path) -> RacineDesDonnees;
pub fn ouvrir_le_fonds(racine: &RacineDesDonnees) -> Document<'_>;
/// Le fonds né d'une idée au texte long : `idee-1`, `texte_long()`, `MESSAGE_DU_GESTE`, `HEURE`.
pub fn faire_naitre(racine: &RacineDesDonnees) -> Document<'_>;
pub fn dossier_du_fonds(racine_des_donnees: &Path) -> PathBuf;
pub fn dossier_des_ajouts(racine_des_donnees: &Path) -> PathBuf;
pub fn lire_le_complet(racine_des_donnees: &Path) -> Vec<u8>;
pub fn poids(chemin: &Path) -> u64;
pub fn rangee(racine_des_donnees: &Path, e: &str) -> PathBuf;
/// L'objet à clés (`ObjType::Map`) que porte `cle` sous `parent` ; sinon, panique.
pub fn objet(doc: &Automerge, parent: &ObjId, cle: &str) -> ObjId;
/// Le fonds né (`faire_naitre`) dans `<dossier jetable>/a`, fermé, puis copié dans `<…>/b` : A, B.
pub fn deux_copies(nom: &str) -> (PathBuf, PathBuf);
/// Le seul chemin d'écriture des tests de la tranche : la tranche 2 le changera seul.
pub fn ecrire(
    document: &mut Document<'_>,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<(), Erreur>; // = document.ecrire(MESSAGE_DU_GESTE, HEURE, geste)
```

L'aide privée `objet` sur `&Transaction` (:247) devient `conteneur`. `copie` (noyau, synchro), `reunir` et
`HEURE` de `synchro` gardent un exemplaire par crate : un fichier de tests ne voit pas le `tests/commun` d'une
autre. Dans `etabli`, `fn dossier(nom)` devient `fn dossier_jetable(nom: &str) -> PathBuf` :
`temp_dir()/pupitre-etabli-<nom>-<processus>`, effacé (`NotFound` toléré, toute autre erreur panique) puis
recréé, comme celui de `stockage`, et sa doc dit « le seul endroit des tests d'`etabli` qui devine un chemin ».

- [ ] **Étape 1 : relever.** `git grep -n -E "const (NAISSANCE|HEURE|MESSAGE_DU_GESTE)|fn (ouvrir|ouvrir_le_fonds|faire_naitre|deux_copies|dossier_du_fonds|dossier_des_ajouts|lire_le_complet|poids|rangee|objet|dossier|dossier_jetable)\b" -- crates/stockage/tests app/src-tauri/src`. Pas de rouge : un déplacement sans changement de comportement.
- [ ] **Étape 2 : déplacer.** Chaque définition recopiée disparaît de son fichier, et les chaînes inlinées (`.fonds().expect(…).expect(…)`, `dossier_du_fonds` inliné, la naissance de `deux_copies`) appellent l'aide ; `deux_copies` monte de `synchronisation.rs` dans `commun`, que la tâche 13 emploie aussi. Les naissances de `fonds.rs`, qui éprouvent la naissance même, restent écrites en entier. Le `rangee` de `captations.rs` prenait un `&RacineDesDonnees`, celui de `commun` prend un `&Path`, comme celui de `h07.rs` : ses appels deviennent `commun::rangee(racine.chemin(), &e)`.
- [ ] **Étape 3 : voir passer.** Committer (`test: les aides des tests du stockage, une fois par crate`), pousser, ouvrir la PR en brouillon. Attendu : dans `verifier` et `windows`, les neuf fichiers de `crates/stockage/tests` gardent leurs comptes (8, 4, 4, 4, 4, 4, 4, 3, 4 : 39) ; dans `windows`, « Tests de l'établi » vert ; puis le même `git grep` ne trouve chaque définition qu'une fois, dans `commun/mod.rs` (`dossier_jetable` compris) ou dans `app/src-tauri/src/lib.rs` (`dossier_jetable` seul, `fn dossier` n'y paraissant plus).

### Tâche 2 : la porte contenu refuse un fonds suivi

**Fichiers :** modifier `scripts/verif-confidentialite.sh` (après le contrôle 5, :48-52, avant le `if` final,
pour que les contrôles se lisent de 1 à 6), `.gitignore` (après la section audio, :29), `CONVENTIONS.md` (ligne
`contenu`, :111).

- [ ] **Étape 1 : l'essai, rouge, sur le poste.** `mkdir pupitre-archive-2026-11-09`, puis y écrire `LISEZMOI.md` (« essai »), écrire `essai.automerge` (« essai ») à la racine, `git add -f essai.automerge pupitre-archive-2026-11-09/LISEZMOI.md`, `bash scripts/verif-confidentialite.sh`. Attendu aujourd'hui : `OK — …`, sortie 0.
- [ ] **Étape 2 : écrire le contrôle 6**, sur les noms, par `git ls-files` brut, sans `EXCLUS` ni exemption :

```bash
# 6. Un document ou une archive d'un fonds versionnes : un fonds ne se versionne jamais
#    (§ 40 · 9). Aucun dossier n'est exempte : la PR qui suivrait un document de test
#    ajouterait l'exemption, et le dirait.
trouve=$(git ls-files | grep -E '\.automerge$|(^|/)pupitre-archive-[^/]*/' || true)
[ -n "$trouve" ] && signale "document ou archive d'un fonds versionne" "$trouve"
```

  Au `.gitignore`, une section « Documents et archives d'un fonds : un fonds ne se versionne jamais (§ 40 · 9). » avec `*.automerge` et `pupitre-archive-*/`. Ligne `contenu` de `CONVENTIONS.md`, colonne « Ce qu'elle refuse » : « … chemin absolu de machine ; un document (`*.automerge`) ou une archive (`pupitre-archive-*/`) d'un fonds, suivis, sans exemption » ; colonne « Pourquoi » : « … Un fonds ne se versionne jamais (§ 40 · 9) : aucun dossier n'en est exempté ».
- [ ] **Étape 3 : l'essai, vert.** Relancer le script : sortie 1, `REFUSE — document ou archive d'un fonds versionne`, les deux chemins nommés. Puis `git restore --staged essai.automerge pupitre-archive-2026-11-09/LISEZMOI.md` ; alors seulement (un fichier indexé échappe à `check-ignore`), `git check-ignore essai.automerge pupitre-archive-2026-11-09/LISEZMOI.md` (sans `-v`) : sortie 0. Retirer les deux fichiers et le dossier, relancer : `OK`. Copier les trois sorties dans la description de la PR.
- [ ] **Étape 4 : committer** `git add scripts/verif-confidentialite.sh .gitignore CONVENTIONS.md`, (`ci: la porte contenu refuse un document ou une archive d'un fonds suivis`) ; `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add` ; pousser. Attendu : `contenu` et `documentation` verts.

---

## PR (b) · l'injection, la garde, la relecture, la lecture seule, les avertissements

**Branche :** `t1b-pannes-et-lecture-seule`. **Titre :** `feat: le stockage tient une panne à chaque étape et
s'ouvre en lecture seule (AM-4, LE-10)` (90). **Documents :** § 31 · 2.1 et 2.2 ; glossaire ; § 40 · 3 ;
`CLAUDE.md` ; `CONVENTIONS.md` (`mutants`, `verifier`, `windows`) ; `.cargo/mutants.toml` ; `ci.yml` (l'étape
sans pannes, les délais), `mutants-complet.yml` (délais) ; registre (LE-10 clos). **Fin :** voir la tâche 8.

### Tâche 3 : le point d'injection et les étapes d'une écriture

**Fichiers :** créer `crates/stockage/src/pannes.rs` ; modifier `crates/stockage/Cargo.toml`, `src/lib.rs`
(:6-20), `src/fichier.rs` (tout), `src/captations.rs` (:35-53, :105-150), `src/documents.rs` (les trois appels
d'`ecrire_a_cote`, :141, :185, :358 : `Refusee` et `NonDurable` rendent encore `Erreur::Disque`, la tâche 6
change ce sens), `.github/workflows/ci.yml` (`verifier` et `windows` : l'étape « Lint du stockage, sans
pannes ») ; tests : `tests/fichier.rs`, `tests/captations.rs`.

**Interfaces — produit :**

```rust
// Cargo.toml : [features] pannes = []  ;  [dev-dependencies] stockage = { path = ".", features = ["pannes"] }
// lib.rs : #[cfg(feature = "pannes")] pub mod pannes;  #[cfg(not(feature = "pannes"))] mod pannes;
// `Etape`, `point` et `relue` existent dans les deux configurations, que leurs appels emploient sans
// condition ; `Panne`, `couper` et `alterer_la_relecture`, sous la fonction seule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etape { VoisinEcrit, SyncAll, Relecture, Renommage, RendreDurable, Save, Retrait, Reception }
/// La panne armée : elle se désarme à sa chute.
#[cfg(feature = "pannes")]
#[must_use]
pub struct Panne { /* privé */ }
/// La `rang`-ième fois (dès 1) que `etape` survient dans ce fil depuis cet appel, elle échoue, une fois.
#[cfg(feature = "pannes")]
pub fn couper(etape: Etape, rang: usize) -> Panne;
/// La `rang`-ième lecture de `fichier::lire` (dès 1) dans ce fil depuis cet appel réussit, mais rend ses
/// octets altérés : le premier inversé (`!`), ou un octet ajouté si rien n'est lu. Une fois.
#[cfg(feature = "pannes")]
pub fn alterer_la_relecture(rang: usize) -> Panne;
pub(crate) fn point(etape: Etape) -> io::Result<()>; // Ok(()) sans la fonction `pannes`
pub(crate) fn relue(octets: Vec<u8>) -> Vec<u8>;     // les octets tels quels sans elle
// stockage::fichier
#[derive(Debug)]
pub enum Echec {
    /// Avant la fin du renommage : `chemin` garde ce qu'il portait, et le voisin est retiré.
    Refusee(io::Error),
    /// Après le renommage : le fichier est en place, mais `rendre_durable` a échoué.
    NonDurable(io::Error),
}
pub fn ecrire_a_cote(chemin: &Path, octets: &[u8]) -> Result<(), Echec>;
pub(crate) fn ecrire_et_synchroniser(chemin: &Path, octets: &[u8]) -> io::Result<()>; // VoisinEcrit, SyncAll
pub(crate) fn lire(chemin: &Path) -> io::Result<Vec<u8>>;                            // Relecture, relue
pub(crate) fn renommer(de: &Path, vers: &Path) -> io::Result<()>;                    // Renommage
pub(crate) fn retirer(chemin: &Path) -> io::Result<()>;                              // Retrait
pub(crate) fn rendre_durable_apres_le_renommage(chemin: &Path) -> io::Result<()>;    // RendreDurable
```

Les points : `VoisinEcrit` après `File::create`, avant `write_all` ; `SyncAll` avant `sync_all` ; `Relecture`
avant `fs::read` ; `Renommage` avant `fs::rename` ; `Retrait` avant `fs::remove_file` ; `RendreDurable` avant
le `rendre_durable` qui suit un renommage (la synchronisation d'un dossier né, sous Linux, n'en est pas un) ;
`Save` et `Reception` vivent dans `documents.rs` (tâche 6). Sous `pannes`, un `thread_local!` tient les
pannes armées, chacune avec son compte ; `point` rend `Err(io::Error::other(format!("panne injectée :
{etape:?}")))` au passage voulu ; `lire` passe ce qu'il lit par `relue`, que seule `alterer_la_relecture`
change. `ecrire_a_cote` : `voisin`, `ecrire_et_synchroniser`, `lire` du voisin, qui
doit rendre `octets` (sinon `Refusee`, `io::ErrorKind::InvalidData`), `renommer`, puis
`rendre_durable_apres_le_renommage`. Sur `Refusee`, `retirer(voisin)` ; s'il échoue, le voisin se tronque à zéro
octet (`File::create`), erreur tue (C4). `verser` passe par ces mêmes fonctions, à la place de son `ecrire` et
de ses `fs::read`, `fs::rename`, `rendre_durable`.

Chaque commande de la CI qui nomme `stockage` construit aussi ses cibles de test, donc sa dépendance de test
sur elle-même, donc `pannes` ; et `etabli` ne dépend de `stockage` qu'en tranche 3. La configuration livrée ne
se compilerait donc jamais : une étape « Lint du stockage, sans pannes »,
`cargo clippy -p stockage --lib -- -D warnings`, la compile et la passe à `clippy`, dans `verifier` après
« Lint », dans `windows` après « Lint du stockage et de la synchronisation ». Sans cible de test, elle ne tire aucune dépendance de test (résolveur 3).

Si cargo refuse la dépendance de `stockage` sur elle-même, `--features stockage/pannes` s'ajoute aux étapes
Lint et Tests de `verifier` et de `windows` qui nomment `stockage`, et aux « Commandes » de `CLAUDE.md` ;
l'étape sans pannes reste telle. Si l'étape « Cargo.lock à jour » échoue, committer l'artefact `cargo-lock`.

- [ ] **Étape 1 : écrire les tests.** Dans `tests/fichier.rs` :

```rust
// § 31 · 10, test 8 : interrompre une écriture à chaque étape
#[test]
fn ecrire_a_cote_coupee_a_chaque_etape() {
    for (i, etape) in [Etape::VoisinEcrit, Etape::SyncAll, Etape::Relecture, Etape::Renommage]
        .into_iter().enumerate()
    {
        let d = dossier_jetable(&format!("fichier-coupee-{i}"));
        let chemin = d.join("complet.automerge");
        fs::write(&chemin, b"avant").unwrap();
        let _panne = pannes::couper(etape, 1);
        assert!(matches!(ecrire_a_cote(&chemin, b"apres"), Err(Echec::Refusee(_))), "{etape:?}");
        assert_eq!(fs::read(&chemin).unwrap(), b"avant");
        assert_eq!(entrees(&d), ["complet.automerge"]);
    }
    // RendreDurable : Err(Echec::NonDurable(_)), le fichier porte "apres", aucun voisin
}

// § 31 · 10, test 8 : renommage refusé, le voisin est retiré ; sa double panne le vide
#[test]
fn un_voisin_qui_ne_se_retire_pas_se_vide() {
    // couper(Renommage, 1) et couper(Retrait, 1) : Err(Echec::Refusee(_)) ; le fichier porte
    // "avant" ; le voisin, nommé en clair et non par `voisin()`, que la tâche 10 change,
    // chemin.with_file_name(format!("complet.automerge.{}.partiel", std::process::id())),
    // existe et fs::metadata(..).len() == 0
}

// LE-10, § 31 · 2.2, point 3 : un voisin relu qui ne rend pas les octets écrits ne prend pas la place
#[test]
fn un_voisin_relu_autre_ne_prend_pas_la_place() {
    // fs::write(chemin, "avant") ; pannes::alterer_la_relecture(1) : ecrire_a_cote(chemin, b"apres")
    // rend Err(Echec::Refusee(e)), e.kind() == io::ErrorKind::InvalidData ; le fichier porte
    // "avant" ; entrees(&d) == ["complet.automerge"]
}
```

  Dans `tests/captations.rs` :

```rust
// § 31 · 10, test 4 : couper l'écriture à chaque étape
#[test]
fn verser_coupe_a_chaque_etape_ne_laisse_aucune_captation_alteree() {
    // pour VoisinEcrit, SyncAll, Relecture, Renommage, chacun dans son dossier jetable :
    //   verser(&octets(7, 500)) → Err(Erreur::Disque { .. }) ; verifier() == [] ;
    //   rangee(..) absente ; aucun fichier dans captations/ même ; puis, sans panne,
    //   verser rend l'empreinte et relire la rend octet pour octet.
    // RendreDurable : Err(Erreur::Disque { .. }), mais la captation est rangée, intacte,
    //   et verifier() == [].
}
```

- [ ] **Étape 2 : voir échouer.** Committer (`test: une écriture coupée à chaque étape`), pousser. Attendu : `verifier` rouge à « Lint », `unresolved import stockage::pannes`.
- [ ] **Étape 3 : écrire `pannes.rs`, la fonction de Cargo, et `fichier.rs`**, puis brancher `captations.rs` et les trois appels de `documents.rs` ; ajouter à `ci.yml` l'étape « Lint du stockage, sans pannes », dans `verifier` et `windows`.
- [ ] **Étape 4 : voir passer.** Committer (`feat: le point d'injection pannes, sur chaque étape d'une écriture`), pousser. Attendu : `tests/fichier.rs` (7 tests) et `tests/captations.rs` (5) : `test result: ok.` dans `verifier` et `windows` ; les 39 toujours verts ; « Lint du stockage, sans pannes » vert dans les deux jobs.

### Tâche 4 : l'accès d'un document, et la compaction qui le reçoit

**Fichiers :** créer `crates/noyau/src/donnees/acces.rs` ; modifier `donnees/mod.rs`, `donnees/compaction.rs`
(:5-42), `crates/noyau/tests/donnees.rs`, `.cargo/mutants.toml`, `CONVENTIONS.md` (ligne `mutants`, :115) ;
`crates/stockage/src/documents.rs` (appels : `Acces::Entier`, `due` à `false` jusqu'à la tâche 6 ; `compacter`
perd sa relecture d'après le renommage, :186-188, et `meme_point` ; `Lu { lisible: est_conforme(…), … }`, :194-202).

**Interfaces — produit :**

```rust
// noyau::donnees::acces
/// Ce que permet un document ouvert (§ 31 · 2.1, § 31 · 2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acces {
    /// Tout se lit.
    Entier,
    /// Un fichier ne se lit pas, ou la relecture du disque a échoué.
    Illisible,
}
impl Acces {
    /// Il écrit une opération, se compacte et transmet ses modifications : `Entier` seul.
    pub fn ecrit(self) -> bool;
    /// Il envoie et reçoit des messages de synchronisation : `Entier` seul.
    pub fn echange(self) -> bool;
}
// noyau::donnees::compaction
/// Au-delà de ce nombre de fichiers d'ajout, une écriture compacte un document qui écrit.
pub const AJOUTS_AU_PLUS: usize = 100;
pub fn a_l_ouverture(acces: Acces, ajouts: usize, voisin_du_complet: bool) -> bool;
// = acces.ecrit() && (ajouts > 0 || voisin_du_complet)
pub fn apres_une_ecriture(acces: Acces, due: bool, octets_du_complet: u64, octets_des_ajouts: u64, ajouts: usize) -> bool;
// = acces.ecrit() && (due || octets_des_ajouts > octets_du_complet || ajouts > AJOUTS_AU_PLUS)
pub struct Lu {
    pub nom: String,
    /// Lisible : son contenu donne l'empreinte de son nom.
    pub lisible: bool,
    pub absorbe: bool,
}
pub fn a_retirer(lus: &[Lu]) -> Vec<String>; // les lisibles et absorbés, dans l'ordre reçu
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/donnees.rs`) ; `a_retirer_ne_retire_rien_si_le_complet_relu_mene_ailleurs` disparaît : la relecture garde désormais le renommage (Éc-3), et deux tests de `stockage` l'éprouvent, sous `pannes::alterer_la_relecture`, que `stockage` n'étant pas sous mutation demande : `un_voisin_relu_autre_ne_prend_pas_la_place` (tâche 3) et `une_compaction_dont_le_complet_se_relit_autre_ne_retire_rien` (tâche 7). `a_retirer_garde_un_fichier_non_conforme` devient `a_retirer_garde_un_fichier_illisible`, l'aide `lu(nom, conforme, absorbe)` devient `lu(nom, lisible, absorbe)`, et la doc de `quatre_lus` dit « lisible ».

```rust
#[test]
fn l_acces_dit_ce_que_permet_un_document() {
    assert!(Acces::Entier.ecrit() && Acces::Entier.echange());
    assert!(!Acces::Illisible.ecrit() && !Acces::Illisible.echange());
}
#[test]
fn compacter_a_l_ouverture() {
    assert!(!a_l_ouverture(Acces::Entier, 0, false));
    assert!(a_l_ouverture(Acces::Entier, 1, false));
    assert!(a_l_ouverture(Acces::Entier, 0, true));
    assert!(!a_l_ouverture(Acces::Illisible, 3, false));
    assert!(!a_l_ouverture(Acces::Illisible, 0, true));
}
#[test]
fn compacter_apres_une_ecriture() {
    assert!(!apres_une_ecriture(Acces::Entier, false, 100, 100, 1)); // l'égalité ne suffit pas
    assert!(apres_une_ecriture(Acces::Entier, false, 100, 101, 1));
    assert!(!apres_une_ecriture(Acces::Entier, false, 10_000, 10, 100));
    assert!(apres_une_ecriture(Acces::Entier, false, 10_000, 10, 101));
    assert!(apres_une_ecriture(Acces::Entier, true, 10_000, 10, 1));   // compaction due
    assert!(!apres_une_ecriture(Acces::Illisible, true, 100, 101, 101));
}
#[test]
fn a_retirer_garde_un_fichier_illisible() {
    // [a lisible absorbé, b illisible absorbé, c lisible non absorbé, d lisible absorbé]
    assert_eq!(a_retirer(&quatre_lus()), ["a", "d"]);
}
```

- [ ] **Étape 2 : voir échouer.** Committer (`test: l'accès d'un document, et la compaction qui le reçoit`), pousser. Attendu : « Lint » rouge, `unresolved import noyau::donnees::acces`.
- [ ] **Étape 3 : écrire `acces.rs` et les nouvelles signatures.** La doc d'`a_retirer` (`compaction.rs`:30-33) suit le mot et la signature : « Points 3 et 4 : les seuls fichiers lisibles et absorbés, dans l'ordre reçu. Un fichier illisible n'est jamais retiré : un fichier altéré se lit sans erreur jusqu'à son premier tronçon altéré, et seule l'empreinte de son nom le dit », sa condition sur le fichier complet relu tombant avec `meme_point`. `.cargo/mutants.toml` : `crates/noyau/src/donnees/acces.rs` dans `examine_globs` ; en tête (l. 2-3), « Dix modules » et « les règles des données (la compaction, l'accès d'un document et la comparaison des copies) » ; l. 17, « ces dix modules ». `CONVENTIONS.md`, ligne `mutants` : « …les règles des données : la compaction, l'accès d'un document et la comparaison des copies ».
- [ ] **Étape 4 : voir passer.** Committer (`feat: l'accès d'un document, au noyau, que la compaction reçoit`), pousser. Attendu : `tests/donnees.rs` : `test result: ok. 5 passed` dans `verifier` ; `mutants` vert, sans `MISSED`.

### Tâche 5 : un seul `Document` du fonds à la fois

**Fichiers :** modifier `crates/stockage/src/racine_des_donnees.rs` (:13-19, :76-94, doc :62-75),
`src/documents.rs` (:32-45, ôter le `#[expect(dead_code)]` ; `impl Drop` ; :247, :333), `src/lib.rs`
(`Erreur`) ; tests : `tests/fonds.rs`.

**Interfaces — produit :**

```rust
// RacineDesDonnees : + fonds_ouvert: AtomicBool (Sync, pour l'état de Tauri en tranche 3)
// Erreur : + FondsDejaOuvert
/// Un `Document` du fonds vit déjà dans ce processus : un seul à la fois (AM-4 (1)).
// `fonds()` et `faire_naitre_le_fonds()` prennent la garde (`compare_exchange(false, true)`) avant
// toute lecture, et la rendent s'ils rendent `Err` ou `Ok(None)` ; `Drop for Document` la rend.
```

- [ ] **Étape 1 : écrire les tests** (`tests/fonds.rs`) :
  - `un_second_fonds_est_refuse` : `a = faire_naitre(&racine)` vivant ; `racine.fonds()` rend `Err(Erreur::FondsDejaOuvert)`, `racine.faire_naitre_le_fonds(&NAISSANCE, …)` aussi ; `drop(a)` ; `racine.fonds()` rend `Ok(Some(_))`.
  - `une_ouverture_qui_echoue_rend_la_garde` (point de revue 1) : un fonds né puis lâché ; `ajouts/` remplacé par un fichier ; `fonds()` rend `Err(Erreur::Disque { .. })` ; `ajouts/` rendu ; `fonds()` rend `Ok(Some(_))`. Puis, dans une racine neuve : `fonds()` rend `Ok(None)`, et `faire_naitre_le_fonds` réussit.
- [ ] **Étape 2 : voir échouer.** Committer (`test: un seul Document du fonds à la fois`), pousser. Attendu : « Lint » rouge, `no variant … FondsDejaOuvert`.
- [ ] **Étape 3 : écrire la garde.**
- [ ] **Étape 4 : voir passer.** Committer (`feat: la garde d'un seul Document du fonds (AM-4)`), pousser. Attendu : `tests/fonds.rs` : `ok. 6 passed`, dans `verifier` et `windows`.

### Tâche 6 : faite, faite avec un avertissement, ou refusée et relue du disque

**Fichiers :** modifier `crates/stockage/src/lib.rs` (`Erreur` :24-66 ; `LectureSeule`, `Avertissement`),
`src/documents.rs` (:32-212, :331-401), `tests/commun/mod.rs` (`ecrire`), `crates/synchro/src/lib.rs`
(`Echange::reprendre`) ; tests : `tests/ajouts.rs` (et, à `un_geste_refuse_n_ecrit_rien_et_le_document_reste_ouvert`,
:183, `Ok(())` devient `Ok(_)`, l'écriture rendant désormais une `Option` ; sa doc de module, :3-4, dont « une
écriture qui échoue laisse le document à relire » devient « une écriture refusée relit le document du disque »), `tests/compaction.rs`,
`tests/synchronisation.rs` (et sa doc de module, :3, qui dit « un document à relire »), `tests/fonds.rs`, et deux
doctests sur `Document::ecrire`.

**Interfaces — consomme :** `pannes::{couper, Etape}`, `fichier::Echec` (tâche 3), `Acces`,
`apres_une_ecriture` (tâche 4). **Produit :**

```rust
/// Pourquoi un document est en lecture seule (§ 31 · 2.1) : il montre ce qui se lit, n'écrit ni ne
/// se compacte, n'envoie ni ne reçoit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LectureSeule {
    /// La relecture du disque, après une écriture refusée, a échoué sur `chemin` : le document
    /// montre sa mémoire d'avant l'écriture refusée.
    RelectureManquee { chemin: PathBuf },
}
impl LectureSeule { pub fn acces(&self) -> Acces; }
/// Ce qui a échoué après qu'une écriture est faite, sans la défaire (§ 31 · 2.1).
#[derive(Debug)]
pub enum Avertissement {
    /// Le fichier `chemin` est en place, mais son renommage n'a pas été rendu durable.
    NonDurable { chemin: PathBuf, erreur: io::Error },
    /// La compaction a échoué (`Disque` ou `CompactionInfidele`) : rien n'est retiré, et elle se
    /// retente à l'écriture suivante.
    Compaction(Erreur),
}
// Erreur : − Genese, − ARelire ; + LectureSeule(LectureSeule)
impl Document<'_> {
    pub fn ecrire(&mut self, message: &str, heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
    ) -> Result<Option<Avertissement>, Erreur>;
    pub fn recevoir(&mut self, echange: &mut Echange, message: &[u8]) -> Result<Option<Avertissement>, Erreur>;
    pub fn lecture_seule(&self) -> Option<&LectureSeule>;
    /// Les avertissements de la naissance ou de l'ouverture, et celui qu'une réception refusée n'a
    /// pu rendre, son refus se rendant à sa place.
    pub fn avertissements(&self) -> &[Avertissement];
}
// commun::ecrire rend désormais Result<Option<Avertissement>, Erreur>.
impl synchro::Echange {
    /// § 31 · 2.1 : la copie locale a perdu ce qu'elle avait reçu (une réception refusée, puis la
    /// relecture du disque). Son prochain message demande à l'autre copie d'oublier ce qu'elle croit
    /// avoir envoyé (`State::needs_reset`, champ public d'automerge 0.12.0 : le drapeau `SYNC_RESET`, ou
    /// des heads vides devant une copie qui ne le connaît pas), et l'autre copie le renvoie.
    pub fn reprendre(&mut self);
}
```

`Document` : `a_relire` disparaît ; viennent `lecture_seule: Option<LectureSeule>`, `compaction_due: bool`,
`avertissements: Vec<Avertissement>`. L'écriture d'un fichier d'ajout (opération ou réception) :
`Echec::Refusee` → relecture du disque, puis `Err(Erreur::Disque { chemin, … })` ; `Echec::NonDurable` →
`Ok(Some(NonDurable))`, `compaction_due` levé, pas de compaction ; en place → la compaction quand
`apres_une_ecriture(acces, compaction_due, …)` le dit, son échec (la pesée comprise) donnant
`Ok(Some(Compaction(e)))`. La relecture du disque et l'ouverture partagent une fonction interne, qui ne
construit aucun `Document` et ne touche pas à la garde (tâche 5) : `lire_le_dossier(dossier: &Path) ->
Result<(Automerge, Option<LectureSeule>, usize), Erreur>`, la mémoire lue, la première raison de lecture seule
(tâche 7) et le nombre de fichiers d'ajout lus ; elle lit le fichier complet, puis les fichiers d'ajout, chacun
par `fichier::lire`, sans compaction ni démarrage. La relecture remplace la mémoire par la sienne et tire un
acteur neuf ; un `Err` de cette lecture laisse `fork_at(heads d'avant)` de la mémoire, en
`RelectureManquee { chemin }`, `chemin` celui du fichier refusé, ou le dossier du document si l'erreur n'en
nomme aucun. Dans `recevoir`, après la relecture qui suit un `Echec::Refusee`, `echange.reprendre()`, puis
`Err(Erreur::Disque)`. Une réception que `synchro` refuse, mais qui a changé les heads, écrit son fichier d'ajout
comme une autre ; si cette écriture donne un avertissement (`NonDurable`, ou la compaction qui suit), `recevoir`
rend `Err(Erreur::Synchro(_))` et l'avertissement rejoint `avertissements()` : le refus de la réception se dit, et
l'avertissement ne se perd pas. `compacter` passe `point(Etape::Save)` après `save()` : en échec, comme des octets qui ne se
relisent pas, `CompactionInfidele`, et rien ne s'écrit. Dans `recevoir`, quand les heads ont changé,
`point(Etape::Reception)` en échec remplace un `Ok` de la réception par
`Err(synchro::Erreur::Automerge(AutomergeError::Fail))`, et non `MessageIllisible`, dont la doc promet que le
document ne bouge pas : seule une panne injectée rend ce cas. La
naissance : `NonDurable` → le document, avec `avertissements() == [NonDurable]`. `Genese` disparaît :
`GENESE` est une constante du noyau, que `la_genese_se_reconstruit_a_l_octet` éprouve, et
`naitre_en_memoire` la lit par `expect("la genèse du noyau se lit")`. En lecture seule, `ecrire`, `compacter`
et `recevoir` rendent `Err(Erreur::LectureSeule(raison.clone()))`, et `prochain_message` rend `None`, selon
`acces().ecrit()` et `acces().echange()`.

- [ ] **Étape 1 : écrire les tests.** Chaque écriture des tests nouveaux passe par `commun::ecrire`. Quatre tests sont des caractérisations, qui passent dès qu'ils compilent, l'essai faisant déjà ce qu'ils vérifient : `un_message_illisible_rend_synchro`, le complément de `ce_qui_est_recu_se_relit_du_disque`, et les deux doctests sur `Document::ecrire`, qui passent aujourd'hui, `ecrire` prenant déjà `&mut self` ; la description de la PR les relève comme tels.
  - `tests/ajouts.rs` :
    - `une_ecriture_refusee_relit_le_document_du_disque` (réécrit `une_ecriture_qui_echoue_laisse_le_document_a_relire` ; AM-4 (2)) — `// § 31 · 10, test 8 : relire du disque après une opération refusée, puis une opération`. `couper(Renommage, 1)` ; `ecrire` (idee-2) rend `Err(Erreur::Disque { .. })` ; `lecture_seule() == None` ; heads égales à celles d'avant ; `get_actor()` différent d'avant ; aucun fichier de plus dans `ajouts/` ; `racine.fonds()` rend `Err(Erreur::FondsDejaOuvert)`, la relecture n'ayant pas rendu la garde. Puis `ecrire` (idee-3) rend `Ok(None)` ; son fichier d'ajout vaut exactement `save_after(&heads_avant)` et `get_changes(&heads_avant).len() == 1`.
    - `une_ecriture_coupee_a_chaque_etape` — `// § 31 · 10, test 8 : interrompre à chaque étape, puis rouvrir`. Pour `VoisinEcrit`, `SyncAll`, `Relecture`, `Renommage` (rang 1), chacun dans son dossier : `Err(Erreur::Disque { .. })`, aucun `.partiel` dans `ajouts/`, et le fonds rouvert a les heads d'avant. Pour `RendreDurable` : `Ok(Some(Avertissement::NonDurable { chemin, .. }))`, `chemin` sous `ajouts/` et présent, et le fonds rouvert porte l'idée.
    - `une_relecture_qui_echoue_met_le_document_en_lecture_seule` (C1 ; décision 12 du plan maître : « Une relecture du disque qui échoue met le document en lecture seule », « Une captation se verse toujours ») : `couper(Renommage, 1)` et `couper(Relecture, 2)` (la première relecture est celle du voisin) ; `ecrire` (idee-2) rend `Err(Erreur::Disque { .. })` ; `lecture_seule() == Some(&LectureSeule::RelectureManquee { chemin: <dossier du fonds>/complet.automerge })` ; idee-2 absente de la mémoire, heads d'avant ; `ecrire` et `compacter` rendent `Err(Erreur::LectureSeule(LectureSeule::RelectureManquee { .. }))` ; `prochain_message` rend `None` ; `recevoir` rend `Err(Erreur::LectureSeule(_))` ; `racine.captations().verser(&octets(3, 100))` rend `Ok(_)`. Lâché puis rouvert sans panne : `lecture_seule() == None`, idee-2 absente.
    - `un_rendre_durable_refuse_apres_le_renommage_laisse_l_operation_faite` (plan maître, tranche 1, quatrième puce : « l'échec de `rendre_durable` après le renommage, qu'aucun test d'H07 ne provoque » ; décision 12) — `// § 31 · 10, test 8 : une opération dite faite ne manque pas`. `couper(RendreDurable, 1)` sur une écriture : `Ok(Some(NonDurable { chemin, .. }))`, rouvert l'idée y est. Puis, dans une racine neuve, `couper(RendreDurable, 1)` sur `faire_naitre_le_fonds` : `Ok(document)`, `avertissements()` est `[NonDurable { chemin: <dossier du fonds>/complet.automerge, .. }]` ; lâché, `fonds()` rend `Ok(Some(_))`.
  - `tests/fonds.rs`, `une_naissance_refusee_avant_le_renommage_laisse_pupitre_neuf` (C2) : dans une racine neuve, `couper(Renommage, 1)` (le premier renommage de la naissance est celui du fichier complet) ; `faire_naitre_le_fonds(&NAISSANCE, …)` rend `Err(Erreur::Disque { .. })` ; `documents/fonds/` ne porte que `ajouts/`, vide ; `fonds()` rend `Ok(None)`, la garde rendue ; puis, sans panne, une seconde naissance réussit.
  - `tests/compaction.rs`, `une_compaction_qui_echoue_apres_une_ecriture_ne_la_defait_pas` (AM-4 (5), décision 8 d'H07) : un fonds né d'une idée courte (« Un refrain qui monte »), puis une écriture de `texte_long()`, plus lourde que le fichier complet, sous `couper(Save, 1)` : `Ok(Some(Avertissement::Compaction(Erreur::CompactionInfidele)))` ; un fichier dans `ajouts/` ; la panne levée, l'écriture suivante rend `Ok(None)` et `ajouts/` est vide ; rouvert, les deux idées y sont.
  - `tests/synchronisation.rs` :
    - `ce_qui_est_recu_se_relit_du_disque` complété (plan maître, tranche 1, huitième puce : « Le fichier reçu comparé à `save_after` des heads d'avant (mineur d'H07) ») : le contenu du fichier reçu vaut exactement `doc.automerge().save_after(&heads_d_avant)`.
    - `une_reception_refusee_en_chemin_ecrit_ce_qui_a_change` (plan maître, tranche 1, quatrième puce : « une réception refusée en chemin, qui écrit ce qui a changé les heads » ; décision 8 d'H07) : `deux_copies`, `b` note une idée ; `couper(Reception, 1)` ; réunir à la main (au plus 1 000 messages, comme `commun::reunir`) ; la réception de `a` qui change ses heads rend `Err(Erreur::Synchro(_))` ; un fichier d'ajout de plus dans `a`, égal à `save_after` des heads d'avant cette réception ; l'idée est dans `a`, et dans `a` rouvert. Puis, sur deux copies neuves, de même sous `couper(Reception, 1)` et `couper(RendreDurable, 1)` (le premier `rendre_durable` est celui du fichier d'ajout de cette réception) : `Err(Erreur::Synchro(_))`, et `a.avertissements()` est `[Avertissement::NonDurable { chemin, .. }]`, `chemin` sous `ajouts/` et présent.
    - `une_reception_refusee_relit_le_document_et_l_echange_reprend` (réécrit `un_document_a_relire_ne_laisse_rien_partir` ; point de revue 2) : `couper(Renommage, 1)` ; la réception qui change les heads de `a` rend `Err(Erreur::Disque { .. })` ; les heads de `a` sont celles d'avant ; puis la boucle continue, sur les mêmes `Echange`, au plus 1 000 messages, jusqu'à ce que ni l'un ni l'autre n'ait rien à dire : après le refus, une réception de `a` change de nouveau ses heads et écrit un fichier d'ajout (`b` a renvoyé ce qu'elle avait envoyé, ce que seul `Echange::reprendre` obtient) ; les heads de `a` et `b` sont égales, et l'idée est dans `a` rouvert.
    - `un_message_illisible_rend_synchro` (plan maître, tranche 1, huitième puce : « chaque variante d'`Erreur` jamais provoquée (`Genese`, `ChampAbsent`, `Synchro`) a son test, ou disparaît ») : `a.recevoir(&mut echange, b"pas un message")` rend `Err(Erreur::Synchro(synchro::Erreur::MessageIllisible(_)))`, et aucun fichier ne s'écrit.
  - Les doctests sur `Document::ecrire` (plan maître, tranche 1, huitième puce : « un doctest `compile_fail` fige qu'aucun message ne se prépare pendant une écriture ») — `// § 31 · 10, test 8 : une synchronisation lancée pendant l'écriture ne voit pas l'opération en cours`. Le premier ne compile pas, le second, identique hors l'endroit de l'appel, compile ; si la CI rend un autre code que E0502, le doctest prend celui-là. Deux caractérisations : ils passent déjà sur le code d'H07 :

```rust
/// ```compile_fail,E0502
/// fn pendant(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange) {
///     let _ = document.ecrire("Idée notée", 0, |_tx| {
///         let _ = document.prochain_message(echange);
///         Ok(())
///     });
/// }
/// ```
///
/// ```
/// fn apres(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange) {
///     let _ = document.ecrire("Idée notée", 0, |_tx| Ok(()));
///     let _ = document.prochain_message(echange);
/// }
/// ```
```

- [ ] **Étape 2 : voir échouer.** Committer (`test: faite, faite avec un avertissement, ou refusée et relue`), pousser. Attendu : « Lint » rouge, `cannot find type Avertissement` et `LectureSeule`.
- [ ] **Étape 3 : écrire `ecrire`, `recevoir`, `lire_le_dossier`, la relecture du disque, la naissance, `Echange::reprendre`**, et les nouvelles variantes.
- [ ] **Étape 4 : voir passer.** Committer (`feat: l'écriture faite avec un avertissement, refusée puis relue du disque (AM-4)`), pousser. Attendu, dans `verifier` et `windows` : `ajouts.rs` `ok. 11 passed`, `compaction.rs` `ok. 5`, `synchronisation.rs` `ok. 6`, `fonds.rs` `ok. 7`, `echange.rs` `ok. 3`, et `Doc-tests stockage` `ok. 2 passed`.

### Tâche 7 : l'ouverture en lecture seule, et la compaction d'ouverture avec un avertissement

**Fichiers :** modifier `crates/stockage/src/lib.rs` (`Erreur`, `LectureSeule`), `src/documents.rs` (:172-212,
:243-307), `src/racine_des_donnees.rs` (doc de `fonds`, :62-70, qui dit encore `AjoutAltere`,
`DependancesManquantes`, `Automerge` et « Une compaction qui échoue fait échouer l'ouverture » : elle dit la lecture
seule et l'avertissement), `crates/noyau/src/donnees/compaction.rs` (doc de `Lu::lisible`) ; tests :
`tests/ajouts.rs` (et sa doc de module, :2-3, dont « Un fichier d'ajout altéré ou manquant fait refuser
l'ouverture » devient « … ouvre le document en lecture seule, en se nommant »), `tests/compaction.rs`,
`tests/fonds.rs`.

**Interfaces — produit :**

```rust
// LectureSeule : + AjoutAltere { chemin: PathBuf }, + AjoutIllisible { chemin: PathBuf },
//                + CompletIllisible { chemin: PathBuf }, + DependancesManquantes { chemin: PathBuf }
// AjoutIllisible : un fichier d'ajout dont le contenu donne l'empreinte de son nom, mais que
//                  `load_incremental` refuse (par exemple `DuplicateSeqNumber`) ; il n'est pas chargé.
// Erreur : − AjoutAltere, − DependancesManquantes ; CompactionInfidele ne sort plus que de `compacter`
//          appelé seul, et des avertissements ; sa doc perd « ou, écrit, le disque n'en rend pas les
//          octets » : un voisin relu autre rend `Disque` (`InvalidData`), par `Echec::Refusee` (tâche 3).
//          La doc d'`Automerge` perd « la lecture de son fichier complet ou d'un fichier d'ajout »,
//          cas devenus `CompletIllisible` et `AjoutIllisible` : « Automerge a refusé : le geste, une
//          écriture du document, ou la lecture de sa racine ».
```

L'ouverture lit le fichier complet par `Automerge::load` ; refusé, il se relit par
`Automerge::load_with_options(&octets, LoadOptions::new().on_partial_load(OnPartialLoad::Ignore))` (sinon
`Automerge::new()`), et la raison est `CompletIllisible`. Les fichiers d'ajout se lisent triés par nom ; un
fichier qui trahit son empreinte n'est pas chargé, et donne `AjoutAltere` ; un fichier qui la donne, mais que
`load_incremental` refuse, donne `AjoutIllisible`, au lieu de l'`Erreur::Automerge` d'H07, qui faisait refuser
l'ouverture : c'est un fichier illisible, qui ouvre en lecture seule en se nommant (décision 12) ; le premier
fichier, dans l'ordre des noms, qui donne l'une ou l'autre l'emporte ; tout lu,
`get_missing_deps(&[])` non vide donne `DependancesManquantes { chemin }`, `chemin` le premier fichier d'ajout,
dans l'ordre des noms, dont une modification attend : seul dans un document neuf (`load_incremental`), il
attend des modifications (`get_missing_deps(&[])`) dont l'une manque au document lu (`get_change_by_hash` rend
`None`). Le fichier qui manque ne se nomme pas, celui qui l'attend si : le bandeau de la tranche 3 a un fichier
à nommer dans chaque cas (décision 12). La première raison trouvée l'emporte, dans cet ordre : le fichier complet,
puis les fichiers d'ajout, puis les dépendances. Un refus du
disque à la lecture rend `Err(Erreur::Disque)`. La compaction d'ouverture suit `a_l_ouverture(acces, …)` ; son échec pousse `Avertissement::Compaction(e)` et lève `compaction_due`.
`compacter` retire par `fichier::retirer` (`point(Etape::Retrait)`), et calcule `Lu::lisible` d'un fichier
d'ajout ainsi : son contenu donne l'empreinte de son nom (`est_conforme`), et `load_incremental` le charge sans
erreur, dans le `fork()` du fichier complet relu où `est_absorbe` le charge déjà ; la doc de `lisible` le dit
(« un fichier d'ajout dont le contenu donne l'empreinte de son nom et que `load_incremental` charge »), et un
fichier `AjoutIllisible` n'est donc pas lisible. Chaque fichier se lit par
`fichier::lire`. Ces lectures vivent dans `lire_le_dossier` (tâche 6), que la relecture du disque appelle
aussi : une raison de lecture seule qu'elle trouve devient celle du document, sur ce qui se lit.

- [ ] **Étape 1 : écrire les tests.** Pour chaque document en lecture seule : `ecrire` et `compacter` rendent `Err(Erreur::LectureSeule(<la raison>))`, `prochain_message` rend `None`, `recevoir` rend `Err(Erreur::LectureSeule(_))`, et `empreintes(<dossier du fonds>)` est la même avant l'ouverture et après ces quatre appels.
  - `tests/ajouts.rs` :
    - `un_ajout_manquant_ouvre_en_lecture_seule` (réécrit `un_ajout_manquant_rend_dependances_manquantes`) : le fichier d'ajout de la première écriture retiré ; `lecture_seule() == Some(&LectureSeule::DependancesManquantes { chemin })`, `chemin` le fichier d'ajout de la seconde, qui l'attend ; la mémoire porte la naissance, sans l'étiquette ni l'idée écrites.
    - `un_ajout_altere_ouvre_en_lecture_seule_et_se_nomme` (réécrit `un_ajout_altere_se_nomme_et_rien_ne_bouge`) : deux écritures, un octet changé dans le fichier de la seconde ; `lecture_seule() == Some(&LectureSeule::AjoutAltere { chemin })`, son chemin ; la première idée se lit, la seconde non.
    - `un_ajout_qu_automerge_refuse_ouvre_en_lecture_seule` : un fonds né, lâché ; le test bâtit, depuis le fichier complet (`Automerge::load`, heads `h`), deux `fork()` auxquels il donne le même acteur (`set_actor(ActorId::from(b"acteur-en-double".as_slice()))`), l'un notant idee-2, l'autre idee-3 : deux modifications du même acteur au même rang ; il écrit `save_after(&h)` de chacun dans `ajouts/<empreinte>.automerge` ; rouvert : `lecture_seule() == Some(&LectureSeule::AjoutIllisible { chemin })`, `chemin` le second des deux dans l'ordre des noms ; l'idée du premier se lit, celle du second non.
    - `un_complet_illisible_ouvre_en_lecture_seule_et_se_nomme` (AM-4 (6) ; décision 12 du plan maître : « un fichier complet que `load` refuse ») : l'octet du milieu du fichier complet changé ; `lecture_seule() == Some(&LectureSeule::CompletIllisible { chemin: <dossier du fonds>/complet.automerge })` ; la mémoire montre ce qui se lit : ses heads sont celles que le test calcule à part, par `Automerge::load_with_options` des octets altérés (`OnPartialLoad::Ignore`), ou un document neuf s'il les refuse, puis `load_incremental` de chaque fichier d'ajout.
  - `tests/compaction.rs` :
    - `la_compaction_garde_un_ajout_altere` réécrit — `// § 31 · 10, test 9 : un fichier d'ajout altéré, puis compacter`. Sa fin : `fonds()` rend un document, `lecture_seule() == Some(&LectureSeule::AjoutAltere { chemin })`, le fichier altéré toujours là, octet pour octet.
    - `une_compaction_infidele_n_ecrit_rien` (LE-10 ; § 31 · 2.2, point 2) : deux écritures, `complet_avant = lire_le_complet(..)`, `couper(Save, 1)`, `compacter()` rend `Err(Erreur::CompactionInfidele)` ; `entrees(<dossier du fonds>) == ["ajouts", "complet.automerge"]` ; `lire_le_complet(..) == complet_avant` ; deux fichiers dans `ajouts/`.
    - `une_compaction_dont_le_complet_se_relit_autre_ne_retire_rien` (LE-10 ; § 31 · 2.2, point 3) : de même, sous `pannes::alterer_la_relecture(1)` (la première relecture de `compacter` est celle du voisin du fichier complet) ; `compacter()` rend `Err(Erreur::Disque { erreur, .. })`, `erreur.kind() == io::ErrorKind::InvalidData` ; `entrees(<dossier du fonds>) == ["ajouts", "complet.automerge"]` ; `lire_le_complet(..) == complet_avant` ; les deux fichiers d'ajout toujours là.
    - `une_compaction_coupee_a_chaque_etape_ne_perd_rien` — `// § 31 · 10, test 9 : compacter en s'interrompant à chaque étape, puis rouvrir, et revenir avant chaque opération`. Pour `Save`, `VoisinEcrit`, `SyncAll`, `Relecture`, `Renommage`, `RendreDurable` et `Retrait` (rang 1), chacun dans son dossier, sur trois écritures : `compacter()` rend `Err(_)` ; pour les cinq premières, le fichier complet est celui d'avant et aucun voisin ne reste ; rouvert, les heads et `hydrate(None)` sont ceux d'avant ; pour chaque modification de `get_changes(&[])`, `fork_at(&[hash])` du document rouvert et de la mémoire d'avant ont le même `hydrate(None)`. Après `Retrait`, la compaction suivante, sans panne, laisse `ajouts/` vide.
    - `une_compaction_d_ouverture_infidele_ouvre_avec_un_avertissement` (décision 12 du plan maître : « Une compaction d'ouverture infidèle, ou refusée par le disque, n'empêche pas d'ouvrir ») : une écriture, lâché ; `couper(Save, 1)` ; `fonds()` rend un document, `lecture_seule() == None`, `avertissements()` est `[Avertissement::Compaction(Erreur::CompactionInfidele)]` ; le fichier complet et l'ajout sont en place ; la panne levée, une écriture rend `Ok(None)` et laisse `ajouts/` vide (compaction due).
    - `une_compaction_d_ouverture_refusee_par_le_disque_ouvre_avec_un_avertissement` (AM-4 (6) ; décision 12 du plan maître) : de même, sous `couper(Renommage, 1)` ; `avertissements()` est `[Avertissement::Compaction(Erreur::Disque { .. })]`.
  - `tests/fonds.rs`, `une_copie_sans_lignee_rend_champ_absent` (plan maître, tranche 1, huitième puce, la variante `ChampAbsent`), une caractérisation, que l'essai passe déjà et que la description relève comme telle : le test écrit `documents/fonds/complet.automerge` d'un document qu'il bâtit (la genèse, `IDENTIFIANT` et `PROPRIETAIRE`, sans `LIGNEE`) et crée `ajouts/` ; `fonds()` rend un document, et `copie()` rend `Err(Erreur::ChampAbsent { champ: "lignee" })`.
- [ ] **Étape 2 : voir échouer.** Committer (`test: l'ouverture en lecture seule, et son avertissement`), pousser. Attendu : « Lint » rouge, `no variant … AjoutAltere` sur `LectureSeule`.
- [ ] **Étape 3 : écrire l'ouverture et `compacter`.**
- [ ] **Étape 4 : voir passer.** Committer (`feat: un fichier illisible ouvre en lecture seule, une compaction d'ouverture en échec avertit`), pousser. Attendu, dans `verifier` et `windows` : `ajouts.rs` `ok. 13 passed`, `compaction.rs` `ok. 10`, `fonds.rs` `ok. 8`.

### Tâche 8 : les documents de la PR (b), et la mutation remesurée

**Fichiers :** modifier `docs/31-persistance-et-synchronisation.md` (§ 2.1, :187-282 ; § 2.2, :284-305),
`docs/80-glossaire.md` (après « Version de schéma », :117), `docs/40-architecture.md` (§ 3, :180-183),
`CLAUDE.md` (« Les crates », `stockage` ; « Commandes »), `CONVENTIONS.md` (lignes `verifier` et `windows`),
`docs/registre.md` (LE-10), `.github/workflows/ci.yml` (`mutants`, :169-176), `.github/workflows/mutants-complet.yml`
(:19).

- [ ] **Étape 1 : le § 31 · 2.1.** Une opération « n'est faite qu'une fois son fichier d'ajout renommé » ; trois issues : faite ; faite, avec un avertissement (le renommage non rendu durable, ou la compaction qui échoue, pour une opération, une réception, le premier fichier complet ou une migration) ; refusée, son voisin retiré, le document relu du disque ; une réception refusée relance en outre l'échange, pour que l'autre copie renvoie ce que la relecture a ôté. Chaque fichier : écrit, synchronisé, fermé, relu, puis renommé. Un voisin qui ne se retire pas se vide ; seule cette double panne peut faire paraître une opération refusée à la relance. Une relecture du disque qui échoue met le document en lecture seule, sur sa mémoire d'avant l'opération refusée. Les deux lectures seules se nomment : illisible (un fichier qui ne se lit pas, une relecture manquée, une version de schéma absente, nulle, ou qui n'est pas un entier non signé, que le § 2.4 dira en (d)), et plus récente (§ 2.4). C'est la réponse à la question C2, tranchée le 2026-11-09 : une précision de l'ADR-023, qui ne change pas ; le texte la dit sans la reposer. Le diagramme gagne `R -.->|"rendre durable refusé, compaction en échec"| A["Opération faite, avec un avertissement"]`.
- [ ] **Étape 2 : le § 31 · 2.2.** Point 3 : « Elle l'écrit dans un voisin, synchronisé, fermé, puis relu, qui doit rendre les octets écrits, et seulement alors renommé en fichier complet (§ 2.1) » (Éc-3, LE-10). Après le point 4 : une compaction qui échoue, à l'ouverture ou après une écriture, n'empêche ni d'ouvrir ni d'écrire ; elle avertit, laisse les fichiers en place et se retente à l'écriture suivante ; un retrait interrompu laisse des fichiers absorbés, que la compaction suivante retire.
- [ ] **Étape 3 : le glossaire, le § 40 · 3, `CLAUDE.md`.** `git grep -n -i -w "lecture seule" -- docs README.md CONVENTIONS.md`, la sortie citée dans la PR. Entrée « **Lecture seule (d'un document)** », sur le modèle des entrées « Racine » et « Conteneur », qui disent leurs autres sens : « L'état d'un document qui montre ce qui se lit, sans y écrire ni se compacter. Devant un fichier illisible, une relecture du disque manquée, ou une version de schéma absente, nulle, ou qui n'est pas un entier non signé, il n'envoie ni ne reçoit ; devant une version de schéma plus récente, il reçoit, et ne transmet aucune modification ([§ 31 · 2.1], [§ 31 · 2.4]). Au sens d'une vue, ce qui se montre sans se modifier, comme la mesure d'avant et celle d'après sous la loupe ([MIC-01]) ; au sens de l'application, un protocole qui sert sans écrire, celui des captations ([§ 40 · 7.5]) ; au sens du nœud, un dossier monté sans écriture ([M-0006]). On dit duquel il s'agit dès que le contexte ne le dit pas ». Les emplois qui ne disent pas un document (SFD 22, l. 38 ; § 40 · 7.5, l. 501 ; M-0006, l. 59, 185 et 199) restent : l'entrée nomme leur sens, et la PR le dit. § 40 · 3 et `CLAUDE.md`, après la crate `stockage` : « sa fonction de Cargo `pannes`, que seuls ses tests activent, coupe une écriture à l'étape voulue ». `CLAUDE.md`, « Commandes », et `CONVENTIONS.md`, colonne « Script » des lignes `verifier` et `windows` : `cargo clippy -p stockage --lib -- -D warnings`, qui compile le stockage sans `pannes`, tel que l'établi le recevra. Puis la recherche de LE-3 sur ce que la PR périme : `git grep -n -i -E "à relire|ARelire|a_relire|Erreur::Genese|Erreur::AjoutAltere|Erreur::DependancesManquantes" -- . ':!docs/registre.md' ':!.claude/plans'` (le registre et les plans disent un état passé) ; aucune ligne de code ni de test ne doit rester ; les deux emplois d'un autre sens, l'artefact `cargo-lock` « à relire puis à committer » (`ci.yml`, `CLAUDE.md`), restent, et la PR le dit. « À » et « relire » tombant parfois sur deux lignes (`crates/stockage/tests/ajouts.rs:3-4`), une deuxième recherche prend le mot seul, avec la ligne d'avant : `git grep -n -i -w -B1 "relire" -- . ':!docs/registre.md' ':!.claude/plans'` ; chaque emploi qui commence une ligne après un « à » se lit, et aucun ne doit dire l'état « à relire » ; les autres (`Captations::relire`, le moment « Relire », « relire le disque ») gardent leur sens. Une troisième prend les refus d'ouverture qu'H07 décrivait : `git grep -n -i -E "refuser l'ouverture|fait échouer l'ouverture|font échouer l'ouverture" -- . ':!docs/registre.md' ':!.claude/plans'` ; aucune ligne ne doit rester (avant la PR : `crates/stockage/src/racine_des_donnees.rs:70`, `crates/stockage/src/documents.rs:245`, `crates/stockage/tests/ajouts.rs:3`, que les tâches 6 et 7 récrivent). Une quatrième prend le mot que la tâche 4 retire du noyau : `git grep -n -w -E "conforme|conformes" -- crates/noyau` ; aucune ligne attendue (avant la PR : `crates/noyau/src/donnees/compaction.rs:25`, `:31`, `:32`, `:39`, `crates/noyau/tests/donnees.rs:7`, `:10`, `:15`, `:16`). Les quatre sorties citées dans la PR.
- [ ] **Étape 4 : le registre.** LE-10, colonne « État » : « clos le AAAA-MM-JJ, par `une_compaction_infidele_n_ecrit_rien` (`crates/stockage/tests/compaction.rs`), PR #NN ». `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add` ; `verif-diagrammes` se lit dans `documentation`.
- [ ] **Étape 5 : remesurer la mutation, à froid**, selon le plan d'H07, tâche 4, pas 5 : `gh workflow run mutants-complet.yml --ref t1b-pannes-et-lecture-seule`, puis `gh run list --workflow mutants-complet --branch t1b-pannes-et-lecture-seule --limit 1` pour son numéro ; dans le journal de l'étape rust-cache, lire « No cache found. ». Si un cache a été restauré (« Restored from cache key … »), retirer ceux que ce dispatch peut lire : `gh cache list --key v0-rust-mutants --json key,ref` (les deux jobs s'appellent `mutants`, d'où la clé), puis `gh cache delete <clé> --ref refs/heads/main` et `--ref refs/heads/t1b-pannes-et-lecture-seule`, et relancer. Relever la durée du job (`gh run view <n> --json jobs`, `startedAt` et `completedAt`) et « N mutants tested ». Si la mesure n'a pu se prendre que sur un cache, les commentaires le disent, et la PR le donne comme un écart au § 70 · 1. Ce passage complet vert, sans `MISSED` ni `TIMEOUT`, son lien dans la description ; les deux `timeout-minutes` à deux fois la durée, arrondie à la minute au-dessus, et leurs commentaires (« à froid », le compte, la durée, la date, la PR).
- [ ] **Étape 6 : committer** par chemins (`docs: la lecture seule et l'écriture faite avec un avertissement`, puis `ci: les délais de la mutation, remesurés avec l'accès d'un document`), pousser. **Fin de la PR (b) :** `verifier`, `windows`, `mutants` (sans `MISSED`), `contenu` et `documentation` verts ; la description dit les tests réécrits (`une_ecriture_qui_echoue_laisse_le_document_a_relire`, `un_document_a_relire_ne_laisse_rien_partir`, `un_ajout_manquant_rend_dependances_manquantes`, `un_ajout_altere_se_nomme_et_rien_ne_bouge`, `la_compaction_garde_un_ajout_altere`, `ce_qui_est_recu_se_relit_du_disque`), le test adapté à la seule signature (`un_geste_refuse_n_ecrit_rien_et_le_document_reste_ouvert`), le test noyau renommé (`a_retirer_garde_un_fichier_non_conforme`, devenu `a_retirer_garde_un_fichier_illisible`), les caractérisations (tâches 6 et 7, les deux doctests compris) et le test noyau retiré, les recherches de LE-2 et de LE-3, `Echange::reprendre` (écart 3) et la réponse à la question C2, tranchée le 2026-11-09 (une précision de l'ADR-023, que le § 31 · 2.1 applique) ; `gh pr ready <n>`, puis fusion par squash.

---

## PR (c) · le démarrage et le suffixe

**Branche :** `t1c-demarrage-et-suffixe`. **Titre :** `feat: le démarrage met en place les voisins et range les
noms provisoires (AM-4)` (80) ; le verbe « reprendre » est écarté (C10), et la description dit le suffixe. **Documents :** § 31 · 2.1, 2.2, 6.1 et 10 (test 4) ; glossaire (Voisin) ;
`CONVENTIONS.md` (`mutants`) ; `.cargo/mutants.toml` ; délais. **Fin :** voir la tâche 12.

### Tâche 9 : les règles du démarrage, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/demarrage.rs`, `crates/noyau/tests/demarrage.rs` ; modifier
`donnees/mod.rs`, `.cargo/mutants.toml`, `CONVENTIONS.md` (`mutants`).

**Interfaces — produit :**

```rust
// noyau::donnees::demarrage (§ 31 · 2.1, § 31 · 6.1)
/// `<nom>.<processus>.partiel`, ou `-2`, `-3`… : le premier que `pris` (les noms du dossier) n'a pas.
/// Une recherche bornée : les rangs 1 à `pris.len() + 1`, dont l'un est forcément libre, le rang 1 sans
/// suffixe, le rang n ≥ 2 en `-n` ; le premier libre (`find`), puis `expect`. Jamais de boucle sans borne
/// (`loop`, `while`) : ses
/// mutants (`+=` en `*=`, le `!` effacé) boucleraient sans fin, et cargo-mutants, sur un TIMEOUT, sort
/// en 3, ce qui fait échouer `mutants` ; bornée, chaque mutant panique ou rend un autre nom.
pub fn voisin_libre(nom: &str, processus: u32, pris: &[String]) -> String;
/// Le nom du fichier dont `nom` est le voisin (`<nom>.<chiffres>[-<chiffres>].partiel`), sinon `None`.
pub fn fichier_du_voisin(nom: &str) -> Option<&str>;
/// Un `.partiel` de `documents/`, tel que l'adaptateur l'a lu (seuls les noms en `.partiel`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Partiel<'a> {
    /// Dans `ajouts/` : son nom, et l'empreinte de son contenu.
    DansAjouts { nom: &'a str, empreinte: &'a str },
    /// Dans le dossier d'un document : son nom, et s'il est lisible (non vide, `load` sans erreur).
    DansLeDossier { nom: &'a str, lisible: bool },
    /// Ailleurs dans `documents/`.
    Ailleurs,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort { MettreEnPlace, Lire, Retirer }
/// DansAjouts : MettreEnPlace si `fichier_du_voisin(nom) == Some("<empreinte>.automerge")` ;
/// DansLeDossier : Lire si `fichier_du_voisin(nom) == Some("complet.automerge")` et lisible ;
/// sinon Retirer.
pub fn sort_d_un_partiel(partiel: Partiel<'_>) -> Sort;
/// Pupitre est neuf : ni fichier complet, ni voisin du complet lisible (§ 31 · 9.1).
pub fn pupitre_neuf(complet: bool, voisins_du_complet_lisibles: usize) -> bool;
/// Un nom provisoire de `captations/` : son nom, l'empreinte de son contenu entier, et si une
/// captation de cette empreinte est déjà rangée (`deja_rangee`, et non `rangee`, que le noyau
/// emploie pour la rangée de la table : `table::Rangee`, `Position { rangee }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provisoire { pub nom: String, pub empreinte: String, pub deja_rangee: bool }
/// (nom provisoire, empreinte) de chaque nom provisoire dont l'empreinte est nommée et non rangée ;
/// de deux du même contenu, le premier dans l'ordre reçu, que l'adaptateur donne trié par nom : le
/// second écrirait sur une captation rangée (I14 ; § 31 · 6.1).
pub fn a_ranger(provisoires: &[Provisoire], nommees: &BTreeSet<String>) -> Vec<(String, String)>;
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/demarrage.rs`), avec `fn e(n: u8) -> String { format!("{n:064x}") }` :

```rust
#[test]
fn le_voisin_prend_le_premier_nom_libre() {
    let p = |s: &[&str]| s.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    let c = "complet.automerge";
    assert_eq!(voisin_libre(c, 4242, &[]), "complet.automerge.4242.partiel");
    assert_eq!(voisin_libre(c, 4242, &p(&["complet.automerge.4242.partiel"])), "complet.automerge.4242-2.partiel");
    assert_eq!(voisin_libre(c, 4242, &p(&["complet.automerge.4242.partiel", "complet.automerge.4242-2.partiel"])), "complet.automerge.4242-3.partiel");
    assert_eq!(voisin_libre(c, 4242, &p(&["complet.automerge.4242.partiel", "complet.automerge.4242-3.partiel"])), "complet.automerge.4242-2.partiel");
    assert_eq!(voisin_libre(c, 4242, &p(&["complet.automerge.7.partiel", "complet.automerge"])), "complet.automerge.4242.partiel");
}
#[test]
fn le_nom_d_un_voisin_se_reconnait_avec_son_suffixe() {
    assert_eq!(fichier_du_voisin("complet.automerge.4242.partiel"), Some("complet.automerge"));
    assert_eq!(fichier_du_voisin("complet.automerge.4242-2.partiel"), Some("complet.automerge"));
    assert_eq!(fichier_du_voisin("complet.automerge.4242-12.partiel"), Some("complet.automerge"));
    for nom in ["complet.automerge", "complet.automerge.partiel", "complet.automerge.42a.partiel",
        "complet.automerge.4242-.partiel", "complet.automerge.4242-x.partiel",
        "complet.automerge.-2.partiel", ".4242.partiel"] {
        assert_eq!(fichier_du_voisin(nom), None, "{nom}");
    }
}
#[test]
fn chaque_partiel_a_son_sort() {
    let a = format!("{}.automerge.4242.partiel", e(1));
    let a2 = format!("{}.automerge.4242-2.partiel", e(1));
    assert_eq!(sort_d_un_partiel(Partiel::DansAjouts { nom: &a, empreinte: &e(1) }), Sort::MettreEnPlace);
    assert_eq!(sort_d_un_partiel(Partiel::DansAjouts { nom: &a2, empreinte: &e(1) }), Sort::MettreEnPlace);
    assert_eq!(sort_d_un_partiel(Partiel::DansAjouts { nom: &a, empreinte: &e(2) }), Sort::Retirer);
    assert_eq!(sort_d_un_partiel(Partiel::DansLeDossier { nom: "complet.automerge.4242.partiel", lisible: true }), Sort::Lire);
    assert_eq!(sort_d_un_partiel(Partiel::DansLeDossier { nom: "complet.automerge.4242-3.partiel", lisible: true }), Sort::Lire);
    assert_eq!(sort_d_un_partiel(Partiel::DansLeDossier { nom: "complet.automerge.4242.partiel", lisible: false }), Sort::Retirer);
    assert_eq!(sort_d_un_partiel(Partiel::DansLeDossier { nom: "autre.automerge.4242.partiel", lisible: true }), Sort::Retirer);
    assert_eq!(sort_d_un_partiel(Partiel::Ailleurs), Sort::Retirer);
}
#[test]
fn pupitre_est_neuf_sans_complet_ni_voisin_lisible() {
    assert!(pupitre_neuf(false, 0));
    assert!(!pupitre_neuf(true, 0) && !pupitre_neuf(false, 1) && !pupitre_neuf(true, 2));
}
// § 31 · 10, test 4 : la captation nommée par un document prend son nom au démarrage
#[test]
fn les_noms_provisoires_a_ranger() {
    let pr = |nom: String, n: u8, deja_rangee| Provisoire { nom, empreinte: e(n), deja_rangee };
    let nommees = BTreeSet::from([e(1), e(2), e(4)]);
    let provisoires = [
        pr(format!("{}.4242.partiel", e(1)), 1, false),   // nommée, absente : rangée
        pr(format!("{}.4242.partiel", e(2)), 2, true),    // déjà rangée : reste
        pr(format!("{}.4242.partiel", e(3)), 3, false),   // non nommée : reste
        pr(format!("{}.4242.partiel", e(4)), 5, false),   // coupée, contenu non nommé : reste
        pr(format!("{}.4242.partiel", e(5)), 4, false),   // le contenu nomme e(4) : rangée sous e(4)
        pr(format!("{}.4242-2.partiel", e(1)), 1, false), // même contenu qu'une rangée plus haut : reste
    ];
    assert_eq!(a_ranger(&provisoires, &nommees),
        [(format!("{}.4242.partiel", e(1)), e(1)), (format!("{}.4242.partiel", e(5)), e(4))]);
}
```

- [ ] **Étape 2 : voir échouer.** Committer (`test: les règles du démarrage`), pousser. Attendu : « Lint » rouge, `unresolved import noyau::donnees::demarrage`.
- [ ] **Étape 3 : écrire `demarrage.rs`.** `.cargo/mutants.toml` : le module dans `examine_globs`, `"--test", "demarrage"` dans `additional_cargo_test_args` ; en tête, « Onze modules » et « les règles des données (la compaction, l'accès d'un document, le démarrage et la comparaison des copies) » ; l. 17, « ces onze modules ». `CONVENTIONS.md`, ligne `mutants` : « …les règles des données : la compaction, l'accès d'un document, le démarrage et la comparaison des copies ».
- [ ] **Étape 4 : voir passer.** Committer (`feat: les règles du démarrage, au noyau`), pousser. Attendu : `tests/demarrage.rs` `ok. 5 passed` dans `verifier` ; `mutants` vert, sans `MISSED`.

### Tâche 10 : le suffixe du voisin

**Fichiers :** modifier `crates/stockage/src/fichier.rs` (`voisin`, `ecrire_a_cote`), `src/captations.rs`
(:44) ; tests : `tests/fichier.rs`, `tests/captations.rs` (et, à `une_captation_alteree_ou_absente_se_nomme`,
:149, `voisin(…)` devient `voisin(…).unwrap()` : rien ne porte encore ce nom, qui reste donc sans suffixe).
`un_voisin_qui_ne_se_retire_pas_se_vide` (tâche 3) nomme son voisin en clair et ne change pas.

**Interfaces — consomme :** `demarrage::voisin_libre`. **Produit :**

```rust
/// Le premier nom libre de `<nom>.<processus>.partiel`, `-2`, `-3`…, dans le dossier de `chemin`,
/// lu sur le disque (§ 31 · 2.1).
pub fn voisin(chemin: &Path) -> io::Result<PathBuf>;
```

`ecrire_a_cote` et `verser` prennent leur voisin par elle ; un refus de lire le dossier rend
`Echec::Refusee`, ou `Erreur::Disque`.

- [ ] **Étape 1 : écrire les tests.**
  - `tests/fichier.rs` : `le_voisin_porte_le_numero_du_processus` réécrit dans un dossier jetable (`voisin(&d.join("complet.automerge")).unwrap()` se termine par `complet.automerge.<pid>.partiel`) ; `un_nom_de_voisin_pris_recoit_un_suffixe` : ce nom créé, `voisin` rend `-2` ; créé, `-3` ; `-2` retiré, `-2` ; puis, `<nom>.<pid>.partiel` présent avec « laissé », `ecrire_a_cote(chemin, b"apres")` rend `Ok(())`, le fichier porte « apres » et le voisin laissé porte toujours « laissé ».
  - `tests/captations.rs`, `verser_laisse_intact_un_nom_provisoire_laisse` (§ 31 · 6.1 : un nom provisoire laissé par un arrêt « n'est jamais retiré d'office » ; sans suffixe, `File::create` le tronquerait, puis l'échec le retirerait) : `captations/<e>.<pid>.partiel` déposé avec `octets(6, 200)`, `e` l'empreinte d'`octets(7, 300)` ; `couper(Renommage, 1)` ; `verser(&octets(7, 300))` rend `Err(_)` ; le nom laissé porte toujours `octets(6, 200)` ; sans panne, `verser` réussit et le nom laissé reste.
- [ ] **Étape 2 : voir échouer.** Committer (`test: le suffixe du voisin`), pousser. Attendu : « Lint » rouge, E0599, `no method named unwrap found for struct PathBuf` : `voisin` rend encore un `PathBuf`.
- [ ] **Étape 3 : écrire `voisin`, et ses deux appels.**
- [ ] **Étape 4 : voir passer.** Committer (`feat: un nom de voisin pris reçoit un suffixe`), pousser. Attendu : `fichier.rs` `ok. 8 passed`, `captations.rs` `ok. 6`, dans `verifier` et `windows`.

### Tâche 11 : le démarrage dans `documents/`

**Fichiers :** modifier `crates/noyau/src/donnees/compaction.rs` (doc de `Lu`, :19-28),
`crates/stockage/src/lib.rs` (`Avertissement`), `crates/stockage/src/documents.rs` (ouverture, `lire_le_dossier`,
`compacter`, `faire_naitre_le_fonds`) ; tests : créer `crates/stockage/tests/demarrage.rs` ; modifier
`tests/ajouts.rs` (:103-141).

**Interfaces — consomme :** `sort_d_un_partiel`, `fichier_du_voisin`, `pupitre_neuf`. **Produit :**

```rust
pub struct Lu {
    /// Son chemin relatif au dossier du document, `/` pour séparateur, tel que l'adaptateur le
    /// retirera : `ajouts/<empreinte>.automerge`, ou `complet.automerge.<processus>.partiel`.
    pub nom: String,
    /// Lisible : un fichier d'ajout dont le contenu donne l'empreinte de son nom et que
    /// `load_incremental` charge, ou un voisin du fichier complet non vide que `load` relit sans
    /// erreur.
    pub lisible: bool,
    pub absorbe: bool,
}
// Avertissement : + VoisinRestant { chemin: PathBuf, erreur: io::Error }
/// Le voisin `chemin` d'un fichier d'ajout n'a pu prendre sa place : il se lit où il est, et se
/// met en place à l'ouverture suivante.
// lire_le_dossier(dossier: &Path)
//     -> Result<(Automerge, Option<LectureSeule>, usize, usize), Erreur>
// la mémoire, la première raison de lecture seule, le nombre de fichiers d'ajout lus (un voisin
// d'ajout resté compris) et, nouveau, le nombre de voisins du fichier complet lus.
```

À chaque `fonds()`, la garde prise : chaque `.partiel` de `documents/` (`Ailleurs`), de `documents/fonds/`
(`DansLeDossier`) et de `documents/fonds/ajouts/` (`DansAjouts`) reçoit son sort. Un dossier absent
(`documents/fonds/` d'une racine des données neuve, `ajouts/` à côté d'un seul voisin du complet) ne contient
aucun `.partiel` : `NotFound` n'est pas une erreur, et le démarrage ne crée aucun dossier. Toute autre erreur de
`read_dir`, ou un refus du disque à la lecture d'un `.partiel` dont le sort demande le contenu (l'empreinte d'un
`DansAjouts`, `lisible` d'un `DansLeDossier`), rend `Err(Erreur::Disque)`, la règle de la tâche 7, la garde
rendue (tâche 5) : un voisin non lu peut porter une opération dite faite, ou le premier fichier complet, et
l'ouverture sans lui montrerait le document sans elle, ou Pupitre neuf, prêt à une seconde naissance. Le rangement
de `captations/` avertit au contraire devant les mêmes refus (tâche 12) : un nom provisoire qui reste ne perd
rien, et se range à l'ouverture suivante. `MettreEnPlace` :
`fichier::renommer` vers `ajouts/<fichier_du_voisin>`, puis `rendre_durable_apres_le_renommage`. Un refus du
renommage ne fait pas échouer l'ouverture, comme le veut AM-4 (6) pour une erreur du disque à l'ouverture : le
voisin porte une opération dite faite (§ 31 · 10, test 8), il reste où il est, et `lire_le_dossier`, qui lit
dans `ajouts/` chaque `.partiel` dont le sort est `MettreEnPlace`, le charge avec les fichiers d'ajout, dans
l'ordre des noms, et le compte parmi eux ; il pousse `Avertissement::VoisinRestant { chemin, erreur }` ; le
démarrage suivant le remet en place. Un `rendre_durable` refusé après ce renommage laisse le fichier en place, avec
`Avertissement::NonDurable { chemin }` (C2). `Retirer` : `fichier::retirer`, un refus tu, retenté à l'ouverture
suivante. `Lire` : le voisin se charge (`load_incremental`, dans l'ordre des noms) après le fichier complet, ou
sur `Automerge::new()` sans lui, et avant les fichiers d'ajout, dans `lire_le_dossier` (tâche 6) : la relecture du
disque lit donc aussi un voisin du complet qu'une compaction d'ouverture en échec a laissé. `lire_le_dossier`
rend désormais aussi le nombre de voisins du fichier complet lus : `pupitre_neuf(complet présent, voisins lus)`
vrai, `Ok(None)` ; la compaction d'ouverture suit `a_l_ouverture(acces, ajouts lus, voisins lus > 0)`.
`compacter` lit, outre les fichiers d'ajout, chaque voisin du complet de `documents/fonds/`, dont il calcule
`lisible` comme la doc de `Lu` le dit (non vide, et que `load` relit sans erreur), et retire par
`dossier.join(nom)` ; un voisin d'ajout resté n'est pas des siens. `faire_naitre_le_fonds` rend `FondsDejaNe`
quand `pupitre_neuf` est faux, voisins lisibles compris (C3). Un fichier qui n'est ni un `.partiel`, ni
`complet.automerge`, ni un fichier d'ajout, n'est ni lu ni retiré.

- [ ] **Étape 1 : écrire les tests** (`tests/demarrage.rs`, sauf mention) :
  - `rouvrir_relit_le_complet_puis_les_ajouts` réécrit (`tests/ajouts.rs`) : le `.partiel` non conforme déposé dans `ajouts/` a disparu après la réouverture.
  - `le_voisin_conforme_d_un_ajout_prend_sa_place` — `// § 31 · 10, test 8 : renommage perdu après la réponse, le voisin entier reste`. Une écriture faite ; son fichier d'ajout renommé par le test en `<e>.automerge.4242.partiel` ; rouvert, l'idée y est, et `ajouts/` ne garde aucun `.partiel`.
  - `un_voisin_d_ajout_qui_ne_se_met_pas_en_place_se_lit_et_avertit` : de même, le voisin `<e>.automerge.4242.partiel` déposé, lâché ; `couper(Renommage, 1)` (le premier renommage de l'ouverture est celui du démarrage) ; `fonds()` rend un document : l'idée y est, `lecture_seule() == None`, `avertissements()` est `[Avertissement::VoisinRestant { chemin, .. }]`, `chemin` ce voisin, toujours là, octet pour octet ; lâché, rouvert sans panne : l'idée y est, et `ajouts/` ne garde aucun `.partiel`.
  - `un_voisin_incomplet_ou_non_conforme_est_retire` — `// § 31 · 10, test 8 : un voisin incomplet n'est pas lu comme une écriture`. Déposés : un voisin d'ajout tronqué de moitié, un voisin d'ajout vide, le voisin vidé d'une opération refusée (tâche 3), `complet.automerge.4242.partiel` qui porte « pas un document », et `documents/x.4242.partiel` ; rouvert : tous retirés, `lecture_seule() == None`, heads d'avant.
  - `le_voisin_du_complet_se_lit_puis_se_fond` — `// § 31 · 10, test 9 : renommage du fichier complet perdu, les ajouts absorbés déjà retirés, puis rouvrir`. `c0` : le fichier complet après la naissance ; trois écritures ; lâché ; `s = relire_le_dossier_du_fonds(..).save()` ; le test écrit `complet.automerge.4242.partiel` = `s`, remet `c0` en fichier complet, vide `ajouts/` ; rouvert : les trois idées, puis le voisin retiré et le fichier complet relu seul les porte.
  - `un_voisin_au_numero_du_processus_n_est_pas_ecrase` — `// § 31 · 10, test 9 : de même quand le voisin laissé porte le numéro du processus qui rouvre`. Comme le précédent, le voisin nommé `complet.automerge.<std::process::id()>.partiel` ; `couper(Renommage, 1)` ; `fonds()` rend un document avec `[Avertissement::Compaction(Erreur::Disque { .. })]` ; le voisin laissé garde son empreinte ; lâché, rouvert sans panne : les trois idées.
  - `a_la_naissance_le_voisin_du_premier_complet_fait_exister_le_fonds` — `// § 31 · 10, test 8 : à la naissance, renommage du premier fichier complet perdu, puis relancer`. Né, lâché ; `complet.automerge` renommé en `complet.automerge.4242.partiel` ; `faire_naitre_le_fonds` rend `Err(Erreur::FondsDejaNe)` et le voisin garde son empreinte ; `fonds()` rend le fonds avec son idée, puis `complet.automerge` existe et le voisin a disparu.
  - `un_voisin_du_complet_vide_laisse_pupitre_neuf` (Éc-4) : seul `complet.automerge.4242.partiel`, vide ; `fonds()` rend `Ok(None)` et le voisin a disparu ; `faire_naitre_le_fonds` réussit.
  - `deux_voisins_du_complet_se_lisent_puis_se_fondent` (§ 31 · 2.1 : « un nom déjà pris reçoit un suffixe » ; deux voisins laissés sous le même numéro de processus) : sans fichier complet, `….4242.partiel` porte la naissance, `….4242-2.partiel` la naissance et une idée ; rouvert : l'idée y est, les deux voisins sont retirés, `complet.automerge` existe.
  - `un_fichier_etranger_est_laisse_tel_quel` (point de revue 3), une caractérisation (le code d'H07 ne lit et ne retire déjà que les `.automerge` d'`ajouts/`, et le test passe dès qu'il compile), que la description relève comme telle : `ajouts/desktop.ini` et `documents/fonds/Thumbs.db` (« inventé ») ; rouvert puis `compacter()` : les deux toujours là, octet pour octet, `lecture_seule() == None`, heads d'avant.
  - `le_demarrage_ne_touche_rien_sans_le_verrou` (§ 31 · 2.1 : « Le démarrage ne retire ni ne renomme rien dans `documents/` ni dans `captations/` avant d'avoir pris le verrou »), une caractérisation (`ouvrir` ne touche déjà rien), que la description relève comme telle : une racine ouverte ; des `.partiel` déposés dans `ajouts/` et `captations/` ; `RacineDesDonnees::ouvrir` du même chemin rend `Err(Erreur::RacineDesDonneesTenue)` ; `empreintes(<racine>/documents)` et `empreintes(<racine>/captations)` inchangés, jamais `empreintes(<racine>)` : sous Windows, le verrou que tient la racine ouverte peut refuser la lecture du fichier `verrou` (`h07.rs`, :252-253).
- [ ] **Étape 2 : voir échouer.** Committer (`test: le démarrage dans documents`), pousser. Attendu : « Lint » rouge, E0599, `no variant or associated item named VoisinRestant found for enum Avertissement`.
- [ ] **Étape 3 : écrire le démarrage, la lecture des voisins, `compacter` et la garde de la naissance.**
- [ ] **Étape 4 : voir passer.** Committer (`feat: le démarrage met en place, lit ou retire chaque voisin de documents`), pousser. Attendu : `tests/demarrage.rs` `ok. 10 passed`, `ajouts.rs` `ok. 13`, dans `verifier` et `windows` ; `mutants` vert.

### Tâche 12 : les noms provisoires de `captations/` rangés, et les documents de la PR (c)

**Fichiers :** modifier `crates/stockage/src/captations.rs`, `src/documents.rs` (fin de l'ouverture),
`src/lib.rs` (`Avertissement`) ; tests : `tests/demarrage.rs` ; documents :
`docs/31-persistance-et-synchronisation.md` (§ 2.1 :209-216, § 2.2 point 1, § 6.1 :624-636, § 10 test 4 :898),
`docs/80-glossaire.md` (Voisin, :112), `.github/workflows/ci.yml`, `.github/workflows/mutants-complet.yml`.

**Interfaces — consomme :** `demarrage::{a_ranger, Provisoire}`. **Produit :**

```rust
impl Captations<'_> {
    /// Range sous son empreinte chaque nom provisoire que `a_ranger` désigne (§ 31 · 6.1) ; rend
    /// ceux qui n'ont pu l'être.
    pub(crate) fn ranger_les_noms_provisoires(&self, nommees: &BTreeSet<String>) -> Vec<Avertissement>;
}
// Avertissement : + NomProvisoireRestant { chemin: PathBuf, erreur: io::Error }
/// Le nom provisoire `chemin` n'a pu se lire, ou prendre le nom de son empreinte : il reste, et se
/// range à l'ouverture suivante.
```

Les noms provisoires sont les fichiers de `captations/` même, triés par nom, lus en entier, chacun par
`fichier::lire`. `nommees` : les clés de `PRISES` de la racine du document, telles qu'elles se lisent, même en
lecture seule. Le rangement suit la compaction d'ouverture ; il crée le sous-dossier au besoin (et, hors de
Windows, rend `captations/` durable), renomme par `fichier::renommer`, puis `rendre_durable_apres_le_renommage` ;
ses avertissements s'ajoutent à `avertissements()`. Une erreur du rangement ne fait jamais échouer l'ouverture
(AM-4 (6) ; décision 12 : une captation se verse toujours, et un nom provisoire n'est jamais retiré) : un nom
provisoire qui ne se lit pas n'entre pas dans `a_ranger`, reste intact, et donne
`NomProvisoireRestant { chemin, erreur }`, comme un renommage refusé ; un refus de lister `captations/` en donne
un, `chemin` ce dossier, et rien ne se range.

- [ ] **Étape 1 : écrire les tests** (`tests/demarrage.rs`) :
  - `une_captation_nommee_prend_son_nom_au_demarrage` — `// § 31 · 10, test 4 : renommage perdu après la réponse, la captation nommée par un document prend son nom au démarrage`. Né avec la prise d'empreinte `e`, `e = empreinte(&octets(5, 300))` (`poser_une_prise`, au premier geste) ; le sous-dossier `captations/<e[..2]>/` n'existe pas (point de revue 4) ; déposés : `captations/<e>.4242.partiel` = `octets(5, 300)`, un nom provisoire d'un contenu non nommé (`octets(8, 100)`), et `captations/<e2>.4242.partiel` qui porte la moitié d'`octets(9, 300)`, `e2` son empreinte entière, nommée aussi ; rouvert : `relire(&e)` rend `octets(5, 300)`, `captations/<e>.4242.partiel` a disparu, les deux autres restent intacts, `verifier()` rend `[]`.
  - `un_nom_provisoire_qui_ne_se_range_pas_avertit` : le même dépôt du nom provisoire de `e` ; `couper(Renommage, 1)` ; `fonds()` rend un document, `avertissements()` est `[Avertissement::NomProvisoireRestant { chemin, .. }]`, ce nom ; lâché, rouvert sans panne : rangée. Puis, dans une racine neuve, le même dépôt sous `couper(Relecture, 2)` (la première relecture de l'ouverture est celle du fichier complet, la seconde celle du nom provisoire) : `fonds()` rend un document, `avertissements()` est `[Avertissement::NomProvisoireRestant { chemin, .. }]`, ce nom, intact, et la captation n'est pas rangée ; rouvert sans panne : rangée.
  - `en_lecture_seule_seul_ce_qui_se_lit_nomme` (décision 12 du plan maître ; en lecture seule, « que le fonds nomme » porte sur ce qui se lit) : né avec la prise d'empreinte `e` ; une écriture pose la prise d'empreinte `e9 = empreinte(&octets(9, 300))`, puis un octet de son fichier d'ajout est changé ; déposés : `captations/<e>.4242.partiel` = `octets(5, 300)` et `captations/<e9>.4242.partiel` = `octets(9, 300)` ; rouvert : `lecture_seule() == Some(&LectureSeule::AjoutAltere { .. })` ; `relire(&e)` rend `octets(5, 300)` et son nom provisoire a disparu ; `<e9>.4242.partiel` reste, intact, `e9` n'étant nommée que par le fichier qui ne se lit pas.
- [ ] **Étape 2 : voir échouer.** Committer (`test: les noms provisoires nommés prennent leur nom au démarrage`), pousser. Attendu : « Lint » rouge, E0599, `no variant or associated item named NomProvisoireRestant found for enum Avertissement`.
- [ ] **Étape 3 : écrire `ranger_les_noms_provisoires`, et son appel.**
- [ ] **Étape 4 : voir passer.** Committer (`feat: au démarrage, un nom provisoire nommé par le fonds prend son nom, partout`), pousser. Attendu : `tests/demarrage.rs` `ok. 13 passed`, dans `verifier` et `windows`.
- [ ] **Étape 5 : les documents.** § 31 · 6.1 : « Sous Windows, où un renommage peut se perdre… » devient « Un renommage peut se perdre après la réponse, sous Windows comme ailleurs (§ 2.1) : au démarrage, à l'ouverture du fonds, un nom provisoire de `captations/` dont le contenu entier donne une empreinte que le fonds nomme, et qu'aucun fichier ne porte encore, prend ce nom ; de deux noms provisoires du même contenu, le premier dans l'ordre des noms. » ; le nom provisoire, `<empreinte>.<processus>.partiel`, prend le suffixe `-2`, `-3`… si le nom est pris. § 31 · 2.1 et § 2.2, point 1 : « le voisin entier » et « que `load` relit sans erreur » deviennent « le voisin lisible : non vide, et que `load` relit sans erreur » ; le voisin d'un fichier d'ajout que le disque refuse de mettre en place se lit où il est, avec un avertissement, et se met en place au démarrage suivant ; un `.partiel` d'un autre nom de `documents/` est retiré, un autre fichier est laissé. § 31 · 10, test 4 : « la captation nommée par un document reprend son nom au démarrage » devient « la captation nommée par un document prend son nom au démarrage », le verbe que suit le code (C10), et la clause que citent les marqueurs de `les_noms_provisoires_a_ranger` et d'`une_captation_nommee_prend_son_nom_au_demarrage`. Glossaire, Voisin : « …`<nom>.<processus>.partiel`, ou, si ce nom est pris, `<nom>.<processus>-2.partiel`, `-3`… : écrit, synchronisé sur le disque, fermé et relu, il prend ensuite la place du fichier par un renommage. … Le nom provisoire d'une captation se forme de même, mais ne remplace aucun fichier : ce n'est pas un voisin ». `git grep -n -i "partiel" -- docs CONVENTIONS.md` (LE-3), la sortie citée dans la PR.
- [ ] **Étape 6 : remesurer la mutation, à froid** (comme à la tâche 8, `--ref t1c-demarrage-et-suffixe`), puis committer par chemins (`docs: le démarrage, le suffixe et les noms provisoires, partout`, `ci: les délais de la mutation, remesurés avec le démarrage`), pousser. **Fin de la PR (c) :** tous les jobs verts, `mutants` sans `MISSED` ; la description dit les tests réécrits (`le_voisin_porte_le_numero_du_processus`, `rouvrir_relit_le_complet_puis_les_ajouts`), le test adapté à la seule signature (`une_captation_alteree_ou_absente_se_nomme`, `voisin(…).unwrap()`), les deux caractérisations de la tâche 11 (`un_fichier_etranger_est_laisse_tel_quel`, `le_demarrage_ne_touche_rien_sans_le_verrou`) et les recherches ; `gh pr ready <n>`, puis fusion par squash.

---

## PR (d) · la version de schéma, les migrations, la teneur, la lignée, le test 11

**Branche :** `t1d-version-teneur-lignee`. **Titre :** `feat: la version de schéma, les migrations, la teneur,
la lignée et le test 11 (AM-4)` (85). **Documents :** § 31 · 2.4 et 2.6 ; § 40 · 3 et `CLAUDE.md` (`fonds_selon`,
sous `pannes`) ; `CONVENTIONS.md` (`mutants`) ; `.cargo/mutants.toml` ; délais de `mutants`, et de `verifier` et
`windows` ; registre (AM-4 et LE-12 clos) ; si le repli de la tâche 14 joue, glossaire (Lecture seule) et, au
registre, la ligne de la contradiction. **Fin :** voir la tâche 17.

### Tâche 13 : la version de schéma et les migrations

**Fichiers :** créer `crates/noyau/src/donnees/version.rs`, `crates/stockage/tests/version.rs` ; modifier
`donnees/mod.rs`, `donnees/acces.rs`, `donnees/fonds.rs`, `crates/noyau/tests/donnees.rs`,
`crates/stockage/src/lib.rs` (`LectureSeule`), `src/racine_des_donnees.rs`, `src/documents.rs` (ouverture),
`tests/commun/mod.rs`, `.cargo/mutants.toml`, `CONVENTIONS.md` (`mutants`).

**Interfaces — produit :**

```rust
// noyau::donnees::version (§ 31 · 2.4)
/// Une migration écrite d'avance : la modification, en octets, qui fait passer le fonds à `vers`,
/// et la version de Pupitre qui l'a apportée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Migration<'a> { pub vers: u64, pub octets: &'a [u8], pub version_de_pupitre: &'a str }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lecture { Ecrire, Migrer { de: u64, vers: u64 }, LireSansEcrire, Illisible }
/// Some(v), 1 ≤ v < connue : Migrer ; v == connue : Ecrire ; v > connue : LireSansEcrire ;
/// None ou Some(0) : Illisible.
pub fn lecture(version: Option<u64>, connue: u64) -> Lecture;
/// 1 + le nombre de migrations : la table se suit, `vers` = 2, 3…
pub fn connue(migrations: &[Migration<'_>]) -> u64;
/// Les migrations au-delà de `de`.
pub fn a_appliquer<'m, 'a>(migrations: &'m [Migration<'a>], de: u64) -> &'m [Migration<'a>];
/// « Fonds passé à la version de schéma {vers} ».
pub fn message_de_la_migration(vers: u64) -> String;
// noyau::donnees::fonds : pub const MIGRATIONS: &[Migration<'static>] = &[];
// noyau::donnees::acces : Acces + PlusRecent ; ecrit() faux, echange() vrai ; la doc d'echange() dit « Entier et PlusRecent ».
// stockage::LectureSeule : + VersionPlusRecente { version: u64 }, + VersionIllisible
impl RacineDesDonnees {
    pub fn fonds(&self) -> Result<Option<Document<'_>>, Erreur>; // = l'ouverture selon MIGRATIONS
    /// L'ouverture selon la table `migrations` qu'un test lui donne : un point d'entrée des tests,
    /// hors de l'API livrée.
    #[cfg(feature = "pannes")]
    pub fn fonds_selon(&self, migrations: &[Migration<'_>]) -> Result<Option<Document<'_>>, Erreur>;
}
// documents::rouvrir_le_fonds(racine, migrations) : pub(crate), compilée dans les deux configurations,
// que `fonds()` appelle avec `MIGRATIONS` et `fonds_selon` avec sa table.
// tests/commun/mod.rs
pub const CONTENEUR_DE_LA_MIGRATION: &str = "elements_de_test";
/// La migration de test vers la version de schéma 2, engendrée depuis `GENESE` : acteur
/// `ACTEUR_DE_LA_GENESE`, `put(ROOT, VERSION_DE_SCHEMA, 2u64)`, `put_object(ROOT,
/// CONTENEUR_DE_LA_MIGRATION, ObjType::Map)`, message `message_de_la_migration(2)`, heure 0 ; les
/// octets de sa modification (`raw_bytes()`), comme `genese_par_la_recette` de `tests/genese.rs`.
pub fn migration_de_test() -> Vec<u8>;
/// [Migration { vers: 2, octets, version_de_pupitre: "0.0.0-essai" }].
pub fn migrations_de_test(octets: &[u8]) -> [Migration<'_>; 1];
```

À l'ouverture, un document qui se lit entier lit `VERSION_DE_SCHEMA` (`ScalarValue::Uint(v)` → `Some(v)`, tout
autre → `None`), puis : `Ecrire` → entier ; `Migrer { de, .. }` → chaque migration de `a_appliquer`
s'applique (`Change::try_from`, `apply_changes`, `expect` : la table est du code, que ses tests éprouvent),
puis `save_after` des heads d'avant s'écrit en un fichier d'ajout ; son refus rend `Err(Erreur::Disque)`, son
`NonDurable` un avertissement, et lève `compaction_due`, comme après une écriture ; `LireSansEcrire` →
`VersionPlusRecente { version }` ; `Illisible` → `VersionIllisible`. Un document illisible ne migre pas. `LectureSeule::acces` : `VersionPlusRecente` →
`PlusRecent`, les autres → `Illisible`. En `PlusRecent`, `recevoir` applique et écrit son fichier d'ajout, et
`apres_une_ecriture` ne compacte pas.

L'ordre de l'ouverture : le démarrage (tâche 11), puis `lire_le_dossier`, qui rend le nombre de fichiers d'ajout
et de voisins du complet lus ; puis la version de schéma, et la migration s'il y a lieu ; puis la compaction
d'ouverture, décidée par `a_l_ouverture(acces, ajouts lus, voisins lus > 0)` sur les nombres lus **avant** la
migration, l'accès étant celui qui suit la lecture de la version ; puis le rangement des noms provisoires
(tâche 12). Le fichier d'ajout de la migration ne compte donc pas : un fonds sans fichier d'ajout ni voisin qui
migre garde ce fichier dans `ajouts/`, ce que veulent le second cas de
`une_migration_que_le_disque_refuse_n_ouvre_pas_le_fonds` et la tâche 14 ; un fonds qui a déjà des fichiers
d'ajout se compacte, et la compaction fond aussi celui de la migration.

`fonds_selon` vit sous la fonction de Cargo `pannes` : c'est un point d'entrée des tests, et l'établi, qui
n'active jamais `pannes` (décision 7 du plan maître), ne le voit pas. La tranche 2 en aura besoin pour ouvrir un
fonds de version de schéma N + 1 sur la paire de fichiers, mais dans ses tests, qui activent déjà `pannes` pour
ses commandes coupées par une panne : la condition ne lui coûte rien. Si son plan en a besoin hors de ses tests, il
lève la condition et le dit.

- [ ] **Étape 1 : écrire les tests.**
  - Noyau (`tests/donnees.rs`) :

```rust
#[test]
fn la_version_de_schema_decide_de_la_lecture() {
    assert_eq!(lecture(Some(1), 1), Lecture::Ecrire);
    assert_eq!(lecture(Some(1), 2), Lecture::Migrer { de: 1, vers: 2 });
    assert_eq!(lecture(Some(2), 3), Lecture::Migrer { de: 2, vers: 3 });
    assert_eq!(lecture(Some(2), 1), Lecture::LireSansEcrire);
    assert_eq!(lecture(Some(3), 1), Lecture::LireSansEcrire);
    assert_eq!(lecture(None, 1), Lecture::Illisible);
    assert_eq!(lecture(Some(0), 1), Lecture::Illisible);
    let (m2, m3) = (Migration { vers: 2, octets: b"m2", version_de_pupitre: "0.4.0" },
                    Migration { vers: 3, octets: b"m3", version_de_pupitre: "0.5.0" });
    assert_eq!((connue(&[]), connue(&[m2])), (1, 2));
    assert_eq!(a_appliquer(&[m2, m3], 1), [m2, m3]);
    assert_eq!(a_appliquer(&[m2, m3], 2), [m3]);
    assert!(a_appliquer(&[m2, m3], 3).is_empty());
    assert_eq!(message_de_la_migration(2), "Fonds passé à la version de schéma 2");
    assert!(MIGRATIONS.iter().enumerate().all(|(i, m)| m.vers == i as u64 + 2));
}
#[test]
fn l_acces_d_un_document_plus_recent() {
    assert!(!Acces::PlusRecent.ecrit() && Acces::PlusRecent.echange());
    assert!(!a_l_ouverture(Acces::PlusRecent, 3, true));
    assert!(!apres_une_ecriture(Acces::PlusRecent, false, 1600, 15150, 101)); // § 31 · 2.2 : en lecture seule, pas de compaction
}
```

  - Stockage (`tests/version.rs`) :
    - `migrer_a_l_ouverture_ecrit_la_migration_avec_son_message` (plan maître, tranche 1, sixième puce : « les migrations, éprouvées par une migration de test engendrée à l'exécution depuis la genèse courante, de façon déterministe (acteur fixe, heure 0) » ; § 31 · 2.4 ; I28) : né, lâché ; `fonds_selon(&migrations_de_test(&m))` : `VERSION_DE_SCHEMA` vaut `ScalarValue::Uint(2)` ; une modification de plus, de message « Fonds passé à la version de schéma 2 », d'heure 0, d'acteur `ActorId::from(ACTEUR_DE_LA_GENESE)`, d'empreinte celle de `Change::try_from(m.as_slice())` ; `CONTENEUR_DE_LA_MIGRATION` est un objet à clés de la racine du document. Lâché, rouvert de même : aucune modification de plus.
    - `un_document_n_plus_1_se_lit_sans_ecrire` — `// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que N`. Migré comme ci-dessus, lâché ; `fonds()` : `lecture_seule() == Some(&LectureSeule::VersionPlusRecente { version: 2 })` ; `ecrire` et `compacter` rendent `Err(Erreur::LectureSeule(LectureSeule::VersionPlusRecente { version: 2 }))` ; `verser(&octets(4, 100))` rend `Ok(_)` ; `empreintes(<dossier du fonds>)` inchangé.
    - `deux_migrations_separees_n_en_font_qu_une` — `// § 31 · 10, test 10 : migrer deux répliques chacune de son côté, puis les fusionner`. `deux_copies` ; chacune ouverte par `fonds_selon` ; `reunir` ; dans chacune, une seule modification au message de la migration, et les mêmes heads.
    - `deux_elements_de_la_donnee_nouvelle_restent` — `// § 31 · 10, test 10 : deux éléments différents de la donnée nouvelle d'un même objet ancien`. Après la migration des deux, `a` pose `idee-1/a` = « x » et `b` `idee-1/b` = « y » sous `CONTENEUR_DE_LA_MIGRATION` ; `reunir` ; les deux clés dans les deux copies.
    - `une_version_de_schema_illisible_ouvre_en_lecture_seule` (une version de schéma qui n'est pas un `Uint`, dont le § 31 · 2.4 ne disait rien ; voir « Le passage de LE-6 ») : une écriture pose `VERSION_DE_SCHEMA` = « deux » (`Str`) ; rouvert : `lecture_seule() == Some(&LectureSeule::VersionIllisible)` ; `ecrire` refusé ; `prochain_message` rend `None`.
    - `un_document_illisible_ne_migre_pas` (point de revue 5) : un fichier d'ajout altéré ; `fonds_selon(&migrations_de_test(&m))` : `lecture_seule() == Some(&LectureSeule::AjoutAltere { .. })`, aucune modification au message de la migration, `empreintes(<dossier du fonds>)` inchangé.
    - `une_migration_que_le_disque_refuse_n_ouvre_pas_le_fonds` (C2) : né, lâché, sans fichier d'ajout, voisin ni nom provisoire, si bien que le premier renommage et le premier `rendre_durable` de l'ouverture sont ceux de la migration. Sous `couper(Renommage, 1)`, `fonds_selon(&migrations_de_test(&m))` rend `Err(Erreur::Disque { .. })` et `empreintes(<dossier du fonds>)` est inchangé ; puis, sans panne, `fonds_selon` rend le fonds migré (la garde rendue). Dans une racine neuve, sous `couper(RendreDurable, 1)`, `fonds_selon` rend le fonds migré, et `avertissements()` est `[Avertissement::NonDurable { chemin, .. }]`, `chemin` sous `ajouts/` et présent.
- [ ] **Étape 2 : voir échouer.** Committer (`test: la version de schéma et les migrations`), pousser. Attendu : « Lint » rouge, `unresolved import noyau::donnees::version`.
- [ ] **Étape 3 : écrire `version.rs`, `PlusRecent`, `fonds_selon` (sous `pannes`) et la migration à l'ouverture, dans l'ordre dit plus haut.** `.cargo/mutants.toml` : `version.rs` dans `examine_globs` (ses tests sont dans `donnees.rs`, déjà nommé) ; en tête, « Douze modules » et « les règles des données (la compaction, l'accès d'un document, le démarrage, la version de schéma et la comparaison des copies) » ; l. 17, « ces douze modules ». `CONVENTIONS.md`, ligne `mutants` : « …les règles des données : la compaction, l'accès d'un document, le démarrage, la version de schéma et la comparaison des copies ».
- [ ] **Étape 4 : voir passer.** Committer (`feat: la version de schéma au noyau, et les migrations à l'ouverture`), pousser. Attendu : `donnees.rs` `ok. 7 passed` dans `verifier` ; `version.rs` `ok. 7 passed` dans `verifier` et `windows` ; `mutants` vert.

### Tâche 14 : une copie plus récente reçoit sans rien transmettre

**Fichiers :** modifier `crates/synchro/src/lib.rs`, `crates/synchro/tests/echange.rs`,
`crates/stockage/src/documents.rs` (`prochain_message`, :66-71), `crates/stockage/tests/version.rs` ; si le repli
joue : `crates/noyau/src/donnees/acces.rs` (`echange()` faux), `crates/noyau/tests/donnees.rs`, `docs/registre.md`
(la ligne de la contradiction).

**Interfaces — produit :**

```rust
impl Echange {
    /// Le prochain message, sans aucune modification (§ 31 · 2.4) : celui de
    /// `generate_sync_message`, `changes` remplacé par `ChunkList::empty()` et `heads` par celles de
    /// l'autre copie (le champ public `State::their_heads`, vide tant qu'elle n'a rien dit). Sans ces heads, l'autre
    /// copie redemanderait sans fin ce qu'elle voit annoncé, et l'échange ne finirait pas.
    pub fn prochain_message_sans_modification(&mut self, doc: &Automerge) -> Option<Vec<u8>>;
}
// Document::prochain_message : Entier → prochain_message ; PlusRecent →
// prochain_message_sans_modification ; Illisible → None.
```

- [ ] **Étape 1 : écrire les tests.**
  - `crates/synchro/tests/echange.rs`, `une_copie_n_plus_1_recoit_sans_rien_transmettre` — `// § 31 · 10, test 10 : la copie plus récente reçoit, et ne transmet aucune modification`. `a = naitre(&copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_1))`, `b = a.fork()` ; `ecrire(&mut a, "a", "x")`, `ecrire(&mut b, "b", "y")` ; au plus `TOURS_AU_PLUS` tours, `a` envoie par `prochain_message_sans_modification`, `b` par `prochain_message`, chacun reçoit ; la boucle finit avant la borne, les deux à `None` ; avec l'aide existante, d'un seul argument, `valeur_de(&b).get("a") == None` et `valeur_de(&a).get("b") == Some(&vec!["y".to_string()])`.
  - `crates/stockage/tests/version.rs`, `une_copie_plus_recente_recoit_sans_rien_transmettre` : `deux_copies` ; la racine de `a` migrée par `fonds_selon`, lâchée, rouverte par `fonds()` (`PlusRecent`) ; `b` note idee-2 ; `avant = entrees(<ajouts de a>)`, où se trouve déjà le fichier d'ajout de la migration ; `commun::reunir(&mut a, &mut b)` finit ; `a` porte idee-2, son fichier complet est celui d'avant la réunion, et `ajouts/` porte au moins un fichier de plus qu'`avant`, chacun nommé par l'empreinte de son contenu ; `a` lâchée puis rouverte par `fonds()` porte idee-2 ; `b` n'a aucune modification au message de la migration, et sa version de schéma vaut `Uint(1)`.
- [ ] **Étape 2 : voir échouer.** Committer (`test: une copie plus récente reçoit sans rien transmettre`), pousser. Attendu : « Lint » rouge, `no method named prochain_message_sans_modification`.
- [ ] **Étape 3 : écrire `prochain_message_sans_modification`**, et son appel dans `Document::prochain_message`.
- [ ] **Étape 4 : voir passer.** Committer (`feat: une copie plus récente reçoit, ses messages partent sans modification`), pousser. Attendu : `echange.rs` `ok. 4 passed`, `version.rs` `ok. 8`, dans `verifier` et `windows`. **Si la boucle atteint sa borne**, la tâche s'arrête : la PR (d) consigne la contradiction au motif « Une spec qui se contredit » (décision 20 du plan maître), retire la fonction et son test de `synchro`, et fait de `PlusRecent` un accès qui n'échange pas : `Acces::PlusRecent.echange()` devient faux (`l_acces_d_un_document_plus_recent`, au noyau, le suit), si bien que `prochain_message` rend `None` et `recevoir` rend `Err(Erreur::LectureSeule(LectureSeule::VersionPlusRecente { .. }))` ; une copie plus récente n'envoie ni ne reçoit avant sa mise à jour. `une_copie_plus_recente_recoit_sans_rien_transmettre` devient alors `une_copie_plus_recente_n_envoie_ni_ne_recoit` : `deux_copies` ; la racine de `a` migrée par `fonds_selon`, lâchée, rouverte par `fonds()` ; `b` note idee-2 ; `avant = empreintes(<dossier du fonds de a>)` ; `a.prochain_message(..)` rend `None` ; le premier message de `b` rend, à `a.recevoir(..)`, `Err(Erreur::LectureSeule(LectureSeule::VersionPlusRecente { version: 2 }))` ; `a` ne porte pas idee-2, et `empreintes(<dossier du fonds de a>) == avant` ; `b` n'a aucune modification au message de la migration. Attendu alors : `echange.rs` `ok. 3`, `version.rs` `ok. 8`. Le § 31 · 2.4 et l'entrée « Lecture seule (d'un document) » du glossaire le disent (tâche 17), et la question va à la maîtrise d'ouvrage avec ce défaut.

### Tâche 15 : la teneur

**Fichiers :** créer `crates/noyau/src/donnees/teneur.rs`, `crates/noyau/tests/teneur.rs`,
`crates/stockage/tests/teneur.rs` ; modifier `donnees/mod.rs`, `donnees/fonds.rs`, `crates/stockage/src/lib.rs`,
`crates/stockage/tests/compaction.rs` (le test 9), `.cargo/mutants.toml`, `CONVENTIONS.md` (`mutants`).

**Interfaces — produit :**

```rust
// noyau::donnees::fonds
/// À la racine du document : les lignées que remplace la lignée (§ 31 · 2.5), que seule la naissance
/// d'un document reconstruit pose, au lot 4. La teneur l'exclut.
pub const LIGNEES_REMPLACEES: &str = "lignees_remplacees";
// noyau::donnees::teneur (§ 31 · 2.6)
/// Une valeur que présente un document, que `stockage` tire du document.
#[derive(Clone, Debug, PartialEq)]
pub enum Valeur {
    ObjetACles(BTreeMap<String, Valeur>), // `Map` ou `Table` : une seule sorte
    Liste(Vec<Valeur>),
    Texte(String),                        // un objet `Text`
    Str(String), Int(i64), Uint(u64), F64(f64), Counter(i64), Timestamp(i64), Boolean(bool),
    Bytes(Vec<u8>), Null,
    Inconnu { type_code: u8 },            // `ScalarValue::Unknown`
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErreurDeTeneur {
    NaN { chemin: Vec<String> },
    Inconnu { chemin: Vec<String>, type_code: u8 },
}
/// Le texte déterministe de la teneur de la racine du document, sans `LIGNEE` ni `LIGNEES_REMPLACEES`.
pub fn ecrire_la_teneur(racine: &BTreeMap<String, Valeur>) -> Result<String, ErreurDeTeneur>;
// stockage
/// La teneur de `doc` : son arbre, lu sur la valeur que présente Automerge (`ReadDoc`), puis écrit.
pub fn teneur(doc: &Automerge) -> Result<String, ErreurDeTeneur>;
```

La forme du texte, en UTF-8, sans fin de ligne ; le chemin d'une erreur nomme chaque clé, et l'indice décimal
dans une liste :

```text
objet  := "{" [ chaine ":" valeur { "," chaine ":" valeur } ] "}"   les clés dans l'ordre de leurs octets
liste  := "[" [ valeur { "," valeur } ] "]"
chaine := '"' octets, chaque '\' et '"' précédé de '\', rien d'autre d'échappé ni de normalisé '"'
valeur := objet | liste | "t" chaine | "s" chaine | "i" décimal | "u" décimal | "c" décimal
        | "d" décimal | "f" flottant | "b0" | "b1" | "x" hexadécimal minuscule | "n"
flottant := la plus courte de format!("{}") et de format!("{:e}"), la première à égalité ;
            `inf`, `-inf` ; NaN refusé
```

- [ ] **Étape 1 : écrire les tests.** Noyau (`tests/teneur.rs`), avec `fn t(paires: &[(&str, Valeur)]) -> Result<String, ErreurDeTeneur>` qui bâtit la racine :

```rust
#[test]
fn les_cles_se_rangent_par_octets() {
    let racine = ["a", "Z", "é", "z", "_"].map(|k| (k, Valeur::Null));
    assert_eq!(t(&racine).unwrap(), r#"{"Z":n,"_":n,"a":n,"z":n,"é":n}"#);
}
#[test]
fn chaque_valeur_s_ecrit_avec_son_type_exact() {
    let racine = [("b", Valeur::Boolean(true)), ("b0", Valeur::Boolean(false)), ("c", Valeur::Counter(3)),
        ("d", Valeur::Timestamp(-5)), ("e", Valeur::Liste(vec![])),
        ("f", Valeur::F64(0.5)), ("i", Valeur::Int(-7)), ("l", Valeur::Liste(vec![Valeur::Uint(1), Valeur::Str("a".into())])),
        ("n", Valeur::Null), ("o", Valeur::ObjetACles(BTreeMap::new())), ("s", Valeur::Str("é\"\\".into())),
        ("t", Valeur::Texte("x".into())), ("u", Valeur::Uint(7)), ("x", Valeur::Bytes(vec![0x00, 0xab]))];
    assert_eq!(t(&racine).unwrap(),
        r#"{"b":b1,"b0":b0,"c":c3,"d":d-5,"e":[],"f":f0.5,"i":i-7,"l":[u1,s"a"],"n":n,"o":{},"s":s"é\"\\","t":t"x","u":u7,"x":x00ab}"#);
}
#[test]
fn un_f64_s_ecrit_sous_sa_forme_la_plus_courte() {
    for (x, forme) in [(0.1, "0.1"), (1e300, "1e300"), (-0.0, "-0"), (0.0, "0"), (100.0, "100"),
        (0.01, "0.01"), (1e15, "1e15"), (1e-7, "1e-7"), (1.5, "1.5"),
        (f64::INFINITY, "inf"), (f64::NEG_INFINITY, "-inf")] {
        assert_eq!(t(&[("v", Valeur::F64(x))]).unwrap(), format!(r#"{{"v":f{forme}}}"#));
    }
}
#[test]
fn nan_et_un_type_inconnu_se_refusent_en_se_nommant() {
    let nan = Valeur::ObjetACles(BTreeMap::from([("b".to_string(), Valeur::F64(f64::NAN))]));
    assert_eq!(t(&[("a", nan)]), Err(ErreurDeTeneur::NaN { chemin: vec!["a".into(), "b".into()] }));
    let liste = Valeur::Liste(vec![Valeur::Null, Valeur::Inconnu { type_code: 9 }]);
    assert_eq!(t(&[("k", liste)]), Err(ErreurDeTeneur::Inconnu { chemin: vec!["k".into(), "1".into()], type_code: 9 }));
}
#[test]
fn les_textes_se_prennent_octet_pour_octet() {
    assert_ne!(t(&[("s", Valeur::Str("\u{e9}".into()))]), t(&[("s", Valeur::Str("e\u{301}".into()))]));
    assert_ne!(t(&[("s", Valeur::Str("x".into()))]), t(&[("s", Valeur::Texte("x".into()))]));
}
#[test]
fn la_teneur_laisse_les_lignees_de_cote() {
    let racine = [(LIGNEE, Valeur::Str("lignee-1".into())),
        (LIGNEES_REMPLACEES, Valeur::Liste(vec![Valeur::Str("lignee-0".into())])),
        ("o", Valeur::ObjetACles(BTreeMap::from([(LIGNEE.to_string(), Valeur::Null)])))];
    assert_eq!(t(&racine).unwrap(), r#"{"o":{"lignee":n}}"#);
}
```

  Stockage (`tests/teneur.rs`) : `la_teneur_prend_la_valeur_que_presente_automerge` (deux `fork()` posent `"k"` = « a » et « b », `merge` ; la teneur contient `"k":s"<la chaîne que rend doc.get(ROOT, "k")>"`) ; `map_et_table_sont_une_seule_sorte` (`put_object(ROOT, "m", ObjType::Map)` et `ObjType::Table`, une même clé dessous : même teneur) ; `un_objet_text_n_est_pas_une_valeur_str` (`ObjType::Text` contre `put` d'une chaîne : `t"…"` contre `s"…"`). Dans `tests/compaction.rs`, `une_compaction_coupee_a_chaque_etape_ne_perd_rien` compare aussi `teneur(..)` du document rouvert à celle d'avant — `// § 31 · 10, test 9 : la teneur ne change pas`.
- [ ] **Étape 2 : voir échouer.** Committer (`test: la teneur`), pousser. Attendu : « Lint » rouge, `unresolved import noyau::donnees::teneur`.
- [ ] **Étape 3 : écrire `teneur.rs`, la constante et `stockage::teneur`.** `.cargo/mutants.toml` : `teneur.rs` dans `examine_globs`, `"--test", "teneur"` dans `additional_cargo_test_args` ; en tête, « Treize modules » et « les règles des données (la compaction, l'accès d'un document, le démarrage, la version de schéma, la teneur et la comparaison des copies) » ; l. 17, « ces treize modules ». `CONVENTIONS.md`, ligne `mutants` : « …les règles des données : la compaction, l'accès d'un document, le démarrage, la version de schéma, la teneur et la comparaison des copies ».
- [ ] **Étape 4 : voir passer.** Committer (`feat: la teneur au noyau, sur l'arbre que le stockage tire du document`), pousser. Attendu : noyau `teneur.rs` `ok. 6 passed` dans `verifier` ; stockage `teneur.rs` `ok. 3` et `compaction.rs` vert, dans `verifier` et `windows` ; `mutants` vert.

### Tâche 16 : la règle de la lignée

**Fichiers :** modifier `crates/noyau/src/donnees/copie.rs`, `crates/noyau/tests/donnees.rs`, `CONVENTIONS.md`
(`mutants`), `.cargo/mutants.toml` (la liste en tête).

**Interfaces — produit :**

```rust
// noyau::donnees::copie (§ 31 · 2.5) ; `comparer`, `Copie`, `Difference` ne changent pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rapport {
    /// Même lignée : les copies s'échangent.
    Meme,
    /// La distante nomme la locale parmi celles qu'elle remplace : la locale se met à part.
    LocaleRemplacee,
    /// La locale nomme la distante : la distante ne fusionne pas.
    DistanteRemplacee,
    /// Aucune ne remplace l'autre, ou chacune nomme l'autre : rien, et Pupitre le dit.
    Etrangeres,
}
/// La liste des lignées remplacées se veut complète, celles que remplaçait une lignée remplacée
/// comprises, que le lot 4 écrira : seule l'appartenance compte (§ 31 · 2.5).
pub fn rapport_des_lignees(locale: &str, remplacees_par_la_locale: &[&str], distante: &str,
    remplacees_par_la_distante: &[&str]) -> Rapport;
```

- [ ] **Étape 1 : écrire le test** (`tests/donnees.rs`) :

```rust
#[test]
fn la_lignee_qui_l_emporte() {
    use Rapport::*;
    assert_eq!(rapport_des_lignees("lignee-1", &[], "lignee-1", &[]), Meme);
    assert_eq!(rapport_des_lignees("lignee-1", &["lignee-0"], "lignee-1", &[]), Meme);
    assert_eq!(rapport_des_lignees("lignee-1", &[], "lignee-2", &["lignee-1"]), LocaleRemplacee);
    assert_eq!(rapport_des_lignees("lignee-2", &["lignee-1"], "lignee-1", &[]), DistanteRemplacee);
    assert_eq!(rapport_des_lignees("lignee-1", &[], "lignee-2", &[]), Etrangeres);
    assert_eq!(rapport_des_lignees("lignee-1", &[], "lignee-3", &["lignee-2"]), Etrangeres);
    assert_eq!(rapport_des_lignees("lignee-1", &[], "lignee-3", &["lignee-2", "lignee-1"]), LocaleRemplacee);
    assert_eq!(rapport_des_lignees("lignee-1", &["lignee-2"], "lignee-2", &["lignee-1"]), Etrangeres);
}
```

- [ ] **Étape 2 : voir échouer.** Committer (`test: la règle de la lignée`), pousser. Attendu : « Lint » rouge, `cannot find function rapport_des_lignees`.
- [ ] **Étape 3 : écrire `rapport_des_lignees`.** `copie.rs` est déjà examiné : le compte des modules ne change pas. `CONVENTIONS.md`, la ligne `mutants` entière d'arrivée, colonne « Ce qu'elle refuse » : « Un mutant du noyau (la théorie, la palette, le clavier et la guitare, et les règles des données : la compaction, l'accès d'un document, le démarrage, la version de schéma, la teneur, et la comparaison des copies et des lignées) qu'aucun test de ces modules ne tue, dans les lignes que la PR modifie ; seulement sur une PR. Le passage complet a lieu chaque lundi et à la demande, par `mutants-complet.yml` ». `.cargo/mutants.toml`, en tête, la même liste : « les règles des données (la compaction, l'accès d'un document, le démarrage, la version de schéma, la teneur, et la comparaison des copies et des lignées) ».
- [ ] **Étape 4 : voir passer.** Committer (`feat: la règle de la lignée, au noyau (AM-4)`), pousser. Attendu : `donnees.rs` `ok. 8 passed` dans `verifier` ; `mutants` vert.

### Tâche 17 : le test 11, les documents de la PR (d), et la clôture de la tranche

**Fichiers :** créer `crates/stockage/tests/references.rs` ; modifier `crates/stockage/tests/captations.rs`,
`tests/fichier.rs`, `tests/ajouts.rs` et `tests/h07.rs` (les marqueurs des tests d'H07, rien d'autre),
`docs/31-persistance-et-synchronisation.md` (§ 2.4 :340-376, § 2.6 :400-425), `docs/40-architecture.md` (§ 3,
`fonds_selon`), `CLAUDE.md` (« Les crates », `stockage`), `docs/registre.md` (AM-4, :27 ; LE-12, :29),
`.github/workflows/ci.yml` (`mutants` ; `verifier` et `windows` si la mesure le demande),
`.github/workflows/mutants-complet.yml` ; `docs/80-glossaire.md` (Lecture seule), si le repli de la tâche 14 joue.

- [ ] **Étape 1 : écrire les tests** (`tests/references.rs`, le seul fichier des empreintes de référence, décision 16 du plan maître). Le document témoin : `NAISSANCE`, puis, en un seul geste, `noter(tx, "idee-1", "Un refrain qui monte, en ré")` et `etiqueter(tx, "idee-1", "refrain")`.

```rust
/// Les empreintes de référence du test 11. Une ligne se change ou se retire avant la `v0.3.0`, et
/// la PR le dit ; dès elle, seule une ligne ajoutée passe (§ 31 · 10, test 11 ; ADR-023).
const GENESE_DU_FONDS: &str = "aa03e0cee230ee1aab90c1d53a6a00e51da250b8b5944b67560dd4f9b301c105";
const TENEUR_DU_TEMOIN: &str = "a349d135957809fcbf348db20cfb69a8752fe5a32b28296f7b8d9744900a58a4";

// § 31 · 10, test 11 : les octets de la genèse et la teneur d'un document témoin, à leurs références
#[test]
fn genese_et_teneur_figees() {
    assert_eq!(empreinte(GENESE), GENESE_DU_FONDS);
    let teneur = teneur(temoin.automerge()).unwrap();
    // {"idees":{"idee-1":{"etiquettes":{"refrain":b1},"texte":s"Un refrain qui monte, en ré"}},
    //  "identifiant":s"fonds-1","prises":{},"proprietaire":s"proprietaire-1","version_de_schema":u1}
    assert_eq!(empreinte(teneur.as_bytes()), TENEUR_DU_TEMOIN, "teneur : {teneur}");
}
// § 31 · 10, test 11 : deux documents qui ne diffèrent que par leur lignée ont la même teneur
#[test]
fn la_lignee_ne_change_pas_la_teneur() {
    // le témoin sous NAISSANCE, et sous NAISSANCE dont la lignée est "lignee-2", une écriture
    // de plus y posant LIGNEES_REMPLACEES = ["lignee-1"] : deux teneurs égales.
}
```

  Puis les tests d'H07 qui prouvent déjà une clause prennent leur marqueur (`// § 31 · 10, test N : <la clause>`), sans autre changement :
  - test 4, « l'empreinte de chaque objet égale son nom » : `une_captation_se_range_sous_son_empreinte` et `une_captation_alteree_ou_absente_se_nomme` (`tests/captations.rs`), `une_copie_restauree_relit_chaque_captation_contre_son_empreinte` et `une_captation_alteree_ou_absente_de_la_copie_se_nomme` (`tests/h07.rs`, sur la copie restaurée) ;
  - test 4, « aucun objet modifié après écriture » : `verser_deux_fois_ne_fait_qu_un_objet` (`tests/captations.rs`) ;
  - test 8, « renommage refusé : le voisin est retiré » : `un_renommage_refuse_retire_le_voisin` (`tests/fichier.rs`) ;
  - test 8, « rouvrir un document […] puis faire une opération : son fichier d'ajout ne porte que sa modification » : `chaque_ouverture_tire_un_acteur_neuf` (`tests/ajouts.rs`).

- [ ] **Étape 2 : voir le vert.** Committer (`test: la genèse et la teneur figées (test 11)`), pousser. Le rouge s'est lu à la tâche 15 : ces tests n'appellent que ce qu'elle livre. Attendu : `references.rs` `ok. 2 passed`, dans `verifier` et `windows`. Une empreinte qui diffère se corrige dans le code de la teneur, jamais dans la référence, qui découle de la forme écrite à la tâche 15.
- [ ] **Étape 3 : le § 31 · 2.4.** « Elle en reçoit encore les modifications par la synchronisation, et ne lui transmet aucune modification : ses messages partent sans modification » (ou, si la tâche 14 s'est arrêtée, la forme de son repli, ici et dans l'entrée « Lecture seule (d'un document) » de `docs/80-glossaire.md`) ; « ce qu'elle reçoit s'écrit en fichiers d'ajout, sans compaction » ; « une version de schéma absente, nulle, ou qui n'est pas un entier non signé, ouvre le document en lecture seule, sans échange ». **Le § 31 · 2.6** : « Un `F64` s'écrit en chiffres les plus courts qui se relisent à l'identique, en notation fixe ou à exposant, la plus courte, la fixe à égalité (`0.1`, `1e300`, `100`) ; `-0` distinct de `0` ; l'infini s'écrit `inf` et `-inf` ; un NaN fait échouer l'écriture. » ; « Sa forme exacte est celle de `noyau::donnees::teneur`, que le test 11 fige » ; et la clé des lignées remplacées, exclue dès la tranche 1. **Le § 40 · 3 et `CLAUDE.md`**, à la fonction `pannes` de `stockage` : « …coupe une écriture à l'étape voulue, et ouvre le fonds selon la table de migrations qu'un test lui donne (`fonds_selon`) ».
- [ ] **Étape 4 : le registre.** AM-4, colonne « État » : « clos le AAAA-MM-JJ : (1) `un_second_fonds_est_refuse` ; (2) `une_ecriture_refusee_relit_le_document_du_disque` ; (3) `un_ajout_altere_ouvre_en_lecture_seule_et_se_nomme`, `un_ajout_manquant_ouvre_en_lecture_seule`, `un_ajout_qu_automerge_refuse_ouvre_en_lecture_seule`, `la_compaction_garde_un_ajout_altere`, `une_compaction_d_ouverture_infidele_ouvre_avec_un_avertissement` ; (4) les tests de `crates/stockage/tests/demarrage.rs`, `un_nom_de_voisin_pris_recoit_un_suffixe` (`crates/stockage/tests/fichier.rs`) et `la_lignee_qui_l_emporte`, la copie remplacée reportée au lot 4 ; (5) `une_compaction_qui_echoue_apres_une_ecriture_ne_la_defait_pas` ; (6) `un_complet_illisible_ouvre_en_lecture_seule_et_se_nomme`, `une_compaction_d_ouverture_refusee_par_le_disque_ouvre_avec_un_avertissement` ; (7) reporté au lot 4 (plan maître, décision 3) ; PR #NN à #MM ». LE-12, colonne « État », son critère de sortie rempli : « clos le AAAA-MM-JJ : les PR (b) #NN, (c) #NN et (d) #NN fusionnées ; sections corrigées : § 31 · 2.1 et 2.2 (b, c), § 31 · 6.1 (c), § 31 · 2.4 et 2.6 (d) ; la question C2 tranchée le 2026-11-09, une précision de l'ADR-023 ». `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add`.
- [ ] **Étape 5 : remesurer.** La mutation, à froid (comme à la tâche 8, `--ref t1d-version-teneur-lignee`) : ce passage complet de `mutants-complet`, sur tout le noyau et non sur le seul diff, vert, sans `MISSED` ni `TIMEOUT`. Puis, sur le dernier passage vert de `ci`, `gh run view <n> --json jobs` : la durée de `verifier` et de `windows` ; si deux fois l'une dépasse son `timeout-minutes`, il passe à deux fois la mesure, arrondie à la minute au-dessus, et son commentaire dit la mesure, la date et la PR ; sinon, la description de la PR donne les deux mesures.
- [ ] **Étape 6 : committer** par chemins (`docs: la version de schéma et la teneur, telles que la tranche 1 les écrit`, `ci: les délais remesurés avec la version de schéma et la teneur`), pousser. **Fin de la PR (d), et de la tranche :** tous les jobs verts ; `mutants` sans `MISSED`, et le passage complet de l'étape 5 vert, sans `MISSED`, son lien dans la description ; la description relève, dans `verifier` et `windows`, les lignes `test result: ok.` de `captations.rs`, `fichier.rs`, `ajouts.rs`, `compaction.rs`, `fonds.rs`, `demarrage.rs`, `version.rs`, `teneur.rs`, `references.rs`, `synchronisation.rs` et `echange.rs`, et de `Doc-tests stockage`, seule preuve de la clause du test 8 « une synchronisation lancée pendant l'écriture ne voit pas l'opération en cours » (tests 4, 8 à 11), et dans `verifier` celles de `donnees.rs`, `demarrage.rs` et `teneur.rs` du noyau ; elle renvoie aux trois sorties de l'essai de la porte `contenu` (le `*.automerge` indexé refusé), dans la description de la PR (a), par son lien ; elle dit les références du test 11, qui changeront en tranche 2 avec le schéma de la `v0.3.0`, et les marqueurs posés sur les tests d'H07 ; aucune dépendance nouvelle ; LE-10, AM-4 et LE-12 clos ; `gh pr ready <n>`, puis fusion par squash.

---

## Auto-relecture

**La couverture**, exigence → tâche. Les numéros E01 à E60 sont ceux du relevé des exigences, dont la description
de la PR #27 porte la table ; chacun est suivi ici de sa source, à lire dans le dépôt.

| Exigences (source) | Tâches |
|---|---|
| E01 (plan maître, tranche 1, « But ») ; E03 (plan maître, « Contraintes générales », « En CI seulement ») ; E09 (plan maître, « Le déroulé », point 3) ; E52 (§ 70 · 4.1 : aucun critère de la SFD livré en entier) ; E60 (décision 6 du plan maître : l'installateur NSIS en artefact) | Contraintes générales ; la fin de chaque PR |
| E02 (plan maître, tranche 1, « Elle dépend de » : les 39 tests d'H07, « réécrits quand un comportement change ») : les huit tests d'H07 qui changent | 6 (`une_ecriture_qui_echoue…`, `un_document_a_relire…`, `ce_qui_est_recu…`), 7 (`un_ajout_manquant…`, `un_ajout_altere…`, `la_compaction_garde…`), 10 (`le_voisin_porte…`), 11 (`rouvrir_relit…`) ; deux de plus, adaptés à la seule signature : 6 (`un_geste_refuse…`, `Ok(_)`), 10 (`une_captation_alteree_ou_absente_se_nomme`, `.unwrap()`) ; un test du noyau renommé : 4 (`a_retirer_garde_un_fichier_illisible`) |
| E04 (décisions 4 et 8 du plan maître ; § 70 · 1 : les règles qui décident, au noyau) ; E37 (décisions 8 et 12 : les règles du démarrage au noyau) ; E48 (§ 70 · 1, la ligne « Le stockage des documents ») | 4, 9, 13, 15, 16 |
| E05 (décision 4 ; § 70 · 1 : un module de règle entre sous mutation dans sa PR, qui remesure à froid) ; E55 (critères de sortie : « `mutants` sans survivant, délais remesurés ») | 4, 9, 13, 15 (modules) ; 8, 12, 17 (remesures) |
| E06 (Contraintes générales ; § 70 · 3, I30 : une seule fonction de dossier jetable par crate) ; E10 (plan maître, tranche 1, première puce : les aides de tests) ; E50 (§ 70 · 3, I30, la part de la tranche) | 1 |
| E07 (Contraintes générales : aucune donnée réelle ; § 40 · 9 : la genèse et les migrations en octets dans un fichier Rust) | 2, 13 (la migration engendrée), contraintes |
| E08 (Contraintes générales : le glossaire, LE-2 et LE-3) | 8, 12, 17 |
| E59 (plan maître, tranche 1, « Documents » ; « Ce qui doit survivre au plan ») | 2, 8, 12, 17 |
| E11 (plan maître, tranche 1, dernière puce : la porte `contenu` ; § 40 · 9) ; E12 (même puce : le `.gitignore`) ; E13 (plan maître, tranche 1, « Documents » : `CONVENTIONS.md`, `contenu`) ; E56 (critères de sortie : un `*.automerge` indexé à l'essai refusé) | 2 |
| E53 (§ 70 · 8 ; décision 14 : aucune porte nouvelle) | 2 ; 3 (une étape, pas un job) |
| E14 (plan maître, tranche 1, deuxième puce : le point d'injection `pannes` ; décision 7) | 3 |
| E15 (§ 31 · 2.2, point 2 ; LE-10 : une compaction infidèle n'écrit rien) ; E57 (critères de sortie ; LE-10 clos) | 7, 8 |
| E16 (§ 31 · 2.2, point 3 : le fichier complet relu rend les octets écrits) | 3 (la relecture avant le renommage ; `un_voisin_relu_autre_ne_prend_pas_la_place`, sous `alterer_la_relecture`), 7 (`une_compaction_dont_le_complet_se_relit_autre_ne_retire_rien`, et le test 9, étape `Relecture`, qui n'éprouve qu'un refus du disque) |
| E17 (AM-4 (1) : la garde d'un seul `Document` du fonds) | 5 |
| E18 (AM-4 (2) ; § 31 · 2.1 : relu du disque, acteur neuf) ; E19 (décision 12 : une relecture qui échoue, lecture seule) ; E22 (décision 12 : une captation se verse toujours ; § 31 · 2.4) ; E24 (AM-4 (5) ; décision 12 : « faite, avec un avertissement ») ; E25 (plan maître, tranche 1, quatrième puce : `rendre_durable` refusé après le renommage) ; E26 (même puce : une réception refusée en chemin ; décisions 7 et 8 d'H07) ; E27 (plan maître, tranche 1, huitième puce : le fichier reçu comparé à `save_after`) ; E28 (même puce : le doctest `compile_fail` ; § 31 · 10, test 8) | 6 ; la captation versée devant une version de schéma plus récente, aussi en 13 |
| E20 (décision 12 : les fichiers illisibles ouvrent en lecture seule ; § 31 · 2.1 ; AM-4 (3)) ; E21 (AM-4 (6), première moitié : le fichier complet illisible se nomme) ; E23 (décision 12 : la compaction d'ouverture en échec avertit ; AM-4 (6), seconde moitié) | 7 ; le fichier d'ajout que `load_incremental` refuse, aussi en 7 (`AjoutIllisible`) |
| E29 (plan maître, tranche 1, huitième puce : chaque variante d'`Erreur` jamais provoquée a son test, ou disparaît) | 6 (`Genese` retirée, `Synchro`), 7 (`ChampAbsent`, `CompactionInfidele`) |
| E30 (§ 31 · 10, test 4, « couper l'écriture à chaque étape » ; § 31 · 6.1) ; E49 (§ 70 · 3, I14 : les captations sont immuables) | 3, 12 ; les tests d'H07 marqués en 17 |
| E31 (§ 31 · 2.1 : rien avant le verrou) ; E32 (§ 31 · 2.1 : le voisin conforme d'un fichier d'ajout prend sa place ; AM-4 (4)) ; E33 (§ 31 · 2.1 et 2.2, point 1 : le voisin du complet lu, même à la naissance, puis fondu) ; E34 (§ 31 · 2.1 : tout autre `.partiel` de `documents/` retiré) | 11 ; un voisin d'ajout qui ne se met pas en place, aussi en 11 (`VoisinRestant`) |
| E35 (décision 12 : les noms provisoires de `captations/` rangés, partout ; § 31 · 6.1) | 9, 12 |
| E36 (§ 31 · 2.1 : le suffixe `-2`, `-3` ; décision 8 d'H07 ; AM-4 (4)) | 9, 10, 11 |
| E38 (décision 12 : le § 31 · 6.1 corrigé) | 12 |
| E39 (plan maître, tranche 1, sixième puce : la règle de la version de schéma au noyau) ; E40 (même puce : les migrations, la migration de test ; § 31 · 2.4) ; E41 (§ 31 · 2.4 : une migration ne touche que la racine du document) ; E42 (§ 31 · 2.4 ; décision 12 : devant une version plus récente, lire sans écrire, recevoir) ; E51 (§ 70 · 3, I28 : le message de la migration) | 13, 14 |
| E43 (plan maître, tranche 1, septième puce ; décision 8 ; § 31 · 2.6 : la teneur au noyau) ; E44 (§ 31 · 2.6 : la teneur laisse l'histoire de côté, lignée comprise) | 15, 17 |
| E45 (plan maître, tranche 1, septième puce : le test 11 ; décision 16) | 17 |
| E46 (plan maître, tranche 1, septième puce : la règle de la lignée ; § 31 · 2.5) | 16 ; le cas de `synchro` reporté au lot 4 (C7) |
| E47 (critères de sortie : les tests 4 et 8 à 11, clause par clause) ; E54 (critères de sortie : ces tests entiers, verts dans `verifier` et `windows`) | le paragraphe « Les clauses des tests du § 31 · 10 », sous ce tableau ; E54, aussi 17 (fin de la tranche) |
| E58 (critères de sortie ; AM-4 clos, son point 7 reporté par la décision 3 du plan maître) | 17 ; LE-12, que cette PR consigne, clos aussi en 17 |

Les clauses des tests du § 31 · 10 : test 4, tâches 3, 9, 12, et les tests d'H07 que la tâche 17 marque ; test 8,
tâches 3, 6, 11, et deux tests d'H07 marqués en 17 ; test 9, tâches 7, 11, 15 ; test 10, tâches 13, 14 ; test 11,
tâches 15, 17. Ce qui part ailleurs est dit à C5.

**Les noms et les types, d'une tâche à l'autre :** `Etape`, `couper`, `Panne` (3) servent en 6, 7, 10, 11, 12,
13 ; `alterer_la_relecture` (3) en 7 ; `point` et `relue` existent sans la fonction `pannes`, que l'étape « Lint du
stockage, sans pannes » (3) compile ; `Echec` (3) en 6 ; `lire_le_dossier` (6) en 7, 11 et 13, sans garde ni
`Document`, qui rend dès 11 le nombre de voisins du fichier complet lus, pour `pupitre_neuf` et
`a_l_ouverture(…, voisins lus > 0)` ; `Echange::reprendre` (6) dans `recevoir` ; `Acces` (4) en 6, 7, 13, 14, son
`PlusRecent` né en 13 ; `a_l_ouverture(acces, ajouts, voisin_du_complet)` et `apres_une_ecriture(acces, due, …)`
(4) en 6, 7, 11, 13 ; `Lu::lisible` dès 4 (l'ancien `conforme`), étendu au chargement par Automerge en 7 et au voisin du complet en 11, comme
`Partiel::DansLeDossier { lisible }` (9) ; `Provisoire { deja_rangee }` (9) en 12 ; `LectureSeule` né en 6
(`RelectureManquee`), étendu en 7 (chacune de ses quatre raisons de lecture porte un `chemin` : `AjoutAltere`,
`AjoutIllisible`, `CompletIllisible`, `DependancesManquantes`) et 13 ; `Avertissement` né en 6, étendu en 11
(`VoisinRestant`) et en 12 (`NomProvisoireRestant`) ; `ecrire` et `recevoir` rendent
`Result<Option<Avertissement>, Erreur>` dès 6, `commun::ecrire` de même ; `avertissements()` en 6 (et
l'avertissement d'une réception refusée), 7, 11, 12, 13 ; `voisin` rend `io::Result<PathBuf>` dès 10 ; `Lu::nom`
relatif dès 11 ; `fonds_selon`, sous `pannes`, et `Migration<'a>` dès 13 ; `teneur` dès 15, que 17 appelle ;
`LIGNEES_REMPLACEES` né en 15, que 16 et 17 emploient.

**Les cas d'erreur décidés par ce plan** (relecture de la PR #27), chacun dans la tâche qui possède le code, au
plus près d'AM-4 (6) et de la décision 12 : ouvrir, nommer, avertir, plutôt que refuser. Un voisin d'ajout que le
disque refuse de mettre en place se lit où il est, avec `VoisinRestant` (11) ; un fichier d'ajout dont l'empreinte
est juste mais qu'Automerge refuse ouvre en lecture seule, `AjoutIllisible` (7) ; une réception refusée dont le
fichier d'ajout avertit rend son refus, et l'avertissement rejoint `avertissements()` (6) ; un nom provisoire qui
ne se lit pas reste, avec `NomProvisoireRestant` (12). Chacun a son test.

**La proportion :** dix-sept tâches pour une tranche qui touche une crate d'adaptateurs, cinq modules du noyau et
deux fonctions de `synchro`. Les corps n'y sont écrits que pour la forme de la teneur, le doctest et le contrôle
de la porte ; le reste donne signatures, valeurs et assertions.

**La relecture** (trois relecteurs, le 2026-11-09 : couverture, exécution, faisabilité) a levé, entre autres :
un échange qui ne reprenait pas sur les mêmes `Echange` après une réception refusée (protocole d'automerge
0.12.0, `sent_hashes`), d'où `Echange::reprendre` ; une comparaison de la relecture qu'aucun test n'exerçait,
d'où `alterer_la_relecture` ; la configuration livrée de `stockage`, jamais compilée en CI, d'où l'étape sans
pannes ; une recherche du voisin libre sans borne, dont les mutants auraient bouclé ; des tests sans panne
(migration, naissance, rangement en lecture seule), des attendus de CI faux, des aides mal citées ; et les trois
écarts au plan maître, que la maîtrise d'ouvrage a tranchés le 2026-11-09, chacun selon son défaut.

**La relecture de la PR #27** (une session neuve, le 2026-11-09) a levé : LE-12, que rien ne close, d'où la
tâche 17 ; des renvois à des étiquettes de travail qu'on ne lit pas dans le dépôt, remplacés par leurs sources ;
trois mots qui doublaient un sens (`Lu::lisible`, `deja_rangee`, le verbe du titre de la PR (c)) ; l'ordre de la
migration et de la compaction d'ouverture ; quatre cas d'erreur sans réponse, décidés plus haut ; la doc d'H07 qui
dit encore les refus d'ouverture ; des caractérisations non relevées ; le repli de la tâche 14 côté `stockage` ;
la remesure à froid sans repli de cache ; les PR en brouillon ; les tests d'H07 sans marqueur ; l'entrée du
glossaire sans ses autres sens ; la question C2 dite encore ouverte ; le compte des voisins lus ; `fonds_selon`
dans l'API livrée.
