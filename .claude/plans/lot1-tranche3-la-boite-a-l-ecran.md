# La tranche 3 du lot 1 d'INC-1 : la boîte d'idées à l'écran — plan d'implémentation

> **Pour les agents :** exécuter ce plan avec `superpowers:subagent-driven-development`, tâche par tâche ;
> les étapes sont des cases à cocher (`- [ ]`).

Plan maître : `.claude/plans/lot1-le-fonds-sur-l-etabli.md`

**Objectif :** Sacha note et range ses idées sur l'établi. `etabli` assemble le stockage et les services sur un fil
qui les possède ; l'interface ne voit que des vues, sous la CSP du § 40 · 7.5 et des droits qui nomment chaque
commande.

**Architecture :** quatre PR. (a) pose les trois styles calculés d'INC-0 propriété par propriété. (b) donne à
`etabli` le module `systeme`, seul à lire le dossier de l'application, l'heure et l'aléa, et le fil du fonds, qui
ouvre la racine des données, possède les services et sert chaque commande reçue par un canal ; les commandes de la
tranche 2 passent par lui ; I30 a son contrôle statique et sa sentinelle. (c) pose la CSP exacte, le manifeste de
l'application, les droits de Windows, un protocole `captation` qui refuse tout, et les tests d'I31. (d) bâtit la
boîte : au noyau, l'ordre, le filtre, le compte et leurs textes, sous mutation ; dans `services`, trois lectures
(la boîte en fenêtre, les idées retirées, l'état du fonds) ; à l'écran, la boîte, le bandeau, `Ctrl+Z` ; puis
l'essai installé.

**Pile :** Rust 1.99.1, édition 2024 ; `tauri` 2.12.5 (et sa fonction `test`, en dépendance de test d'`etabli`),
`tauri-build` 2.7.4, `tauri-plugin-dialog` 2.8.4 ; nouvelles : `getrandom` (dépendance d'`etabli`, à sa dernière
version publiée, que la tâche 2 relève sur crates.io ; 0.4.3 au `Cargo.lock` le 2026-11-15), `serde_json` 1.0.154
(dépendance de l'interface, et de test d'`etabli`), `commandes` en dépendance de construction d'`etabli` ; Leptos
0.8.23 ; tests : `proptest` 1.12.0 ; en CI, `cargo-mutants` 27.1.0, trunk 0.22.1, tauri-cli 2.12.4.

**Spécification :** le plan maître, section « Tranche 3 · la boîte d'idées à l'écran », ses décisions 6, 7, 10,
11, 14, 17 et 18 ; ce qu'il cite : CAP-01, CAP-02, CAP-07 ; § 30 · 9 ; § 31 · 1, 2.1 et 2.4 ; § 40 · 7.5 et 8 ;
§ 41 · 6 ; § 50 · 4 et 5 ; § 70 · 3 (I29, I30, I31), 4.1, 7 et 8 ; ADR-006, ADR-022 ; et le plan de la tranche 2,
`.claude/plans/lot1-tranche2-services-et-commandes.md`, pour les noms qu'il a posés.

Refs #24

---

## Contraintes générales

Recopiées du plan maître (« Contraintes générales », « Le déroulé »), comme l'ont fait les tranches 1 et 2 ; chaque
tâche les suppose.

- « **En CI seulement.** Ni cargo, ni trunk, ni npm sur le poste : chaque vérification de Rust se lit dans le job que nomme le critère, le rouge puis le vert dans `verifier`, le vert aussi dans `windows` si le critère l'y met. » Les tests d'`etabli` ne tournent que dans `windows` : leur rouge se lit à « Lint de l'établi » ou à « Tests de l'établi ».
- « Seul `rustfmt --edition 2024` formate les `.rs` touchés ; les vérifications de documentation tournent sur le poste, après le `git add` (LE-7). » `rustfmt` se lance hors du dépôt, sur une copie dans le dossier temporaire, recopiée ensuite.
- « Les commandes de Rust prennent `-p <crate>`, et `ci.yml` nomme chaque crate qu'il passe à `clippy` et aux tests. »
- « **Le noyau** reste sans entrée-sortie, ni `automerge`, ni horloge, ni aléa ; chaque règle qui décide y vit, sous mutation (décision 4). »
- « **Les chemins se reçoivent** (I30) : hors de `systeme`, aucun code ne devine un chemin ; chaque test prend son dossier jetable par une seule fonction de sa crate. » Celle d'`etabli` est `dossier_jetable`, dans le module de tests de `app/src-tauri/src/lib.rs` ; chaque test d'`etabli` qui demande un dossier vit donc dans `src/`, sous `#[cfg(test)]`, et l'appelle.
- « **Aucune donnée réelle** : textes et noms inventés ; identifiants, documents et captations engendrés à l'exécution ; aucun audio, MIDI ni `.automerge` suivi. »
- « **Les mots du glossaire**, dits en entier : la racine des données et la racine du document, la version de schéma, l'origine déclarée, l'identifiant de propriétaire, le manifeste de l'application. »
- « Une PR qui ajoute un mot au glossaire le cherche d'abord (`git grep -n -i -w <mot> -- docs README.md CONVENTIONS.md`), requalifie chaque emploi antérieur et cite la recherche (LE-2) ; une PR qui périme un mot ou un identifiant le cherche dans tout le dépôt, maquettes et commentaires SVG compris (LE-3). » La recherche se refait aussi sans accent et sur la majuscule accentuée (`-i` ne replie pas les accents).
- « **Chaque écran** qu'une tranche ajoute se mène au clavier, focus visible, avec ses libellés, au vocabulaire du glossaire, sert à 200 % (§ 50 · 7), et ses feuilles passent I17 et I18 (`encre_statique.rs`). » La boîte et le bandeau sont les écrans de cette tranche.
- « Conventional Commits en français, en-tête de 100 caractères au plus, titre de 93 au plus ; « Refs #24 » » ; `git add` par chemins explicites, jamais `git add .` ; `git commit -F <scratchpad>/msg-t3-<tâche>-<étape>.txt` (un texte accentué passe par un fichier).
- « Chaque PR qui livre un critère de la SFD en entier l'inscrit au tableau du § 70 · 4.1, et un test le nomme (`cap06_7_…`) et en calcule l'exemple ; `verif-faits.mjs` refuse un critère inscrit sans test qui le nomme. La porte ne voit pas un critère livré que la PR n'inscrit pas : la revue de chaque PR compare le tableau aux critères que la tranche dit livrer. » La tranche inscrit CAP-02 · 1, CAP-02 · 3 et CAP-07 · 4 en PR (d) ; CAP-01 · 4, CAP-02 · 4, 5 et 6 gardent leurs parts, chacune avec son test nommé (question 33).
- « fusion par squash, une fois tout vert » ; mode `session` : la PR s'est ouverte en brouillon, donc `gh pr ready <n>`, puis, depuis le clone principal `pupitre`, sans `cd` ni `-R`, `gh pr merge <n> --squash --match-head-commit <SHA complet de la tête relue>`.
- « une tranche qui alourdit les tests remesure les délais de `verifier` et de `windows` ; l'installateur NSIS reste en artefact. »
- Le corps de chaque PR porte la section `## Écarts à la spec`, avec « Aucun. » : un écart à la spec se lève en corrigeant la spec dans la PR même, jamais en le déclarant. Le corps final s'écrit avant l'envoi qui le rend final (une retouche du corps ne relance que `texte-pr`, et `texte-pr` ne se relance jamais par `gh run rerun`).
- Chaque PR part de `main` à jour, depuis le clone principal (`git fetch`, puis `git worktree add ../pupitre.worktrees/<branche> -b <branche> origin/main`), s'ouvre en brouillon au premier envoi (`ci.yml` ne tourne que sur une PR), et dit dans sa description les tests réécrits et pourquoi.
- **Lire la CI** : `gh run list --branch <branche> --workflow ci --limit 1`, `gh run watch <n> --exit-status`, `gh run view <n> --log-failed`. Un rouge attendu se lit tel que la CI le rend : une erreur de compilation qui nomme ce qui manque, quel que soit le premier nom qu'elle donne, à « Lint » de `verifier` (ou à « Lint de l'établi » de `windows`) ; ou un test, à « Tests » (`panicked at`, avec le nom du test). `cargo test` s'arrête au premier fichier de tests rouge, pris dans l'ordre de leurs noms. Un vert se lit à « Tests » de `verifier`, et, pour `etabli`, `stockage` ou `synchro`, à leur étape de `windows` : `test result: ok.` pour chaque fichier touché, relevé par une file (le journal entremêle `Running` et `test result`).
- Chaque nom de `tauri` 2.12.5 (dont `tauri::test`), de `tauri-build` 2.7.4, de Leptos 0.8.23 et de `getrandom` que le code emploie se vérifie sur docs.rs avant d'écrire ; un commentaire dans un bloc de test de ce plan est une spécification, qui devient des assertions.
- Chaque test qui prouve un critère de la SFD, inscrit ou livré par parts, se nomme `capNN_n_…`, précédé de `#[test]` ; chaque test d'un invariant se nomme `i29_…`, `i30_…` ou `i31_…`.
- Les heures sont des secondes, partout où le code les reçoit ; seul `systeme` lit l'horloge (`SystemTime`), et l'interface, le décalage local, dans les seules fonctions que nomme son contrôle statique (tâches 12 et 18).
- Un numéro de ligne qu'une tâche cite vaut sur `main` à `2a7714e` : une tâche antérieure du plan peut le décaler, et l'élément nommé (la fonction, l'import, la doc) fait foi ; l'implémenteur le cherche par son nom.
- Les identifiants d'Automerge, d'ULID et de propriétaire ne s'écrivent jamais dans un message montré : le bandeau ne dit que des constats, sans chemin de la machine, sauf le refus d'ouvrir la racine des données, qui nomme son dossier (question 3).

---

## Ce que les passages ont levé

Trois passages, faits avant ce plan, ne sont pas dans le dépôt : la carte du code (pièges P1 à P17), le passage de
LE-6 (L1 à L16, et vingt-deux règles calculées sans écart) et celui d'AM-2 (C1 à C17). Leur détail a servi à écrire
ce plan ; la description de sa PR porte la table de couverture. Trente-cinq questions en sont nées, posées en un
bloc à la maîtrise d'ouvrage, qui les a tranchées le 2026-11-15, chacune selon son défaut (« ok pour les
défauts ») ; elles sont recopiées plus bas. La PR de ce plan consigne (`/consigner`), au motif « Une spec qui se
contredit » (sixième constat ; la porte existe depuis LE-11), une ligne, LE-15, qui nomme L1, L2, L3, L4, L5, L6,
L14, C2, C3, C4, C5, C10, C13 et C17 ; la tâche 20 la clôt.

### La carte du code

| Piège | Décision de ce plan | Tâche |
|---|---|---|
| P1. Les services empruntent la racine des données ; l'état de Tauri veut `'static` | Le fil du fonds possède la racine et les services ; l'état de Tauri n'est que l'`Ouverture`, qui garde l'émetteur ; le stockage ne change pas ; question 1 | 3 |
| P2. Les droits s'écrivent `allow-<nom>` ; `exporter_cahier` manque à `NOMS` ; les droits engendrés ne sont pas ignorés | `exporter_cahier` entre dans `NOMS` ; I31 compare `allow-` suivi du nom, `_` devenu `-` ; `app/src-tauri/permissions/autogenerated/` ignoré ; question 10 | 9, 11 |
| P3. `windows` ne teste ni le noyau, ni les services, ni l'interface ; `verifier` ne teste pas l'établi | Le contrôle d'I30 est un test d'`etabli` (`windows`) ; I29 a un test dans l'interface (`verifier`) et un dans l'établi (`windows`) ; question 5 | 5, 17, 18 |
| P4. Automerge tire un acteur hors du stockage (`services/src/application.rs:68`, `en_memoire.rs:52`) | L'exception de l'acteur d'Automerge nomme ces fichiers ; question 6 | 5 |
| P5. `style=` de Leptos pose un attribut, même calculé | `style:<propriété>`, seule forme que le contrôle de l'interface admet | 1, 12 |
| P6. `useHttpsScheme` vit sous `app > windows[]` ; chaque clé a une forme en kebab-case | Les tests d'I31 cherchent les deux formes, aux deux places | 11 |
| P7. Aucune lecture de la boîte ; `vue_d_une_idee` relit les trois conteneurs à chaque appel ; pas de photo au noyau | Une lecture `boite`, en un seul passage sur les conteneurs, `photos` compris ; question 15 | 14, 16 |
| P8. I18 interdit `request_animation_frame`, `set_timeout` | La liste virtualisée suit le seul événement de défilement | 19 |
| P9. Les arguments d'une commande se lisent en camelCase ; une commande sans `async` tourne sur le fil de l'IPC ; un refus rejette la promesse d'un objet | Un seul argument, `commande` ; chaque commande `async`, qui attend le fil hors de l'IPC ; l'interface lit l'objet `Refus` | 4, 18 |
| P10. `getrandom::fill` peut échouer, `Alea::remplir` non | `systeme` s'arrête par une panique au message dit ; question 2 | 2 |
| P11. Le dossier de l'application est aussi celui de WebView2 | « Sans rien écrire » s'entend sous `donnees/` ; question 4 | 3 |
| P12. Le refus d'une racine tenue n'a pas de chemin vers un `Refus` ; une erreur du disque nomme un chemin absolu | `stockage::refus_a_l_ouverture`, le chemin entier du dossier des données dit ; question 3 | 3 |
| P13. CAP-02 · 3, 4 et 6 ont des tests sans être inscrits | Question 33 | 16, 20 |
| P14. `getTimezoneOffset` vaut UTC moins l'heure locale | `decalage_depuis_js`, fonction pure, testée | 18 |
| P15. Les contrôles du noyau et des services lisent les commentaires en sous-chaîne | Les noms et les docs de la boîte évitent `Instant`, `File`, `io::`, `time::`, `HashMap` | 14, 16 |
| P16. Aucun `devCsp` ; `cargo tauri dev` suit la CSP | Aucun `devCsp`, et aucun job ne lance `cargo tauri dev` ; la PR (c) le dit | 8 |
| P17. `core:default` sans héritier probable | Aucun droit du noyau de Tauri au lot 1 ; l'essai installé le confirme | 9, 21 |

### Le passage de LE-6

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| L1. `cssText` : la CSP l'admet | Le contrôle de l'interface le refuse quand même ; le § 40 · 7.5 dit la vraie raison ; question 9 | 8, 12 |
| L2. La sentinelle ne voit ni le dossier personnel ni le dossier temporaire | Le § 70 · 3 se corrige ; le contrôle statique les nomme, chacun avec sa plantation ; question 7 | 5, 7 |
| L3. Les droits : `allow-<nom>`, aucun droit du noyau au lot 1 | Question 10 | 9, 11 |
| L4. `ipc:` n'est la forme d'aucune cible | Retiré : la CSP exacte fait 277 octets ; question 11 | 8 |
| L5. L'assiette n'a pas de forme écrite ; l'exemple du § 50 · 5 a deux nombres | Les formes de la question 18 ; le § 50 · 5 reprend l'exemple de CAP-06 | 15, 20 |
| L6. Une idée illisible sort de la boîte en silence | Comptée parmi les interrogées, « · K illisibles », absente de la liste ; question 19 | 14, 16 |
| L7. Ranger par étiquette, quand une idée en porte un ensemble | Sur la plus petite, octet pour octet ; sans étiquette, en fin ; question 20 | 14 |
| L8. Une étiquette « vide » ou précédée de `#` | Refusée au noyau une fois ses blancs ôtés ; l'interface ôte les blancs autour et un `#` de tête ; question 29 | 14, 15, 19 |
| L9. Les ordres n'ont ni sens, ni défaut, ni départage | Question 21 | 14 |
| L10. La durée affichée | Arrondie à la seconde inférieure, m:ss, h:mm:ss ; question 23 | 15 |
| L11. La date dans la liste | « 3 nov. 2026 », au décalage de l'instant ; la piste de l'écart d'un jour ; question 24 | 15, 18, 20 |
| L12. Le `—` | Dans chaque case vide, étiquettes comprises ; question 25 | 19 |
| L13. `Ctrl+Z` dans un champ de texte | Hors d'un champ seulement ; le bouton porte le message du geste visé ; question 30 | 19 |
| L14. Les critères inscriptibles ; `idee::reliee` ignore « devenue » | Question 33 ; la doc de `idee::reliee` le dit | 14, 16, 20 |
| L15. Le second processus « s'arrête en le disant » | Une fenêtre sur le seul bandeau ; question 4 | 3, 18 |
| L16. Aucun seuil ; la liste des retirées est deux fois plus longue | Des lignes de hauteur fixe ; les retirées en commande à part ; un test borne les lignes rendues ; question 26 | 16, 19 |

Les vingt-deux règles sans écart sont recopiées dans les tâches qui les livrent : l'exemple de CAP-07 · 4 (règle 1)
et CAP-02 · 1, 3 et 5 (règles 3 à 5) à la tâche 16 ; le filtre « photo » (règle 6) aux tâches 14 et 16 ; les
durées (règles 7 et 8) à la tâche 15 ; les accords (règle 9) et la forme de CAP-06 · 7 (règle 10) à la tâche 15 ;
les styles (règles 11 et 12) à la tâche 1 ; la CSP (règles 13 à 15) à la tâche 8 ; le dossier de WebView2
(règle 16) et le second processus (règle 19) à la tâche 3 ; le bandeau d'un fichier illisible (règle 17) aux
tâches 17 et 18 ; la redirection (règle 18) à la tâche 6 ; le signe du décalage (règle 20) à la tâche 18 ; le poids
de la vue (règle 21) à la tâche 16 ; les noms des droits (règle 22) à la tâche 11.

### Le passage d'AM-2

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| C1. L'assemblage engage les tranches 5, 6 et 7 | Le fil du fonds ; « Effets cherchés » dit ce que reçoivent les tranches 5 et 6 ; question 1 | 3 |
| C2. Aucune commande ne lit l'état sans geste | `etat_du_fonds` : né ou non, la lecture seule, les avertissements de l'ouverture, le geste qu'annuler vise ; question 16 | 16, 17, 18 |
| C3. « Tri », deux sens | « Tri » reste aux gestes ; l'ordre de la boîte s'écrit `Ordre`, `ordonner`, « Ranger par » ; question 17 | 14, 20 |
| C4. « Boîte d'idées » : « avant tout tri » | La boîte liste toutes les idées non retirées ; le glossaire se corrige ; question 14 | 16, 20 |
| C5. « Bandeau », deux sens ; plusieurs messages ; un rejet qui n'est pas un `Refus` | Question 31 ; le glossaire gagne « Bandeau », au sens du message | 18, 20 |
| C6. La vue de la boîte contre « des vues petites » | Une fenêtre, bornée à `FENETRE_MAX` ; question 15 | 16 |
| C7. L'écran de la tranche 3 n'a pas de vue du fonds | Un sélecteur « Table » / « Boîte d'idées » ; l'écart provisoire à CAP-07, dit dans la PR (d) ; question 27 | 19 |
| C8. Les gestes à l'écran | Compléter, étiqueter, relier, délier, retirer, rétablir, annuler ; question 28 | 19 |
| C9. Le filtre par contenu | « Texte de l'idée », « MIDI versé » au sens du glossaire, « photo » sur `photos` ; question 22 | 14, 19 |
| C10. La décision 10 contre l'interface | Elle vaut côté natif ; l'interface a sa propre règle ; question 5 | 5, 12, 18 |
| C11. Le garde-fou d'I30 sous `mock_context` | `systeme` reçoit le dossier en argument ; le garde-fou résout avec l'identifiant de `tauri.conf.json` ; question 8 | 2, 6 |
| C12. L'essai installé | Question 34 | 21 |
| C13. Le protocole des captations admis deux tranches avant d'être servi | Un gestionnaire qui refuse tout ; question 12 | 10 |
| C14. La boîte de message du greffon | Le contrôle d'I29 côté natif refuse `.message(` et `MessageDialog` ; question 13 | 17 |
| C15. Ce qu'une recherche en sous-chaîne manque | `innerHTML`, `outer_html`, `insert_adjacent_html` cherchés ; les sources de la CSP comparées entières ; `TAURI_CONFIG`, limite dite ; les droits engendrés ignorés ; `cargo tree` vise la cible Windows | 6, 9, 11, 12 |
| C16. « État », « droit », des mots de l'écran sans entrée | `Reliee` au code ; le glossaire gagne « Droits (de Tauri) », « Reliée (idée) », « Relier, délier », « interrogées » ; question 32 | 14, 20 |
| C17. Les documents au-delà de la liste | Le § 41 · 6 (`js-sys`), le § 70 · 8, le § 40 · 7.5 (la démonstration, la liste des commandes) | 7, 13, 20 |

### La relecture par une session neuve

Une session neuve a relu ce plan le 2026-11-15, selon « La revue du plan » (`docs/modele/methode.md`), chaque
mécanisme annoncé relu dans le code de `2a7714e`, chaque valeur recalculée, les noms de `tauri` 2.12.5 et de
`tauri-build` 2.7.4 vérifiés sur docs.rs : vingt-trois constats (quatre bloquants, cinq majeurs, quatorze mineurs),
tous retenus. Chacun a été cherché aussi ailleurs dans le plan, et la correction suit partout où la même croyance
paraissait.

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| R1. Bloquant : la photographie de la racine tenue lisait le fichier `verrou`, que Windows peut refuser sous le verrou (`crates/stockage/tests/demarrage.rs:791-792`) | La photographie ne lit que `documents/` et `captations/`, et de `verrou` sa présence et sa taille ; une ligne d'outil | 3 |
| R2. Bloquant : le stockage écrit un chemin avec `/` sous Windows aussi (`chemin_dit`), et le test le cherchait par `display()` | Le chemin comparé sous sa forme `/` ; le contrôle d'I29 natif cherche le nom du dossier jetable ; le refus de lecture seule comparé à un texte écrit ; une ligne d'outil | 3, 16, 17 |
| R3. Bloquant : `exporter_cahier` prend `AppHandle<Wry>`, qui ne passe pas dans le gestionnaire générique | `exporter_cahier` devient générique sur `R: tauri::Runtime` ; une ligne d'outil | 4 |
| R4. Bloquant : `NOMS` passe à quatorze en (c), quand `serialisation.rs:167` en exige treize ; le `Cargo.lock` sans origine | `serialisation.rs` passe à quatorze dans la tâche 9 ; le `Cargo.lock` de l'artefact, relu | 9 |
| R5. Majeur : `Documents::avertissements` sur fichiers n'était éprouvé par aucun test ; `texte_de` non nommé | Chaque avertissement par `texte_de`, relatif à la racine des données ; `l_etat_du_fonds_dit_l_avertissement_de_la_naissance` | 16 |
| R6. Majeur : « deux prises » ne se lit pas sur une ligne de la boîte, et le scénario n'était pas celui du test de la tranche 2 | Une fonction neuve, `deux_prises_regroupees_paraissent_ensemble_dans_la_boite`, et `cap02_3_la_boite_montre_ensemble_deux_prises_regroupees` ; les prises lues par la vue de l'idée | 16 |
| R7. Majeur : une idée reliée, retirée, ne quitte pas le compte des non reliées | Les valeurs de `cap02_5_…` écrites, X reliée à Y seule | 16 |
| R8. Majeur : le § 70 · 8 oubliait I01, I10 et la part d'I14 | La ligne 4 les nomme | 7 |
| R9. Majeur : la doc de `decalage_a` disait « en secondes », quand son appelant attend des minutes | En minutes, la conversion du `Date` écrite | 18 |
| R10. L'essai de `cargo tree` ne prouvait pas que `-e no-dev` écarte une fonction activée en test | Un second essai, sans les dépendances de test, qui ne doit pas la trouver | 6 |
| R11. Des méthodes du résolveur de Tauri et `BaseDirectory` échappaient au contrôle d'I30 | Onze noms et `BaseDirectory` ajoutés, et une plantation | 5 |
| R12. Les racines fixes ne liraient pas une crate native nouvelle | Chaque dossier de `crates/`, listé à l'exécution | 5 |
| R13. Un `use` de `dossier_de_l_application` tomberait sous le contrôle d'I28 | Elle s'appelle par son chemin, jamais par un `use` | 2 |
| R14. Le contrôle natif d'I29 ne disait pas s'il lit les commentaires | Il les ignore, comme celui d'I30, avec sa plantation | 17 |
| R15. Le test des entiers sous 2⁵³ ne voyait pas les types nouveaux | `LigneDeLaBoite`, `VueDeLaBoite` et `VueDesRetirees` y entrent | 16 |
| R16. `retirees` du noyau ne disait pas qu'il écarte les non retirées | Sa doc le dit, et le jeu gagne une idée non retirée | 14 |
| R17. Deux idées du jeu avaient une durée sans prise | Elles gagnent une prise ; le filtre `Prise` recalculé | 14 |
| R18. Hors de l'application, toute l'interface devenait la page du bandeau | Hors de l'application, tout s'ouvre, et la boîte dit son refus en bandeau (`dans_l_application`) | 18 |
| R19. Le rouge attendu d'I31 dans `windows` après la tâche 16 n'était pas annoncé | Annoncé, jusqu'à la tâche 17 | 16 |
| R20. Le nom de chaque directive de la CSP n'est pas une source | Le premier mot d'une directive, hors de la comparaison | 11 |
| R21. `exporter_cahier` garde trois arguments | La ligne d'effet le dit | effets, 18 |
| R22. « Aucune idée à envoyer » et « une version plus récente » sans test nommé | La couverture les nomme ; `un_etat_ne_se_ferme_pas` joue une version plus récente | 18, auto-relecture |
| R23. La recherche de LE-2 oubliait `ranger`, `rangée`, `ordonner` | Ajoutés ; É5 **Ordre** et **Rangée** se distinguent nommément | 20 |

## Les écarts au plan maître

Le plan maître le veut : un plan de tranche qui le contredit le dit dans sa PR, et la maîtrise d'ouvrage relit ce
point. Ce plan ne touche pas le plan maître (une retouche le ferait vérifier par `plan.py` sans ses sections de
tranche 3 refaites) ; chaque écart est dit ici, et la maîtrise d'ouvrage les a tranchés avec les questions, le
2026-11-15, chacun selon son défaut.

1. **Quatre PR au lieu de trois** (question 35) : (a) les styles ; (b) l'assemblage, `getrandom`, I30, les commandes
   de la tranche 2 branchées et l'étape `cargo tree` d'`etabli` ; (c) la CSP, les droits, le protocole qui refuse,
   I31 ; (d) la boîte, l'état à l'ouverture, le bandeau, I29, l'essai installé. La garde vient avant le code : la
   boîte s'écrit sous la CSP et sous le contrôle de l'interface.
2. **Trois lectures que le plan maître ne nomme pas** : `boite` (une fenêtre, avec les deux comptes et leurs
   assiettes), `retirees` (une fenêtre) et `etat_du_fonds` (questions 15, 16, 26). `idees` reste.
3. **La décision 10 vaut côté natif** (question 5) : le contrôle d'I30 lit les crates natives ; l'interface lit le
   décalage local dans des fonctions nommées, sous son propre contrôle.
4. **L'exception de l'acteur d'Automerge** nomme aussi `crates/services/src/application.rs` (la naissance du fonds)
   et `crates/services/src/en_memoire.rs` (l'adaptateur en mémoire), quand la décision 10 ne cite que le stockage
   (question 6).
5. **Le second processus ouvre sa fenêtre** sur le seul bandeau, quand le plan maître dit qu'il « s'arrête en le
   disant » : il ne sert aucune commande et n'écrit rien sous `donnees/` ; fermer la fenêtre l'arrête (question 4).
6. **`ipc:` sort de la CSP exacte** (question 11), et le § 40 · 7.5 se corrige en (c).
7. **Le protocole `captation` s'enregistre dès la tranche 3**, avec un gestionnaire qui refuse tout (question 12).
8. **Les critères de sortie** : CAP-01 · 4, CAP-02 · 4, 5 et 6 sortent de la tranche en parts, chacune avec son test
   nommé ; CAP-02 · 1, CAP-02 · 3 et CAP-07 · 4 s'inscrivent (question 33).
9. **Regrouper et écouter n'ont pas de contrôle à l'écran en tranche 3** (question 28) : la commande `regrouper` est
   servie, sans bouton ; l'écoute vient avec le protocole, en tranche 5.
10. **L'essai installé** n'est plus conditionnel (la décision 21, question 1, est tranchée depuis le 2026-11-09) ; il
    couvre aussi la table, la palette, la fiche, les textes piégés et le bandeau ; le fonds d'essai se jette quand la
    genèse change (question 34).
11. **`cssText`** : la ligne du plan maître (« Outils et cas limites », `style-src 'self'`) dit que la CSP le refuse ;
    elle l'admet. Le contrôle le refuse pour une autre raison, que le § 40 · 7.5 écrit en (c) (question 9).

## Les questions tranchées

Posées le 2026-11-15 en un bloc, tranchées le même jour, chacune selon son défaut.

1. **Où vivent les services entre deux commandes** : un fil dédié possède la racine et les services ; les commandes,
   `async`, lui écrivent par un canal ; le stockage ne change pas ; la tranche 6 ferme en arrêtant le fil, le verrou
   en dernier.
2. **L'aléa du système qui échoue** : `systeme` s'arrête par une panique, au message « Le système n'a pas donné
   d'aléa. » ; `Alea` reste infaillible.
3. **Une racine que le disque refuse d'ouvrir** : le bandeau nomme le dossier des données par son chemin entier, seul
   chemin absolu montré.
4. **Le second processus** : une fenêtre sur le seul bandeau « Un autre processus tient la racine des données. »,
   sans commande servie ; « sans rien écrire » s'entend sous `donnees/`.
5. **La décision 10** vaut côté natif ; le contrôle d'I30 lit les `src/` des crates natives ; l'interface n'admet
   l'horloge que dans deux fonctions nommées, sous son propre contrôle.
6. **Les exceptions d'I30** : l'acteur d'Automerge dans le stockage, la naissance des services et l'adaptateur en
   mémoire ; le seul `temp_dir()` admis est celui de `dossier_jetable` ; une plantation hors de chacune.
7. **La sentinelle** ne redirige ni le dossier personnel ni le dossier temporaire : le § 70 · 3 se corrige, le
   contrôle statique les nomme.
8. **Le garde-fou** : `systeme` reçoit le dossier en argument ; seuls `run()` et le garde-fou le résolvent, ce
   dernier avec l'identifiant de `tauri.conf.json`.
9. **`cssText`** : refusé par le contrôle de l'interface ; le § 40 · 7.5 dit la vraie raison.
10. **Les droits** : aucun droit du noyau de Tauri au lot 1 ; `windows.json` ne porte que les `allow-<nom>`.
11. **`ipc:`** sort de la CSP (277 octets).
12. **Le protocole des captations** : un gestionnaire qui refuse toute requête (404), que la tranche 5 remplira.
13. **I29 côté natif** : le contrôle refuse aussi `.message(` et `MessageDialog` ; la boîte d'enregistrement reste.
14. **Ce que la boîte liste** : toutes les idées non retirées, et le compte des non reliées à côté.
15. **La commande de la boîte** : l'ordre, le filtre, un début et un nombre ; les deux comptes et leurs assiettes ;
    la seule fenêtre demandée ; l'ordre et le filtre côté natif, par le noyau.
16. **L'état à l'ouverture** : `etat_du_fonds`.
17. **« Tri » contre « ordre »** : `Ordre`, `ordonner`, « Ranger par » ; le glossaire gagne les deux entrées.
18. **Les formes de l'assiette** : « N idées non reliées · M interrogées » et « N idées · M interrogées » ; « · K
    illisibles » seulement si K > 0.
19. **Les idées illisibles** : comptées parmi les interrogées, absentes de la liste.
20. **Ranger par étiquette** : sur la plus petite, octet pour octet ; sans étiquette, en fin.
21. **Les sens et le départage** : la date, la plus récente d'abord, par défaut ; la durée, la plus longue d'abord,
    sans durée en fin ; les non reliées d'abord ; les captées d'abord ; départage par la date décroissante, puis par
    l'identifiant ; les retirées par la date du retrait, la plus récente d'abord.
22. **Le filtre** : « texte de l'idée » ; « MIDI versé » seul ou qui accompagne un stem ; « photo » ne retient rien au
    lot 1.
23. **La durée** : arrondie à la seconde inférieure, m:ss, h:mm:ss.
24. **La date** : « 3 nov. 2026 », sans heure, au décalage de l'instant ; l'écart d'un jour devient une piste.
25. **Le `—`** dans chaque case vide.
26. **La liste virtualisée** : lignes de hauteur fixe ; les retirées en commande à part, en fenêtre ; un test borne
    les lignes rendues ; aucun seuil de temps en CI.
27. **La place de la boîte** : un sélecteur « Table » / « Boîte d'idées » ; le champ de note toujours présent ; pas
    d'état de synchronisation avant la tranche 4, écart provisoire dit dans la PR.
28. **Les gestes à l'écran** : compléter, étiqueter, relier, délier, retirer, rétablir, annuler.
29. **La saisie d'une étiquette** : refusée vide au noyau ; l'interface ôte les blancs et un `#` de tête.
30. **Annuler** : `Ctrl+Z` hors d'un champ de texte ; le bouton porte le message du geste visé, puis « Rien à
    annuler ».
31. **Le bandeau** : le mot garde le sens du message ; les messages s'empilent, un par source, les états d'abord ; un
    refus reste jusqu'au geste réussi suivant ou jusqu'à « Fermer » ; un rejet qui n'est pas un `Refus` s'affiche
    « Le geste n'a pas abouti. ».
32. **Le glossaire** gagne « Droits (de Tauri) », « Reliée (idée) », « Relier, délier » et « interrogées ».
33. **Les inscriptions** : CAP-02 · 1, CAP-02 · 3 et CAP-07 · 4 ; CAP-01 · 4, CAP-02 · 4, 5 et 6 en parts.
34. **L'essai installé** (👤) : à la fin de la tranche ; table, palette, fiche, boîte, textes piégés, export ; le
    fonds d'essai se jette si la genèse change.
35. **Quatre PR** : l'écart 1.

Les choix techniques que ce plan tranche sans question : `exporter_cahier` entre dans `NOMS` ; les droits engendrés
sont ignorés par git ; `getrandom` à sa dernière version publiée ; les styles par `style:<propriété>` ; le
`cargo tree` d'`etabli` vise `x86_64-pc-windows-msvc` ; aucun `devCsp`.

---

## Points de revue

Cinq conditions que la spec implique sans qu'aucune de ses sources ne demande de les éprouver ; chacune a son test,
dans la tâche qui possède le code.

1. **Deux commandes arrivent ensemble** : le fil les sert l'une après l'autre, sans en perdre ni s'arrêter.
   `deux_demandes_concurrentes_sont_servies_l_une_apres_l_autre`, tâche 3.
2. **Une demande qui panique** (un défaut) laisse un refus, « Le fonds ne répond plus. », à elle et à chaque demande
   suivante, jamais une attente sans fin. `une_demande_qui_panique_laisse_un_refus_et_non_une_attente`, tâche 3.
3. **Une fenêtre hors du fonds** (un début au-delà du nombre d'idées, un nombre nul, un nombre au-delà de
   `FENETRE_MAX`) rend les comptes entiers et les seules lignes qui existent.
   `une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop`, tâche 16.
4. **La boîte d'un Pupitre neuf** : « 0 idée non reliée · 0 interrogée », aucun fonds né, et noter le fait naître.
   `la_boite_d_un_pupitre_neuf`, tâche 16.
5. **`Ctrl+Z` dans le champ de note** n'annule aucun geste de tri ; après une annulation, le bouton dit « Rien à
   annuler ». `ctrl_z_n_annule_un_geste_que_hors_d_un_champ` et `le_bouton_d_annulation_dit_le_geste_vise`,
   tâche 19.


## Mécanismes annoncés

- `app/ui/src/vue_table.rs:59` — `fn pc(v: f32) -> String` — `{:.3}%` écrit chaque valeur en pour cent à trois
  décimales : les places de la tâche 1 le gardent, si bien que les valeurs posées restent celles d'INC-0.
- `app/ui/src/vue_table.rs:65` — `pub(crate) fn style_du_recouvrement(r: Rect) -> String` — rend une chaîne de
  quatre déclarations, posée en attribut : elle devient une place à quatre propriétés (tâche 1).
- `app/ui/src/vue_table.rs:162` — `style=style_du_recouvrement(rect)` — un attribut `style`, que `style-src 'self'`
  refuse : il devient quatre `style:<propriété>` (tâche 1).
- `app/ui/src/vue_table.rs:176` — `style=style_de_la_palette(rect)` — de même pour le bouton de la palette
  (tâche 1).
- `app/ui/src/panneau_palette.rs:47` — `calc({largeur} * var(--mm))` — la largeur de la fiche, posée en attribut :
  elle passe par `style:width` (tâche 1).
- `app/ui/src/vue_table.rs:126` — `inner_html=svg` — la page du noyau, l'un des deux seuls lieux qu'admet le
  contrôle de l'interface (tâche 12).
- `app/ui/src/panneau_palette.rs:48` — `inner_html=svg` — la fiche du noyau, l'autre lieu (tâche 12).
- `crates/stockage/src/racine_des_donnees.rs:62` — `pub fn ouvrir(chemin: &Path)` — ouvre la racine des données et
  prend son verrou : le fil du fonds l'appelle dans son propre fil (tâche 3).
- `crates/stockage/src/racine_des_donnees.rs:72` — `TryLockError::WouldBlock => Erreur::RacineDesDonneesTenue` — le
  refus vient avant `documents/` et `captations/` : un second processus n'écrit rien sous `donnees/` (tâche 3).
- `crates/stockage/src/racine_des_donnees.rs:30` — `l'état de l'application, en tranche 3` — le commentaire se
  corrige : la racine vit dans le fil du fonds, jamais dans l'état de Tauri (tâche 3).
- `crates/stockage/src/lib.rs:222` — `Un autre processus tient la racine des données.` — le texte du bandeau du
  second processus (tâches 3 et 18).
- `crates/stockage/src/adaptateurs.rs:18` — `racine: &'r RacineDesDonnees` — le fonds emprunte la racine : les
  services ne vivent que tant qu'elle vit, dans le fil qui la possède (tâche 3).
- `crates/stockage/src/adaptateurs.rs:25` — `pub fn ouvrir(racine: &'r RacineDesDonnees) -> Result<Self, Refus>` —
  le fil ouvre le fonds après la racine, et rend son refus tel quel (tâche 3).
- `crates/stockage/src/adaptateurs.rs:117` — `fn refus(erreur: Erreur, racine: &RacineDesDonnees) -> Refus` —
  privée, elle demande la racine qui n'a pas pu s'ouvrir : `refus_a_l_ouverture` s'ajoute à côté (tâche 3).
- `crates/stockage/src/adaptateurs.rs:76-79` — `.first()` — `faire_naitre` ne rend que le premier avertissement :
  ceux de l'ouverture se lisent par `Documents::avertissements` (tâche 16).
- `crates/stockage/src/documents.rs:113` — `pub fn avertissements(&self) -> &[Avertissement]` — les avertissements
  de l'ouverture, que l'adaptateur sur fichiers rend en textes (tâche 16).
- `crates/services/src/lib.rs:71` — `dernier_geste: Option<GesteDeTri>` — privé : `etat_du_fonds` en rend le message
  (tâche 16).
- `crates/services/src/lib.rs:328` — `pub fn idees(&self) -> Reponse<Vec<String>>` — reste ; la boîte est une
  lecture de plus (tâche 16).
- `crates/services/src/lecture.rs:103` — `.filter_map(|id| lire_l_idee(doc, &id))` — une idée illisible disparaît
  sans être comptée : `idees_et_illisibles` la compte (tâche 16).
- `crates/services/src/lecture.rs:67` — `pub fn captations_de(` — un passage sur les trois conteneurs à chaque
  appel : `captations_par_idee` n'en fait qu'un pour toutes les idées (tâche 16).
- `crates/noyau/src/donnees/idee.rs:75` — `pub fn assiette<'a>(` — écarte les idées retirées ; l'assiette de la boîte
  s'appuie sur elle, les illisibles en plus (tâche 14).
- `crates/noyau/src/donnees/idee.rs:36` — `pub fn reliee(` — ne lit que les liens « apparenté » : sa doc le dit
  (tâche 14).
- `crates/noyau/src/donnees/gestes.rs:411` — `if etiquette.is_empty() {` — une étiquette de blancs passe : la règle
  devient celle d'une note, blancs ôtés (tâche 14).
- `crates/noyau/src/donnees/gestes.rs:241` — `pub fn date_du_message(heure: i64, decalage_minutes: i32) -> String`
  — le calcul civil, que la date de la liste partage (tâche 15).
- `crates/noyau/src/donnees/fonds.rs:32` — `pub const PHOTOS: &str = "photos";` — le conteneur que lit le filtre
  « photo », vide au lot 1 (tâche 16).
- `crates/commandes/src/lib.rs:18` — `pub const NOMS: &[&str] = &[` — la liste des noms : `exporter_cahier` y entre
  (tâche 9), puis les trois lectures (tâche 16).
- `app/src-tauri/src/lib.rs:127` — `pub fn run()` — gagne `setup`, le protocole et le gestionnaire des commandes
  (tâches 4 et 10).
- `app/src-tauri/src/lib.rs:130` — `tauri::generate_handler![exporter_cahier]` — passe dans `gestionnaire()`, avec
  chaque commande (tâche 4).
- `app/src-tauri/src/lib.rs:170` — `std::env::temp_dir()` — le seul `temp_dir()` qu'admet le contrôle d'I30, celui de
  `dossier_jetable` (tâche 5).
- `app/src-tauri/build.rs:2` — `tauri_build::build();` — devient `try_build`, avec le manifeste de l'application
  (tâche 9).
- `app/src-tauri/tauri.conf.json:17` — `"csp": null` — la CSP exacte prend sa place (tâche 8).
- `app/src-tauri/tauri.conf.json:5` — `"identifier": "app.pupitre.etabli"` — l'identifiant que le garde-fou lit
  (tâche 6).
- `app/src-tauri/capabilities/default.json:6` — `"permissions": ["core:default"]` — le fichier disparaît (tâche 9).
- `app/ui/src/export.rs:46` — `js_sys::Date::new_0()` — l'horloge de `date_du_jour`, que le contrôle de l'interface
  admet (tâche 12).
- `app/ui/src/export.rs:52` — `"__TAURI__"` — l'appel de Tauri par `Reflect::get`, que le pont natif reprend
  (tâche 18).
- `app/ui/tests/encre_statique.rs:309` — `"set_timeout"` — I18 refuse les minuteries : la liste suit le défilement,
  et le bandeau ne s'efface que par un geste (tâches 18 et 19).
- `.github/workflows/ci.yml:102` — `Le graphe de l'interface` — le modèle de l'étape du graphe de l'établi (tâche 6).
- `.github/workflows/ci.yml:186` — `cargo test -p etabli` — l'étape que suit celle de la sentinelle (tâche 6).
- `crates/services/tests/statique.rs:120` — `const RACINES_LUES: &[&str] = &[` — la façon de lire les sources, que
  le contrôle d'I30 reprend (tâche 5).
- `scripts/verif-faits.mjs:439` — `export function section(texte, titre) {` — le point 9 y lit le § 40 · 7.5
  (tâche 8).
- `docs/40-architecture.md:517` — `connect-src 'self' ipc: http://ipc.localhost` — la CSP exacte, corrigée (tâche 8).
- `docs/40-architecture.md:548` — `La liste exacte des commandes vit dans ces fichiers` — la correction (f)
  (tâche 9).
- `docs/70-qualite-et-verification.md:139` — `Tout chemin deviné y tombe ainsi` — la phrase d'I30 que la tâche 7
  corrige.
- `docs/80-glossaire.md:209` — `avant tout tri` — l'entrée « Boîte d'idées », corrigée (tâche 20).
- `docs/20-sfd/21-capter-et-organiser.md:128` — `La boîte se trie par date` — devient « se range par » (tâche 20).
- `docs/20-sfd/21-capter-et-organiser.md:445` — `Tri libre` — précise « pour les morceaux » (tâche 20).
- `docs/41-pile-technique.md:288` — `écoute le signal d'un document changé` — se corrige (tâche 20).
- `docs/50-interfaces-et-appareils.md:277` — `Toujours l'assiette` — l'exemple prend les trois nombres de CAP-06
  (tâche 20).
- `.cargo/mutants.toml:2` — `Seize modules sont examinés` — dix-sept, avec `donnees/boite.rs` (tâche 14).

## Outils et cas limites

- Leptos 0.8.23 (tachys 0.2.19), `style=` — pose un attribut, que `style-src 'self'` refuse, même d'une valeur
  calculée ; `style:<propriété>` pose une propriété, admise — `le_recouvrement_se_place_en_pour_cent_de_la_page`
  et `palette_se_pose_au_bas_de_la_case_touchee` (tâche 1), sur les quatre valeurs, et la plantation d'un `style=`,
  refusée (tâche 12)
- Chromium (WebView2), `style.cssText` — la CSP l'admet, puisque c'est du CSSOM — la plantation d'un `cssText`,
  refusée par le contrôle de l'interface (tâche 12), qui veut qu'un style calculé ne se pose que d'une façon
- `getrandom`, `fill` — rend une erreur que `Alea::remplir` ne peut pas rendre — aucun test ne la provoque, limite
  que la PR (b) dit ; `l_alea_du_systeme_remplit_ses_octets` (tâche 2) éprouve le cas ordinaire, et la panique porte
  son message
- `std::sync::mpsc` — un envoi vers un récepteur tombé échoue, et une réception dont l'émetteur est tombé, par une
  panique du fil, échoue aussi, jamais une attente sans fin — `une_demande_qui_panique_laisse_un_refus_et_non_une_attente`
  (tâche 3)
- `File::try_lock`, le verrou de la racine des données — deux ouvertures dans un même processus se refusent comme
  deux processus, chacune son descripteur ; sous Windows, le verrou est obligatoire, et un autre descripteur peut se
  voir refuser la lecture du fichier `verrou` tant que la racine est ouverte —
  `un_second_processus_devant_une_racine_tenue_s_arrete_en_le_disant` (tâche 3), qui ouvre deux fois la même racine
  dans le processus de test, et dont la photographie ne lit pas les octets de `verrou`
- `chemin_dit`, le texte d'un chemin dans une erreur du stockage — ses composants se joignent par `/`, sous Windows
  aussi, quand `Path::display` y écrit `\` — `une_racine_que_le_disque_refuse_se_dit_par_son_chemin` (tâche 3) et
  `i29_une_ecriture_refusee_revient_en_refus_et_l_etat_le_dit` (tâche 17), qui comparent le chemin sous sa forme `/`,
  ou cherchent le nom du dossier jetable, qui paraît sous les deux formes
- `tauri::AppHandle` sans paramètre — vaut `AppHandle<Wry>` : une commande qui le prend ne passe pas dans un
  gestionnaire générique — `les_commandes_passent_par_l_ipc_de_tauri` (tâche 4), dont le gestionnaire sert le
  `MockRuntime` de `tauri::test`, `exporter_cahier` devenu générique
- `tauri::test::mock_context` — son identifiant est vide, et `app_local_data_dir` y rend le dossier local lui-même,
  non celui de l'application — `i30_le_dossier_que_tauri_resout_tombe_dans_la_sentinelle` (tâche 6), qui pose
  l'identifiant de `tauri.conf.json` et exige un dossier qui finit par `app.pupitre.etabli`
- `SHGetKnownFolderPath`, par `dirs` — suit `USERPROFILE` pour les dossiers de l'application, jamais pour le dossier
  personnel, et échoue sans les sous-dossiers `AppData\Roaming` et `AppData\Local` — l'étape « La sentinelle
  d'I30 » (tâche 6), qui crée les sous-dossiers, et les plantations de `home_dir` et de `temp_dir` (tâche 5)
- `Get-ChildItem`, dans PowerShell, le shell de `windows` — sans `-Force`, il ne voit pas un fichier caché ; et un
  exécutable natif en échec n'arrête pas le script — l'essai de l'étape de la sentinelle (tâche 6), qui plante un
  fichier caché dans une sentinelle d'essai et exige de le voir, puis lit `$LASTEXITCODE` après `cargo test`
- `cargo` et `rustup` sous `USERPROFILE` redirigé — ils tirent leurs dossiers de `USERPROFILE`, et chercheraient la
  chaîne dans la sentinelle — l'étape de la sentinelle (tâche 6), qui pose `CARGO_HOME` et `RUSTUP_HOME` à leurs
  vrais dossiers avant de rediriger, et rougirait sinon, faute de chaîne
- `cargo tree -e features` — une fonction ne paraît que sur une ligne `<paquet> feature "<nom>"` ; sans `--target`,
  le graphe se résout pour l'hôte Linux, où l'établi ne se construit pas — l'étape « Le graphe de l'établi »
  (tâche 6), avec `--target x86_64-pc-windows-msvc`, et son essai sur `cargo tree -p stockage -e features`, qui doit
  trouver `stockage feature "pannes"`, puis sur `cargo tree -p stockage -e no-dev,features`, qui, sans les
  dépendances de test, ne doit pas la trouver
- `#[ignore]` et `--include-ignored` — un test ignoré qui ne tourne jamais passe sans bruit — l'étape de la sentinelle
  (tâche 6) le lance par `--include-ignored`, et le test refuse de passer sans `PUPITRE_SENTINELLE`
- `tauri-build`, `AppManifest::commands` — écrit `permissions/autogenerated/` dans la crate à chaque construction, et
  n'efface jamais le droit d'une commande retirée — `.gitignore` l'ignore (tâche 9) ; les tests d'I31 comparent
  `windows.json` à `NOMS`, jamais à ce que la construction accepte, limite que la PR (c) dit
- les droits engendrés de Tauri — chacun s'écrit `allow-` suivi du nom, chaque `_` devenu `-` —
  `i31_les_droits_nomment_chaque_commande_et_rien_d_autre` (tâche 11), et sa plantation d'un `allow-exporter_cahier`,
  refusée
- les fichiers de droits de Tauri — chaque fichier de `capabilities/` s'applique, faute de liste dans
  `app > security > capabilities` — `i31_seul_windows_json_porte_des_droits` (tâche 11), et sa plantation d'un
  `default.json`
- les clés de `tauri.conf.json` — chacune a une forme en kebab-case, et `useHttpsScheme` vit sous chaque fenêtre de
  `app > windows`, non sous `security` — `i31_tauri_conf_ne_pose_aucune_cle_interdite` (tâche 11), une plantation
  par forme et par place
- la CSP que `verif-faits.mjs` lit — Tauri l'admet en chaîne ou en objet, et fusionne par-dessus un
  `tauri.windows.conf.json` — les tests du point 9 dans `scripts/verif-faits.test.mjs` (tâche 8), qui plantent une
  CSP en objet, puis un `tauri.windows.conf.json`, refusés
- une recherche de `unsafe-eval` dans la CSP — `'wasm-unsafe-eval'` contient `unsafe-eval'` — `controler_la_csp`
  (tâche 11) compare des sources entières, et la CSP exacte passe quand la plantation de `'unsafe-eval'` est refusée
- `TAURI_CONFIG`, une configuration fusionnée à la construction — aucun fichier ne la montre — aucun test ne le peut,
  limite que la PR (c) dit : la CI ne la pose pas
- `register_uri_scheme_protocol` sous Windows — WebView2 sert le protocole à `http://captation.localhost/` —
  `le_protocole_des_captations_refuse_toute_requete` (tâche 10), et la comparaison de la CSP (tâche 8)
- les commandes de Tauri — leurs arguments se lisent en camelCase, et une commande sans `async` tourne sur le fil de
  l'IPC — `les_commandes_passent_par_l_ipc_de_tauri` (tâche 4), qui invoque par l'IPC simulée de `tauri::test`
  avec le seul argument `commande`
- une promesse de Tauri rejetée — par un `Refus` sérialisé, ou par un texte de Tauri, en anglais, comme un refus des
  droits — `un_rejet_qui_n_est_pas_un_refus_se_dit_par_un_constat` (tâche 18)
- `Date.getTimezoneOffset` — vaut UTC moins l'heure locale, l'opposé de `decalage_minutes` —
  `le_decalage_de_javascript_change_de_signe` (tâche 18)
- les contrôles statiques, en sous-chaîne — ils lisent aussi les commentaires ; `inner_html` a d'autres formes
  (`innerHTML`, `outer_html`, `insert_adjacent_html`) ; un import groupé laisse `env::temp_dir()` sans `std::env` —
  `i30_chaque_controle_refuse_ce_qu_on_y_plante` (tâche 5) et `chaque_controle_refuse_ce_qu_on_y_plante` de
  l'interface (tâche 12), une plantation par forme
- `git grep -n -i -w`, la recherche de LE-2 — `-i` ne replie pas les accents, et `-w` prend une lettre accentuée pour
  une borne de mot — aucun test ne le peut, limite que la PR (d) dit : sa revue relit la recherche citée, faite aussi
  sans accent

## Effets cherchés

- Le fil du fonds possède la racine des données et les services, et `fermer` les rend, la racine et son verrou en
  dernier — la reprise de la tranche 6, qui ferme avant de renommer `donnees/`, et le protocole des captations de la
  tranche 5, servi par le même fil — `le_fil_ferme_la_racine_quand_on_le_ferme` (tâche 3) : une seconde ouverture
  réussit après `fermer`
- Chaque commande de `commandes_de_l_etabli` prend au plus un argument, `commande`, et rend une `Reponse` de
  `commandes` ; `exporter_cahier` garde ses trois arguments d'INC-0, que `export.rs` envoie toujours — l'interface,
  qui envoie `{"commande": …}` et lit l'objet `Refus` d'une promesse rejetée, et chaque tranche qui ajoute une
  commande — `les_commandes_passent_par_l_ipc_de_tauri` (tâche 4) et `refus_du_rejet` (tâche 18)
- Le contrôle d'I30 lit les `src/` de chaque crate de `crates/` et d'`app/src-tauri`, et l'interface a sa propre
  règle de l'horloge — toute PR suivante qui ajoute un module ou une crate native, ou une lecture de l'horloge dans
  l'interface — `i30_chaque_racine_lue_porte_des_sources` et les plantations des tâches 5 et 12
- La sentinelle relance tous les tests d'`etabli` sous `USERPROFILE` redirigé, dans PowerShell — le job `windows` et
  son délai — l'essai du fichier caché, l'étape verte et son temps relevé (tâches 6 et 7)
- La CSP exacte perd `ipc:` — le § 40 · 7.5, `tauri.conf.json` et le job `documentation` — le point 9 de
  `verif-faits.mjs`, vert, et ses tests (tâche 8)
- Avec le manifeste, chaque nom de `NOMS` demande son entrée dans `generate_handler!` et son droit — chaque tranche
  suivante qui ajoute une commande — les tests d'I31 (tâche 11), qui refusent une commande sans droit ou un droit
  sans commande
- `.gitignore` ignore `app/src-tauri/permissions/autogenerated/` — chaque clone et chaque worktree du poste —
  `git check-ignore -q app/src-tauri/permissions/autogenerated/allow-noter.toml`, code 0 (tâche 9)
- Le protocole `captation` refuse toute requête dès la tranche 3 — la tranche 5, qui remplit son gestionnaire —
  `le_protocole_des_captations_refuse_toute_requete` (tâche 10)
- « Tri » reste aux gestes, et l'ordre de la boîte s'écrit `Ordre`, `ordonner`, « Ranger par » — le glossaire,
  CAP-02, et la tranche 4, qui étend l'ordre et le filtre en requêtes — la recherche de LE-2, citée dans la PR (d)
  (tâche 20)
- Le contrat `Documents` gagne `avertissements` — les deux adaptateurs, et celui qu'un lot suivant écrira — la suite
  des commandes sur les deux paires, et `l_etat_du_fonds_dit_l_avertissement_de_la_naissance` sur fichiers (tâche 16)
- Une étiquette vide une fois ses blancs ôtés est refusée au noyau — le schéma de la `v0.3.0`, avant son gel, et le
  carnet au lot 2 — `une_etiquette_de_blancs_est_refusee` (tâche 14)
- La commande de la boîte rend une fenêtre de `FENETRE_MAX` lignes au plus — l'interface, et la tranche 4, qui
  l'étend en requête — `une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop` (tâche 16)
- Le fonds d'essai de l'essai installé se jette quand la genèse change — la maîtrise d'ouvrage (👤), avant l'essai de
  la revue finale, si la tranche 4 ou 5 change la genèse — le relevé du § 70 · 7 le dit (tâche 21)
- `etabli` tire `stockage`, `services`, `commandes` et `getrandom`, sans `pannes` ni `en-memoire` —
  l'installateur livré — l'étape « Le graphe de l'établi » de `verifier` (tâche 6)

## Revues

- la revue de chaque tâche : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12, Tâche 13, Tâche 14, Tâche 15, Tâche 16, Tâche 17, Tâche 18, Tâche 19, Tâche 20
- la revue de la PR (a) entière, sur sa tête, avant `gh pr ready` : Tâche 1
- la revue de la PR (b) entière, sur sa tête, la sentinelle et les délais compris : Tâche 7
- la revue de la PR (c) entière, sur sa tête, la CSP relue contre le § 40 · 7.5 : Tâche 13
- la revue de la PR (d) entière, sur sa tête, le tableau du § 70 · 4.1 comparé aux critères de la tranche et LE-15 compris : Tâche 20
- la relecture des corrections de chaque revue de PR, sur la tête qui part, celle que nomme `--match-head-commit` : Tâche 1, Tâche 7, Tâche 13, Tâche 20
- la relecture du relevé de l'essai installé, sur la tête qui part : Tâche 21

---

## La carte des fichiers

```text
app/ui/src/vue_table.rs            (a) Place, place_du_recouvrement, place_de_la_palette
app/ui/src/panneau_palette.rs      (a) largeur_de_la_fiche, style:width
app/src-tauri/
  Cargo.toml                       (b) + stockage, services, commandes, getrandom ; [dev-dependencies] tauri (test),
                                   serde_json ; (c) [build-dependencies] + commandes
  build.rs                         (c) try_build, le manifeste de l'application tiré de commandes::NOMS
  tauri.conf.json                  (c) la CSP exacte
  capabilities/windows.json        (c) créé ; (d) les droits des trois lectures
  capabilities/default.json        (c) retiré
  src/lib.rs                       (b) mod systeme, fil, commandes_de_l_etabli ; run() et son setup ; gestionnaire() ;
                                   (c) le protocole captation, refuser_la_captation ; (d) les trois lectures au gestionnaire
  src/systeme.rs                   (b) heure, AleaDuSysteme, dossier_de_l_application, racine_des_donnees, ouvrir ; le garde-fou
  src/fil.rs                       (b) Ouverture, FilDuFonds, ServicesSurFichiers, ouvrir, demander, demander_bloquant, fermer ;
                                   (d) le test d'I29
  src/commandes_de_l_etabli.rs     (b) les commandes de la tranche 2 ; (d) boite, retirees, etat_du_fonds
  tests/statique.rs                (b) I30 ; (d) I29 côté natif
  tests/i31.rs                     (c) la CSP, la configuration, les droits, generate_handler!, build.rs, Cargo.toml
crates/stockage/src/adaptateurs.rs (b) refus_a_l_ouverture ; (d) Documents::avertissements
crates/stockage/src/racine_des_donnees.rs  (b) le commentaire de fonds_ouvert
crates/commandes/src/lib.rs        (c) exporter_cahier dans NOMS ; (d) les constantes des noms, Ordre, Contenu, Boite,
                                   Fenetre, Assiette, LigneDeLaBoite, VueDeLaBoite, VueDesRetirees, EtatDuFonds,
                                   FENETRE_MAX, LONGUEUR_D_UNE_LIGNE
crates/commandes/tests/serialisation.rs    (d) les types nouveaux ; cap01_4
crates/noyau/src/donnees/boite.rs  (d) Ordre, Contenu, EnBoite, Boite, contenus_de, ordonner, boite, retirees,
                                   texte_du_compte, texte_du_filtre, date_de_la_liste, duree_affichee,
                                   etiquette_saisie                                                        (muté)
crates/noyau/src/donnees/gestes.rs (d) l'étiquette de blancs refusée ; le calcul civil partagé              (muté)
crates/noyau/src/donnees/idee.rs   (d) la doc de reliee
crates/noyau/tests/boite.rs        (d) les tests de boite.rs
crates/services/src/contrats.rs    (d) Documents::avertissements
crates/services/src/lecture.rs     (d) idees_et_illisibles, captations_par_idee, idees_avec_une_photo
crates/services/src/lib.rs         (d) boite, retirees, etat_du_fonds
crates/services/src/en_memoire.rs  (d) avertissements
crates/services/tests/suite/mod.rs (d) cap02_1, cap02_3 (complété), cap02_5 (part), cap07_4, le texte piégé, les fenêtres
crates/services/tests/commandes_en_memoire.rs, crates/stockage/tests/commandes_sur_fichiers.rs  (d) les tests nommés
app/ui/Cargo.toml                  (d) + serde, serde_json
app/ui/src/export.rs               (d) EXPORTER_CAHIER au lieu du nom écrit
app/ui/src/main.rs                 (d) mod natif, bandeau, boite
app/ui/src/app.rs                  (d) Vue (Table, Boite), la page d'ouverture
app/ui/src/entete.rs               (d) le sélecteur « Table » / « Boîte d'idées »
app/ui/src/natif.rs                (d) invoquer, refus_du_rejet, decalage_du_geste, decalage_a, decalage_depuis_js
app/ui/src/bandeau.rs              (d) Bandeau, MessageDuBandeau
app/ui/src/boite.rs                (d) la boîte, la fenêtre, la vue de l'idée, les retirées
app/ui/style.css                   (d) la boîte, la liste, le bandeau
app/ui/tests/statique.rs           (c) inner_html, style, stockage du navigateur, horloge ; (d) I29, les décalages
scripts/verif-faits.mjs, scripts/verif-faits.test.mjs   (c) le point 9, la CSP
.github/workflows/ci.yml           (b) « Le graphe de l'établi » (verifier), « La sentinelle d'I30 » (windows), délais ;
                                   (c), (d) délais
.github/workflows/mutants-complet.yml  (d) délai
.cargo/mutants.toml                (d) donnees/boite.rs, tests/boite.rs
Cargo.lock                         (b), (c), (d) celui de l'artefact `cargo-lock`, relu
.gitignore                         (c) app/src-tauri/permissions/autogenerated/
docs/70-qualite-et-verification.md (b) I30, § 8 ; (c) § 8 ; (d) § 4.1, § 8 ; (fin) § 7, le relevé
docs/40-architecture.md            (c) § 7.5 : la CSP, style-src, connect-src, les droits, (f), la démonstration
docs/41-pile-technique.md          (d) js-sys, serde_json de l'interface
docs/80-glossaire.md, docs/20-sfd/21-capter-et-organiser.md, docs/50-interfaces-et-appareils.md, docs/pistes.md (d)
docs/registre.md                   (plan) LE-15 ; (d) LE-15 clos ; (fin) AM-3 clos
CONVENTIONS.md                     (b) verifier, windows ; (c) documentation, windows, verifier ; (d) verifier, windows, mutants
CLAUDE.md                          (b) « Les crates », « Commandes » ; (c), (d) de même
```


## PR (a) · les styles en propriétés

**Branche :** `t3a-styles-en-proprietes`. **Titre :** `refactor: les styles calculés d'INC-0 posés propriété par
propriété` (69). **Documents :** aucun : le § 40 · 7.5 le dit déjà (« passent ainsi en propriétés au lot 1, avant
que la CSP ne soit posée »). **Fin :** la tâche 1.

### Tâche 1 : les trois styles calculés, posés propriété par propriété

**Fichiers :** modifier `app/ui/src/vue_table.rs` (`:59-97`, `:162`, `:176` ; les tests `:320-351`),
`app/ui/src/panneau_palette.rs` (`:47`).

**Interfaces — produit** (`app/ui/src/vue_table.rs`, `app/ui/src/panneau_palette.rs`) :

```rust
/// Une place en pour cent de la page : les valeurs de `left`, `top`, `width` et `height`, chacune
/// écrite par `pc` (`{:.3}%`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub gauche: String,
    pub haut: String,
    pub largeur: String,
    pub hauteur: String,
}
/// La place du recouvrement d'une case : son rectangle rapporté à la page.
pub(crate) fn place_du_recouvrement(r: Rect) -> Place;
/// La place du bouton de la palette : `place_du_recouvrement(rect_de_la_palette(r))`.
pub(crate) fn place_de_la_palette(r: Rect) -> Place;
/// La largeur de la fiche : `calc(<largeur> * var(--mm))`, `largeur` écrite par `{}`.
pub(crate) fn largeur_de_la_fiche(largeur: f32) -> String;
```

`style_du_recouvrement` et `style_de_la_palette` disparaissent. La vue pose chaque valeur par
`style:left=…`, `style:top=…`, `style:width=…`, `style:height=…` (`:162`, `:176`) ; la fiche, par
`style:width=largeur_de_la_fiche(largeur)` (`panneau_palette.rs:47`). Aucun `style=` ne reste dans `app/ui/src/`.

- [ ] **Étape 1 : réécrire les tests.** Les deux tests d'INC-0 se réécrivent (la description de la PR le dit : la
  chaîne de déclarations devient quatre valeurs ; l'ancien second test était une tautologie,
  `style_de_la_palette(r) == style_du_recouvrement(rect_de_la_palette(r))`).

```rust
#[test]
fn le_recouvrement_se_place_en_pour_cent_de_la_page() {
    let place = place_du_recouvrement(Rect { x: 20.95, y: 15.7, l: 41.9, h: 31.4 });
    assert_eq!(place.gauche, "10.000%");
    assert_eq!(place.haut, "10.000%");
    assert_eq!(place.largeur, "20.000%");
    assert_eq!(place.hauteur, "20.000%");
}

#[test]
fn palette_se_pose_au_bas_de_la_case_touchee() {
    let r = Rect { x: 20.0, y: 30.0, l: 27.0, h: 20.0 };
    // Le rectangle, à 1e-4 près, comme INC-0 : (26 ; 45 ; 15 ; 4,6).
    let p = rect_de_la_palette(r);
    assert!((p.x - 26.0).abs() < 1e-4 && (p.y - 45.0).abs() < 1e-4);
    assert!((p.l - 15.0).abs() < 1e-4 && (p.h - 4.6).abs() < 1e-4);
    // Ses valeurs, calculées en f32 (passage de LE-6, règle 11).
    let place = place_de_la_palette(r);
    assert_eq!(place.gauche, "12.411%");
    assert_eq!(place.haut, "28.662%");
    assert_eq!(place.largeur, "7.160%");
    assert_eq!(place.hauteur, "2.930%");
}

#[test]
fn la_largeur_de_la_fiche_se_pose_en_millimetres_de_la_feuille() {
    assert_eq!(largeur_de_la_fiche(37.214), "calc(37.214 * var(--mm))");
    assert_eq!(largeur_de_la_fiche(56.0), "calc(56 * var(--mm))");
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: les styles d'INC-0 en quatre valeurs`), pousser, ouvrir la PR
  en brouillon. Attendu à « Lint de l'interface (natif, tests compris) » de `verifier` : `cannot find function
  place_du_recouvrement` (ou le premier nom manquant).
- [ ] **Étape 3 : écrire le code.** `place_du_recouvrement` reprend le calcul de `style_du_recouvrement`, chaque
  valeur par `pc` ; la vue et la fiche posent les propriétés comme dit plus haut.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : l'interface `ok`, ses trois tests nommés ; et
  `les_mesures_de_la_feuille_de_style_sont_celles_du_noyau` reste vert.
- [ ] **Étape 5 : committer** `git add app/ui/src/vue_table.rs app/ui/src/panneau_palette.rs`
  (`refactor: les styles calculés d'INC-0 posés propriété par propriété`).
- [ ] **Étape 6 : la revue de la PR (a).** La revue de la tâche, puis celle de la PR entière, sur sa tête ; la
  relecture des corrections, sur la tête qui part ; le corps final (« Écarts à la spec : Aucun. ») ; `gh pr ready`,
  puis la fusion par squash, `--match-head-commit` nommant cette tête.

## PR (b) · l'assemblage, `getrandom` et I30

**Branche :** `t3b-assemblage-et-i30`. **Titre :** `feat: l'établi assemble le fonds sur un fil, avec son heure et
son aléa ; I30` (79). **Documents :** § 70 · 3 (I30) et § 70 · 8 ; `CONVENTIONS.md` (`verifier`, `windows`) ;
`CLAUDE.md` (« Les crates », « Commandes ») ; `ci.yml` ; le commentaire de `fonds_ouvert`. **Fin :** la tâche 7.

### Tâche 2 : `systeme`, l'heure, l'aléa et le dossier de l'application

**Fichiers :** créer `app/src-tauri/src/systeme.rs` ; modifier `app/src-tauri/Cargo.toml` (dépendances
`stockage`, `services`, `commandes` par chemin, `getrandom = "<la dernière version publiée>"` ;
`[dev-dependencies]` : `tauri = { version = "2.12.5", features = ["test"] }` et `serde_json = "1.0.154"`),
`app/src-tauri/src/lib.rs` (`mod systeme;`), `Cargo.lock` (de l'artefact).

**Interfaces — produit** (`etabli_lib::systeme`) :

```rust
/// L'heure du système, en secondes depuis 1970 (`SystemTime`) ; une horloge d'avant 1970 compte pour 0.
pub fn heure() -> i64;
/// L'aléa du système, par `getrandom::fill`. Un échec arrête l'application : la panique dit
/// « Le système n'a pas donné d'aléa. » (question 2 ; `Alea::remplir` ne rend rien).
pub struct AleaDuSysteme;
impl services::Alea for AleaDuSysteme { fn remplir(&mut self, octets: &mut [u8]); }
/// Le dossier local de l'application, `app_local_data_dir`, jamais `app_data_dir` (§ 31 · 1 ; § 40 · 8) :
/// le seul appel du résolveur de Tauri. Seuls `run()` et le garde-fou d'I30 l'appellent (question 8).
pub fn dossier_de_l_application<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<PathBuf>;
/// La racine des données, `donnees/`, sous le dossier de l'application reçu.
pub fn racine_des_donnees(dossier: &Path) -> PathBuf;
```

`dossier_de_l_application` s'appelle toujours par son chemin, `systeme::dossier_de_l_application(…)`, jamais par un
`use`, ni seul ni groupé : le contrôle d'I28 refuse toute ligne `use` qui porte `application` hors des services
(`crates/services/tests/statique.rs:190`), et il lit `app/src-tauri/src`.

- [ ] **Étape 1 : relever la version de `getrandom`.** La dernière version publiée, sur crates.io ; la description
  de la PR la nomme, avec la date du relevé (décision 17). Le `Cargo.lock` en porte déjà deux (0.4.3, que tirent
  `automerge` et Leptos ; 0.3.4, que tire `tauri`) : la version déclarée ne doit pas en ajouter une troisième, sauf si
  la dernière publiée est plus récente que 0.4.3, ce que la PR dit.
- [ ] **Étape 2 : écrire les tests** (dans `systeme.rs`, `#[cfg(test)]`).

```rust
#[test]
fn l_heure_du_systeme_suit_l_ecriture_de_ce_plan() {
    // 1 794 700 800 : le 2026-11-15 à 0 h UTC ; 4 102 444 800 : le 2100-01-01.
    let h = heure();
    assert!(h > 1_794_700_800 && h < 4_102_444_800, "{h}");
}

#[test]
fn l_alea_du_systeme_remplit_ses_octets() {
    let (mut a, mut b) = ([0u8; 32], [0u8; 32]);
    AleaDuSysteme.remplir(&mut a);
    AleaDuSysteme.remplir(&mut b);
    assert_ne!(a, [0u8; 32]);
    assert_ne!(a, b);
}

#[test]
fn la_racine_des_donnees_est_donnees_sous_le_dossier_recu() {
    let dossier = crate::tests::dossier_jetable("racine-des-donnees");
    assert_eq!(racine_des_donnees(&dossier), dossier.join("donnees"));
}
```

  `dossier_jetable` (`app/src-tauri/src/lib.rs:168`) devient `pub(crate)`, et son module `pub(crate) mod tests` :
  c'est la seule fonction de dossier jetable d'`etabli`.
- [ ] **Étape 3 : voir le rouge.** Committer (`test: l'heure, l'aléa et la racine des données de systeme`), pousser,
  ouvrir la PR en brouillon. Attendu à « Lint de l'établi » de `windows` : `cannot find function heure` (ou le premier
  nom manquant). Le `Cargo.lock` en retard échoue d'abord à « Cargo.lock à jour » de `verifier` : prendre l'artefact
  `cargo-lock`, le relire (`getrandom` en dépendance d'`etabli`, `tauri` avec `test` en dépendance de test), le
  committer.
- [ ] **Étape 4 : écrire le code.** `heure` : `SystemTime::now().duration_since(UNIX_EPOCH)`, `as_secs`, borné à
  `i64::MAX` ; une erreur (avant 1970) donne 0. `remplir` : `getrandom::fill(octets).expect("Le système n'a pas donné
  d'aléa.")`. `dossier_de_l_application` : `app.path().app_local_data_dir()`.
- [ ] **Étape 5 : voir le vert.** Attendu à « Tests de l'établi » de `windows` : les trois tests `ok`, et les tests
  d'INC-0 de l'établi gardent leurs comptes.
- [ ] **Étape 6 : committer** `git add app/src-tauri/Cargo.toml app/src-tauri/src/lib.rs app/src-tauri/src/systeme.rs
  Cargo.lock` (`feat: systeme lit l'heure, tire l'aléa et reçoit le dossier de l'application`).

### Tâche 3 : le fil du fonds

**Fichiers :** créer `app/src-tauri/src/fil.rs` ; modifier `app/src-tauri/src/lib.rs` (`mod fil;`),
`app/src-tauri/src/systeme.rs` (`ouvrir`), `crates/stockage/src/adaptateurs.rs` (`refus_a_l_ouverture`, à côté de
`refus`, `:117`), `crates/stockage/src/lib.rs` (sa réexportation, comme celle de `FondsSurFichiers`),
`crates/stockage/src/racine_des_donnees.rs` (`:29-30`, le commentaire de `fonds_ouvert` : « pour que la racine des
données reste `Sync` ; elle vit dans le fil du fonds de l'établi, qui la possède »).

**Interfaces — consomme :** `RacineDesDonnees::ouvrir` ; `FondsSurFichiers::ouvrir` ; `RacineDesDonnees::captations` ;
`Services::new(documents, captations, gestes::Appareil::Etabli)` ; `systeme::heure`, `systeme::AleaDuSysteme`,
`systeme::racine_des_donnees`.

**Interfaces — produit :**

```rust
// crates/stockage/src/adaptateurs.rs
/// Le refus d'ouvrir la racine des données, qui n'a pas de racine à qui rapporter un chemin : la raison
/// que donne `refus` à chaque variante (`RacineDesDonneesTenue` : `Raison::Impossible`), et le texte de
/// l'erreur, qui nomme le dossier par son chemin entier, ses composants joints par `/`, sous Windows aussi
/// (`chemin_dit` ; question 3).
pub fn refus_a_l_ouverture(erreur: Erreur) -> Refus;

// app/src-tauri/src/fil.rs
/// Les services de l'établi, sur fichiers : ils empruntent la racine des données que le fil possède.
pub type ServicesSurFichiers<'r> = Services<FondsSurFichiers<'r>, stockage::Captations<'r>>;
/// L'ouverture du fonds, que Tauri garde dans son état (`Manager::manage`) : le fil qui sert les commandes,
/// ou le refus d'ouvrir, que chaque commande rend.
pub enum Ouverture { Ouverte(FilDuFonds), Refusee(Refus) }
/// Le fil du fonds : il ouvre la racine des données, puis le fonds, possède les services, et sert
/// chaque demande reçue, l'une après l'autre ; à chaque demande, le contexte du geste porte
/// `systeme::heure()` et `systeme::AleaDuSysteme`.
pub struct FilDuFonds { /* l'émetteur et le fil, chacun sous un Mutex<Option<…>> */ }
/// Ouvre le fil sur la racine des données `racine`, et attend le résultat de l'ouverture.
pub fn ouvrir(racine: PathBuf) -> Ouverture;
impl Ouverture {
    /// Demande `f` au fil, et attend sa réponse ; refusée, ou fil tombé : le refus.
    pub fn demander_bloquant<T: Send + 'static>(
        &self,
        f: impl for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<T> + Send + 'static,
    ) -> Reponse<T>;
    /// La même demande, attendue hors du fil de l'IPC (`tauri::async_runtime::spawn_blocking`), sur un
    /// émetteur cloné.
    pub async fn demander<T: Send + 'static>(
        &self,
        f: impl for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<T> + Send + 'static,
    ) -> Reponse<T>;
    /// Ferme le fil : l'émetteur tombe, le fil finit sa demande en cours, rend les services, puis la
    /// racine des données et son verrou, et `fermer` attend sa fin. Sans effet sur une `Refusee`.
    pub fn fermer(&self);
}

// app/src-tauri/src/systeme.rs
/// Ouvre `donnees/` sous le dossier de l'application reçu, par le fil du fonds.
pub fn ouvrir(dossier: &Path) -> Ouverture;   // fil::ouvrir(racine_des_donnees(dossier))
```

Le refus d'un fil tombé ou fermé : `Refus { raison: Raison::Impossible, message: "Le fonds ne répond plus." }`.
Aucun `catch_unwind` : une demande qui panique fait tomber le fil, donc la racine des données et son verrou ; chaque
demande suivante reçoit ce refus.

- [ ] **Étape 1 : écrire les tests** (dans `fil.rs`, `#[cfg(test)]`, chacun sur `crate::tests::dossier_jetable`).
  `noter_une(texte)` est une fonction de test qui rend la fermeture de `noter`, `decalage_minutes: 60`.
  `photographie(racine)` rend chaque fichier sous `documents/` et `captations/` de la racine des données, son chemin
  relatif et ses octets, dans l'ordre des chemins ; le fichier `verrou` n'y entre que par sa présence et sa taille
  (`std::fs::metadata`), jamais par ses octets : sous Windows, le verrou que tient la première ouverture peut en
  refuser la lecture (`crates/stockage/tests/demarrage.rs:791-792`).

```rust
#[test]
fn le_fil_note_puis_relit_l_idee() {
    let racine = dossier_jetable("fil-note").join("donnees");
    let ouverture = ouvrir(racine);
    let notee = ouverture.demander_bloquant(noter_une("Une idée de pont")).unwrap();
    let vue = ouverture
        .demander_bloquant(move |services, _| services.vue_d_une_idee(&notee.idee))
        .unwrap();
    assert_eq!(vue.texte.as_deref(), Some("Une idée de pont"));
    assert_eq!(vue.provenance, Provenance::Captee { par: Appareil::Etabli });
}

#[test]
fn un_second_processus_devant_une_racine_tenue_s_arrete_en_le_disant() {
    let racine = dossier_jetable("fil-tenue").join("donnees");
    let premiere = ouvrir(racine.clone());
    premiere.demander_bloquant(noter_une("Une idée")).unwrap();
    let avant = photographie(&racine);
    let seconde = ouvrir(racine.clone());
    let Ouverture::Refusee(refus) = &seconde else { panic!("la seconde ouverture devait être refusée") };
    assert_eq!(refus.raison, Raison::Impossible);
    assert_eq!(refus.message, "Un autre processus tient la racine des données.");
    // Aucune commande servie, et rien d'écrit sous donnees/ (question 4 ; LE-6, règle 19).
    assert_eq!(seconde.demander_bloquant(noter_une("Une autre")).unwrap_err(), refus.clone());
    assert_eq!(photographie(&racine), avant);
}

#[test]
fn le_fil_ferme_la_racine_quand_on_le_ferme() {
    let racine = dossier_jetable("fil-ferme").join("donnees");
    let premiere = ouvrir(racine.clone());
    premiere.fermer();
    assert!(matches!(ouvrir(racine), Ouverture::Ouverte(_)));
    let refus = premiere.demander_bloquant(noter_une("Une idée")).unwrap_err();
    assert_eq!(refus.message, "Le fonds ne répond plus.");
}

#[test]
fn une_racine_que_le_disque_refuse_se_dit_par_son_chemin() {
    let dossier = dossier_jetable("fil-disque");
    let racine = dossier.join("donnees");
    std::fs::write(&racine, b"un fichier, pas un dossier").unwrap();
    let Ouverture::Refusee(refus) = ouvrir(racine.clone()) else { panic!("l'ouverture devait être refusée") };
    assert_eq!(refus.raison, Raison::Disque);
    // Le stockage joint les composants du chemin par `/`, sous Windows aussi (`chemin_dit`), puis dit le
    // constat du disque (`constat_du_disque`).
    let dit = racine.display().to_string().replace('\\', "/");
    assert!(refus.message.starts_with(&format!("{dit} ")) || refus.message.starts_with(&format!("{dit},")),
        "{}", refus.message);
}

#[test]
fn deux_demandes_concurrentes_sont_servies_l_une_apres_l_autre() {
    let racine = dossier_jetable("fil-concurrence").join("donnees");
    let ouverture = std::sync::Arc::new(ouvrir(racine));
    let fils: Vec<_> = (0..2)
        .map(|n| {
            let ouverture = ouverture.clone();
            std::thread::spawn(move || {
                (0..10).all(|i| ouverture.demander_bloquant(noter_une(&format!("Idée {n}-{i}"))).is_ok())
            })
        })
        .collect();
    assert!(fils.into_iter().all(|fil| fil.join().unwrap()));
    assert_eq!(ouverture.demander_bloquant(|services, _| services.idees()).unwrap().len(), 20);
}

#[test]
fn une_demande_qui_panique_laisse_un_refus_et_non_une_attente() {
    let racine = dossier_jetable("fil-panique").join("donnees");
    let ouverture = ouvrir(racine);
    let refus = ouverture
        .demander_bloquant(|_, _| -> Reponse<()> { panic!("un défaut planté") })
        .unwrap_err();
    assert_eq!(refus.message, "Le fonds ne répond plus.");
    assert_eq!(ouverture.demander_bloquant(noter_une("Une idée")).unwrap_err(), refus);
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: le fil du fonds, la racine tenue, la fermeture et la panique`),
  pousser. Attendu à « Lint de l'établi » de `windows` : `cannot find function ouvrir` (ou le premier nom manquant).
- [ ] **Étape 3 : écrire le code.** `ouvrir` crée un canal de demandes (`std::sync::mpsc`) et un canal
  d'ouverture, lance le fil (`std::thread::Builder`, nommé « fonds »), qui ouvre la racine (son refus par
  `refus_a_l_ouverture`), puis le fonds (son refus tel quel), envoie le résultat de l'ouverture, et, ouvert, sert
  chaque demande jusqu'à la chute de l'émetteur ; `ouvrir` attend le résultat. Chaque demande emballe `f` avec un
  émetteur de réponse ; la réponse perdue (fil tombé) ou l'envoi refusé (fil fermé) donnent le refus « Le fonds ne
  répond plus. ». `demander` clone l'émetteur, puis attend dans `spawn_blocking`. Le commentaire de `fonds_ouvert`
  se corrige.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests de l'établi » de `windows` : les six tests `ok` ; à « Tests du
  stockage et de la synchronisation (H07) », les comptes du stockage inchangés.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/fil.rs app/src-tauri/src/lib.rs app/src-tauri/src/systeme.rs
  crates/stockage/src/adaptateurs.rs crates/stockage/src/lib.rs crates/stockage/src/racine_des_donnees.rs`
  (`feat: le fil du fonds possède la racine des données et sert les commandes une à une`).

### Tâche 4 : les commandes de la tranche 2, servies par l'établi

**Fichiers :** créer `app/src-tauri/src/commandes_de_l_etabli.rs` ; modifier `app/src-tauri/src/lib.rs` (`run`,
`:127-133` ; `gestionnaire` ; `exporter_cahier`, `:85-124`, devient générique : `async fn exporter_cahier<R:
tauri::Runtime>(app: tauri::AppHandle<R>, date: String, instrument: String, couche: String) -> Result<Option<String>,
String>`, son corps inchangé, puisque `DialogExt`, `get_webview_window` et `set_parent` valent pour tout `R` ; un
`AppHandle` sans paramètre vaut `AppHandle<Wry>`, qui ne passe pas dans un gestionnaire générique).

**Interfaces — consomme :** `Ouverture::demander` (tâche 3) ; les méthodes de `Services` et les types de
`commandes`, tels que la tranche 2 les a posés.

**Interfaces — produit :**

```rust
// app/src-tauri/src/commandes_de_l_etabli.rs : chaque commande est `async`, prend l'état et au plus un
// argument, nommé `commande` (un seul mot, que le camelCase de Tauri ne change pas), et rend une
// `Reponse` de `commandes`.
#[tauri::command]
pub async fn noter(ouverture: tauri::State<'_, Ouverture>, commande: Noter) -> Reponse<Notee>;
// De même : completer (Completer) -> Faite ; poser_une_etiquette, oter_une_etiquette (Etiqueter) -> Faite ;
// relier, delier (Relier) -> Faite ; regrouper (Regrouper) -> Faite ; retirer, retablir (Viser) -> Faite ;
// annuler () -> Faite ; vue_d_une_idee (commande: String, l'identifiant de l'idée) -> VueDUneIdee ;
// idees () -> Vec<String> ; journal () -> Vec<EntreeDuJournal>.

// app/src-tauri/src/lib.rs
/// Le gestionnaire des commandes : `generate_handler!`, qui nomme `exporter_cahier` et chaque commande de
/// `commandes_de_l_etabli` ; `run()` et les tests le partagent.
pub(crate) fn gestionnaire<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static;
```

`run()` : `.setup(|app| { let dossier = systeme::dossier_de_l_application(app.handle())?;
app.manage(systeme::ouvrir(&dossier)); Ok(()) })`, puis `.invoke_handler(gestionnaire())`.

- [ ] **Étape 1 : écrire le test** (dans `commandes_de_l_etabli.rs`, `#[cfg(test)]`), par l'IPC simulée de
  `tauri::test` : `mock_builder()`, `.invoke_handler(crate::gestionnaire())`, `.manage(fil::ouvrir(<dossier
  jetable>/donnees))`, `build(mock_context(noop_assets()))`, une fenêtre `main`, puis `get_ipc_response` ; chaque
  nom se vérifie sur docs.rs de `tauri` 2.12.5. Le test `les_commandes_passent_par_l_ipc_de_tauri` :
  - invoque `noter` avec le corps `{"commande": {"texte": "Une idée de pont", "decalage_minutes": 60}}`, et lit une
    `Notee` ;
  - invoque `vue_d_une_idee` avec `{"commande": <l'idée née>}`, et lit `texte == Some("Une idée de pont")` ;
  - invoque `annuler` avec `{}`, et lit un rejet, désérialisé en `Refus`, de raison `Raison::RienAAnnuler`.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les commandes de la tranche 2 par l'IPC de Tauri`), pousser.
  Attendu à « Lint de l'établi » de `windows` : `cannot find function gestionnaire` (ou le premier nom manquant).
- [ ] **Étape 3 : écrire le code.** Chaque commande appelle `ouverture.demander(move |services, contexte| …)` et la
  méthode de même nom ; `run()` comme dit plus haut. `use tauri::Manager;` sert aussi `setup`, hors de
  `#[cfg(desktop)]` si le lint le demande.
- [ ] **Étape 4 : voir le vert.** Attendu : « Construire l'installateur » vert, puis « Tests de l'établi » `ok`, le
  test nommé compris.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/commandes_de_l_etabli.rs app/src-tauri/src/lib.rs`
  (`feat: l'établi sert les commandes de la tranche 2 par le fil du fonds`).

### Tâche 5 : I30, le contrôle statique

**Fichiers :** créer `app/src-tauri/tests/statique.rs`.

**Ce que le contrôle lit :** chaque `.rs` sous `src/` de chaque dossier de `crates/` (aujourd'hui `noyau`, `cahier`,
`stockage`, `synchro`, `commandes` et `services`), listés à l'exécution, et sous `app/src-tauri/src`, à partir de
`env!("CARGO_MANIFEST_DIR")/../..`, à la façon de `RACINES_LUES` des services
(`crates/services/tests/statique.rs:120`) : une crate native ajoutée est lue sans retouche du contrôle. Les lignes dont le début, blancs ôtés, est `//` (commentaires et docs)
ne comptent pas ; tout le reste se lit en sous-chaîne. L'interface n'y est pas : elle a sa propre règle (tâche 12 ;
question 5).

**Ce qu'il refuse**, une fonction pure, `controler(sources: &[(String, String)]) -> Vec<String>`, chaque faute
nommée par son fichier, sa ligne et le mot trouvé :

| Famille | Mots | Admis seulement dans |
|---|---|---|
| Les chemins devinés | `home_dir`, `temp_dir`, `data_dir`, `data_local_dir`, `config_dir`, `cache_dir`, `document_dir`, `app_data_dir`, `app_local_data_dir`, `app_config_dir`, `app_cache_dir`, `app_log_dir`, `download_dir`, `desktop_dir`, `audio_dir`, `picture_dir`, `video_dir`, `public_dir`, `template_dir`, `font_dir`, `runtime_dir`, `executable_dir`, `resource_dir`, `BaseDirectory`, `SHGetKnownFolderPath`, `GetTempPath`, `"USERPROFILE"`, `"HOME"`, `"APPDATA"`, `"LOCALAPPDATA"`, `"TEMP"`, `"TMP"`, `dirs::` | `app/src-tauri/src/systeme.rs` ; et un seul `temp_dir`, dans `app/src-tauri/src/lib.rs` (celui de `dossier_jetable`) |
| L'horloge | `SystemTime`, `Instant`, `UNIX_EPOCH` | `app/src-tauri/src/systeme.rs` |
| L'aléa | `getrandom`, `rand::`, `OsRng`, `thread_rng` | `app/src-tauri/src/systeme.rs` |
| L'acteur d'Automerge | `ActorId::random`, `Automerge::new`, `.fork(`, `fork_at`, `Automerge::load`, `load_with_options` | `crates/stockage/src/documents.rs`, `crates/stockage/src/demarrage.rs`, `crates/services/src/application.rs`, `crates/services/src/en_memoire.rs` (question 6) |

- [ ] **Étape 1 : écrire les tests.** `i30_hors_de_systeme_ni_chemin_devine_ni_horloge_ni_alea` lance `controler`
  sur les sources lues et exige une liste vide ; `i30_chaque_racine_lue_porte_des_sources` exige au moins une
  source sous le `src/` de chaque dossier de `crates/`, au moins six dossiers, et dans `app/src-tauri/src` (une
  racine renommée laisserait le contrôle passer sans rien lire) ;
  `i30_chaque_controle_refuse_ce_qu_on_y_plante` appelle `controler` sur des sources plantées, une faute par cas,
  et exige la faute nommée :
  - un second `std::env::temp_dir()` dans `app/src-tauri/src/lib.rs` ;
  - `use std::{env, path::Path};` puis `env::temp_dir()` dans `crates/cahier/src/main.rs` (l'import groupé) ;
  - `app.path().home_dir()` dans `app/src-tauri/src/lib.rs` ;
  - `app.path().resolve("x", BaseDirectory::Download)` dans `app/src-tauri/src/lib.rs` ;
  - `std::env::var("LOCALAPPDATA")` dans `crates/stockage/src/lib.rs` ;
  - `SystemTime::now()` dans `crates/services/src/lib.rs` ;
  - `getrandom::fill(&mut octets)` dans `crates/stockage/src/documents.rs` ;
  - `Automerge::new()` dans `crates/synchro/src/lib.rs`, et `.fork()` dans `crates/services/src/lecture.rs` ;
  - `ActorId::random()` dans `app/src-tauri/src/systeme.rs` (l'acteur n'y est pas admis) ;

  et une source dont seule une ligne `/// home_dir` porte le mot ne donne aucune faute.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: I30, ni chemin deviné, ni horloge, ni aléa hors de systeme`),
  pousser. Attendu à « Lint de l'établi » de `windows` : `cannot find function controler`.
- [ ] **Étape 3 : écrire le code** de `controler` et de la lecture des sources, dans le fichier de tests.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests de l'établi » de `windows` : `statique.rs` `ok. 3 passed`. Une
  faute trouvée sur les vraies sources se corrige dans le code qui la porte, jamais en ajoutant une exception.
- [ ] **Étape 5 : committer** `git add app/src-tauri/tests/statique.rs` (`test: le contrôle statique d'I30`).

### Tâche 6 : I30, la sentinelle ; le graphe de l'établi

**Fichiers :** modifier `app/src-tauri/src/systeme.rs` (le garde-fou), `.github/workflows/ci.yml` (`verifier` : « Le
graphe de l'établi », après « Le graphe de l'interface », `:102` ; `windows` : « La sentinelle d'I30 », après
« Tests de l'établi », `:185-186`).

- [ ] **Étape 1 : écrire le garde-fou** (dans `systeme.rs`, `#[cfg(test)]`).

```rust
#[test]
#[ignore = "le garde-fou d'I30 : l'étape « La sentinelle d'I30 » du job windows le lance"]
fn i30_le_dossier_que_tauri_resout_tombe_dans_la_sentinelle() {
    let sentinelle = std::env::var("PUPITRE_SENTINELLE")
        .expect("PUPITRE_SENTINELLE nomme la sentinelle : le garde-fou ne tourne que sous elle");
    let identifiant = identifiant_de_tauri_conf(); // lu dans tauri.conf.json par serde_json
    assert_eq!(identifiant, "app.pupitre.etabli");
    let mut contexte = tauri::test::mock_context(tauri::test::noop_assets());
    contexte.config_mut().identifier = identifiant.clone();
    let app = tauri::test::mock_builder().build(contexte).unwrap();
    let dossier = dossier_de_l_application(app.handle()).unwrap();
    assert!(dossier.starts_with(&sentinelle), "le dossier résolu n'est pas dans la sentinelle");
    assert!(dossier.ends_with(&identifiant), "le dossier résolu n'est pas celui de l'application");
}
```

  `identifiant_de_tauri_conf()` lit `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json"))`. Le
  message d'échec ne nomme aucun chemin.
- [ ] **Étape 2 : écrire l'étape de la sentinelle**, dans `windows`, sous PowerShell :

```yaml
      # I30 (§ 70 · 3) : les tests de l'établi, relancés sous un dossier de l'application redirigé vers
      # une sentinelle vide. USERPROFILE porte la sentinelle, que SHGetKnownFolderPath développe ; APPDATA et
      # LOCALAPPDATA, ses sous-dossiers, créés d'abord (sans eux, l'appel échoue). CARGO_HOME et RUSTUP_HOME
      # gardent leurs vrais dossiers, lus avant la redirection. L'essai prouve que la recherche voit un
      # fichier caché ; $LASTEXITCODE se lit, puisqu'un exécutable en échec n'arrête pas PowerShell.
      - name: La sentinelle d'I30
        run: |
          $cargo = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
          $rustup = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE '.rustup' }
          $essai = Join-Path $env:RUNNER_TEMP 'sentinelle-essai'
          New-Item -ItemType Directory -Force $essai | Out-Null
          (New-Item -ItemType File (Join-Path $essai 'cache')).Attributes = 'Hidden'
          if (-not (Get-ChildItem -LiteralPath $essai -Recurse -File -Force)) { throw "l'essai ne voit pas le fichier caché" }
          $s = Join-Path $env:RUNNER_TEMP 'sentinelle'
          New-Item -ItemType Directory -Force (Join-Path $s 'AppData\Roaming'), (Join-Path $s 'AppData\Local') | Out-Null
          $env:CARGO_HOME = $cargo
          $env:RUSTUP_HOME = $rustup
          $env:USERPROFILE = $s
          $env:APPDATA = Join-Path $s 'AppData\Roaming'
          $env:LOCALAPPDATA = Join-Path $s 'AppData\Local'
          $env:PUPITRE_SENTINELLE = $s
          cargo test -p etabli -- --include-ignored
          if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
          $paru = Get-ChildItem -LiteralPath $s -Recurse -File -Force
          if ($paru) { $paru | ForEach-Object { $_.FullName.Substring($s.Length) }; throw 'un fichier a paru dans la sentinelle' }
```

- [ ] **Étape 3 : écrire l'étape du graphe de l'établi**, dans `verifier`, sous `bash -e` :

```yaml
      # L'établi tel qu'il se livre, pour Windows, sa seule cible : ni pannes, ni en-memoire, ni
      # protocol-asset (décision 7 ; I31). L'essai sur le stockage prouve que la recherche trouve une
      # fonction quand ses dépendances de test l'activent, et que -e no-dev l'écarte sans elles.
      - name: Le graphe de l'établi
        run: |
          interdits='feature "(en-memoire|pannes|protocol-asset)"'
          sortie=$(cargo tree -p stockage -e features --prefix none)
          grep -Eq '^stockage feature "pannes"' <<<"$sortie"
          sortie=$(cargo tree -p stockage -e no-dev,features --prefix none)
          if grep -Eq '^stockage feature "pannes"' <<<"$sortie"; then exit 1; fi
          sortie=$(cargo tree -p etabli -e no-dev,features --prefix none --target x86_64-pc-windows-msvc)
          grep -Eq '^tauri feature "' <<<"$sortie"
          if grep -E "$interdits" <<<"$sortie"; then exit 1; fi
```

- [ ] **Étape 4 : voir le rouge, puis le vert.** Pousser un premier commit où le garde-fou exige
  `dossier.ends_with("autre.identifiant")` (`test: le garde-fou d'I30 devant un faux identifiant`) : attendu à « La
  sentinelle d'I30 », `panicked at`, `i30_le_dossier_que_tauri_resout_tombe_dans_la_sentinelle`, et « Tests de
  l'établi » vert (le test y est ignoré). Puis l'identifiant de `tauri.conf.json` : l'étape verte, `ok` pour le
  garde-fou, aucun fichier paru ; « Le graphe de l'établi » vert. Si l'essai d'une étape échoue (le format de
  `cargo tree`, l'attribut caché), l'étape se corrige, jamais l'essai ne s'ôte.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/systeme.rs .github/workflows/ci.yml`
  (`ci: la sentinelle d'I30 sous windows et le graphe de l'établi dans verifier`).

### Tâche 7 : les documents de la PR (b), les délais, et la revue de la PR (b)

**Fichiers :** `docs/70-qualite-et-verification.md` (I30, `:139` ; § 8, « Où en est le tableau »),
`CONVENTIONS.md` (lignes `verifier` et `windows` des portes de CI), `CLAUDE.md` (« Les crates », « Commandes »),
`.github/workflows/ci.yml` (les délais).

- [ ] **Étape 1 : I30, au § 70 · 3** (question 7 ; L2). « … par une étape du job `windows` ou par l'exécutable de test
  lui-même, ce que fixe le plan du lot 1, … » devient « … par une étape du job `windows`, « La sentinelle d'I30 », … ».
  La phrase « Tout chemin deviné y tombe ainsi, qu'il passe par Tauri, par `SHGetKnownFolderPath` ou par une
  variable. » devient : « Les dossiers de l'application, locaux et itinérants, y tombent, comme tout ce qui lit
  `USERPROFILE`, `APPDATA` ou `LOCALAPPDATA` ; le dossier personnel (`FOLDERID_Profile`, `home_dir` de Tauri et de
  `dirs`) et le dossier temporaire (`TMP`) n'y tombent pas, et le contrôle statique seul les refuse. » Le contrôle
  statique : « dans le code de l'application » devient « dans les `src/` des crates natives » ; « hors du seul endroit
  qui reçoit les chemins du système » nomme `systeme` ; la phrase ajoute l'horloge et l'aléa, et l'exception de
  l'acteur d'Automerge.
- [ ] **Étape 2 : le § 70 · 8.** « Où en est le tableau, après INC-0 » devient « Où en est le tableau » : la ligne 4
  dit qu'I01, I10 et I28 ont leur test depuis la tranche 2 du lot 1 d'INC-1, qu'I14 en a une part depuis la tranche 1
  (le reste en tranches 5 et 6), et qu'I30 a son garde-fou et son contrôle statique depuis cette PR, son second test
  (la démonstration) venant en tranche 7. Les tâches 13 (I31) et 20 (I29) complètent ce même paragraphe.
- [ ] **Étape 3 : `CONVENTIONS.md` et `CLAUDE.md`.** Ligne `verifier` : « Le graphe de l'établi » ; ligne `windows` :
  « La sentinelle d'I30 » et les tests d'I30. « Les crates », `etabli` : son module `systeme`, seul à lire le dossier
  de l'application, l'heure et l'aléa, et le fil du fonds, qui possède la racine des données et les services.
  « Commandes » : la commande du graphe de l'établi, et ce qu'elle refuse.
- [ ] **Étape 4 : les délais.** Lire la durée de `verifier` et de `windows` sur une exécution sans cache (« No cache
  found. ») ; chaque délai vaut deux fois le passage mesuré, arrondi à la minute ; le commentaire du délai dit la
  mesure, sa date et la PR. Un délai qui ne change pas le dit dans la description de la PR.
- [ ] **Étape 5 : committer** `git add docs/70-qualite-et-verification.md CONVENTIONS.md CLAUDE.md
  .github/workflows/ci.yml` (`docs: I30, sa sentinelle et son contrôle ; les délais de la PR (b)`).
- [ ] **Étape 6 : la revue de la PR (b).** La revue de la PR entière, sur sa tête, par le modèle le plus capable :
  le fil et sa fermeture, la sentinelle (son essai, son relevé), le contrôle d'I30 contre les vraies sources, les
  délais. Puis les corrections, leur relecture sur la tête qui part, le corps final (« Écarts à la spec :
  Aucun. » ; la version de `getrandom` relevée ; les limites : l'échec de `getrandom`, non provoqué), `gh pr ready`,
  la fusion par squash.


## PR (c) · la CSP, les droits, le protocole qui refuse, I31

**Branche :** `t3c-csp-et-droits`. **Titre :** `feat: la CSP exacte, les droits nommés et le protocole des
captations ; I31` (76). **Documents :** § 40 · 7.5 ; § 70 · 8 ; `CONVENTIONS.md` (`documentation`, `verifier`,
`windows`) ; `CLAUDE.md` ; `.gitignore`. **Fin :** la tâche 13.

### Tâche 8 : la CSP exacte, et le point 9 de `verif-faits.mjs`

**Fichiers :** modifier `app/src-tauri/tauri.conf.json` (`:17`), `scripts/verif-faits.mjs` (l'en-tête des points,
`:6-48` ; `principal`, `:734`, le point 9 avant l'appel au réseau), `scripts/verif-faits.test.mjs`,
`docs/40-architecture.md` (§ 7.5 : la ligne `style-src`, `:509` ; la ligne `img-src`, `media-src` ; la ligne
`connect-src` ; le bloc, `:517`).

**Interfaces — produit** (`scripts/verif-faits.mjs`) :

```js
/** Le point 9 : la CSP de tauri.conf.json, une chaîne, égale au bloc qui suit « La CSP exacte, que
 * `tauri.conf.json` recopie : » dans le § 40 · 7.5 ; et aucun tauri.windows.conf.json, que Tauri fusionnerait
 * par-dessus. Rend la liste des fautes, en français, vide si tout est juste. */
export function comparerLaCsp({ conf, section, fichierDeWindows }) { /* … */ }
```

La CSP exacte, 277 octets, en ASCII, sur une ligne (L4 ; question 11) :

```text
default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' http://captation.localhost; media-src 'self' http://captation.localhost; connect-src 'self' http://ipc.localhost; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'
```

- [ ] **Étape 1 : écrire les tests** dans `scripts/verif-faits.test.mjs` (`node:test`), sur une section et une
  configuration écrites dans le test : la CSP égale au bloc, aucune faute ; une CSP en objet (`{"default-src":
  "'self'"}`), une faute « n'est pas une chaîne » ; la CSP d'un octet différente (`'none'` devenu `'self'` dans
  `object-src`), une faute « diffère » ; `fichierDeWindows: true`, une faute qui nomme `tauri.windows.conf.json` ;
  une section sans le bloc, une faute « bloc absent ».
- [ ] **Étape 2 : voir le rouge.** `node --test scripts/verif-faits.test.mjs` sur le poste, après le `git add`
  (LE-7) : `comparerLaCsp` n'est pas exportée. Committer (`test: le point 9 de verif-faits, la CSP`).
- [ ] **Étape 3 : écrire le code.** `comparerLaCsp`, puis le point 9 dans `principal` : il lit
  `app/src-tauri/tauri.conf.json` (`app.security.csp`), la section « 7.5 » de `docs/40-architecture.md` par
  `section` (`:439`), et l'existence de `app/src-tauri/tauri.windows.conf.json` ; chaque faute fait échouer.
  L'en-tête du script ajoute le point 9.
- [ ] **Étape 4 : poser la CSP et corriger le § 40 · 7.5.** `tauri.conf.json:17` : `"security": { "csp": "<la CSP
  exacte>" }`, sans `devCsp`, `dangerousDisableAssetCspModification` ni `assetProtocol`. Au § 7.5 :
  - le bloc devient la CSP exacte ci-dessus ;
  - la ligne `connect-src` : « `'self'`, et le canal des commandes de Tauri, `http://ipc.localhost`, sa forme sous
    Windows et Android ; `ipc:`, sa forme sur les plateformes qui ne sont pas des cibles, n'est pas admis » ;
  - la ligne `img-src`, `media-src` : « comme `ipc` » devient « comme le canal des commandes » ;
  - la ligne `style-src` (question 9 ; L1) : « … jamais par un attribut `style`, que la CSP refuse, ni par
    `cssText`, que la CSP admet mais que le contrôle statique de l'interface refuse, pour qu'un style calculé ne se
    pose que d'une façon » ;
  - après la phrase de la comparaison : « Aucun `devCsp` : `cargo tauri dev` suit la même CSP, et aucun job ne le
    lance. »
- [ ] **Étape 5 : voir le vert.** Sur le poste, après le `git add` : `node scripts/verif-faits.mjs` et
  `node --test scripts/verif-faits.test.mjs`, verts ; en CI, `documentation` vert, et « Construire l'installateur »
  vert dans `windows`.
- [ ] **Étape 6 : committer** `git add app/src-tauri/tauri.conf.json scripts/verif-faits.mjs
  scripts/verif-faits.test.mjs docs/40-architecture.md` (`feat: la CSP exacte du § 40 · 7.5, que verif-faits compare`).

### Tâche 9 : le manifeste de l'application et les droits de Windows

**Fichiers :** modifier `crates/commandes/src/lib.rs` (`NOMS`, `:15-32` : `"exporter_cahier"` y entre, et la doc
dit que la liste est complète), `crates/commandes/tests/serialisation.rs` (`:167`, `NOMS.len()` passe de `13` à
`14`), `app/src-tauri/Cargo.toml` (`[build-dependencies]` : `commandes`, par chemin),
`app/src-tauri/build.rs`, `.gitignore`, `docs/40-architecture.md` (§ 7.5 : les droits, la table, `:548-551`) ;
créer `app/src-tauri/capabilities/windows.json`, `app/src-tauri/tests/i31.rs` ; retirer
`app/src-tauri/capabilities/default.json` (`git rm`).

**`build.rs` :**

```rust
fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(commandes::NOMS)),
    )
    .expect("le manifeste de l'application ne s'est pas construit");
}
```

**`capabilities/windows.json`** : `"$schema": "../gen/schemas/desktop-schema.json"`, `"identifier": "windows"`, une
description (« Les droits de la fenêtre principale sous Windows : chaque commande de l'application, nommée ; aucun
droit du noyau de Tauri ni d'un greffon (§ 40 · 7.5) »), `"platforms": ["windows"]`, `"windows": ["main"]`, et
`"permissions"` : `allow-noter`, `allow-completer`, `allow-poser-une-etiquette`, `allow-oter-une-etiquette`,
`allow-relier`, `allow-delier`, `allow-regrouper`, `allow-retirer`, `allow-retablir`, `allow-annuler`,
`allow-vue-d-une-idee`, `allow-idees`, `allow-journal`, `allow-exporter-cahier`.

**`.gitignore`** : `app/src-tauri/permissions/autogenerated/`, à côté de `app/src-tauri/gen/`.

**Interfaces — produit** (`app/src-tauri/tests/i31.rs`, des fonctions pures que la tâche 11 complète) :

```rust
/// Les fichiers de `capabilities/` : seul `windows.json`.
fn controler_les_fichiers_de_droits(noms: &[String]) -> Vec<String>;
/// Les droits de `windows.json` : `platforms` vaut `["windows"]`, `windows` vaut `["main"]`, et
/// `permissions` égale, comme ensemble, `allow-` suivi de chaque nom de `NOMS`, chaque `_` devenu `-` ;
/// aucun droit qui porte `:` (un droit du noyau de Tauri ou d'un greffon).
fn controler_les_droits(droits: &serde_json::Value, noms: &[&str]) -> Vec<String>;
/// `build.rs` : `AppManifest::new().commands(commandes::NOMS)`, blancs ignorés, et aucun
/// `default_permission`, qui permettrait tout.
fn controler_le_manifeste(build_rs: &str) -> Vec<String>;
```

- [ ] **Étape 1 : écrire les tests.** `i31_seul_windows_json_porte_des_droits`,
  `i31_les_droits_nomment_chaque_commande_et_rien_d_autre` et `i31_build_rs_tire_le_manifeste_de_noms`, sur les vrais
  fichiers ; `i31_chaque_controle_des_droits_refuse_ce_qu_on_y_plante`, sur des entrées plantées : un `default.json`
  à côté de `windows.json` ; `allow-exporter-cahier` absent ; `allow-supprimer` en trop ; `allow-exporter_cahier` (le
  `_` gardé) ; `core:default` ; `dialog:default` ; `platforms` absent ; `build.rs` sans manifeste, puis avec
  `.default_permission(…)`.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: I31, les droits de Windows et le manifeste`), pousser, ouvrir la
  PR en brouillon si ce n'est fait. Attendu à « Tests de l'établi » de `windows` : `panicked at`,
  `i31_seul_windows_json_porte_des_droits` (`default.json` est là).
- [ ] **Étape 3 : écrire les fichiers**, `NOMS` compris, et corriger le § 7.5 :
  - les droits : « … nomme une à une les commandes permises, et les seuls droits du noyau de Tauri que l'interface
    emploie : au lot 1, aucun ; l'écoute d'un signal viendra avec le lot 3 » (question 10 ; L3) ;
  - la ligne Windows de la table : « en mode démonstration seulement » s'achève par « une borne que le côté natif
    tient (tranche 7 du lot 1), les droits ne distinguant pas les modes » ;
  - (f), `:548-551` : « La liste exacte des commandes vit dans `commandes::NOMS` : `build.rs` en tire le manifeste
    de l'application, chaque fichier de droits la recopie en `allow-<nom>`, chaque `_` devenu `-`, et I31 compare
    les trois, avec `generate_handler!` ; la CSP se compare au bloc de la CSP exacte, plus haut (§ 70 · 3). Si un
    écran demande davantage, la PR qui l'ajoute le dit et l'écrit ici. »
- [ ] **Étape 4 : voir le vert.** « Cargo.lock à jour » de `verifier` échoue d'abord : prendre l'artefact
  `cargo-lock`, le relire (`commandes` en dépendance de construction d'`etabli`), le committer, comme à la tâche 2.
  Puis, à « Tests » de `verifier`, `les_noms_sont_uniques_et_en_snake_case` vert ; « Construire l'installateur » vert
  (`try_build` accepte le manifeste et les droits), puis « Tests de l'établi » : `i31.rs` `ok. 4 passed`. Sur le poste :
  `git check-ignore -q app/src-tauri/permissions/autogenerated/allow-noter.toml`, code 0 (sans `-v`, qui rendrait 0
  aussi sur une négation).
- [ ] **Étape 5 : committer** `git add crates/commandes/src/lib.rs crates/commandes/tests/serialisation.rs
  app/src-tauri/Cargo.toml app/src-tauri/build.rs app/src-tauri/capabilities/windows.json app/src-tauri/tests/i31.rs
  .gitignore docs/40-architecture.md Cargo.lock`,
  après `git rm app/src-tauri/capabilities/default.json` (`feat: le manifeste de l'application et les droits nommés de
  Windows`).

### Tâche 10 : le protocole des captations, qui refuse tout

**Fichiers :** modifier `app/src-tauri/src/lib.rs` (`run`, et la fonction du gestionnaire).

**Interfaces — produit :**

```rust
/// Le gestionnaire du protocole `captation` jusqu'à la tranche 5 : toute requête est refusée, 404,
/// sans corps (question 12). WebView2 le sert à `http://captation.localhost/`.
pub(crate) fn refuser_la_captation(requete: &tauri::http::Request<Vec<u8>>) -> tauri::http::Response<Vec<u8>>;
```

`run()` : `.register_uri_scheme_protocol("captation", |_contexte, requete| refuser_la_captation(&requete))`, la
signature de la fermeture vérifiée sur docs.rs de `tauri` 2.12.5.

- [ ] **Étape 1 : écrire le test** `le_protocole_des_captations_refuse_toute_requete` (dans `lib.rs`, module de
  tests) : pour `http://captation.localhost/`, pour `http://captation.localhost/` suivi de soixante-quatre `a` (la
  forme d'une empreinte), et pour `http://captation.localhost/../donnees/verrou`, la réponse a le statut 404 et un
  corps vide.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: le protocole des captations refuse toute requête`), pousser.
  Attendu à « Lint de l'établi » : `cannot find function refuser_la_captation`.
- [ ] **Étape 3 : écrire le code.**
- [ ] **Étape 4 : voir le vert.** « Tests de l'établi » `ok`, le test nommé compris.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/lib.rs` (`feat: le protocole des captations, enregistré, qui
  refuse tout`).

### Tâche 11 : I31 côté établi, la configuration et le gestionnaire

**Fichiers :** modifier `app/src-tauri/tests/i31.rs`.

**Interfaces — produit** (de plus, dans `i31.rs`) :

```rust
/// La CSP : chaque directive (`;`) ; son premier mot est son nom, hors de la comparaison ; chacun des suivants
/// (séparés par des blancs) est une source, et seules sont admises `'self'`, `'none'`,
/// `'wasm-unsafe-eval'`, `http://captation.localhost` et `http://ipc.localhost`, comparées entières, si bien
/// que `'unsafe-inline'`, `'unsafe-eval'` et toute autre origine sont refusées.
fn controler_la_csp(csp: &str) -> Vec<String>;
/// La configuration : `app.security.csp` est une chaîne ; sous `app.security`, aucune de
/// `dangerousDisableAssetCspModification`, `dangerous-disable-asset-csp-modification`, `assetProtocol`,
/// `asset-protocol`, `devCsp`, `dev-csp` ; sous chaque fenêtre de `app.windows`, ni `useHttpsScheme` ni
/// `use-https-scheme`.
fn controler_la_configuration(conf: &serde_json::Value) -> Vec<String>;
/// `generate_handler![…]` de `src/lib.rs` : le dernier segment de chaque chemin, comme ensemble, égale `NOMS`.
fn controler_le_gestionnaire(source_de_lib: &str, noms: &[&str]) -> Vec<String>;
/// Le `Cargo.toml` d'`etabli` : aucune fonction `protocol-asset`.
fn controler_le_cargo_toml(cargo_toml: &str) -> Vec<String>;
```

- [ ] **Étape 1 : écrire les tests.** `i31_la_csp_n_admet_que_ce_que_l_application_apporte`,
  `i31_tauri_conf_ne_pose_aucune_cle_interdite`, `i31_generate_handler_porte_chaque_nom` et
  `i31_le_cargo_toml_ne_tire_pas_protocol_asset`, sur les vrais fichiers ; `i31_chaque_controle_refuse_ce_qu_on_y_plante`,
  une plantation par forme : `'unsafe-inline'` dans `script-src` ; `'unsafe-eval'` ; `https://exemple.org` dans
  `img-src` ; et la CSP exacte, qui passe (`'wasm-unsafe-eval'`, le cas limite) ; une CSP en objet ; chacune des six
  clés de `security`, sous chacune de ses deux formes ; `useHttpsScheme` et `use-https-scheme` sous une fenêtre ;
  un `generate_handler!` sans `journal`, puis avec `supprimer` en trop ; `features = ["protocol-asset"]`.
- [ ] **Étape 2 : voir le rouge**, à « Lint de l'établi » (`cannot find function controler_la_csp`), puis le
  **vert**, les fonctions écrites : `i31.rs` `ok. 9 passed`.
- [ ] **Étape 3 : committer** `git add app/src-tauri/tests/i31.rs` (`test: I31, la CSP, la configuration et le
  gestionnaire des commandes`).

### Tâche 12 : I31 côté interface, son contrôle statique

**Fichiers :** créer `app/ui/tests/statique.rs`.

**Ce que le contrôle lit :** chaque `.rs` sous `app/ui/src/`, et `app/ui/index.html`, à partir de
`env!("CARGO_MANIFEST_DIR")`. Comme celui d'I30, il ignore les lignes qui commencent par `//`, blancs ôtés, et lit
le reste en sous-chaîne. La fonction englobante d'une ligne est le dernier `fn <nom>` qui la précède dans son
fichier.

**Ce qu'il refuse** (`controler(sources: &[(String, String)]) -> Vec<String>`) :

| Règle | Mots | Admis seulement |
|---|---|---|
| Du HTML posé | `inner_html`, `innerHTML`, `outer_html`, `outerHTML`, `insert_adjacent_html`, `insertAdjacentHTML` | `inner_html` une fois dans `src/vue_table.rs` (la page) et une fois dans `src/panneau_palette.rs` (la fiche) |
| Un style en attribut | `style=`, `"style"`, `cssText`, `css_text`, `<style` | nulle part (`style:` reste admis) |
| Le stockage du navigateur | `localStorage`, `local_storage`, `sessionStorage`, `session_storage`, `indexedDB`, `indexed_db`, `IdbFactory`, `CacheStorage`, `"caches"`, `cookie` | nulle part, l'appel comme le nom écrit en chaîne |
| L'horloge | `Date::new_0`, `Date::now`, `performance` | dans la fonction `date_du_jour` (question 5) |
| Le décalage local | `get_timezone_offset`, `getTimezoneOffset` | nulle part, jusqu'à la tâche 18 |

- [ ] **Étape 1 : écrire les tests.** `i31_l_interface_ne_pose_ni_html_ni_style_ni_stockage` sur les vraies sources,
  liste vide ; `i31_chaque_lieu_lu_porte_des_sources` ; `i31_chaque_controle_refuse_ce_qu_on_y_plante`, une faute par
  cas : un second `inner_html` dans `src/vue_table.rs` ; `prop:innerHTML=texte` ; `set_outer_html` ;
  `insert_adjacent_html` ; `style="left: 0"` dans `index.html` ; `style=format!(…)` dans un `.rs` ; `.set_css_text(…)`
  ; `cssText` ; `Reflect::get(&fenetre, &"localStorage".into())` ; `window.local_storage()` ; `"indexedDB"` ;
  `document.cookie` ; `js_sys::Date::new_0()` dans une fonction `autre`, refusée, et la même dans `date_du_jour`,
  admise ; `getTimezoneOffset`.
- [ ] **Étape 2 : voir le rouge** à « Lint de l'interface (natif, tests compris) » de `verifier` (`cannot find
  function controler`), puis le **vert** à « Tests » : `statique.rs` de l'interface `ok. 3 passed`.
- [ ] **Étape 3 : committer** `git add app/ui/tests/statique.rs` (`test: I31, le contrôle statique de l'interface`).

### Tâche 13 : les documents de la PR (c), les délais, et la revue de la PR (c)

**Fichiers :** `docs/70-qualite-et-verification.md` (§ 8), `CONVENTIONS.md` (`documentation` : le point 9 ;
`verifier` : le contrôle statique de l'interface ; `windows` : les tests d'I31), `CLAUDE.md` (la ligne de
`verif-faits.mjs`, qui compare aussi la CSP ; `etabli` : le manifeste, les droits, le protocole), `ci.yml` (les
délais).

- [ ] **Étape 1 : les documents.** Le § 70 · 8 ajoute I31, hors du protocole des captations (tranche 5). Les lignes de
  `CONVENTIONS.md` et de `CLAUDE.md` comme dit plus haut.
- [ ] **Étape 2 : les délais**, comme à la tâche 7.
- [ ] **Étape 3 : committer** `git add docs/70-qualite-et-verification.md CONVENTIONS.md CLAUDE.md
  .github/workflows/ci.yml` (`docs: I31, la CSP et les droits ; les délais de la PR (c)`).
- [ ] **Étape 4 : la revue de la PR (c).** La revue de la PR entière, sur sa tête, par le modèle le plus capable : la
  CSP relue octet pour octet contre le § 40 · 7.5, les droits contre `NOMS`, chaque plantation. Le corps final dit
  les limites : `TAURI_CONFIG`, que rien ne montre ; les droits engendrés d'une commande retirée, qui restent sur un
  poste ; aucun `devCsp` ; la fenêtre blanche, que seul l'essai installé de la tâche 21 montre. Puis les
  corrections, leur relecture sur la tête qui part, `gh pr ready`, la fusion par squash.


## PR (d) · la boîte, l'état à l'ouverture, le bandeau, I29

**Branche :** `t3d-la-boite`. **Titre :** `feat: la boîte d'idées à l'écran, son bandeau et l'annulation ; I29`
(71). **Documents :** le glossaire ; CAP-02 ; CAP-07 ; § 41 · 6 ; § 50 · 5 ; § 70 · 4.1, 7 et 8 ; `docs/pistes.md` ;
le registre (LE-15, AM-3) ; `.cargo/mutants.toml` ; `CONVENTIONS.md` (`verifier`, `windows`, `mutants`) ;
`CLAUDE.md`. **Fin :** les tâches 20 et 21.

### Tâche 14 : la boîte au noyau : l'ordre, le filtre, le compte

**Fichiers :** créer `crates/noyau/src/donnees/boite.rs`, `crates/noyau/tests/boite.rs` ; modifier
`crates/noyau/src/donnees/mod.rs`, `crates/noyau/src/donnees/gestes.rs` (`:411` et `:440`),
`crates/noyau/src/donnees/idee.rs` (la doc de `reliee`, `:35`), `crates/noyau/tests/gestes.rs`,
`.cargo/mutants.toml` (`"crates/noyau/src/donnees/boite.rs"` dans `examine_globs`, `"--test", "boite"` dans
`additional_cargo_test_args`, « Seize » devenu « Dix-sept » à `:2` et « seize » à `:26`).

**Interfaces — consomme :** `IdeeLue`, `Provenance`, `Sorte` (`noyau::donnees::gestes`) ; `Assiette`, `assiette`
(`noyau::donnees::idee`).

**Interfaces — produit** (`noyau::donnees::boite`) :

```rust
/// L'ordre de la boîte : il n'écrit rien et ne s'annule pas ; le tri est un geste (question 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordre { Date, Etiquette, Duree, Reliee, Provenance }
/// Un contenu que le filtre retient (CAP-02) : le texte de l'idée, une prise, une photo, un MIDI versé
/// (seul ou qui accompagne un stem), un texte versé (question 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Contenu { TexteDeLIdee, Prise, Photo, MidiVerse, TexteVerse }
/// Une idée dans la boîte : l'idée lue, ses contenus, sa durée (`idee::duree`) et son état de reliée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnBoite { pub idee: IdeeLue, pub contenus: BTreeSet<Contenu>, pub duree_ms: Option<u64>, pub reliee: bool }
/// La boîte rangée : le compte des non reliées, l'assiette, et les idées que retient le filtre, dans l'ordre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boite { pub non_reliees: usize, pub assiette: Assiette, pub retenues: Vec<EnBoite> }
/// Les contenus d'une idée : son texte (`TexteDeLIdee` s'il y en a un), les sortes de ses captations
/// (`Prise`, `MidiVerseSeul` → `MidiVerse`, `TexteVerse`), et une photo si `une_photo`.
pub fn contenus_de(idee: &IdeeLue, sortes: impl IntoIterator<Item = Sorte>, une_photo: bool) -> BTreeSet<Contenu>;
/// Range `idees` selon `ordre`, en un ordre total (question 21) : la date, la plus récente d'abord ;
/// l'étiquette, sur la plus petite de l'idée, octet pour octet, les idées sans étiquette en fin ; la durée,
/// la plus longue d'abord, les idées sans durée en fin ; les non reliées d'abord ; les captées d'abord ;
/// puis, à égalité, la date décroissante, puis l'identifiant, octet pour octet (ce n'est pas l'ordre de
/// création de deux idées notées dans la même seconde).
pub fn ordonner(idees: &mut [EnBoite], ordre: Ordre);
/// La boîte : les idées non retirées (question 14), le compte des non reliées, qui ne suit pas le filtre,
/// l'assiette (`idee::assiette` sur les non retirées, chacune portant la donnée, puis `illisibles` ajoutées
/// aux seules considérées : question 19), et celles que retient `filtre` (aucun : toutes), rangées.
pub fn boite(idees: Vec<EnBoite>, illisibles: usize, ordre: Ordre, filtre: Option<Contenu>) -> Boite;
/// Les seules idées retirées de `idees`, par la date du retrait, la plus récente d'abord, puis par
/// l'identifiant.
pub fn retirees(idees: Vec<EnBoite>) -> Vec<EnBoite>;
```

Le module évite, jusque dans ses commentaires, les mots que refuse le contrôle statique du noyau
(`crates/noyau/tests/statique.rs:22-48` : `Instant`, `File`, `io::`, `time::`, `HashMap`, `random`…).

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/boite.rs`), sur un jeu inventé de six idées : `idee(id, le,
  etiquettes, captee)` et `en_boite(idee, contenus, duree_ms, reliee)` sont des fonctions de test ;
  `jeu_des_retirees()` rend F (retirée à 500), G (`01G`, retirée à 400), H (`01H`, retirée à 500) et E, non retirée.

| Idée | `le` | Étiquettes | Contenus | Durée | Reliée | Provenance | Retirée |
|---|---|---|---|---|---|---|---|
| A, `01A` | 100 | `pont` | texte de l'idée, prise | 47 000 | non | captée | non |
| B, `01B` | 300 | aucune | prise, MIDI versé | aucune | oui | importée | non |
| C, `01C` | 200 | `Pont`, `refrain` | prise, texte versé | 12 000 | non | importée | non |
| D, `01D` | 300 | `élan` | texte de l'idée, prise | 47 000 | oui | captée | non |
| E, `01E` | 200 | `pont` | texte de l'idée | aucune | non | captée | non |
| F, `01F` | 400 | `pont` | prise | aucune | non | captée | à 500 |

```rust
#[test]
fn chaque_ordre_est_total_et_se_departage_par_la_date_puis_l_identifiant() {
    let ids = |ordre| ids_de(boite(jeu(), 0, ordre, None).retenues);
    assert_eq!(ids(Ordre::Date), ["01B", "01D", "01C", "01E", "01A"]);
    // `Pont` (P, 0x50) passe avant `pont` ; `élan` (0xC3) après ; sans étiquette, en fin.
    assert_eq!(ids(Ordre::Etiquette), ["01C", "01E", "01A", "01D", "01B"]);
    assert_eq!(ids(Ordre::Duree), ["01D", "01A", "01C", "01B", "01E"]);
    assert_eq!(ids(Ordre::Reliee), ["01C", "01E", "01A", "01B", "01D"]);
    assert_eq!(ids(Ordre::Provenance), ["01D", "01E", "01A", "01B", "01C"]);
}

#[test]
fn le_filtre_retient_un_contenu_a_la_fois() {
    let ids = |filtre| ids_de(boite(jeu(), 0, Ordre::Date, Some(filtre)).retenues);
    // Une idée qui a une durée a une prise (`idee::duree`) ; B a une prise sans durée ; F, retirée, n'entre jamais.
    assert_eq!(ids(Contenu::Prise), ["01B", "01D", "01C", "01A"]);
    assert_eq!(ids(Contenu::Photo), Vec::<&str>::new()); // « 0 idée » (LE-6, règle 6)
    assert_eq!(ids(Contenu::MidiVerse), ["01B"]);
    assert_eq!(ids(Contenu::TexteVerse), ["01C"]);
    assert_eq!(ids(Contenu::TexteDeLIdee), ["01D", "01E", "01A"]);
}

#[test]
fn le_compte_des_non_reliees_ne_suit_pas_le_filtre() {
    for filtre in [None, Some(Contenu::Prise), Some(Contenu::Photo)] {
        let b = boite(jeu(), 0, Ordre::Date, filtre);
        assert_eq!(b.non_reliees, 3); // A, C, E ; F, retirée, n'y compte pas
        assert_eq!(b.assiette, Assiette { considerees: 5, portant: 5, decidees: 5 });
    }
}

#[test]
fn une_idee_illisible_compte_parmi_les_interrogees() {
    let b = boite(jeu(), 1, Ordre::Date, None);
    assert_eq!(b.non_reliees, 3);
    assert_eq!(b.assiette, Assiette { considerees: 6, portant: 5, decidees: 5 });
}

#[test]
fn les_retirees_se_rangent_par_la_date_du_retrait() {
    // F (retirée à 500), G (à 400), H (à 500, identifiant après F) : F, H, G ; E, non retirée, n'y entre pas.
    assert_eq!(ids_de(retirees(jeu_des_retirees())), ["01F", "01H", "01G"]);
}

#[test]
fn les_contenus_d_une_idee_viennent_de_son_texte_de_ses_sortes_et_de_la_photo() {
    let avec_texte = idee_avec_texte("01A", Some("Une idée de pont"));
    assert_eq!(
        contenus_de(&avec_texte, [Sorte::Prise, Sorte::MidiVerseSeul], false),
        BTreeSet::from([Contenu::TexteDeLIdee, Contenu::Prise, Contenu::MidiVerse])
    );
    let sans_texte = idee_avec_texte("01B", None);
    assert_eq!(
        contenus_de(&sans_texte, [Sorte::TexteVerse], true),
        BTreeSet::from([Contenu::Photo, Contenu::TexteVerse])
    );
}
```

  Dans `crates/noyau/tests/gestes.rs`, `une_etiquette_de_blancs_est_refusee` : poser `"   "` et ôter `"  "` rendent
  `Impossible::EtiquetteVide` ; poser `"pont "` passe, l'étiquette gardée telle quelle (son identité reste ses
  octets, question 7 de la tranche 2).
- [ ] **Étape 2 : voir le rouge.** Committer (`test: l'ordre, le filtre et le compte de la boîte au noyau`), pousser,
  ouvrir la PR en brouillon. Attendu à « Lint » de `verifier` : `unresolved import noyau::donnees::boite`.
- [ ] **Étape 3 : écrire le code.** `ordonner` : `sort_by` sur la clé de l'ordre, puis `le` décroissant, puis `id` ;
  la plus petite étiquette est la première de l'ensemble (`BTreeSet`). `boite` : les non retirées, le compte, l'assiette,
  le filtre, puis `ordonner`. `gestes.rs:411` et `:440` : `etiquette.trim().is_empty()`. La doc de `idee::reliee` :
  « le lien « apparenté » seul, jusqu'à « devenue » (INC-2) » (L14). `.cargo/mutants.toml`, comme dit plus haut.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `boite.rs` `ok. 6 passed`, `gestes.rs` du noyau
  un test de plus, `statique.rs` du noyau vert ; `mutants` sans survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/boite.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/src/donnees/gestes.rs crates/noyau/src/donnees/idee.rs crates/noyau/tests/boite.rs
  crates/noyau/tests/gestes.rs .cargo/mutants.toml` (`feat: l'ordre, le filtre et le compte de la boîte, au noyau`).

### Tâche 15 : les textes de la boîte au noyau : le compte, la date, la durée, l'étiquette saisie

**Fichiers :** modifier `crates/noyau/src/donnees/boite.rs`, `crates/noyau/src/donnees/gestes.rs` (le calcul civil
de `date_du_message`, `:241`, partagé), `crates/noyau/tests/boite.rs`.

**Interfaces — produit** (`noyau::donnees::boite`) :

```rust
/// « N idée(s) non reliée(s) », puis « · K illisible(s) » si l'assiette en compte (considérées − portant),
/// puis « · M interrogée(s) » ; singulier pour 0 et 1, pluriel dès 2 (question 18 ; LE-6, règle 9).
pub fn texte_du_compte(non_reliees: usize, assiette: Assiette) -> String;
/// « N idée(s) », puis de même.
pub fn texte_du_filtre(retenues: usize, assiette: Assiette) -> String;
/// « 3 nov. 2026 », « 1er janv. 2027 » : le jour de `heure` au décalage `decalage_minutes`, avec l'année, sans
/// heure ; les mois : janv., févr., mars, avr., mai, juin, juil., août, sept., oct., nov., déc. (question 24).
pub fn date_de_la_liste(heure: i64, decalage_minutes: i32) -> String;
/// « m:ss » sous une heure, « h:mm:ss » dès une heure, arrondie à la seconde inférieure ; `—` sans durée
/// (question 23).
pub fn duree_affichee(duree_ms: Option<u64>) -> String;
/// L'étiquette saisie, telle qu'elle s'envoie : les blancs autour ôtés, puis un `#` de tête (question 29).
pub fn etiquette_saisie(texte: &str) -> String;

// noyau::donnees::gestes, partagée
/// (année, mois de 0 à 11 depuis janvier, jour) de `heure` au décalage `decalage_minutes`.
pub(crate) fn date_civile(heure: i64, decalage_minutes: i32) -> (i64, usize, i64);
```

- [ ] **Étape 1 : écrire les tests.** `a(c, p)` vaut `Assiette { considerees: c, portant: p, decidees: p }`.

```rust
#[test]
fn le_compte_et_le_filtre_disent_leur_assiette_et_s_accordent() {
    assert_eq!(texte_du_compte(2, a(2, 2)), "2 idées non reliées · 2 interrogées");
    assert_eq!(texte_du_compte(0, a(0, 0)), "0 idée non reliée · 0 interrogée");
    assert_eq!(texte_du_compte(1, a(1, 1)), "1 idée non reliée · 1 interrogée");
    assert_eq!(texte_du_compte(50, a(50, 50)), "50 idées non reliées · 50 interrogées");
    assert_eq!(texte_du_compte(2, a(3, 2)), "2 idées non reliées · 1 illisible · 3 interrogées");
    assert_eq!(texte_du_compte(2, a(4, 2)), "2 idées non reliées · 2 illisibles · 4 interrogées");
    assert_eq!(texte_du_filtre(0, a(4, 4)), "0 idée · 4 interrogées");
    assert_eq!(texte_du_filtre(1, a(4, 4)), "1 idée · 4 interrogées");
    assert_eq!(texte_du_filtre(2, a(5, 4)), "2 idées · 1 illisible · 5 interrogées");
}

#[test]
fn la_date_de_la_liste_porte_l_annee_au_decalage_donne() {
    assert_eq!(date_de_la_liste(1_793_748_600, 0), "3 nov. 2026"); // 2026-11-03, 23 h 30 UTC
    assert_eq!(date_de_la_liste(1_793_748_600, 60), "4 nov. 2026");
    assert_eq!(date_de_la_liste(1_798_761_600, 0), "1er janv. 2027");
    assert_eq!(date_de_la_liste(1_798_761_600, -60), "31 déc. 2026");
    // Une idée notée le 25 octobre 2026 à 0 h 30, heure d'été : au décalage de l'instant, +120, elle reste
    // du 25 ; au décalage d'hiver, +60, elle serait du 24 (L11, la piste).
    assert_eq!(date_de_la_liste(1_792_881_000, 120), "25 oct. 2026");
    assert_eq!(date_de_la_liste(1_792_881_000, 60), "24 oct. 2026");
}

#[test]
fn chaque_mois_a_son_abreviation() {
    // Le 15 de chaque mois de 2026, à midi UTC : 20 454 jours vont du 1er janvier 1970 au 1er janvier 2026.
    let mois = [(14, "janv."), (45, "févr."), (73, "mars"), (104, "avr."), (134, "mai"), (165, "juin"),
        (195, "juil."), (226, "août"), (257, "sept."), (287, "oct."), (318, "nov."), (348, "déc.")];
    for (jour_de_l_annee, abreviation) in mois {
        let heure = (20_454 + jour_de_l_annee) * 86_400 + 43_200;
        assert_eq!(date_de_la_liste(heure, 0), format!("15 {abreviation} 2026"));
    }
}

#[test]
fn la_duree_s_arrondit_a_la_seconde_inferieure() {
    assert_eq!(duree_affichee(None), "—");
    assert_eq!(duree_affichee(Some(999)), "0:00");
    assert_eq!(duree_affichee(Some(47_000)), "0:47");
    assert_eq!(duree_affichee(Some(59_500)), "0:59");
    assert_eq!(duree_affichee(Some(60_000)), "1:00");
    assert_eq!(duree_affichee(Some(61_000)), "1:01");
    assert_eq!(duree_affichee(Some(3_599_999)), "59:59");
    assert_eq!(duree_affichee(Some(3_600_000)), "1:00:00");
    assert_eq!(duree_affichee(Some(3_661_000)), "1:01:01");
    assert_eq!(duree_affichee(Some(36_000_000)), "10:00:00");
}

#[test]
fn l_etiquette_saisie_perd_ses_blancs_et_un_diese_de_tete() {
    assert_eq!(etiquette_saisie("  #pont "), "pont");
    assert_eq!(etiquette_saisie("##pont"), "#pont");
    assert_eq!(etiquette_saisie("pont"), "pont");
    assert_eq!(etiquette_saisie("  "), "");
    assert_eq!(etiquette_saisie("#"), "");
}
```

- [ ] **Étape 2 : voir le rouge** à « Lint » (`cannot find function texte_du_compte`), committer (`test: les textes
  de la boîte au noyau`).
- [ ] **Étape 3 : écrire le code.** `date_civile` reprend le calcul de `date_du_message`, qui l'appelle ensuite : ses
  tests (`crates/noyau/tests/gestes.rs`) le gardent. `etiquette_saisie` : `trim`, puis `strip_prefix('#')` une fois,
  puis `trim` de nouveau, si bien que `"# pont"` donne `"pont"`.
- [ ] **Étape 4 : voir le vert.** `boite.rs` `ok. 11 passed` ; les tests de `date_du_message` inchangés ; `mutants`
  sans survivant.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/boite.rs crates/noyau/src/donnees/gestes.rs
  crates/noyau/tests/boite.rs` (`feat: les textes de la boîte au noyau, compte, date, durée et étiquette`).

### Tâche 16 : les commandes et les services de la boîte

**Fichiers :** modifier `crates/commandes/src/lib.rs`, `crates/commandes/tests/serialisation.rs`,
`crates/services/src/contrats.rs` (`Documents::avertissements`), `crates/services/src/lecture.rs`,
`crates/services/src/lib.rs`, `crates/services/src/en_memoire.rs`, `crates/stockage/src/adaptateurs.rs`,
`crates/services/tests/suite/mod.rs`, `crates/services/tests/commandes_en_memoire.rs`,
`crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — produit** (`commandes`) :

```rust
/// Chaque nom de commande, une constante ; `NOMS` les liste toutes, dans cet ordre.
pub const NOTER: &str = "noter"; /* … une par nom, puis : */
pub const BOITE: &str = "boite";
pub const RETIREES: &str = "retirees";
pub const ETAT_DU_FONDS: &str = "etat_du_fonds";
pub const EXPORTER_CAHIER: &str = "exporter_cahier";
/// Le plus grand nombre de lignes qu'une fenêtre rend (ADR-022 : des vues petites).
pub const FENETRE_MAX: u32 = 200;
/// Le plus grand nombre de caractères du texte d'une ligne ; la vue de l'idée le porte entier.
pub const LONGUEUR_D_UNE_LIGNE: usize = 200;
pub enum Ordre { Date, Etiquette, Duree, Reliee, Provenance }
pub enum Contenu { TexteDeLIdee, Prise, Photo, MidiVerse, TexteVerse }
/// Lire la boîte : l'ordre, le filtre, et la fenêtre demandée.
pub struct Boite { pub ordre: Ordre, pub filtre: Option<Contenu>, pub debut: u32, pub nombre: u32 }
/// Une fenêtre de lignes : son début et son nombre.
pub struct Fenetre { pub debut: u32, pub nombre: u32 }
/// L'assiette d'un compte (I09) : considérées, portant, décidées ; considérées − portant, au lot 1, ce sont
/// les idées illisibles.
pub struct Assiette { pub considerees: u64, pub portant: u64, pub decidees: u64 }
/// Une ligne de la boîte, ou des idées retirées : ce que la ligne montre, sans ancienneté (CAP-02 · 1).
pub struct LigneDeLaBoite {
    pub idee: String, pub le: i64, pub texte: Option<String>, pub etiquettes: Vec<String>,
    pub duree_ms: Option<u64>, pub reliee: bool, pub provenance: Provenance, pub retiree: Option<i64>,
}
/// La vue de la boîte : le compte des non reliées et son assiette, le nombre que retient le filtre et son
/// assiette, et les seules lignes de la fenêtre.
pub struct VueDeLaBoite {
    pub non_reliees: u64, pub assiette_du_compte: Assiette,
    pub retenues: u64, pub assiette_du_filtre: Assiette, pub lignes: Vec<LigneDeLaBoite>,
}
/// La vue des idées retirées : leur nombre, que l'écran ne montre pas (il règle la hauteur de la liste),
/// et les lignes de la fenêtre.
pub struct VueDesRetirees { pub total: u64, pub lignes: Vec<LigneDeLaBoite> }
/// L'état du fonds à l'ouverture, et après chaque commande : né ou non, la lecture seule, les avertissements
/// de l'ouverture, et le message du geste qu'annuler vise.
pub struct EtatDuFonds { pub ne: bool, pub lecture_seule: Option<Refus>, pub avertissements: Vec<String>, pub a_annuler: Option<String> }
```

Chaque type dérive `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`, comme ceux de la tranche 2 ; chaque entier
d'une réponse vient d'une source bornée, que sa doc nomme (un compte d'idées, un nombre de lignes).

**Interfaces — produit** (`services`) :

```rust
// contrats.rs
/// Les avertissements de l'ouverture du fonds, en français ; aucun, en mémoire.
fn avertissements(&self) -> Vec<String>;   // dans le trait Documents
// lecture.rs
/// Chaque idée qui se lit, retirée comprise, et le nombre de celles qui ne se lisent pas.
pub fn idees_et_illisibles(doc: &Automerge) -> (Vec<IdeeLue>, usize);
/// Les captations de chaque idée, en un seul passage sur les trois conteneurs.
pub fn captations_par_idee(doc: &Automerge) -> BTreeMap<String, Vec<CaptationLue>>;
/// Les idées que nomme une clé `<empreinte>/idee` du conteneur `photos` (vide au lot 1).
pub fn idees_avec_une_photo(doc: &Automerge) -> BTreeSet<String>;
// lib.rs
/// La boîte : chaque idée lue une fois, ses contenus, sa durée et son état de reliée, puis
/// `noyau::donnees::boite::boite`, puis la fenêtre (`nombre` borné à `FENETRE_MAX`), chaque texte borné à
/// `LONGUEUR_D_UNE_LIGNE` caractères. Sans fonds : aucune ligne, les comptes à 0. En lecture seule comme
/// ailleurs.
pub fn boite(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite>;
/// Les idées retirées, rangées par `noyau::donnees::boite::retirees`, en fenêtre.
pub fn retirees(&self, commande: &Fenetre) -> Reponse<VueDesRetirees>;
/// L'état du fonds : `documents.fonds().is_some()`, `documents.lecture_seule()`,
/// `documents.avertissements()`, et le message de l'opération du dernier geste de tri.
pub fn etat_du_fonds(&self) -> Reponse<EtatDuFonds>;
```

L'adaptateur sur fichiers rend en textes les avertissements de son document (`documents.rs:113`, ceux de la
naissance ou de l'ouverture), chacun par `texte_de` (`adaptateurs.rs:163`), chaque chemin relatif à la racine des
données, jamais par le `Display` d'un `Avertissement`, qui écrirait le chemin absolu ; l'adaptateur en mémoire aucun.

- [ ] **Étape 1 : écrire les tests de `commandes`.** Chaque type nouveau fait l'aller-retour par `serde_json` ;
  `les_noms_sont_uniques_et_en_snake_case` voit dix-sept noms ; `cap01_4_noter_ne_demande_aucun_titre` (une part :
  noter à l'établi) : les clés de `serde_json::to_value(Noter { … })` sont `["decalage_minutes", "texte"]`, rien
  d'autre ; `les_entiers_d_une_reponse_restent_sous_2_puissance_53` gagne une `LigneDeLaBoite` à
  `PREMIER_JANVIER_2100` (`le` et `retiree` à 4 102 444 800, `duree_ms` à 86 400 000), une `VueDeLaBoite` et une
  `VueDesRetirees` dont chaque compte vaut `FENETRE_MAX`, chaque champ lu par `entier_sur`.
- [ ] **Étape 2 : écrire les tests de la suite** (`crates/services/tests/suite/mod.rs`, sur les deux paires, chacun
  écrit dans les deux crates comme ceux de la tranche 2) :
  - `cap02_1_aucun_compteur_d_anciennete` : 50 idées notées à l'heure 1 600 000 000 (en 2020) ; la boîte rend
    `non_reliees == 50`, une assiette `{50, 50, 50}`, et `texte_du_compte` du noyau, « 50 idées non reliées · 50
    interrogées » ; une ligne se défait par un motif exhaustif, `let LigneDeLaBoite { idee, le, texte, etiquettes,
    duree_ms, reliee, provenance, retiree } = ligne;`, si bien qu'un champ d'ancienneté ajouté casse la compilation ;
  - CAP-02 · 3 s'inscrit, par une fonction neuve de la suite, `deux_prises_regroupees_paraissent_ensemble_dans_la_boite`,
    appelée sur les deux paires par `cap02_3_la_boite_montre_ensemble_deux_prises_regroupees` (le test de la tranche 2,
    `deux_prises_regroupees_restent_deux_captations`, reste tel quel) : P et Q, nées du versement de deux prises
    (`fiche_wav(12_000)`, `fiche_wav(47_000)`) ; la boîte rend `non_reliees == 2` et deux lignes ; la prise de Q
    regroupée dans P : les lignes de la boîte sont `[P]`, sa ligne porte `duree_ms == Some(47_000)` (`idee::duree`, la
    plus longue des prises) ; `vue_d_une_idee(&p).captations` égale `prises(&[&p_emp, &q_emp])` ;
    `vue_d_une_idee(&q).retiree` est `Some(HEURE)`, Q vidée et retirée par le même geste ; `non_reliees == 1` ;
    `captations().relire` rend les octets de chaque prise (LE-6, règle 4) ;
  - `cap02_5_retirer_ote_l_idee_de_la_boite_et_retablir_l_y_rend` (une part : les requêtes viennent en tranche 4,
    les photos au lot 2) : X porte un texte, une étiquette, une prise et un lien vers Y, seule autre idée ; avant :
    deux lignes, X et Y, `non_reliees == 0`, l'assiette du compte `{2, 2, 2}` ; X retirée : les lignes sont `[Y]`,
    `non_reliees == 1` (Y ne relie plus rien de non retiré), l'assiette `{1, 1, 1}`, et `retirees` rend `[X]`,
    `total == 1` ; X rétablie : sa ligne égale celle d'avant, `non_reliees == 0`, l'assiette `{2, 2, 2}` (règle 5) ;
  - `cap07_4_le_nombre_d_idees_non_reliees_suit_chaque_geste` : A et B, 2 ; reliées, 0 ; C notée, 1 ; B retirée,
    2 (A redevient non reliée, B sort) ; B rétablie, 1 ; A et B déliées, 3 ; A reliée à B et à C, 0 ; B retirée,
    0 (A reste reliée à C) ; et la lecture de la boîte n'écrit rien : le journal a le même nombre d'entrées avant et
    après (« sans qu'aucun compteur ne soit écrit » ; règle 1) ;
  - `i31_un_texte_piege_revient_tel_quel` : `<script>alert(1)</script>` et `<img src=x onerror=alert(1)>`, notés ;
    la vue de l'idée et la ligne de la boîte les rendent octet pour octet ;
  - `une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop` : trois idées ; `debut: 5` rend aucune
    ligne, `non_reliees == 3`, `retenues == 3` ; `nombre: 0` aucune ligne ; puis 205 idées et `nombre: 1000` rendent
    200 lignes, `retenues == 205` ;
  - `la_boite_d_un_pupitre_neuf` : sans fonds, aucune ligne, les comptes à 0, l'assiette `{0, 0, 0}`,
    `etat_du_fonds().ne == false` ; une note, puis `ne == true` ;
  - `une_ligne_borne_son_texte` : un texte de 250 `é` ; la ligne en porte 200 caractères, la vue de l'idée 250 ;
  - `l_etat_du_fonds_dit_le_geste_qu_annuler_vise` : A notée, `a_annuler == None` ; une étiquette posée,
    `a_annuler` égale le message de la dernière entrée du journal ; B notée, `a_annuler` inchangé ; une annulation,
    `None` ;
  - `les_retirees_se_rangent_par_la_date_du_retrait` : A retirée à `HEURE + 10`, B à `HEURE + 20` ; `retirees`
    rend B puis A, `total == 2`, et la boîte ne les montre pas.

  Sur la paire de fichiers seule, `l_etat_du_fonds_dit_la_lecture_seule` : un fichier du fonds altéré (par la façon
  des tests de la tranche 1), `lecture_seule` porte le refus, de raison `Raison::Illisible`, qui nomme le fichier
  relatif à la racine des données, comparé à un texte écrit dans le test (`documents/fonds/…`, comme
  `crates/stockage/tests/commandes_sur_fichiers.rs:429`), jamais à un `Path::join` affiché ; et
  `l_etat_du_fonds_dit_l_avertissement_de_la_naissance` : la première note, coupée par `couper(Etape::RendreDurable,
  1)`, comme `la_premiere_note_coupee_apres_le_renommage_fait_naitre_le_fonds_avec_un_avertissement`
  (`commandes_sur_fichiers.rs:400-440`) ; puis `etat_du_fonds().avertissements` égale `["documents/fonds/complet.automerge
  est en place, mais son renommage n'est pas rendu durable."]`, sans le chemin de la racine des données. Sur la paire
  en mémoire seule, `une_idee_illisible_compte_a_part_dans_l_assiette` :
  un document écrit par le test, dont une idée porte `le` en texte, rouvert par `DocumentsEnMemoire::de` ; la boîte
  rend `considerees == portant + 1`, et `texte_du_compte` porte « · 1 illisible ».
- [ ] **Étape 3 : voir le rouge** à « Lint » de `verifier` (`cannot find type VueDeLaBoite`, ou le premier nom
  manquant) ; committer (`test: la boîte, les retirées et l'état du fonds sur les deux paires`).
- [ ] **Étape 4 : écrire le code.** `boite` lit les idées par `idees_et_illisibles`, les captations par
  `captations_par_idee`, les photos par `idees_avec_une_photo` ; l'état de reliée par `idee::reliee`, l'autre bout
  retiré lu dans la même liste (un bout illisible compte non retiré, comme `vue_d_une_idee`) ; les contenus par
  `contenus_de` ; puis le noyau, puis la fenêtre.
- [ ] **Étape 5 : voir le vert.** « Tests » de `verifier` : `serialisation.rs`, `commandes_en_memoire.rs` et
  `commandes_sur_fichiers.rs` `ok`, chaque test nommé relevé ; dans `windows`, `commandes_sur_fichiers.rs` `ok` à
  « Tests du stockage et de la synchronisation (H07) » ; « Tests de l'établi » rouge, attendu, sur
  `i31_les_droits_nomment_chaque_commande_et_rien_d_autre` et `i31_generate_handler_porte_chaque_nom` (les trois noms
  ajoutés à `NOMS`), jusqu'à la tâche 17, qui le remet au vert dans la même PR.
- [ ] **Étape 6 : committer** `git add crates/commandes/src/lib.rs crates/commandes/tests/serialisation.rs
  crates/services/src/contrats.rs crates/services/src/lecture.rs crates/services/src/lib.rs
  crates/services/src/en_memoire.rs crates/stockage/src/adaptateurs.rs crates/services/tests/suite/mod.rs
  crates/services/tests/commandes_en_memoire.rs crates/stockage/tests/commandes_sur_fichiers.rs`
  (`feat: la boîte en fenêtre, les idées retirées et l'état du fonds`).

### Tâche 17 : l'établi sert la boîte ; I29 côté natif

**Fichiers :** modifier `app/src-tauri/src/commandes_de_l_etabli.rs` (`boite`, `retirees`, `etat_du_fonds`),
`app/src-tauri/src/lib.rs` (`gestionnaire`), `app/src-tauri/capabilities/windows.json` (`allow-boite`,
`allow-retirees`, `allow-etat-du-fonds`), `app/src-tauri/src/fil.rs` (le test d'I29), `app/src-tauri/tests/statique.rs`.

**Interfaces — produit :** `boite (commande: Boite) -> VueDeLaBoite`, `retirees (commande: Fenetre) ->
VueDesRetirees`, `etat_du_fonds () -> EtatDuFonds`, comme les commandes de la tâche 4 ; une `Ouverture::Refusee`
les refuse comme les autres, et l'interface y lit la racine tenue (tâche 18).

- [ ] **Étape 1 : écrire les tests.** Dans `fil.rs`, `i29_une_ecriture_refusee_revient_en_refus_et_l_etat_le_dit` :
  une note, `fermer`, le fichier complet du fonds remplacé par `b"pas un document"`, une seconde ouverture ; noter
  rend un refus de raison `Raison::Illisible`, `etat_du_fonds` porte `lecture_seule`, et aucun de leurs messages ne
  contient le nom du dossier jetable (`dossier.file_name()`, `pupitre-etabli-<nom>-<processus>`), qui paraît quelle
  que soit la forme du chemin, `\` ou `/` (LE-6, règle 17). Dans `tests/statique.rs`,
  `i29_ni_greffon_ni_api_de_notification_cote_natif` : ni `notification` (sans égard à la casse) dans
  `app/src-tauri/Cargo.toml` ni dans `app/src-tauri/src/`, ni `.message(` ni `MessageDialog` dans
  `app/src-tauri/src/` (question 13 ; la boîte d'enregistrement, `.file()`, reste). Comme celui d'I30, il ignore les
  lignes qui commencent par `//`, blancs ôtés, et, dans `Cargo.toml`, celles qui commencent par `#`. Sa plantation :
  un `tauri-plugin-notification = "2"`, un `app.dialog().message("…")`, un `MessageDialogBuilder`, refusés ; une ligne
  `/// aucune notification (I29)`, sans faute. Les tests
  d'I31 (tâche 11) rougissent d'eux-mêmes tant que les trois noms de `NOMS` n'ont ni commande ni droit.
- [ ] **Étape 2 : voir le rouge** à « Tests de l'établi » de `windows` : `i31_les_droits_nomment_chaque_commande_et_rien_d_autre`
  et `i31_generate_handler_porte_chaque_nom` (les trois noms de la tâche 16), puis le test d'I29 natif ; committer
  (`test: I29 côté natif, et les lectures de la boîte attendues par I31`).
- [ ] **Étape 3 : écrire le code** et les droits.
- [ ] **Étape 4 : voir le vert.** « Tests de l'établi » et « La sentinelle d'I30 » verts, `i31.rs` et `statique.rs`
  compris.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/commandes_de_l_etabli.rs app/src-tauri/src/lib.rs
  app/src-tauri/capabilities/windows.json app/src-tauri/src/fil.rs app/src-tauri/tests/statique.rs` (`feat: l'établi
  sert la boîte, les retirées et l'état du fonds ; I29 côté natif`).

### Tâche 18 : l'interface : le pont natif, le bandeau, la page d'ouverture ; I29

**Fichiers :** créer `app/ui/src/natif.rs`, `app/ui/src/bandeau.rs` ; modifier `app/ui/Cargo.toml` (`serde =
"1.0.229"`, `serde_json = "1.0.154"`), `app/ui/src/main.rs`, `app/ui/src/app.rs`, `app/ui/src/export.rs`
(`EXPORTER_CAHIER` au lieu du nom écrit), `app/ui/style.css`, `app/ui/tests/statique.rs`, `Cargo.lock`.

**Interfaces — produit :**

```rust
// app/ui/src/natif.rs
/// Invoque la commande `nom` (une constante de `commandes`), avec `{"commande": …}` ou `{}`, par
/// `serde_json` puis `JSON.parse` (décision 11) ; la réponse se relit par `JSON.stringify` puis
/// `serde_json`. Un rejet : `refus_du_rejet`. Hors de l'application : un refus, « Hors de l'application,
/// aucune commande ne passe. ».
pub(crate) async fn invoquer<R: serde::de::DeserializeOwned>(nom: &str, commande: Option<serde_json::Value>) -> Result<R, Refus>;
/// Le refus que dit un rejet, lu en JSON : un `Refus`, tel quel ; tout autre rejet, ou aucun, le constat
/// `Refus { raison: Raison::Impossible, message: "Le geste n'a pas abouti." }` (question 31).
pub(crate) fn refus_du_rejet(rejet_en_json: Option<String>) -> Refus;
/// Le décalage local du geste, en minutes : `decalage_depuis_js` de `Date::new_0().get_timezone_offset()`.
pub(crate) fn decalage_du_geste() -> i32;
/// Le décalage local de l'instant `le` (en secondes), rendu en minutes, celles qu'attend `date_de_la_liste` :
/// `decalage_depuis_js` de `Date::new(&JsValue::from_f64(le as f64 * 1000.0)).get_timezone_offset()`
/// (question 24).
pub(crate) fn decalage_a(le: i64) -> i32;
/// Vrai dans l'application : `window.__TAURI__.core.invoke` existe, comme le lit `invoke_de_tauri`
/// (`export.rs`) ; `invoquer` rend hors de l'application le refus dit plus haut.
pub(crate) fn dans_l_application() -> bool;
/// `getTimezoneOffset` vaut UTC moins l'heure locale : `decalage_minutes` en est l'opposé (LE-6, règle 20).
pub(crate) fn decalage_depuis_js(minutes_de_javascript: f64) -> i32;

// app/ui/src/bandeau.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Source { Etat, Refus, Avertissement }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MessageDuBandeau { pub source: Source, pub texte: String, pub fermable: bool }
/// Le bandeau (I29 ; question 31) : un état (lecture seule, version de schéma plus récente), qui ne se ferme
/// pas ; un refus, et des avertissements, qui restent jusqu'au geste réussi suivant ou jusqu'à « Fermer ».
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Bandeau { /* etat, refus, avertissements */ }
impl Bandeau {
    /// L'état du fonds, relu à l'ouverture et après chaque commande : sa lecture seule devient l'état ; à
    /// l'ouverture, ses avertissements s'ajoutent.
    pub fn suivre(&mut self, etat: &EtatDuFonds, a_l_ouverture: bool);
    /// L'issue d'un geste : réussi, le refus et les avertissements d'avant s'effacent, puis le sien s'ajoute ;
    /// refusé, son message devient le refus.
    pub fn apres_un_geste(&mut self, issue: &Result<Option<String>, Refus>);
    /// Ferme le refus ou les avertissements ; l'état ne se ferme pas.
    pub fn fermer(&mut self, source: Source);
    /// Les messages, l'état d'abord, puis le refus, puis les avertissements.
    pub fn messages(&self) -> Vec<MessageDuBandeau>;
}
/// Ce que l'établi ouvre : tout, ou, dans l'application, devant un refus de l'état du fonds (une racine tenue,
/// une racine que le disque refuse), la page du seul bandeau (question 4). Hors de l'application (l'interface
/// servie seule, dans un navigateur), tout s'ouvre, comme la table d'INC-0 aujourd'hui, et la boîte dit son
/// refus en bandeau.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Page { Etabli, BandeauSeul(String) }
pub(crate) fn page_d_ouverture(etat: &Result<EtatDuFonds, Refus>, dans_l_application: bool) -> Page;
```

Le bandeau se rend en tête de l'établi : chaque message en texte, `role="alert"` pour un refus, `role="status"`
pour un état ou un avertissement, et un bouton « Fermer » s'il est fermable ; la source se dit aussi par un mot en
tête (« Lecture seule : », « Refusé : », « Avertissement : »), jamais par la seule couleur. `App` lit
`etat_du_fonds` au montage, puis `page_d_ouverture(&etat, natif::dans_l_application())` : `Page::BandeauSeul` rend la
seule page du bandeau. `export.rs` garde ses trois arguments pour `exporter_cahier`, nommé par `EXPORTER_CAHIER`.

- [ ] **Étape 1 : écrire les tests** (dans `bandeau.rs` et `natif.rs`, `#[cfg(test)]`, en natif, comme ceux d'`Etat`) :
  - `i29_une_ecriture_refusee_s_affiche_en_bandeau` : `apres_un_geste(&Err(Refus { raison: Raison::Disque, message:
    "<un texte de refus du disque>" }))` ; `messages()` rend ce seul message, `Source::Refus`, fermable ;
  - `un_refus_reste_jusqu_au_geste_reussi_suivant` : le refus, puis `Ok(None)`, aucun message ;
  - `un_etat_ne_se_ferme_pas` : `suivre` d'un état en lecture seule, puis `fermer(Source::Etat)` ; le message reste,
    non fermable ; de même pour un `EtatDuFonds` dont `lecture_seule` porte un `Refus` de raison
    `Raison::VersionPlusRecente { version: 4 }` : `messages()` le rend en `Source::Etat`, non fermable, son texte
    celui du refus ;
  - `les_etats_passent_avant_les_refus` : un état, un refus, un avertissement : `[Etat, Refus, Avertissement]` ;
  - `un_rejet_qui_n_est_pas_un_refus_se_dit_par_un_constat` : `refus_du_rejet(Some("\"Command noter not allowed by
    ACL\"".into()))` et `refus_du_rejet(None)` rendent « Le geste n'a pas abouti. » ; le JSON d'un `Refus` rend ce
    `Refus` ;
  - `le_decalage_de_javascript_change_de_signe` : `decalage_depuis_js(-60.0) == 60`, `(-120.0) == 120`,
    `(300.0) == -300`, `(0.0) == 0` ;
  - `la_page_d_ouverture_d_une_racine_tenue_n_a_que_le_bandeau` : `page_d_ouverture(&Err(Refus { raison:
    Raison::Impossible, message: "Un autre processus tient la racine des données.".into() }), true)` rend
    `Page::BandeauSeul` de ce texte ; un état `Ok`, `Page::Etabli` ; hors de l'application, le refus « Hors de
    l'application, aucune commande ne passe. » avec `false`, `Page::Etabli`.

  Dans `app/ui/tests/statique.rs` : la règle de l'horloge admet `Date::new_0` dans `date_du_jour` et
  `decalage_du_geste`, et `get_timezone_offset`, `getTimezoneOffset` dans `decalage_du_geste` et `decalage_a` ; une
  règle d'I29 refuse `Notification` et `notification` dans `app/ui/src/`, `index.html` et `app/ui/Cargo.toml` ; les
  plantations : `getTimezoneOffset` dans une fonction `autre`, refusée, et `web_sys::Notification`, refusée ; le test
  se nomme `i29_ni_greffon_ni_api_de_notification_dans_l_interface`.
- [ ] **Étape 2 : voir le rouge** à « Lint de l'interface (natif, tests compris) » (`unresolved module natif`, ou le
  premier nom manquant) ; le `Cargo.lock` en retard, par l'artefact ; committer (`test: le bandeau, le pont natif et
  I29 dans l'interface`).
- [ ] **Étape 3 : écrire le code.**
- [ ] **Étape 4 : voir le vert.** « Tests » de `verifier` : l'interface `ok`, ses sept tests nommés, `statique.rs`
  et `encre_statique.rs` compris ; « Lint de l'interface (WebAssembly) » vert ; « Le graphe de l'interface » vert
  (`serde_json` n'y tire ni `automerge`, ni les services, ni le stockage).
- [ ] **Étape 5 : committer** `git add app/ui/Cargo.toml app/ui/src/natif.rs app/ui/src/bandeau.rs app/ui/src/main.rs
  app/ui/src/app.rs app/ui/src/export.rs app/ui/style.css app/ui/tests/statique.rs Cargo.lock` (`feat: le bandeau, le
  pont des commandes et la page d'ouverture de l'interface`).

### Tâche 19 : la boîte à l'écran

**Fichiers :** créer `app/ui/src/boite.rs` ; modifier `app/ui/src/main.rs`, `app/ui/src/app.rs` (`Etat` gagne
`vue: RwSignal<Vue>`, `Vue { Table, Boite }`, la table par défaut), `app/ui/src/entete.rs` (le sélecteur),
`app/ui/style.css`.

**Interfaces — produit** (`app/ui/src/boite.rs`) :

```rust
/// La marge de la fenêtre, en lignes, de chaque côté de ce qui se voit.
pub(crate) const MARGE: u32 = 5;
/// La fenêtre à demander : la première ligne vue, `floor(defilement / hauteur_ligne)`, moins `MARGE` ; les
/// lignes vues, `ceil(hauteur_vue / hauteur_ligne)`, plus `MARGE` après ; bornée par `total` et par
/// `FENETRE_MAX`.
pub(crate) fn fenetre(defilement_px: f64, hauteur_vue_px: f64, hauteur_ligne_px: f64, total: u64) -> Fenetre;
/// « Annuler : <message> », ou « Rien à annuler » (question 30).
pub(crate) fn libelle_d_annulation(a_annuler: Option<&str>) -> String;
/// `Ctrl+Z` annule un geste de tri hors d'un champ de texte seulement.
pub(crate) fn ctrl_z_annule(touche: &str, ctrl: bool, dans_un_champ: bool) -> bool;
/// Noter en un geste (CAP-01 · 4) : `Entrée` dans le champ, un texte non vide une fois ses blancs ôtés.
pub(crate) fn note_a_envoyer(champ: &str, touche: &str, decalage_minutes: i32) -> Option<Noter>;
/// Les cases d'une ligne : le texte, ou `—` ; les étiquettes, `#a #b`, ou `—` ; « reliée » ou « non reliée » ;
/// « captée » ou « importée » (question 25).
pub(crate) fn texte_de_la_ligne(ligne: &LigneDeLaBoite) -> String;
pub(crate) fn etiquettes_de_la_ligne(ligne: &LigneDeLaBoite) -> String;
pub(crate) fn etat_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str;
pub(crate) fn provenance_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str;
```

**L'écran** (questions 27, 28, 30) : en tête, le champ « Noter une idée », toujours présent, même dans un Pupitre
neuf ; dessous, le compte (`texte_du_compte`), et, sous un filtre, son nombre (`texte_du_filtre`) ; « Ranger par : »
date, étiquette, durée, reliée, provenance ; « Contenu : » tout, texte de l'idée, prise, photo, MIDI versé, texte
versé ; la liste, des lignes de hauteur fixe (`2.5rem`), chacune : la date (`date_de_la_liste(le, decalage_a(le))`),
le texte coupé sur une ligne, les étiquettes, la durée (`duree_affichee`), l'état, la provenance, et une case à cocher
pour relier ; « Relier », actif quand deux lignes sont cochées ; le bouton d'annulation (`libelle_d_annulation` de
`etat_du_fonds`) ; « Idées retirées », qui ouvre la liste des retirées, en fenêtre, chacune avec « Rétablir ». Une
ligne choisie ouvre la vue de l'idée : le texte entier, « Compléter », « Poser l'étiquette » (par
`etiquette_saisie`), « Ôter » pour chaque étiquette, « Délier » pour chaque lien, « Retirer ». Après chaque commande,
l'interface relit `etat_du_fonds` et la fenêtre de la boîte, et le bandeau reçoit l'issue. La liste suit le seul
événement de défilement (I18). Chaque contrôle est un bouton, une case ou un champ, focus visible ; les tailles en
`rem`. La table d'INC-0 reste l'autre place du sélecteur.

- [ ] **Étape 1 : écrire les tests** (dans `boite.rs`, `#[cfg(test)]`, en natif) :
  - `la_fenetre_borne_les_lignes_rendues` : `total` 2 000, des lignes de 40 px, une vue de 800 px (vingt lignes) ;
    à 0 px, début 0, 25 lignes ; à 40 000 px, début 995, 30 lignes ; à 79 200 px, début 1 975, 25 lignes ; à
    200 000 px, 0 ligne ; jamais plus de 30 (vingt plus deux marges) ;
  - `ctrl_z_n_annule_un_geste_que_hors_d_un_champ` : `("z", true, false)` et `("Z", true, false)` annulent ;
    `("z", true, true)`, `("z", false, false)` et `("y", true, false)` n'annulent pas ;
  - `le_bouton_d_annulation_dit_le_geste_vise` : `libelle_d_annulation(Some("Idée du 3 novembre : étiquette #pont
    posée"))` rend « Annuler : Idée du 3 novembre : étiquette #pont posée » ; `None`, « Rien à annuler » ;
  - `cap01_4_noter_se_fait_en_un_geste_sans_titre` (une part : la capture du carnet vient au lot 2) :
    `note_a_envoyer("Une idée de pont", "Enter", 60)` rend `Some(Noter { texte: "Une idée de pont", decalage_minutes:
    60 })` ; une autre touche, ou un champ de blancs, `None` ;
  - `un_texte_piege_se_montre_tel_quel` : `texte_de_la_ligne` d'une ligne au texte `<img src=x onerror=alert(1)>`
    rend ce texte, octet pour octet ; le contrôle statique (tâche 12) garantit qu'il se pose en texte ;
  - `une_case_vide_s_ecrit_d_un_tiret_cadratin` : texte absent et étiquettes vides, `—` ; et `#pont #refrain` pour
    deux étiquettes.
- [ ] **Étape 2 : voir le rouge** à « Lint de l'interface (natif, tests compris) » (`unresolved module boite`) ;
  committer (`test: la fenêtre, l'annulation et les cases de la boîte`).
- [ ] **Étape 3 : écrire le code** de l'écran.
- [ ] **Étape 4 : voir le vert.** « Tests » de `verifier` : l'interface `ok`, les six tests nommés, `encre_statique.rs`
  (I17, I18) et `statique.rs` verts ; « Construire l'installateur » vert dans `windows`.
- [ ] **Étape 5 : committer** `git add app/ui/src/boite.rs app/ui/src/main.rs app/ui/src/app.rs app/ui/src/entete.rs
  app/ui/style.css` (`feat: la boîte d'idées à l'écran, ses ordres, son filtre et ses gestes`).

### Tâche 20 : les documents de la PR (d), la mutation, les délais, et la revue de la PR (d)

**Fichiers :** `docs/80-glossaire.md`, `docs/20-sfd/21-capter-et-organiser.md`, `docs/50-interfaces-et-appareils.md`,
`docs/41-pile-technique.md`, `docs/70-qualite-et-verification.md` (§ 4.1, § 8), `docs/pistes.md`,
`docs/registre.md` (LE-15), `CONVENTIONS.md`, `CLAUDE.md`, `ci.yml` (les délais), `mutants-complet.yml` (son délai).

- [ ] **Étape 1 : la recherche de LE-2**, citée dans la description de la PR, pour chaque mot ajouté ou requalifié :
  `tri`, `trier`, `ordre`, `ordonner`, `ranger`, `range`, `rangée`, `bandeau`, `droit`, `reliée`, `relier`, `délier`,
  `interrogée`, chacune aussi sans accent et sur la majuscule accentuée ; chaque emploi antérieur se requalifie.
  L'entrée É5 **Ordre** (une suite candidate des morceaux d'un album, `docs/80-glossaire.md:85`) et l'entrée
  **Rangée** (une ligne de la table, `:215`) se distinguent nommément de « Ordre (de la boîte) » et de « ranger ».
- [ ] **Étape 2 : le glossaire** (questions 14, 17, 31, 32) : « Boîte d'idées » : « L'endroit où arrivent les idées,
  captées ou importées, et où elles restent, triées ou non ; seule une idée retirée la quitte » ; « Geste de tri »
  (compléter, étiqueter, relier, délier, regrouper, retirer, rétablir : il écrit, le journal le nomme, annuler le
  vise) ; « Ordre (de la boîte) » (date, étiquette, durée, reliée, provenance : il n'écrit rien et ne s'annule pas) ;
  « Bandeau » (un message d'état ou d'échec, dans l'application, I29 ; la zone d'écran du § 50 · 3.1 se renomme en
  tranche 4) ; « Droits (de Tauri) » (un fichier de droits, la *capability* de Tauri ; un droit, sa *permission*,
  `allow-<nom>` ; renvoi au statut d'une valeur, l'autre sens) ; « Reliée (idée) » (§ 30 · 9.1) ; « Relier,
  délier » ; « Interrogées (idées) » (les idées qu'un compte considère : les non retirées, illisibles comprises).
- [ ] **Étape 3 : la SFD et les autres documents.**
  - CAP-02 (`:128`) : « La boîte se range par date, étiquette, durée, état (reliée ou non) et provenance (captée ou
    importée) ; par défaut, par date, la plus récente d'abord. » ;
  - CAP-07 (`:445`) : « Pour les morceaux, ordre libre ; par défaut, la dernière modification. » ;
  - § 50 · 5 (`:277-278`) : l'exemple de CAP-06, « 3 morceaux en mi mineur · 6 dont la tonalité n'est pas décidée
    · 41 interrogés » ;
  - § 41 · 6 (`:288`) : « L'interface appelle les commandes du côté natif (`invoke` de Tauri), et lit la date du
    jour, celle de l'export du cahier, et le décalage local d'un geste et d'une date » ; la ligne de `serde_json`
    nomme l'interface, si elle ne le fait ;
  - § 70 · 4.1 : la ligne CAP-02 devient « 1, 3, 7, 8 » ; une ligne CAP-07, « 4 », livrée par INC-1, lot 1 ;
  - § 70 · 8 : I29 a ses tests ;
  - `docs/pistes.md` : l'écart d'un jour au changement d'heure (L11) ; le texte d'une erreur d'export, que l'export
    garde (C5) ;
  - `CONVENTIONS.md` : `verifier` (le contrôle de l'interface, I29), `windows` (I29 côté natif), `mutants`
    (`donnees/boite.rs`) ; `CLAUDE.md` : « Les crates » (`commandes`, ses constantes de noms ; l'interface, la boîte et
    le bandeau).
- [ ] **Étape 4 : LE-15 se clôt** au registre, son critère de sortie tenu (chaque section en cause corrigée).
- [ ] **Étape 5 : les délais** de `verifier`, `windows` et `mutants` (et de `mutants-complet.yml`), comme à la tâche 7.
- [ ] **Étape 6 : committer** les documents (`docs: la boîte au glossaire, à la SFD et aux portes ; LE-15 clos`).
- [ ] **Étape 7 : la revue de la PR (d).** La revue de la PR entière, sur sa tête, par le modèle le plus capable : le
  tableau du § 70 · 4.1 contre les critères de la tranche, le job de chaque test nommé, la recherche de LE-2, chaque
  règle de LE-6 recopiée. Puis les corrections, leur relecture sur la tête qui part. Le corps dit l'écart provisoire
  à CAP-07 (aucun état de synchronisation avant la tranche 4), comme une limite de la tranche, CAP-07 n'étant pas
  livré.

### Tâche 21 : l'essai installé, son relevé et AM-3

- [ ] **Étape 1 : préparer l'essai.** L'installateur `etabli-installateur` de l'exécution verte de la tête relue
  (tâche 20) ; la liste de l'essai, dans le dossier temporaire, à remettre à la maîtrise d'ouvrage.
- [ ] **Étape 2 (👤) : l'essai.** La maîtrise d'ouvrage installe l'établi sous `app.pupitre.etabli`, sur un fonds
  d'essai (un `donnees/` neuf, des textes inventés), puis, chaque point noté « vu » ou « écart » :
  - la table, la palette et la fiche s'ouvrent, sans fenêtre blanche ;
  - trois idées notées d'un geste, sans demande de titre ; le compte, puis chaque ordre, puis chaque contenu
    (« photo » dit « 0 idée · 3 interrogées ») ;
  - une étiquette posée, puis ôtée ; deux idées reliées, puis déliées ; une idée retirée, retrouvée dans « Idées
    retirées », rétablie ;
  - « Annuler » par le bouton, puis par `Ctrl+Z` hors du champ, puis « Rien à annuler » ;
  - les deux textes piégés d'I31, notés, s'affichent tels quels ;
  - l'export du cahier ;
  - un second lancement montre la seule page du bandeau « Un autre processus tient la racine des données. ».

  Une fenêtre blanche ou un écart se corrige dans la PR (d), revu, puis l'essai reprend.
- [ ] **Étape 3 : le relevé.** Au § 70 · 7, un paragraphe : la date, le SHA de la tête essayée, chaque point et ce
  qu'il a montré, sans donnée personnelle ni chemin de la machine ; et que le fonds d'essai se jette si la genèse
  change avant l'essai suivant (question 34). Au registre, AM-3 se clôt, avec la date. Committer (`docs: le relevé de
  l'essai installé de la tranche 3 ; AM-3 clos`).
- [ ] **Étape 4 : la relecture du relevé**, sur la tête qui part ; le corps final (« Écarts à la spec : Aucun. ») ;
  `gh pr ready` ; la fusion par squash, `--match-head-commit` nommant cette tête.

### Tâche 22 : la clôture de la tranche

- [ ] Le clone principal à jour (`git fetch`, `main..origin/main` vide) ; les worktrees de la tranche retirés ; l'espace
  de travail SDD de la tranche supprimé ; la mémoire du lot mise à jour (la tranche 3 close, ses PR, ce qu'elle laisse
  aux suivantes). Ce plan reste dans `.claude/plans/` jusqu'à la clôture du lot, que la tranche 9 conduit.

---

## Auto-relecture

- **La couverture.** Chaque puce de « Elle livre » du plan maître a sa tâche : les styles (1) ; l'assemblage (2 à 4) ;
  I30 (5, 6) ; la boîte et ses commandes (14 à 17, 19) ; noter, annuler, le bandeau, I29, le `—` (15, 17 à 19), une
  version de schéma plus récente au bandeau (`un_etat_ne_se_ferme_pas`, 18) ; aucune idée « à envoyer » :
  `LigneDeLaBoite` n'a pas d'état d'envoi, ce que garde le motif exhaustif de `cap02_1_aucun_compteur_d_anciennete`
  (16), et aucune case de la boîte ne le dit (19) ; la CSP, le manifeste, les droits, `exporter_cahier` (8, 9) ; I31
  (9, 11, 12, 16, 19) et l'étape `cargo tree` (6).
  Chaque critère de sortie a son test nommé : CAP-01 · 4 (16, 19, en parts), CAP-01 · 6 (tranche 2, inscrit), CAP-02
  · 1 (16), CAP-02 · 3 (16), CAP-02 · 4 et 6 (tranche 2, en parts), CAP-02 · 5 pour la boîte (16, en part), CAP-02 · 7
  et 8 (tranche 2, inscrits), CAP-07 · 4 (16), I29 (17 dans `windows`, 18 dans `verifier`), I30 (5, 6, dans
  `windows`), I31 (8 dans `documentation`, 9, 11 dans `windows`, 12, 16, 19 dans `verifier`). L'essai 👤 (21).
- **Les types.** `Ouverture`, `ServicesSurFichiers` et `demander` (3) servent aux tâches 4 et 17 ; `gestionnaire` (4)
  aux tâches 4, 10, 11 et 17 ; `boite::Ordre` et `Contenu` du noyau (14) ont leurs doubles dans `commandes` (16), que
  les services convertissent ; `Assiette` du noyau rend `Assiette` de `commandes` ; `LigneDeLaBoite`, `VueDeLaBoite`,
  `EtatDuFonds` (16) servent aux tâches 17 à 19 ; `texte_du_compte` et ses voisins (15) aux tâches 16 et 19 ;
  `dossier_jetable` (2) aux tâches 2 à 4, 6 et 17.
- **Les points de revue** ont chacun leur test : 3 (deux), 16 (deux), 19 (un).
- **Les nombres recopiés** l'ont été du passage de LE-6 : les quatre valeurs de la palette, la CSP de 277 octets,
  les dates (recalculées : 1 793 748 600, le 3 novembre 2026 à 23 h 30 UTC ; 1 798 761 600, le 1er janvier 2027 ;
  1 792 881 000, le 24 octobre 2026 à 22 h 30 UTC), les durées, les ordres du jeu inventé (calculés à la main sur ses
  six idées).
