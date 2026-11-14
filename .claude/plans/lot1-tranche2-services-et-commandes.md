# La tranche 2 du lot 1 d'INC-1 : les services, les commandes, les opérations nommées — plan d'implémentation

> **Pour les agents :** exécuter ce plan avec `superpowers:subagent-driven-development`, tâche par tâche ;
> les étapes sont des cases à cocher (`- [ ]`).

**Objectif :** faire naître la seule porte d'écriture du fonds. Chaque geste devient une opération nommée,
décidée au noyau, conduite par `services`, appliquée par l'adaptateur `Documents` ; une même suite de tests des
commandes passe sur la paire d'adaptateurs en mémoire et sur celle des fichiers, en natif.

**Architecture :** deux crates naissent. `commandes` porte les types que l'interface enverra et recevra, et la
liste des noms des commandes ; `services` porte les contrats `Documents`, `Captations` et `Alea`, le seul code
qui applique des changements à un document Automerge (`application`), les lectures du fonds (`lecture`), la paire
en mémoire (sous la fonction de Cargo `en-memoire`) et `Services`, qui conduit chaque commande : lire le fonds,
demander au noyau ses changements et son message, les faire appliquer. Le noyau gagne les identifiants, le type
des changements, les gestes (préconditions, changements, inverse, message) et les règles d'une idée, sous
mutation. `stockage` réalise les contrats : son API livrée applique des opérations, et ses fermetures d'écriture
ne restent que comme points d'entrée des tests, sous `pannes`.

**Pile :** Rust 1.99.1, édition 2024 ; `automerge` 0.12.0 ; `sha2` 0.11.1 ; nouvelles : `serde` 1.0.229 (fonction
`derive`, déjà au `Cargo.lock`), `serde_json` 1.0.154 (dépendance de test de `commandes`, déjà au `Cargo.lock`),
`ulid` 3.0.0 (sans ses fonctions par défaut ; une entrée de plus au `Cargo.lock`) ; tests : `proptest` 1.12.0 ; en
CI, `cargo-mutants` 27.1.0.

**Spécification :** le plan maître, `.claude/plans/lot1-le-fonds-sur-l-etabli.md`, section « Tranche 2 · les
services, les commandes, les opérations nommées », ses décisions 7 à 13, 17, 18 et 20 ; ce qu'il cite : CAP-01,
CAP-02 ; § 30 · 2, 8.1, 9, 11 et 13 ; § 31 · 2, 2.3, 2.4, 2.6, 3 et 10 (tests 3, 4, 7, 8, 10, 11, 13) ; § 40 · 3,
7.5 et 9 ; § 41 · 6 ; § 50 · 5 ; § 70 · 1, 2, 3 (I01, I10, I28) et 4.1 ; ADR-006, ADR-022, ADR-023, ADR-024 ; et le
plan de la tranche 1, « Les écarts au plan maître », point 1.

Refs #24

---

## Contraintes générales

Recopiées du plan maître (« Contraintes générales », « Le déroulé »), comme l'a fait la tranche 1 ; chaque tâche
les suppose.

- « **En CI seulement.** Ni cargo, ni trunk, ni npm sur le poste : chaque vérification de Rust se lit dans le job que nomme le critère, le rouge puis le vert dans `verifier`, le vert aussi dans `windows` si le critère l'y met. »
- « Seul `rustfmt --edition 2024` formate les `.rs` touchés ; les vérifications de documentation tournent sur le poste, après le `git add` (LE-7). » `rustfmt` se lance hors du dépôt, sur une copie dans le dossier temporaire, recopiée ensuite.
- « Les commandes de Rust prennent `-p <crate>`, et `ci.yml` nomme chaque crate qu'il passe à `clippy` et aux tests. »
- « **Le noyau** reste sans entrée-sortie, ni `automerge`, ni horloge, ni aléa ; chaque règle qui décide y vit, sous mutation (décision 4). »
- « **Les chemins se reçoivent** (I30) : hors de `systeme`, aucun code ne devine un chemin ; chaque test prend son dossier jetable par une seule fonction de sa crate. »
- « **Aucune donnée réelle** : textes et noms inventés ; identifiants, documents et captations engendrés à l'exécution ; aucun audio, MIDI ni `.automerge` suivi. »
- « **Les mots du glossaire**, dits en entier : la racine des données et la racine du document, la version de schéma, l'origine déclarée, l'identifiant de propriétaire, le manifeste de l'application. »
- « Une PR qui ajoute un mot au glossaire le cherche d'abord (`git grep -n -i -w <mot> -- docs README.md CONVENTIONS.md`), requalifie chaque emploi antérieur et cite la recherche (LE-2) ; une PR qui périme un mot ou un identifiant le cherche dans tout le dépôt, maquettes et commentaires SVG compris (LE-3). »
- « **Chaque écran** qu'une tranche ajoute se mène au clavier, focus visible […] » : la tranche 2 n'en ajoute aucun.
- « Conventional Commits en français, en-tête de 100 caractères au plus, titre de 93 au plus ; « Refs #24 » » ; `git add` par chemins explicites, jamais `git add .` ; `git commit -F <scratchpad>/msg-t2-<tâche>-<étape>.txt` (un texte accentué passe par un fichier).
- « Chaque PR qui livre un critère de la SFD en entier l'inscrit au tableau du § 70 · 4.1, et un test le nomme (`cap06_7_…`) et en calcule l'exemple ; `verif-faits.mjs` refuse un critère inscrit sans test qui le nomme. La porte ne voit pas un critère livré que la PR n'inscrit pas : la revue de chaque PR compare le tableau aux critères que la tranche dit livrer. » La tranche inscrit CAP-01 · 6 en PR (a), CAP-02 · 7 et 8 en PR (b) ; CAP-02 · 3, 4, 5 et 6 ont leurs parts, sans inscription (§ 70 · 4.1 : « Un critère qui arrive par parts a un test nommé pour chaque part, et ne s'inscrit qu'avec la dernière »). La règle vaut pour chaque critère : relu dans la SFD, un critère qui vise un geste ou un cas qu'une tranche ou un lot suivant livre (relier à un motif ou à un morceau, regrouper des photos…) garde son test de part, nommé (`cap02_6_…`), sans inscription ; un critère inscrit l'est parce que la tâche qui le livre dit, en une phrase, pourquoi il est entier.
- « fusion par squash, une fois tout vert » ; mode `session` : la PR s'est ouverte en brouillon, donc `gh pr ready <n>`, puis, depuis le clone principal `pupitre`, sans `cd` ni `-R`, `gh pr merge <n> --squash --match-head-commit <SHA complet de la tête relue>`.
- « une tranche qui alourdit les tests remesure les délais de `verifier` et de `windows` ; l'installateur NSIS reste en artefact. »
- Le corps de chaque PR porte la section `## Écarts à la spec`, avec « Aucun. » : un écart à la spec se lève en corrigeant la spec dans la PR même, jamais en le déclarant. Le corps final s'écrit avant l'envoi qui le rend final (une retouche du corps ne relance que `texte-pr`, et `texte-pr` ne se relance jamais par `gh run rerun`).
- Chaque PR part de `main` à jour, depuis le clone principal (`git fetch`, puis `git worktree add ../pupitre.worktrees/<branche> -b <branche> origin/main`), s'ouvre en brouillon au premier envoi (`ci.yml` ne tourne que sur une PR), et dit dans sa description les tests réécrits et pourquoi.
- **Lire la CI** : `gh run list --branch <branche> --workflow ci --limit 1`, `gh run watch <n> --exit-status`, `gh run view <n> --log-failed`. Un rouge attendu se lit à l'étape « Lint » de `verifier` : toute erreur de compilation qui nomme l'élément manquant (`unresolved import`, `cannot find`, E0599, E0425) ; ou à « Tests » (`panicked at`, avec le nom du test). Un rouge à « Format » se corrige (`style: le format de rustfmt`) avant de relire. Un vert se lit à « Tests » de `verifier`, et, quand `stockage` ou `synchro` change, à « Tests du stockage et de la synchronisation (H07) » de `windows` : `test result: ok.` pour chaque fichier touché.
- Chaque nom d'`automerge` 0.12.0 que le code emploie se vérifie sur docs.rs avant d'écrire ; un commentaire dans un bloc de test de ce plan est une spécification, qui devient des assertions.
- Chaque test qui prouve un des tests 3, 4, 7, 8, 10, 11 ou 13 du § 31 · 10 porte au-dessus de lui `// § 31 · 10, test N : <la clause>`, la clause recopiée mot pour mot ; chaque test qui prouve un critère de la SFD, inscrit ou livré par parts, se nomme `capNN_n_…` (`cap02_7_…` pour un critère inscrit, `cap02_3_…` pour une part), précédé de `#[test]`.
- Les heures sont des secondes, partout où le code les reçoit (l'heure d'une modification Automerge, `le`, `retiree`) ; un ULID les prend en millisecondes, par `heure × 1 000`. L'heure et l'aléa sont des arguments jusqu'à la tranche 3 ; les tests les donnent.
- Un numéro de ligne qu'une tâche cite vaut sur `main` à `4d6fd2d` : une tâche antérieure du plan peut le décaler,
  et l'élément nommé (la fonction, l'import, la doc) fait foi ; l'implémenteur le cherche par son nom (balayage
  préalable, 2026-11-14).
- Un rouge attendu se lit tel que la CI le rend : une erreur de compilation qui nomme ce qui manque, quel que soit
  le premier nom qu'elle donne. `cargo test` s'arrête au premier fichier de tests rouge, pris dans l'ordre de leurs
  noms (`demarrage.rs` passe avant `genese.rs`) : un rouge attendu dans un fichier ne se lit que si aucun fichier
  qui le précède ne rougit.

---

## Ce que les passages ont levé

Les deux passages (AM-2, LE-6), faits avant ce plan, ne sont pas dans le dépôt : leur détail a servi à écrire ce
plan ; la description de sa PR porte la table de couverture. La relecture du plan par une session neuve, après eux,
a sa table à la fin de cette section. La PR de ce plan consigne (`/consigner`), au motif
« Une spec qui se contredit » (cinquième constat ; la porte existe depuis LE-11), une ligne, LE-14, qui nomme C3, C4,
C12, C17, C22, C23, C24, C30, les cinq sens doubles (C5, C6, C8, C9, C10), et les contradictions que le plan tranche
par une question ou une correction : le test 13 (Éc-1 à Éc-3), « une seule idée visible » (Éc-8), la date d'un lien
au § 30 · 1 (Éc-9) et l'exemple d'annulation du § 30 · 11 (Éc-11), avec les points hors tranche ; la tâche 19 la
clôt.

### Le passage d'AM-2

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| C1. Les critères lisent le fonds, et aucune lecture n'est à la tranche | Trois lectures sans tri, filtre, compte ni recherche : `vue_d_une_idee`, `idees` (les idées non retirées) et `journal` (les messages de l'historique) ; jamais « requête », jamais « vue du fonds » ; écart 1 | 5, 11 |
| C2. Rien ne crée une prise, un MIDI versé ou un texte versé avant l'import ; le générateur doit être livré | Les changements qui versent une fiche au noyau (`gestes::verser`), conduits par `Services::verser_une_fiche`, livrée hors d'`en-memoire`, sans commande avant la tranche 5 ; écart 2 | 10, 11, 18 |
| C3. La vigilance 1 sur les prises | Les faits d'une captation en valeurs simples sous `<empreinte>/<fait>`, sans objet par empreinte : d'une prise, `idee`, `format` et `duree` ; d'un MIDI versé ou d'un texte versé, `idee` et `format`. La date et l'appareil d'une prise sont des faits de sa captation, « lus une fois, sur la captation ou à sa création » (§ 30 · 9.2) : à INC-1, toute prise est importée, et rien ne les lit sur la captation (CAP-03, l'établi ne lit que l'en-tête d'un WAV ou d'un MIDI ; ADR-024, décision 6) ; ils restent absents, `—`, et n'ont pas de clé ; l'heure du versement et l'appareil qui verse sont ceux de l'idée qui naît (`le`, `par`), jamais ceux de la captation. `format` prend le vocabulaire du § 30 · 9.2 et de l'ADR-024, jamais l'extension (tâche 10). Deux répliques qui versent la même captation posent chacune chaque fait : sous chaque clé, `get_all` rend deux valeurs, qu'Automerge ne fusionne pas, égales pour `format` et `duree`, différentes au besoin pour `idee` ; rien ne se perd, et le relevé des conflits vient au lot 3 ; une clé se coupe au premier `/` ; les empreintes nommées se tirent au noyau | 8, 10, 15 |
| C4. La décision 9 énumère moins de conteneurs que sa règle n'en demande | Aucun conteneur de plus : chaque fait de la tranche 5 a déjà sa clé dans un conteneur de la décision 9, que le § 31 · 2 écrit (stems sous les prises, `<prise>/stem/<stem>/<fait>` ; noms d'origine, ensemble `<e>/nom/<nom>` → `true` sous la sorte de leur contenu ; manifeste, `<prise>/manifeste` ; rapport, `<import>/rapport` sous les imports ; la division d'un MIDI et les autres faits d'un en-tête, sous des clés `<e>/<fait>` ajoutées, ce qui ne demande aucune migration) ; l'écart É3 du passage n'est pas pris | 8, 13 |
| C5. « Vue du fonds », deux sens | Le noyau lit « le fonds lu » (`IdeeLue`, `CaptationLue`) ; « vue » seul pour ce que rend une lecture ; « vue du fonds » reste à CAP-07 | 2, 4 |
| C6. « Écritures » du noyau contre « écriture » du disque | « Changements » (`Changement`, le mot du § 31 · 3) pour ce que rend le noyau, « écriture » pour le disque ; `Simple` n'a ni `Counter` ni objet `Text` | 2 |
| C7. Ce qu'est une commande en tranche 2 | Une méthode de `Services`, qui prend un argument de `commandes` et rend une `Reponse` de `commandes`, avec à part le contexte du côté natif (`Contexte` : l'heure, l'aléa ; l'appareil, à la construction) ; les types de `commandes` ne portent que ce que l'interface envoie, dont `decalage_minutes` | 3, 5 |
| C8. « Requête », deux sens | « Lecture » et « vue », jamais « requête » ; aucune entrée « Requête » au glossaire en tranche 2 | 5, 11 |
| C9. « Étiquette », deux sens | Entrée « Étiquette » à deux sens (d'une idée ; de git, `vX.Y.Z`), après la recherche de LE-2 ; l'étiquette se range sans `#`, que le message ajoute | 10, 13 |
| C10. « Annuler », deux sens ; « la place » | Entrée « Annuler » à deux sens ; « la place » n'entre ni au code ni aux documents : le code dit `dernier_geste`, le texte « le dernier geste de tri » | 12, 13 |
| C11. « La racine des données sans document » sur la paire en mémoire | `pupitre_neuf` reste la seule règle de « neuf » sur le disque, que l'adaptateur sur fichiers applique ; `services` ne connaît que l'absence de fonds (`Documents::fonds` rend `None`) ; la suite dit « aucun document » | 5, 6 |
| C12. Un geste sans effet | Refusé, nommé par un constat (`Impossible`), sans prendre la place de l'annulation ; question 1 | 10, 12 |
| C13. Annuler : les valeurs d'avant, la place, le message | Chaque geste de tri rend son inverse, calculé sur le fonds lu avant le geste (`GesteDeTri::inverse`) ; `Services` garde le dernier ; une annulation refusée le garde, un geste refusé ne le prend pas, la réouverture le vide ; la table des messages à la tâche 10 | 10, 12 |
| C14. La version de schéma N + 1 sur la paire en mémoire | `DocumentsEnMemoire::de(octets, migrations)`, sous `en-memoire` ; la migration de test recopiée dans la suite ; les deux adaptateurs lisent `version::lecture` ; le refus nommé `Raison::VersionPlusRecente { version }` ; une captation s'y verse | 4, 5, 6 |
| C15. Qui conduit la migration | Chaque adaptateur, à son ouverture, le noyau décidant (`lecture`, `a_appliquer`), par le code partagé `services::application::migrer` ; `le_fonds_nait_a_la_version_de_schema_connue` reste la porte | 4, 6 |
| C16. Le doctest « pendant l'écriture », sans fermeture | Sur `Document::appliquer` : une lecture tenue (`let lu = document.automerge();`) puis `appliquer` puis `echange.prochain_message(lu)` ne compile pas, `compile_fail,E0502` | 6 |
| C17. Les fermetures publiques de l'écriture et de la naissance | L'API livrée applique des opérations (`Document::appliquer`, `RacineDesDonnees::faire_naitre`) ; `Document::ecrire`, `faire_naitre_le_fonds` et `Naissance` passent sous `pannes`, points d'entrée des tests ; la naissance au noyau (`gestes::naissance`) ; aucun test de la tranche 1 ne se réécrit | 6 |
| C18. Où vit la suite générique | Un fichier partagé par `#[path]` : `crates/services/tests/suite/mod.rs`, que `crates/stockage/tests/commandes_sur_fichiers.rs` inclut ; chaque test nommé d'après un critère s'écrit dans chaque crate | 5, 6 |
| C19. Les preuves à plusieurs répliques ne passent pas sur la paire en mémoire | Sur la paire de fichiers seule, dans `verifier` et `windows` ; écart 4 | 13, 15 |
| C20. Le partage des PR | L'écart 5 | — |
| C21. Les documents que la liste omet | Chaque document à sa PR (carte des fichiers) ; écart 6 | 7, 13, 19 |
| C22. Les corrections (a) et (c) | Écrites comme le passage les lève (tâche 7) | 7 |
| C23. « Jamais un ordre » ; le texte des erreurs étrangères | Un constat ; le texte d'une erreur étrangère n'est jamais recopié ; le § 31 · 2.4 et CAP-01 se récrivent en (c) | 16, 19 |
| C24. L'identifiant de propriétaire, l'aléa des acteurs, l'unicité des ULID | `identifiant_de_proprietaire`, fonction privée de `services` qui ne fait que cela, seul appel d'`uuid_v4` (contrôle statique) ; l'exception de l'aléa nomme l'acteur d'Automerge où qu'il soit tiré ; chaque réplique d'un test a son aléa | 5, 15, 17 |
| C25. La première mesure | Sur la paire en mémoire, sauvée en fichier complet, rouverte par `fonds()`, en `--release` ; aucune captation, donc aucun rangement | 18 |
| C26. Le témoin du test 11, écrit par des gestes de test | Il naît par les opérations, à heure et aléa fixés, et porte une valeur dans chaque conteneur que la tranche écrit | 13 |
| C27. `commandes` en WebAssembly ; la forme de la liste des noms | L'interface dépend de `commandes` dès la PR (a) ; `NOMS: &[&str]` | 3, 6 |
| C28. Deux regroupements concurrents | Un test les écrit, une piste au lot 3 | 15, 19 |
| C29. Les règles au noyau ne tournent que dans `verifier` | Chaque test de critère vit dans la suite, sur les deux paires | 5, 11, 12 |
| C30. Les sorties des tranches 2 et 3 se recouvrent | Chaque critère relu dans la SFD (§ 70 · 4.1 ; la leçon d'AM-2 pour REN-08). Inscrits : CAP-01 · 6 en (a), son geste, noter à l'établi, étant la commande `noter`, livrée en entier, et chacun de ses faits se lisant dans la vue dès noter (aucune prise ne s'écrit en (a)) ; CAP-02 · 7 en (b), son cas, deux idées reliées en sens contraires puis la fusion des deux répliques, étant livré en entier (un lien vers un motif ou un morceau n'est pas son cas) ; CAP-02 · 8 en (b), son cas, une idée qui n'a qu'une prise, regroupée puis le geste annulé, se calculant en entier sur les gestes de la tranche. Des parts, sans inscription : CAP-02 · 4 et 6 visent chaque geste de tri, dont relier à un motif ou un morceau (INC-2) et regrouper une photo (lot 2), que la tranche ne journalise ni n'annule : leurs tests de part, `cap02_4_…` et `cap02_6_…` ; CAP-02 · 3 et 5, la boîte et les requêtes (tranches 3 et 4), et les photos de CAP-02 · 5 (lot 2) | 5, 7, 11, 12, 13 |
| C31. Le contrôle statique d'I28 | « Écrire » : créer une modification ; le contrôle lit les `src/` ; trois fichiers permis ; `receive_sync_message` cité comme reprise | 17 |

### Le passage de LE-6

| Écart | Décision de ce plan | Tâche |
|---|---|---|
| Éc-1. 2 000 idées pour 6 000 prises | 2 000 idées visibles et 4 000 retirées par les regroupements ; question 3 | 18 |
| Éc-2. « Cinq opérations par idée » | Sept par idée visible : trois textes complétés, trois étiquettes, et un lien pour deux idées (1 000 liens) ; question 3 | 18 |
| Éc-3. « 4,5 Mo » sans unité | 4 500 000 octets ; question 3 | 18 |
| Éc-4. Une borne basse | La PR (c) le dit, avec ce qui manque (noms d'origine, stems, origines déclarées, acteurs) | 18, 20 |
| Éc-5. Un geste sans effet n'écrit rien | Refusé (C12) | 10 |
| Éc-6. L'inverse repose les valeurs d'avant | Calculé sur le fonds lu avant le geste ; une clé en conflit avant le geste revient à sa seule valeur présentée, ce que le § 30 · 11 dit | 10, 12, 13 |
| Éc-7. Le retrait d'office sur la vue locale | Le test des regroupements croisés ; la piste | 15, 19 |
| Éc-8. « Une seule idée visible » | « La prise ne paraît que dans une idée » ; écart 10 | 15 |
| Éc-9. « N'en font qu'un » ; la date d'un lien | Le lien vaut `true`, sans date : la PR (b) corrige le § 30 · 1, dont la classe `Lien` porte `le`, la date d'un lien se lisant dans l'historique du fonds ; « n'en font qu'un » se lit sur les clés, et `get_all` rend deux valeurs `true` ; la piste du lot 3 : deux valeurs égales ne sont pas un conflit « à trancher » | 13, 15, 19 |
| Éc-10. Délier depuis l'autre bout d'une idée retirée | Tout geste qui touche une idée retirée est refusé, sauf rétablir ; question 6 (l'ADR-006 n'admet que l'impossibilité stricte) | 10 |
| Éc-11. Les formes du message | La table de la tâche 10 : le décalage du geste, sans année, « 1er », « Annulé : » suivi du message entier ; questions 2 et 4 | 2, 10, 12 |
| Éc-12. « Jamais un ordre » contre le refus N + 1 | Un constat | 16 |
| Éc-13. Le texte d'une `io::Error` | Une table par `io::ErrorKind`, qui nomme le fichier | 16 |
| Éc-14. `JSON.parse` et les entiers | Aucun flottant ; les entiers sous 2⁵³ | 3 |
| Éc-15. « Décidés » égale « portant » | `Assiette`, « décidés » écrit une fois ; ses mutants tués par un exemple où considérés et portant diffèrent | 9 |
| Éc-16. Seize octets pour un ULID | Dix ; le § 41 · 6 le dit | 1, 7 |
| Éc-17. « Sans horloge ni aléa » et `web-time` | Le § 41 · 6 le dit ; `cargo tree` ne refuse qu'`automerge`, `services` et `stockage` | 7, 17 |
| Éc-18. « Ni `Counter` ni objet `Text` » et le témoin des types | Le test vise les documents des opérations, jamais le témoin des types | 14 |
| Éc-19. Des critères dans deux tranches | C30 | 13 |
| Éc-20. Trois répliques dans un ordre aléatoire | Réunions tirées, puis une réunion finale A–B, B–C, A–B | 15 |
| Éc-21. « La même étiquette » | Octet pour octet, sans normalisation ; le § 31 · 2 le dit ; question 7 (le choix contraire ne se rattraperait pas sans migration après la `v0.3.0`) | 10, 13 |
| Éc-22. Une note au texte vide | Refusée ; question 1 | 2 |
| Éc-23. La décision 16 | Hors tranche 2 (tranche 9) | — |
| Éc-24. « 40 Go » | Hors tranche 2 (tranches 5, 6) | — |

Les tensions que le relevé des exigences a trouvées, et leur sort :

- T1, T2, T3 (CAP-02 · 3, 4, 5 et CAP-01 · 6 dans deux tranches, les photos de CAP-02 · 5) → C30 ;
- T4 (rien ne crée une captation avant l'import) → C2, écart 2 ;
- T5 (la liste des commandes, que le § 40 · 7.5 met dans les droits) → la PR (a) le dit, et la tranche 3 corrige (f) ;
- T6 (« il faut mettre à jour ») → C23 ;
- T7 et T10 (l'aléa des acteurs, les unités) → C24, et la dernière contrainte générale ;
- T8 (la date d'un lien) → Éc-9 ;
- T9 (les conteneurs de la tranche 5) → C4 ;
- T11 (les empreintes nommées au noyau) → C3, tâche 8 ;
- T12 (I28 et `synchro`) → C31 ;
- T13 (le doctest) → C16, C17 ;
- T14, T15 (« une seule idée visible », « intactes ») → Éc-8 ; `get_all` aussitôt après la réunion ;
- T16 (l'assiette) → une seule règle, `idee::assiette` ;
- T17, T18 (les opérations du test 13, les métadonnées des prises) → Éc-2 ; en tranche 2, les prises portent idée, format et durée ; leur date et leur appareil, des faits de la captation que rien ne lit encore sur elle, restent absents, `—` (§ 30 · 9.2 ; CAP-03 ; ADR-024, décision 6) ; les MIDI versés et les textes versés, idée et format (§ 30 · 9.2), `format` au vocabulaire du § 30 · 9.2 et de l'ADR-024, jamais l'extension (C3).

### La relecture par une session neuve

Une session neuve a relu ce plan selon « La revue du plan » (`docs/modele/methode.md`), chaque mécanisme annoncé relu
dans le code de `a36eef2` : dix-sept constats, tous retenus. Chacun a été cherché aussi ailleurs dans le plan, et la
correction suit partout où la même croyance paraissait.

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| N1. Bloquant : une prise importée portait sous `le` et `par` l'heure et l'appareil du versement, des faits faux de sa captation (§ 30 · 9.2 ; CAP-03 ; ADR-024, décision 6) | Une prise n'écrit que `idee`, `format` et `duree` ; sa date et son appareil restent `—`, sans clé (C3) ; la liste blanche, le § 31 · 2, le témoin et sa teneur suivent | 2, 10, 11, 13, 14, 15 |
| N2. Majeur : CAP-02 · 6 s'inscrivait, contre le § 70 · 4.1 que le plan applique à CAP-02 · 4 | Chaque critère relu dans la SFD (C30) : CAP-02 · 6, une part (`cap02_6_…`) ; CAP-01 · 6, CAP-02 · 7 et 8 inscrits, chacun avec sa phrase | 5, 11, 12, 13, 20 |
| N3. Majeur : « n'a pas pu s'écrire » pour un refus du disque qui peut être une lecture | Un défaut neutre quant au geste, « S, le disque l'a refusé », éprouvé sur une lecture à l'ouverture (`ErrorKind::Other`) | 16 |
| N4. Le test de l'outil `ulid` n'exerçait pas l'heure au-delà de 2⁴⁸ ms | L'heure se borne à 2⁴⁸ − 1 ms, et `une_heure_au_dela_de_2_puissance_48_millisecondes_s_y_borne` l'éprouve | 1 |
| N5. Le test de l'outil `serde_json` ne vérifiait que des valeurs qu'il choisit | `verser` refuse une durée au-delà d'`ENTIER_SUR` (`DureeTropLongue`), éprouvé ; la source bornée de chaque autre entier d'une réponse, nommée | 3, 10 |
| N6. `cle_du_lien` et le mutant équivalent `<` → `<=` | `(a.min(b), a.max(b))` | 9 |
| N7. Éc-10, une règle de produit, sans question | Question 6, avec le renvoi à l'ADR-006 | 10 |
| N8. Éc-21, inscrit au schéma de la `v0.3.0`, sans question | Question 7 | 10, 13 |
| N9. `format` portait l'extension (`mid`, `txt`) | Le vocabulaire du § 30 · 9.2 et de l'ADR-024 : `wav`, `flac`… ; `smf0` à `smf2` ; `brut`, `markdown`, `chordpro` ; la division et les autres faits d'un en-tête en tranche 5, sous des clés ajoutées | 2, 10, 13 |
| N10. Les raisons d'un refus ne couvraient pas chaque variante de `stockage::Erreur` | Une raison par variante, et celles qui ne surviennent qu'avant `Services` ; `CaptationsEnMemoire::relire` d'une captation absente | 4, 6 |
| N11. La plage de la migration de test oubliait `CONTENEUR_DE_LA_MIGRATION` | `crates/stockage/tests/commun/mod.rs:264-303` | 4 |
| N12. Des imports qui ne servent plus que sous `pannes` | `#[cfg(feature = "pannes")]` sur l'import, ou le chemin complet : `Transaction`, `CommitOptions`, `Transactable` | 6 |
| N13. La seconde mention du nombre de modules de `.cargo/mutants.toml` | `:2` et `:22`, à chaque tâche qui ajoute un module | 1, 2, 9 |
| N14. « sans `features =` » contredisait la ligne exigée | La ligne entière, égale à l'octet, et aucune clé `features` (hors `default-features`), avec sa plantation | 17 |
| N15. Une vingtaine de textes attendus n'étaient pas écrits | Le texte de chaque variante d'`Erreur`, de `LectureSeule` et d'`Avertissement` | 16 |
| N16. Une seule plantation pour quatre contrôles | Une plantation par contrôle : une commande en deux modifications, un message hors table, un `ScalarValue::counter` et un objet `Text`, une durée sous une idée | 14 |
| N17. `message_connu` admettait tout texte après `Idée du <date> : ` | Les formes fermées de la table de la tâche 10 ; « Idée du 3 novembre : autre chose » refusé | 14 |

## Les écarts au plan maître

Le plan maître le veut (en tête) : un plan de tranche qui le contredit le dit dans sa PR, et la maîtrise d'ouvrage
relit ce point. **La PR de ce plan ne touche pas le plan maître** : sous la `v0.10.0` du modèle, l'étape « Les
plans » de la CI vérifie chaque plan de `.claude/plans/` qu'une PR modifie, et le plan maître, dont les titres sont
« ### Tranche N » et qui n'a pas les trois sections, serait refusé. Chaque point est donc dit ici, avec ce que la
maîtrise d'ouvrage décide et le défaut que ce plan suit. La description de la PR de ce plan (#34) a listé ces dix
écarts et les sept questions qui suivent ; la maîtrise d'ouvrage les a tranchés le 2026-11-12, chacun selon son
défaut (« Ok pour les défauts ») : chaque écart est retenu, et chaque question reçoit la réponse de son défaut.

1. **Trois lectures dès la tranche 2** (C1). Le plan maître met la boîte en tranche 3, les requêtes en tranche 4.
   À décider : la vue d'une idée, la liste des idées non retirées et le journal, sans tri, filtre ni compte, dès
   la tranche 2. Défaut : oui ; la tranche 3 bâtit la boîte dessus.
2. **Une opération livrée qui verse une fiche** (C2). Le plan maître n'en a aucune avant l'import. À décider :
   `Services::verser_une_fiche`, hors d'`en-memoire`, sans commande avant la tranche 5. Défaut : oui.
3. **CAP-02 · 7 et les preuves à plusieurs répliques sur la paire de fichiers seule** (C19), quand le plan maître
   dit « la suite verte sur les deux paires […] CAP-02 · […] 7 […] y passent ». Défaut : oui, verts dans
   `verifier` et `windows`.
4. **Le partage des PR** (C20). Le plan maître : (a) commandes, services, contrats, adaptateurs en mémoire,
   identifiants ; (b) schéma, genèse, gestes, annulation ; (c) preuves, messages, mesure. Ce plan : (a) prend
   aussi la réalisation des contrats par `stockage`, la naissance et noter, l'API qui applique et son doctest, la
   migration et le refus N + 1 ; I28 statique et l'étape `cargo tree` restent en (c). Défaut : oui.
5. **Des documents de plus** (C21, C23) : le § 30 · 11, CAP-02, le § 70 · 4.1, le glossaire, le registre, le § 31 ·
   2.4 et CAP-01, `docs/pistes.md`. Défaut : oui, chacun à sa PR.
6. **L'interface dépend de `commandes` dès la PR (a)** (C27), quand le plan maître la lui donne en tranche 3 : le
   lint en WebAssembly compile ainsi `commandes` dès sa naissance. Défaut : oui ; elle ne l'emploie qu'en tranche 3.
7. **La mesure en `--release`** (C25). Le plan maître ne dit rien du profil en tranche 2, et réserve `--release` à
   la tranche 5 « si elle passe dix minutes » ; quelque 23 000 opérations en debug, chacune lisant le fonds,
   dépasseraient les 15 minutes de `verifier`. Défaut : `--release` dès la tranche 2.
8. **`sha2` quitte le stockage**, quand la décision 7 fait dépendre `stockage` de « `noyau`, `synchro`, `services`,
   `automerge`, `sha2` ». L'empreinte d'une captation se calcule au noyau (`identifiants::empreinte`), que la paire
   en mémoire appelle aussi, et le stockage la réexporte sans employer `sha2` (tâche 1). Défaut : oui ; le § 40 · 3
   et le § 41 · 6 le disent en (a).
9. **Le fonds lu et l'arbre de la teneur, tirés par `services::lecture`**, quand la décision 8 dit « une vue du fonds
   que construit l'adaptateur `Documents` » et « un arbre de valeurs neutre que `stockage` tire du document ».
   L'adaptateur rend le document (`Documents::fonds()`), et `services::lecture` en tire, une fois pour les deux
   paires, le fonds lu (tâche 4) et la teneur (tâche 12). Défaut : oui ; la doc de `noyau::donnees::teneur` le dit
   en (b).
10. **Deux regroupements d'une même prise** (Éc-8), quand la tranche 2 du plan maître veut qu'ils « laissent une
    seule idée visible » : deux idées cibles qui portent autre chose restent visibles toutes deux, et la prise ne
    paraît que dans l'une, la même sur chaque copie. Défaut : cette lecture, que la tâche 15 éprouve.

Les questions posées avec ces écarts, chacune tranchée le 2026-11-12 selon son défaut :

1. **Un geste sans effet, une note au texte vide, un texte complété vide** (C12, Éc-5, Éc-22) : refusés, nommés par
   un constat, sans prendre la place de l'annulation ; vider le texte d'une idée n'est ainsi pas un geste. L'ADR-006
   l'admet-elle ? Défaut : oui, l'ADR ne change pas.
2. **L'année dans un message** (Éc-11) : « Idée du 3 novembre », sans année, comme le § 30 · 11. Défaut : sans année.
3. **Le test 13** (Éc-1, Éc-2, Éc-3) : 2 000 idées visibles et 4 000 retirées ; sept opérations par idée visible ;
   1 000 liens ; 4 500 000 octets. Défaut : oui ; la tranche 5, qui éprouve le test 13 entier, récrit sa ligne du
   § 31 · 10 en ce sens.
4. **La forme d'une annulation** (Éc-11) : « Annulé : » suivi du message entier du geste (« Annulé : Idée du
   3 novembre : étiquette #pont posée »), quand l'exemple du § 30 · 11 ôte l'idée. Défaut : le message entier ; la
   PR (b) corrige l'exemple.
5. **L'idée laissée vide par deux regroupements croisés, ou par un même fichier importé sur deux appareils** (C3,
   C28, Éc-7) : rien en tranche 2 hors du test qui l'écrit ; une piste pour le lot 3, et pour la tranche 5.
   Défaut : oui, la PR (c) écrit la piste.
6. **Un geste de tri qui touche une idée retirée** (Éc-10) : refusé (`IdeeRetiree`), sauf rétablir ; délier, depuis
   l'idée visible, un lien vers une idée retirée devient ainsi impossible, tant qu'elle n'est pas rétablie. Ce
   n'est pas une impossibilité stricte, le seul refus que l'ADR-006 admet (« Le seul refus admis est
   l'impossibilité stricte ») : c'est une règle de produit. Défaut : refusé, sauf rétablir ; l'ADR ne change pas,
   et la PR (b) le dit.
7. **Deux étiquettes qui s'affichent pareil** (Éc-21) : comparées octet pour octet, sans normalisation ; « été » en
   NFC et en NFD font deux clés. Le choix est inscrit au § 31 · 2, dans le schéma de la `v0.3.0` : le choix
   contraire, une normalisation à l'écriture, ne se rattraperait plus après elle sans migration. Défaut : octet
   pour octet, sans normalisation.

---

## Points de revue

Cinq conditions que la spec implique sans qu'aucune de ses sources ne demande de les éprouver ; chacune a son test,
dans la tâche qui possède le code.

1. **Une paire en mémoire rouverte de ses octets migre comme celle des fichiers.** Sinon la suite, sur la paire en
   mémoire, verrait un fonds plus ancien qu'aucune migration ne rattrape. `la_paire_en_memoire_migre_a_son_ouverture`,
   tâche 4.
2. **Un geste de tri sans fonds est refusé, et rien ne naît** : seul un geste qui crée une idée (noter, verser) fait
   naître le fonds. `un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait`, tâche 11, sur les deux paires (sur
   celle des fichiers, `documents/fonds/` reste absent, ce qu'assure son enveloppe, qui connaît la racine).
3. **Une fiche déjà versée ne se reverse pas**, et sa captation, elle, se reverse sans erreur (I14 : rien ne change
   une captation, ni sa fiche). `une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change`, tâche 11.
4. **Un geste refusé ne prend pas ce qu'annuler vise, une annulation refusée le garde.** La décision 18 se tait sur
   les deux. `un_geste_refuse_ne_change_pas_ce_qu_annuler_vise` (suite, tâche 12) et
   `une_annulation_refusee_par_le_disque_se_refait` (paire de fichiers, sous `pannes`, tâche 12).
5. **La réouverture vide ce qu'annuler vise** (« depuis l'ouverture »). `la_reouverture_vide_ce_qu_annuler_vise`,
   tâche 12.

## Mécanismes annoncés

- `crates/stockage/src/documents.rs:264-267` — `tete.is_none()` — une transaction sans modification n'écrit
  rien et rend `Ok(None)`, son message perdu : le noyau refuse un geste sans effet avant d'appliquer (Éc-5 ;
  tâche 10), et `application::appliquer` rend `Inapplicable::Vide` (tâche 4).
- `crates/stockage/src/documents.rs:251-256` — `pub fn ecrire(` — l'écriture publique prend une fermeture sur la
  `Transaction` : elle passe sous `pannes`, point d'entrée des tests (C17 ; tâche 6).
- `crates/stockage/src/documents.rs:235` — `§ 31 · 10, test 8` — le marqueur du doctest `compile_fail,E0502`, qui
  passe sur `Document::appliquer` (C16 ; tâche 6).
- `crates/stockage/src/documents.rs:123` — `pub fn prochain_message(&self, echange: &mut Echange)` — un message se
  prépare d'un document emprunté en lecture : le second doctest l'appelle après `appliquer` (tâche 6).
- `crates/synchro/src/lib.rs:40` — `pub fn prochain_message(&mut self, doc: &Automerge)` — l'échange lit un
  `&Automerge` : tenu pendant `appliquer`, qui emprunte le document en écriture, il fait refuser la compilation
  (E0502 ; tâche 6).
- `crates/stockage/src/documents.rs:1051-1063` — `automerge.transaction()` — la naissance écrit des valeurs que le
  stockage décide : elle passe au noyau, `gestes::naissance` (C17 ; tâches 2 et 6).
- `crates/stockage/src/documents.rs:1048` — `.apply_changes([genese])` — la genèse s'applique en octets à un
  document neuf : `application::naitre` le reprend (tâche 4).
- `crates/stockage/src/documents.rs:399` — `fn migrer(&mut self, a_appliquer: &[Migration<'_>])` — le stockage
  conduit la migration à l'ouverture : il garde ce rôle, par `application::migrer` (C15 ; tâche 6).
- `crates/stockage/src/documents.rs:631-636` — `fn empreintes_nommees` — les empreintes nommées ne se lisent que
  des clés de `PRISES` : la règle passe au noyau et s'étend aux MIDI versés seuls et aux textes versés (C3 ;
  tâche 8).
- `crates/stockage/src/racine_des_donnees.rs:179-185` — `fonds_selon` — l'ouverture selon des migrations de test
  reste sous `pannes` : le fonds N + 1 de la paire de fichiers se bâtit par elle (tâche 6).
- `crates/stockage/src/racine_des_donnees.rs:199-207` — `pub fn faire_naitre_le_fonds(` — la naissance publique
  prend une fermeture : elle passe sous `pannes`, à côté de `faire_naitre`, livrée (C17 ; tâche 6).
- `crates/stockage/Cargo.toml:17-18` — `pannes = []` — une fonction de Cargo sans dépendance : `en-memoire` suit le
  même modèle (tâche 4).
- `crates/stockage/Cargo.toml:22` — `features = ["pannes"]` — une crate dépend d'elle-même en test, avec sa
  fonction : `services` fait de même avec `en-memoire` (tâche 4).
- `Cargo.toml:3` — `resolver = "3"` — sans cible de test, `cargo clippy -p services --lib` ne tire aucune
  dépendance de test : l'étape « Lint des services, sans en-memoire » compile les services tels qu'ils se livrent
  (tâche 4).
- `crates/stockage/tests/commun/mod.rs:305-318` — `il changera seul` — la promesse de la tranche 1 ne tient pas :
  une fermeture ne se traduit pas en changements, et `commun::ecrire` reste sur `Document::ecrire`, sous `pannes`
  (C17 ; tâche 6).
- `crates/stockage/tests/commun/mod.rs:638` — `tx.put_object(&prises, empreinte, ObjType::Map)` — la prise de test
  est un objet sous son empreinte : elle passe en clés à plat (tâche 8).
- `crates/stockage/tests/genese.rs:78-81` — `la recette rend d'autres octets que GENESE` — le test imprime les
  octets de la recette : la CI donne la genèse neuve (tâche 8).
- `crates/stockage/tests/genese.rs:101` — `assert_eq!(cles, [IDEES, PRISES, VERSION_DE_SCHEMA]);` — la liste des
  clés de la racine du document change avec la genèse (tâche 8).
- `crates/stockage/tests/references.rs:19-20` — `Une ligne se change ou se retire avant la` — avant la `v0.3.0`,
  une référence du test 11 se change, et la PR le dit (tâches 8 et 13).
- `crates/stockage/tests/references.rs:23` — `TENEUR_DES_TYPES` — le témoin des types se bâtit sans genèse : sa
  référence ne change pas (Éc-18).
- `crates/stockage/tests/version.rs:699-712` — `le_fonds_nait_a_la_version_de_schema_connue` — la porte de la
  naissance sans migration : elle suit la naissance par `faire_naitre` (C15 ; tâche 6).
- `crates/noyau/src/donnees/demarrage.rs:90` — `pub fn pupitre_neuf` — la seule règle de « neuf » sur le disque :
  `RacineDesDonnees::faire_naitre` la reprend, sans en écrire une autre (C11 ; tâche 6).
- `crates/synchro/src/lib.rs:78` — `receive_sync_message` — la synchronisation reprend des modifications hors des
  adaptateurs : le contrôle d'I28 la cite comme reprise, sans l'interdire (C31 ; tâche 17).
- `crates/stockage/src/lib.rs:36` — `vient avec les commandes` — les messages en français promis au lot 1 : la
  tâche 16 les écrit.
- `crates/stockage/src/lib.rs:211` — `pub fn empreinte(octets: &[u8]) -> String` — l'empreinte d'un contenu vit
  dans le stockage : elle passe au noyau, pour la paire en mémoire, et le stockage la réexporte (tâche 1).
- `crates/noyau/src/rendu/cahier.rs:11` — `use sha2::{Digest, Sha256};` — le noyau dépend déjà de `sha2` :
  l'empreinte y entre sans dépendance nouvelle (tâche 1).
- `crates/noyau/Cargo.toml:9` — `default-features = false` — le modèle de la ligne d'`ulid`, que le contrôle
  statique du noyau lit (tâches 1 et 17).
- `app/ui/tests/encre_statique.rs:318-322` — `sous-chaîne, sensible à la casse` — le modèle des contrôles statiques :
  une recherche en sous-chaîne, commentaires compris (tâche 17).
- `app/ui/tests/encre_statique.rs:332` — `aucune source Rust trouvée` — un contrôle qui ne lit rien échoue : chaque
  contrôle statique de la tranche le reprend (tâche 17).
- `scripts/verif-faits.mjs:643-656` — `fonctionsNommeesCritere` — un test de critère est un `fn capNN_n_…`
  littéral, `#[test]` au-dessus : les fonctions de la suite ne portent pas ce nom, leurs enveloppes si (tâche 5).
- `scripts/verif-faits.mjs:741-744` — `--exclude-standard` — la porte lit chaque `.rs` suivi ou à suivre : le
  fichier de la suite partagée compris (tâche 5).
- `.cargo/mutants.toml:26` — `Un nouveau fichier de tests` — chaque fichier de tests d'un module muté entre dans
  `additional_cargo_test_args` (tâches 1, 2, 9).
- `.github/workflows/ci.yml:99` — `cargo clippy -p noyau -p cahier -p stockage -p synchro --all-targets` — le lint
  de `verifier` nomme ses crates : `-p commandes -p services` s'y ajoutent (tâche 4).
- `.github/workflows/ci.yml:112` — `cargo test -p noyau -p cahier -p interface -p stockage -p synchro` — de même
  pour les tests (tâche 4).
- `.github/workflows/ci.yml:146` — `cargo test -p stockage -p synchro` — le job `windows` teste le stockage, donc
  la suite sur la paire de fichiers, sans changement (tâche 6).
- `.github/workflows/ci.yml:110` — `--target wasm32-unknown-unknown` — le lint en WebAssembly de l'interface
  compile ses dépendances, `commandes` dès la tâche 6.
- `.github/workflows/ci.yml:4` — `pull_request:` — `ci.yml` tourne sur une PR : chaque PR s'ouvre en brouillon au
  premier envoi.
- `.github/workflows/ci.yml:86-89` — `cargo update --workspace` — un `Cargo.lock` en retard arrête `verifier` avant
  tout lint : l'étape « Le graphe de l'interface » se place juste après lui, avant « Format », pour que son essai
  la voie rougir (tâche 17).
- `.github/workflows/ci.yml:72-75` — `mesuré à froid` — le délai de `verifier` se ramène à deux fois un passage
  mesuré à froid : la remesure de la tâche 19 se fait sans cache.
- `scripts/verif-faits.mjs:894-908` — `cratesDeClaude` — la liste « Les crates » de `CLAUDE.md` nomme chaque membre
  de l'espace de travail, et un membre sans `Cargo.toml` arrête le script : la puce et le `Cargo.toml` d'une crate
  entrent avec son membre (tâches 3 et 4).
- `crates/stockage/src/lib.rs:22` — `use std::fmt::Write;` — seul `empreinte` l'emploie (`write!`, :215) : l'import
  part avec elle, sinon `-D warnings` le refuse (tâche 1).
- `crates/stockage/src/lib.rs:21-28` — `ecrire_la_teneur` — les imports de la teneur (`BTreeMap`, les noms
  d'`automerge` hors `AutomergeError`, ceux de `noyau::donnees::teneur`) partent avec elle (tâche 12).
- `crates/stockage/Cargo.toml:20-22` — `[dev-dependencies]` — un test du stockage ne voit que ses dépendances et ses
  dépendances de test : la suite partagée, qui nomme les types de `commandes`, y demande `commandes` (tâche 6).
- `crates/stockage/Cargo.toml:14-16` — `fonds_selon` — le commentaire de `pannes`, comme la doc de la crate
  (`crates/stockage/src/lib.rs:6-9`), se récrit avec ce qu'elle compile de plus (tâche 6).
- `crates/noyau/src/donnees/version.rs:30` — `LireSansEcrire,` — une version de schéma plus récente se lit sans
  écrire : `DocumentsEnMemoire::de` met le fonds en lecture seule (tâche 4).
- `crates/stockage/tests/references.rs:157-171` — `lignee-2` — deux témoins ne diffèrent aujourd'hui que par leur
  `Naissance` ; nés par les services, ils ne diffèrent que par le deuxième tirage de l'aléa (tâche 13).
- `crates/stockage/src/documents.rs:585` — `ActorId::random()` — chaque ouverture tire son acteur : un cas de
  `proptest` qui échoue ne se rejoue pas à l'identique (tâche 15).
- `crates/stockage/src/lib.rs:44-47` — `chemin: PathBuf` — une erreur porte le chemin absolu que bâtit le stockage :
  l'adaptateur le rend relatif à la racine des données avant d'en former le message (tâche 16).
- `crates/synchro/src/lib.rs:33` — `Result<Echange, Difference>` — deux copies étrangères ne s'ouvrent aucun
  échange : `Difference` gagne son message (tâche 16).
- `crates/stockage/src/lib.rs:155` — `ErreurDeTeneur` — ce que la teneur refuse gagne aussi son message (tâche 16).
- `docs/30-modele-de-domaine.md:59-63` — `class Lien` — le modèle donne une date (`le`) au lien : la PR (b) la
  retire, sa date se lisant dans l'historique (Éc-9 ; tâche 13).
- `docs/30-modele-de-domaine.md:718` — `l'annulation est refusée` — le refus d'une annulation après une
  synchronisation reste au § 30 · 11, renvoyé au lot 3 (tâche 13).

## Outils et cas limites

- `ulid` 3.0.0, `Ulid::from_parts` — des bits au-delà de 48 pour l'heure ou de 80 pour l'aléa sont masqués sans
  erreur — `une_heure_au_dela_de_2_puissance_48_millisecondes_s_y_borne` (tâche 1), où l'heure, bornée à 2⁴⁸ − 1 ms
  avant `from_parts`, rend `7ZZZZZZZZZ0000000000000000` au lieu de l'heure masquée ; l'aléa, dix octets, n'a jamais
  plus de quatre-vingts bits ; et `l_ulid_porte_l_heure_en_millisecondes_puis_l_alea`, `une_heure_negative_compte_pour_zero`
- `serde_json` — un entier au-delà de 2⁵³ − 1 s'écrit, puis `JSON.parse` l'arrondit ; NaN et l'infini s'écrivent
  `null` — `verser_une_duree_au_dela_d_entier_sur_est_refusee` (tâche 10), qui borne le seul entier d'une réponse
  venu d'une fiche ; chaque autre entier d'une réponse vient d'une source bornée, que la tâche 3 nomme ;
  `les_entiers_d_une_reponse_restent_sous_2_puissance_53` (tâche 3) éprouve la forme des nombres écrits ; aucun type
  de `commandes` n'a de flottant
- `commit_with` d'Automerge — une transaction dont chaque changement ne change rien rend `None`, sans modification
  ni message — `une_operation_sans_effet_est_inapplicable` (tâche 4)
- `put` d'Automerge sur une clé en conflit — il écrit, et ôte les autres valeurs — `l_inverse_d_un_texte_en_conflit_rend_la_valeur_presentee`
  (tâche 15)
- `get` d'Automerge — deux valeurs concurrentes d'une clé, dont il ne présente qu'une — `un_texte_change_des_deux_cotes_garde_ses_deux_valeurs`,
  par `get_all` (tâche 15)
- `put` d'Automerge, la même valeur posée sur deux répliques — deux valeurs, qu'Automerge ne fusionne pas
  (`test_overwriting_a_conflict`, dans les tests d'automerge 0.12.0) — `une_prise_nee_sur_deux_repliques_ne_cache_aucun_de_ses_faits`
  et `une_etiquette_posee_des_deux_cotes_n_en_fait_qu_une` (tâche 15), qui attendent deux valeurs égales
- `get_changes(&[])` d'Automerge — l'ordre d'arrivée, causal, et non l'heure — `cap02_6_annuler_rend_le_fonds_d_avant_et_le_journal_porte_le_geste_puis_son_annulation`
  (tâche 12), qui lit le geste avant son annulation
- `ScalarValue::as_i64` et `as_u64` d'Automerge — ils convertissent sans distinguer `Int` et `Uint` — `une_valeur_d_un_autre_type_rend_l_idee_illisible`
  (tâche 4) : la lecture compare la variante (`ScalarValue::Int`), jamais la conversion
- `rustdoc` — un doctest `compile_fail` passe sur toute erreur de compilation — le doctest de `Document::appliquer`
  (tâche 6), qui dit la sienne, `compile_fail,E0502`
- `cargo tree` — une erreur de `cargo tree` ne laisse rien sur la sortie, qu'un `grep` lirait comme une absence —
  l'étape « Le graphe de l'interface » (tâche 17), qui capture la sortie sous `bash -e`, puis l'essai sur `stockage`,
  qui doit trouver `^automerge v` et `^services v`
- `cargo tree` — sa première ligne est le paquet racine, sans préfixe (`stockage v0.0.0 (…)`), qu'un motif qui nomme
  `stockage` trouve toujours — l'essai de la même étape cherche une dépendance, `^automerge v`, jamais la racine
- `cargo tree` — sans `--target`, seules les dépendances de l'hôte paraissent — la même étape, deux fois : en natif
  et avec `--target wasm32-unknown-unknown`
- `verifier`, l'ordre de ses étapes — un `Cargo.lock` en retard échoue à « Cargo.lock à jour », et `getrandom` sans
  `wasm_js` arrête « Lint de l'interface (WebAssembly) » : une étape placée après eux ne tourne pas — l'essai de la
  tâche 17, qui ajoute `services` à l'interface, l'étape du graphe placée juste après le `Cargo.lock`
- `verif-faits.mjs` (`fonctionsNommeesCritere`) — une fonction `capNN_n_…` sans `#[test]` juste au-dessus ne compte
  pas, et un critère inscrit sans test qui le nomme refuse — `node scripts/verif-faits.mjs` après le `git add` de la
  tâche 13, et ses tests (`node --test scripts/verif-faits.test.mjs`) inchangés
- la porte du § 70 · 4.1 (`verif-faits.mjs`) — elle ne voit pas un critère livré que la PR n'inscrit pas — la revue
  de la PR (b) compare le tableau aux critères de la tranche (tâche 13)
- la recherche en sous-chaîne des contrôles statiques — un mot interdit cité dans un commentaire ou une chaîne la
  fait échouer, et `.insert(` est aussi celui d'un `BTreeSet` — `chaque_controle_refuse_ce_qu_on_y_plante` (tâche 17) ;
  le contrôle d'I28 ne cherche pas `.insert(`, mais `Transactable` et `automerge::transaction`, sans lesquels
  aucune insertion ne s'écrit
- la recherche en sous-chaîne des contrôles statiques — un import groupé (`use std::{fs, path::Path};`) laisse
  `fs::read` sans `std::fs` — les chemins courts (`fs::`, `io::`, `env::`, `thread::`, `net::`, `process::`,
  `time::`) cherchés aussi, et `chaque_controle_refuse_ce_qu_on_y_plante` (tâche 17), qui y plante un import groupé
- `Path::display` sous Windows — `\` entre les composants d'un chemin, `/` sous Linux — `chaque_erreur_a_son_message_en_francais`
  (tâche 16), dont les chemins ont plusieurs composants, joints par `/` dans le message, et qui tourne dans `windows`
- `std::fs::read_dir` dans les contrôles statiques — un dossier qui ne se lit pas, ou vide, laisserait passer
  le contrôle — `assert!(!sources.is_empty(), …)` dans chaque contrôle (tâche 17)
- `proptest` — un cas qui échoue se réduit, sans borne utile par défaut, et la graine s'écrit dans
  `proptest-regressions/` ; l'acteur d'Automerge, tiré à chaque ouverture, empêche de le rejouer à l'identique —
  `trois_repliques_convergent` (tâche 15), seize cas, la réduction bornée (`max_shrink_iters`, `max_shrink_time`),
  aucun fichier de régression suivi
- `cargo-mutants` — un module sans test dans `additional_cargo_test_args` laisse ses mutants survivre (`MISSED`) —
  la remesure à froid des tâches 7 et 13
- `cargo-mutants` — `|` sur des bits que le masque a mis à zéro, remplacé par `^`, ne change rien (mutant
  équivalent) ; un terme qui vaut 0 sur chaque date des tests survit à son mutant — `uuid_v4` écrit par `+`
  (tâche 1), dont les mutants `-` et `*` meurent sur `[0; 16]`, et les 29 février 2000 et 1er mars 2100 de
  `la_date_du_message_suit_le_decalage_et_dit_le_premier` (tâche 2)
- `cargo-mutants` — `<` remplacé par `<=`, dans une comparaison dont les deux branches rendent la même chose à
  l'égalité, ne change rien (mutant équivalent), et survivrait — `la_cle_du_lien_range_ses_bouts_par_octets`
  (tâche 9), sur `cle_du_lien` écrit par `a.min(b)` et `a.max(b)`, qu'aucun opérateur ne fait muter
- `cargo-mutants` — un `Display` dans un module muté (`copie.rs`, `teneur.rs`) a ses mutants — les messages de
  `Difference` et d'`ErreurDeTeneur` comparés dans `tests/donnees.rs` et `tests/teneur.rs` du noyau (tâche 16),
  que lance la mutation

## Revues

- la revue de chaque tâche : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12, Tâche 13, Tâche 14, Tâche 15, Tâche 16, Tâche 17, Tâche 18, Tâche 19
- la revue de la PR (a) entière, sur sa tête, par le modèle le plus capable, avant `gh pr ready` : Tâche 7
- la revue de la PR (b) entière, sur sa tête, le tableau du § 70 · 4.1 comparé aux critères de la tranche : Tâche 13
- la revue de la PR (c) entière, sur sa tête, LE-14 et les délais compris, avant les gestes de clôture : Tâche 19
- la relecture des corrections de chaque revue de PR, sur la tête qui part, celle que nomme `--match-head-commit` : Tâche 7, Tâche 13, Tâche 19

---

## La carte des fichiers

```text
Cargo.toml                         (a) membres : + crates/commandes, crates/services
crates/commandes/                  (a) Cargo.toml, src/lib.rs, tests/serialisation.rs
crates/services/
  Cargo.toml                       (a) noyau, commandes, automerge ; [features] en-memoire ; dépendance de test sur elle-même
  src/lib.rs                       (a) Services, Contexte, noter, les lectures ; (b) les gestes de tri, annuler, verser_une_fiche ;
                                   (c) pub mod mesure
  src/contrats.rs                  (a) Documents, Captations, Alea, Ecriture
  src/application.rs               (a) appliquer, naitre, faire_naitre, migrer, Inapplicable ; (c) Display
  src/lecture.rs                   (a) lire_l_idee, lire_la_captation, captations_de, idees, journal, version_de_schema ;
                                   (b) empreintes_nommees, teneur
  src/en_memoire.rs                (a) DocumentsEnMemoire, CaptationsEnMemoire, sous en-memoire ; (c) les messages des refus
  src/mesure.rs                    (c) engendrer_le_fonds_de_mesure, et engendrer_le_fonds_de_mesure_selon (pub),
                                   qu'elle appelle avec ses comptes
  tests/suite/mod.rs               (a, b, c) la suite générique, Paire, AleaDeTest, IMPORT
  tests/commandes_en_memoire.rs    (a, b, c) la suite sur la paire en mémoire ; (c) le_generateur_est_deterministe
  tests/application.rs             (a) appliquer, migrer ; (c) les messages d'Inapplicable
  tests/statique.rs                (c) I28 statique ; ni fichier, fil, réseau ni horloge ; uuid_v4 en un seul endroit
crates/noyau/
  Cargo.toml                       (a) + ulid
  src/donnees/mod.rs               (a) + changement, gestes, identifiants ; (b) + idee
  src/donnees/identifiants.rs      (a) ulid, uuid_v4, empreinte                                        (muté)
  src/donnees/changement.rs        (a) Simple, Changement, Operation
  src/donnees/gestes.rs            (a) le fonds lu, naissance, noter, date_du_message ; (b) les gestes de tri, verser, annulation (muté)
  src/donnees/idee.rs              (b) cle_du_lien, reliee, duree, est_vide, assiette, empreintes_nommees (muté)
  src/donnees/fonds.rs             (a) les clés du schéma ; (b) CONTENEURS_DE_LA_GENESE, GENESE
  src/donnees/teneur.rs            (b) sa doc : l'arbre tiré par services ; (c) Display d'ErreurDeTeneur       (muté)
  src/donnees/copie.rs             (c) Display de Difference                                           (muté)
  tests/identifiants.rs (a) ; tests/gestes.rs (a, b) ; tests/idee.rs (b) ; tests/statique.rs (c) ;
  tests/donnees.rs, tests/teneur.rs (c) les messages de Difference et d'ErreurDeTeneur
crates/stockage/
  Cargo.toml                       (a) + services ; sha2 retiré ; en test, services avec en-memoire, et commandes ;
                                   le commentaire de pannes
  src/lib.rs                       (a) empreinte réexportée ; Naissance sous pannes ; la doc de pannes ;
                                   (b) teneur réexportée ; (c) Display
  src/documents.rs                 (a) appliquer, la naissance par opérations, ecrire sous pannes, migrer partagé ;
                                   (b) empreintes_nommees du noyau
  src/racine_des_donnees.rs        (a) faire_naitre ; faire_naitre_le_fonds sous pannes
  src/adaptateurs.rs               (a) FondsSurFichiers, Captations réalise le contrat ; (c) les chemins relatifs des messages
  src/fichier.rs                   (c) Display d'Echec
  tests/commandes_sur_fichiers.rs  (a, b, c) la suite sur la paire de fichiers, et ses tests sous pannes
  tests/commun/mod.rs              (b) poser_une_prise en clés à plat
  tests/genese.rs, references.rs   (b) la genèse et les références du test 11 ; references.rs inclut la suite
  tests/demarrage.rs               (b) un MIDI ou un texte versé nommé prend son nom
  tests/h07.rs                     (b) les prises lues par idee::empreintes_nommees
  tests/repliques.rs               (b) CAP-02 · 7 ; (c) deux et trois répliques ; il inclut la suite
  tests/messages.rs                (c) un message pour chaque erreur
  tests/mesure.rs                  (c) le fonds de mesure, #[ignore]
crates/synchro/src/lib.rs          (c) Display de synchro::Erreur
app/ui/Cargo.toml                  (a) + commandes
.cargo/mutants.toml                (a) identifiants, gestes ; (b) idee
.github/workflows/ci.yml           (a) -p commandes -p services, « Lint des services, sans en-memoire » ; (a, b) délais de mutants ;
                                   (c) « Le graphe de l'interface », après « Déposer le Cargo.lock résolu »,
                                   « Le fonds de mesure » ; délais de verifier et windows
.github/workflows/mutants-complet.yml  (a, b) délais
docs/40-architecture.md            § 3 (a)
docs/41-pile-technique.md          § 6 (a)
docs/31-persistance-et-synchronisation.md  § 2, 2.3 (b) ; § 2.4, 2.6 (c)
docs/30-modele-de-domaine.md       § 1, § 9.3, § 11 (b)
docs/20-sfd/21-capter-et-organiser.md  CAP-02 (b) ; CAP-01, cas limite (c)
docs/70-qualite-et-verification.md § 4.1 : CAP-01 (a) ; CAP-02 (b)
docs/80-glossaire.md               Étiquette, Annuler (b)
docs/pistes.md                     (c) les regroupements croisés ; deux valeurs égales
CLAUDE.md                          « Les crates » (a : commandes à la tâche 3, services à la tâche 4, stockage à la
                                   tâche 7), « Commandes » (a) ; « Commandes » (c)
CONVENTIONS.md                     verifier, mutants (a) ; mutants (b) ; verifier (c)
docs/registre.md                   LE-14 clos (tâche 19, avant la revue de la PR (c))
```

---

## PR (a) · les commandes, les services, la paire en mémoire, la naissance et noter

**Branche :** `t2a-commandes-et-services`. **Titre :** `feat: les commandes, les services et leur paire en
mémoire ; noter fait naître le fonds` (87). **Documents :** § 40 · 3 (correction (a) ; `sha2`, `pannes`) ; § 41 · 6
(correction (c) ; la ligne `sha2`) ; § 70 · 4.1 (CAP-01 · 6) ; `CLAUDE.md` (« Les crates », « Commandes ») ;
`CONVENTIONS.md` (`verifier`, `mutants`) ; `.cargo/mutants.toml` ; `ci.yml`, `mutants-complet.yml`. **Fin :** voir la
tâche 7.

### Tâche 1 : les identifiants, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/identifiants.rs`, `crates/noyau/tests/identifiants.rs` ; modifier
`crates/noyau/Cargo.toml` (ligne `ulid = { version = "3.0.0", default-features = false }`, après `sha2`),
`crates/noyau/src/donnees/mod.rs`, `crates/stockage/src/lib.rs` (:22, `use std::fmt::Write;`, que seul `empreinte`
emploie ; :29 et :210-218 : `empreinte` réexportée),
`crates/stockage/Cargo.toml` (`sha2` retiré, plus employé), `.cargo/mutants.toml`, `Cargo.lock` (de l'artefact).

**Interfaces — produit** (`noyau::donnees::identifiants`) :

```rust
/// L'ULID de l'heure `heure`, en secondes (négative, elle compte pour 0 ; au-delà de 2⁴⁸ − 1 ms,
/// la plus grande heure d'un ULID, elle s'y borne), et de dix octets d'aléa (§ 30 · 2 ; § 41 · 6) :
/// `Ulid::from_parts(heure × 1 000, aléa)`, en 26 caractères de Crockford.
pub fn ulid(heure: i64, alea: [u8; 10]) -> String;
/// L'UUID version 4 de seize octets reçus (RFC 9562) : l'octet 6 prend `0100` en tête, l'octet 8
/// `10` ; en minuscules, 8-4-4-4-12.
pub fn uuid_v4(octets: [u8; 16]) -> String;
/// Le SHA-256 de `octets`, en 64 caractères hexadécimaux minuscules : l'empreinte d'une captation.
pub fn empreinte(octets: &[u8]) -> String;
```

`stockage::empreinte` devient `pub use noyau::donnees::identifiants::empreinte;` : ses appels ne changent pas.

- [ ] **Étape 1 : écrire les tests.**

```rust
use noyau::donnees::identifiants::{empreinte, ulid, uuid_v4};

#[test]
fn l_ulid_porte_l_heure_en_millisecondes_puis_l_alea() {
    assert_eq!(ulid(1_794_441_600, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]), "01M76GVQ00041061050R3GG28A");
    assert_eq!(ulid(1_700_000_000, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]), "01HF7YAT000000000000000001");
    // Des secondes prises pour des millisecondes donneraient le préfixe 0001JN7W80.
    assert_eq!(&ulid(1_700_000_000, [0; 10])[..10], "01HF7YAT00");
}
#[test]
fn une_heure_negative_compte_pour_zero() {
    assert_eq!(ulid(0, [0; 10]), "00000000000000000000000000");
    assert_eq!(ulid(-1, [0; 10]), "00000000000000000000000000");
}
#[test]
fn une_heure_au_dela_de_2_puissance_48_millisecondes_s_y_borne() {
    // 2⁴⁸ − 1 ms, la plus grande heure d'un ULID : from_parts masquerait l'heure, sans erreur
    // (00000000AR0000000000000000), et l'ULID ne se rangerait plus comme elle.
    assert_eq!(ulid(281_474_976_711, [0; 10]), "7ZZZZZZZZZ0000000000000000");
    assert_eq!(ulid(i64::MAX, [0; 10]), "7ZZZZZZZZZ0000000000000000");
    assert_eq!(ulid(281_474_976_710, [0; 10]), "7ZZZZZZZBG0000000000000000"); // sous la borne
}
#[test]
fn deux_ulid_se_rangent_comme_leurs_heures() {
    assert!(ulid(1_700_000_000, [0xFF; 10]) < ulid(1_700_000_001, [0; 10]));
}
#[test]
fn l_uuid_v4_pose_sa_version_et_sa_variante() {
    assert_eq!(uuid_v4([0; 16]), "00000000-0000-4000-8000-000000000000");
    assert_eq!(uuid_v4([0xFF; 16]), "ffffffff-ffff-4fff-bfff-ffffffffffff");
    let suite: [u8; 16] = core::array::from_fn(|i| i as u8);
    assert_eq!(uuid_v4(suite), "00010203-0405-4607-8809-0a0b0c0d0e0f");
    let tire = [0x91, 0x91, 0x08, 0xF7, 0x52, 0xD1, 0x33, 0x20, 0x5B, 0xAC, 0xF8, 0x47, 0xDB, 0x41, 0x48, 0xA8];
    assert_eq!(uuid_v4(tire), "919108f7-52d1-4320-9bac-f847db4148a8");
}
#[test]
fn l_empreinte_est_le_sha256_en_hexadecimal() {
    assert_eq!(empreinte(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: les identifiants du noyau`), pousser, ouvrir la PR en brouillon. Attendu à « Lint » de `verifier` : `unresolved import noyau::donnees::identifiants`.
- [ ] **Étape 3 : écrire le code.** `ulid` : `const HEURE_MAX_MS: u64 = (1 << 48) - 1;` (privée, la plus grande heure d'un ULID), puis `let ms = (heure.max(0) as u64).saturating_mul(1000).min(HEURE_MAX_MS);` puis `Ulid::from_parts(ms, u128::from_be_bytes(…))`, l'aléa dans les dix octets de poids faible (quatre-vingts bits, que `from_parts` ne masque jamais), puis `to_string()` ; les mutants de la borne, `(1 << 48) + 1` et `(1 << 48) / 1`, rendent `00000000010000000000000000` et `00000000000000000000000000` au premier cas de `une_heure_au_dela_de_2_puissance_48_millisecondes_s_y_borne`, et meurent (simulés sur le poste). `uuid_v4` : `octets[6] = (octets[6] & 0x0F) + 0x40; octets[8] = (octets[8] & 0x3F) + 0x80;` puis l'écriture 8-4-4-4-12 ; `+` et non `|` : sur des bits que le masque a mis à zéro, le mutant `|` → `^` ne change rien, et survivrait (mutant équivalent), quand les mutants `-` (débordement, en debug) et `*` de `+` meurent sur `[0; 16]`. `empreinte` : le corps de `stockage::empreinte`, déplacé. `.cargo/mutants.toml` : `"crates/noyau/src/donnees/identifiants.rs"` dans `examine_globs`, `"--test", "identifiants"` dans `additional_cargo_test_args`, les deux mentions du nombre de modules, l'en-tête (:2, « Quatorze modules sont examinés ») et le commentaire d'`additional_cargo_test_args` (:22, « ces quatorze modules »), et la liste des règles des données qui nomme « les identifiants ».
- [ ] **Étape 4 : voir le vert.** Pousser. Le `Cargo.lock` en retard échoue à « Cargo.lock à jour » : prendre l'artefact `cargo-lock`, le relire (une entrée `ulid 3.0.0`, `sha2` ôté des dépendances de `stockage`), le committer. Attendu : `identifiants.rs` `ok. 6 passed` dans `verifier` ; dans `windows`, les tests du stockage gardent leurs comptes.
- [ ] **Étape 5 : committer** `git add crates/noyau/Cargo.toml crates/noyau/src/donnees/mod.rs crates/noyau/src/donnees/identifiants.rs crates/stockage/src/lib.rs crates/stockage/Cargo.toml .cargo/mutants.toml Cargo.lock` (`feat: les identifiants au noyau, ULID, UUID et empreinte`).

### Tâche 2 : les changements, le fonds lu, la naissance et noter, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/changement.rs`, `crates/noyau/src/donnees/gestes.rs`,
`crates/noyau/tests/gestes.rs` ; modifier `crates/noyau/src/donnees/mod.rs`, `crates/noyau/src/donnees/fonds.rs`
(les clés du schéma, sans toucher `GENESE`), `.cargo/mutants.toml`.

**Interfaces — produit :**

```rust
// noyau::donnees::changement : ce que rend le noyau, que l'adaptateur `Documents` applique (§ 31 · 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Simple { Chaine(String), Entier(i64), Naturel(u64), Vrai }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Changement {
    /// Pose `valeur` sous la dernière clé de `chemin`, dans l'objet que nomment les autres.
    Poser { chemin: Vec<String>, valeur: Simple },
    /// Crée un objet à clés sous `chemin`, et, dans la même modification, chacun de ses `conteneurs`.
    Creer { chemin: Vec<String>, conteneurs: Vec<String> },
    /// Ôte la dernière clé de `chemin`.
    Oter { chemin: Vec<String> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation { pub changements: Vec<Changement>, pub message: String, pub heure: i64 }

// noyau::donnees::gestes : le fonds lu, et les gestes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appareil { Etabli, Carnet }
impl Appareil { pub fn nom(self) -> &'static str } // "etabli", "carnet"
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance { Captee { par: Appareil }, Importee { par: Appareil, depuis: String } }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sorte { Prise, MidiVerseSeul, TexteVerse }
impl Sorte { pub fn conteneur(self) -> &'static str } // PRISES, MIDI_VERSES, TEXTES_VERSES
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdeeLue {
    pub id: String,
    pub provenance: Provenance,
    pub le: i64,
    pub texte: Option<String>,
    pub etiquettes: BTreeSet<String>,
    pub retiree: Option<i64>,
    /// Les clés des liens dont elle est un bout.
    pub liens: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptationLue { pub sorte: Sorte, pub empreinte: String, pub idee: String, pub duree_ms: Option<u64> }
/// Ce qui se verse d'une captation, ses faits lus : `format`, au vocabulaire du § 30 · 9.2 et de
/// l'ADR-024, jamais l'extension (la table de la tâche 10) ; `duree_ms`, d'une prise seulement.
/// La date et l'appareil d'une prise n'y sont pas : rien ne les lit sur la captation à INC-1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fiche { pub format: String, pub duree_ms: Option<u64> }
/// Pourquoi un geste ne se fait pas : un constat, jamais un ordre (§ 50 · 5). Les variantes des
/// gestes de tri arrivent à la tâche 10.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Impossible { TexteVide /* , … */ }
impl std::fmt::Display for Impossible { /* « Le texte de la note est vide. » */ }
/// « 3 novembre », « 1er novembre » : le jour de `heure`, en secondes, au décalage local
/// `decalage_minutes` que l'interface donne (décision 10), sans année.
pub fn date_du_message(heure: i64, decalage_minutes: i32) -> String;
/// La naissance : l'identifiant, la lignée et l'identifiant de propriétaire posés à la racine du
/// document, sous `MESSAGE_DE_LA_NAISSANCE`.
pub fn naissance(identifiant: &str, lignee: &str, proprietaire: &str, heure: i64) -> Operation;
/// Noter : l'idée `idee` créée avec son objet des étiquettes, son texte et sa provenance `captee`,
/// par `appareil`, née à `heure` ; message « Idée du <jour> notée ». Un texte vide une fois ses
/// blancs ôtés : `Impossible::TexteVide`.
pub fn noter(idee: &str, texte: &str, appareil: Appareil, heure: i64, decalage_minutes: i32)
    -> Result<Operation, Impossible>;
```

Au `fonds.rs`, les clés du schéma de la `v0.3.0` que les tâches suivantes écrivent, chacune avec sa doc :
`MIDI_VERSES = "midi_verses"`, `TEXTES_VERSES = "textes_verses"`, `LIENS = "liens"`,
`REQUETES_ENREGISTREES = "requetes_enregistrees"`, `IMPORTS = "imports"`, `ORIGINES_DECLAREES =
"origines_declarees"`, `PHOTOS = "photos"` ; d'une idée, `PROVENANCE = "provenance"`, `PAR = "par"`, `LE = "le"`,
`DEPUIS = "depuis"`, `RETIREE = "retiree"`, et les valeurs `CAPTEE = "captee"`, `IMPORTEE = "importee"` ; d'une
captation, sous `<empreinte>/<fait>`, `IDEE`, `FORMAT` (déjà là), et, d'une prise seulement (§ 30 · 9.2), `DUREE = "duree"` ; la date et l'appareil d'une captation n'ont pas de clé en tranche 2 (C3), `LE` et `PAR` restant des clés d'une idée ; d'un lien,
`APPARENTE = "apparente"`. La doc de `PRISES` dit les clés à plat (« de `<empreinte>/<fait>` à sa valeur »).

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/gestes.rs`).

```rust
const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023, 22 h 13 min 20 s UTC

#[test]
fn la_date_du_message_suit_le_decalage_et_dit_le_premier() {
    assert_eq!(date_du_message(1_793_748_600, 0), "3 novembre"); // 2026-11-03, 23 h 30 UTC
    assert_eq!(date_du_message(1_793_748_600, 60), "4 novembre");
    assert_eq!(date_du_message(1_793_748_600, -300), "3 novembre");
    assert_eq!(date_du_message(1_793_491_200, 0), "1er novembre");
    assert_eq!(date_du_message(1_793_491_200, -60), "31 octobre");
    assert_eq!(date_du_message(1_798_759_800, 60), "1er janvier");
    assert_eq!(date_du_message(1_835_438_400, 0), "29 février");
    assert_eq!(date_du_message(0, 0), "1er janvier");
    assert_eq!(date_du_message(-1, 0), "31 décembre");
    // Les termes de siècle de l'algorithme ne changent l'année qu'aux abords du 1er mars :
    // doe / 36 524, aux premiers jours de mars après le premier siècle de l'ère (dès le 1er mars
    // 2100 dans celle de 2000) ; doe / 146 096, au seul 29 février qui finit l'ère (celui de 2000
    // dans l'ère de 1600).
    assert_eq!(date_du_message(951_825_600, 0), "29 février"); // 2000-02-29, 12 h UTC
    assert_eq!(date_du_message(4_107_585_600, 0), "1er mars"); // 2100-03-01, 12 h UTC
}
#[test]
fn la_naissance_pose_trois_valeurs_a_la_racine() {
    let op = naissance("01HF7YAT000000000000000001", "01HF7YAT000000000000000002",
                       "00000000-0000-4000-8000-000000000003", HEURE);
    assert_eq!(op.message, "Fonds créé");
    assert_eq!(op.heure, HEURE);
    assert_eq!(op.changements, vec![
        poser(&["identifiant"], Simple::Chaine("01HF7YAT000000000000000001".into())),
        poser(&["lignee"], Simple::Chaine("01HF7YAT000000000000000002".into())),
        poser(&["proprietaire"], Simple::Chaine("00000000-0000-4000-8000-000000000003".into())),
    ]);
}
#[test]
fn noter_cree_l_idee_captee_datee_avec_ses_etiquettes() {
    let op = noter("01HF7YAT000000000000000004", "Un refrain qui monte, en ré", Appareil::Etabli, HEURE, 0).unwrap();
    assert_eq!(op.message, "Idée du 14 novembre notée");
    let i = "01HF7YAT000000000000000004";
    assert_eq!(op.changements, vec![
        Changement::Creer { chemin: chemin(&["idees", i]), conteneurs: vec!["etiquettes".into()] },
        poser(&["idees", i, "texte"], Simple::Chaine("Un refrain qui monte, en ré".into())),
        poser(&["idees", i, "provenance"], Simple::Chaine("captee".into())),
        poser(&["idees", i, "par"], Simple::Chaine("etabli".into())),
        poser(&["idees", i, "le"], Simple::Entier(HEURE)),
    ]);
    let au_carnet = noter(i, "x", Appareil::Carnet, HEURE, 0).unwrap();
    assert!(au_carnet.changements.contains(&poser(&["idees", i, "par"], Simple::Chaine("carnet".into()))));
}
#[test]
fn une_note_vide_est_refusee() {
    assert_eq!(noter("i", "", Appareil::Etabli, HEURE, 0), Err(Impossible::TexteVide));
    assert_eq!(noter("i", " \t\n", Appareil::Etabli, HEURE, 0), Err(Impossible::TexteVide));
    assert_eq!(Impossible::TexteVide.to_string(), "Le texte de la note est vide.");
}
#[test]
fn chaque_sorte_a_son_conteneur() {
    assert_eq!(Sorte::Prise.conteneur(), "prises");
    assert_eq!(Sorte::MidiVerseSeul.conteneur(), "midi_verses");
    assert_eq!(Sorte::TexteVerse.conteneur(), "textes_verses");
    assert_eq!((Appareil::Etabli.nom(), Appareil::Carnet.nom()), ("etabli", "carnet"));
}
```

  Les aides `poser(&[&str], Simple) -> Changement` et `chemin(&[&str]) -> Vec<String>` sont privées au fichier.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la naissance et noter, au noyau`), pousser. Attendu à « Lint » : `unresolved import noyau::donnees::gestes`.
- [ ] **Étape 3 : écrire le code.** `date_du_message` : `let jours = (heure + i64::from(decalage_minutes) * 60).div_euclid(86_400);` puis le calcul civil des jours depuis 1970 (algorithme « days from civil » inversé, sur des entiers ; les dates de 2000 et de 2100 du test tuent les mutants de ses termes de siècle, qu'aucune autre ne fait jouer : simulés sur le poste, deux survivants sans elles, aucun avec), le jour `1` écrit « 1er », les mois en minuscules (« janvier » … « décembre », « février », « août », « décembre » accentués). `noter` : `texte.trim().is_empty()` refuse ; le texte se range tel qu'il est reçu. `.cargo/mutants.toml` : `gestes.rs` et `"--test", "gestes"`, les deux mentions du nombre de modules (:2, « Quinze modules sont examinés » ; :22, « ces quinze modules »), la liste qui nomme « les gestes ». `changement.rs` ne porte que des types : il n'entre pas sous mutation.
- [ ] **Étape 4 : voir le vert.** Pousser. Attendu : `gestes.rs` `ok. 5 passed` dans `verifier`.
- [ ] **Étape 5 : committer** par chemins (`feat: les changements, le fonds lu, la naissance et noter, au noyau`).

### Tâche 3 : la crate `commandes`

**Fichiers :** créer `crates/commandes/Cargo.toml`, `crates/commandes/src/lib.rs`,
`crates/commandes/tests/serialisation.rs` ; modifier `Cargo.toml` (membres), `Cargo.lock` (de l'artefact),
`.github/workflows/ci.yml` (:99 et :112, `-p commandes`), `CLAUDE.md` (« Les crates », la puce de `commandes`).

`Cargo.toml` : `serde = { version = "1.0.229", features = ["derive"] }` ; en test, `serde_json = "1.0.154"`.

**Interfaces — produit** (`commandes`, chaque type `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]`) :

```rust
/// Les noms des commandes de la tranche 2 ; `exporter_cahier` les rejoint en tranche 3 (décision 11).
pub const NOMS: &[&str] = &["noter", "completer", "poser_une_etiquette", "oter_une_etiquette", "relier",
    "delier", "regrouper", "retirer", "retablir", "annuler", "vue_d_une_idee", "idees", "journal"];
/// Le plus grand entier que `JSON.parse` relit à l'identique : 2⁵³ − 1. Le noyau, qui ne voit pas
/// cette crate, le répète (`noyau::donnees::gestes::ENTIER_SUR`, tâche 10) : un test de chacune le
/// compare à `(1 << 53) - 1`.
pub const ENTIER_SUR: u64 = 9_007_199_254_740_991;
pub struct Noter { pub texte: String, pub decalage_minutes: i32 }
pub struct Completer { pub idee: String, pub texte: String, pub decalage_minutes: i32 }
pub struct Etiqueter { pub idee: String, pub etiquette: String, pub decalage_minutes: i32 }
pub struct Relier { pub a: String, pub b: String, pub decalage_minutes: i32 }
pub enum Sorte { Prise, MidiVerseSeul, TexteVerse }
pub struct Captation { pub sorte: Sorte, pub empreinte: String }
pub struct Regrouper { pub captation: Captation, pub cible: String, pub decalage_minutes: i32 }
/// Retirer ou rétablir l'idée `idee`.
pub struct Viser { pub idee: String, pub decalage_minutes: i32 }
pub enum Appareil { Etabli, Carnet }
pub enum Provenance { Captee { par: Appareil }, Importee { par: Appareil, depuis: String } }
pub struct Notee { pub idee: String, pub avertissement: Option<String> }
pub struct Faite { pub avertissement: Option<String> }
pub enum Raison { Impossible, RienAAnnuler, Illisible, VersionPlusRecente { version: u64 }, Disque }
pub struct Refus { pub raison: Raison, pub message: String }
pub type Reponse<T> = Result<T, Refus>;
pub struct VueDUneIdee {
    pub idee: String,
    pub provenance: Provenance,
    pub le: i64,
    pub texte: Option<String>,
    /// Dans l'ordre de leurs octets.
    pub etiquettes: Vec<String>,
    /// L'autre bout de chaque lien, dans l'ordre de leurs octets.
    pub liens: Vec<String>,
    /// Dans l'ordre (sorte, empreinte).
    pub captations: Vec<Captation>,
    pub reliee: bool,
    pub duree_ms: Option<u64>,
    pub retiree: Option<i64>,
}
pub struct EntreeDuJournal { pub message: String, pub heure: i64 }
```

**Les entiers d'une réponse, et leur source bornée** (Éc-14). Chacun vient d'un producteur qui le tient sous
`ENTIER_SUR`, et la doc de son champ le nomme :

- `VueDUneIdee::le`, `VueDUneIdee::retiree` et `EntreeDuJournal::heure` : une heure en secondes, celle d'une
  opération (`Operation::heure`), qui ne vient que du `Contexte` (l'horloge de l'établi dès la tranche 3, en
  secondes depuis 1970, quelque 1,8 · 10⁹ en 2026 ; dans les tests, `HEURE` à quelques heures près, et le
  1er janvier 2100 au plus), de la genèse (0) ou d'une migration (son heure, écrite d'avance ; 0 pour celle de
  test). Une réplique reçue n'en porte pas d'autres : les mêmes opérations les ont écrites. 2⁵³ secondes sont à
  quelque 285 millions d'années de 1970 ;
- `VueDUneIdee::duree_ms` : la plus longue des durées des prises (`idee::duree`), que seul `gestes::verser` écrit,
  sous `<e>/duree`, et qui refuse une durée au-delà d'`ENTIER_SUR` (`verser_une_duree_au_dela_d_entier_sur_est_refusee`,
  tâche 10) ;
- `Raison::VersionPlusRecente { version }` : la version de schéma d'un document, que seules les migrations
  écrivent, chacune d'un de plus (`version::connue` : 1 + le nombre de migrations), bornée par le nombre de
  migrations que Pupitre publiera ;
- les `decalage_minutes`, que l'interface envoie : des `i32`, bornés par leur type.

- [ ] **Étape 1 : écrire les tests** (`tests/serialisation.rs`) : `chaque_type_fait_l_aller_retour_par_serde_json`, qui sérialise puis relit un exemple de chaque type (de `Noter { texte: "Un refrain qui monte, en ré", decalage_minutes: 60 }` à `Refus { raison: Raison::VersionPlusRecente { version: 2 }, message: … }`) et compare ; `les_noms_sont_uniques_et_en_snake_case` (treize noms, aucun double, chacun `[a-z_]+`) ; `les_entiers_d_une_reponse_restent_sous_2_puissance_53` : une `VueDUneIdee` aux valeurs extrêmes que la tranche écrit (`le: 4_102_444_800`, le 1er janvier 2100 ; `duree_ms: Some(86_400_000)` ; `retiree: Some(4_102_444_800)`) et un `EntreeDuJournal { heure: 4_102_444_800 }` se sérialisent en nombres sans `.` ni `e`, chacun `<= ENTIER_SUR`, et `ENTIER_SUR == (1u64 << 53) - 1`. Ce test éprouve la forme des nombres écrits, sur des valeurs qu'il choisit ; la borne, ce sont les producteurs nommés plus haut, et le seul entier qui vient d'une fiche, la durée, a son test de refus à la tâche 10.
- [ ] **Étape 2 : voir le rouge.** Membres du `Cargo.toml` : `"crates/commandes"`, après `"crates/synchro"` ; `ci.yml`, `-p commandes` au lint et aux tests de `verifier` ; `crates/commandes/Cargo.toml`, avec ses dépendances, et un `src/lib.rs` vide (sans eux, `cargo update --workspace` ne charge pas le membre, et `verif-faits.mjs` s'arrête) ; dans « Les crates » de `CLAUDE.md`, après `synchro`, la puce « `commandes` (`crates/commandes`) : les types des commandes et la liste de leurs noms, que l'interface et l'établi partagent ; » (le point 6 de `verif-faits.mjs` compare cette liste aux membres). `node scripts/verif-faits.mjs` vert après le `git add`. Committer (`test: les types des commandes`), pousser. Attendu : « Cargo.lock à jour » rouge ; l'artefact pris, relu (`commandes` et ses dépendances déjà présentes), committé ; puis « Lint » rouge, `unresolved import` sur `commandes::Noter`.
- [ ] **Étape 3 : écrire le code** ; la doc de la crate dit qu'elle ne porte que ce que l'interface envoie et reçoit (C7), et qu'aucun type n'a de flottant (Éc-14).
- [ ] **Étape 4 : voir le vert.** Attendu : `serialisation.rs` `ok. 3 passed` dans `verifier`.
- [ ] **Étape 5 : committer** par chemins (`feat: la crate commandes, ses types et la liste des noms`).

### Tâche 4 : la crate `services` : les contrats, l'application des changements, la lecture, la paire en mémoire

**Fichiers :** créer `crates/services/Cargo.toml`, `src/lib.rs`, `src/contrats.rs`, `src/application.rs`,
`src/lecture.rs`, `src/en_memoire.rs`, `tests/application.rs` ; modifier `Cargo.toml` (membres),
`Cargo.lock`, `.github/workflows/ci.yml` (:99 et :112, `-p services` ; après « Lint du stockage, sans pannes », :104,
l'étape « Lint des services, sans en-memoire », `cargo clippy -p services --lib -- -D warnings`, avec le commentaire
de l'étape du stockage), `CLAUDE.md` (« Les crates », la puce de `services`).

`Cargo.toml` : `automerge = "0.12.0"`, `commandes = { path = "../commandes" }`, `noyau = { path = "../noyau" }` ;
`[features] en-memoire = []`, commenté comme `pannes` (« ses tests et ceux du stockage la demandent ; l'établi,
jamais (décision 7) ») ; en test, `services = { path = ".", features = ["en-memoire"] }`.

**Interfaces — produit :**

```rust
// services::contrats. Au lib.rs : pub mod application; pub mod lecture;
// #[cfg(feature = "en-memoire")] pub mod en_memoire; mod contrats;
// pub use contrats::{Alea, Captations, Documents, Ecriture}; pub use commandes::{Raison, Refus};
pub type Ecriture = Result<Option<String>, Refus>; // l'avertissement, en français
pub trait Documents {
    /// Le fonds tel qu'il se lit ; `None` tant qu'aucun fonds n'est né.
    fn fonds(&self) -> Option<&Automerge>;
    /// Pourquoi le fonds n'écrit pas ; `None` s'il écrit.
    fn lecture_seule(&self) -> Option<Refus>;
    /// La genèse, `naissance`, puis `geste`, en un premier fichier complet ; refusé, rien ne reste.
    fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture;
    /// `operation`, en une modification atomique, avec son message et son heure.
    fn appliquer(&mut self, operation: &Operation) -> Ecriture;
}
pub trait Captations {
    fn verser(&self, octets: &[u8]) -> Result<String, Refus>;
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus>;
}
pub trait Alea { fn remplir(&mut self, octets: &mut [u8]); }

// services::application — le seul code livré, hors du stockage, qui ouvre une transaction (I28).
#[derive(Debug)]
pub enum Inapplicable { ObjetAbsent { chemin: Vec<String> }, Vide, Automerge(AutomergeError) }
/// Une transaction : chaque changement, dans l'ordre, puis `commit_with(message, heure)`. Un objet
/// absent sur le chemin annule la transaction (`rollback`) ; une modification vide rend `Vide`.
pub fn appliquer(doc: &mut Automerge, operation: &Operation) -> Result<ChangeHash, Inapplicable>;
/// Un document neuf (son acteur, tiré par Automerge), la genèse (`GENESE`, par `apply_changes`),
/// puis `naissance`.
pub fn naitre(naissance: &Operation) -> Result<Automerge, Inapplicable>;
/// `naitre`, puis `geste`.
pub fn faire_naitre(naissance: &Operation, geste: &Operation) -> Result<Automerge, Inapplicable>;
/// Les octets de chaque migration (`Change::try_from`, puis `apply_changes`).
pub fn migrer(doc: &mut Automerge, a_appliquer: &[Migration<'_>]) -> Result<(), AutomergeError>;

// services::lecture — le fonds lu, que le noyau reçoit (C5).
pub fn lire_l_idee(doc: &Automerge, id: &str) -> Option<IdeeLue>;
pub fn lire_la_captation(doc: &Automerge, sorte: Sorte, empreinte: &str) -> Option<CaptationLue>;
/// Les captations dont `<empreinte>/idee` vaut `id`, dans les trois conteneurs, ordre (sorte, empreinte).
pub fn captations_de(doc: &Automerge, id: &str) -> Vec<CaptationLue>;
/// Chaque idée qui se lit, retirée comprise, dans l'ordre des identifiants.
pub fn idees(doc: &Automerge) -> Vec<IdeeLue>;
/// (message, heure) de chaque modification, dans l'ordre de `get_changes(&[])`.
pub fn journal(doc: &Automerge) -> Vec<(String, i64)>;
pub fn version_de_schema(doc: &Automerge) -> Option<u64>;

// services::en_memoire, sous `en-memoire`.
pub struct DocumentsEnMemoire { /* automerge: Option<Automerge>, lecture_seule: Option<Refus> */ }
impl DocumentsEnMemoire {
    pub fn neuf() -> Self;
    /// Ouvre les octets d'un fonds selon `migrations` : `version::lecture(version_de_schema, connue)`
    /// décide ; `Lecture::Migrer` applique `a_appliquer` par `application::migrer` ;
    /// `Lecture::LireSansEcrire` met en lecture seule (`Raison::VersionPlusRecente { version }`) ;
    /// `Lecture::Illisible` aussi (`Raison::Illisible`). Des octets qu'`Automerge::load` refuse :
    /// `Err`, `Raison::Illisible`.
    pub fn de(octets: &[u8], migrations: &[Migration<'_>]) -> Result<Self, Refus>;
    /// `save()` du fonds ; `None` sans fonds.
    pub fn octets(&self) -> Option<Vec<u8>>;
}
impl Documents for DocumentsEnMemoire { /* … */ }
#[derive(Default)]
pub struct CaptationsEnMemoire { /* objets: RefCell<BTreeMap<String, Vec<u8>>> */ }
impl Captations for CaptationsEnMemoire { /* verser : empreinte (noyau), une seule fois ; relire ; absente, Impossible */ }
```

`lire_l_idee` rend `None` quand l'idée manque, ou qu'une de ses valeurs n'a pas le type du schéma (le test compare
la variante, `ScalarValue::Int(le)`, jamais `as_i64`) : une idée illisible n'est pas une idée. Les liens d'une idée
sont les clés de `LIENS` dont un bout est elle (`apparente:<a>:<b>`, coupée aux deux premiers `:`). Un conteneur
absent de la racine du document (`liens`, `midi_verses`, `textes_verses`, que seule la genèse de la tâche 8 crée) se
lit vide : l'idée reste lisible, et `captations_de` le saute.
`captations_de` parcourt chaque conteneur par `map_range`, en un passage, sans `get` par clé. Le refus d'une
`Inapplicable` est `Raison::Impossible`, son message le `Debug` de l'erreur jusqu'à la tâche 16.
`CaptationsEnMemoire::relire` d'une empreinte qu'elle ne garde pas rend `Raison::Impossible`, « Aucune captation ne
porte l'empreinte {e}. », la raison que rend le stockage devant une captation absente (tâche 6) ; les deux paires
ne donnent le même texte qu'à partir de la tâche 16, le stockage rendant le `Debug` de son erreur jusque-là.

- [ ] **Étape 1 : écrire les tests** (`tests/application.rs`, sur `Automerge` directement) : `appliquer_pose_cree_et_ote` (sur `faire_naitre(&naissance(…), &noter(I, "Un refrain", Appareil::Etabli, HEURE, 0))`, puis une opération qui pose `idees/I/etiquettes/pont` = `Vrai`, une opération à trois changements : `Poser [idees, I, texte]` = `Chaine("Un refrain qui descend")`, `Creer { chemin: [idees, J], conteneurs: [etiquettes] }`, `Oter [idees, I, etiquettes, pont]` ; alors `lire_l_idee(&doc, I)` rend `IdeeLue { id: I, provenance: Captee { par: Etabli }, le: HEURE, texte: Some("Un refrain qui descend"), etiquettes: {}, retiree: None, liens: {} }`, `lire_l_idee(&doc, J)` rend `None` (ni `le` ni `provenance`), et `idees/J` est un objet à clés qui porte `etiquettes`) ; `un_objet_absent_annule_toute_l_operation` (`Poser` sous `idees/absente` après un `Poser` valide : `ObjetAbsent { chemin: ["idees", "absente"] }`, et les heads n'ont pas bougé) ; `une_operation_sans_effet_est_inapplicable` (`Oter` d'une clé absente : `Vide`, heads inchangés) ; `l_operation_porte_son_message_et_son_heure` (`journal` rend `("Idée du 14 novembre notée", HEURE)` en dernier) ; `la_genese_puis_la_naissance` (`naitre` : `journal` rend `[("Genèse du fonds", 0), ("Fonds créé", HEURE)]`, `version_de_schema` 1) ; `une_valeur_d_un_autre_type_rend_l_idee_illisible` (`le` posé en `Naturel` : `lire_l_idee` rend `None`) ; `la_paire_en_memoire_migre_a_son_ouverture` (point de revue 1 : `DocumentsEnMemoire::de(octets, &migrations_de_test(&migration_de_test()))` porte la version de schéma 2 et le conteneur de la migration ; rouvert de ses octets avec `MIGRATIONS`, `lecture_seule()` rend `Raison::VersionPlusRecente { version: 2 }`) ; `une_captation_versee_deux_fois_rend_la_meme_empreinte` (les objets de `CaptationsEnMemoire` sont privés : ce qui s'observe, c'est la même empreinte rendue deux fois, celle d'`empreinte(octets)`, et `relire` qui rend les octets ; `relire` d'une autre empreinte rend `Raison::Impossible`). La migration de test se recopie de `crates/stockage/tests/commun/mod.rs:264-303` (`CONTENEUR_DE_LA_MIGRATION`, :267, que `migration_de_test` emploie, puis `migration_de_test` et `migrations_de_test`) dans `tests/application.rs`, puis, à la tâche 5, dans la suite.
- [ ] **Étape 2 : voir le rouge.** Membre `"crates/services"`, après `"crates/commandes"`, `-p services` au lint et aux tests, l'étape sans `en-memoire` ; `crates/services/Cargo.toml`, avec ses dépendances, sa fonction et sa dépendance de test, et un `src/lib.rs` vide ; dans « Les crates » de `CLAUDE.md`, après `commandes`, la puce « `services` (`crates/services`) : les opérations nommées et les lectures du fonds, conduites sur les contrats `Documents` et `Captations` ; sa fonction de Cargo `en-memoire`, que seuls les tests activent, compile la paire d'adaptateurs en mémoire ; ». `node scripts/verif-faits.mjs` vert après le `git add`. Committer (`test: l'application des changements et la paire en mémoire`), pousser ; l'artefact `cargo-lock` relu et committé ; attendu à « Lint » : `unresolved import services::application`.
- [ ] **Étape 3 : écrire le code.** `appliquer` : `let mut tx = doc.transaction();` ; pour chaque changement, le parent se trouve depuis `ROOT` par `tx.get(…)`, chaque pas un `Value::Object(ObjType::Map)`, sinon `tx.rollback()` et `ObjetAbsent { chemin: <le préfixe> }` ; `Poser` par `tx.put` (`Simple::Chaine` en `ScalarValue::Str`, `Entier` en `Int`, `Naturel` en `Uint`, `Vrai` en `Boolean(true)`) ; `Creer` par `tx.put_object(…, ObjType::Map)`, puis chaque conteneur ; `Oter` par `tx.delete` ; puis `tx.commit_with(CommitOptions::default().with_message(…).with_time(heure))`, et `None` rend `Vide`. La doc du module dit qu'il est le code partagé des deux adaptateurs, et le seul de `services` qui écrit (C31).
- [ ] **Étape 4 : voir le vert.** Attendu : `application.rs` `ok. 8 passed` ; « Lint des services, sans en-memoire » vert.
- [ ] **Étape 5 : committer** par chemins (`feat: la crate services, ses contrats et sa paire en mémoire`).

### Tâche 5 : les services : noter, la vue d'une idée, le journal, et la suite des commandes

**Fichiers :** modifier `crates/services/src/lib.rs` ; créer `crates/services/tests/suite/mod.rs`,
`crates/services/tests/commandes_en_memoire.rs`.

**Interfaces — produit** (`services`) :

```rust
pub struct Contexte<'a> { pub heure: i64, pub alea: &'a mut dyn Alea }
pub struct Services<D: Documents, C: Captations> { /* documents, captations, appareil ; dernier_geste: Option<GesteDeTri>, dès la tâche 11 */ }
impl<D: Documents, C: Captations> Services<D, C> {
    pub fn new(documents: D, captations: C, appareil: noyau::donnees::gestes::Appareil) -> Self;
    pub fn documents(&self) -> &D;
    pub fn documents_mut(&mut self) -> &mut D;
    pub fn captations(&self) -> &C;
    pub fn en_parties(self) -> (D, C);
    /// Sans fonds, la naissance : l'ULID du document, puis celui de la lignée (dix octets chacun),
    /// l'identifiant de propriétaire (seize), l'ULID de l'idée (dix), dans cet ordre ; puis
    /// `Documents::faire_naitre`. Avec un fonds, l'ULID de l'idée, puis `Documents::appliquer`.
    /// Ne touche pas au dernier geste de tri.
    pub fn noter(&mut self, commande: &commandes::Noter, contexte: &mut Contexte<'_>) -> Reponse<Notee>;
    pub fn vue_d_une_idee(&self, idee: &str) -> Reponse<VueDUneIdee>;
    pub fn journal(&self) -> Reponse<Vec<EntreeDuJournal>>;
}
/// La seule fonction qui forme l'identifiant de propriétaire (§ 30 · 2 ; C24) : seize octets
/// d'aléa, puis `uuid_v4`.
fn identifiant_de_proprietaire(alea: &mut dyn Alea) -> String;
```

Chaque commande qui écrit (noter, les gestes de tri, annuler ; `verser_une_fiche` pour sa fiche, la captation
versée d'abord) commence par `lecture_seule()` : `Some(refus)` se rend tel quel, rien n'est tiré. Les lectures
(`vue_d_une_idee`, `idees`, `journal`) ne le consultent pas : elles rendent ce qui se lit, en lecture seule comme
ailleurs (§ 31 · 2.4 : « elle lit sans écrire. Elle montre ce qu'elle connaît »). Un fonds absent
rend, pour `vue_d_une_idee`, `Raison::Impossible` (« Aucune idée ne porte l'identifiant … », la variante
`IdeeAbsente` de la tâche 10 ; jusque-là, le même texte écrit dans `services`), et pour `journal`, une liste vide.
La vue tire `reliee` et `duree_ms` des règles de la tâche 9, à la tâche 11 ; jusque-là, `reliee: false` et `duree_ms: None`, que
rien ne peut démentir en (a), aucun lien ni aucune prise ne s'écrivant. CAP-01 · 6 s'inscrit donc en (a) (tâche 7),
entier : son geste, noter à l'établi, est la commande `noter`, livrée ici, et chacun de ses faits (datée, `captee`,
née sur l'établi, sans prise ni durée) se lit dans la vue ; l'écran de la tranche 3 appelle la même commande, et le
critère ne dit rien de lui.

Deux familles de types portent les mêmes noms : `Appareil`, `Provenance` et `Sorte` de `noyau::donnees::gestes`,
que reçoivent les services (`new`, `verser_une_fiche`), et ceux de `commandes`, que rend une vue. Les signatures et
la suite les écrivent qualifiés, `gestes::Appareil` (`use noyau::donnees::gestes;`) et `commandes::Appareil`, sans
`use` qui en importe un seul.

**La suite** (`tests/suite/mod.rs`, `#![allow(dead_code)]`, un fichier inclus par deux crates) :

```rust
pub const HEURE: i64 = 1_700_000_000;
/// L'aléa des tests : à chaque appel, l'octet 0 vaut `graine`, les huit derniers le rang de
/// l'appel, en gros-boutiste, compté de 1 ; le reste, 0. Chaque réplique d'un test a sa graine.
pub struct AleaDeTest { pub graine: u8, pub appels: u64 }
impl Alea for AleaDeTest { /* … */ }
/// Ce que la suite demande à une paire d'adaptateurs.
pub trait Paire {
    type D: Documents;
    type C: Captations;
    /// Des services sur une racine sans document (fichiers), ou sans fonds (mémoire).
    fn neufs(&mut self) -> Services<Self::D, Self::C>;
    /// Ferme `services`, puis les rouvre selon `migrations`.
    fn rouvrir(&mut self, services: Services<Self::D, Self::C>, migrations: &[Migration<'_>]) -> Services<Self::D, Self::C>;
}
pub fn migration_de_test() -> Vec<u8>;                      // recopiée de la tâche 4
pub fn migrations_de_test(octets: &[u8]) -> [Migration<'_>; 1];
```

Les ULID d'une naissance sous `AleaDeTest { graine: 0, appels: 0 }`, à `HEURE` : document
`01HF7YAT000000000000000001`, lignée `01HF7YAT000000000000000002`, propriétaire
`00000000-0000-4000-8000-000000000003`, idée `01HF7YAT000000000000000004` ; sous la graine 1, l'idée est
`01HF7YAT000400000000000004`. La suite porte aussi `IMPORT = "01HF7YAT00ZZZZZZZZZZZZZZZZ"` (`ulid(HEURE, [0xFF; 10])`),
l'identifiant d'import des fiches que versent ses tests (tâche 11).

- [ ] **Étape 1 : écrire les tests.** Dans la suite, des fonctions génériques sur `P: Paire`, jamais nommées d'après un critère (`scripts/verif-faits.mjs:643-656`) :
  - `une_idee_notee_est_captee_datee_sans_prise_ni_duree` : `noter("Un refrain qui monte, en ré", 0)` rend `Notee { idee: "01HF7YAT000000000000000004", avertissement: None }` ; la vue : `provenance: commandes::Provenance::Captee { par: commandes::Appareil::Etabli }`, `le: HEURE`, `texte: Some(…)`, `captations: []`, `duree_ms: None`, `retiree: None` ; rouverte (`rouvrir(…, &[])`), la même vue ;
  - `noter_fait_naitre_le_fonds_en_trois_modifications` : `journal` rend `[("Genèse du fonds", 0), ("Fonds créé", HEURE), ("Idée du 14 novembre notée", HEURE)]` ; une seconde note y ajoute une entrée, l'idée `…05` ;
  - `une_note_vide_refusee_ne_fait_rien_naitre` : `Refus { raison: Impossible, message: "Le texte de la note est vide." }`, puis `documents().fonds()` est `None` ;
  - `une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme` (`// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que la version de schéma N`) : noter, rouvrir selon `migrations_de_test`, puis selon `&[]` ; `journal()` et `vue_d_une_idee(<l'idée notée>)` répondent, ce qui se lit montré ; noter rend `Raison::VersionPlusRecente { version: 2 }`, et `journal()` rend ensuite la même liste qu'avant la note refusée ; `captations().verser(b"une captation devant un fonds plus recent")` rend son empreinte, que `relire` rend (la tâche 11 y ajoute `verser_une_fiche` et `idees()`).

  Dans `commandes_en_memoire.rs` : `mod suite;`, `struct PaireEnMemoire;` (`neufs` : `DocumentsEnMemoire::neuf()`, `CaptationsEnMemoire::default()`, `gestes::Appareil::Etabli` ; `rouvrir` : `en_parties`, puis `DocumentsEnMemoire::de(&octets, migrations)` ou `neuf()` sans octets, les captations gardées), et une fonction `#[test]` par fonction de la suite, du même nom, sauf celle de CAP-01 · 6 :

```rust
#[test]
fn cap01_6_une_idee_notee_a_l_etabli_est_captee_datee_sans_prise_ni_duree() {
    suite::une_idee_notee_est_captee_datee_sans_prise_ni_duree(&mut PaireEnMemoire);
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: la suite des commandes, sur la paire en mémoire`), pousser. Attendu à « Lint » : l'erreur qui nomme
  ce qui manque, `unresolved import services::Services` (ou `cannot find type Services`) : `Services` et `Contexte`
  naissent à l'étape 3.
- [ ] **Étape 3 : écrire le code.** `noter` : `lecture_seule()` ; puis, sans fonds, les quatre tirages dans l'ordre, `gestes::naissance` et `gestes::noter`, `faire_naitre` ; avec un fonds, un tirage, `gestes::noter`, `appliquer`. Les conversions entre les types du noyau et ceux de `commandes` (`Appareil`, `Provenance`, `Sorte`) sont des fonctions privées de `lib.rs`.
- [ ] **Étape 4 : voir le vert.** Attendu : `commandes_en_memoire.rs` `ok. 4 passed` dans `verifier`.
- [ ] **Étape 5 : committer** par chemins (`feat: noter, la vue d'une idée et le journal, conduits par les services`).

### Tâche 6 : `stockage` réalise les contrats : `appliquer`, `faire_naitre`, le doctest, la migration, la suite sur fichiers

**Fichiers :** créer `crates/stockage/src/adaptateurs.rs`, `crates/stockage/tests/commandes_sur_fichiers.rs` ;
modifier `crates/stockage/Cargo.toml` (`services = { path = "../services" }` ; en test, `services = { path =
"../services", features = ["en-memoire"] }` et `commandes = { path = "../commandes" }`, que la suite partagée
nomme : un test du stockage ne voit que ses dépendances et ses dépendances de test ; le commentaire de `pannes`,
:14-16, dit aussi ce qu'elle compile de plus), `src/lib.rs` (`Naissance` et sa réexportation sous `pannes`, `mod
adaptateurs` et `pub use adaptateurs::FondsSurFichiers` ; la doc de la crate, :6-9, de même que le commentaire de
`pannes`), `src/documents.rs` (:206-268 ; :393-410 ; :960-1072),
`src/racine_des_donnees.rs` (:186-208), `app/ui/Cargo.toml` (`commandes = { path = "../../crates/commandes" }`),
`Cargo.lock`.

**Interfaces — produit** (`stockage`) :

```rust
impl Document<'_> {
    /// § 31 · 2.1 : `operation`, appliquée par `services::application::appliquer`, faite une fois son
    /// fichier d'ajout en place ; les trois issues de la tranche 1 (faite, faite avec un
    /// avertissement, refusée et relue du disque). Une opération inapplicable : `Erreur::Inapplicable`,
    /// et rien ne s'écrit.
    pub fn appliquer(&mut self, operation: &Operation) -> Result<Option<Avertissement>, Erreur>;
    #[cfg(feature = "pannes")]
    pub fn ecrire(&mut self, message: &str, heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>) -> Result<Option<Avertissement>, Erreur>;
}
impl RacineDesDonnees {
    /// § 31 · 2.3 : `services::application::faire_naitre(naissance, geste)`, en un seul fichier
    /// complet ; la garde, `pupitre_neuf` et les refus de `faire_naitre_le_fonds`, inchangés.
    pub fn faire_naitre(&self, naissance: &Operation, geste: &Operation) -> Result<Document<'_>, Erreur>;
    #[cfg(feature = "pannes")]
    pub fn faire_naitre_le_fonds(&self, naissance: &Naissance<'_>, message: &str, heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>) -> Result<Document<'_>, Erreur>;
}
// Erreur gagne `Inapplicable(services::application::Inapplicable)`.
// stockage::adaptateurs
pub struct FondsSurFichiers<'r> { /* racine: &'r RacineDesDonnees, document: Option<Document<'r>> */ }
impl<'r> FondsSurFichiers<'r> {
    /// `racine.fonds()` : le document, ou aucun ; une erreur d'ouverture se rend en `Refus`.
    pub fn ouvrir(racine: &'r RacineDesDonnees) -> Result<Self, Refus>;
    #[cfg(feature = "pannes")]
    pub fn ouvrir_selon(racine: &'r RacineDesDonnees, migrations: &[Migration<'_>]) -> Result<Self, Refus>;
    pub fn document(&self) -> Option<&Document<'r>>;
    /// Pour les tests à plusieurs répliques, qui réunissent deux documents (`commun::reunir`).
    pub fn document_mut(&mut self) -> Option<&mut Document<'r>>;
}
impl services::Documents for FondsSurFichiers<'_> { /* … */ }
impl services::Captations for Captations<'_> { /* verser, relire ; une Erreur en Refus */ }
```

Le partage : un `fn naitre<'r>(racine, construire: impl FnOnce() -> Result<Automerge, Erreur>)` privé porte la
garde, `pupitre_neuf` et l'écriture du fichier complet, appelé par `faire_naitre` (livrée) et par
`faire_naitre_le_fonds` (sous `pannes`, qui construit par `application::naitre(&gestes::naissance(…))` puis sa
fermeture). `ecrire` (sous `pannes`) et `appliquer` (livrée) appellent `ecrire_l_ajout`. `migrer` applique par
`services::application::migrer`, puis écrit comme avant. Une fonction privée qui ne servirait que sous `pannes`
passe sous `#[cfg(feature = "pannes")]` : l'étape « Lint du stockage, sans pannes » refuse le code mort. De même
d'un import qui ne sert plus que sous `pannes` : il prend `#[cfg(feature = "pannes")]` sur son `use`, ou s'écrit
par son chemin complet dans la fonction gardée ; et un import qui ne sert plus du tout se retire. La liste qui suit
n'est pas fermée, les deux lints (« Lint », avec `pannes`, et « Lint du stockage, sans pannes », tous deux en
`-D warnings`) disent le reste : `Transaction` (`documents.rs:29`, que n'emploient que `ecrire`, :255,
`faire_naitre_le_fonds`, :974, et `naitre_en_memoire`, :1043 ; `racine_des_donnees.rs:9`, pour :204) et
`CommitOptions` (`a_l_heure`, :1068), sous `pannes` ; `Naissance` et `AutomergeError` (`racine_des_donnees.rs:14`
et :8, employés aux seules :201 et :204), sous `pannes`, sans quoi l'import de `crate::Naissance` ne se résout pas
sans `pannes` ; `Transactable`, qui ne sert qu'au `tx.put` de la naissance (:1057), que `application::naitre`
reprend : retiré ; `GENESE`, `MESSAGE_DE_LA_NAISSANCE` (`documents.rs:38`) et `Change` (:31), retirés quand
`migrer` passe à `application::migrer` et que `naitre_en_memoire` part.

Les raisons d'un refus, une par variante de `stockage::Erreur` (`crates/stockage/src/lib.rs:38-75`, et
`Inapplicable`, que la tâche ajoute), dans un `match` sans `_` :

- `LectureSeule(LectureSeule::VersionPlusRecente { version })` → `Raison::VersionPlusRecente { version }` ; toute
  autre `LectureSeule` → `Illisible` ;
- `Disque` et `CaptationAlteree` → `Disque` ;
- `Inapplicable` → `Impossible` ; `CaptationAbsente` (`relire` d'une empreinte qu'aucun fichier ne porte) →
  `Impossible`, comme `CaptationsEnMemoire::relire` (tâche 4) ; `FondsDejaNe` (`faire_naitre` sur une racine des
  données qui n'est pas neuve, selon `pupitre_neuf`) → `Impossible` ;
- `Automerge` → `Illisible` ;
- celles qui ne surviennent qu'avant `Services`, ou hors de ses commandes, prennent une raison pour que le `match`
  reste complet, sans test de la suite : `RacineDesDonneesTenue` (`RacineDesDonnees::ouvrir`, avant tout
  `FondsSurFichiers`) et `FondsDejaOuvert` (`racine.fonds()`, dans `FondsSurFichiers::ouvrir`) → `Impossible` ;
  `CompactionInfidele` (que seul `Document::compacter`, appelé seul, rend, et que `Services` n'appelle pas ; après
  une écriture, la compaction échouée est un avertissement) → `Disque` ; `ChampAbsent` et `Synchro` (situer une
  copie, recevoir un message, § 31 · 2.5 : la synchronisation, que `Services` ne conduit pas) → `Illisible`.

Le message, le `Debug` de l'erreur jusqu'à la tâche 16.

- [ ] **Étape 1 : écrire les tests.**
  - Le doctest déplacé de `ecrire` (:231-250) sur `appliquer`, le marqueur au-dessus ; il se pose à l'étape 3, avec
    la méthode, qui n'existe pas avant (`clippy --all-targets` ne compile aucun doctest) :

```rust
// § 31 · 10, test 8 : une synchronisation lancée pendant l'écriture ne voit pas l'opération en cours
/// ```compile_fail,E0502
/// fn pendant(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange,
///            operation: &noyau::donnees::changement::Operation) {
///     let lu = document.automerge();
///     let _ = document.appliquer(operation);
///     let _ = echange.prochain_message(lu);
/// }
/// ```
///
/// ```
/// fn apres(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange,
///          operation: &noyau::donnees::changement::Operation) {
///     let _ = document.appliquer(operation);
///     let _ = document.prochain_message(echange);
/// }
/// ```
```

  - `tests/commandes_sur_fichiers.rs` : `#[path = "../../services/tests/suite/mod.rs"] mod suite;` (de même, à la tâche 13, `repliques.rs` et `references.rs`, qui prennent `AleaDeTest`, `HEURE` et `IMPORT` de la suite), `mod commun;`, `struct PaireSurFichiers<'r> { racine: &'r RacineDesDonnees }` (`neufs` : `FondsSurFichiers::ouvrir`, `racine.captations()`, `gestes::Appareil::Etabli` ; `rouvrir` : `drop(services)`, puis `ouvrir_selon(racine, migrations)`), chacun de ses tests prenant sa racine par `commun::dossier_jetable` ; une enveloppe `#[test]` par fonction de la suite, comme en mémoire, `cap01_6_…` comprise ; et, sur cette paire seule :

```rust
// § 31 · 10, test 8 : interrompre une écriture à chaque étape (fichier voisin écrit, synchronisé, renommé), puis rouvrir le document
#[test]
fn une_commande_coupee_avant_le_renommage_est_refusee() {
    // une note faite ; puis, sous couper(Etape::Renommage, 1), une seconde : Refus { raison: Disque, .. } ;
    // rouverte, la vue de l'idée …05 rend Impossible, et le journal a trois entrées.
}
// § 31 · 10, test 8 : interrompre une écriture à chaque étape (fichier voisin écrit, synchronisé, renommé), puis rouvrir le document
#[test]
fn une_commande_coupee_apres_le_renommage_est_faite_avec_un_avertissement() {
    // sous couper(Etape::RendreDurable, 1) : Notee { avertissement: Some(_), .. } ; rouverte, l'idée …05 se lit.
}
#[test]
fn le_premier_geste_refuse_laisse_la_racine_des_donnees_sans_document() {
    // sous couper(Etape::Renommage, 1), la première note : Refus { raison: Disque, .. } ;
    // `documents/fonds/complet.automerge` absent, et rouverte, documents().fonds() est None.
}
```

  - `tests/version.rs:699-712` : `faire_naitre(&racine)` de `commun` reste ; la PR le dit, la porte suivant la naissance par `faire_naitre_le_fonds`, sous `pannes`, qui partage `naitre` avec `faire_naitre`.
- [ ] **Étape 2 : voir le rouge.** Committer le fichier de tests avec les dépendances de test du stockage (`services` avec `en-memoire`, et
  `commandes`) (`test: la suite des commandes sur la paire de fichiers`), pousser. Attendu : « Cargo.lock à jour »
  rouge, l'artefact `cargo-lock` relu et committé ; puis, à « Lint », `unresolved import stockage::FondsSurFichiers`.
- [ ] **Étape 3 : écrire le code**, et le doctest sur `appliquer` ; `app/ui/Cargo.toml` gagne `commandes`, que l'interface n'emploie pas encore ; l'artefact `cargo-lock` relu et committé (le stockage y gagne `services`, et `commandes` en test ; l'interface, `commandes`). Le commentaire de `pannes` et la doc de la crate : elle coupe une écriture à l'étape voulue, ouvre le fonds selon les migrations qu'un test lui donne (`fonds_selon`, `FondsSurFichiers::ouvrir_selon`), et garde pour les tests l'écriture et la naissance par une fermeture (`Document::ecrire`, `faire_naitre_le_fonds`, `Naissance`).
- [ ] **Étape 4 : voir le vert.** Attendu : `commandes_sur_fichiers.rs` `ok. 7 passed` dans `verifier` et `windows` ; `Doc-tests stockage` vert (les deux exemples de `appliquer`) ; les comptes des autres fichiers de `crates/stockage/tests` inchangés ; « Lint du stockage, sans pannes » et « Lint de l'interface (WebAssembly) » verts.
- [ ] **Étape 5 : committer** par chemins (`feat: le stockage réalise les contrats des services, et sa naissance passe au noyau`).

### Tâche 7 : les documents de la PR (a), et la mutation remesurée

**Fichiers :** modifier `docs/40-architecture.md` (§ 3, :172-174, :182-186 et :193-195), `docs/41-pile-technique.md`
(§ 6, :290-294 et :296), `docs/70-qualite-et-verification.md` (§ 4.1, le tableau des critères livrés), `CLAUDE.md`
(« Les crates » : la puce de `stockage`, :19 ; « Commandes »), `CONVENTIONS.md` (lignes `verifier` et `mutants`, :115 et
:117 ; le bloc « Les lancer avant de pousser », :138-141), `.github/workflows/ci.yml` (`mutants`, :180-185),
`.github/workflows/mutants-complet.yml`.

- [ ] **Étape 1 : le § 40 · 3, correction (a).** « Les services et les adaptateurs en mémoire se compileront en WebAssembly, avec la fonction `wasm` d'`automerge`, pour la construction navigateur, à la ligne 5 des portes ; le lot 1 ne pose aucune dépendance de cible `wasm32`. » ; les crates `commandes` et `services`, la fonction `en-memoire`. Le stockage (:182-186) « dépend du noyau, de `synchro`, de `services` et d'`automerge` » (écart 8), et sa fonction `pannes` coupe une écriture à l'étape voulue, ouvre le fonds selon les migrations qu'un test lui donne (`fonds_selon`, `ouvrir_selon`), et garde pour les tests l'écriture et la naissance par une fermeture (`Document::ecrire`, `faire_naitre_le_fonds`, `Naissance`). La recherche de LE-3 : `git grep -n -i "WebAssembly" -- docs CONVENTIONS.md README.md`, citée dans la PR ; l'ADR-022, le glossaire et le § 70 · 2 décrivent la cible et restent justes, et la PR le dit. De même `git grep -n "sha2" -- . ':!Cargo.lock'`, chaque sortie relue : l'ADR-023 (« Automerge, `sha2` et le système de fichiers suffisent au reste ») reste juste, `sha2` calculant l'empreinte au noyau, et la PR le dit.
- [ ] **Étape 2 : le § 41 · 6, correction (c).** « L'ULID d'une idée, d'une requête enregistrée, d'un document, d'une lignée et d'un import, formé au noyau par `ulid`, sans ses fonctions par défaut, de l'heure en millisecondes et de quatre-vingts bits d'aléa (dix octets) reçus ; l'UUID de l'identifiant de propriétaire, de seize octets reçus, sans crate. Aucune fonction d'horloge ni d'aléa d'`ulid` n'est compilée hors de `std` ; `web-time` entre au graphe de `wasm32-unknown-unknown` sans être appelé. `serde`, et `serde_json` lu par `JSON.parse`. » La ligne `sha2` (:296) : sa première colonne dit « Noyau », et son texte, qu'au noyau `sha2` calcule l'empreinte d'une page, et celle d'une captation, d'un fichier d'ajout et de la teneur d'un document (`identifiants::empreinte`), que le côté natif et le nœud appellent (écart 8). La PR dit que le § 40 · 7.5 (la liste des commandes) attend la tranche 3 (T5). **Le § 70 · 4.1** : la ligne `| [CAP-01](20-sfd/21-capter-et-organiser.md#cap-01--capter-en-un-geste) | 6 | INC-1, lot 1 |`, que `cap01_6_…` nomme dans les deux crates (tâches 5 et 6).
- [ ] **Étape 3 : `CLAUDE.md`.** « Les crates » porte déjà `commandes` et `services` (tâches 3 et 4) ; la puce de `stockage` (:19) dit de `pannes` ce que dit le § 40 · 3 (étape 1). « Commandes » : `-p commandes -p services` au lint et aux tests ; `cargo clippy -p services --lib -- -D warnings`. `CONVENTIONS.md`, ligne `verifier` et bloc « Les lancer » : les mêmes commandes, et « les commandes et les services » parmi ce qu'elle refuse ; ligne `mutants` : « les identifiants, les gestes » parmi les règles des données. Jamais de compte des modules dans `CONVENTIONS.md`, `CLAUDE.md` ou le README (la porte lit « neuf modules » comme ceux de la SFD). `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add`.
- [ ] **Étape 4 : remesurer la mutation, à froid**, comme à la tâche 8 de la tranche 1 : `gh workflow run mutants-complet.yml --ref t2a-commandes-et-services`, « No cache found. » dans le journal de rust-cache (sinon `gh cache delete` des clés `v0-rust-mutants` de `main` et de la branche, et relancer) ; la durée du job et « N mutants tested » ; ce passage vert, sans `MISSED` ni `TIMEOUT`, son lien dans la description ; les deux `timeout-minutes` à deux fois la durée, arrondie à la minute au-dessus, et leurs commentaires (« à froid », le compte, la durée, la date, la PR).
- [ ] **Étape 5 : committer** par chemins (`docs: les commandes et les services, au § 40 · 3, au § 41 · 6 et dans CLAUDE.md`, puis `ci: les délais de la mutation, remesurés avec les identifiants et les gestes`), pousser. **Fin de la PR (a) :** `verifier`, `windows`, `mutants` (sans `MISSED`), `contenu`, `documentation` et `texte-pr` verts ; la description dit les écarts au plan maître 4, 6 et 8, l'API livrée et les points d'entrée des tests sous `pannes` (aucun test de la tranche 1 réécrit), CAP-01 · 6 inscrit, le message des refus en `Debug` jusqu'à la PR (c), les recherches de LE-3 (`WebAssembly`, `sha2`) ; la revue de la PR entière faite et ses retours traités ; puis la relecture des corrections, sur la tête qui part, dont le SHA complet est celui de `--match-head-commit` (un commit poussé après elle la relance) ; `gh pr ready <n>`, puis la fusion depuis le clone principal (contraintes générales).

---

## PR (b) · le schéma, les gestes de tri, l'annulation

**Branche :** `t2b-schema-et-gestes`. **Titre :** `feat: le schéma de la v0.3.0, les gestes de tri et
l'annulation, décidés au noyau` (81). **Documents :** § 31 · 2 et 2.3 ; `noyau::donnees::fonds` ; § 30 · 1 et 9.3
(le lien sans date) ; § 30 · 11 et CAP-02 (décision 18) ; § 70 · 4.1 ; glossaire (Étiquette, Annuler) ;
`.cargo/mutants.toml` ; `CONVENTIONS.md` (`mutants`) ; délais. **Fin :** voir la tâche 13.

### Tâche 8 : le schéma de la `v0.3.0`, sa genèse, et les empreintes que nomme le fonds

**Fichiers :** créer `crates/noyau/src/donnees/idee.rs`, `crates/noyau/tests/idee.rs` ; modifier
`crates/noyau/src/donnees/mod.rs` (`pub mod idee;`), `crates/noyau/src/donnees/fonds.rs` (`CONTENEURS_DE_LA_GENESE`,
`GENESE`), `crates/stockage/tests/genese.rs` (:13-27, :101, :109), `crates/stockage/tests/references.rs` (:21-22),
`crates/stockage/tests/commun/mod.rs` (:631-642), `crates/stockage/src/documents.rs` (:623, :629-636),
`crates/services/src/lecture.rs`, `crates/stockage/tests/demarrage.rs`, `crates/stockage/tests/h07.rs` (:181, dans
`constater_la_reunion`, et :562, dans `une_copie_restauree_relit_chaque_captation_contre_son_empreinte` : les prises
se lisaient comme les clés de `PRISES` ; elles se lisent par `idee::empreintes_nommees`, ce seul changement, que la
PR dit).

**Interfaces — produit :**

```rust
// noyau::donnees::fonds
/// Les conteneurs de la racine du document que crée la genèse, dans l'ordre de sa recette (décision 9).
pub const CONTENEURS_DE_LA_GENESE: [&str; 9] = [IDEES, PRISES, MIDI_VERSES, TEXTES_VERSES, LIENS,
    REQUETES_ENREGISTREES, IMPORTS, ORIGINES_DECLAREES, PHOTOS];
/// Les conteneurs dont les clés nomment une captation, `<empreinte>/<fait>` (§ 31 · 6.1) ; le lot 2
/// y ajoute les photos, avec le cas du test 4.
pub const CONTENEURS_NOMMANT_UNE_CAPTATION: [&str; 3] = [PRISES, MIDI_VERSES, TEXTES_VERSES];
// noyau::donnees::idee (le fichier naît ici ; ses règles, à la tâche 9)
/// Les empreintes que nomment des clés `<empreinte>/<fait>` : ce qui précède le premier `/` ; une clé
/// sans `/` ne nomme rien.
pub fn empreintes_nommees<'a>(cles: impl IntoIterator<Item = &'a str>) -> BTreeSet<String>;
// services::lecture
/// Les clés des `CONTENEURS_NOMMANT_UNE_CAPTATION`, par `idee::empreintes_nommees`.
pub fn empreintes_nommees(doc: &Automerge) -> BTreeSet<String>;
```

- [ ] **Étape 1 : écrire les tests.** `genese_par_la_recette` crée la version de schéma puis chaque conteneur de `CONTENEURS_DE_LA_GENESE`, dans l'ordre ; `:101` attend les dix clés triées (`idees`, `imports`, `liens`, `midi_verses`, `origines_declarees`, `photos`, `prises`, `requetes_enregistrees`, `textes_verses`, `version_de_schema`) ; `:109` parcourt les neuf conteneurs. Au noyau (`tests/idee.rs`, créé ici) : `les_empreintes_nommees_se_coupent_au_premier_trait` : `["e1/idee", "e1/format", "e2/idee", "sans-trait", "e3/nom/a/b.wav"]` → `{"e1", "e2", "e3"}`. Dans `tests/demarrage.rs` : `un_midi_ou_un_texte_verse_nomme_prend_son_nom_au_demarrage` (`// § 31 · 10, test 4 : renommage perdu après la réponse : la captation nommée par un document prend son nom au démarrage`), qui pose par `commun::ecrire` la clé `<e>/idee` sous `MIDI_VERSES`, puis sous `TEXTES_VERSES`, laisse un nom provisoire de chacun, et rouvre : chacun prend son nom. Il se committe sous
`#[ignore = "la genèse de la v0.3.0, à l'étape 4"]` : sous la genèse provisoire, ses conteneurs n'existent pas, et
son échec arrêterait `cargo test` avant `genese.rs` ; l'`#[ignore]` se retire à l'étape 4, avec la genèse neuve.
Son rouge n'est pas vu, ce que la PR dit. `poser_une_prise` écrit `<e>/idee` et `<e>/format` sous `PRISES`, sans objet. Les références du test 11 : `GENESE_DU_FONDS` = `a02890acd15f12619da7f1f5abea9a8056db7188f909a467896ea50d4b5a6fc1` (calculée sur le poste, par un encodeur relu à l'octet sur la genèse d'aujourd'hui : 195 octets), et `TENEUR_DU_TEMOIN` = `6edd898e994a41036e5a4e7d42653910fcb99cb24f4f1ff42244f140b3bceabc`, le SHA-256 de la teneur du témoin de la tranche 1 avec les sept conteneurs nouveaux, vides (`{"idees":{"idee-1":{"etiquettes":{"refrain":b1},"texte":s"Un refrain qui monte, en ré"}},"identifiant":s"fonds-1","imports":{},"liens":{},"midi_verses":{},"origines_declarees":{},"photos":{},"prises":{},"proprietaire":s"proprietaire-1","requetes_enregistrees":{},"textes_verses":{},"version_de_schema":u1}`), calculé sur le poste par `python -X utf8` (le même calcul redonne `a349d135…` pour la teneur d'aujourd'hui), son commentaire récrit : `genese_et_teneur_figees` compare la genèse avant d'imprimer la teneur, qui ne paraîtrait qu'un envoi après la genèse neuve. Si la CI rend une autre teneur, c'est la sienne qui fait foi, et la PR dit l'écart au calcul.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la genèse de la v0.3.0 et les empreintes nommées`), pousser. Attendu : « Lint » rouge (`cannot find value CONTENEURS_DE_LA_GENESE`).
- [ ] **Étape 3 : écrire le code** : les constantes ; `documents::empreintes_nommees` remplacée par `services::lecture::empreintes_nommees`, et les imports qui ne servaient qu'à elle retirés (`PRISES`, `documents.rs:38`, et `ObjType`, `:31`, employés aux seules :632-633 ; « Lint », en `-D warnings`, dit tout autre reste) ; `GENESE` provisoire inchangée, pour lire l'octet de la recette ; `lecture::empreintes_nommees` saute un conteneur
absent (sous la genèse provisoire, `midi_verses` et `textes_verses` n'existent pas).
- [ ] **Étape 4 : la genèse neuve.** Pousser : `la_genese_se_reconstruit_a_l_octet` rouge, la liste des octets imprimée (si un fichier qui précède
`genese.rs` rougit sous la genèse provisoire, son test s'ignore de même le temps de cet envoi, et la PR le dit) ; la recopier dans `GENESE`, sur une ligne, sous `#[rustfmt::skip]` ; vérifier sur le poste (`python -X utf8`) que son SHA-256 égale `a02890ac…6fc1` (si la CI rend d'autres octets, ce sont les siens qui font foi, et la PR dit l'écart au calcul). Puis, chaque `#[ignore]` de cette tâche retiré, `genese.rs`, `references.rs`, `demarrage.rs`, `fonds.rs` et
`version.rs` verts dans `verifier` et `windows` ; `migration_de_test`, qui part de `GENESE`, suit seule.
- [ ] **Étape 5 : committer** par chemins (`feat: le schéma de la v0.3.0, sa genèse, et les empreintes nommées au noyau`).

### Tâche 9 : les règles d'une idée, au noyau

**Fichiers :** modifier `crates/noyau/src/donnees/idee.rs`, `crates/noyau/tests/idee.rs`, `.cargo/mutants.toml`.

**Interfaces — produit** (`noyau::donnees::idee`) :

```rust
/// La clé d'un lien « apparenté » : `apparente:<a>:<b>`, les deux bouts dans l'ordre de leurs octets.
pub fn cle_du_lien(a: &str, b: &str) -> String;
/// Les deux bouts d'une clé de lien ; `None` si la clé n'en est pas une.
pub fn bouts_du_lien(cle: &str) -> Option<(&str, &str)>;
/// Reliée : au moins un lien dont l'autre bout n'est pas une idée retirée (§ 30 · 9.1).
pub fn reliee(idee: &IdeeLue, retiree: impl Fn(&str) -> bool) -> bool;
/// La durée d'une idée, des durées de ses prises : la plus longue ; aucune prise, ou une prise
/// sans durée lue, `None` (§ 30 · 9.2).
pub fn duree(prises: &[Option<u64>]) -> Option<u64>;
/// Sans rien : ni captation, ni texte, ni étiquette, ni lien (§ 30 · 9.3).
pub fn est_vide(idee: &IdeeLue, captations: usize) -> bool;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assiette { pub considerees: usize, pub portant: usize, pub decidees: usize }
/// L'assiette d'un compte sur les idées : les idées retirées n'y entrent pas ; `porte` dit si une
/// idée porte la donnée ; au lot 1, toute donnée portée est décidée (Éc-15).
pub fn assiette<'a>(idees: impl IntoIterator<Item = &'a IdeeLue>, porte: impl Fn(&IdeeLue) -> bool) -> Assiette;
```

- [ ] **Étape 1 : écrire les tests** (`tests/idee.rs`), chacun avec l'exemple du passage de LE-6 :
  - `la_cle_du_lien_range_ses_bouts_par_octets` : `cle_du_lien("01M76GVQ00041061050R3GG28A", "01HF7YAT00000000000000000B")` = `cle_du_lien` en sens contraire = `"apparente:01HF7YAT00000000000000000B:01M76GVQ00041061050R3GG28A"` ; `cle_du_lien("idee-10", "idee-2")` = `"apparente:idee-10:idee-2"` ; `bouts_du_lien` rend les deux bouts, et `None` pour `"autre:a:b"` et `"apparente:a"` ;
  - `une_idee_n_est_reliee_que_par_un_bout_non_retire` : A–B, B retirée : A non reliée, B reliée ; B rétablie : A reliée ; C sans lien : non reliée ;
  - `la_duree_est_celle_de_la_plus_longue_prise` : `[]` → `None` ; `[Some(30_000), None]` → `None` ; `[Some(30_000), Some(64_000)]` → `Some(64_000)` ;
  - `une_idee_est_vide_sans_rien` : sans rien et `0` → vrai ; une captation, un texte, une étiquette ou un lien, chacun seul → faux ;
  - `l_assiette_ecarte_les_idees_retirees` : douze idées, neuf portant la donnée, et une treizième retirée qui la porte → `Assiette { considerees: 12, portant: 9, decidees: 9 }`.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les règles d'une idée`), pousser. Attendu à « Lint » : `cannot find function cle_du_lien`.
- [ ] **Étape 3 : écrire le code** ; `cle_du_lien` range ses bouts par `let (p, q) = (a.min(b), a.max(b));`, puis `format!("apparente:{p}:{q}")` : aucun opérateur que `cargo-mutants` ferait muter, quand `if a < b { (a, b) } else { (b, a) }` laisserait le mutant `<` → `<=`, équivalent (pour `a == b`, les deux branches rendent la même clé), qui survivrait ; `.cargo/mutants.toml` : `idee.rs`, `"--test", "idee"`, les deux mentions du nombre de modules (:2, « Seize modules sont examinés » ; :22, « ces seize modules »), la liste qui nomme « les règles d'une idée ». La vue (`Services::vue_d_une_idee`) les prend à la tâche 11, qui
  l'éprouve.
- [ ] **Étape 4 : voir le vert.** Attendu : `idee.rs` `ok. 6 passed` dans `verifier`.
- [ ] **Étape 5 : committer** par chemins (`feat: les règles d'une idée au noyau, lien, reliée, durée, assiette`).

### Tâche 10 : les gestes de tri au noyau, verser une fiche, et l'inverse de chacun

**Fichiers :** modifier `crates/noyau/src/donnees/gestes.rs`, `crates/noyau/tests/gestes.rs`.

**Interfaces — produit** (`noyau::donnees::gestes`) :

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GesteDeTri { pub operation: Operation, pub inverse: Vec<Changement> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Impossible {
    TexteVide, IdeeAbsente { idee: String }, IdeeRetiree { idee: String }, IdeeNonRetiree { idee: String },
    MemeTexte, EtiquetteVide, EtiquetteDejaPosee { etiquette: String }, EtiquetteAbsente { etiquette: String },
    RelieeAElleMeme, LienDejaPose, LienAbsent, CaptationAbsente { empreinte: String }, DejaDansLIdee,
    CaptationDejaVersee { empreinte: String }, DureeHorsDUnePrise, DureeTropLongue { duree_ms: u64 }, HorsDeLImport,
}
/// Le plus grand entier que `JSON.parse` relit à l'identique, 2⁵³ − 1 : celui de
/// `commandes::ENTIER_SUR`, que le noyau ne voit pas.
pub const ENTIER_SUR: u64 = 9_007_199_254_740_991;
pub fn presente(id: &str, lue: Option<IdeeLue>) -> Result<IdeeLue, Impossible>;
pub fn completer(idee: &IdeeLue, texte: &str, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn poser_une_etiquette(idee: &IdeeLue, etiquette: &str, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn oter_une_etiquette(idee: &IdeeLue, etiquette: &str, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn relier(a: &IdeeLue, b: &IdeeLue, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn delier(a: &IdeeLue, b: &IdeeLue, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn regrouper(captation: &CaptationLue, source: &IdeeLue, captations_de_la_source: usize, cible: &IdeeLue,
    heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn retirer(idee: &IdeeLue, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
pub fn retablir(idee: &IdeeLue, heure: i64, decalage_minutes: i32) -> Result<GesteDeTri, Impossible>;
/// Verser la fiche d'une captation : l'idée `idee` naît avec elle, de provenance `provenance`.
pub fn verser(sorte: Sorte, empreinte: &str, fiche: &Fiche, idee: &str, provenance: &Provenance,
    existante: Option<&CaptationLue>, heure: i64, decalage_minutes: i32) -> Result<Operation, Impossible>;
/// L'annulation : l'inverse du geste, sous « Annulé : » suivi de son message.
pub fn annulation(geste: &GesteDeTri, heure: i64) -> Operation;
```

**Les règles.** Tout geste de tri qui touche une idée retirée rend `IdeeRetiree`, sauf `retablir` (Éc-10 ;
question 6) ; un geste sans effet est refusé (C12) ; l'inverse se calcule sur l'idée lue avant le geste (Éc-6). Une
étiquette se range sans `#`, comparée octet pour octet, sans normalisation (Éc-21 ; question 7) ; vide,
`EtiquetteVide`. Un texte complété vide une fois ses blancs ôtés, `TexteVide` (question 1). Un MIDI versé ou un texte
versé n'entre que par l'import (glossaire ; § 30 · 8.1) : versé de provenance `captee`, `HorsDeLImport`. Une durée
au-delà d'`ENTIER_SUR` ne se transmettrait pas telle quelle à l'interface (Éc-14) : `DureeTropLongue`, une
impossibilité stricte (ADR-006).

**Les faits d'une captation versée** (§ 30 · 9.2 ; ADR-024, décisions 4 et 6 ; C3). Une prise porte `idee`,
`format` et `duree` ; un MIDI versé ou un texte versé, `idee` et `format`. La date et l'appareil d'une prise ne
s'écrivent pas : ce sont des faits de sa captation, que rien ne lit sur elle à INC-1, où toute prise est importée ;
ils valent `—`, et la capture audio, quand elle viendra, les portera par la `Fiche`. `format` prend le vocabulaire
du § 30 · 9.2 et de l'ADR-024, jamais l'extension, et le noyau le range tel que la fiche le porte :

- une prise : son format tel que sa signature et son en-tête le nomment, en minuscules, `wav` pour un en-tête
  RIFF/WAVE, `flac` pour un en-tête `fLaC` ; les autres, nommés de même par la tranche 5, qui les reconnaît à leur
  signature (CAP-03). Une prise `flac` n'a pas de durée à INC-1 : l'établi ne lit que l'en-tête d'un WAV, et les
  autres formats sont versés avec leurs faits à `—` (ADR-024, décision 6). En tranche 2, les tests et le générateur
  n'écrivent donc que `wav`, avec sa durée, sauf la prise sans durée lue de la tâche 11, en `flac` ;
- un MIDI versé : son format SMF, `smf0`, `smf1` ou `smf2` ;
- un texte versé : `brut`, `markdown` ou `chordpro`.

Le codage, les canaux, la fréquence et les bits d'un WAV, et la division d'un MIDI, viennent en tranche 5 avec
l'en-tête qui les donne, sous des clés `<e>/<fait>` ajoutées, ce qui ne demande aucune migration.

Un chemin s'écrit ici par ses segments, `[idees, <i>, texte]` : `<e>/idee` est **une seule clé**, sans objet par
empreinte (C3), et `[prises, "<e>/idee"]` a deux segments ; `<conteneur>` est celui de la sorte (`Sorte::conteneur`).

| Geste | Changements | Inverse | Message (`{j}` : `date_du_message(le de l'idée, décalage)`) |
|---|---|---|---|
| compléter | `Poser [idees, <i>, texte]` | `Poser` du texte d'avant, ou `Oter` s'il n'y en avait pas | `Idée du {j} : texte complété` |
| poser une étiquette | `Poser [idees, <i>, etiquettes, <e>]` = `Vrai` | `Oter` | `Idée du {j} : étiquette #<e> posée` |
| ôter une étiquette | `Oter [idees, <i>, etiquettes, <e>]` | `Poser` = `Vrai` | `Idée du {j} : étiquette #<e> ôtée` |
| relier | `Poser [liens, <clé>]` = `Vrai` | `Oter` | `Idée du {ja} reliée à l'idée du {jb}` |
| délier | `Oter [liens, <clé>]` | `Poser` = `Vrai` | `Idée du {ja} déliée de l'idée du {jb}` |
| regrouper | `Poser [<conteneur>, "<e>/idee"]` = cible ; la source vidée : `Poser [idees, <s>, retiree]` = heure | `Poser [<conteneur>, "<e>/idee"]` = source ; la source vidée : `Oter [idees, <s>, retiree]` | `Idée du {jc} : prise regroupée` (`MIDI versé regroupé`, `texte versé regroupé`), suivi de `, idée du {js} retirée` si la source est vidée |
| retirer | `Poser [idees, <i>, retiree]` = `Entier(heure)` | `Oter` | `Idée du {j} retirée` |
| rétablir | `Oter [idees, <i>, retiree]` | `Poser` = `Entier(<date d'avant>)` | `Idée du {j} rétablie` |
| verser | `Creer [idees, <i>]` [`etiquettes`] ; `Poser [idees, <i>, provenance]`, puis `par`, `le`, (`depuis`) ; `Poser [<conteneur>, "<e>/idee"]` et `[<conteneur>, "<e>/format"]` ; d'une prise seulement, `[prises, "<e>/duree"]` (`Naturel`, si la fiche la porte) ; ni `<e>/le` ni `<e>/par`, à aucune sorte | — | `Idée du {j} : prise versée` (`MIDI versé`, `texte versé`) |
| annuler | l'inverse | — | `Annulé : <message du geste>` |

La source d'un regroupement est vidée quand `est_vide` de la source, la captation ôtée, est vrai
(`captations_de_la_source == 1`, ni texte, ni étiquette, ni lien). Les refus : `presente` rend `IdeeAbsente` pour
`None` ; `MemeTexte` ; `EtiquetteDejaPosee`, `EtiquetteAbsente` ; `RelieeAElleMeme`, `LienDejaPose`, `LienAbsent` ;
`DejaDansLIdee` (source = cible) ; `IdeeNonRetiree` (rétablir) ; `IdeeRetiree` (retirer une idée retirée, et toute
idée retirée touchée) ; `CaptationDejaVersee` (`existante`) ; `DureeHorsDUnePrise` ; `DureeTropLongue`
(au-delà d'`ENTIER_SUR`) ; `HorsDeLImport`. Chaque `Display`, un constat :
« Aucune idée ne porte l'identifiant {idee}. », « L'idée {idee} est retirée. », « L'idée {idee} n'est pas retirée. »,
« Le texte est déjà celui-ci. », « L'étiquette est vide. », « L'étiquette #{e} est déjà posée. », « L'étiquette #{e}
n'est pas posée. », « Une idée ne se relie pas à elle-même. », « Ces deux idées sont déjà reliées. », « Ces deux idées
ne sont pas reliées. », « Aucune captation ne porte l'empreinte {e}. », « Cette captation est déjà dans cette
idée. », « La captation {e} est déjà versée. », « Seule une prise a une durée. », « Une durée de {duree_ms} ms passe le plus
grand entier que l'interface relit à l'identique. », « Un MIDI versé ou un texte versé n'entre que par l'import. ».

- [ ] **Étape 1 : écrire les tests** (`tests/gestes.rs`), sur des `IdeeLue` écrites dans le test, nées à `HEURE` (« 14 novembre ») : un test par ligne de la table, qui compare les changements, l'inverse et le message entiers (par exemple `poser_une_etiquette(&a, "pont", HEURE, 0)` : message `"Idée du 14 novembre : étiquette #pont posée"`, changements `[poser(&["idees", A, "etiquettes", "pont"], Simple::Vrai)]`, inverse `[oter(&["idees", A, "etiquettes", "pont"])]`) ; `rétablir` d'une idée retirée à `HEURE - 3_600` repose `Entier(HEURE - 3_600)` ; `compléter` d'une idée sans texte a pour inverse `Oter` ; `regrouper` la seule prise d'une idée qui n'a rien d'autre retire la source, message `"Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée"`, et la même avec un texte à la source ne la retire pas ; un test par refus, `Display` compris ; `deux_formes_d_une_meme_etiquette_font_deux_cles` (`"été"` en NFC et en NFD : deux changements différents) ; `annuler_rend_l_inverse_sous_annule` (`"Annulé : Idée du 14 novembre : étiquette #pont posée"`) ; `verser_une_prise_cree_son_idee_et_sa_fiche` (ses changements entiers, ci-dessous : `<e>/idee`, `<e>/format` et `<e>/duree`, ni `<e>/le` ni `<e>/par`), `verser_un_midi_n_a_pas_de_duree` (`DureeHorsDUnePrise`), `verser_un_midi_ou_un_texte_ne_pose_que_son_idee_et_son_format` (ses changements entiers : `Creer`, les quatre valeurs de l'idée, puis `<e>/idee` et `<e>/format` seuls, le format `smf1` pour un MIDI, `brut` pour un texte), `verser_un_midi_ou_un_texte_capte_est_refuse` (`HorsDeLImport`, les deux sortes), `verser_une_fiche_deja_versee_est_refuse`, `verser_une_duree_au_dela_d_entier_sur_est_refusee` (`ENTIER_SUR == (1u64 << 53) - 1` ; une prise de `duree_ms: Some(ENTIER_SUR)` se verse, `Naturel(ENTIER_SUR)` sous `<e>/duree` ; `Some(ENTIER_SUR + 1)` rend `DureeTropLongue { duree_ms: ENTIER_SUR + 1 }`, son `Display` compris : chaque mutant de la comparaison `>` (`>=`, `==`, `<`) meurt sur l'un de ces deux cas). Deux exemples entiers, que suivent les autres tests (`P` une empreinte, `A`, `V` deux idées, `IMPORT` l'ULID d'un import) :

```rust
// regrouper la seule prise P de V, qui n'a rien d'autre, dans A : V est vidée.
let cle = format!("{P}/idee");
assert_eq!(geste.operation.changements, vec![
    poser(&["prises", cle.as_str()], Simple::Chaine(A.into())),
    poser(&["idees", V, "retiree"], Simple::Entier(HEURE)),
]);
assert_eq!(geste.inverse, vec![
    poser(&["prises", cle.as_str()], Simple::Chaine(V.into())),
    oter(&["idees", V, "retiree"]),
]);
// verser la prise P, importée par l'établi, sous l'idée V qui naît avec elle : l'heure et
// l'appareil du versement sont ceux de l'idée ; la date et l'appareil de la prise restent « — ».
let op = verser(Sorte::Prise, P, &Fiche { format: "wav".into(), duree_ms: Some(30_000) }, V,
                &Provenance::Importee { par: Appareil::Etabli, depuis: IMPORT.into() }, None, HEURE, 0).unwrap();
assert_eq!(op.message, "Idée du 14 novembre : prise versée");
assert_eq!(op.changements, vec![
    Changement::Creer { chemin: chemin(&["idees", V]), conteneurs: vec!["etiquettes".into()] },
    poser(&["idees", V, "provenance"], Simple::Chaine("importee".into())),
    poser(&["idees", V, "par"], Simple::Chaine("etabli".into())),
    poser(&["idees", V, "le"], Simple::Entier(HEURE)),
    poser(&["idees", V, "depuis"], Simple::Chaine(IMPORT.into())),
    poser(&["prises", &format!("{P}/idee")], Simple::Chaine(V.into())),
    poser(&["prises", &format!("{P}/format")], Simple::Chaine("wav".into())),
    poser(&["prises", &format!("{P}/duree")], Simple::Naturel(30_000)),
]);
```
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les gestes de tri, verser et l'annulation, au noyau`), pousser. Attendu à « Lint » : `cannot find function completer`.
- [ ] **Étape 3 : écrire le code.**
- [ ] **Étape 4 : voir le vert.** Attendu : `gestes.rs` vert dans `verifier`, le compte dans la description.
- [ ] **Étape 5 : committer** par chemins (`feat: les gestes de tri au noyau, leurs refus, leur inverse et leur message`).

### Tâche 11 : les services : les gestes de tri, verser une fiche, la liste des idées, et leurs critères

**Fichiers :** modifier `crates/services/src/lib.rs`, `crates/services/tests/suite/mod.rs`,
`crates/services/tests/commandes_en_memoire.rs`, `crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — produit** (`Services`, chaque geste de tri rendant `Reponse<Faite>`) :

```rust
pub fn completer(&mut self, commande: &Completer, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn poser_une_etiquette(&mut self, commande: &Etiqueter, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn oter_une_etiquette(&mut self, commande: &Etiqueter, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn relier(&mut self, commande: &Relier, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn delier(&mut self, commande: &Relier, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn regrouper(&mut self, commande: &Regrouper, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn retirer(&mut self, commande: &Viser, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn retablir(&mut self, commande: &Viser, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
/// Les idées non retirées, dans l'ordre de leurs identifiants.
pub fn idees(&self) -> Reponse<Vec<String>>;
/// Verse `octets` par le contrat `Captations` (toujours, même devant un fonds en lecture seule),
/// puis la fiche, sous une idée qui naît avec elle (son ULID tiré) ; sans fonds, la naissance
/// d'abord, comme `noter`. Pas une commande avant la tranche 5 (écart 2). Le décalage, celui du
/// message (« Idée du … : prise versée »), passe tel quel à `gestes::verser` ; la tranche 5 le
/// recevra de la commande d'import.
pub fn verser_une_fiche(&mut self, sorte: gestes::Sorte, octets: &[u8], fiche: &gestes::Fiche,
    provenance: &gestes::Provenance, decalage_minutes: i32, contexte: &mut Contexte<'_>) -> Result<Versee, Refus>; // gestes : noyau::donnees::gestes
pub struct Versee { pub empreinte: String, pub idee: String, pub avertissement: Option<String> }
```

Un geste de tri : `lecture_seule()` ; sans fonds, `IdeeAbsente` ; la lecture (`lire_l_idee`, `captations_de`,
`lire_la_captation`) ; le noyau ; `appliquer` ; faite, il prend `dernier_geste` ; refusé, `dernier_geste` ne change
pas. `idees()`, une lecture, ne consulte pas `lecture_seule()` (tâche 5).

- [ ] **Étape 1 : écrire les tests** dans la suite, avec leurs enveloppes dans les deux crates :
  - `chaque_geste_de_tri_parait_au_journal` (enveloppe `cap02_4_chaque_geste_de_tri_parait_au_journal_comme_une_operation_nommee`, une part, non inscrite : relier à un motif ou un morceau et regrouper une photo viennent après) : sur deux notes A et B et une prise P versée (idée V), compléter A, poser `#pont` sur A, ôter `#pont`, relier A–B, délier A–B, regrouper P dans A (V retirée), retirer B, rétablir B ; après chaque geste, une entrée de plus au journal, au message de la table de la tâche 10, à l'heure du contexte ;
  - `l_idee_videe_par_un_regroupement_est_retiree` (enveloppe `cap02_8_l_idee_videe_par_un_regroupement_est_retiree_et_l_annulation_la_retablit`, que la tâche 12 complète par l'annulation ; inscrit en (b), entier : son cas, une idée qui n'a qu'une prise, regroupée puis le geste annulé, se calcule sur les gestes de la tranche, et une idée qui porte aussi une photo, au lot 2, n'est pas « une idée qui n'a rien d'autre ») : X notée avec une prise P1 versée puis regroupée dans X ; V, née par `verser_une_fiche` de P2 seule ; regrouper P2 dans X : le journal gagne exactement une entrée, « Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée » (« par le même geste »), la vue de V a `retiree: Some(HEURE)`, `idees()` ne la rend plus, la vue de X porte P1 et P2 ;
  - `deux_prises_regroupees_restent_deux_captations` (enveloppe `cap02_3_deux_prises_regroupees_restent_deux_captations_d_empreinte_inchangee`, une part, non inscrite) : les deux captations de X, distinctes, d'empreintes celles que `verser_une_fiche` a rendues, que `relire` rend ;
  - `une_idee_retiree_quitte_la_liste_et_se_retablit_telle_qu_elle_etait` (enveloppe `cap02_5_…`, la part de la boîte, non inscrite) : texte, étiquettes, liens, captations rendus ; l'idée porte au moins deux liens, et le test compare le champ `liens` de la vue entier, son ordre et son contenu (l'autre bout de chaque lien, dans l'ordre de leurs octets), avant le retrait et après le rétablissement ;
  - `un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait` (point de revue 2) ; son enveloppe sur la paire de fichiers, qui connaît la racine, ajoute `assert!(!racine.chemin().join("documents").join("fonds").exists())`, le trait `Paire` ne donnant aucun chemin ;
  - `une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change` (point de revue 3) : la seconde rend `Raison::Impossible`, « La captation … est déjà versée. », le journal inchangé ;
  - `un_geste_qui_touche_une_idee_retiree_est_refuse` : poser, relier (depuis l'autre bout), regrouper vers elle : `Raison::Impossible` ;
  - `la_vue_tire_reliee_et_la_duree_des_regles_du_noyau` : A et B reliées, B retirée : la vue de A dit `reliee: false` ; B rétablie : `true`. Puis V1 née d'une prise de 30 000 ms, V2 d'une prise de 64 000 ms regroupée dans V1 : la vue de V1 dit `duree_ms: Some(64_000)` ; une prise sans durée lue (`Fiche { format: "flac", duree_ms: None }`) regroupée dans V1 la ramène à `None` (CAP-02, Règles) ;
  - `une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme` (tâche 5) gagne, devant le fonds N + 1 : `idees()` répond ; `verser_une_fiche(gestes::Sorte::Prise, b"une prise devant un fonds plus recent", …)` rend `Raison::VersionPlusRecente { version: 2 }`, et `captations().relire(&empreinte(b"une prise devant un fonds plus recent"))` rend ces octets (§ 31 · 2.4 : « Une captation n'est jamais refusée pour autant »).

  La prise de test : `verser_une_fiche(gestes::Sorte::Prise, b"une prise de test", &gestes::Fiche { format: "wav".into(), duree_ms: Some(30_000) }, &gestes::Provenance::Importee { par: gestes::Appareil::Etabli, depuis: IMPORT.into() }, …)` : à INC-1, une prise n'entre sur l'établi que par l'import (la capture audio de l'établi vient après INC-1). Elle écrit `<e>/idee`, `<e>/format` (`wav`) et `<e>/duree` ; sa date et son appareil restent `—`, sans clé (C3) : l'heure et l'appareil du versement sont ceux de son idée, `le` et `par`.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les gestes de tri et leurs critères, sur les deux paires`), pousser. Attendu à « Lint » : `no method named completer found`.
- [ ] **Étape 3 : écrire le code.** La vue
  (`Services::vue_d_une_idee`) prend `reliee` et `duree` de `noyau::donnees::idee` (tâche 9), la durée sur les seules
  prises. Elle tire aussi ses `liens` du noyau : l'autre bout de chaque clé que lit `lire_l_idee`, par
  `noyau::donnees::idee::bouts_du_lien` (tâche 9), dans l'ordre de leurs octets ; aucune découpe de clé n'est écrite
  dans `services`. Jusque-là, la vue rend `liens: Vec::new()`, comme `reliee: false` et `duree_ms: None` (revue de la
  tâche 5) ; sa doc, qui le dit, se réécrit avec le code. La doc de `noter` dit « Ne touche pas au dernier geste de
  tri. », avec le champ `dernier_geste` (revue de la PR (a)).
- [ ] **Étape 4 : voir le vert.** Attendu : `commandes_en_memoire.rs` dans `verifier`, `commandes_sur_fichiers.rs` dans `verifier` et `windows`, verts, les comptes dans la description.
- [ ] **Étape 5 : committer** par chemins (`feat: les gestes de tri et le versement d'une fiche, conduits par les services`).

### Tâche 12 : annuler

**Fichiers :** modifier `crates/services/src/lib.rs`, `crates/services/src/lecture.rs` (`teneur`),
`crates/stockage/src/lib.rs` (:151-208 : `teneur` et ses aides passent à `services::lecture` ; `pub use
services::lecture::teneur;` ; leurs imports, :21, :26 et :28, partent, `use automerge::AutomergeError;` restant pour
`Erreur`), `crates/noyau/src/donnees/teneur.rs` (sa doc, :2 et :10 : l'arbre, que `services` tire du document ;
écart 9), la suite et ses deux enveloppes.

**Interfaces — produit :**

```rust
/// Le dernier geste de tri fait par ces services, son inverse, sous « Annulé : … » ; fait, il vide
/// le dernier geste ; refusé par l'adaptateur, il le garde ; sans geste, `Raison::RienAAnnuler`,
/// « Aucun geste de tri à annuler depuis l'ouverture. » (décision 18). Le refus d'une annulation
/// après une synchronisation qui a changé l'élément (§ 30 · 11) vient au lot 3.
pub fn annuler(&mut self, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
// services::lecture
/// La teneur (§ 31 · 2.6), déplacée du stockage, à l'identique : `TENEUR_DES_TYPES` ne change pas.
pub fn teneur(doc: &Automerge) -> Result<String, ErreurDeTeneur>;
```

- [ ] **Étape 1 : écrire les tests.** Dans la suite :
  - `annuler_rend_le_fonds_d_avant` (enveloppe `cap02_6_annuler_rend_le_fonds_d_avant_et_le_journal_porte_le_geste_puis_son_annulation`, une part, non inscrite : « le dernier geste de tri » est aussi relier à un motif ou à un morceau (INC-2), et regrouper des photos (lot 2), que la tranche n'annule pas) : pour chacun des huit gestes de tri, la teneur d'avant le geste égale celle d'après l'annulation, et les deux dernières entrées du journal sont le message du geste puis `Annulé : ` suivi de lui ;
  - la seconde moitié de `l_idee_videe_par_un_regroupement_est_retiree` : annuler rend P2 à V, rétablie, et un second annuler rend `Raison::RienAAnnuler` ;
  - `la_suite_de_la_decision_18` : la table du passage de LE-6, § 3.9, rejouée pas à pas (noter A ; annuler refusé ; étiqueter A `#pont` ; noter B ; annuler, « Annulé : Idée du 14 novembre : étiquette #pont posée » ; annuler refusé ; relier A–B ; retirer B ; étiqueter A `#pont` ; reposer `#pont`, refusé ; annuler ; annuler refusé), puis le lien A–B présent, B retirée, A sans étiquette ;
  - `un_geste_refuse_ne_change_pas_ce_qu_annuler_vise` (point de revue 4) ; `la_reouverture_vide_ce_qu_annuler_vise` (point de revue 5) ;
  - `une_annulation_ne_s_annule_pas` : annuler, puis annuler, `RienAAnnuler`.

  Sur la paire de fichiers seule, `une_annulation_refusee_par_le_disque_se_refait` (point de revue 4) : poser, puis sous `couper(Etape::Renommage, 1)` annuler, `Raison::Disque` ; la panne levée, annuler fait.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: annuler le dernier geste de tri`), pousser. Attendu à « Lint » : `no method named annuler found`.
- [ ] **Étape 3 : écrire le code** ; `teneur` déplacée, ses tests (`crates/stockage/tests/teneur.rs`, `references.rs`) inchangés.
- [ ] **Étape 4 : voir le vert**, dans `verifier` et `windows`.
- [ ] **Étape 5 : committer** par chemins (`feat: annuler le dernier geste de tri, par son inverse nommé`).

### Tâche 13 : CAP-02 · 7, le témoin du test 11 par les opérations, les documents de la PR (b), la mutation remesurée

**Fichiers :** créer `crates/stockage/tests/repliques.rs` ; modifier `crates/services/tests/suite/mod.rs` (`temoin`),
`crates/stockage/tests/references.rs` (le
témoin, :22, son commentaire, :25-44 ; la lignée, :157-206), `docs/31-persistance-et-synchronisation.md` (§ 2,
:125-164 ; § 2.3, :344-376), `docs/30-modele-de-domaine.md` (§ 1, la classe `Lien`, :59-63 ; § 9.3, :666-671 ; § 11,
:696-722), `docs/20-sfd/21-capter-et-organiser.md` (CAP-02, Règles),
`docs/70-qualite-et-verification.md` (§ 4.1, :156 et suivantes), `docs/80-glossaire.md`, `CONVENTIONS.md` (`mutants`),
`.github/workflows/ci.yml` (`mutants`), `.github/workflows/mutants-complet.yml`.

- [ ] **Étape 1 : CAP-02 · 7.** Dans `tests/repliques.rs`, qui inclut la suite (`#[path = "../../services/tests/suite/mod.rs"] mod suite;`), `cap02_7_deux_liens_poses_en_sens_contraires_n_en_font_qu_un` : deux notes A et B notées par les services sur la paire de fichiers, dans `<dossier jetable>/a`, fermées, l'arbre copié en `<…>/b` (`commun::copier_l_arbre`), chaque copie rouverte avec son `AleaDeTest` (graines 0 et 1) ; sur l'une `relier(A, B)`, sur l'autre `relier(B, A)` ; `commun::reunir`, sur `documents_mut().document_mut()` de chacune ; le conteneur `liens` des deux copies a une seule clé, `cle_du_lien(A, B)`, et `get_all` sur elle rend deux valeurs `true` (Éc-9). CAP-02 · 7 s'inscrit, entier : son cas, deux idées reliées hors ligne en sens contraires sur deux répliques, puis leur fusion par l'échange de `synchro` (H07), est livré ici ; un lien vers un motif ou un morceau (INC-2) n'est pas son cas. Le rouge se lit en faussant un temps l'attendu (deux clés sous `liens`) : `panicked at` dans
  `verifier`, puis l'attendu juste ; committer, pousser, vert dans `verifier` et `windows`.
- [ ] **Étape 2 : le témoin par les opérations** (C26). `references.rs` inclut la suite, comme `repliques.rs`. `faire_naitre_le_temoin(racine, alea: &mut dyn Alea)` devient : `Services` sur `FondsSurFichiers`, qui joue les gestes
  du témoin, écrits une seule fois dans la suite, `suite::temoin<D: Documents, C: Captations>(services: &mut
  Services<D, C>, contexte: &mut Contexte<'_>)`, que `le_fonds_de_chaque_geste` (tâche 14) rejoue aussi ; l'aléa reçu (`AleaDeTest { graine: 0, appels: 0 }` pour le témoin), `HEURE`, décalage 0 ; noter « Un refrain qui monte, en ré » ; poser `refrain` ; noter « Un pont en mi mineur » ; relier les deux ; verser une prise (`b"une prise du temoin"`, format `wav`, 30 000 ms), un MIDI (`b"un midi du temoin"`, format `smf1`), un texte (`b"un texte du temoin"`, format `brut`), chacun `Importee { par: Etabli, depuis: IMPORT }` (une idée née d'un fichier versé est `importee`, § 30 · 8.1 ; l'import lui-même ne s'écrit qu'en tranche 5, et `imports` reste vide) ; regrouper le MIDI dans la première idée. La prise ne porte que `idee`, `format` et `duree` (tâche 10). La teneur attendue, écrite en commentaire, calculée sur le poste, 1 708 octets (un sérialiseur qui redonne à l'octet la teneur et l'empreinte des versions précédentes de ce plan, `13b6086f…`, puis `ae625f61…` sur 1 869 octets ; sans les deux faits de la prise ni le vocabulaire de `format`, `73565774…` sur 1 706, la valeur de la relecture par une session neuve) :

```text
{"idees":{"01HF7YAT000000000000000004":{"etiquettes":{"refrain":b1},"le":i1700000000,"par":s"etabli","provenance":s"captee","texte":s"Un refrain qui monte, en ré"},"01HF7YAT000000000000000005":{"etiquettes":{},"le":i1700000000,"par":s"etabli","provenance":s"captee","texte":s"Un pont en mi mineur"},"01HF7YAT000000000000000006":{"depuis":s"01HF7YAT00ZZZZZZZZZZZZZZZZ","etiquettes":{},"le":i1700000000,"par":s"etabli","provenance":s"importee"},"01HF7YAT000000000000000007":{"depuis":s"01HF7YAT00ZZZZZZZZZZZZZZZZ","etiquettes":{},"le":i1700000000,"par":s"etabli","provenance":s"importee","retiree":i1700000000},"01HF7YAT000000000000000008":{"depuis":s"01HF7YAT00ZZZZZZZZZZZZZZZZ","etiquettes":{},"le":i1700000000,"par":s"etabli","provenance":s"importee"}},"identifiant":s"01HF7YAT000000000000000001","imports":{},"liens":{"apparente:01HF7YAT000000000000000004:01HF7YAT000000000000000005":b1},"midi_verses":{"dee9bf9e8aaeb03dce938f41ee65b33989d3436072478a95514ca26df98f39d3/format":s"smf1","dee9bf9e8aaeb03dce938f41ee65b33989d3436072478a95514ca26df98f39d3/idee":s"01HF7YAT000000000000000004"},"origines_declarees":{},"photos":{},"prises":{"8c12b061b3b4734f4f62d617674f892c49cb31c9212878903601d7ca369a3299/duree":u30000,"8c12b061b3b4734f4f62d617674f892c49cb31c9212878903601d7ca369a3299/format":s"wav","8c12b061b3b4734f4f62d617674f892c49cb31c9212878903601d7ca369a3299/idee":s"01HF7YAT000000000000000006"},"proprietaire":s"00000000-0000-4000-8000-000000000003","requetes_enregistrees":{},"textes_verses":{"05474c7f2b8f6390a4a215fa6edca1ac61a8f97aa9cad1e08652909008b565b8/format":s"brut","05474c7f2b8f6390a4a215fa6edca1ac61a8f97aa9cad1e08652909008b565b8/idee":s"01HF7YAT000000000000000008"},"version_de_schema":u1}
```

  `TENEUR_DU_TEMOIN` = `291a0df280e2274fe96fc10d24a232a57213a0e938fa0a7b844be8f056657dbc`. Une empreinte qui diffère se corrige dans le code, sauf si la teneur imprimée montre une erreur de ce calcul, que la PR dit. `la_lignee_ne_change_pas_la_teneur` fait naître deux témoins : l'un sous `AleaDeTest { graine: 0, appels: 0 }`, l'autre sous `AutreLignee`, un `Alea` de test de `references.rs` qui rend les tirages d'`AleaDeTest { graine: 0, appels: 0 }` et met à 1 l'octet 1 du deuxième, celui de la lignée ; seul ce tirage diffère, donc seule la lignée : `01HF7YAT000000000000000002` et `01HF7YAT00000G000000000002`, calculées sur le poste. Le second pose ensuite `LIGNEES_REMPLACEES`, comme aujourd'hui, par `commun::ecrire` sur `documents_mut().document_mut()`, sous `pannes` ; les deux teneurs égales. Une autre graine changerait aussi l'identifiant du document, l'identifiant de propriétaire et chaque idée, que la teneur porte. Pousser : `references.rs` vert dans `verifier` et `windows`.
- [ ] **Étape 3 : les documents.** § 31 · 2 : le schéma de la `v0.3.0`, conteneur par conteneur, l'objet d'une idée, les clés à plat d'une captation (d'une prise, `idee`, `format` et `duree`, sa date et son appareil restant `—`, sans clé, tant que rien ne les lit sur la captation ; d'un MIDI versé ou d'un texte versé, `idee` et `format` ; `format` au vocabulaire du § 30 · 9.2 et de l'ADR-024, celui que la signature et l'en-tête d'une prise nomment (`wav`, `flac`…), `smf0`, `smf1` ou `smf2` d'un MIDI, `brut`, `markdown` ou `chordpro` d'un texte, jamais l'extension ; deux répliques qui versent la même captation laissent deux valeurs sous chaque clé, gardées, relevées au lot 3), les liens et leur clé, un lien et une étiquette en `true`, sans date, une étiquette rangée sans `#` et comparée octet pour octet, sans normalisation (question 7), et où vivront les faits de la tranche 5, dont la division d'un MIDI et les autres faits d'un en-tête, sous des clés ajoutées, sans migration (C4) ; § 31 · 2.3 : la genèse de dix opérations. § 30 · 1 : la classe `Lien` perd `le` ; § 30 · 9.3 : « Un lien ne porte pas de date : celle du geste qui l'a posé se lit dans le journal du fonds » (Éc-9). § 30 · 11 et CAP-02 : la décision 18, dite sans « la place » (« Annuler ne vise que le dernier geste de tri fait sur l'appareil depuis l'ouverture ; une note, une requête enregistrée, un import ou une déclaration ne le visent ni ne le remplacent ; une annulation faite, un second « annuler » est refusé, et le dit »), l'exemple « Annulé : Idée du 3 novembre : étiquette #pont posée », un geste refusé qui ne change pas ce qu'annuler vise, et une clé en conflit avant le geste qui revient à sa seule valeur présentée (Éc-6) ; la phrase « Si une synchronisation a changé l'élément depuis, l'annulation est refusée, et Pupitre dit ce qui a changé » reste, ce refus venant au lot 3 (plan maître, « Ce qui est reporté »), et la PR le dit. § 70 · 4.1 : la ligne `| [CAP-02](20-sfd/21-capter-et-organiser.md#cap-02--trier-la-boîte-didées) | 7, 8 | INC-1, lot 1 |`, sous celle de CAP-01 (tâche 7) ; CAP-02 · 4 et 6 n'y entrent pas : ils visent chaque geste de tri, dont relier à un motif ou un morceau et regrouper une photo, et ne s'inscriront qu'avec leur dernière part (C30) ; ni CAP-02 · 3 et 5, qui attendent la boîte et les requêtes. Glossaire, après `git grep -n -i -w "étiquette" -- docs README.md CONVENTIONS.md` et `git grep -n -i -w "annuler" -- docs README.md CONVENTIONS.md`, les sorties citées dans la PR : « **Étiquette** » (d'une idée : un mot posé sur elle, sans `#`, que le message et l'écran ajoutent ; de git : `vX.Y.Z`, qui publie une version ; on dit duquel il s'agit dès que le contexte ne le dit pas) et « **Annuler** » (au fonds, le dernier geste de tri, § 30 · 11 ; pour une proposition, l'acceptation, INC-2). `CONVENTIONS.md`, ligne `mutants` : « les règles d'une idée ». `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add` ; `verif-diagrammes` se lit dans `documentation`.
- [ ] **Étape 4 : remesurer la mutation, à froid**, comme à la tâche 7, `--ref t2b-schema-et-gestes`.
- [ ] **Étape 5 : committer** par chemins (`test: CAP-02 · 7 et le témoin du test 11 par les opérations`, `docs: le schéma de la v0.3.0, annuler, et les critères livrés`, `ci: les délais de la mutation, remesurés avec les règles d'une idée`), pousser. **Fin de la PR (b) :** tous les jobs verts, `mutants` sans `MISSED`, le passage complet à froid lié ; la description nomme les deux références changées du test 11 (`GENESE_DU_FONDS`, `TENEUR_DU_TEMOIN`), dit `TENEUR_DES_TYPES` inchangée, la lecture des prises réécrite dans `h07.rs` (tâche 8), CAP-02 · 7 et 8 inscrits, chacun avec sa phrase (tâches 11 et 13), les parts de CAP-02 · 3, 4, 5 et 6 non inscrites, et pourquoi (CAP-02 · 4 et 6 visent aussi relier à un motif ou un morceau et regrouper une photo), CAP-02 · 7 sur la paire de fichiers (écart 3), l'écart 9, la teneur du témoin sans date ni appareil de prise et au vocabulaire de `format` (C3), les recherches de LE-2, et les questions 1, 2, 4, 6 et 7 tranchées ; la revue de la PR entière, ses retours traités ; puis la relecture des corrections, sur la tête qui part, celle de `--match-head-commit` ; `gh pr ready <n>`, puis la fusion.

---

## PR (c) · les preuves, les messages d'erreur, la première mesure

**Branche :** `t2c-preuves-et-mesure`. **Titre :** `feat: les preuves de la tranche 2, les messages d'erreur en
français et la première mesure` (90). **Documents :** § 31 · 2.4 et 2.6 ; CAP-01 (cas limite) ; `docs/pistes.md` ;
`CLAUDE.md` (« Commandes ») ; `CONVENTIONS.md` (`verifier`) ; `ci.yml` ; registre (LE-14 clos). **Fin :** voir la
tâche 20.

### Tâche 14 : I28 sur l'historique, I01, ni `Counter` ni objet `Text`

**Fichiers :** modifier `crates/services/tests/suite/mod.rs` et ses deux enveloppes.

- [ ] **Étape 1 : écrire les tests** dans la suite, sur un fonds où chaque geste de la tranche a été fait une fois (une aide de la suite, `le_fonds_de_chaque_geste`, qui rejoue le témoin de la tâche 13 puis chaque geste de tri et une annulation) :
  - `chaque_operation_laisse_une_seule_modification_avec_son_message` : la naissance compte à part, la première note en écrivant trois (« Genèse du fonds », « Fonds créé », puis son message) ; chaque autre commande faite en ajoute exactement une à `get_changes`, dont le message est celui que rend le noyau (une aide, `une_modification(doc, heads_d_avant, message) -> Result<(), String>`) ;
  - `l_historique_ne_porte_que_des_operations_connues` : chaque message de `journal` est la genèse, la naissance, un message de migration (`message_de_la_migration`), ou suit l'une des formes fermées de la table de la tâche 10, que l'aide `message_connu` énumère, `<date>` étant une date de `date_du_message` (« 1er » ou de 2 à 31, puis l'un des douze mois) et `<e>` une étiquette non vide : `Idée du <date> notée` ; `Idée du <date> : texte complété` ; `Idée du <date> : étiquette #<e> posée`, `Idée du <date> : étiquette #<e> ôtée` ; `Idée du <date> reliée à l'idée du <date>`, `Idée du <date> déliée de l'idée du <date>` ; `Idée du <date> : prise regroupée`, `Idée du <date> : MIDI versé regroupé`, `Idée du <date> : texte versé regroupé`, chacune suivie au besoin de `, idée du <date> retirée` ; `Idée du <date> retirée`, `Idée du <date> rétablie` ; `Idée du <date> : prise versée`, `Idée du <date> : MIDI versé`, `Idée du <date> : texte versé` ; et `Annulé : ` suivi de la forme d'un geste de tri, jamais d'une note ni d'un versement, qu'annuler ne vise pas. Aucune forme ne finit par une suite libre ;
  - `rien_d_engendre_n_est_stocke` (`// § 31 · 10, test 7 : chercher les champs interdits du § 30 · 13`) : chaque clé du document est dans la liste blanche tirée de `noyau::donnees::fonds` (la racine du document : la version de schéma, l'identifiant, la lignée, l'identifiant de propriétaire, les neuf conteneurs ; une idée : `texte`, `etiquettes`, `provenance`, `par`, `le`, `depuis`, `retiree` ; sous `prises`, `<empreinte>/` suivi de `idee`, `format` ou `duree` ; sous `midi_verses` et `textes_verses`, `<empreinte>/` suivi de `idee` ou `format` ; un lien : `apparente:<a>:<b>`), et aucun segment n'est l'un des noms interdits (`duree` sous une idée, `reliee`, `prises`, `photos`, `midi_verses` ou `textes_verses` sous une idée, `etiquettes` à la racine du document, `compte`, `total`, `a_envoyer`) ; un `<empreinte>/le` ou un `<empreinte>/par` sous `prises` sort de la liste blanche (C3) ;
  - `aucun_counter_ni_objet_text_dans_un_document_des_operations` : le parcours ne rencontre ni `ScalarValue::Counter`, ni `ObjType::Text`, ni `ObjType::Table`, ni `ObjType::List`.

  Chacun prouve aussi qu'il refuse, sur une plantation qui lui est propre, et la plantation paraît dans ce que rend son aide. Les plantations se posent à la main dans un document de test, né par `application::faire_naitre` : les seuls `put`, `put_object` et `commit` de la suite hors des opérations et de `migration_de_test`, dans un test, que le contrôle statique d'I28, qui ne lit que les `src/`, ne voit pas.
  - `chaque_operation_…` : une commande plantée en deux modifications, deux `application::appliquer` de suite après les heads d'avant ; `une_modification` le refuse, et nomme les deux messages ;
  - `l_historique_…` : `message_connu` refuse « Idée du 3 novembre : autre chose », une forme que la table n'a pas, et « Annulé : Idée du 3 novembre notée », une note qu'annuler ne vise pas ; il reconnaît un exemple de chaque forme de la table, et « Annulé : » suivi de chaque forme d'un geste de tri ;
  - `rien_d_engendre_…` : `idees/<i>/duree`, posé à la main ; l'aide `parcourir(doc) -> Vec<String>` (les chemins fautifs) le rend ;
  - `aucun_counter_…` : un `ScalarValue::counter(0)` sous `idees/<i>/compteur_de_test`, et un objet `ObjType::Text` sous `idees/<i>/texte_de_test`, posés à la main ; le parcours rend les deux chemins.
- [ ] **Étape 2 : voir le rouge, puis le vert.** Ces tests n'appellent que ce que les PR (a) et (b) livrent : le rouge se lit sur leur aide de refus. Committer (`test: I28 sur l'historique, I01 et ni Counter ni Text`), pousser ; attendu, verts dans `verifier` et `windows`.
- [ ] **Étape 3 : committer** par chemins.

### Tâche 15 : à deux et à trois répliques

**Fichiers :** modifier `crates/stockage/tests/repliques.rs`.

- [ ] **Étape 1 : écrire les tests**, chaque réplique avec sa graine :
  - `un_texte_change_des_deux_cotes_garde_ses_deux_valeurs` : compléter X des deux côtés, réunir ; aussitôt, `get_all(idees/X, "texte")` rend les deux textes, entiers (T15) ;
  - `une_etiquette_posee_des_deux_cotes_n_en_fait_qu_une` : une clé `pont`, deux valeurs `true`, égales ;
  - `deux_regroupements_d_une_meme_prise_ne_la_montrent_que_dans_une_idee` (Éc-8, écart 10) : `e1` regroupée dans Y d'un côté, dans Z de l'autre (Y et Z ont un texte) ; réunies : `e1` paraît dans la vue d'une seule idée, la même sur les deux copies ; `get_all(<e1>/idee)` rend deux valeurs ;
  - `deux_regroupements_croises_retirent_les_deux_idees` (C28, Éc-7) : X n'a que P, Y n'a que Q ; P → Y d'un côté, Q → X de l'autre ; réunies : X et Y retirées toutes deux, `idees()` vide ; ce que la piste du lot 3 traitera ;
  - `une_prise_nee_sur_deux_repliques_ne_cache_aucun_de_ses_faits` (vigilance 1) : la même prise, importée par l'établi de chaque côté, à deux heures (`HEURE` et `HEURE + 60`, que portent les deux idées, jamais la prise) ; réunies : une seule clé par fait, et `get_all` rend deux valeurs pour chacun des trois faits (chaque réplique les a posés) : deux `idee` différents, deux `format` et deux `duree` égaux, dont `get` présente la valeur ; ni `<e>/le` ni `<e>/par` sous `prises` (C3) ; l'idée laissée vide reste visible, ce que la question 5 laisse à la tranche 5 ;
  - `l_inverse_d_un_texte_en_conflit_rend_la_valeur_presentee` (Éc-6) : le texte en conflit, compléter puis annuler : la teneur d'avant, et `get_all` ne rend plus qu'une valeur ;
  - `trois_repliques_convergent` (`// § 31 · 10, test 3 : trois répliques, opérations concurrentes aléatoires, synchronisation dans un ordre aléatoire`), `proptest`, `ProptestConfig { cases: 16, max_shrink_iters: 64, max_shrink_time: 60_000, ..ProptestConfig::default() }` : quatre idées nées sur A, copiées en B et C ; trente gestes de tri tirés (le geste, la réplique, ses arguments, refusés compris), des réunions tirées en chemin, puis A–B, B–C, A–B ; les trois copies ont les mêmes heads et la même teneur (Éc-20). La réduction bornée garde un échec sous les délais de `verifier` et de `windows`, où il se lit comme un échec et non comme un délai dépassé ; chaque ouverture tirant son acteur (`ActorId::random()`), un cas qui échoue se rejoue sans garantie, ce que dit le commentaire du test.
- [ ] **Étape 2 : voir le rouge, puis le vert.** Le rouge se lit en faussant un temps l'attendu de `deux_regroupements_d_une_meme_prise_ne_la_montrent_que_dans_une_idee` (deux idées) : `panicked at`, puis l'attendu juste ; verts dans `verifier` et `windows`. Le délai de `windows` se relit (les seize cas de trois répliques sur fichiers).
- [ ] **Étape 3 : committer** par chemins (`test: deux et trois répliques, I10 et la vigilance 1`).

### Tâche 16 : un message en français pour chaque erreur, jamais un ordre

**Fichiers :** modifier `crates/stockage/src/lib.rs` (`impl Display` d'`Erreur`, `LectureSeule`,
`Avertissement`), `crates/stockage/src/fichier.rs` (`Echec`), `crates/synchro/src/lib.rs` (`Erreur`),
`crates/services/src/application.rs` (`Inapplicable`), `crates/noyau/src/donnees/copie.rs` (`Difference`, que rend
`synchro::Echange::ouvrir` devant deux copies étrangères, § 31 · 2.5 : « et Pupitre le dit »),
`crates/noyau/src/donnees/teneur.rs` (`ErreurDeTeneur`), les conversions en `Refus` (`crates/stockage/src/adaptateurs.rs`,
`crates/services/src/en_memoire.rs`), et les tests `crates/stockage/tests/messages.rs` (créé), `crates/services/tests/suite/mod.rs` (le message de
`une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme`),
`crates/stockage/tests/commandes_sur_fichiers.rs`, `crates/services/tests/application.rs`,
`crates/noyau/tests/donnees.rs` et `crates/noyau/tests/teneur.rs` (les messages des deux types du noyau : `copie.rs`
et `teneur.rs` sont mutés, et seuls les tests de la liste de `.cargo/mutants.toml` tuent les mutants de leur
`Display`).

**Interfaces — produit :** `impl std::fmt::Display` pour chacun de ces types ; le message d'un `Refus` devient leur
`to_string()`.

Les règles : un constat, au vocabulaire du glossaire ; un chemin s'écrit par ses composants joints par `/`, le même
texte sous Linux et sous Windows. Les erreurs gardent le chemin absolu qu'elles portent (aucun test de la tranche 1
ne change) : l'adaptateur sur fichiers, qui connaît la racine des données, rend chaque chemin d'une `Erreur`, d'une
`LectureSeule` ou d'un `Avertissement` relatif à elle (`strip_prefix(racine.chemin())`) avant d'en former le message
d'un `Refus` ou d'un avertissement : le fichier se nomme `documents/fonds/ajouts/<e>.automerge`. Ce chemin, relatif,
est le sujet de la phrase, sans article : il nomme un fichier ou un dossier (`Erreur::Disque` porte aussi celui d'un
dossier, `documents/fonds/ajouts` ou `captations`), et « le fichier » dirait faux d'un dossier. Le texte d'une
`io::Error` ou d'une `AutomergeError` n'est jamais recopié, mais dit par une table de `io::ErrorKind`, qui forme le
constat sur un sujet `S`, le chemin ou « Le fichier écrit à côté » : `NotFound`, « S est absent » ;
`PermissionDenied`, « S est refusé par le système » ; `StorageFull`, « S ne tient plus sur le disque » ;
`InvalidData`, « S ne se relit pas tel qu'il a été écrit » ; tout autre genre, « S, le disque l'a refusé », neutre
quant au geste : `Erreur::Disque` couvre une lecture, une écriture, une création, un renommage, un retrait ou le
renommage rendu durable (`crates/stockage/src/lib.rs:41-43`), et « n'a pas pu s'écrire » dirait faux d'une lecture
refusée. Pour Automerge, le texte dit ce qui ne se fait pas. Le refus N + 1 : « Ce fonds vient d'une version plus
récente de Pupitre ; celle-ci le lit sans y écrire. » (Éc-12). Le texte de chaque variante, comparé en entier,
`{c}` étant un chemin joint par `/`, `{table}` le constat de la table sur le sujet que la ligne nomme :

| Type, variante | Texte |
|---|---|
| `Erreur::RacineDesDonneesTenue` | « Un autre processus tient la racine des données. » |
| `Erreur::Disque { chemin, erreur }` | « {table, sur {c}}. » : « documents/fonds/complet.automerge est absent. », « documents/fonds/complet.automerge, le disque l'a refusé. » |
| `Erreur::CaptationAlteree { empreinte }` | « Le contenu relu de la captation {empreinte} ne donne pas son empreinte. » |
| `Erreur::CaptationAbsente { empreinte }` | « Aucune captation ne porte l'empreinte {empreinte}. » |
| `Erreur::FondsDejaOuvert` | « Le fonds est déjà ouvert dans ce processus. » |
| `Erreur::FondsDejaNe` | « Le fonds est déjà né : il ne naît qu'une fois. » |
| `Erreur::Automerge(_)` | « Le document a refusé ce geste, ou sa racine ne se lit pas. » |
| `Erreur::LectureSeule(l)` | le texte de `l` |
| `Erreur::CompactionInfidele` | « Le fichier complet de la compaction, relu, ne mène pas au document en mémoire : aucun fichier d'ajout n'est retiré. » |
| `Erreur::ChampAbsent { champ }` | « La racine du document ne porte pas de chaîne sous « {champ} » : la copie ne se situe pas. » |
| `Erreur::Synchro(e)` | le texte de `e` |
| `Erreur::Inapplicable(i)` | le texte de `i` |
| `LectureSeule::RelectureManquee { chemin }` | « La relecture de {c}, après une écriture refusée, a échoué : le document montre ce qu'il était avant elle, et se lit sans écrire ni échanger. » |
| `LectureSeule::AjoutAltere { chemin }` | « Le contenu de {c} ne donne pas l'empreinte que porte son nom : il n'est pas chargé, et le document se lit sans écrire ni échanger. » |
| `LectureSeule::AjoutIllisible { chemin }` | « {c} donne l'empreinte de son nom, mais ne se charge pas : le document se lit sans lui, sans écrire ni échanger. » |
| `LectureSeule::CompletIllisible { chemin }` | « {c} ne se lit pas en entier : le document montre ce qui s'en lit, sans écrire ni échanger. » |
| `LectureSeule::DependancesManquantes { chemin }` | « {c} attend une modification qu'aucun fichier lu ne porte : un fichier d'ajout manque, et le document se lit sans écrire ni échanger. » |
| `LectureSeule::VersionPlusRecente { version }` | « Ce fonds vient d'une version plus récente de Pupitre ; celle-ci le lit sans y écrire. » |
| `LectureSeule::VersionIllisible` | « La version de schéma du document ne se lit pas : il se lit sans écrire ni échanger. » |
| `LectureSeule::MigrationRefusee` | « Une migration de ce fonds ne s'applique pas : il se lit sans écrire ni échanger. » |
| `Avertissement::NonDurable { chemin, erreur }` | « {c} est en place, mais son renommage n'est pas rendu durable. » |
| `Avertissement::Compaction(e)` | « La compaction a échoué, sans rien perdre, et se retente à l'écriture suivante : {le texte de `e`} » |
| `Avertissement::VoisinRestant { chemin, erreur }` | « Le voisin {c} n'a pas pris la place de son fichier : il se lit où il est, et la prend à l'ouverture suivante. » |
| `Avertissement::NomProvisoireRestant { chemin, erreur }` | « {c} n'a pas pu se lire, ou prendre le nom de son empreinte : il reste, et l'ouverture suivante le relit. » |

Les cinq types qui ne portent pas de fichier :

| Type, variante | Texte |
|---|---|
| `Echec::Refusee(e)` | « {table de `e.kind()`, sur « Le fichier écrit à côté »} : celui qu'il devait remplacer n'a pas changé. » (« Le fichier écrit à côté, le disque l'a refusé : celui qu'il devait remplacer n'a pas changé. ») |
| `Echec::NonDurable(_)` | « Le fichier est en place, mais son renommage n'est pas rendu durable. » |
| `synchro::Erreur::MessageIllisible(_)` | « Le message reçu ne se lit pas : le document n'a pas changé. » |
| `synchro::Erreur::Automerge(_)` | « Le message reçu ne s'applique pas au document. » |
| `Inapplicable::ObjetAbsent { chemin }` | « L'opération vise un objet absent du fonds : {chemin, joint par `/`}. » |
| `Inapplicable::Vide` | « L'opération ne change rien au fonds. » |
| `Inapplicable::Automerge(_)` | « Le document ne se lit pas. » |
| `Difference::AutreProprietaire` | « Ces deux copies n'ont pas le même identifiant de propriétaire : elles ne s'échangent rien. » |
| `Difference::AutreDocument` | « Ces deux copies ne sont pas du même document : elles ne s'échangent rien. » |
| `Difference::AutreLignee` | « Ces deux copies ne sont pas de la même lignée : elles ne s'échangent rien. » |
| `ErreurDeTeneur::NaN { chemin }` | « La teneur ne s'écrit pas : la valeur {chemin, joint par `/`} n'est pas un nombre. » |
| `ErreurDeTeneur::Inconnu { chemin, type_code }` | « La teneur ne s'écrit pas : la valeur {chemin} est d'un type que cette version de Pupitre ne connaît pas (code {type_code}). » |

- [ ] **Étape 1 : écrire les tests.** `tests/messages.rs` : `chaque_erreur_a_son_message_en_francais`, une ligne par variante d'`Erreur`, de `LectureSeule`, d'`Avertissement`, d'`Echec` et de `synchro::Erreur`, son texte attendu en entier, chaque chemin relatif et de plusieurs composants (`Path::new("documents").join("fonds").join("ajouts").join(…)`, donc `\` sous Windows, `/` dans le texte attendu), une `io::Error::new(ErrorKind::PermissionDenied, "Access is denied. (os error 5)")` comprise, dont le texte anglais ne paraît pas, et une ligne par genre de la table ; dont le refus d'une lecture à l'ouverture, `Erreur::Disque` sur `documents/fonds/complet.automerge` avec `io::Error::new(ErrorKind::Other, "read failed")`, qui rend « documents/fonds/complet.automerge, le disque l'a refusé. », sans « s'écrire » ; `aucun_message_n_est_un_ordre` : aucun de ces textes, ni ceux d'`Impossible`, d'`Inapplicable`, de `Difference` et d'`ErreurDeTeneur`, ne contient « il faut », « veuillez », « vous devez », ni l'un des impératifs « mettez », « essayez », « réessayez », « vérifiez », « relancez », « choisissez », « ouvrez », « fermez », en casse indifférente. `tests/application.rs` des services : le texte de chaque variante d'`Inapplicable`, en entier. Au noyau, `tests/donnees.rs` : celui de chaque `Difference` ; `tests/teneur.rs` : celui de chaque `ErreurDeTeneur`. Dans `commandes_sur_fichiers.rs`, `une_commande_coupee_avant_le_renommage_est_refusee` (tâche 6) vérifie que le message de son refus nomme le fichier par un chemin relatif, qu'il contient « documents/fonds/ », et pas le texte de `racine.chemin()`. Le message de `une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme` (tâche 5) s'y ajoute, en assertion de la suite.
- [ ] **Étape 2 : voir le rouge.** Committer, pousser ; attendu à « Lint » : `Erreur doesn't implement std::fmt::Display`.
- [ ] **Étape 3 : écrire le code** ; la doc de `stockage::Erreur` (`crates/stockage/src/lib.rs:35-36`) ne dit plus « vient avec les commandes ».
- [ ] **Étape 4 : voir le vert**, dans `verifier` et `windows` (le même texte sous Linux et sous Windows : la table et les chemins joints par `/` le garantissent) ; `mutants`, sur le diff, tue les mutants des deux `Display` du noyau, sans `MISSED`.
- [ ] **Étape 5 : committer** par chemins (`feat: un message en français pour chaque erreur, qui dit ce qui est`).

### Tâche 17 : les contrôles statiques du noyau et des services, et l'étape `cargo tree`

**Fichiers :** créer `crates/noyau/tests/statique.rs`, `crates/services/tests/statique.rs` ; modifier
`.github/workflows/ci.yml` (`verifier`, après « Déposer le Cargo.lock résolu », :90-95, avant « Format », l'étape
« Le graphe de l'interface » : placée après les lints, elle ne tournerait pas à l'essai de l'étape 2, qu'arrêtent
« Cargo.lock à jour » puis, `getrandom` n'ayant pas `wasm_js`, « Lint de l'interface (WebAssembly) »).

Chaque contrôle est une fonction `fn controler(sources: &[(String, String)]) -> Result<(), Vec<String>>`, qui rend
les fichiers et les mots fautifs ; les sources se lisent depuis `env!("CARGO_MANIFEST_DIR")`, en sous-chaîne,
sensible à la casse, commentaires compris (`app/ui/tests/encre_statique.rs:318-322`), et un contrôle qui ne lit
aucune source échoue.

- **Le noyau** (`crates/noyau/src/`, récursif) : ni `SystemTime`, `Instant`, `std::time`, `time::`, `getrandom`,
  `rand::`, `random`, `OsRng`, `std::fs`, `fs::`, `File`, `std::io`, `io::`, `std::env`, `env::`, `env!`,
  `std::thread`, `thread::`, `std::net`, `net::`, `std::process`, `process::`, `HashMap`, `HashSet`, `RandomState`
  (`BTreeMap` et `BTreeSet` admis) ; et `crates/noyau/Cargo.toml` porte
  `ulid = { version = "3.0.0", default-features = false }`, cette ligne entière, égale à l'octet, et aucune clé `features` (hors `default-features`) : la sous-chaîne `features =` paraît dans `default-features = false`, et un contrôle qui la refuserait refuserait le dépôt sain. Les chemins courts attrapent un
  import groupé (`use std::{fs, path::Path};` puis `fs::read`) ; aucun ne paraît aujourd'hui dans le noyau
  (`grep -rnE "fs::|env::|process::|thread::|net::|io::|time::" crates/noyau/src`, vide sur `main`).
- **Les services** (`crates/services/src/`) : ni `std::fs`, `fs::`, `File`, `std::thread`, `thread::`, `spawn`,
  `std::net`, `net::`, `TcpStream`, `UdpSocket`, `SystemTime`, `Instant`, `std::time`, `time::`, `std::env`,
  `env::`, `env!`, `std::process`, `process::`, `HashMap`, `HashSet`, `RandomState` (`BTreeMap` et `BTreeSet`
  admis) (décision 13 ; l'aléa n'y est pas interdit : l'acteur d'Automerge s'y
  tire, exception que la décision 10 cite, où qu'il soit tiré).
- **I28** (chaque `crates/*/src/` et `app/*/src/`) : hors de `crates/services/src/application.rs`,
  `crates/stockage/src/documents.rs` et `crates/stockage/src/racine_des_donnees.rs`, aucun de `.put(`,
  `.put_object(`, `.delete(`, `.increment(`, `.splice(`, `.splice_text(`, `.insert_object(`, `.transaction(`,
  `.transact(`, `.transact_with(`, `.empty_commit(`, `.commit_with(`, `.commit(`, `AutoCommit`, `Transactable`,
  `automerge::transaction`, `Transaction<` ; `receive_sync_message` (`crates/synchro/src/lib.rs:78`) n'en est pas :
  une reprise, que le § 70 · 3 admet, et le commentaire du contrôle le dit. Et les fonctions publiques qui écrivent,
  `services::application::{appliquer, naitre, faire_naitre, migrer}`, ne s'appellent que depuis `crates/services/src/`,
  `crates/stockage/src/documents.rs` et `crates/stockage/src/racine_des_donnees.rs` : ailleurs, aucun de
  `application::appliquer(`, `application::naitre(`, `application::faire_naitre(`, `application::migrer(`, qui portent
  une parenthèse que les docs qui les nomment n'ont pas (revue de la PR (a) : I28 statique ne voit pas un appel de
  l'API publique) ; ni une ligne `use` qui porte `application` et l'un des quatre noms (`use
  services::application::appliquer;`, puis `appliquer(`, que la forme qualifiée laisserait passer). Un `use` écrit
  sur plusieurs lignes échappe au contrôle, et son commentaire le dit ; le type `services::application::Inapplicable`,
  que nomme `crates/stockage/src/lib.rs`, reste admis.
- **Pourquoi `HashMap`, `HashSet` et `RandomState`** (plan maître, « Outils et cas limites », ajouté le
  2026-11-13) : sous leur hacheur par défaut, `new()`, `default()` et `collect()` tirent leur graine du système
  sans qu'aucun mot de l'aléa ne paraisse, et l'ordre d'itération change d'un passage à l'autre ; une règle qui
  en dépendrait ne serait plus déterministe. Aucun ne paraît aujourd'hui dans `crates/noyau/src/`
  (`grep -rn "HashMap\|HashSet\|RandomState" crates/noyau/src`, vide sur `main`). Ils restent permis dans les
  tests, hors de `src/` (`crates/noyau/tests/guitare.rs:1` s'en sert), et dans les autres crates, où le contrôle
  d'I30 (tranche 3) ne les cherche pas : limite que la description de la PR (c) dit.
- **L'identifiant de propriétaire** : `uuid_v4(` ne paraît, hors de sa définition, qu'une fois, dans
  `crates/services/src/lib.rs`.

L'étape de CI :

```yaml
      # L'interface ne tire ni automerge, ni les services, ni le stockage (décision 7) : en natif
      # et en WebAssembly. La sortie se capture d'abord : une erreur de cargo tree fait échouer
      # l'étape (bash -e), au lieu de passer pour une absence. L'essai sur le stockage prouve que
      # la recherche trouve une dépendance, jamais la ligne du paquet racine, qui n'a pas de
      # préfixe. Placée avant les lints : un graphe fautif les ferait échouer avant elle.
      - name: Le graphe de l'interface
        run: |
          interdits='^(automerge|services|stockage) v'
          sortie=$(cargo tree -p stockage -e no-dev --prefix none)
          grep -Eq '^automerge v' <<<"$sortie"
          grep -Eq '^services v' <<<"$sortie"
          for cible in "" "--target wasm32-unknown-unknown"; do
            sortie=$(cargo tree -p interface -e no-dev --prefix none $cible)
            if grep -E "$interdits" <<<"$sortie"; then exit 1; fi
          done
```

- [ ] **Étape 1 : écrire les tests** : pour chaque contrôle, `chaque_controle_refuse_ce_qu_on_y_plante` (une source inventée qui porte `tx.put(` dans `crates/interface_essai/src/lib.rs`, puis une autre qui y appelle `services::application::appliquer(&mut doc, &operation)`, une autre qui porte `use services::application::appliquer;` puis `appliquer(&mut doc, &operation)`, un `use std::fs;` dans le noyau, puis un import groupé, `use std::{fs, path::Path}; fn f() { let _ = fs::read("x"); }`, dans le noyau et dans les services, un `std::thread::spawn` dans les services, un `HashMap::new()` dans le noyau et dans les services, un second
`uuid_v4(`, un `Cargo.toml` du noyau dont la ligne d'`ulid` porte aussi `features = ["std"]` : chacun nommé dans le `Err` ; la ligne exigée seule, `default-features = false` compris, admise), puis le contrôle sur le dépôt : `Ok(())`.
- [ ] **Étape 2 : voir le rouge**, sur l'essai planté (le contrôle n'existe pas encore : `cannot find function controler`), puis le vert ; l'étape « Le graphe de l'interface » verte, puis rouge à l'essai : `services` ajouté à `app/ui/Cargo.toml`, un envoi rouge à « Cargo.lock à jour », l'artefact `cargo-lock` committé, un envoi rouge à « Le graphe de l'interface » (`services v` et `automerge v` imprimés) ; puis `services` retiré et le `Cargo.lock` remis, vert (les journaux liés dans la description).
- [ ] **Étape 3 : committer** par chemins (`test: les contrôles statiques du noyau, des services et d'I28`, `ci: le graphe de l'interface, sans automerge, services ni stockage`).

### Tâche 18 : la première mesure du poids

**Fichiers :** créer `crates/services/src/mesure.rs`, `crates/stockage/tests/mesure.rs` ; modifier
`crates/services/src/lib.rs`, `crates/services/tests/commandes_en_memoire.rs`, `.github/workflows/ci.yml`
(`verifier`, après « Tests », :112, l'étape « Le fonds de mesure »).

**Interfaces — produit :**

```rust
/// Le fonds de mesure du test 13, la part de la tranche 2 (Éc-1 à Éc-4), par les seules opérations :
/// 6 000 fiches de prises versées (`importee`, par l'établi, depuis un import tiré par groupe de
/// trois), chaque groupe regroupé dans sa première idée (4 000 regroupements, 4 000 idées retirées,
/// 2 000 visibles) ; pour chaque idée visible, trois textes complétés, chacun d'au moins 200 octets
/// (203 en moyenne), et trois étiquettes ; 1 000 liens, l'idée 2k reliée à l'idée 2k + 1. L'aléa
/// est le sien, déterministe (splitmix64, graine fixe) ; l'heure avance d'une seconde par
/// opération, depuis `heure`, sous un décalage de 0. `engendrer_le_fonds_de_mesure_selon(services, heure, 2_000)`.
pub fn engendrer_le_fonds_de_mesure<D: Documents, C: Captations>(services: &mut Services<D, C>, heure: i64)
    -> Result<Engendre, Refus>;
/// Le même fonds sur `groupes` groupes de trois prises, au lieu de 2 000 : publique, pour que le
/// test d'intégration `le_generateur_est_deterministe` l'appelle sur deux groupes.
pub fn engendrer_le_fonds_de_mesure_selon<D: Documents, C: Captations>(services: &mut Services<D, C>,
    heure: i64, groupes: usize) -> Result<Engendre, Refus>;
pub struct Engendre { pub idees_visibles: usize, pub idees_retirees: usize, pub operations: usize }
```

Les textes : des mots inventés de deux ou trois syllabes, tirés de `["la", "mi", "ré", "do", "sol", "fa", "si",
"ta", "po", "nu", "ver", "bri", "cla", "mon", "ton", "pen", "lu", "sa", "gra", "fle", "cor", "dan", "vo", "ri"]`,
séparés d'une espace, ajoutés tant que le texte fait moins de 200 octets : chaque texte en fait de 200 à 209, 203,3
en moyenne (calculé sur le poste, 200 000 tirages ; la règle « jusqu'à 180 octets » donnait 183,3 en moyenne,
jamais plus de 189, sous les « 200 octets de texte en moyenne » du test 13) ; les étiquettes, trois distinctes de
deux syllabes ; les octets d'une prise, seize octets de l'aléa ; la durée, de 5 000 à 600 000 ms ; le format, `wav`
(une prise `flac` n'aurait pas de durée à INC-1, voir la tâche 10).

- [ ] **Étape 1 : écrire le test** (`crates/stockage/tests/mesure.rs`) :

```rust
// § 31 · 10, test 13 : 2 000 idées, 200 octets de texte en moyenne, chaque texte écrit trois fois, trois étiquettes et un lien par idée
#[test]
#[ignore = "le fonds de mesure : une étape à part de verifier, en --release"]
fn le_fonds_de_mesure_pese_moins_de_4_500_000_octets() {
    // Engendré sur la paire en mémoire ; Engendre { 2 000, 4 000, 23 000 } ; `octets()` écrits en
    // `documents/fonds/complet.automerge` d'une racine jetable (`commun::dossier_jetable`) ; rouvert par
    // `FondsSurFichiers::ouvrir` (qui appelle `fonds()`, entier), sous `Services` : `Services::idees()` en rend 2 000 ; la moyenne des textes de ces 2 000 idées, relus, est
    // d'au moins 200 octets ; le poids du fichier `< 4_500_000`, imprimé, avec la durée.
}
```

  Et, dans `crates/services/tests/commandes_en_memoire.rs`, `le_generateur_est_deterministe` : `engendrer_le_fonds_de_mesure_selon(…, HEURE, 2)`, publique (un test d'intégration est une crate à part, qui ne voit pas une fonction `pub(crate)`), deux fois sur deux paires neuves : `Engendre { idees_visibles: 2, idees_retirees: 4, operations: 23 }` et la même teneur ; chaque texte des deux idées visibles fait au moins 200 octets.
- [ ] **Étape 2 : voir le rouge.** L'étape : `- name: Le fonds de mesure (test 13, la part de la tranche 2)` / `run: cargo test -p stockage --release --test mesure -- --ignored --nocapture`, commentée (hors de `windows` ; `--release`, écart 7 ; la tranche 5 la complète). Committer, pousser ; attendu à « Lint » : `cannot find function engendrer_le_fonds_de_mesure`.
- [ ] **Étape 3 : écrire le code.**
- [ ] **Étape 4 : voir le vert.** Relever dans le journal de l'étape le poids et la durée, à recopier dans la description ; si le poids passe 4 500 000 octets, le test reste rouge, et la PR le dit à la maîtrise d'ouvrage avant toute autre chose.
- [ ] **Étape 5 : committer** par chemins (`feat: le générateur du fonds de mesure et sa première mesure`).

### Tâche 19 : les documents de la PR (c), LE-14, les délais, et la revue de la PR (c)

**Fichiers :** modifier `docs/31-persistance-et-synchronisation.md` (§ 2.4, :377-422, :410-411 ; § 2.6, :445-484),
`docs/20-sfd/21-capter-et-organiser.md` (CAP-01, cas limite, :68-69), `docs/80-glossaire.md` (« Lecture seule (d'un
document) »), `docs/pistes.md`, `CLAUDE.md`
(« Commandes »), `CONVENTIONS.md` (ligne `verifier`, bloc « Les lancer »), `docs/registre.md` (LE-14),
`.github/workflows/ci.yml` (`verifier`, `windows`, si la mesure le demande).

Tout ce qui part avec la PR (c) se pousse ici, avant sa revue : la revue lit la tête qui part, et la tâche 20 ne
pousse plus rien.

- [ ] **Étape 1 : le § 31 · 2.4 et CAP-01.** « un bandeau dit qu'il vient d'une version plus récente de Pupitre, que celle-ci le lit sans y écrire » ; CAP-01 : « Pupitre dit que ce fonds vient d'une version plus récente, qu'il lit sans y écrire » : des constats (Éc-12). Le § 31 · 2.4 gagne aussi : « Une migration qui ne s'applique pas ouvre le document en lecture seule, sans échange. » (`LectureSeule::MigrationRefusee`, revue de la PR (a)) ; le glossaire, à l'entrée « Lecture seule (d'un document) », range de même « une migration qui ne s'applique pas » parmi ce qui fait qu'il n'envoie ni ne reçoit. La recherche de LE-3 : `git grep -n -i -E "faut mettre|mettre (à|l'application)" -- .`, chaque sortie relue (le § 31 · 2.4 coupe aujourd'hui la phrase sur deux lignes, « il faut mettre à » / « jour l'application », qu'une recherche de la phrase entière ne trouve pas ; `docs/90-questions-ouvertes.md:117`, « le mettre à jour quand l'outil change », est d'un autre sujet) : aucune ligne du sujet hors du registre et des plans. **Le § 31 · 2.6** : « Pupitre n'écrit ni `Counter`, ni objet `Text` : un test le vérifie sur les documents qu'écrivent les opérations nommées » (Éc-18).
- [ ] **Étape 2 : les pistes.** `docs/pistes.md` : « Deux regroupements croisés, ou un même fichier importé sur deux appareils, laissent une idée vide visible, ou deux idées retirées qui portent une prise (`deux_regroupements_croises_retirent_les_deux_idees`, `une_prise_nee_sur_deux_repliques_ne_cache_aucun_de_ses_faits`) : le relevé des conflits du lot 3 les montre ; la tranche 5 dit le sort de l'idée laissée par l'import. » Et : « Une étiquette ou un lien posés des deux côtés, ou un fait d'une captation versée des deux côtés, laissent deux valeurs égales sous une clé (`une_etiquette_posee_des_deux_cotes_n_en_fait_qu_une`, `cap02_7_…`) : le relevé des conflits du lot 3 ne les présente pas « à trancher » (Éc-9). »
- [ ] **Étape 3 : `CLAUDE.md` et `CONVENTIONS.md`.** « Commandes » : `cargo test -p stockage --release --test mesure -- --ignored --nocapture`, la mesure du test 13 ; la ligne `verifier` : l'étape du graphe de l'interface et celle du fonds de mesure, et, parmi ce qu'elle refuse, une écriture hors des deux adaptateurs (I28), un champ engendré (I01), une horloge, un aléa ou un fichier au noyau, un fichier, un fil, un réseau ou une horloge dans les services. `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add`. Committer par chemins (`docs: le refus d'un fonds plus récent, un constat ; les pistes des regroupements croisés`), pousser.
- [ ] **Étape 4 : remesurer, à froid.** Les délais de `verifier` (l'étape du fonds de mesure comprise, et sa construction en `--release`) et de `windows` se ramènent à deux fois un passage mesuré à froid (`.github/workflows/ci.yml:72-75`, LE-29 de modele-projet) : `gh cache delete` des clés `v0-rust-verifier` et `v0-rust-windows` de `main` et de la branche, puis un passage de `ci` (`gh run rerun <n>`, qui rejoue l'événement d'origine, sans dommage pour `ci` ; jamais pour `texte-pr`) ; « No cache found. » dans le journal de rust-cache de chaque job ; `gh run view <n> --json jobs` : les deux durées. Si deux fois l'une dépasse son `timeout-minutes`, il passe à deux fois la mesure, arrondie à la minute au-dessus, et son commentaire dit « à froid », la mesure, la date et la PR ; sinon, la description donne les deux mesures. Committer par chemins, s'il le faut (`ci: les délais remesurés à froid avec la suite des commandes et la mesure`), pousser.
- [ ] **Étape 5 : le registre.** LE-14, colonne « État » : « clos le AAAA-MM-JJ : les PR (a) #NN et (b) #NN fusionnées, et (c) #NN, qui le clôt ; corrigés : § 40 · 3 (a), § 41 · 6 (c), § 31 · 2 et 2.3, § 30 · 1, 9.3 et 11, et CAP-02 (b), § 31 · 2.4, 2.6 et CAP-01 (c) ; les questions 1 à 7 tranchées le 2026-11-12 ». `node scripts/verif-liens.mjs` et `node scripts/verif-faits.mjs` verts après le `git add`. Le corps final de la PR écrit (voir la tâche 20), committer par chemins (`docs: LE-14 clos avec la tranche 2`), pousser.
- [ ] **Étape 6 : la revue de la PR (c) entière**, sur sa tête, LE-14 et les délais compris ; ses retours traités, poussés ; puis la relecture des corrections, sur la tête qui part, dont le SHA complet est celui que la tâche 20 passe à `--match-head-commit`. Un commit poussé après cette relecture la relance.

### Tâche 20 : la clôture de la tranche

**Fichiers :** aucun : la tâche ne pousse rien. Un commit qu'elle demanderait ramène à la tâche 19, étape 6.

- [ ] **Étape 1 : vérifier la fin.** **Fin de la PR (c), et de la tranche :** tous les jobs verts sur la tête relue ; la description relève, dans `verifier`, les lignes `test result: ok.` de `identifiants.rs`, `gestes.rs`, `idee.rs`, `statique.rs` (noyau), `donnees.rs` et `teneur.rs` (noyau), `serialisation.rs`, `application.rs`, `commandes_en_memoire.rs`, `statique.rs` (services), et, dans `verifier` et `windows`, celles de `commandes_sur_fichiers.rs`, `repliques.rs`, `messages.rs`, `genese.rs`, `references.rs`, `demarrage.rs` et de `Doc-tests stockage` ; le poids et la durée de la mesure, une borne basse (Éc-4 : ni noms d'origine, ni stems, ni origines déclarées, ni 3 000 acteurs) ; les écarts 7 (`--release`) et 10 (tâche 15) ; les délais remesurés à froid ; I01, I10 et I28 verts ; LE-14 clos ; le tableau du § 70 · 4.1 porte, de la tranche, CAP-01 · 6 et CAP-02 · 7, 8, et rien d'autre, chacun avec la phrase de sa tâche (5, 11, 13), et les parts de CAP-02 · 3, 4, 5 et 6 ont leurs tests nommés, sans inscription. Une retouche du corps ne relance que `texte-pr`, jamais par `gh run rerun`.
- [ ] **Étape 2 : fusionner.** `gh pr ready <n>`, puis la fusion depuis le clone principal, avec `--squash` et `--match-head-commit <SHA complet de la tête relue à la tâche 19>` (contraintes générales).

---

## Auto-relecture

**La couverture**, exigence → tâche. Les numéros E01 à E64 sont ceux du relevé des exigences, dont la description
de la PR de ce plan porte la table ; chacun est suivi de sa source.

| Exigences (source) | Tâches |
|---|---|
| E01 (plan maître, tranche 2, « But ») ; E02 (« Contraintes générales ») ; E03 (même section : le noyau pur, les chemins reçus, aucune donnée réelle) | 4, 5, 6 ; contraintes générales ; 17 |
| E04 (contraintes : le glossaire, LE-2, LE-3) | 7, 13, 19 |
| E05 (décision 21, question 3 : les critères inscrits, nommés, l'exemple calculé) ; E57 (§ 70 · 4.1) | 5, 11, 12, 13 |
| E06 (les passages avant le plan, la relecture par une session neuve) | « Ce que les passages ont levé » ; la PR de ce plan |
| E07 (chaque PR : son texte, ses dépendances, ses références, ses délais) ; E08 (la forme du plan, le plan maître intouchable) | la fin de chaque PR (7, 13, 19 et 20) ; « Les écarts au plan maître » |
| E09 (`commandes`) ; E17 (décision 11 : la même sérialisation) | 3 |
| E10 (`services`) ; E14 (`ci.yml`, la configuration livrée compilée) | 4 ; 3, 4 (lint, tests, étape sans `en-memoire`) |
| E11 (`stockage` réalise les contrats ; `synchro` ne change pas) ; E15 (le doctest) ; E43 (faite, panne avant, panne après) | 6 ; `synchro` ne gagne que le `Display` de son erreur (16) |
| E12 (la suite générique, sur les deux paires) | 5, 6, puis 11, 12, 14 |
| E13 (décision 8 : le noyau décide, l'adaptateur applique) | 2, 4, 10 |
| E16 (qui conduit la migration) ; E44 (le refus N + 1, sur les deux paires) | 4, 5, 6 |
| E18 (les ULID) ; E19 (l'UUID, un seul service) ; E20 (l'heure et l'aléa, des arguments) | 1, 5, 17 |
| E21 (décision 17 : le budget) | 1, 3, 4 ; aucune autre dépendance |
| E22 (le schéma de la `v0.3.0`) ; E23 (la vigilance 1 dans le schéma) ; E24 (la genèse en octets) ; E61 (§ 31 · 2 et `fonds`) | 2, 8, 13 ; la vigilance 1 éprouvée en 15 |
| E25 (les références du test 11) | 8, 13 |
| E26 (le fonds naît au premier geste qui écrit) ; E27 (la naissance) ; E28 (noter) | 2, 5, 6 |
| E29 à E33 (compléter, étiqueter, relier, regrouper, retirer et rétablir) ; E34 (les préconditions) ; E35 (les messages) | 9, 10, 11 |
| E36 (annuler) ; E37 (annuler, écrit au § 30 · 11 et dans CAP-02) | 12 ; 13 |
| E38 (reliée) ; E39 (la durée) ; E40 (l'assiette) | 9 |
| E41 (les empreintes nommées, étendues) | 8 |
| E42 (la mutation suit le noyau) | 1, 2, 9 (modules) ; 7, 13 (remesures) ; 16 (les `Display` de deux modules mutés) |
| E45 (I28, statique et sur l'historique) ; E46 (I01) ; E47 (ni `Counter` ni `Text`) | 14, 17 |
| E48 (I10, test 3) ; E49 (à deux répliques) ; E50 (la vigilance 1 à deux répliques) | 13 (CAP-02 · 7), 15 |
| E51 (un message en français pour chaque erreur) | 16, 19 |
| E52, E53 (les contrôles statiques) ; E54 (`cargo tree`) ; E55 (aucun job nouveau) | 17 ; 18 (une étape, pas un job) |
| E56 (le fonds de mesure) | 18 |
| E58 (les critères de sortie) | 19, 20 |
| E59 (§ 40 · 3) ; E60 (§ 41 · 6) ; E62 (`CLAUDE.md`) ; E63 (`CONVENTIONS.md`, `.cargo/mutants.toml`, `ci.yml`) | 3, 4, 7 ; 13, 17, 18, 19 |
| E64 (le registre) | la PR de ce plan (LE-14) ; 19 |

Les clauses du § 31 · 10 : test 3, tâche 15 ; test 4, tâche 8 ; test 7, tâche 14 ; test 8, tâche 6 ; test 10,
tâche 5 ; test 11, tâches 8 et 13 ; test 13, tâche 18. Ce qui part ailleurs : la part « à trancher » du test 3
(lot 3), le test 13 entier (tranche 5).

**Les noms et les types, d'une tâche à l'autre :** `Operation`, `Changement`, `Simple` (2) servent en 4, 6, 10 et
12 ; `IdeeLue`, `CaptationLue`, `Sorte`, `Appareil`, `Provenance`, `Fiche` (2) en 4, 9, 10, 11 ; `Impossible` né en 2
(`TexteVide`), complété en 10 ; `GesteDeTri` (10) en 11 et 12, gardé par `Services::dernier_geste` ; `ulid`, `uuid_v4`,
`empreinte` (1) en 5, 11 et 18, `uuid_v4` par `identifiant_de_proprietaire` seul ; `Documents`, `Captations`,
`Alea`, `Ecriture` (4) en 5, 6, 18 ; `Refus` et `Raison` (3) partout où un refus se rend, réexportés par
`services` ; `application::appliquer`, `naitre`, `faire_naitre`, `migrer` (4) en 6 ; `lecture::empreintes_nommees`
(8) au démarrage du stockage ; `lecture::teneur` (12) en 13 et 14 ; `Paire`, `AleaDeTest`, `HEURE`, `IMPORT` de la
suite (5) en 6, 11 à 15, la suite incluse par `commandes_sur_fichiers.rs` (6), `repliques.rs` et `references.rs`
(13), le stockage recevant `commandes` en dépendance de test (6) ; `Impossible::HorsDeLImport` (10) en 11 ;
`AutreLignee` (13) ; `FondsSurFichiers` (6) en 13, 15 et 18 ; `CONTENEURS_DE_LA_GENESE` (8) en 14 ; `Engendre`,
`engendrer_le_fonds_de_mesure_selon` (18) ; `commandes::ENTIER_SUR` (3) et `gestes::ENTIER_SUR` (10), égaux, chacun
comparé à `(1 << 53) - 1` par un test de sa crate, avec `Impossible::DureeTropLongue` (10), seul refus d'une valeur
qu'une réponse ne transmettrait pas telle quelle. Les types de même nom, `Appareil`, `Provenance`, `Sorte`, s'écrivent
qualifiés (`gestes::`, `commandes::`), aux signatures (5, 11) comme dans la suite.

**La proportion :** vingt tâches pour deux crates neuves, quatre modules du noyau, la réalisation des contrats par
le stockage et trois sortes de preuves. Les corps ne s'écrivent ici que là où la forme fait la règle (le doctest,
l'étape `cargo tree`, la table des gestes, la teneur du témoin) ; le reste donne signatures, valeurs et assertions.

**Les valeurs calculées sur le poste** (`python -X utf8`, dans le dossier temporaire), avant d'écrire ce plan : les
ULID et UUID des tâches 1 et 5, les dates des messages de la tâche 2, la genèse de la tâche 8 (195 octets, par un
encodeur qui rend à l'octet la genèse d'aujourd'hui), la teneur et les empreintes du témoin de la tâche 13, et la
longueur des trois titres de PR (87, 81, 90). Puis, à la relecture : la teneur intermédiaire de la tâche 8
(`6edd898e…`, le même calcul redonnant `a349d135…` aujourd'hui) ; la teneur neuve du témoin (`ae625f61…`, par un
sérialiseur qui redonne la première, `13b6086f…`) ; les ULID `01HF7YAT00ZZZZZZZZZZZZZZZZ` et `01HF7YAT00000G000000000002` ;
la longueur des textes du générateur (183,3 octets en moyenne avec un arrêt à 180, 203,3 avec un arrêt à 200) ; les
mutants binaires de `date_du_message`, simulés sur les cas de la tâche 2 (deux survivants sans 2000 et 2100, aucun
avec). Puis, après la relecture par une session neuve : la teneur du témoin sans les deux faits de la prise et au
vocabulaire de `format` (`291a0df2…`, 1 708 octets, par le même sérialiseur, qui redonne d'abord `ae625f61…` sur
la teneur d'avant, lue dans ce plan, et `73565774…`, la valeur de la relecture, sans le vocabulaire de `format`) ;
l'ULID borné de la tâche 1 (`7ZZZZZZZZZ0000000000000000` pour 2⁴⁸ − 1 ms, `7ZZZZZZZBG0000000000000000` juste sous la
borne, `00000000AR0000000000000000` masqué sans elle) et les deux mutants de la borne, qui meurent.

**La relecture de trois relecteurs** (couverture, exécution, faisabilité ; le 2026-11-12), constat par constat
vérifié dans les sources et le code, a levé : un test d'intégration ne voit pas une fonction `pub(crate)` (le
générateur réduit devient publique) ; la suite partagée nomme `commandes`, que le stockage reçoit en test ; une
lecture ne consulte pas `lecture_seule()`, et le test 10 lit devant le fonds N + 1 ; deux répliques qui posent la même
valeur en gardent deux (`get_all`), pour chaque fait d'une captation ; CAP-02 · 4 n'est qu'une part, et CAP-01 · 6
s'inscrit dès (a) ; trois écarts au plan maître de plus (8 à 10) ; le lien sans date corrigé au § 30 · 1 ; un MIDI
versé et un texte versé, sans date ni appareil, et `importee` ; les messages de `Difference` et d'`ErreurDeTeneur`,
et les textes des cinq types sans fichier ; les chemins des messages rendus relatifs par l'adaptateur ; `uuid_v4`
par `+` et le 29 février 2000 et le 1er mars 2100, contre les mutants qui survivraient ; la puce de chaque crate avec son membre ;
l'étape du graphe avant les lints, et son essai sur une dépendance ; les imports groupés aux contrôles statiques ;
la lignée du témoin, changée par le seul deuxième tirage ; la revue de la PR (c) sur la tête qui part, et la
relecture des corrections ; la remesure à froid ; la réduction de `proptest` bornée.

**La relecture par une session neuve** (le 2026-11-12), dix-sept constats, tous retenus (« Ce que les passages ont
levé », N1 à N17), chacun cherché aussi ailleurs dans le plan, a levé : une prise qui portait sous `le` et `par`
l'heure et l'appareil du versement, des faits faux de sa captation (C3, tâches 2, 10, 11, 13, 14 et 15, la teneur
du témoin recalculée) ; CAP-02 · 6 inscrit contre le § 70 · 4.1, et chaque critère relu (C30 : CAP-01 · 6,
CAP-02 · 7 et 8 inscrits, chacun avec sa phrase ; CAP-02 · 3 à 6, des parts) ; « n'a pas pu s'écrire » pour une
lecture refusée ; l'heure d'un ULID bornée ; la durée d'une fiche bornée sous 2⁵³, et la source de chaque autre
entier d'une réponse ; `cle_du_lien` sans mutant équivalent ; deux règles de produit devenues les questions 6 et 7 ;
`format` au vocabulaire du § 30 · 9.2, jamais l'extension ; les raisons de chaque refus du stockage et de la paire
en mémoire ; la plage de la migration de test ; les imports gardés par `pannes` ; les deux mentions du nombre de
modules de `.cargo/mutants.toml` ; la clé `features` du contrôle du noyau ; le texte de chaque variante d'`Erreur`,
de `LectureSeule` et d'`Avertissement` ; une plantation par contrôle de la tâche 14, et les formes fermées de
`message_connu`.
