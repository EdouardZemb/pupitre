# La tranche 5 du lot 1 d'INC-1 : l'import — plan d'implémentation

> **Pour les agents :** exécuter ce plan avec `superpowers:subagent-driven-development`, tâche par tâche ;
> les étapes sont des cases à cocher (`- [ ]`).

Plan maître : `.claude/plans/lot1-le-fonds-sur-l-etabli.md`

**Objectif :** Sacha verse son fonds existant, prises, jeux de stems, MIDI et textes, avec l'origine déclarée qu'il
donne, lit le rapport de chaque import, et réécoute une prise, stem par stem pour une prise en stems.

**Architecture :** quatre PR (question 34). (a) pose au noyau, sous mutation, les règles de l'import : la table des
signatures et des extensions, les en-têtes d'un WAV et d'un MIDI, le texte lisible, l'empreinte par tronçons, le
manifeste et le rapport à l'octet, les comptes, la place libre, l'origine la plus forte, le premier nom vu, la portée
d'une déclaration, le type proposé d'un stem, le MIDI d'un stem, le jeu proposé, la durée d'une prise en stems, les
opérations nommées de l'import et la règle des empreintes que nomme le fonds, étendue. (b) bâtit l'import : un
troisième contrat, la source à verser, et la copie en flux des captations, au stockage et en mémoire ; la lecture du
fonds importé ; les commandes et les services de l'import, conduit unité par unité ; les déclarations après l'import ;
la reprise ; l'établi, qui choisit le dossier, lit la place libre dans `systeme` et sert les commandes ; le test 13
entier. (c) scinde `app/ui/src/boite.rs` en un module `boite/`, sans rien changer à ce qu'il fait. (d) sert le
protocole des captations, puis l'écran : l'import, ses déclarations et son rapport, la vue d'une idée importée et sa
lecture simple, la ligne d'une idée sans texte, le regroupement, les titres des colonnes à 200 % ; puis l'essai
installé.

**Pile :** Rust 1.99.1, édition 2024 ; `tauri` 2.12.5 et `tauri-plugin-dialog` 2.8.4 ; Leptos 0.8.23 ; `automerge`
0.12.0 ; `sha2` (déjà au noyau) ; aucune dépendance nouvelle ; tests : `proptest` 1.12.0 ; en CI, `cargo-mutants`
27.1.0, trunk 0.22.1, tauri-cli 2.12.4.

**Spécification :** le plan maître, section « Tranche 5 · l'import », ses décisions 9, 11, 12, 19 et 20 et ses points
de vigilance 1, 3 et 4 ; CAP-02, CAP-03, CAP-06, CAP-07 ; SYN-07 ; l'ADR-024 (et les ADR-022, ADR-023) ; § 30 · 2, 8.1
et 9.2 ; § 31 · 2, 2.1, 2.3, 2.4, 6.1 et 10 (tests 4, 11 et 13) ; § 40 · 3 et 7.5 ; § 41 · 6 ; § 50 · 2, 3.1, 4.1, 4.2
et 7 ; § 70 · 1, 3 (I01, I14, I29, I30, I31), 4.1, 7 et 8 ; le glossaire ; les questions tranchées le 2026-11-21,
recopiées plus bas ; et les plans des tranches 2 à 4, pour les noms qu'ils ont posés.

Refs #24

---

## Contraintes générales

Recopiées du plan maître (« Contraintes générales », « Le déroulé »), comme l'ont fait les tranches 1 à 4 ; chaque
tâche les suppose.

- « **En CI seulement.** Ni cargo, ni trunk, ni npm sur le poste : chaque vérification de Rust se lit dans le job que nomme le critère, le rouge puis le vert dans `verifier`, le vert aussi dans `windows` si le critère l'y met. » Les tests d'`etabli` ne tournent que dans `windows` ; ceux du noyau, de `commandes`, des services et de l'interface, dans `verifier` ; ceux du stockage, dont la suite des commandes sur fichiers (`crates/stockage/tests/commandes_sur_fichiers.rs`), dans les deux (`ci.yml:207`). Un critère « dans `verifier` et `windows` » a donc son test dans la suite des commandes ou au stockage, jamais sous le seul `#[cfg(windows)]`.
- « Seul `rustfmt --edition 2024` formate les `.rs` touchés ; les vérifications de documentation tournent sur le poste, après le `git add` (LE-7). » `rustfmt` se lance hors du dépôt, sur une copie dans le dossier temporaire, recopiée ensuite.
- « Les commandes de Rust prennent `-p <crate>`, et `ci.yml` nomme chaque crate qu'il passe à `clippy` et aux tests. »
- « **Le noyau** reste sans entrée-sortie, ni `automerge`, ni horloge, ni aléa ; chaque règle qui décide y vit, sous mutation (décision 4). » Les signatures, les en-têtes, le texte lisible, le manifeste, le rapport, les comptes, la place libre, l'origine, le premier nom vu, la portée, le type proposé, le jeu proposé, les opérations de l'import et les règles du protocole y vivent ; `services` lit le document et les contrats, et ne décide rien.
- « **Les chemins se reçoivent** (I30) : hors de `systeme`, aucun code ne devine un chemin ; chaque test prend son dossier jetable par une seule fonction de sa crate. » Le dossier à verser se reçoit de la boîte de dialogue, côté natif ; la place libre ne se lit que dans `systeme`.
- « **Aucune donnée réelle** : textes et noms inventés ; identifiants, documents et captations engendrés à l'exécution ; aucun audio, MIDI ni `.automerge` suivi. » Les WAV, MIDI, M4A, zip et PDF des tests sont des octets que le test écrit, jamais un fichier suivi ; le dossier de l'essai installé s'engendre par un programme d'exemple du dépôt (question 7).
- « **Les mots du glossaire**, dits en entier. » Les mots que la tranche fait entrer ou requalifie : illisible (fichier versé), verser, rassembler, nom provisoire, voisin, proposition, rapport d'import, manifeste, prise en stems, origine déclarée, source à verser (questions 10, 18, 31, 33).
- « Une PR qui ajoute un mot au glossaire le cherche d'abord (`git grep -n -i -w <mot> -- docs README.md CONVENTIONS.md`), requalifie chaque emploi antérieur et cite la recherche (LE-2) ; une PR qui périme un mot ou un identifiant le cherche dans tout le dépôt, maquettes et commentaires SVG compris (LE-3). » La recherche se refait aussi sans accent et sur la majuscule accentuée (`-i` ne replie pas les accents).
- « **Chaque écran** qu'une tranche ajoute se mène au clavier, focus visible, avec ses libellés, au vocabulaire du glossaire, sert à 200 % (§ 50 · 7), et ses feuilles passent I17 et I18 (`encre_statique.rs`). » L'écran de l'import, ses déclarations, son rapport, la vue d'une idée importée, la lecture simple et le regroupement sont les écrans de cette tranche ; I18 refuse `set_timeout`, `set_interval` et `request_animation_frame`, jusque dans les commentaires : l'avancée de l'import se lit dans les réponses, sans minuterie (question 1).
- « Conventional Commits en français, en-tête de 100 caractères au plus, titre de 93 au plus ; « Refs #24 » » ; `git add` par chemins explicites, jamais `git add .` ; `git commit -F <scratchpad>/msg-t5-<tâche>-<étape>.txt` (un texte accentué passe par un fichier).
- « Chaque PR qui livre un critère de la SFD en entier l'inscrit au tableau du § 70 · 4.1, et un test le nomme (`cap06_7_…`) et en calcule l'exemple ; `verif-faits.mjs` refuse un critère inscrit sans test qui le nomme. La porte ne voit pas un critère livré que la PR n'inscrit pas : la revue de chaque PR compare le tableau aux critères que la tranche dit livrer. » La PR (b) inscrit CAP-03 · 1 à 3, 7, 9 à 11 et 13, et dit le 8 livré par part (`jeu_propose` de l'inspection, sans écran) ; la PR (d) y ajoute le 8, que l'écran propose, et le 12 (R24) ; CAP-03 · 4 reçoit sa part « sans morceaux », nommée, sans s'inscrire (la règle des parts, § 70 · 4.1 ; P15).
- « fusion par squash, une fois tout vert » ; mode `session` : la PR s'est ouverte en brouillon, donc `gh pr ready <n>`, puis, depuis le clone principal `pupitre`, sans `cd` ni `-R`, `gh pr merge <n> --squash --match-head-commit <SHA complet de la tête relue>`.
- « une tranche qui alourdit les tests remesure les délais de `verifier` et de `windows` ; l'installateur NSIS reste en artefact. »
- Le corps de chaque PR porte la section `## Écarts à la spec`, avec « Aucun. » : un écart à la spec se lève en corrigeant la spec dans la PR même, jamais en le déclarant. Le corps final s'écrit avant l'envoi qui le rend final (une retouche du corps ne relance que `texte-pr`, et `texte-pr` ne se relance jamais par `gh run rerun`).
- Chaque PR part de `main` à jour, depuis le clone principal (`git fetch`, puis `git worktree add ../pupitre.worktrees/<branche> -b <branche> origin/main`), s'ouvre en brouillon au premier envoi (`ci.yml` ne tourne que sur une PR), et dit dans sa description les tests réécrits et pourquoi.
- **Lire la CI** : `gh run list --branch <branche> --workflow ci --limit 1`, `gh run watch <n> --exit-status`, `gh run view <n> --log-failed`. Un rouge attendu se lit tel que la CI le rend : une erreur de compilation qui nomme ce qui manque, à « Lint » de `verifier` (ou à « Lint de l'établi » de `windows`) ; ou un test, à « Tests » (`panicked at`, avec le nom du test). `cargo test` s'arrête au premier fichier de tests rouge, pris dans l'ordre de leurs noms. Un vert se lit à « Tests » de `verifier`, et, pour `etabli` ou `stockage`, à leur étape de `windows` : `test result: ok.` pour chaque fichier touché, relevé par une file.
- Chaque nom de Leptos 0.8.23, de `web-sys`, d'`automerge` 0.12.0 (dont `get_all`, `keys_at`, `get_at`, `map_range_at`, `hash_for_opid`, `save`, `set_actor`), de `tauri` 2.12.5 (dont `register_asynchronous_uri_scheme_protocol`, `UriSchemeContext::app_handle`, `UriSchemeResponder::respond`) et de `tauri-plugin-dialog` 2.8.4 (`blocking_pick_folder`) que le code emploie se vérifie sur docs.rs avant d'écrire ; un nom qui n'existe pas se remplace par son équivalent documenté, et la tâche le dit dans son commit. Un commentaire dans un bloc de test de ce plan est une spécification, qui devient des assertions.
- Chaque test qui prouve un critère de la SFD, inscrit ou livré par parts, se nomme `capNN_n_…`, précédé de `#[test]` ; chaque test d'un invariant se nomme `i01_…`, `i14_…`, `i30_…` ou `i31_…`.
- Les heures sont des secondes, partout où le code les reçoit ; seul `systeme` lit l'horloge, l'aléa et la place libre ; l'interface ne lit le décalage local que par `natif::decalage_du_geste` et `natif::decalage_a`.
- Un numéro de ligne qu'une tâche cite vaut sur `main` à `fe918aa` : une tâche antérieure du plan peut le décaler, et l'élément nommé (la fonction, la constante, la doc) fait foi ; l'implémenteur le cherche par son nom. Les numéros périmés du plan maître (`documents.rs:944-947` et `:931-932`, `captations.rs:47`) se citent ici à jour : le `fork()` de la compaction est à `crates/stockage/src/documents.rs:951`, dans `charger_contre` (`:950`) ; le versement d'une captation, à `crates/stockage/src/captations.rs:53` (C28).
- Les identifiants d'Automerge, d'ULID et de propriétaire, et les empreintes, ne s'écrivent jamais dans un message montré : un message nomme une captation par son premier nom vu, ou par « cette prise » (question 26) ; le rapport d'import, qui n'est pas un message, nomme une empreinte et un ULID.
- Les contrôles statiques cherchent en sous-chaîne, commentaires compris, sauf ceux d'I29 et d'I30, qui sautent une ligne `//` : le noyau refuse `SystemTime`, `Instant`, `time::`, `random`, `getrandom`, `rand::`, `fs::`, `File`, `std::io`, `io::`, `env::`, `thread::`, `net::`, `process::`, `HashMap`, `HashSet`, `RandomState` (`crates/noyau/tests/statique.rs:22`) ; les services, `std::fs`, `fs::`, `File`, `thread::`, `spawn`, `net::`, `SystemTime`, `Instant`, `time::`, `env::`, `process::`, `HashMap`, `HashSet` (`crates/services/tests/statique.rs:25`), `io::` y restant permis ; l'interface, `animation`, `transition`, `set_timeout`, `set_interval`, `request_animation_frame` ; l'établi (I29), `notification`, `.message(`, `MessageDialog`. Les noms et les docs de la tranche évitent ces mots : « Instantané », « Filet », « Profile », « transition d'un import », « tirage aléatoire », et, au noyau, un type dont le nom finit par `io` suivi de `::` (`FormatAudio::Wav` contient `io::` : le type des formats audio se nomme donc `FormatSonore` ; P4 du balayage préalable) ; aucune méthode de l'établi ne se nomme `message` (P11, C25).
- Les compteurs écrits en dur changent dans la PR qui ajoute ce qu'ils comptent : `NOMS.len()` et `NOMS[15..]` (`crates/commandes/tests/serialisation.rs:388`, `:413`), `Engendre` (`crates/stockage/tests/mesure.rs:44`), le compte des modules de `.cargo/mutants.toml` (`:2`, et `:33`, « ces … modules », une ligne plus bas à chaque glob ajouté), `AU_MOINS_DES_CRATES` du contrôle d'I30 s'il le faut (P13).

---

## Ce que les passages ont levé

Trois passages, faits avant ce plan, ne sont pas dans le dépôt : la carte du code (pièges P1 à P15, 107 citations
vérifiées), le passage de LE-6 (L1 à L24, et le test 13 entier estimé à quelque 2,1 Mo) et celui d'AM-2 (C1 à C28, dont
C1 à C5 bloquants). Leur détail a servi à écrire ce plan ; la description de sa PR porte la table de couverture.
Trente-quatre questions en sont nées, posées en un bloc à la maîtrise d'ouvrage, qui les a tranchées le 2026-11-21,
chacune selon son défaut (« Ok pour les défauts ») ; elles sont recopiées plus bas. La PR de ce plan consigne
(`/consigner`), au motif « Une spec qui se contredit » (huitième constat ; la porte existe depuis LE-11), une ligne,
LE-17, qui nomme les constats de la spec ; la tâche 25 la clôt.

### La carte du code

| Piège | Décision de ce plan | Tâche |
|---|---|---|
| P1. La règle des empreintes ne voit ni stems, ni MIDI de stem, ni rapport | `idee::empreintes_nommees` lit le segment qui suit `stem` à sa place, celui qui suit `midi` dans l'entrée d'un stem, et la valeur de `imports/<i>/rapport` ; seules les empreintes valides comptent ; le cas du test 4 sur un stem | 7, 9, 10 |
| P2. Un import de plusieurs gigaoctets dans un fil et une promesse | L'interface conduit l'import unité par unité, chaque unité une demande courte au fil ; ni signal, ni minuterie ; question 1 | 12, 15, 22 |
| P3. Un Pupitre neuf ne range rien | La première opération de l'import précède toute captation, et fait naître le fonds ; question 15 | 6, 12 |
| P4. I01 refuse toute clé que la tranche écrit | `permis` gagne les clés de la tranche, `interdit` ne coupe plus un nom, une déclaration ni un nom de générateur | 10 |
| P5. Les services ne touchent ni fichier, ni horloge | Un troisième contrat, `Source`, que le stockage réalise sur fichiers et les services en mémoire ; la place libre arrive en argument | 9, 15 |
| P6. Le contrat `Captations` ne prend que des octets | `verser_en_flux` ; la voie par octets passe par lui ; les quatre sites changent ensemble | 9 |
| P7. `pannes` ne coupe pas entre deux tronçons | `Etape::Troncon` ; une lecture en flux qui passe par `Relecture` et s'altère | 9 |
| P8. Le fonds de mesure et le test 13 | Le générateur passe par l'import ; les 100 prises en stems parmi les 6 000 ; date, appareil et diapason absents ; acteurs tournants en mémoire ; `Engendre` récrit | 16 |
| P9. Le gestionnaire du protocole ne voit pas la racine | Asynchrone, chaque requête une demande au fil par `Ouverture::demander` ; `demander_bloquant` reste aux tests | 21 |
| P10. Le premier `unsafe` du dépôt | `GetDiskFreeSpaceExW` sous `#[cfg(windows)]`, dans `systeme` seul, chemin en `\\?\` ; le contrôle d'I30 gagne la place libre et `unsafe`, chacun planté | 15 |
| P11. Les mots que les contrôles lisent | Relus avant chaque doc ; aucune méthode `message` dans l'établi | toutes |
| P12. Les extensions de la porte `contenu` | Ajoutées, `pro` excepté ; les fichiers des tests engendrés à l'exécution | 17 |
| P13. Les compteurs écrits en dur | Changés dans la PR qui ajoute ce qu'ils comptent | 8, 11, 16 |
| P14. Le `fork()` est une mesure | La compaction se mesure dans `crates/stockage/tests/mesure.rs`, qu'I30 ne lit pas | 16 |
| P15. CAP-03 · 4 n'est livré qu'en part | Nommé par `cap03_4_…`, jamais inscrit au § 70 · 4.1 à cette tranche | 12, 17 |

### Le passage de LE-6

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| L1. Le premier nom vu, sans date dans un ensemble à `true` | `<e>/nom/<chemin>` vaut l'ULID de l'import qui l'a vu le premier ; le plus petit couple (heure de l'ULID, octets du chemin) ; question 20 | 4, 6 |
| L2. Les empreintes nommées coupées au premier `/` ; la copie en flux | La règle étendue (P1) ; le nom `import-<ULID>-<rang>` ; question 31 | 6, 7, 9 |
| L3. La taille à verser, et CAP-03 · 13 | Les octets à verser, l'empreinte seulement à taille égale ; question 8 | 3, 12 |
| L4. Le `.m4a` de CAP-03 · 3 sans signature | `ftyp` à l'octet 4 → `m4a` ; question 11 | 1, 12 |
| L5. Les octets du manifeste | Fixés, témoin `340999f9…` ; question 19 | 3 |
| L6. Le rapport sans forme écrite | Fixé, témoin ; question 25 | 3 |
| L7. La « même durée » | Mêmes trames à la même fréquence ; plancher ; question 16 | 2, 5 |
| L8. Une proposition de type pour tout nom | Liste fermée, mot entier, « autre » ; question 18 | 5 |
| L9. Les comptes et la reprise | Comptes contre la tête d'ouverture de l'import ; question 2 | 3, 14 |
| L10. Le rejeu ne vaut qu'à déclarations égales | Seule une déclaration ajoute un rôle ; question 9 | 6, 12 |
| L11. La place libre, sans marge ni unité | Deux fois le fichier complet, 4 096 octets par fichier, Go décimaux ; question 8 | 3 |
| L12. L'origine : forme, portée, noms | Un ensemble, la plus forte, segments entiers ; questions 22, 23 | 4 |
| L13. La ligne du test 13 | Récrite avec les nombres de la tranche 2 ; question 28 | 16, 17 |
| L14. Trois mille réouvertures | Des acteurs tournants en mémoire ; question 29 | 16 |
| L15. « Trop lente » | 2 s pour une compaction de 101 fichiers d'ajout, en `--release` ; question 30 | 16 |
| L16. Le protocole sans nombres | 1 048 576 octets, la table des types, 200, 206, 416 ; question 27 | 20, 21 |
| L17. Le texte versé | Extensions, UTF-8 ou UTF-16 avec BOM ; questions 10, 12 | 1, 2 |
| L18. L'en-tête d'un MIDI | Format au-delà de 2 ou format 0 à plusieurs pistes : illisible ; division brute | 2 |
| L19. Les sous-dossiers d'un jeu, le MIDI d'un stem | Seuls les fichiers du dossier ; même nom à l'extension près ; question 17 | 5, 12 |
| L20. Un nom que Windows seul connaît | Versé sans nom, compté ; 300 unités UTF-16 ; deux formes ; question 13 | 3, 15 |
| L21. « Rien n'en est ouvert » | Seize octets au plus pour un fichier ignoré, compté par la source ; question 11 | 1, 12 |
| L22. « Une captation n'est jamais refusée » | La règle d'une capture ; l'import entier refusé avant toute captation ; question 15 | 12, 17 |
| L23. LE-11 (d) et (e) | Les textes de la question 32 | 17 |
| L24. Les clés d'une prise en stems | Comme l'ADR-024 ; question 21 | 6, 10 |

### Le passage d'AM-2

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| C1. L'import tient le fil du fonds | Unité par unité ; le protocole passe entre deux unités ; question 1 | 12, 21, 22 |
| C2. La taille à verser et la copie en flux | Question 8 ; une copie déjà connue retire son nom provisoire | 3, 9 |
| C3. Le premier vu, sans date ; le nom d'un stem | Questions 20 et 21 | 4, 6 |
| C4. « Illisible », trois sens | Le sens de l'établi, au glossaire ; question 10 | 2, 8 |
| C5. Le rôle nouveau, l'import rejoué | Question 9 | 6, 12 |
| C6. Regrouper à l'écran | La tranche l'ajoute ; question 3 | 24 |
| C7. Captations et première opération | Question 15 | 12, 14 |
| C8. Le protocole sert « une empreinte connue » | L'audio d'une prise ou d'un stem seul ; question 27 | 20, 21 |
| C9. L'état d'un import, la reprise | `imports/<i>/ouvert` et `rapport` ; le bandeau ; question 2 | 14 |
| C10. Le test 13 entier | Question 28 | 16 |
| C11. Le nom provisoire d'une copie | Question 31 | 7, 9, 14 |
| C12. Le type d'un stem | Question 18 | 5, 13, 23 |
| C13. La ligne d'une idée sans texte | « Fichier : » ; question 4 | 10, 24 |
| C14. Le rapport à l'octet | Question 25 | 3 |
| C15. Déclarations : opérations ou paramètres | Paramètres pendant l'import, opérations après ; question 22 | 6, 12, 13 |
| C16. La portée par préfixe | Segments entiers, octet pour octet ; question 23 | 4, 13 |
| C17. Les messages de l'import | Question 26 ; la piste se clôt | 6, 17 |
| C18. Un troisième contrat | `Source` ; § 40 · 3 | 9, 17 |
| C19. Les cas limites du parcours | Liens ignorés, dossier de l'application refusé, fichier changé ; question 14 | 9, 14, 15 |
| C20. L'essai installé | Un dossier engendré par un programme d'exemple ; question 7 | 26 |
| C21. Les pistes de la tranche 5 | Chacune tranchée (questions 5, 6, 24, 26) | 17, 19, 25 |
| C22. LE-11 (d) et (e) | Question 32 | 17 |
| C23. Les critères de sortie | Un test nommé chacun ; question 7 pour le 12 | 12, 13, 23 |
| C24. I30 et la place libre | Question des choix du plan ; § 70 · 3 | 15 |
| C25. Les autres portes | I29, I31, `contenu` | 15, 17 |
| C26. Les documents hors de la ligne « Documents » | La ligne de chaque PR les nomme | 8, 17, 25 |
| C27. « Verser », deux sens | Question 33 | 8, 17 |
| C28. Corrections ponctuelles | Les lieux cités à jour ; § 40 · 7.5 ; § 41 · 6 | 17, 25 |

### La relecture par une session neuve

Une session neuve a relu le plan sur `fe918aa` : chaque mécanisme annoncé contre le code, les noms d'`automerge`
0.12.0, de `tauri` 2.12.5, de `tauri-plugin-dialog` 2.8.4 et de `web-sys` 0.3.107 sur docs.rs, et chaque valeur témoin
recalculée. Elle a relevé 27 constats, 3 bloquants, 9 importants et 15 mineurs. La session du lot les a tous retenus,
R22 et R25 autrement que la relecture ne le proposait, et R8 tenu sans écart à la question 14. Quatre questions en
sont nées ; la maîtrise d'ouvrage les a tranchées le 2026-11-21, chacune selon son défaut, recopié en 35 à 38 des
questions tranchées. Une autre session neuve a ensuite vérifié les corrections, contre le
plan, le code et la documentation de la bibliothèque standard : des 27 constats et des quatre questions, 27 étaient
faits et 4 incomplets (R4, R9, C et D, que V1 et V2 complètent) ; elle a relevé 15 défauts nouveaux, V1 à V15,
2 importants et 13 mineurs, tous corrigés, V12 en soumettant à la maîtrise d'ouvrage, avec la question 35, le texte de
son défaut.

| Constat | Ce qui change | Tâches |
|---|---|---|
| R1. Une copie coupée retire son nom provisoire | Chaque test qui garde le nom provisoire coupe aussi `Etape::Retrait` ; la reprise coupée compare son rapport à celui d'une traite | 9, 10, 14 |
| R2. La règle de reprise | Sur les seuls noms de fichiers à l'ULID, une prise en stems par son dossier ; sans nom, aucune reprise ; les déclarations de l'ouverture ; trois tests ; question 35 | 10, 11, 12, 14, 22 |
| R3. `caf` et `wma` sans type | La table de la question 27, à six lignes ; une prise `caf` reçoit 404, une prise `wma` dit la lecture impossible | 20, 21, 23, 25 |
| R4. L'inspection lisait les stems | Le jeu proposé sans empreinte, par les tailles et les en-têtes lus ; question 37 | 5, 12 |
| R5. Le `par` d'un type choisi | Ni `par` ni `depuis` pour un type choisi ; le § 30 · 8.1 gagne le type d'un stem | 6, 8 |
| R6. Le préfixe après un import rejoué | La table (chemin → empreinte) de l'import clos, gardée jusqu'à l'inspection suivante | 12, 13, 22 |
| R7. Les illisibles d'une reprise | Un fichier repris est illisible si sa captation n'a pas de `format` ; ses octets, ceux de la source | 12, 14 |
| R8. Un contenu changé à taille égale | La marque de dernière écriture, relue avant et après la copie | 9, 12, 14 |
| R9. Un fichier illisible à l'inspection | L'inspection s'arrête et le nomme ; question 38 | 9, 12, 15 |
| R10. Les 3 000 acteurs | Un acteur toutes les sept opérations, au moins 3 000 | 16 |
| R11. Deux décisions sans test | Le jeu d'un seul contenu ; le contenu « MIDI versé » d'une requête | 10, 12 |
| R12. CAP-02 et « Fichier : » | CAP-02 gagne la règle | 17 |
| R13. Deux empreintes témoins croisées | `VOIX` et `BASSE` échangées | 3 |
| R14. Deux comptes faux | Un onzième test à la tâche 6 ; « tâches 1 à 6, 20 » | 6 |
| R15. Un renvoi hors du dépôt | La recette du WAV recopiée, les renvois retirés | 2, 3 |
| R16. Un avertissement qui ment | Il ne vaut que pour « de la main de Sacha » sur une origine générée | 6, 13 |
| R17. Un illisible ne s'écoute pas | Dit dans la PR (a), les points de revue et CAP-03 | 8, 17 |
| R18. Un second générateur d'aléa | `Splitmix64` renommé `AleaDuGenerateur`, rendu public | 16 |
| R19. La place libre de la racine | Lue sur le dossier de l'application, le même volume | 15 |
| R20. Une captation changée, rangée sans nom | La copie reçoit l'empreinte attendue, et ne range rien en désaccord | 9, 12 |
| R21. « Annuler » après « Regrouper ici » | Dit à l'écran, au § 50 · 3.1 et dans la PR (d) ; un test | 24, 25 |
| R22. La reproposition par le chemin seul | La lettre de la question 9 se garde, et CAP-03 le dit | 17 |
| R23. Un jeu sans nom en UTF-8 | La prise en stems sans clé `nom`, sans ligne au rapport | 6, 12 |
| R24. CAP-03 · 8 avant l'écran | Inscrit en (d), livré par part en (b) | 17, 25 |
| R25. « Fiche » | Aucun sens de plus : « la captation » et « sa ligne dans le volet de l'idée » | 11, 12, 22 à 26 |
| R26. Un contenu connu, relu à chaque unité | Les empreintes nommées, lues à l'ouverture et tenues dans l'état de l'import | 10, 12 |
| R27. L'archive et les noms provisoires | Un effet cherché de plus | — |
| V1. Le test de R4 ne prouvait rien | Trois WAV de 100 044 octets, distincts dès leurs données ; la borne `SIGNATURE_AU_PLUS + ENTETE_AU_PLUS` ; deux WAV égaux sur 65 536 octets ne proposent rien | 12 |
| V2. La paire en mémoire ne rendait pas la lecture | `refuser(rang, refusee)` ; `refuser_la_lecture(…, refusee)`, sans garde | 9, 12 |
| V3. Le § 30 · 8.1 hors des documents de (a) | La ligne « Documents » de la PR (a) le nomme | — |
| V4. Une marque de dernière écriture à 0 sous Windows | 0 se lit `None` | 9 |
| V5. `SystemTime` dans une doc de `source.rs` | La doc de `parcourir` ne le nomme plus ; I29 et I30 sautent une ligne `//` | 9 |
| V6. La table de la relecture pour R8 et R9 | R8 : 9, 12, 14 ; R9 : 9, 12, 15 | — |
| V7. L'adaptateur en mémoire et `attendue` | Dit, et un test dans `commandes_en_memoire.rs` | 9 |
| V8. Un WAV tronqué resté lisible | Un WAV coupé à 30 octets, avant l'en-tête de `data` | 14 |
| V9. Une déclaration mêlée | L'avertissement dès qu'une cible générée est gardée, l'opération faite pour les autres ; un test | 13 |
| V10. « Rassemblement » | « Annuler ne défait que le dernier regroupement : celui de la dernière captation. » | 24, 25 |
| V11. « Marque », deux sens | Pour `ftyp`, « le code de format aux octets 8 à 11 » ; « marque de dernière écriture » seule | 1, 8, 9, 12, 14, 17 |
| V12. « Un autre dossier » quand c'est le même | Le texte de la question 35 : « Un import est resté ouvert, que ce dossier ne reprend pas : verser celui-ci le clôt, avec ce qu'il a versé. » | 11, 17, 22 |
| V13. La part de CAP-03 · 8 à l'écran | Le test renommé `cap03_8_un_dossier_dit_son_jeu_propose_en_attente`, cité à la revue de (d) | 22, 25 |
| V14. `0o000` sous root | Dit dans « Outils et cas limites » ; la paire vérifie qu'une ouverture échoue, sinon panique | 12 |
| V15. `source_en_memoire.rs` hors de la carte | Une ligne à la carte des fichiers | — |

### Le balayage préalable de la PR (a)

Avant l'exécution de la PR (a), une session neuve a balayé ses tâches 1 à 8 sur `8b5e809` : chaque tâche contre
elle-même, contre ses voisines, contre les contraintes générales et contre le code. Elle a relevé 23 défauts, 4 lourds,
10 moyens et 9 mineurs ; la session du lot les a tous retenus, tels que le balayage les proposait. Ils sont numérotés
P1 à P23, comme dans le rendu du balayage, sans rapport avec les pièges P1 à P15 de la carte du code : le plan les cite
« balayage P4 ». Chaque tâche porte désormais elle-même ce qui la concerne. La question 11 garde sa lettre : c'est la
table de la tâche 1 qui porte l'exception de P3, que la question 10 impose.

| Constat | Ce qui change | Tâches |
|---|---|---|
| P1. La suite des commandes lit les anciens messages de `CaptationAbsente` et de `CaptationDejaVersee` | Ses deux tests (`crates/services/tests/suite/mod.rs:1904`, `:1988`) attendent les messages de la question 26 ; le corps de la PR (a) les nomme | 6, 8 |
| P2. `h07.rs` appelle la règle des empreintes nommées à un argument | `crates/stockage/tests/h07.rs:146` prend `[]` pour les rapports | 7 |
| P3. Le BOM `FF FE` d'un UTF-16 petit-boutiste passait pour un MP3 | La ligne MP3 de la table excepte `0xFE` ; le § 30 · 9.2 et le corps de la PR (a) le disent ; le texte UTF-16 de la tâche 13 se verse en texte | 1, 8, 13 |
| P4. `FormatAudio::` contient `io::`, refusé au noyau | Le type se nomme `FormatSonore` ; la contrainte des contrôles statiques le dit | 1, 6 |
| P5. La borne `ENTIER_SUR` de `faits_wav` sans test | `une_duree_au_dela_d_entier_sur_est_illisible`, sur un RF64 sans données ; `entete.rs` à 13 tests | 2 |
| P6. La taille *syncsafe* éprouvée sur ses deux derniers octets | Un cas ID3 aux quatre octets non nuls (`suite` 2 130 318) ; les entiers de largeur fixe lus par `from_le_bytes` et `from_be_bytes` | 1, 2 |
| P7. Les messages de l'import, dictés à l'octet, sans test | `les_refus_de_l_import_se_disent_a_l_octet` ; les huit variantes dans `aucun_message_n_est_un_ordre` | 6 |
| P8. La naissance d'une idée recopiée trois fois | `gestes::naissance_de_l_idee`, que `gestes::verser` et les deux versements de l'import appellent ; le mécanisme de `gestes.rs:674` récrit | 6 |
| P9. Neuf et dix arguments sous `clippy -D warnings` | `#[allow(clippy::too_many_arguments)]` et sa raison, comme `gestes::verser` | 6 |
| P10. Le rouge attendu de la tâche 7, inversé | `this function takes 1 argument but 2 arguments were supplied`, et `copies_a_retirer` introuvable | 7 |
| P11. Le préambule de la PR (a) hors du texte des tâches ; `:27` au lieu de `:33` | Chaque tâche dit son glob, son `--test`, son compte (`:2` et `:33`), son thème et l'interdit de `while` et de `loop` ; la tâche 7, aucun module nouveau | 1 à 8 |
| P12. Un cas « même durée » aux durées autres | Deux WAV de 500 ms | 5 |
| P13. Deux modules `import` dans les services | Celui des services se nomme `etat_de_l_import` | 12, 14 |
| P14. Le rapport écrit au noyau, relu aux services | `rapport::lire_le_rapport` et `RapportLu` ; `rapport.rs` à 8 tests ; la tâche 13 l'appelle | 3, 13 |
| P15. Des aides de test qui ne compilent pas telles que dites | Les empreintes et `IDEE` en constantes écrites en dur ; les `Faits` de `stems_d_essai` par `LazyLock` ; `Declaree` dérive `Debug` | 6 |
| P16. Les constantes et l'appel de la tâche 7 | `I`, `AUTRE` (un ULID valide) et les empreintes dits ; `[]` partout | 7 |
| P17. Les empreintes témoins écrites deux fois | Au seul `references.rs`, dont l'en-tête nomme le manifeste et le rapport ; le test 11 du § 31 · 10 aussi | 3, 8 |
| P18. Trois écarts de la tâche 2 | Deux tests de plus dans `identifiants.rs` ; `repeat` au lieu d'un `while` ; le BOM coupé entre deux tronçons | 2 |
| P19. `Some(b"ftyp")`, d'un autre type que `get(4..8)` | `Some(b"ftyp".as_slice())` | 1 |
| P20. Deux tests faibles, un nom de test qui devient faux | Des assertions positives pour l'illisible ; `I` refusé par `copie_de` ; `une_captation_absente_ne_nomme_pas_son_empreinte` | 6 |
| P21. La table des mots recopiée sans sa raison | La raison dite dans la tâche | 5 |
| P22. « lui-même compris », ambigu | « du sien au dossier versé (`""`) compris » | 4 |
| P23. Le passage complet de `mutants`, le `git add` et le type du commit de la tâche 8 | `gh workflow run mutants-complet.yml` sur la branche ; une liste de chemins ; les délais dans un commit `ci:` | 8 |

## Les écarts au plan maître

Le plan maître le veut : un plan de tranche qui le contredit le dit dans sa PR, et la maîtrise d'ouvrage relit ce
point. Ce plan ne touche pas le plan maître ; chaque écart est dit ici, et la maîtrise d'ouvrage les a tranchés avec
les questions, le 2026-11-21, chacun selon son défaut.

1. **Quatre PR au lieu de trois** (question 34) : la scission de `boite.rs` passe dans une PR à elle, (c), entre
   l'import et l'écran ; (d) reprend ce que le plan maître mettait en (c).
2. **Les déclarations pendant l'import ne sont pas des opérations nommées** (question 22) : ce sont des paramètres, que
   l'opération d'ouverture écrit et que les versements appliquent. Après l'import, « déclarer une origine » et
   « choisir le type d'un stem » sont des opérations nommées ; « déclarer un jeu de stems » n'en est jamais une.
3. **Le protocole ne sert pas « une empreinte connue »** mais l'audio d'une prise ou d'un stem seul (question 27,
   C8).
4. **L'import n'est pas une commande attendue** : l'interface le conduit unité par unité (question 1, C1) ; « Une
   captation se verse toujours » reste la règle d'une capture, non d'un import (question 15, L22).
5. **« Un import rejoué n'ajoute que son rapport »** vaut à déclarations égales ; un contenu connu ne prend un rôle
   nouveau que par une déclaration (question 9, C5).
6. **Le test 13** : la ligne du § 31 · 10 se récrit avec les nombres de la tranche 2 ; les 3 000 acteurs viennent
   d'acteurs tournants en mémoire, non de réouvertures ; la date, l'appareil et le diapason mesuré restent absents
   (questions 28, 29) ; « trop lente » vaut 2 s (question 30) ; l'étape tourne en `--release` depuis la tranche 2.
7. **Un troisième contrat**, la source à verser, et `Captations` en flux et par plages (choix du plan, C18) : le
   § 40 · 3 dit trois contrats.
8. **Des documents de plus** à la ligne « Documents » de la tranche (C26) : chaque PR les nomme ; « Ce qui doit
   survivre » gagne les clés de l'import et de l'origine (§ 31 · 2), que la clôture du lot reporte.
9. **CAP-03 · 12** s'inscrit avec un test de l'interface et le relevé de l'essai (C23), et **CAP-03 · 8** avec lui,
   dans la PR (d), qui montre la proposition (R24) ; **CAP-03 · 4** ne s'inscrit pas (P15).
10. **Le § 41 · 6** dit trois usages de la boîte de dialogue, non deux : le cahier, le dossier à verser et, d'avance,
    la destination de l'archive (C28).

## Les questions tranchées

Posées le 2026-11-21 en un bloc, tranchées le même jour, chacune selon son défaut. Les renvois nomment les constats
des passages.

1. **L'avancée** [C1, P2, P9] : l'interface conduit l'import unité par unité (l'inspection, le versement d'un fichier
   ou d'un stem, l'opération d'une prise en stems, l'ouverture, la clôture), chaque unité une demande courte au fil ;
   elle montre l'avancée d'après les réponses, sans signal ni minuterie, et ne relit la boîte qu'à la fin ; le
   protocole s'enregistre en asynchrone et passe entre deux unités ; fermer la fenêtre arrête l'import.
2. **La reprise** [C9, L9] : à l'ouverture suivante, le bandeau dit qu'un import est resté ouvert ; rechoisir le même
   dossier le reprend, sous le même identifiant, et son rapport est celui d'un import fait d'une traite.
3. **Regrouper à l'écran** [C6] : des prises, des MIDI versés seuls ou des textes versés, puis l'idée qui les reçoit ;
   les refus au bandeau.
4. **La ligne d'une idée importée sans texte** [C13] : « Fichier : » suivi du premier nom vu de sa première
   captation, écrit comme texte ; la recherche ne lit pas ce nom.
5. **`boite.rs`** [C21] : une PR à part le scinde en `boite/`, sans changement, avant l'écran.
6. **Les titres « Provenance » et « Relier » à 200 %** [C21] : la PR de l'écran les corrige ; la piste se clôt.
7. **L'essai installé** [C20] : à la fin de la dernière PR, sous `app.pupitre.etabli`, sur un fonds d'essai ; le
   dossier à verser engendré par un programme d'exemple du dépôt ; le relevé au § 70 · 7.
8. **La place libre** [C2, L3, L11] : la taille à verser, les octets des fichiers audio, MIDI et texte dont le contenu
   n'est pas déjà une captation ; l'empreinte ne se calcule qu'à taille égale à celle d'une captation présente ; on
   verse si taille + 2 × fichier complet + 4 096 × fichiers tient dans la place offerte à l'appelant ; le refus en Go
   décimaux à une décimale, le manque arrondi au dixième supérieur ; une copie déjà connue retire son nom provisoire ;
   CAP-03 · 13 se lit « une taille à verser plus grande que la place libre » ; une règle du noyau.
9. **L'import rejoué** [C5, L10] : sans déclaration, un contenu connu ne prend aucun rôle ; l'inspection repropose
   les déclarations d'un import précédent, sans les appliquer.
10. **« Illisible »** [C4, L17, L18] : un WAV ou un MIDI dont l'établi ne lit pas l'en-tête, un texte ni UTF-8 (BOM
    admis) ni UTF-16 avec BOM ; versé, ses faits à `—`, compté à part parmi les versés, une fois par contenu ; un
    format non décodé n'est pas illisible ; le glossaire gagne le sens.
11. **Les signatures** [L4, C23] : la table fermée (ci-dessous, tâche 1) ; la signature l'emporte sur l'extension ;
    un fichier ignoré n'est lu que de ses 16 premiers octets au plus.
12. **Les textes** [L17] : `brut` `.txt` ; `markdown` `.md`, `.markdown` ; `chordpro` `.cho`, `.crd`, `.chopro`,
    `.chord`, `.chordpro`, `.pro` ; sans égard à la casse.
13. **Un nom que seul Windows sait écrire** [L20] : versé sans nom, compté à part parmi les versés ; le test de
    `windows` sur 300 unités UTF-16 et sur les deux formes d'un nom.
14. **Liens, dossier de Pupitre, fichier qui change** [C19] : liens et jonctions ignorés, sans être lus ; le dossier
    de l'application, un dossier qui le contient ou qui s'y trouve, refusé par `systeme` ; un fichier changé depuis
    l'inspection arrête l'import, qui le nomme.
15. **Lecture seule, illisible, plus récent** [C7, L22, P3] : l'import entier refusé avant toute captation ; sa
    première opération précède toute captation, et fait naître le fonds dans un Pupitre neuf.
16. **La même durée** [L7] : même nombre de trames à la même fréquence ; `duree` = ⌊trames × 1 000 / fréquence⌋ ; une
    taille de `data` plus grande que le fichier se borne au fichier.
17. **Sous-dossiers, MIDI d'un stem** [L19] : seuls les fichiers du dossier forment le jeu ; le MIDI du même dossier
    au même nom, à l'extension près ; un jeu versé seul prend le dernier segment de son dossier.
18. **Le type d'un stem** [C12, L8] : voix, chœurs, batterie, percussions, basse, guitare, clavier, cordes, cuivres,
    synthé, effets, autre ; proposé d'un mot entier du premier nom vu, replié ; proposé, non stocké, en italique avec
    `~` ; gardé, provenance `importee` (`par` = `nom`, `depuis` = le stem), `⇣` ; choisi, `saisie`, sans glyphe.
19. **Le manifeste** [L5] : les empreintes distinctes, rangées, chacune suivie d'un LF ; témoin `340999f9…` ; un jeu
    sans audio refusé ; un jeu d'un seul contenu donne une prise en stems distincte.
20. **Le premier nom vu** [L1, C3] : `<e>/nom/<chemin>` à l'ULID, écrit s'il est absent ; le plus petit couple
    (heure de l'ULID, octets du chemin), sur toutes les valeurs (`get_all`).
21. **Les clés d'un stem** [L24, C3] : comme l'ADR-024 ; `prises/<p>/manifeste` à `true` ; sous la prise, l'entrée de
    chaque stem (`format`, `duree`, `type`, `midi/<m>`, `nom/<chemin>`) ; le MIDI d'un stem sous `midi_verses`, sans
    `idee`.
22. **Les déclarations** [C15, L12] : paramètres pendant l'import (`imports/<i>/declaration/<chemin>` à `stems`,
    `main` ou `generee:<nom>`), opérations après ; `aucune` refusée ; `main` sur une générée, fait sans rien changer ;
    l'origine en ensemble, `origines_declarees/<e>/main` et `…/generee/<nom>`.
23. **La portée par préfixe** [C16, L12] : segments entiers, octet pour octet, sur les noms qu'a vus un import.
24. **Chercher par origine déclarée** [C21] : aucun critère au lot 1 ; la piste se déclenche à la PR de conception
    d'INC-2.
25. **Le rapport** [C14, L6, L9] : la forme à l'octet (tâche 3) ; `imports/<i>/ouvert` et `imports/<i>/rapport` ; un
    import est clos dès que son rapport existe ; aucun chemin absolu dans le document.
26. **Les messages** [C17] : « Import du 3 novembre ouvert », « Import du 3 novembre clos » ; une captation nommée par
    son premier nom vu ou « cette prise » ; la piste se clôt.
27. **Le protocole** [C8, L16] : l'audio d'une prise ou d'un stem ; la table des types ; `nosniff` ; 1 048 576
    octets ; 200, 206, 416 ; `GET` et `HEAD` seuls ; un texte versé et un rapport se lisent par une commande.
28. **Le test 13 entier** [C10, L13] : par les opérations de l'import ; « échoue s'il pèse 4 500 000 octets ou plus ».
29. **Les 3 000 acteurs** [L14] : l'adaptateur en mémoire change d'acteur toutes les huit opérations, seize octets de
    l'aléa du générateur. La relecture (R10) l'a recompté : 23 090 / 8 donne 2 886 acteurs ; le plan prend sept, quelque
    3 298, pour tenir les 3 000 de la spec, que la question visait.
30. **La compaction** [L15, C28] : 101 fichiers d'ajout, mesurée, trop lente au-delà de 2 s en `--release` ; le plan
    dit le nombre de compactions d'un import du fonds visé.
31. **La copie coupée** [C11, L2] : `import-<ULID>-<rang>.<processus>.partiel` ; le démarrage la range si elle est
    entière et nommée ; la reprise retire les siennes qui ne sont pas nommées ; la clôture les retire toutes ; une
    capture garde sa règle.
32. **LE-11 (d) et (e)** [C22, L23] : les textes de CAP-03 et de SYN-07, recopiés à la tâche 17.
33. **« Verser »** [C27] : réservé à l'import ; regrouper « rassemble ».
34. **Le découpage** : les quatre PR de ce plan.

Nées de la relecture, et tranchées par la maîtrise d'ouvrage le 2026-11-21, chacune selon son défaut :

35. **Un import resté ouvert que son dossier ne reprend plus** [R2] :
    quand l'inspection ne propose pas de reprise et qu'un import est ouvert, le volet le dit (« Un import est resté
    ouvert, que ce dossier ne reprend pas : verser celui-ci le clôt, avec ce qu'il a versé. »), le même dossier
    rechoisi compris, quand l'import ouvert n'a posé aucun nom (V12), et l'ouverture du nouvel import
    clôt d'abord l'ancien ; son rapport compte ce que le fonds sait de lui (les fichiers nommés à son ULID, leurs
    octets ; dédoublonnés et ignorés à zéro), au format de la question 25 ; son message est « Import du … clos ».
    Aucune commande de plus.
36. **Les photos prises pour du M4A** : les codes de format d'image
    (`heic`, `heix`, `hevc`, `mif1`, `msf1`, `avif`, aux octets 8 à 11 d'un `ftyp`) sont ignorés ; les autres codes
    restent `m4a`, une vidéo comprise.
37. **La « même durée » sans empreinte** [R4] : pour proposer un jeu,
    deux WAV sont distincts si leurs tailles ou les octets déjà lus de leur en-tête diffèrent ; l'inspection ne calcule
    aucune empreinte pour la proposition.
38. **Un fichier qui ne se lit pas à l'inspection** [R9] :
    l'inspection s'arrête et le nomme (« « x.wav » ne se lit pas : <erreur>. », sans nom « Le fichier n° <rang> ne se
    lit pas : … »), en `Raison::Disque` ; rechoisir le dossier relance. Un fichier seulement en ligne (OneDrive) est une
    limite dite.

## Points de revue

Six conditions que la spec implique sans qu'aucune de ses sources ne demande de les éprouver ; chacune a son test,
dans la tâche qui possède le code.

1. **Deux fichiers du même contenu dans un même import** : un versé, au plus petit rang, l'autre dédoublonné, deux
   noms d'origine, quel que soit l'ordre dans lequel l'interface les verse.
   `deux_copies_d_un_contenu_versees_dans_le_desordre_comptent_un_verse`, tâche 12.
2. **Un fichier ignoré au nom d'un format reconnu** (un zip nommé `.wav`) : ignoré, lu de 16 octets au plus.
   `un_zip_nomme_wav_est_ignore_et_lu_de_seize_octets`, tâche 12.
3. **Un stem déjà prise** : il reste la prise de son idée, et devient stem du jeu déclaré ; il compte parmi les
   dédoublonnés. `un_stem_deja_prise_reste_la_prise_de_son_idee`, tâche 12.
4. **Une reprise après une coupure entre deux tronçons** : le rapport est celui d'un import fait d'une traite, et
   aucune copie de l'import ne reste. `une_reprise_apres_une_coupure_entre_deux_troncons_ne_laisse_aucune_copie`,
   et, sans coupure, `une_reprise_rend_le_rapport_d_une_traite`, tâche 14.
5. **Une requête du protocole pendant un import** : elle passe entre deux unités et rend sa plage.
   `une_requete_du_protocole_passe_entre_deux_unites`, tâche 21.
6. **Un import pendant une requête en cours** : la requête reste, la boîte se relit une fois, à la fin.
   `l_import_ne_relit_la_boite_qu_a_la_fin`, tâche 22.

Et des limites, sans test, que les PR disent dans leur corps :

- **Une marque de dernière écriture absente** (R8) : un système de fichiers qui ne la tient pas la rend 0 sous
  Windows, que la source lit `None`, qui ne compare rien ; un contenu changé à taille égale s'y verse alors sous son nouveau contenu. PR (b).
- **Un réimport** relit à l'inspection chaque fichier dont la taille égale celle d'une captation (question 8) : une
  seule demande longue, le fil tenu, sans avancée à l'écran (R4). PR (b).
- **Un fichier seulement en ligne** (OneDrive) se télécharge en entier à sa première lecture, celle de sa signature ;
  hors connexion, sa lecture échoue, et l'inspection s'arrête en le nommant (question 38). PR (b).
- **Un fichier illisible ne s'écoute pas** : sans `format`, le protocole rend 404, même pour un WAV que l'appareil
  saurait jouer, dont l'en-tête passe 65 536 octets (R17). PR (a), et CAP-03 (tâche 17).

## Mécanismes annoncés

- `crates/noyau/src/donnees/idee.rs:12` — `pub fn empreintes_nommees` — ne garde que ce qui précède le premier `/` : ni le stem (`<p>/stem/<s>/…`), ni le MIDI d'un stem, ni le rapport ne sont nommés ; la tâche 7 l'étend (P1)
- `crates/noyau/src/donnees/fonds.rs:48` — `CONTENEURS_NOMMANT_UNE_CAPTATION: [&str; 3]` — les trois conteneurs dont la clé nomme une captation par son premier segment ; `imports` n'y entre pas, ses clés commençant par un ULID (tâche 7)
- `crates/noyau/src/donnees/fonds.rs:27` — `pub const IMPORTS: &str = "imports";` — conteneur de la genèse, vide : `ouvert`, `rapport` et `declaration` s'y écrivent sans migration (tâche 6)
- `crates/noyau/src/donnees/fonds.rs:30` — `ORIGINES_DECLAREES` — conteneur de la genèse, vide : l'ensemble des origines s'y écrit sans migration (tâche 6)
- `crates/noyau/src/donnees/fonds.rs:123` — `pub const MIGRATIONS: &[Migration<'static>] = &[];` — aucune migration : la genèse ne change pas, ni son empreinte de référence (tâches 6, 8)
- `crates/stockage/tests/references.rs:26` — `const GENESE_DU_FONDS` — l'empreinte de la genèse, inchangée ; le manifeste et le rapport témoins s'y ajoutent (tâche 3)
- `crates/noyau/src/donnees/gestes.rs:674` — `pub fn verser(` — refuse une captation déjà versée (`existante`) et écrit toujours `format` : l'import ne l'appelle pas, et partage avec elle la naissance de l'idée, que la tâche 6 en extrait (balayage P8)
- `crates/noyau/src/donnees/gestes.rs:704` — `let mut changements = vec![` — le bloc de la naissance d'une idée (son objet, `provenance`, `par`, `le`, `depuis`), que la tâche 6 extrait en `naissance_de_l_idee` (balayage P8)
- `crates/noyau/src/donnees/gestes.rs:673` — `#[allow(clippy::too_many_arguments)]` — le précédent, avec sa raison en commentaire, des deux versements de l'import (tâche 6 ; balayage P9)
- `crates/noyau/tests/gestes.rs:838` — `fn une_captation_absente_se_dit_par_son_empreinte() {` — lit l'ancien message, que la tâche 6 récrit, et se renomme (balayage P20)
- `crates/services/tests/suite/mod.rs:1904` — `"La captation {} est déjà versée.",` — le message de `CaptationDejaVersee` que la suite attend, sur les deux paires : la tâche 6 le récrit (balayage P1)
- `crates/services/tests/suite/mod.rs:1988` — `"Aucune captation ne porte l'empreinte {jamais_versee}."` — le message de `CaptationAbsente` que la suite attend : la tâche 6 le récrit (balayage P1)
- `crates/stockage/tests/messages.rs:301` — `fn aucun_message_n_est_un_ordre() {` — énumère chaque variante d'`Impossible` : la tâche 6 y ajoute les siennes (balayage P7)
- `crates/stockage/tests/h07.rs:146` — `empreintes_nommees(cles_des_prises.iter().map(String::as_str))` — un appelant de la règle du noyau, à un argument : la tâche 7 lui donne `[]` (balayage P2)
- `crates/stockage/tests/references.rs:6` — `Ce fichier est le seul des empreintes de référence` — le manifeste et le rapport témoins n'ont leurs empreintes qu'ici (tâche 3 ; balayage P17)
- `crates/noyau/tests/statique.rs:35` — `"io::",` — cherché en sous-chaîne dans `crates/noyau/src` : `FormatAudio::` le contiendrait, d'où `FormatSonore` (tâches 1, 6 ; balayage P4)
- `crates/noyau/src/donnees/gestes.rs:693` — `if sorte != Sorte::Prise && depuis.is_none() {` — un MIDI versé ou un texte versé n'entre que par l'import (tâche 6)
- `crates/noyau/src/donnees/gestes.rs:215` — `Aucune captation ne porte l'empreinte {empreinte}.` — un message montré qui nomme une empreinte : la tâche 6 le récrit (question 26)
- `crates/noyau/src/donnees/gestes.rs:219` — `La captation {empreinte} est déjà versée.` — de même (tâche 6)
- `crates/stockage/src/lib.rs:230` — `Le contenu relu de la captation {empreinte} ne donne pas son empreinte.` — de même (tâche 9)
- `crates/services/src/en_memoire.rs:154` — `Aucune captation ne porte l'empreinte {empreinte}.` — de même (tâche 9)
- `crates/noyau/src/donnees/gestes.rs:71` — `pub const TOUTES: [Sorte; 3]` — les sortes d'une captation d'une idée ; la tranche n'en ajoute pas : un stem et le MIDI d'un stem ne sont la captation d'aucune idée (tâche 10)
- `crates/noyau/src/donnees/idee.rs:48` — `pub fn duree(prises: &[Option<u64>]) -> Option<u64> {` — la durée d'une idée, `None` dès qu'une prise n'a pas la sienne ; celle d'une prise en stems lui arrive de ses stems (tâches 5, 10)
- `crates/noyau/src/donnees/identifiants.rs:46` — `let condense = Sha256::digest(octets);` — l'empreinte en un seul appel ; `Hacheur` la calcule par tronçons, et la donne égale (tâche 2)
- `crates/noyau/src/donnees/demarrage.rs:14` — `pub fn voisin_libre(` — `<nom>.<processus>.partiel`, puis `-2`, `-3` : la copie en flux prend pour nom `import-<ULID>-<rang>` (tâches 7, 9)
- `crates/noyau/src/donnees/demarrage.rs:107` — `pub fn a_ranger(` — range un nom provisoire par l'empreinte de son contenu, quel que soit son nom : une copie en flux entière et nommée se range au démarrage (tâche 9)
- `crates/stockage/src/captations.rs:53` — `pub fn verser(&self, octets: &[u8])` — la voie par octets, qui passera par la voie en flux (tâche 9)
- `crates/stockage/src/captations.rs:63` — `voisin(&self.dossier().join(&e))` — un nom provisoire formé sur l'empreinte, connue d'avance : la voie par octets le garde (tâche 9)
- `crates/stockage/src/captations.rs:200` — `fn lire_un_nom_provisoire(&self, nom: &str)` — lu en entier par `fichier::lire` : la tâche 9 le lit en flux (vigilance 3)
- `crates/stockage/src/fichier.rs:134` — `fichier.write_all(octets)?;` — un seul `write_all` : la copie en flux écrit tronçon par tronçon (tâche 9)
- `crates/stockage/src/fichier.rs:144` — `fs::read(chemin).map(pannes::relue)` — la seule lecture altérable (tâche 9)
- `crates/stockage/src/pannes.rs:18` — `pub enum Etape {` — aucune étape entre deux tronçons : la tâche 9 ajoute `Troncon` (P7)
- `crates/stockage/src/pannes.rs:71` — `pub fn alterer_la_relecture(rang: usize) -> Panne {` — la lecture en flux compte aussi parmi les relectures (tâche 9)
- `crates/stockage/Cargo.toml:20` — `pannes = []` — la fonction que seuls les tests du stockage activent ; aucune fonction nouvelle (tâche 9)
- `crates/stockage/src/documents.rs:610` — `if demarrage::pupitre_neuf(present, voisins) {` — un Pupitre neuf ne range rien : la première opération de l'import précède toute captation (tâche 12, P3)
- `crates/stockage/src/documents.rs:652` — `ranger_les_noms_provisoires(&nommees)` — range au démarrage chaque nom provisoire que le fonds nomme (tâche 9)
- `crates/stockage/src/documents.rs:951` — `let mut avec = relu.fork();` — le `fork()` de la compaction, dans `charger_contre`, que le test 13 mesure (tâche 16, C28)
- `crates/stockage/src/racine_des_donnees.rs:210` — `pub fn faire_naitre(` — la genèse, la naissance et un geste en un premier fichier complet : l'ouverture de l'import est ce geste (tâche 12)
- `crates/stockage/src/adaptateurs.rs:122` — `fn verser(&self, octets: &[u8]) -> Result<String, Refus> {` — l'un des quatre sites du contrat (tâche 9, P6)
- `crates/services/src/contrats.rs:37` — `fn verser(&self, octets: &[u8]) -> Result<String, Refus>;` — le contrat, par octets seulement (tâche 9, P6)
- `crates/services/src/contrats.rs:28` — `fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture;` — la naissance par un premier geste, que l'ouverture de l'import emploie (tâche 12)
- `crates/services/src/en_memoire.rs:137` — `fn verser(&self, octets: &[u8]) -> Result<String, Refus> {` — l'un des quatre sites du contrat (tâche 9)
- `crates/services/src/lib.rs:388` — `let empreinte = self.captations.verser(octets)?;` — verse avant de lire la lecture seule : la règle d'une capture, que l'import ne suit pas (tâche 12, question 15)
- `crates/services/src/lecture.rs:208` — `pub fn empreintes_nommees(doc: &Automerge) -> BTreeSet<String> {` — ne lit que les clés des trois conteneurs ; la tâche 10 y ajoute les rapports (P1)
- `crates/services/src/lecture.rs:404` — `_ => {}` — saute toute clé autre qu'`idee` et `duree` : la durée d'une prise en stems doit lire ses stems (tâche 10)
- `crates/services/src/mesure.rs:3` — `Il y manque encore les stems` — ce que la tâche 16 ajoute (P8)
- `crates/services/src/mesure.rs:134` — `versees.push(services.verser_une_fiche(` — le générateur verse par `verser_une_fiche`, la voie d'une capture ; il passera par l'import (tâche 16, question 28)
- `crates/services/tests/suite/mod.rs:890` — `fn permis(chemin: &[String]) -> bool {` — n'admet que `idee`, `format` et `duree` sous une captation, rien sous `imports` ni `origines_declarees` (tâche 10, P4)
- `crates/services/tests/suite/mod.rs:916` — `fn interdit(chemin: &[String]) -> bool {` — coupe chaque clé en segments : un nom d'origine `total` serait refusé (tâche 10, P4)
- `crates/services/tests/statique.rs:25` — `const INTERDITS_AUX_SERVICES: &[&str] = &[` — `fs::`, `File`, `spawn` refusés jusque dans les commentaires ; `io::` permis (tâches 9 à 14)
- `crates/noyau/tests/statique.rs:22` — `const INTERDITS: &[&str] = &[` — `io::`, `File`, `random` refusés au noyau (tâches 1 à 7, 20)
- `crates/commandes/src/lib.rs:66` — `pub const NOMS: &[&str] = &[` — la liste des commandes, d'où `build.rs` tire le manifeste de l'application (tâche 11)
- `crates/commandes/tests/serialisation.rs:388` — `assert_eq!(NOMS.len(), 21);` — le compte écrit en dur (tâche 11, P13)
- `crates/stockage/tests/mesure.rs:44` — `operations: 23_000,` — le compte des opérations du générateur, gardé à l'unité (tâche 16)
- `crates/stockage/tests/mesure.rs:21` — `const POIDS_MAX: u64 = 4_500_000;` — la borne du test 13, inchangée (tâche 16)
- `crates/stockage/tests/captations.rs:208` — `fn verser_coupe_a_chaque_etape_ne_laisse_aucune_captation_alteree() {` — I14 et le test 4 sur la voie par octets, rejoués sur la copie en flux (tâche 9)
- `crates/stockage/tests/demarrage.rs:1119` — `fn une_captation_nommee_prend_son_nom_au_demarrage() {` — le renommage perdu du test 4, rejoué sur une copie en flux et sur un stem (tâche 9)
- `app/src-tauri/src/lib.rs:183` — `.register_uri_scheme_protocol("captation", |_contexte, requete| {` — synchrone, sans la racine : la tâche 21 l'enregistre en asynchrone (P9)
- `app/src-tauri/src/lib.rs:197` — `fn le_protocole_des_captations_refuse_toute_requete() {` — remplacé par les tests d'I31 sur le protocole (tâche 21)
- `app/src-tauri/src/lib.rs:119` — `.blocking_save_file()` — la boîte attendue dans `spawn_blocking`, hors du fil principal : le dossier à verser se choisit de même (tâche 15)
- `app/src-tauri/src/fil.rs:116` — `while let Ok(demande) = demandes.recv() {` — une demande à la fois : chaque unité de l'import et chaque requête du protocole en est une (tâches 15, 21)
- `app/src-tauri/src/fil.rs:158` — `expect(dead_code, reason` — `demander_bloquant` reste aux tests : le protocole passe par `demander` (tâche 21, P9)
- `app/src-tauri/src/fil.rs:171` — `pub async fn demander<T: Send + 'static>(` — la voie de chaque commande et de chaque requête du protocole (tâches 15, 21)
- `app/src-tauri/tests/statique.rs:29` — `const CHEMINS_DEVINES: &[&str] = &[` — le contrôle d'I30, sans famille « place libre » ni `unsafe` (tâche 15)
- `app/src-tauri/tests/statique.rs:72` — `const ACTEUR: &[&str] = &[` — les mots d'où naît un acteur ; `set_actor` s'y ajoute (tâche 16)
- `app/src-tauri/tests/statique.rs:83` — `const ACTEUR_ADMIS: &[&str] = &[` — admet déjà `crates/services/src/en_memoire.rs`, où tournent les acteurs du test 13 (tâche 16)
- `app/src-tauri/tests/statique.rs:137` — `fn controler_i29(` — refuse `.message(` dans l'établi : aucune méthode ne se nomme `message` (tâches 15, 21)
- `app/src-tauri/tauri.conf.json:18` — `img-src 'self' http://captation.localhost;` — la CSP admet déjà le protocole, et ne change pas (tâche 21)
- `app/ui/src/natif.rs:50` — `pub(crate) async fn invoquer<R: DeserializeOwned>(` — une promesse par commande, sans progression : l'avancée vient des réponses (tâche 22)
- `app/ui/src/boite.rs:361` — `fn mot_de_la_provenance(` — « importée » en toutes lettres, jamais `⇣` (tâches 18, 24)
- `app/ui/src/boite.rs:255` — `pub(crate) fn page_du_fonds_vide(` — deux constats ; la page propose aussi d'importer (tâches 18, 22)
- `app/ui/src/boite.rs:2382` — `<span>"Provenance"</span>` — le titre qui déborde à 200 % (tâches 18, 24)
- `app/ui/tests/encre_statique.rs:306` — `"animation",` — refusé jusque dans un commentaire de l'interface (tâches 22 à 24)
- `.github/workflows/ci.yml:85` — `timeout-minutes: 13` — le délai de `verifier`, remesuré (tâches 8, 17, 25)
- `.github/workflows/ci.yml:164` — `run: cargo test -p stockage --release --test mesure -- --ignored --nocapture` — l'étape du test 13, en `--release` (tâche 16)
- `.github/workflows/ci.yml:207` — `run: cargo test -p stockage -p synchro` — `windows` ne passe que le stockage, la synchronisation et l'établi (tâches 9, 12, 15)
- `.cargo/mutants.toml:2` — `Vingt et un modules` — le compte des modules mutés, qui monte avec la tranche (tâches 1 à 6, 20)
- `.cargo/mutants.toml:33` — `ces vingt et un modules` — la seconde phrase du compte, une ligne plus bas à chaque glob ajouté (tâches 1 à 6 ; balayage P11)
- `.github/workflows/ci.yml:268` — `Deux fois le plus long passage complet mesuré` — le délai de `mutants` se mesure sur un passage complet, que le job d'une PR ne fait pas : la tâche 8 le lance par `mutants-complet.yml` (balayage P23)
- `crates/services/src/lecture.rs:214` — `idee::empreintes_nommees(cles.iter().map(String::as_str))` — l'appel de la règle du noyau, qui prend `[]` pour les rapports à la tâche 7 (balayage P10)
- `crates/noyau/tests/idee.rs:23` — `empreintes_nommees(cles),` — l'ancien test de la règle, qui prend `[]` à la tâche 7 (balayage P10)
- `scripts/verif-confidentialite.sh:47` — `mp3|m4a|wav|aif|aiff|flac|ogg|mid|midi|musicxml|mxl` — les extensions refusées, sans `caf`, `wma`, ni ChordPro (tâche 17, P12)
- `scripts/verif-faits.mjs:676` — `export function fonctionsNommeesCritere(` — reconnaît un test nommé `cap03_12_…` (tâches 17, 25)
- `crates/stockage/src/captations.rs:67` — `let _ = retirer(&provisoire);` — sur tout refus du rangement, le nom provisoire se retire : un test qui veut le garder, comme une coupure du processus, coupe aussi `Etape::Retrait` (tâches 9, 10, 14 ; R1)
- `crates/stockage/tests/demarrage.rs:356` — `let _retrait = couper(Etape::Retrait, 1);` — le précédent : le retrait coupé avec l'étape visée, le nom provisoire reste (tâches 9, 10, 14 ; R1)
- `crates/services/src/mesure.rs:253` — `struct Splitmix64 {` — l'aléa du générateur, privé : la tâche 16 le renomme `AleaDuGenerateur` et le rend public, sans en écrire un second (R18)
- `app/src-tauri/tests/statique.rs:66` — `const HORLOGE: &[&str] = &["SystemTime", "Instant", "UNIX_EPOCH"];` — vaut pour chaque `src/` de `crates/` : `Metadata::modified`, qui rend un `SystemTime`, y tomberait ; la marque de dernière écriture se lit par `last_write_time` et `mtime` (tâche 9 ; R8)
- `docs/30-modele-de-domaine.md:496` — `Sacha l'a entré` — `saisie` n'a ni `par` ni `depuis` : un type de stem choisi n'en porte pas (tâches 6, 8 ; R5)

## Outils et cas limites

- `fs::symlink_metadata` sous Windows — une jonction est un point d'analyse « substitut de nom », que `FileType::is_symlink` dit lien, quand `metadata` et `read_dir` la suivraient — `une_jonction_compte_parmi_les_ignores_sans_etre_suivie` (tâche 15)
- `OsStr::to_str` — rend `None`, sans erreur, pour un nom Windows qui porte une demi-paire de substitution, et `Path::extension` peut alors rester lisible — `un_nom_sans_forme_utf8_se_parcourt_sans_chemin` (tâche 15)
- NTFS — garde comme deux noms deux fichiers qui ne diffèrent que par la forme Unicode, composée ou décomposée — `les_formes_composee_et_decomposee_d_un_nom_restent_deux_noms` (tâche 15)
- `std::fs` sous Windows — pose seul le préfixe `\\?\` d'un chemin de plus de 260 caractères, quand un appel nu de l'API ne le pose pas — `un_chemin_de_300_unites_utf16_se_parcourt_et_garde_son_nom` (tâche 15)
- `GetDiskFreeSpaceExW` — refuse un chemin long sans `\\?\`, échoue sur un dossier absent, et rend trois nombres dont seul `lpFreeBytesAvailableToCaller` est la place offerte à l'appelant — `la_place_libre_se_lit_sur_un_chemin_long` et `la_place_libre_d_un_dossier_absent_est_refusee` (tâche 15)
- `fs::canonicalize` sous Windows — rend un chemin `\\?\`, qui ne se compare qu'à un autre chemin canonique — `un_dossier_dans_celui_de_l_application_est_refuse` (tâche 15)
- `Path::extension` — ne voit aucune extension à `.txt`, et une vide à `a.` : `signature::extension` rend les mêmes, pour la source en mémoire — `l_extension_suit_path_extension` (tâche 1)
- `str::from_utf8` sur un tronçon — refuse un caractère coupé à la frontière de deux tronçons, que le contrôle du texte doit porter au tronçon suivant — `un_texte_coupe_au_milieu_d_un_caractere_reste_lisible` (tâche 2)
- la taille de `data` d'un WAV, un `u32` — vaut `0xFFFFFFFF` dans un RF64, qui renvoie à `ds64`, et peut passer le fichier — `un_rf64_prend_sa_taille_dans_ds64` et `la_taille_de_data_se_borne_au_fichier` (tâche 2)
- le produit `trames × 1 000` — déborde un `u64` au-delà de 1,8 × 10¹⁶ trames — `la_duree_se_calcule_sans_debordement` (tâche 2)
- `cargo-mutants` 27.1.0 — ne mute pas une constante : la table des signatures et celle des mots des types ne se gardent que par leurs tests exhaustifs — `chaque_signature_de_la_table_se_reconnait` (tâche 1) et `la_table_des_mots_est_rangee_et_chaque_mot_donne_son_type` (tâche 5)
- `cargo-mutants` 27.1.0, sur un décalage — change `<<` en `>>`, et un octet nul décalé rend la même valeur : le mutant survit si les octets de poids fort valent 0 — `chaque_signature_de_la_table_se_reconnait`, son cas ID3 aux quatre octets de taille non nuls (tâche 1)
- `cargo-mutants` 27.1.0, sur une comparaison — change `>` en `==` et en `>=` : une borne qu'aucun test n'atteint garde ces mutants vivants — `une_duree_au_dela_d_entier_sur_est_illisible` (tâche 2), à `ENTIER_SUR` et à `ENTIER_SUR + 1`
- la synchronisation d'une trame MPEG, `FF` puis `E0` à `FF` — couvre aussi le BOM `FF FE` d'un UTF-16 petit-boutiste, qui n'est pas un MP3 — `la_signature_l_emporte_sur_l_extension` (tâche 1)
- `Read::read` — peut rendre un tronçon plus court que demandé, d'un seul octet : le BOM d'un UTF-16 arrive alors coupé — `un_texte_utf16_avec_bom_est_lisible` (tâche 2), le BOM `FF` puis `FE`
- `clippy::too_many_arguments` — refuse sous `-D warnings` une fonction de plus de sept arguments — aucun test : « Lint » de `verifier`, après l'`allow` raisonné des deux versements de l'import (tâche 6)
- une boucle sans borne sur les morceaux d'un WAV — un mutant y tourne sans fin, et `mutants` sort en échec sur un TIMEOUT — `un_wav_aux_morceaux_sans_fin_est_illisible` (tâche 2), le parcours borné par `take`
- `Automerge::get_all` — rend chaque valeur concurrente d'une clé posée sur deux répliques, quand `get` n'en rend qu'une — `le_premier_nom_vu_de_deux_repliques_est_le_plus_petit` (tâche 13)
- `Automerge::hash_for_opid` et les lectures `*_at` — lisent le fonds à la tête de l'ouverture de l'import, que le rapport compare à sa fin — `une_reprise_rend_le_rapport_d_une_traite` (tâche 14)
- `Automerge::save` — rend des octets compressés, dont la longueur est le poids d'un fichier complet, que la place libre compte deux fois — `la_place_compte_deux_fois_le_poids_du_fonds` (tâche 12)
- `register_asynchronous_uri_scheme_protocol` — répond par `UriSchemeResponder::respond`, une fois, hors du fil de la WebView ; un gestionnaire synchrone le tiendrait pendant la demande au fil — `i31_le_protocole_s_enregistre_en_asynchrone` (tâche 21)
- `<audio>` sous WebView2 — demande `Range: bytes=0-`, puis des plages, et ne cherche dans la prise que si la réponse dit `Accept-Ranges: bytes` — `une_requete_sans_plage_au_dela_de_la_borne_rend_206` (tâche 20), et l'essai installé (tâche 26)
- `HTMLMediaElement::can_play_type` — rend `""`, `"maybe"` ou `"probably"`, jamais un booléen — `cap03_12_une_prise_que_l_appareil_ne_sait_pas_jouer_montre_ses_faits` (tâche 23)
- `blocking_pick_folder` — bloque le fil qui l'appelle ; appelé dans `spawn_blocking`, comme `blocking_save_file` — aucun test sans fenêtre ne le peut : l'essai installé (tâche 26) choisit un dossier, puis ferme la boîte sans choisir
- les contrôles statiques, en sous-chaîne — refusent `File`, `io::`, `fs::`, `spawn`, `animation`, `.message(` jusque dans un commentaire en français — `statique.rs` du noyau, des services, de l'interface et de l'établi, verts après chaque tâche qui y écrit
- `git grep -n -i -w`, la recherche de LE-2 — `-i` ne replie pas les accents, et `-w` prend une lettre accentuée pour une borne de mot — aucun test ne le peut, limite que chaque PR dit : sa revue relit la recherche citée, faite aussi sans accent
- `MetadataExt::last_write_time` sous Windows, `MetadataExt::mtime` et `mtime_nsec` ailleurs — une marque de dernière écriture que tient le système de fichiers, non une lecture de l'horloge (aucun mot d'`HORLOGE`) ; un système de fichiers qui ne la tient pas la rend 0 sous Windows, que la source lit `None`, qui ne compare rien — `un_contenu_change_a_taille_egale_arrete_l_import` (tâche 14), la marque de dernière écriture d'une source en mémoire avancée par `changer`
- `OpenOptionsExt::share_mode(0)` sous Windows, les droits `0o000` ailleurs — un fichier tenu sans partage, ou sans droit de lecture, refuse toute autre ouverture, que l'inspection voit comme une erreur de lecture ; root lit malgré `0o000`, et la paire de fichiers, sous Linux, vérifie d'abord qu'une ouverture échoue (V14) — `un_fichier_qui_ne_se_lit_pas_arrete_l_inspection` (tâche 12), sur la paire de fichiers dans `verifier` et `windows`
- la boîte `ftyp` d'ISO, aux octets 4 à 7 — porte aussi une photo HEIC ou AVIF, que le code de format aux octets 8 à 11 distingue d'un audio ou d'une vidéo — `chaque_signature_de_la_table_se_reconnait` (tâche 1)

## Effets cherchés

- `Captations` gagne la voie en flux, et la voie par octets passe par elle — la tranche 6 (l'archive et la réimportation copient en flux), le carnet du lot 2 (une capture) — `verser_par_octets_passe_par_le_flux` et les tests d'I14 sur la copie en flux (tâche 9)
- Un troisième contrat, `Source`, réalisé sur fichiers et en mémoire, la marque de dernière écriture de chaque entrée comprise — la tranche 6, qui lira une archive par lui ; le § 40 · 3 — les tests de la source (tâche 9) et la suite des commandes sur les deux paires (tâches 12, 14)
- Les clés de l'import et de l'origine, et `<e>/nom/<chemin>` à l'ULID — le schéma de la `v0.3.0`, que la clôture du lot gèle ; le miroir du lot 4 ; « Ce qui doit survivre » du plan maître — I01 étendu (tâche 10) et les tests à deux répliques (tâche 13)
- La règle des empreintes nommées lit les stems, leurs MIDI et les rapports — le démarrage, sous Windows et sous Linux, et la tranche 6 (l'archive porte ce que le fonds nomme) — `un_stem_nomme_prend_son_nom_au_demarrage` (tâche 9)
- La copie en flux sous `import-<ULID>-<rang>`, retirée à la reprise et à la clôture — le démarrage, sous Windows et sous Linux ; une capture garde sa règle — `la_cloture_retire_chaque_copie_de_l_import` (tâche 14) et `une_copie_de_capture_n_est_jamais_retiree` (tâche 7)
- `NOMS` gagne dix noms — les droits de Windows, `generate_handler!` et chaque tranche suivante — les tests d'I31 (tâche 15)
- La place libre et le premier `unsafe`, dans `systeme` seul, sous `#[cfg(windows)]` — le contrôle statique d'I30, et la compilation de l'établi hors de Windows (`clippy` de `verifier`) — `i30_chaque_controle_refuse_ce_qu_on_y_plante` (tâche 15)
- L'import conduit unité par unité, chaque unité une demande courte au fil — le protocole, qui passe entre deux unités ; la tranche 6, qui ferme le fil ; la tranche 7, qui chronomètre l'ouverture — `une_requete_du_protocole_passe_entre_deux_unites` (tâche 21)
- Le protocole, chaque requête une demande au fil, qui lit une plage et rend le fichier — la tranche 6 (vigilance 5 : fermer chaque fichier tenu avant de renommer la racine) — `la_lecture_d_une_plage_ne_garde_aucun_fichier_ouvert` (tâche 21)
- L'interface ne relit la boîte qu'à la fin de l'import — l'index de la tranche 4, rebâti une fois à la lecture qui suit — `l_import_ne_relit_la_boite_qu_a_la_fin` (tâche 22)
- Le générateur du test 13 passe par l'import, et l'adaptateur en mémoire change d'acteur — la tranche 7 (le mode démonstration prend le fonds de mesure) et le contrôle d'I30 (`set_actor`) — le test du fonds de mesure et `i30_hors_de_systeme_ni_chemin_devine_ni_horloge_ni_alea` (tâche 16)
- La compaction mesurée, 2 s au plus en `--release` — la tranche 6 et le budget de l'ADR-024 — l'étape « Le fonds de mesure » de `verifier` (tâche 16)
- Le refus de place dit en Go décimaux, par une règle du noyau — la tranche 6, dont le test 14 exporte vers un support trop petit — `la_place_manque_se_dit_en_go_decimaux` (tâche 3)
- La porte `contenu` refuse les extensions des formats reconnus et des textes ChordPro — chaque PR qui suit ; un dépôt né du modèle n'est pas touché, le script appartenant au projet — `bash scripts/verif-confidentialite.sh` sur la tête (tâche 17)
- `verser_une_fiche` reste la voie d'une capture, qui verse toujours — le carnet du lot 2 (CAP-01) — les tests de la tranche 2, inchangés (tâche 12)
- La boîte de l'interface en module `boite/` — chaque tranche qui touche l'écran — les 47 tests de `boite.rs`, nommés et comptés avant et après, et le multiensemble de ses lignes (tâche 18)
- Le rapport d'import, un texte UTF-8 qui commence par `pupitre-rapport-d-import 1` — la tranche 6 (l'archive porte les rapports), le lot 4 (le miroir) — le rapport témoin de `references.rs` (tâche 3)
- Le type d'un stem gardé ou choisi s'écrit avec sa provenance — INC-2, où le stem entre dans un morceau — `garder_puis_choisir_le_type_d_un_stem` (tâche 13)
- Un import resté ouvert, que le dossier choisi ne reprend pas, se clôt à l'ouverture du suivant (question 35) — la tranche 6, dont une archive réimportée porte un import ouvert sans son dossier — `un_import_ouvert_ailleurs_se_clot_a_l_ouverture_du_suivant` (tâche 14)
- Les noms provisoires `import-<ULID>-<rang>.<processus>.partiel` de `captations/` — la tranche 6, dont l'archive ne copie que les captations rangées (R27) — le test d'export de la tranche 6
- Les messages de `CaptationAbsente` et de `CaptationDejaVersee` ne nomment plus l'empreinte — la suite des commandes, sur les deux paires, dans `verifier` et `windows` — ses deux tests réécrits (tâche 6 ; balayage P1)
- Le BOM `FF FE` n'est plus une signature MP3 : un texte UTF-16 petit-boutiste se verse en texte — l'inspection et le versement de la PR (b), le § 30 · 9.2 — `la_signature_l_emporte_sur_l_extension` (tâche 1) et `lire_un_texte_verse_le_rend_tel_quel` (tâche 13 ; balayage P3)
- Le rapport d'import se relit au noyau, par `rapport::lire_le_rapport` — la tâche 13, et la tranche 6, dont l'archive porte les rapports — `le_rapport_se_relit` (tâche 3 ; balayage P14)
- La naissance d'une idée, partagée par `gestes::verser` et les versements de l'import — la capture (`verser_une_fiche`, CAP-01) — les tests de `gestes::verser`, inchangés, et ceux de l'import (tâche 6 ; balayage P8)

## Revues

- la revue de chaque tâche : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12, Tâche 13, Tâche 14, Tâche 15, Tâche 16, Tâche 17, Tâche 18, Tâche 19, Tâche 20, Tâche 21, Tâche 22, Tâche 23, Tâche 24, Tâche 25
- la revue de la PR (a) entière, sur sa tête, chaque règle de LE-6 recopiée et chaque valeur témoin recalculée : Tâche 8
- la revue de la PR (b) entière, sur sa tête, le tableau du § 70 · 4.1 comparé aux critères de la tranche : Tâche 17
- la revue de la PR (c) entière, sur sa tête, le compte des tests avant et après : Tâche 19
- la revue de la PR (d) entière, sur sa tête, LE-17 compris : Tâche 25
- la relecture des corrections de chaque revue de PR, sur la tête qui part, celle que nomme `--match-head-commit` : Tâche 8, Tâche 17, Tâche 19, Tâche 25
- la relecture du programme de l'essai, de son étape de CI et du relevé de l'essai installé, sur la tête qui part : Tâche 26

---

## La carte des fichiers

```text
crates/noyau/src/donnees/
  signature.rs                     (a) SIGNATURE_AU_PLUS, Nature, FormatSonore, FormatTexte, Reconnu, reconnaitre,
                                   apres_id3, extension                                                (muté)
  entete.rs                        (a) ENTETE_AU_PLUS, FaitsWav, FaitsMidi, faits_wav, faits_midi,
                                   ControleDuTexte                                                      (muté)
  identifiants.rs                  (a) Hacheur                                                          (muté)
  rapport.rs                       (a) manifeste, Compte, Comptes, FichierDeLImport, comptes, rapport,
                                   RapportLu, lire_le_rapport, MARGE_PAR_FICHIER, place, go             (muté)
  origine.rs                       (a) Origine, OriginesLues, la_plus_forte, ajout_d_origine, premier_nom_vu,
                                   sous_le_prefixe, Declaration, declaration_du_fichier                 (muté)
  stems.rs                         (a) TypeDeStem, MOTS_DES_TYPES, mots_du_nom, type_propose, stem_du_midi,
                                   jeu_propose, duree_de_la_prise_en_stems                              (muté)
  import.rs                        (a) les opérations nommées de l'import, nom_de_la_copie, copie_de    (muté)
  idee.rs                          (a) empreintes_nommees étendue                                       (muté)
  demarrage.rs                     (a) copies_a_retirer                                                 (muté)
  gestes.rs                        (a) Impossible : les variantes de l'import, les messages sans empreinte ;
                                   naissance_de_l_idee, partagée avec l'import                          (muté)
  fonds.rs                         (a) les clés de l'import, de l'origine, d'un stem et d'un en-tête
  protocole.rs                     (d) REPONSE_AU_PLUS, type_du_format, Plage, plage, Servie, servir    (muté)
  mod.rs                           (a) signature, entete, rapport, origine, stems, import ; (d) protocole
crates/noyau/tests/signature.rs, entete.rs, rapport.rs, origine.rs, stems.rs, import.rs   (a)
crates/noyau/tests/protocole.rs    (d)
crates/noyau/tests/identifiants.rs, idee.rs, demarrage.rs, gestes.rs   (a)
crates/stockage/tests/references.rs (a) le manifeste et le rapport témoins
crates/stockage/tests/h07.rs       (a) la règle des empreintes nommées, à deux arguments
crates/stockage/tests/messages.rs  (a) les variantes de l'import dans aucun_message_n_est_un_ordre
crates/services/src/contrats.rs    (b) Source, Entree, SorteDEntree, Versement ; Captations en flux et par plages
crates/services/src/source_en_memoire.rs (b) SourceEnMemoire, hors d'`en-memoire`
crates/services/src/en_memoire.rs  (b) CaptationsEnMemoire en flux ; DocumentsEnMemoire aux acteurs tournants
crates/services/src/lib.rs         (b) l'état de l'import, inspecter, ouvrir_l_import, verser_un_fichier,
                                   verser_une_prise_en_stems, clore_l_import, declarer_une_origine,
                                   choisir_le_type_d_un_stem, lire_un_texte_verse, lire_un_rapport, generateurs ;
                                   (d) captation_a_servir
crates/services/src/etat_de_l_import.rs (b) l'état de l'import et ses conversions (module des services)
crates/services/src/lecture.rs     (b) noms, origines, stems, manifeste, rapports, imports ouverts
crates/services/src/mesure.rs      (b) le générateur du test 13 entier
crates/services/tests/suite/mod.rs (a) deux messages ; (b) Paire::source, les fichiers inventés, cap03_*, i01,
                                   les répliques
crates/services/tests/commandes_en_memoire.rs, crates/stockage/tests/commandes_sur_fichiers.rs   (b)
crates/stockage/src/source.rs      (b) SourceSurFichiers
crates/stockage/src/captations.rs, fichier.rs, pannes.rs, adaptateurs.rs, lib.rs   (b)
crates/stockage/tests/captations.rs, demarrage.rs, source.rs, mesure.rs   (b)
crates/stockage/examples/dossier_a_verser.rs   (d) le programme d'exemple de l'essai
crates/commandes/src/lib.rs        (b) les types de l'import, dix noms
crates/commandes/tests/serialisation.rs   (b)
app/src-tauri/src/systeme.rs       (b) place_libre, admettre_le_dossier, DossierDeLApplication
app/src-tauri/src/commandes_de_l_etabli.rs, src/lib.rs   (b) les commandes ; (d) le protocole
app/src-tauri/src/protocole.rs     (d) servir_une_captation
app/src-tauri/capabilities/windows.json, tests/statique.rs, tests/i31.rs   (b), (d)
app/ui/src/boite.rs → app/ui/src/boite/   (c) mod.rs, fenetre.rs, textes.rs, gestes.rs, requetes.rs, etat.rs,
                                   focus.rs, ecran.rs, vue.rs, volets.rs
app/ui/src/import.rs               (d) l'écran de l'import, ses déclarations, son rapport
app/ui/src/ecoute.rs               (d) la lecture simple
app/ui/src/boite/*.rs, app.rs, bandeau.rs, main.rs, style.css   (d)
docs/…                             (a) tâche 8 ; (b) tâche 17 ; (c) tâche 19 ; (d) tâches 25, 26
.cargo/mutants.toml                (a) six modules ; (d) protocole
.github/workflows/ci.yml, mutants-complet.yml   les délais ; (b) l'étape du test 13
scripts/verif-confidentialite.sh   (b) les extensions
CONVENTIONS.md, CLAUDE.md          (a), (b), (c), (d)
```

## PR (a) · les règles de l'import, au noyau

**Branche :** `t5a-les-regles-de-l-import`. **Titre :** `feat: les règles de l'import au noyau : signatures, en-têtes,
manifeste, rapport, origines` (90). **Documents :** § 30 · 8.1 et 9.2 ; § 31 · 2 et 10 (test 11) ; § 70 · 1 ; le
glossaire ; `.cargo/mutants.toml` ; `CONVENTIONS.md` ; `CLAUDE.md`. **Fin :** la tâche 8.

Chacune des tâches 1 à 6 de cette PR ajoute un module du noyau à `.cargo/mutants.toml` (son chemin dans
`examine_globs`, son fichier de tests dans `additional_cargo_test_args`, le compte des modules à `:2` et dans « ces …
modules », à `:33` sur `fe918aa`, et son thème à l'énumération des lignes 2 à 7), et le déclare dans
`crates/noyau/src/donnees/mod.rs` ; la tâche 7 n'en ajoute aucun. Aucune boucle `while` ni `loop` écrite à la main
dans `src/` : un parcours se borne par un itérateur fini (`take`, une plage), sinon un mutant tourne sans fin et
`mutants` échoue (`voisin_libre`, `crates/noyau/src/donnees/demarrage.rs:14`, le dit). Chaque tâche le redit dans son
texte, que son agent lit seul (balayage P11).

### Tâche 1 : les signatures et les extensions, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/signature.rs`, `crates/noyau/tests/signature.rs` ; modifier
`crates/noyau/src/donnees/mod.rs` (`pub mod signature;`), `.cargo/mutants.toml`.

**`.cargo/mutants.toml`** (balayage P11) : `"crates/noyau/src/donnees/signature.rs"` dans `examine_globs`,
`"--test", "signature"` dans `additional_cargo_test_args` ; le compte devient « Vingt-deux modules » à `:2` et « ces
vingt-deux modules » à la phrase qui suit la liste (`:33` sur `fe918aa`, une ligne plus bas après le glob ajouté) ;
l'énumération des lignes 2 à 7 gagne « les signatures et les extensions ». Aucune boucle `while` ni `loop` dans `src/` :
un parcours se borne par un itérateur fini.

**Le nom des formats audio** (balayage P4) : `FormatSonore`, jamais `FormatAudio`. Le contrôle statique du noyau
cherche `io::` en sous-chaîne dans `crates/noyau/src`, commentaires compris (`crates/noyau/tests/statique.rs:35`), et
le chemin d'une variante d'un type nommé ainsi le contiendrait. Les tests, qu'il ne lit pas, nomment le type librement.

**Interfaces — produit** (`noyau::donnees::signature`) :

```rust
/// Les octets qu'on lit d'un fichier pour le reconnaître, au plus (question 11) : un fichier
/// ignoré n'est jamais lu au-delà.
pub const SIGNATURE_AU_PLUS: usize = 16;

/// Les formats audio que l'import reconnaît à leur signature (§ 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatSonore { Wav, Flac, Mp3, M4a, Ogg, Aiff, Caf, Wma }
impl FormatSonore {
    pub const TOUS: [FormatSonore; 8];
    /// `wav`, `flac`, `mp3`, `m4a`, `ogg`, `aiff`, `caf`, `wma` : la valeur de `<e>/format`.
    pub fn nom(self) -> &'static str;
}

/// Les formats d'un texte versé, d'après son extension (question 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatTexte { Brut, Markdown, Chordpro }
impl FormatTexte {
    /// `brut`, `markdown`, `chordpro`.
    pub fn nom(self) -> &'static str;
}

/// Ce qu'est un fichier, d'après sa signature ou son extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nature { Audio(FormatSonore), Midi, Texte(FormatTexte), Ignore }

/// Ce que dit le début d'un fichier : sa nature, ou une étiquette ID3, dont la suite, à l'octet
/// `suite`, dit FLAC ou MP3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reconnu { Nature(Nature), Id3 { suite: u64 } }

/// La nature d'un fichier d'après `debut`, ses `SIGNATURE_AU_PLUS` premiers octets au plus (moins
/// s'il est plus court), puis son extension, sans égard à la casse ; la signature l'emporte.
pub fn reconnaitre(debut: &[u8], extension: Option<&str>) -> Reconnu;

/// FLAC si `quatre`, les quatre octets qui suivent une étiquette ID3, valent `fLaC` ; MP3 sinon.
pub fn apres_id3(quatre: &[u8]) -> FormatSonore;

/// L'extension d'un segment de chemin, comme `Path::extension` : ce qui suit son dernier point,
/// `None` sans point ou quand le seul point le commence.
pub fn extension(segment: &str) -> Option<&str>;
```

La table, fermée (question 11), dans cet ordre de lecture. La ligne MP3 excepte l'octet 1 `0xFE` : `FF FE` est le BOM
d'un UTF-16 petit-boutiste, qu'admet la question 10 ; sans l'exception, un `.txt` enregistré « Unicode » sous Windows
serait versé en prise `mp3`, la signature l'emportant sur l'extension (balayage P3). La question 11 garde sa lettre :
c'est cette table qui porte l'exception, que la tâche 8 recopie au § 30 · 9.2.

| Début du fichier | Nature |
|---|---|
| `RIFF` ou `RF64` aux octets 0 à 3, `WAVE` aux octets 8 à 11 | `Audio(Wav)` |
| `RIFF` aux octets 0 à 3, `RMID` aux octets 8 à 11 | `Ignore` |
| `fLaC` aux octets 0 à 3 | `Audio(Flac)` |
| `ID3` aux octets 0 à 2 (dix octets au moins) | `Id3 { suite }`, `suite` = 10 + la taille *syncsafe* des octets 6 à 9 (7 bits de chacun), + 10 si l'octet 5 porte le drapeau `0x10` |
| `0xFF` à l'octet 0, et l'octet 1 de `0xE0` à `0xFF`, sauf `0xFE` (le BOM d'un UTF-16 petit-boutiste, question 10) | `Audio(Mp3)` |
| `ftyp` aux octets 4 à 7, et un code de format d'image aux octets 8 à 11 : `heic`, `heix`, `hevc`, `mif1`, `msf1` ou `avif` (question 36) | `Ignore` |
| `ftyp` aux octets 4 à 7, tout autre code de format, une vidéo comprise | `Audio(M4a)` |
| `OggS` aux octets 0 à 3 | `Audio(Ogg)` |
| `FORM` aux octets 0 à 3, `AIFF` ou `AIFC` aux octets 8 à 11 | `Audio(Aiff)` |
| `caff` aux octets 0 à 3 | `Audio(Caf)` |
| les seize octets `30 26 B2 75 8E 66 CF 11 A6 D9 00 AA 00 62 CE 6C` | `Audio(Wma)` |
| `MThd 00 00 00 06` aux octets 0 à 7 | `Midi` |
| sinon, l'extension `txt` | `Texte(Brut)` |
| sinon, `md` ou `markdown` | `Texte(Markdown)` |
| sinon, `cho`, `crd`, `chopro`, `chord`, `chordpro` ou `pro` | `Texte(Chordpro)` |
| sinon | `Ignore` |

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/signature.rs`). Les octets sont écrits par le test :

```rust
const ASF: [u8; 16] = [0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C];

fn debut(octets: &[u8]) -> Vec<u8> { let mut v = octets.to_vec(); v.resize(16, 0); v }

#[test]
fn chaque_signature_de_la_table_se_reconnait() {
    use FormatSonore::*;
    let cas: Vec<(Vec<u8>, Reconnu)> = vec![
        (debut(b"RIFF\0\0\0\0WAVE"), Reconnu::Nature(Nature::Audio(Wav))),
        (debut(b"RF64\0\0\0\0WAVE"), Reconnu::Nature(Nature::Audio(Wav))),
        (debut(b"RIFF\0\0\0\0RMID"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"fLaC"), Reconnu::Nature(Nature::Audio(Flac))),
        (debut(&[0xFF, 0xFB, 0x90]), Reconnu::Nature(Nature::Audio(Mp3))),
        (debut(&[0xFF, 0xE0]), Reconnu::Nature(Nature::Audio(Mp3))),
        (debut(b"\0\0\0\x18ftypM4A "), Reconnu::Nature(Nature::Audio(M4a))),
        (debut(b"\0\0\0\x18ftypisom"), Reconnu::Nature(Nature::Audio(M4a))),
        (debut(b"\0\0\0\x14ftypqt  "), Reconnu::Nature(Nature::Audio(M4a))),
        (debut(b"\0\0\0\x18ftypheic"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"\0\0\0\x18ftypheix"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"\0\0\0\x18ftyphevc"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"\0\0\0\x18ftypmif1"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"\0\0\0\x18ftypmsf1"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"\0\0\0\x1cftypavif"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"OggS"), Reconnu::Nature(Nature::Audio(Ogg))),
        (debut(b"FORM\0\0\0\0AIFF"), Reconnu::Nature(Nature::Audio(Aiff))),
        (debut(b"FORM\0\0\0\0AIFC"), Reconnu::Nature(Nature::Audio(Aiff))),
        (debut(b"caff"), Reconnu::Nature(Nature::Audio(Caf))),
        (ASF.to_vec(), Reconnu::Nature(Nature::Audio(Wma))),
        (debut(b"MThd\0\0\0\x06\0\x01\0\x03\x01\xE0"), Reconnu::Nature(Nature::Midi)),
        (debut(b"ID3\x04\0\0\0\0\x01\x48"), Reconnu::Id3 { suite: 210 }),
        (debut(b"ID3\x04\0\x10\0\0\x01\x48"), Reconnu::Id3 { suite: 220 }),
        // Les quatre octets de taille non nuls : (1 << 21) + (2 << 14) + (3 << 7) + 4 + 10 (balayage P6).
        (debut(b"ID3\x04\0\0\x01\x02\x03\x04"), Reconnu::Id3 { suite: 2_130_318 }),
    ];
    for (octets, attendu) in cas {
        assert_eq!(reconnaitre(&octets, None), attendu, "{octets:02x?}");
    }
    assert_eq!(reconnaitre(&debut(&[0xFF, 0xDF]), None), Reconnu::Nature(Nature::Ignore));
    assert_eq!(reconnaitre(&debut(b"FORM\0\0\0\0WAVE"), None), Reconnu::Nature(Nature::Ignore));
    assert_eq!(reconnaitre(&debut(b"MThd\0\0\0\x07"), None), Reconnu::Nature(Nature::Ignore));
}

#[test]
fn apres_une_etiquette_id3_flac_ou_mp3() {
    assert_eq!(apres_id3(b"fLaC"), FormatSonore::Flac);
    assert_eq!(apres_id3(&[0xFF, 0xFB, 0x90, 0x00]), FormatSonore::Mp3);
    assert_eq!(apres_id3(b"fLa"), FormatSonore::Mp3);
}

#[test]
fn la_signature_l_emporte_sur_l_extension() {
    assert_eq!(reconnaitre(&debut(b"RIFF\0\0\0\0WAVE"), Some("txt")), Reconnu::Nature(Nature::Audio(FormatSonore::Wav)));
    assert_eq!(reconnaitre(&debut(b"PK\x03\x04"), Some("wav")), Reconnu::Nature(Nature::Ignore));
    assert_eq!(reconnaitre(&debut(b"PK\x03\x04"), Some("txt")), Reconnu::Nature(Nature::Texte(FormatTexte::Brut)));
    assert_eq!(reconnaitre(&debut(b"%PDF-1.7"), Some("pdf")), Reconnu::Nature(Nature::Ignore));
    // Le BOM d'un UTF-16 petit-boutiste n'est pas une trame MP3 (question 10 ; balayage P3).
    assert_eq!(reconnaitre(&debut(&[0xFF, 0xFE, 0x50, 0x00]), Some("txt")), Reconnu::Nature(Nature::Texte(FormatTexte::Brut)));
    assert_eq!(reconnaitre(&debut(&[0xFF, 0xFE, 0x50, 0x00]), None), Reconnu::Nature(Nature::Ignore));
}

#[test]
fn les_extensions_des_textes_sans_egard_a_la_casse() {
    let texte = |e| reconnaitre(b"Pont du soir", Some(e));
    for e in ["txt", "TXT", "Txt"] { assert_eq!(texte(e), Reconnu::Nature(Nature::Texte(FormatTexte::Brut)), "{e}"); }
    for e in ["md", "Md", "MARKDOWN"] { assert_eq!(texte(e), Reconnu::Nature(Nature::Texte(FormatTexte::Markdown)), "{e}"); }
    for e in ["cho", "CRD", "chopro", "chord", "ChordPro", "pro"] {
        assert_eq!(texte(e), Reconnu::Nature(Nature::Texte(FormatTexte::Chordpro)), "{e}");
    }
    for e in ["doc", "rtf", "", "tx"] { assert_eq!(texte(e), Reconnu::Nature(Nature::Ignore), "{e}"); }
    assert_eq!(reconnaitre(b"Pont du soir", None), Reconnu::Nature(Nature::Ignore));
}

#[test]
fn un_debut_court_ne_se_reconnait_pas_a_tort() {
    assert_eq!(reconnaitre(b"RIFF", None), Reconnu::Nature(Nature::Ignore));
    assert_eq!(reconnaitre(b"ID3\x04", None), Reconnu::Nature(Nature::Ignore));
    assert_eq!(reconnaitre(&ASF[..15], None), Reconnu::Nature(Nature::Ignore));
    assert_eq!(reconnaitre(b"", Some("txt")), Reconnu::Nature(Nature::Texte(FormatTexte::Brut)));
    assert_eq!(reconnaitre(b"", None), Reconnu::Nature(Nature::Ignore));
}

#[test]
fn l_extension_suit_path_extension() {
    for (segment, attendue) in [("a.txt", Some("txt")), (".txt", None), ("a.", Some("")), ("a.tar.gz", Some("gz")),
                                ("sans", None), (".a.b", Some("b")), ("Démo d'hiver.m4a", Some("m4a"))] {
        assert_eq!(extension(segment), attendue, "{segment}");
        assert_eq!(std::path::Path::new(segment).extension().and_then(|e| e.to_str()), attendue, "{segment}");
    }
}

#[test]
fn les_noms_des_formats() {
    let noms: Vec<&str> = FormatSonore::TOUS.iter().map(|f| f.nom()).collect();
    assert_eq!(noms, ["wav", "flac", "mp3", "m4a", "ogg", "aiff", "caf", "wma"]);
    assert_eq!([FormatTexte::Brut.nom(), FormatTexte::Markdown.nom(), FormatTexte::Chordpro.nom()], ["brut", "markdown", "chordpro"]);
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: les signatures et les extensions de l'import, au noyau`),
  pousser, ouvrir la PR en brouillon. Attendu à « Lint » de `verifier` : `unresolved import noyau::donnees::signature`.
- [ ] **Étape 3 : écrire le code.** La table en comparaisons de tranches (`starts_with`,
  `debut.get(4..8) == Some(b"ftyp".as_slice())` : `Some(b"ftyp")` serait un `Option<&[u8; 4]>`, d'un autre type que
  `get(4..8)` ; balayage P19), dans l'ordre de la table, la ligne MP3 avec son exception `0xFE` ; l'extension comparée
  par `eq_ignore_ascii_case` ; `TOUS` dans l'ordre de la déclaration. Aucune boucle `while` ni `loop`, et jamais
  `FormatAudio` (balayage P4). La doc de `reconnaitre` dit qu'un fichier qu'elle dit ignoré n'est jamais lu au-delà de
  `debut`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `signature.rs` `ok. 7 passed`, `statique.rs` du
  noyau vert ; `mutants` sans survivant sur le diff (le cas ID3 aux quatre octets non nuls tue les mutants des
  décalages de la taille *syncsafe* ; balayage P6).
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/signature.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/signature.rs .cargo/mutants.toml` (`feat: la table des signatures et des extensions, au noyau`).

### Tâche 2 : les en-têtes d'un WAV et d'un MIDI, le texte lisible et l'empreinte par tronçons

**Fichiers :** créer `crates/noyau/src/donnees/entete.rs`, `crates/noyau/tests/entete.rs` ; modifier
`crates/noyau/src/donnees/identifiants.rs`, `crates/noyau/tests/identifiants.rs`, `crates/noyau/src/donnees/mod.rs`
(`pub mod entete;`), `.cargo/mutants.toml`.

**`.cargo/mutants.toml`** (balayage P11) : `"crates/noyau/src/donnees/entete.rs"` dans `examine_globs`,
`"--test", "entete"` dans `additional_cargo_test_args` (`identifiants` y est déjà) ; le compte devient « Vingt-trois
modules » à `:2` et « ces vingt-trois modules » à la phrase qui suit la liste (`:33` sur `fe918aa`, plus bas des globs
ajoutés) ; l'énumération des lignes 2 à 7 gagne « les en-têtes d'un WAV et d'un MIDI, le texte lisible ». Aucune
boucle `while` ni `loop` dans `src/` : un parcours se borne par un itérateur fini.

**Interfaces — produit** (`noyau::donnees::entete`, `noyau::donnees::identifiants`) :

```rust
/// Les octets du début d'un WAV où l'établi cherche son en-tête (`fmt ` et l'en-tête de `data`) :
/// au-delà, le WAV est illisible (question 10).
pub const ENTETE_AU_PLUS: usize = 65_536;

/// Les faits d'un WAV lus dans son en-tête (§ 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaitsWav { pub codage: u16, pub canaux: u16, pub frequence: u32, pub bits: u16, pub trames: u64 }
impl FaitsWav {
    /// ⌊trames × 1 000 / fréquence⌋, calculé en `u128` (question 16).
    pub fn duree_ms(&self) -> u64;
}

/// Les faits d'un WAV : `debut`, ses `ENTETE_AU_PLUS` premiers octets au plus ; `taille`, celle du
/// fichier. `RIFF` ou `RF64` puis `WAVE` ; les morceaux depuis l'octet 12, chacun `<id><taille u32
/// LE><données>`, complété à un nombre pair ; `fmt ` (16 octets au moins) donne le codage, les
/// canaux, la fréquence, l'alignement d'un bloc et les bits ; `ds64` (RF64) donne la taille de
/// `data` quand celle de son morceau vaut `0xFFFF_FFFF` ; `data` donne les trames : sa taille,
/// bornée à ce que le fichier porte après son en-tête, divisée par l'alignement. `None` (illisible)
/// sans `fmt ` ou sans l'en-tête de `data` dans `debut`, ou si la fréquence ou l'alignement vaut 0,
/// ou si la durée passe `gestes::ENTIER_SUR`. Le parcours des morceaux est borné (`take`).
pub fn faits_wav(debut: &[u8], taille: u64) -> Option<FaitsWav>;

/// Les faits d'un MIDI (§ 30 · 9.2 ; L18).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaitsMidi { pub format: u16, pub pistes: u16, pub division: u16 }
impl FaitsMidi {
    /// `smf0`, `smf1` ou `smf2`.
    pub fn format_smf(&self) -> &'static str;
}

/// Les faits d'un MIDI, de ses 14 premiers octets : `MThd 00 00 00 06`, le format, les pistes et la
/// division, gros-boutistes, la division brute (SMPTE compris). `None` (illisible) sous 14 octets,
/// pour un format au-delà de 2, ou un format 0 qui n'a pas une seule piste.
pub fn faits_midi(debut: &[u8]) -> Option<FaitsMidi>;

/// Le contrôle d'un texte versé, tronçon par tronçon (question 10) : UTF-8, `EF BB BF` admis en
/// tête ; ou UTF-16 dont les deux premiers octets sont `FF FE` (petit-boutiste) ou `FE FF`
/// (gros-boutiste), même coupés entre deux tronçons, de longueur paire, chaque demi-paire de
/// substitution appariée. Un caractère coupé entre deux tronçons se reporte au suivant ; au plus
/// trois octets se reportent.
pub struct ControleDuTexte { /* l'encodage lu, le report */ }
impl ControleDuTexte {
    pub fn nouveau() -> Self;
    pub fn ajouter(&mut self, troncon: &[u8]);
    /// Vrai si tout ce qui a été ajouté forme un texte lisible, rien ne restant reporté.
    pub fn lisible(&self) -> bool;
}

// noyau::donnees::identifiants
/// L'empreinte calculée par tronçons : `Hacheur::nouveau()`, `ajouter` autant qu'il faut, puis
/// `finir`, qui rend ce que rendrait `empreinte` de leur suite.
pub struct Hacheur(/* sha2::Sha256 */);
impl Hacheur {
    pub fn nouveau() -> Self;
    pub fn ajouter(&mut self, octets: &[u8]);
    pub fn finir(self) -> String;
}
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/entete.rs`, et deux tests de plus dans
  `crates/noyau/tests/identifiants.rs`, `le_hacheur_par_troncons_egale_l_empreinte` et `le_hacheur_du_vide`, le
  fichier important `proptest::prelude::*` ; balayage P18). L'aide `wav` écrit un WAV, chaque entier petit-boutiste : `RIFF`, la
  taille du reste (4 + 24 + les morceaux + 8 + les données), `WAVE` ; `fmt ` et 16, puis le codage (1), les canaux,
  la fréquence, les octets par seconde (fréquence × alignement), l'alignement (canaux × bits / 8) et les bits ;
  d'éventuels morceaux (`avant_data`) ; puis `data`, sa taille (`data_declaree`, sinon celle des données) et ses
  données, `trames` × alignement octets à zéro. Un WAV de 8 000 trames, mono, 8 bits, fait ainsi 8 044 octets.

```rust
fn wav(frequence: u32, canaux: u16, bits: u16, trames: usize, data_declaree: Option<u32>, avant_data: &[u8]) -> Vec<u8>;
fn midi(format: u16, pistes: u16, division: u16) -> Vec<u8>; // MThd, 00 00 00 06, puis les trois, gros-boutistes
// `RF64`, `WAVE`, `ds64` (riff u64, data u64 = `trames`, compte u64), `fmt ` mono 8 bits, `data` de taille
// 0xFFFF_FFFF, puis ses `trames` octets à zéro.
fn rf64(frequence: u32, trames: usize) -> Vec<u8>;
// Le même en-tête, la taille de `data` dans `ds64` valant `data`, sans aucune donnée après lui (balayage P5).
fn rf64_sans_donnees(frequence: u32, data: u64) -> Vec<u8>;

#[test]
fn un_wav_donne_ses_faits_et_sa_duree() {
    let octets = wav(8_000, 1, 8, 8_000, None, &[]);
    assert_eq!(octets.len(), 8_044);
    let faits = faits_wav(&octets[..ENTETE_AU_PLUS.min(octets.len())], 8_044).expect("lisible");
    assert_eq!(faits, FaitsWav { codage: 1, canaux: 1, frequence: 8_000, bits: 8, trames: 8_000 });
    assert_eq!(faits.duree_ms(), 1_000);
    let stereo = wav(44_100, 2, 16, 1_000, None, &[]);
    assert_eq!(stereo.len(), 4_044);
    assert_eq!(faits_wav(&stereo, 4_044).map(|f| (f.trames, f.duree_ms())), Some((1_000, 22)));
}

#[test]
fn la_taille_de_data_se_borne_au_fichier() {
    let octets = wav(8_000, 1, 8, 4_000, Some(8_000), &[]);
    assert_eq!(octets.len(), 4_044);
    assert_eq!(faits_wav(&octets, 4_044).map(|f| (f.trames, f.duree_ms())), Some((4_000, 500)));
}

#[test]
fn un_morceau_avant_data_et_un_morceau_impair_se_sautent() {
    // `LIST` de 9 octets, complété d'un octet, entre `fmt ` et `data`.
    let liste = [b"LIST".as_slice(), &9u32.to_le_bytes(), b"INFOabcde", &[0]].concat();
    let octets = wav(8_000, 1, 8, 800, None, &liste);
    assert_eq!(faits_wav(&octets, octets.len() as u64).map(|f| f.trames), Some(800));
}

#[test]
fn un_rf64_prend_sa_taille_dans_ds64() {
    // `RF64`, `WAVE`, `ds64` (riff u64, data u64 = 800, compte u64), `fmt `, `data` de taille 0xFFFF_FFFF.
    let octets = rf64(8_000, 800);
    assert_eq!(faits_wav(&octets, octets.len() as u64).map(|f| f.trames), Some(800));
}

#[test]
fn la_duree_se_calcule_sans_debordement() {
    let faits = FaitsWav { codage: 1, canaux: 1, frequence: 48_000, bits: 8, trames: 20_000_000_000_000_000 };
    assert_eq!(faits.duree_ms(), 416_666_666_666_666);
}

#[test]
fn une_duree_au_dela_d_entier_sur_est_illisible() {
    // 1 000 Hz, mono, 8 bits : une trame vaut une milliseconde ; la taille du fichier, `u64::MAX`, ne borne rien.
    // `ENTIER_SUR` vient de `noyau::donnees::gestes` (balayage P5).
    let au_plus = rf64_sans_donnees(1_000, ENTIER_SUR);
    assert_eq!(faits_wav(&au_plus, u64::MAX).map(|f| f.duree_ms()), Some(ENTIER_SUR));
    let au_dela = rf64_sans_donnees(1_000, ENTIER_SUR + 1);
    assert_eq!(faits_wav(&au_dela, u64::MAX), None);
}

#[test]
fn un_en_tete_incoherent_est_illisible() {
    let bon = wav(8_000, 1, 8, 80, None, &[]);
    assert!(faits_wav(&bon[..36], 124).is_none()); // sans l'en-tête de `data`
    assert!(faits_wav(&bon[..20], 124).is_none()); // `fmt ` coupé
    let mut sans_frequence = bon.clone(); sans_frequence[24..28].copy_from_slice(&0u32.to_le_bytes());
    assert!(faits_wav(&sans_frequence, 124).is_none());
    let mut sans_bloc = bon.clone(); sans_bloc[32..34].copy_from_slice(&0u16.to_le_bytes());
    assert!(faits_wav(&sans_bloc, 124).is_none());
    assert!(faits_wav(b"RIFF\0\0\0\0AVI ", 12).is_none());
    // `data` au-delà des 65 536 premiers octets : un morceau `JUNK` de 70 000 octets avant lui.
    let loin = wav(8_000, 1, 8, 80, None, &[b"JUNK".as_slice(), &70_000u32.to_le_bytes(), &vec![0; 70_000]].concat());
    assert!(faits_wav(&loin[..ENTETE_AU_PLUS], loin.len() as u64).is_none());
}

#[test]
fn un_wav_aux_morceaux_sans_fin_est_illisible() {
    // Des morceaux de taille nulle jusqu'au bout des 65 536 octets, sans `data` : le parcours finit.
    // 12 + 8 191 × 8 = 65 540 octets, sans boucle écrite à la main (balayage P18).
    let mut octets = b"RIFF\0\0\0\0WAVE".to_vec();
    octets.extend(b"JUNK\0\0\0\0".repeat(8_191));
    assert_eq!(octets.len(), 65_540);
    assert!(faits_wav(&octets, octets.len() as u64).is_none());
}

#[test]
fn un_midi_donne_son_format_et_sa_division() {
    assert_eq!(faits_midi(&midi(1, 3, 480)), Some(FaitsMidi { format: 1, pistes: 3, division: 480 }));
    assert_eq!(faits_midi(&midi(1, 3, 480)).map(|f| f.format_smf()), Some("smf1"));
    assert_eq!(faits_midi(&midi(0, 1, 0xE728)).map(|f| (f.format_smf(), f.division)), Some(("smf0", 59_176)));
    assert_eq!(faits_midi(&midi(2, 4, 96)).map(|f| f.format_smf()), Some("smf2"));
    assert_eq!(faits_midi(&midi(3, 1, 96)), None);
    assert_eq!(faits_midi(&midi(0, 2, 96)), None);
    assert_eq!(faits_midi(&midi(0, 0, 96)), None);
    assert_eq!(faits_midi(&midi(1, 3, 480)[..13]), None);
}

fn lisible(troncons: &[&[u8]]) -> bool {
    let mut controle = ControleDuTexte::nouveau();
    for t in troncons { controle.ajouter(t); }
    controle.lisible()
}

#[test]
fn un_texte_utf8_avec_ou_sans_bom_est_lisible() {
    assert!(lisible(&["Pont du soir\n".as_bytes()]));
    assert!(lisible(&[b"\xEF\xBB\xBF", "Pont du soir".as_bytes()]));
    assert!(lisible(&[]));
}

#[test]
fn un_texte_utf16_avec_bom_est_lisible() {
    assert!(lisible(&[&[0xFF, 0xFE, 0x6C, 0x00, 0x61, 0x00]]));
    assert!(lisible(&[&[0xFE, 0xFF, 0x00, 0x6C, 0x00, 0x61]]));
    assert!(lisible(&[&[0xFF, 0xFE, 0x3D, 0xD8], &[0x00, 0xDE]])); // U+1F600, coupé entre deux tronçons
    assert!(lisible(&[&[0xFF], &[0xFE, 0x6C, 0x00]]));             // le BOM coupé : un `Read` rend un octet (balayage P18)
    assert!(lisible(&[&[0xFE], &[0xFF, 0x00, 0x6C]]));
}

#[test]
fn un_texte_ni_utf8_ni_utf16_avec_bom_est_illisible() {
    assert!(!lisible(&[b"\xE9t\xE9"]));                         // « été » en Latin-1
    assert!(!lisible(&[&[0xFF, 0xFE, 0x6C]]));                  // UTF-16 de longueur impaire
    assert!(!lisible(&[&[0xFF, 0xFE, 0x00, 0xD8]]));            // demi-paire haute seule
    assert!(!lisible(&[&[0xFF, 0xFE, 0x00, 0xDC, 0x6C, 0x00]])); // demi-paire basse seule
}

#[test]
fn un_texte_coupe_au_milieu_d_un_caractere_reste_lisible() {
    assert!(lisible(&[&[0xC3], &[0xA9]]));         // « é », coupé
    assert!(!lisible(&[b"pont ", &[0xC3]]));       // coupé au bout du texte
    assert!(lisible(&[&[0xF0, 0x9F], &[0x98], &[0x80]]));
}

// crates/noyau/tests/identifiants.rs
proptest! {
    #[test]
    fn le_hacheur_par_troncons_egale_l_empreinte(octets in proptest::collection::vec(any::<u8>(), 0..4_096), coupes in proptest::collection::vec(0usize..4_096, 0..8)) {
        // Les octets coupés aux positions `coupes`, rangées et bornées : la suite des tronçons.
        prop_assert_eq!(par_troncons(&octets, &coupes), empreinte(&octets));
    }
}

#[test]
fn le_hacheur_du_vide() {
    assert_eq!(Hacheur::nouveau().finir(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: les en-têtes d'un WAV et d'un MIDI, le texte lisible, au
  noyau`), pousser. Attendu à « Lint » : `unresolved import noyau::donnees::entete`.
- [ ] **Étape 3 : écrire le code.** `empreinte` passe par `Hacheur` ; `faits_wav` borne son parcours à
  `debut.len() / 8 + 1` morceaux, et compare la durée à `ENTIER_SUR` ; les entiers de largeur fixe se lisent par
  `u16::from_le_bytes` et `u32::from_le_bytes` pour le WAV (`u64::from_le_bytes` pour `ds64`), par `u16::from_be_bytes`
  pour le MIDI, jamais par décalages, dont les mutants survivraient sur des octets de poids fort nuls (balayage P6) ;
  `ControleDuTexte` s'appuie sur `str::from_utf8` et `Utf8Error::error_len` pour l'UTF-8, et sur `char::decode_utf16`
  pour l'UTF-16, et garde un premier octet `FF` ou `FE` en report jusqu'au second (balayage P18) ; aucune boucle
  `while` ni `loop` ; aucun mot des contrôles statiques (`io::`, `File`) dans les docs.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `entete.rs` `ok. 13 passed`, `identifiants.rs`
  vert ; `mutants` sans survivant sur le diff (la borne `ENTIER_SUR` gardée par
  `une_duree_au_dela_d_entier_sur_est_illisible` ; balayage P5).
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/entete.rs crates/noyau/src/donnees/identifiants.rs
  crates/noyau/src/donnees/mod.rs crates/noyau/tests/entete.rs crates/noyau/tests/identifiants.rs .cargo/mutants.toml`
  (`feat: les en-têtes, le texte lisible et l'empreinte par tronçons, au noyau`).

### Tâche 3 : le manifeste, le rapport d'import, ses comptes et la place libre

**Fichiers :** créer `crates/noyau/src/donnees/rapport.rs`, `crates/noyau/tests/rapport.rs` ; modifier
`crates/stockage/tests/references.rs`, `crates/noyau/src/donnees/mod.rs` (`pub mod rapport;`), `.cargo/mutants.toml`.

**`.cargo/mutants.toml`** (balayage P11) : `"crates/noyau/src/donnees/rapport.rs"` dans `examine_globs`,
`"--test", "rapport"` dans `additional_cargo_test_args` ; le compte devient « Vingt-quatre modules » à `:2` et « ces
vingt-quatre modules » à la phrase qui suit la liste (`:33` sur `fe918aa`, plus bas des globs ajoutés) ; l'énumération
des lignes 2 à 7 gagne « le manifeste, le rapport d'import et la place libre ». Aucune boucle `while` ni `loop` dans
`src/` : un parcours se borne par un itérateur fini.

**Interfaces — produit** (`noyau::donnees::rapport`) :

```rust
/// Le manifeste d'une prise en stems (question 19) : les empreintes distinctes de ses stems,
/// rangées dans l'ordre de leurs octets, chacune suivie d'un LF (`0x0A`), la dernière comprise.
pub fn manifeste(empreintes: &[&str]) -> Vec<u8>;

/// Un compte du rapport : des fichiers, et leurs octets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Compte { pub fichiers: u64, pub octets: u64 }

/// Les comptes d'un import (question 25) : vus = versés + dédoublonnés + ignorés ; illisibles et
/// sans nom sont des parts des versés.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Comptes {
    pub vus: Compte, pub verses: Compte, pub dedoublonnes: Compte,
    pub illisibles: Compte, pub ignores: Compte, pub sans_nom: Compte,
}

/// Un fichier vu par l'import, de rang `rang` (le premier fichier du parcours a le rang 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FichierDeLImport {
    /// Ignoré : ni lu au-delà de sa signature, ni versé ; un lien, une jonction compris.
    Ignore { rang: u64, octets: u64 },
    /// Lu en entier : son empreinte, s'il est illisible, s'il n'a pas de nom en UTF-8.
    Lu { rang: u64, octets: u64, empreinte: String, illisible: bool, sans_nom: bool },
}

/// Les comptes de `fichiers`, `connues` les empreintes des captations du fonds à la tête de
/// l'ouverture de l'import : un fichier lu est versé si son contenu n'était pas connu et qu'aucun
/// fichier lu de rang plus petit n'a le même contenu ; dédoublonné sinon. L'ordre de `fichiers`
/// ne compte pas : une reprise rend les comptes d'un import fait d'une traite (question 2).
pub fn comptes(fichiers: &[FichierDeLImport], connues: &BTreeSet<String>) -> Comptes;

/// Une origine qu'un import change : l'empreinte du contenu, et l'élément ajouté à son ensemble,
/// `main` ou `generee:<nom>`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrigineChangee { pub empreinte: String, pub element: String }

/// Le rapport d'import, à l'octet (question 25) : en UTF-8, chaque ligne finie par LF,
/// `pupitre-rapport-d-import 1`, l'ULID de l'import, puis `vus=`, `verses=`, `dedoublonnes=`,
/// `illisibles=`, `ignores=`, `sans-nom=`, chacun suivi du nombre de fichiers, d'une espace et de
/// leurs octets ; puis `origine=<empreinte> <élément>` pour chaque origine changée, dans l'ordre
/// (empreinte, élément) ; puis `fichier-sans-nom=<rang>` pour chaque fichier lu sans nom, versé ou
/// dédoublonné, dans l'ordre des rangs.
pub fn rapport(import: &str, fichiers: &[FichierDeLImport], connues: &BTreeSet<String>, origines: &[OrigineChangee]) -> Vec<u8>;

/// Un rapport d'import relu : son import, ses comptes, le nombre de ses lignes `origine=` et les
/// rangs de ses lignes `fichier-sans-nom=`, dans leur ordre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RapportLu { pub import: String, pub comptes: Comptes, pub origines_changees: u64, pub fichiers_sans_nom: Vec<u64> }

/// L'inverse de `rapport` (balayage P14) : `None` si la première ligne n'est pas
/// `pupitre-rapport-d-import 1` (une autre version), si une ligne manque ou si une ligne est mal
/// formée. Les services n'en relisent pas le format : ils appellent cette règle (tâche 13).
pub fn lire_le_rapport(octets: &[u8]) -> Option<RapportLu>;

/// Les octets laissés pour chaque fichier à verser, en plus de ses octets (question 8).
pub const MARGE_PAR_FICHIER: u64 = 4_096;

/// La place qui manque : la taille à verser, la place libre, et ce qui manque.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManqueDePlace { pub a_verser: u64, pub libre: u64, pub manque: u64 }

/// On verse si `a_verser` + 2 × `poids_du_fonds` + `MARGE_PAR_FICHIER` × `fichiers` tient dans
/// `libre`, calculé en `u128` ; sinon le manque, borné à `u64::MAX`.
pub fn place(a_verser: u64, fichiers: u64, poids_du_fonds: u64, libre: u64) -> Result<(), ManqueDePlace>;

/// `octets` en Go décimaux (10⁹), à une décimale, la virgule en séparateur : « 4,0 Go » ; arrondi
/// au dixième supérieur si `vers_le_haut`, inférieur sinon.
pub fn go(octets: u64, vers_le_haut: bool) -> String;
// impl fmt::Display for ManqueDePlace : « La taille à verser, <go(a_verser, haut)>, passe la place
// libre, <go(libre, bas)> : il manque <go(manque, haut)>. »
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/rapport.rs`, et les témoins dans
  `crates/stockage/tests/references.rs`). Les valeurs viennent du passage de LE-6, recalculées par la relecture ;
  chaque empreinte est celle du texte UTF-8 de son commentaire. Les tests du noyau fixent les octets et leur longueur ;
  les empreintes du manifeste et du rapport témoins ne s'écrivent que dans `references.rs`, « le seul des empreintes
  de référence » (`crates/stockage/tests/references.rs:6` ; balayage P17) :

```rust
const VOIX: &str = "1c0f0638f51a94205014847b526474bdd04c8c00ad2a02ac4dbeba88d5a3d425";      // « stem inventé : voix »
const BATTERIE: &str = "29b0ea58a7431fa5865927c2c38540e99b5ac828bcdd06391038e6bdf56c739f";  // « stem invente : batterie »
const BASSE: &str = "c7fabc575b006385e571f7b2857d3239dcf042211a610b60903585fe9cc99e17";     // « stem invente : basse »

#[test]
fn le_manifeste_temoin() {
    let octets = manifeste(&[VOIX, BATTERIE, BASSE, VOIX]);
    assert_eq!(octets.len(), 195);
    assert_eq!(octets, format!("{VOIX}\n{BATTERIE}\n{BASSE}\n").into_bytes());
    assert_eq!(manifeste(&[VOIX]), format!("{VOIX}\n").into_bytes());
}

fn e(n: u64) -> String { format!("{n:064x}") }
fn lu(rang: u64, octets: u64, contenu: u64) -> FichierDeLImport {
    FichierDeLImport::Lu { rang, octets, empreinte: e(contenu), illisible: false, sans_nom: false }
}

#[test]
fn les_comptes_de_cap03_1() {
    // 80 fichiers, dont 3 doublons de contenu des rangs 1 à 3.
    let mut fichiers: Vec<_> = (1..=77).map(|r| lu(r, 1_000 + r, r)).collect();
    fichiers.extend((1..=3).map(|r| lu(77 + r, 1_000 + r, r)));
    let c = comptes(&fichiers, &BTreeSet::new());
    assert_eq!(c.vus, Compte { fichiers: 80, octets: 83_009 });
    assert_eq!(c.verses, Compte { fichiers: 77, octets: 80_003 });
    assert_eq!(c.dedoublonnes, Compte { fichiers: 3, octets: 3_006 });
    assert_eq!((c.illisibles, c.ignores, c.sans_nom), (Compte::default(), Compte::default(), Compte::default()));
    fichiers.reverse();
    assert_eq!(comptes(&fichiers, &BTreeSet::new()), c, "l'ordre ne compte pas");
}

#[test]
fn un_contenu_connu_a_l_ouverture_est_dedoublonne() {
    let connues = BTreeSet::from([e(1)]);
    let c = comptes(&[lu(1, 10, 1), lu(2, 20, 2)], &connues);
    assert_eq!((c.verses, c.dedoublonnes), (Compte { fichiers: 1, octets: 20 }, Compte { fichiers: 1, octets: 10 }));
}

#[test]
fn un_illisible_en_deux_copies_compte_une_fois() {
    let illisible = |rang| FichierDeLImport::Lu { rang, octets: 50, empreinte: e(9), illisible: true, sans_nom: false };
    let c = comptes(&[illisible(1), illisible(2)], &BTreeSet::new());
    assert_eq!(c.illisibles, Compte { fichiers: 1, octets: 50 });
    assert_eq!(c.dedoublonnes, Compte { fichiers: 1, octets: 50 });
}

const IMPORT_TEMOIN: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ";

/// Les fichiers du rapport témoin, partagés par `le_rapport_temoin` et `le_rapport_se_relit`.
fn fichiers_du_temoin() -> Vec<FichierDeLImport> {
    vec![
        lu(1, 650, 1),
        FichierDeLImport::Ignore { rang: 2, octets: 300 },
        FichierDeLImport::Lu { rang: 3, octets: 50, empreinte: e(3), illisible: true, sans_nom: false },
        FichierDeLImport::Lu { rang: 4, octets: 200, empreinte: e(4), illisible: false, sans_nom: true },
        lu(5, 100, 5),
    ]
}

/// L'origine changée du rapport témoin.
fn origines_du_temoin() -> [OrigineChangee; 1] {
    [OrigineChangee {
        empreinte: "3ab628c95bde7c17cc00f4931000a16a5425d8deb0e5e91fbc055731ac76cd3b".into(), // « une prise de test »
        element: "generee:Atelier invent\u{e9}".into(),
    }]
}

#[test]
fn le_rapport_temoin() {
    let octets = rapport(IMPORT_TEMOIN, &fichiers_du_temoin(), &BTreeSet::from([e(5)]), &origines_du_temoin());
    assert_eq!(String::from_utf8(octets.clone()).unwrap(), "pupitre-rapport-d-import 1\n01HF7YAT00ZZZZZZZZZZZZZZZZ\n\
        vus=5 1300\nverses=3 900\ndedoublonnes=1 100\nillisibles=1 50\nignores=1 300\nsans-nom=1 200\n\
        origine=3ab628c95bde7c17cc00f4931000a16a5425d8deb0e5e91fbc055731ac76cd3b generee:Atelier invent\u{e9}\n\
        fichier-sans-nom=4\n");
    assert_eq!(octets.len(), 259);
}

#[test]
fn le_rapport_se_relit() {
    // L'inverse de `rapport`, sur le rapport témoin (balayage P14).
    let connues = BTreeSet::from([e(5)]);
    let octets = rapport(IMPORT_TEMOIN, &fichiers_du_temoin(), &connues, &origines_du_temoin());
    let relu = lire_le_rapport(&octets).expect("le rapport témoin se relit");
    assert_eq!(relu, RapportLu {
        import: IMPORT_TEMOIN.to_string(),
        comptes: comptes(&fichiers_du_temoin(), &connues),
        origines_changees: 1,
        fichiers_sans_nom: vec![4],
    });
    let texte = String::from_utf8(octets).unwrap();
    let autre = texte.replacen("pupitre-rapport-d-import 1", "pupitre-rapport-d-import 2", 1);
    assert_eq!(lire_le_rapport(autre.as_bytes()), None, "une autre version");
    assert_eq!(lire_le_rapport(b"pupitre-rapport-d-import 1\n"), None, "des lignes manquent");
    let mal_formee = texte.replacen("vus=5 1300", "vus=cinq 1300", 1);
    assert_eq!(lire_le_rapport(mal_formee.as_bytes()), None, "une ligne mal formée");
}

#[test]
fn la_place_manque_se_dit_en_go_decimaux() {
    let manque = place(4_000_000_000, 2, 0, 1_000_000_000).unwrap_err();
    assert_eq!(manque, ManqueDePlace { a_verser: 4_000_000_000, libre: 1_000_000_000, manque: 3_000_008_192 });
    assert_eq!(manque.to_string(), "La taille à verser, 4,0 Go, passe la place libre, 1,0 Go : il manque 3,1 Go.");
    assert_eq!(place(1_000, 1, 10, 1_000 + 20 + 4_096), Ok(()));
    assert_eq!(place(1_000, 1, 10, 1_000 + 20 + 4_095).unwrap_err().manque, 1);
    assert_eq!(place(u64::MAX, 1, 0, u64::MAX).unwrap_err().manque, 4_096);
}

#[test]
fn les_go_decimaux_a_une_decimale() {
    for (octets, haut, bas) in [(4_000_000_000, "4,0 Go", "4,0 Go"), (1_234_567_890, "1,3 Go", "1,2 Go"),
                                (99_999_999, "0,1 Go", "0,0 Go"), (100_000_000, "0,1 Go", "0,1 Go"),
                                (100_000_001, "0,2 Go", "0,1 Go"), (0, "0,0 Go", "0,0 Go")] {
        assert_eq!((go(octets, true).as_str(), go(octets, false).as_str()), (haut, bas), "{octets}");
    }
}

// crates/stockage/tests/references.rs : son en-tête (`//!`) nomme aussi le manifeste et le rapport témoins.
const MANIFESTE_TEMOIN: &str = "340999f94394a8aaa1b93777c67d6939b70fe427a221ff7478ab78dd3f5d0caa";
const RAPPORT_TEMOIN: &str = "6d161268e13683005ded38007cf7d27412723752529170cf929390406038ff76";
#[test]
fn le_manifeste_et_le_rapport_temoins_gardent_leurs_empreintes() {
    // Le manifeste de [VOIX, BATTERIE, BASSE, VOIX], et le rapport des fichiers et de l'origine de `le_rapport_temoin`,
    // rebâtis par `noyau::donnees::rapport` (les trois empreintes des stems recalculées ici par `empreinte` de leur
    // texte), ont ces empreintes.
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: le manifeste, le rapport d'import et la place libre, au noyau`),
  pousser. Attendu à « Lint » : `unresolved import noyau::donnees::rapport`.
- [ ] **Étape 3 : écrire le code.** `comptes` range les fichiers lus par (empreinte, rang) dans un `BTreeMap` ;
  `rapport` appelle `comptes` ; `lire_le_rapport` lit les lignes dans l'ordre que `rapport` écrit, par `split_once('=')`
  et `parse`, sans boucle `while` ni `loop` ; `place` et `go` calculent en `u128`. La doc du module dit que le rapport
  n'est pas un message montré : il nomme l'ULID et des empreintes, que l'écran ne montre pas (tâche 22). L'en-tête
  (`//!`) de `crates/stockage/tests/references.rs` nomme le manifeste et le rapport témoins à côté de la genèse et de
  la teneur (balayage P17).
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `rapport.rs` `ok. 8 passed`, `references.rs`
  vert ; dans `windows`, `references.rs` vert ; `mutants` sans survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/rapport.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/rapport.rs crates/stockage/tests/references.rs .cargo/mutants.toml` (`feat: le manifeste, le
  rapport d'import et la place libre, au noyau`).

### Tâche 4 : l'origine déclarée, le premier nom vu et la portée d'une déclaration

**Fichiers :** créer `crates/noyau/src/donnees/origine.rs`, `crates/noyau/tests/origine.rs` ; modifier
`crates/noyau/src/donnees/mod.rs` (`pub mod origine;`), `.cargo/mutants.toml`.

**`.cargo/mutants.toml`** (balayage P11) : `"crates/noyau/src/donnees/origine.rs"` dans `examine_globs`,
`"--test", "origine"` dans `additional_cargo_test_args` ; le compte devient « Vingt-cinq modules » à `:2` et « ces
vingt-cinq modules » à la phrase qui suit la liste (`:33` sur `fe918aa`, plus bas des globs ajoutés) ; l'énumération des
lignes 2 à 7 gagne « l'origine déclarée, le premier nom vu et la portée ». Aucune boucle `while` ni `loop` dans
`src/` : un parcours se borne par un itérateur fini.

**Interfaces — produit** (`noyau::donnees::origine`) :

```rust
/// L'ensemble des origines d'un contenu, tel qu'il se lit sous `origines_declarees/<e>/…`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OriginesLues { pub main: bool, pub generees: BTreeSet<String> }

/// L'origine la plus forte d'un contenu (CAP-03) : générée, avec chacun de ses noms ; puis de la
/// main de Sacha ; `None` vaut `—`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origine { Main, Generee(BTreeSet<String>) }
pub fn la_plus_forte(lues: &OriginesLues) -> Option<Origine>;

/// Un élément d'origine qu'une déclaration ajoute.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ajout { Main, Generee(String) }
impl Ajout {
    /// `main`, ou `generee:<nom>` : l'élément que dit le rapport.
    pub fn element(&self) -> String;
}

/// Ce que `ajout` change à `lues` : `None` s'il y est déjà, ou si c'est `Main` sur un contenu
/// généré (question 22 : fait, sans rien changer).
pub fn ajout_d_origine(lues: &OriginesLues, ajout: &Ajout) -> Option<Ajout>;

/// La déclaration d'un dossier, pendant l'import (question 22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration { Stems, Origine(Ajout) }
impl Declaration {
    /// `stems`, `main` ou `generee:<nom>` : la valeur de `imports/<i>/declaration/<chemin>`.
    pub fn valeur(&self) -> String;
    /// L'inverse de `valeur` ; `None` pour toute autre chaîne, `generee:` sans nom compris.
    pub fn lire(valeur: &str) -> Option<Declaration>;
}

/// Ce que les déclarations de l'import donnent au fichier `chemin` : l'origine du plus proche de
/// ses dossiers déclarés, du sien au dossier versé (`""`) compris, qui déclare une origine ; et le
/// jeu de stems, son propre dossier s'il est déclaré `stems` (question 17), jamais un dossier plus
/// haut. Une déclaration `stems` n'arrête pas l'origine d'un dossier plus haut. Seuls des dossiers
/// se déclarent : le chemin du fichier lui-même n'est jamais une clé lue (balayage P22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationDuFichier { pub origine: Option<Ajout>, pub jeu: Option<String> }
pub fn declaration_du_fichier(chemin: &str, declarations: &BTreeMap<String, Declaration>) -> DeclarationDuFichier;

/// Le premier nom vu (question 20) : de `noms`, des couples (chemin, ULID de l'import qui l'a vu
/// le premier), une valeur par réplique comprise, celui dont l'heure de l'ULID (ses dix premiers
/// caractères) est la plus petite, puis le chemin, dans l'ordre de ses octets.
pub fn premier_nom_vu<'a>(noms: impl IntoIterator<Item = (&'a str, &'a str)>) -> Option<&'a str>;

/// Vrai si `chemin` est sous le préfixe `prefixe`, par segments entiers, octet pour octet
/// (question 23) ; le préfixe vide prend tout.
pub fn sous_le_prefixe(chemin: &str, prefixe: &str) -> bool;
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/origine.rs`) :

```rust
const A: &str = "01HF7YAT000000000000000001"; // un import
const B: &str = "01HF7YAT010000000000000001"; // un import d'une milliseconde plus tard
const A_BIS: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ"; // la même heure que A, un autre aléa

fn lues(main: bool, generees: &[&str]) -> OriginesLues {
    OriginesLues { main, generees: generees.iter().map(|n| n.to_string()).collect() }
}

#[test]
fn la_plus_forte_generee_puis_main_puis_rien() {
    assert_eq!(la_plus_forte(&lues(false, &[])), None);
    assert_eq!(la_plus_forte(&lues(true, &[])), Some(Origine::Main));
    assert_eq!(la_plus_forte(&lues(true, &["Atelier"])), Some(Origine::Generee(BTreeSet::from(["Atelier".to_string()]))));
    assert_eq!(la_plus_forte(&lues(false, &["Atelier", "Forge"])).map(|o| match o { Origine::Generee(n) => n.len(), _ => 0 }), Some(2));
}

#[test]
fn main_sur_une_generee_ne_change_rien() {
    let gen = |n: &str| Ajout::Generee(n.to_string());
    assert_eq!(ajout_d_origine(&lues(false, &["Atelier"]), &Ajout::Main), None);
    assert_eq!(ajout_d_origine(&lues(false, &[]), &Ajout::Main), Some(Ajout::Main));
    assert_eq!(ajout_d_origine(&lues(true, &[]), &Ajout::Main), None);
    assert_eq!(ajout_d_origine(&lues(true, &[]), &gen("Atelier")), Some(gen("Atelier")));
    assert_eq!(ajout_d_origine(&lues(false, &["Atelier"]), &gen("Atelier")), None);
    assert_eq!(ajout_d_origine(&lues(false, &["Atelier"]), &gen("Forge")), Some(gen("Forge")));
    assert_eq!((Ajout::Main.element(), gen("Atelier inventé").element()), ("main".to_string(), "generee:Atelier inventé".to_string()));
}

#[test]
fn les_declarations_se_lisent_et_s_ecrivent() {
    for valeur in ["stems", "main", "generee:Atelier inventé", "generee:a:b"] {
        assert_eq!(Declaration::lire(valeur).map(|d| d.valeur()), Some(valeur.to_string()));
    }
    for valeur in ["generee:", "aucune", "Main", "", "stems "] { assert_eq!(Declaration::lire(valeur), None, "{valeur}"); }
}

#[test]
fn la_declaration_d_un_fichier_vient_de_son_plus_proche_dossier_declare() {
    let declarations = BTreeMap::from([
        (String::new(), Declaration::Origine(Ajout::Generee("Atelier".into()))),
        ("Démos".to_string(), Declaration::Origine(Ajout::Main)),
        ("Démos/Jeu A".to_string(), Declaration::Stems),
    ]);
    assert_eq!(declaration_du_fichier("Démos/Jeu A/voix.wav", &declarations),
               DeclarationDuFichier { origine: Some(Ajout::Main), jeu: Some("Démos/Jeu A".into()) });
    assert_eq!(declaration_du_fichier("Démos/Jeu A/sous/x.wav", &declarations),
               DeclarationDuFichier { origine: Some(Ajout::Main), jeu: None });
    assert_eq!(declaration_du_fichier("Autre/y.wav", &declarations),
               DeclarationDuFichier { origine: Some(Ajout::Generee("Atelier".into())), jeu: None });
    assert_eq!(declaration_du_fichier("z.wav", &BTreeMap::new()), DeclarationDuFichier { origine: None, jeu: None });
    let jeu_seul = BTreeMap::from([(String::new(), Declaration::Stems)]);
    assert_eq!(declaration_du_fichier("voix.wav", &jeu_seul).jeu, Some(String::new()));
}

#[test]
fn le_premier_nom_vu_est_le_plus_petit_couple() {
    assert_eq!(premier_nom_vu([("Démos/b.wav", A), ("Démos/a.wav", A), ("Autre/z.wav", B)]), Some("Démos/a.wav"));
    assert_eq!(premier_nom_vu([("A.wav", B), ("Z.wav", A)]), Some("Z.wav"));
    assert_eq!(premier_nom_vu([("b.wav", A), ("a.wav", A_BIS)]), Some("a.wav"), "même heure : le chemin décide");
    assert_eq!(premier_nom_vu([]), None);
}

#[test]
fn la_portee_par_segments_entiers() {
    assert!(sous_le_prefixe("Démos/a.wav", "Démos"));
    assert!(sous_le_prefixe("Démos/sous/a.wav", "Démos"));
    assert!(sous_le_prefixe("Démos", "Démos"));
    assert!(!sous_le_prefixe("Démos 2015/a.wav", "Démos"));
    assert!(!sous_le_prefixe("démos/a.wav", "Démos"));
    assert!(!sous_le_prefixe("De\u{301}mos/a.wav", "D\u{e9}mos"), "octet pour octet, sans normaliser");
    assert!(sous_le_prefixe("x.wav", ""));
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: l'origine déclarée, le premier nom vu et la portée, au noyau`),
  pousser. Attendu à « Lint » : `unresolved import noyau::donnees::origine`.
- [ ] **Étape 3 : écrire le code.** Les ancêtres d'un chemin par `rsplit_once('/')` répété sur une plage bornée au
  nombre de ses `/` plus un, jamais par une boucle `while` ni `loop` ; `premier_nom_vu` par `min_by_key` sur
  `(ulid.get(..10).unwrap_or(ulid), chemin.as_bytes())`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `origine.rs` `ok. 6 passed` ; `mutants` sans
  survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/origine.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/origine.rs .cargo/mutants.toml` (`feat: l'origine déclarée, le premier nom vu et la portée, au
  noyau`).

### Tâche 5 : les stems : le type proposé, le MIDI d'un stem, le jeu proposé et la durée

**Fichiers :** créer `crates/noyau/src/donnees/stems.rs`, `crates/noyau/tests/stems.rs` ; modifier
`crates/noyau/src/donnees/mod.rs` (`pub mod stems;`), `.cargo/mutants.toml`.

**`.cargo/mutants.toml`** (balayage P11) : `"crates/noyau/src/donnees/stems.rs"` dans `examine_globs`,
`"--test", "stems"` dans `additional_cargo_test_args` ; le compte devient « Vingt-six modules » à `:2` et « ces
vingt-six modules » à la phrase qui suit la liste (`:33` sur `fe918aa`, plus bas des globs ajoutés) ; l'énumération des
lignes 2 à 7 gagne « les stems : le type proposé, le MIDI d'un stem, le jeu proposé et la durée ». Aucune boucle
`while` ni `loop` dans `src/` : un parcours se borne par un itérateur fini.

**Interfaces — produit** (`noyau::donnees::stems`) :

```rust
/// Le type d'un stem, une liste fermée (question 18 ; § 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeDeStem { Voix, Choeurs, Batterie, Percussions, Basse, Guitare, Clavier, Cordes, Cuivres, Synthe, Effets, Autre }
impl TypeDeStem {
    pub const TOUS: [TypeDeStem; 12];
    /// La valeur stockée : `voix`, `choeurs`, `batterie`, `percussions`, `basse`, `guitare`,
    /// `clavier`, `cordes`, `cuivres`, `synthe`, `effets`, `autre`.
    pub fn nom(self) -> &'static str;
    /// Ce que l'écran écrit : « voix », « chœurs », « batterie », « percussions », « basse »,
    /// « guitare », « clavier », « cordes », « cuivres », « synthé », « effets », « autre ».
    pub fn libelle(self) -> &'static str;
    pub fn lire(nom: &str) -> Option<TypeDeStem>;
}

/// Les mots, français ou anglais, repliés, qui proposent un type ; rangés, chacun une fois.
pub const MOTS_DES_TYPES: &[(&str, TypeDeStem)];

/// Les mots d'un nom de fichier : son dernier segment, sans son extension (`signature::extension`),
/// replié (`repli::replier`), coupé à chaque caractère qui n'est ni lettre ni chiffre
/// (`char::is_alphanumeric`), les morceaux vides écartés. Plus large que `repli::mots`, qui garde
/// `'` et `-` dans un mot : un nom de fichier lie ses mots par `_`, `-` ou `.`.
pub fn mots_du_nom(chemin: &str) -> Vec<String>;

/// Le type proposé d'un stem (question 18) : celui du premier mot de `mots_du_nom` que
/// `MOTS_DES_TYPES` porte, cherché par dichotomie ; `Autre` sans nom ou sans mot qui réponde.
pub fn type_propose(premier_nom_vu: Option<&str>) -> TypeDeStem;

/// Le stem qu'accompagne le MIDI `midi` (question 17) : de `stems`, des chemins du même dossier
/// rangés dans l'ordre de leurs octets, le premier dont le dernier segment, sans son extension,
/// égale celui du MIDI, octet pour octet.
pub fn stem_du_midi<'a>(midi: &str, stems: &[&'a str]) -> Option<&'a str>;

/// Un WAV tel que l'inspection l'a lu, sans empreinte : sa taille, les octets lus de son début (au
/// plus `ENTETE_AU_PLUS`), ses trames et sa fréquence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavInspecte<'a> { pub taille: u64, pub debut: &'a [u8], pub trames: u64, pub frequence: u32 }

/// Vrai si au moins deux WAV distincts ont le même nombre de trames à la même fréquence (question
/// 16) ; deux WAV sont distincts si leurs tailles, ou les octets lus de leur début, diffèrent
/// (question 37) : aucune empreinte ne se calcule pour une proposition.
pub fn jeu_propose(wav: &[WavInspecte<'_>]) -> bool;

/// La durée d'une prise en stems : celle de son stem le plus long ; `None` dès qu'un stem n'a pas
/// la sienne, ou sans stem.
pub fn duree_de_la_prise_en_stems(stems: &[Option<u64>]) -> Option<u64>;
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/stems.rs`). `ATTENDUS` est la table, recopiée, rangée dans
  l'ordre des octets. La copie est voulue : `cargo-mutants` ne mute pas une constante, et seule cette copie, comparée
  en entier à `MOTS_DES_TYPES`, garde la table (« Outils et cas limites » ; balayage P21) :

```rust
use TypeDeStem::*;
const ATTENDUS: &[(&str, TypeDeStem)] = &[
    ("backing", Choeurs), ("bass", Basse), ("basse", Basse), ("batterie", Batterie), ("brass", Cuivres),
    ("cello", Cordes), ("chant", Voix), ("choeur", Choeurs), ("choeurs", Choeurs), ("choir", Choeurs),
    ("clavier", Clavier), ("claviers", Clavier), ("cordes", Cordes), ("cuivres", Cuivres), ("drum", Batterie),
    ("drums", Batterie), ("effects", Effets), ("effet", Effets), ("effets", Effets), ("fx", Effets),
    ("guitar", Guitare), ("guitare", Guitare), ("guitares", Guitare), ("guitars", Guitare), ("keyboard", Clavier),
    ("keys", Clavier), ("organ", Clavier), ("orgue", Clavier), ("pad", Synthe), ("perc", Percussions),
    ("percussion", Percussions), ("percussions", Percussions), ("piano", Clavier), ("strings", Cordes),
    ("synth", Synthe), ("synthe", Synthe), ("synths", Synthe), ("trompette", Cuivres), ("trumpet", Cuivres),
    ("violin", Cordes), ("violon", Cordes), ("vocal", Voix), ("vocals", Voix), ("voice", Voix), ("voix", Voix),
    ("vox", Voix),
];

#[test]
fn la_table_des_mots_est_rangee_et_chaque_mot_donne_son_type() {
    assert!(MOTS_DES_TYPES.windows(2).all(|p| p[0].0 < p[1].0));
    assert_eq!(MOTS_DES_TYPES, ATTENDUS);
    for (mot, attendu) in ATTENDUS {
        assert_eq!(type_propose(Some(&format!("Jeu/{mot}.wav"))), *attendu, "{mot}");
    }
}

#[test]
fn le_type_se_propose_d_un_mot_entier() {
    assert_eq!(type_propose(Some("Jeu A/Voix lead.wav")), Voix);
    assert_eq!(type_propose(Some("Jeu A/02_Batterie-overheads.wav")), Batterie);
    assert_eq!(type_propose(Some("Jeu A/Chœurs.wav")), Choeurs);
    assert_eq!(type_propose(Some("Jeu A/SYNTHÉ pad.wav")), Synthe);
    assert_eq!(type_propose(Some("Jeu A/voix.batterie.wav")), Voix);
    assert_eq!(type_propose(Some("Basse et voix.wav")), Basse);
    assert_eq!(type_propose(Some("Voix/piste.wav")), Autre, "seul le dernier segment compte");
}

#[test]
fn sans_mot_connu_le_type_est_autre() {
    assert_eq!(type_propose(None), Autre);
    assert_eq!(type_propose(Some("Jeu A/bassline.wav")), Autre);
    assert_eq!(type_propose(Some("Jeu A/piste 7.wav")), Autre);
    assert_eq!(mots_du_nom("Jeu A/02_Batterie-overheads.wav"), ["02", "batterie", "overheads"]);
    assert_eq!(mots_du_nom(".wav"), ["wav"], "`.wav` n'a pas d'extension : son seul mot est `wav`");
}

#[test]
fn les_types_ont_leur_nom_et_leur_libelle() {
    let noms: Vec<_> = TypeDeStem::TOUS.iter().map(|t| t.nom()).collect();
    assert_eq!(noms, ["voix", "choeurs", "batterie", "percussions", "basse", "guitare", "clavier", "cordes", "cuivres", "synthe", "effets", "autre"]);
    assert_eq!((Choeurs.libelle(), Synthe.libelle(), Voix.libelle()), ("chœurs", "synthé", "voix"));
    for t in TypeDeStem::TOUS { assert_eq!(TypeDeStem::lire(t.nom()), Some(t)); }
    assert_eq!(TypeDeStem::lire("chœurs"), None);
}

#[test]
fn le_midi_accompagne_le_stem_de_meme_nom() {
    let stems = ["Jeu A/basse.wav", "Jeu A/voix.wav"];
    assert_eq!(stem_du_midi("Jeu A/voix.mid", &stems), Some("Jeu A/voix.wav"));
    assert_eq!(stem_du_midi("Jeu A/Voix.mid", &stems), None);
    assert_eq!(stem_du_midi("Jeu A/voix 2.mid", &stems), None);
    assert_eq!(stem_du_midi("Jeu A/voix", &stems), Some("Jeu A/voix.wav"));
}

#[test]
fn trois_wav_de_meme_duree_proposent_un_jeu() {
    let w = |taille, debut: &'static [u8], trames, frequence| WavInspecte { taille, debut, trames, frequence };
    let (a, b, c): (&[u8], &[u8], &[u8]) = (&[1; 44], &[2; 44], &[3; 44]);
    assert!(jeu_propose(&[w(8_044, a, 8_000, 8_000), w(8_044, b, 8_000, 8_000), w(8_044, c, 8_000, 8_000)]));
    assert!(!jeu_propose(&[w(8_044, a, 8_000, 8_000), w(8_044, a, 8_000, 8_000)]), "même taille, même début : rien ne les distingue");
    assert!(jeu_propose(&[w(8_044, a, 8_000, 8_000), w(8_052, a, 8_000, 8_000)]), "des tailles distinctes suffisent");
    assert!(!jeu_propose(&[w(8_044, a, 8_000, 8_000), w(8_044, b, 8_000, 44_100)]));
    // 4 000 trames à 8 000 Hz et 8 000 trames à 16 000 Hz : 500 ms chacun (balayage P12).
    assert!(!jeu_propose(&[w(4_044, a, 4_000, 8_000), w(8_044, b, 8_000, 16_000)]), "même durée, trames et fréquence autres");
    assert!(!jeu_propose(&[w(8_044, a, 8_000, 8_000)]));
}

#[test]
fn la_duree_d_une_prise_en_stems() {
    assert_eq!(duree_de_la_prise_en_stems(&[Some(1_000), Some(1_500)]), Some(1_500));
    assert_eq!(duree_de_la_prise_en_stems(&[Some(1_000), None]), None);
    assert_eq!(duree_de_la_prise_en_stems(&[]), None);
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: le type proposé, le MIDI d'un stem et le jeu proposé, au
  noyau`), pousser. Attendu à « Lint » : `unresolved import noyau::donnees::stems`.
- [ ] **Étape 3 : écrire le code.** `MOTS_DES_TYPES` en constante, cherchée par `binary_search_by_key` ; `jeu_propose`
  range les WAV dans un `BTreeMap<(u64, u32), BTreeSet<(u64, &[u8])>>`, (trames, fréquence) vers (taille, début) ;
  aucune boucle `while` ni `loop`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `stems.rs` `ok. 7 passed` ; `mutants` sans
  survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/stems.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/stems.rs .cargo/mutants.toml` (`feat: le type proposé d'un stem et le jeu proposé, au noyau`).

### Tâche 6 : les opérations nommées de l'import, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/import.rs`, `crates/noyau/tests/import.rs` ; modifier
`crates/noyau/src/donnees/fonds.rs` (les clés), `crates/noyau/src/donnees/gestes.rs` (`Impossible`, ses messages, la
naissance d'une idée partagée), `crates/noyau/tests/gestes.rs`, `crates/noyau/src/donnees/mod.rs` (`pub mod import;`),
`.cargo/mutants.toml`, `crates/services/tests/suite/mod.rs` (deux messages ; balayage P1),
`crates/stockage/tests/messages.rs` (les variantes de l'import ; balayage P7).

**`.cargo/mutants.toml`** (balayage P11) : `"crates/noyau/src/donnees/import.rs"` dans `examine_globs`,
`"--test", "import"` dans `additional_cargo_test_args` (`gestes` y est déjà) ; le compte devient « Vingt-sept modules »
à `:2` et « ces vingt-sept modules » à la phrase qui suit la liste (`:33` sur `fe918aa`, plus bas des globs ajoutés) ;
l'énumération des lignes 2 à 7 gagne « les opérations nommées de l'import ». Aucune boucle `while` ni `loop` dans
`src/` : un parcours se borne par un itérateur fini. Jamais `FormatAudio` : le type se nomme `FormatSonore`, et
`Faits::du_wav` écrit le format par `FormatSonore::Wav.nom()` (balayage P4).

**Interfaces — consomme :** `gestes::{Sorte, Appareil, Impossible, date_du_message, ENTIER_SUR}`, `fonds::{IMPORTS,
ORIGINES_DECLAREES, PRISES, MIDI_VERSES, IDEES, ETIQUETTES, PROVENANCE, PAR, LE, DEPUIS, IDEE, FORMAT, DUREE, NOM,
IMPORTEE}`, `origine::{Ajout, OriginesLues, Declaration, ajout_d_origine}`, `stems::TypeDeStem`,
`signature::{FormatSonore, FormatTexte}`, `entete::{FaitsWav, FaitsMidi}`, `changement::{Changement, Operation, Simple}`.

**Interfaces — produit** (`noyau::donnees::fonds`, `noyau::donnees::import`, `noyau::donnees::gestes`) :

```rust
// fonds.rs : les clés de la tranche (§ 31 · 2), toutes sous des conteneurs de la genèse
pub const OUVERT: &str = "ouvert";            // imports/<i>/ouvert : l'heure de l'ouverture
pub const RAPPORT: &str = "rapport";          // imports/<i>/rapport : l'empreinte du rapport
pub const DECLARATION: &str = "declaration";  // imports/<i>/declaration/<chemin> : stems | main | generee:<nom>
pub const MANIFESTE: &str = "manifeste";      // prises/<p>/manifeste : true
pub const STEM: &str = "stem";                // prises/<p>/stem/<s>/…
pub const MIDI: &str = "midi";                // prises/<p>/stem/<s>/midi/<m> : true
pub const TYPE: &str = "type";                // prises/<p>/stem/<s>/type, et type/provenance, type/par, type/depuis, type/le
pub const CODAGE: &str = "codage"; pub const CANAUX: &str = "canaux"; pub const FREQUENCE: &str = "frequence";
pub const BITS: &str = "bits"; pub const DIVISION: &str = "division";
pub const MAIN: &str = "main"; pub const GENEREE: &str = "generee"; // origines_declarees/<e>/main, <e>/generee/<nom>
pub const SAISIE: &str = "saisie";            // la provenance d'un type choisi par Sacha
pub const PAR_LE_NOM: &str = "nom";           // le `par` d'un type gardé (question 18)

// import.rs
/// Les faits d'un fichier versé, tels que l'import les écrit ; un illisible n'en a aucun
/// (question 10), son format compris.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Faits {
    pub format: Option<String>, pub duree_ms: Option<u64>, pub codage: Option<u16>, pub canaux: Option<u16>,
    pub frequence: Option<u32>, pub bits: Option<u16>, pub division: Option<u16>,
}
impl Faits {
    pub fn du_wav(faits: &FaitsWav) -> Faits;         // format `wav`, la durée et les quatre faits d'en-tête
    pub fn du_midi(faits: &FaitsMidi) -> Faits;       // `smf0` à `smf2`, la division
    pub fn d_un_audio(format: FormatSonore) -> Faits; // le format seul : FLAC, MP3… (ADR-024, décision 6)
    pub fn d_un_texte(format: FormatTexte) -> Faits;  // le format seul
}

/// Ce qu'un import ouvert donne à chacune de ses opérations : son ULID, l'heure de son ouverture
/// (le jour que disent ses messages), l'appareil.
pub struct ImportOuvert<'a> { pub import: &'a str, pub ouvert_le: i64, pub par: Appareil }

/// Ce que le fonds porte déjà d'un contenu : sa captation, ses noms (les chemins), ses origines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DejaLu { pub captation: bool, pub noms: BTreeSet<String>, pub origines: OriginesLues }

/// Ouvrir l'import (question 22) : `imports/<i>/ouvert` à `heure`, et chaque déclaration sous
/// `imports/<i>/declaration/<chemin>` ; « Import du 3 novembre ouvert ». Refusé : un nom de
/// générateur vide, ou qui porte un caractère de contrôle.
pub fn ouvrir(import: &str, declarations: &BTreeMap<String, Declaration>, heure: i64, decalage_minutes: i32) -> Result<Operation, Impossible>;

/// Verser un fichier : un contenu neuf (`!deja.captation`) fait naître son idée, `importee`, par
/// l'appareil, depuis l'import, à `heure`, et écrit sa captation sous le conteneur de sa sorte
/// (`<e>/idee`, puis chaque fait présent : `format` ; d'une prise, `duree`, `codage`, `canaux`,
/// `frequence`, `bits` ; d'un MIDI, `division`), « Idée du 3 novembre : prise versée » (comme
/// `gestes::verser`). Puis, neuf ou connu : `<e>/nom/<chemin>` à l'ULID de l'import s'il est absent
/// (question 20) ; l'élément d'origine que `ajout_d_origine` garde, sous `origines_declarees`. Un
/// contenu connu à qui rien ne s'ajoute : `Ok(None)` ; sinon « Import du 3 novembre : nom d'origine
/// ajouté », « … : origine déclarée », ou les deux, séparés par une virgule. L'idée naît par
/// `gestes::naissance_de_l_idee`, la même que celle de `gestes::verser`.
pub fn verser_un_fichier(ouvert: &ImportOuvert<'_>, sorte: Sorte, empreinte: &str, faits: &Faits, chemin: Option<&str>,
    origine: Option<&Ajout>, deja: &DejaLu, idee: &str, heure: i64, decalage_minutes: i32) -> Result<Option<Operation>, Impossible>;

/// Un stem à verser dans une prise en stems, et le MIDI qui l'accompagne.
pub struct StemAVerser<'a> { pub empreinte: &'a str, pub faits: &'a Faits, pub chemin: Option<&'a str>, pub midi: Option<MidiDuStem<'a>> }
pub struct MidiDuStem<'a> { pub empreinte: &'a str, pub faits: &'a Faits, pub chemin: Option<&'a str> }

/// Ce que le fonds porte déjà d'une prise en stems : elle (son manifeste), ses noms, et, par
/// empreinte, ce que portent ses stems et leurs MIDI.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DejaDeLaPrise { pub prise: DejaLu, pub stems: BTreeMap<String, DejaLu>, pub midis: BTreeMap<String, DejaLu> }

/// Verser une prise en stems (questions 19, 21), `manifeste` l'empreinte de son manifeste, `nom` le
/// chemin de son dossier, ou son dernier segment pour un jeu versé seul ; `None` pour un jeu versé
/// seul dont le dossier n'a pas de nom en UTF-8 : la prise n'a alors aucune clé `nom`, et ses stems
/// gardent les leurs (R23). Neuve : son idée naît ;
/// `prises/<p>/idee`, `<p>/manifeste` à `true`, `<p>/nom/<nom>` ; pour chaque stem,
/// `<p>/stem/<s>/format` et `…/duree` s'ils sont présents, `…/nom/<chemin>`, `…/midi/<m>` à `true` ;
/// pour chaque MIDI d'un stem sans entrée sous `midi_verses`, `midi_verses/<m>/format` et
/// `…/division`, jamais `idee` ; « Idée du 3 novembre : prise en stems versée ». Neuve ou connue :
/// chaque nom absent, de la prise, des stems et des MIDI, et l'origine, sur la prise, chaque stem et
/// chaque MIDI. `Ok(None)` si rien ne s'ajoute. Sans stem : `Impossible::JeuSansAudio`. L'idée naît
/// par `gestes::naissance_de_l_idee`.
pub fn verser_une_prise_en_stems(ouvert: &ImportOuvert<'_>, manifeste: &str, nom: Option<&str>, stems: &[StemAVerser<'_>],
    origine: Option<&Ajout>, deja: &DejaDeLaPrise, idee: &str, heure: i64, decalage_minutes: i32) -> Result<Option<Operation>, Impossible>;

/// Clore l'import : `imports/<i>/rapport` à l'empreinte du rapport ; « Import du 3 novembre clos ».
pub fn clore(ouvert: &ImportOuvert<'_>, rapport: &str, heure: i64, decalage_minutes: i32) -> Operation;

/// Une captation que vise une déclaration d'origine faite après l'import : son empreinte, ce qu'elle
/// porte d'origines, et ce qui la nomme dans un message (son premier nom vu, ou « cette prise »).
pub struct Cible<'a> { pub empreinte: &'a str, pub origines: &'a OriginesLues, pub nommee: &'a str }

/// Ce que rend une déclaration d'origine : l'opération, s'il y a un changement ; les cibles qui
/// portaient déjà l'élément déclaré ; et celles, générées, où « de la main de Sacha » ne change rien
/// (question 22), seules à valoir un avertissement (R16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaree { pub operation: Option<Operation>, pub deja_portees: usize, pub generees_gardees: usize }

/// Déclarer une origine après l'import (question 22), sur une captation ou sur les captations d'un
/// préfixe, `portee` ce qui la nomme dans le message (« « Démo 1.wav » », « cette prise »,
/// « « Démos/ » »). `None` (`aucune`) : `Impossible::OrigineNonRetiree`. Un nom vide, ou qui porte
/// un caractère de contrôle : refusé. Chaque élément que `ajout_d_origine` garde s'écrit ;
/// « Origine déclarée pour <portee> : de la main de Sacha », ou « … : générée par <nom> ».
pub fn declarer_une_origine(cibles: &[Cible<'_>], ajout: Option<&Ajout>, portee: &str, heure: i64) -> Result<Declaree, Impossible>;

/// Ce que Sacha fait du type d'un stem : garder la proposition, ou choisir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoixDuType { Garder, Choisir(TypeDeStem) }

/// Choisir le type d'un stem (question 18) : `prises/<p>/stem/<s>/type` à `nom()` ; gardé,
/// `…/type/provenance` à `importee`, `…/type/par` à `nom`, `…/type/depuis` à `<s>` ; choisi,
/// `…/type/provenance` à `saisie`, `…/type/par` et `…/type/depuis` ôtés s'ils existaient
/// (§ 30 · 8.1 : `saisie` n'a ni `par` ni `depuis` ; R5) ; `…/type/le` à `heure`. « Type de
/// « voix.wav » : voix », ou « Type de ce stem : voix ». `Impossible::StemAbsent` si la prise ne
/// porte pas ce stem (`existe`).
pub fn choisir_le_type(prise: &str, stem: &str, existe: bool, propose: TypeDeStem, choix: ChoixDuType,
    nomme: Option<&str>, heure: i64) -> Result<Operation, Impossible>;

/// Le nom d'une copie en flux de l'import (question 31) : `import-<ULID>-<rang>`, que `voisin`
/// complète de `.<processus>.partiel`.
pub fn nom_de_la_copie(import: &str, rang: u64) -> String;
/// L'import et le rang d'un nom provisoire de `captations/` que `nom_de_la_copie` a formé
/// (`import-<26 caractères de Crockford>-<chiffres>.<processus>[-<n>].partiel`) ; `None` sinon,
/// le nom provisoire d'une capture compris.
pub fn copie_de(nom: &str) -> Option<(&str, u64)>;

// gestes.rs : Impossible gagne JeuSansAudio, OrigineNonRetiree, NomDeGenerateurVide,
// NomDeGenerateurIllisible, StemAbsent, AucunImportOuvert, ImportClos, FichierChange { nom: Option<String>, rang: u64 } ;
// CaptationAbsente et CaptationDejaVersee gardent leur champ, que leur message ne nomme plus.

// gestes.rs : la naissance d'une idée, extraite de `gestes::verser` (gestes.rs:704-721), que `gestes::verser`,
// `import::verser_un_fichier` et `import::verser_une_prise_en_stems` appellent (balayage P8) : l'objet de l'idée et
// celui de ses étiquettes, `provenance` à `origine` (`captee` ou `importee`), `par`, `le` à `heure`, et `depuis` s'il
// est donné.
pub(crate) fn naissance_de_l_idee(idee: &str, origine: &str, par: Appareil, depuis: Option<&str>, heure: i64) -> Vec<Changement>;
```

  `verser_un_fichier` (10 arguments) et `verser_une_prise_en_stems` (9) passent le seuil de `clippy::too_many_arguments`
  (7), que `-D warnings` refuse : chacune porte `#[allow(clippy::too_many_arguments)]`, précédé d'une ligne `//` qui
  dit pourquoi (chaque donnée du versement, puis ce que le fonds porte déjà), comme `gestes::verser`
  (`crates/noyau/src/donnees/gestes.rs:671-673` ; balayage P9).

  Les messages, à l'octet : `JeuSansAudio` « Ce dossier n'a aucun fichier audio : il ne forme pas un jeu de stems. » ;
  `OrigineNonRetiree` « Une origine déclarée ne se retire pas. » ; `NomDeGenerateurVide` « Le nom du générateur est
  vide. » ; `NomDeGenerateurIllisible` « Le nom du générateur tient sur une ligne, sans caractère de contrôle. » ;
  `StemAbsent` « Ce stem n'est pas dans cette prise. » ; `AucunImportOuvert` « Aucun import n'est ouvert. » ;
  `ImportClos` « Cet import est clos. » ; `FichierChange` « « <nom> » a changé depuis l'inspection : l'import
  s'arrête, et rechoisir le dossier le reprend. », ou, sans nom, « Le fichier n° <rang> a changé depuis l'inspection :
  … » ; `CaptationAbsente` « Cette captation n'est pas dans le fonds. » ; `CaptationDejaVersee` « Cette captation est
  déjà versée. » (question 26).

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/import.rs`, et la mise à jour des messages dans
  `crates/noyau/tests/gestes.rs`, `crates/services/tests/suite/mod.rs` et `crates/stockage/tests/messages.rs`, plus
  bas). Chaque test lit les changements de l'opération par une aide `cles_posees(&Operation)
  -> BTreeMap<String, Simple>`, qui joint le chemin de chaque `Changement::Poser` par `|` :

```rust
const I: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ";
const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023
// L'idée qui naît, un ULID de test ; les empreintes, des chaînes de 64 caractères hexadécimaux écrites en dur,
// distinctes : le noyau ne vérifie pas qu'une empreinte est celle d'un contenu, et `empreinte(b"…")` n'est pas une
// constante (balayage P15).
const IDEE: &str = "01HF7YAT000000000000000002";
const E1: &str = "e100000000000000000000000000000000000000000000000000000000000001";
// E2, E3 (des fichiers), P (le manifeste), S1, S2 (les stems voix et basse), M1 (le MIDI de la voix), R (le
// rapport) : de même, chacune distincte des autres.
fn ouvert() -> ImportOuvert<'static> { ImportOuvert { import: I, ouvert_le: HEURE, par: Appareil::Etabli } }
// Les faits des stems d'essai, en `static` : `stems_d_essai()` rend des `StemAVerser<'static>`, qui empruntent
// leurs `Faits` (balayage P15).
static FAITS_D_UN_STEM: std::sync::LazyLock<Faits> =
    std::sync::LazyLock::new(|| Faits::du_wav(&FaitsWav { codage: 1, canaux: 1, frequence: 8_000, bits: 8, trames: 8_000 }));
static FAITS_DU_MIDI: std::sync::LazyLock<Faits> =
    std::sync::LazyLock::new(|| Faits::du_midi(&FaitsMidi { format: 1, pistes: 3, division: 480 }));

#[test]
fn ouvrir_ecrit_l_heure_et_les_declarations() {
    let declarations = BTreeMap::from([(String::new(), Declaration::Origine(Ajout::Main)), ("Jeu A".into(), Declaration::Stems)]);
    let op = ouvrir(I, &declarations, HEURE, 0).unwrap();
    assert_eq!(op.message, "Import du 14 novembre ouvert");
    let cles = cles_posees(&op);
    assert_eq!(cles[&format!("imports|{I}/ouvert")], Simple::Entier(HEURE));
    assert_eq!(cles[&format!("imports|{I}/declaration/")], Simple::Chaine("main".into()));
    assert_eq!(cles[&format!("imports|{I}/declaration/Jeu A")], Simple::Chaine("stems".into()));
    let mauvais = BTreeMap::from([(String::new(), Declaration::Origine(Ajout::Generee("a\nb".into())))]);
    assert_eq!(ouvrir(I, &mauvais, HEURE, 0).unwrap_err(), Impossible::NomDeGenerateurIllisible);
}

#[test]
fn verser_un_wav_neuf_fait_naitre_son_idee_et_ecrit_ses_faits() {
    let faits = Faits::du_wav(&FaitsWav { codage: 1, canaux: 2, frequence: 44_100, bits: 16, trames: 44_100 });
    let op = verser_un_fichier(&ouvert(), Sorte::Prise, E1, &faits, Some("Démos/Démo 1.wav"), Some(&Ajout::Main),
                               &DejaLu::default(), IDEE, HEURE + 5, 0).unwrap().unwrap();
    assert_eq!(op.message, "Idée du 14 novembre : prise versée");
    let cles = cles_posees(&op);
    assert_eq!(cles[&format!("idees|{IDEE}|provenance")], Simple::Chaine("importee".into()));
    assert_eq!(cles[&format!("idees|{IDEE}|depuis")], Simple::Chaine(I.into()));
    assert_eq!(cles[&format!("prises|{E1}/format")], Simple::Chaine("wav".into()));
    assert_eq!(cles[&format!("prises|{E1}/duree")], Simple::Naturel(1_000));
    for (cle, valeur) in [("codage", 1), ("canaux", 2), ("frequence", 44_100), ("bits", 16)] {
        assert_eq!(cles[&format!("prises|{E1}/{cle}")], Simple::Naturel(valeur), "{cle}");
    }
    assert_eq!(cles[&format!("prises|{E1}/nom/Démos/Démo 1.wav")], Simple::Chaine(I.into()));
    assert_eq!(cles[&format!("origines_declarees|{E1}/main")], Simple::Vrai);
}

#[test]
fn un_illisible_n_a_aucun_fait() {
    let op = verser_un_fichier(&ouvert(), Sorte::Prise, E1, &Faits::default(), Some("a.wav"), None,
                               &DejaLu::default(), IDEE, HEURE, 0).unwrap().unwrap();
    let cles = cles_posees(&op);
    // Sa captation et son nom sont écrits, et rien d'autre sous elle (balayage P20).
    assert_eq!(cles[&format!("prises|{E1}/idee")], Simple::Chaine(IDEE.into()));
    assert_eq!(cles[&format!("prises|{E1}/nom/a.wav")], Simple::Chaine(I.into()));
    assert!(!cles.keys().any(|c| c.starts_with(&format!("prises|{E1}/")) && !c.ends_with("/idee") && !c.contains("/nom/")), "{cles:?}");
}

#[test]
fn un_contenu_connu_ne_prend_que_ses_noms_et_son_origine() {
    let deja = DejaLu { captation: true, noms: BTreeSet::from(["a.wav".to_string()]), origines: OriginesLues::default() };
    assert_eq!(verser_un_fichier(&ouvert(), Sorte::Prise, E1, &Faits::default(), Some("a.wav"), None, &deja, IDEE, HEURE, 0).unwrap(), None);
    let op = verser_un_fichier(&ouvert(), Sorte::Prise, E1, &Faits::default(), Some("b.wav"), None, &deja, IDEE, HEURE, 0).unwrap().unwrap();
    assert_eq!(op.message, "Import du 14 novembre : nom d'origine ajouté");
    assert!(!cles_posees(&op).keys().any(|c| c.starts_with("idees|")));
    let generee = DejaLu { origines: OriginesLues { main: false, generees: BTreeSet::from(["Atelier".into()]) }, ..deja.clone() };
    assert_eq!(verser_un_fichier(&ouvert(), Sorte::Prise, E1, &Faits::default(), Some("a.wav"), Some(&Ajout::Main), &generee, IDEE, HEURE, 0).unwrap(), None);
    let op = verser_un_fichier(&ouvert(), Sorte::Prise, E1, &Faits::default(), Some("c.wav"), Some(&Ajout::Generee("Forge".into())), &generee, IDEE, HEURE, 0).unwrap().unwrap();
    assert_eq!(op.message, "Import du 14 novembre : nom d'origine ajouté, origine déclarée");
}

#[test]
fn un_midi_et_un_texte_verses_ont_leurs_faits() {
    let midi = Faits::du_midi(&FaitsMidi { format: 1, pistes: 3, division: 480 });
    let cles = cles_posees(&verser_un_fichier(&ouvert(), Sorte::MidiVerseSeul, E2, &midi, Some("m.mid"), None, &DejaLu::default(), IDEE, HEURE, 0).unwrap().unwrap());
    assert_eq!((cles[&format!("midi_verses|{E2}/format")].clone(), cles[&format!("midi_verses|{E2}/division")].clone()),
               (Simple::Chaine("smf1".into()), Simple::Naturel(480)));
    let cles = cles_posees(&verser_un_fichier(&ouvert(), Sorte::TexteVerse, E3, &Faits::d_un_texte(FormatTexte::Chordpro), Some("t.cho"), None, &DejaLu::default(), IDEE, HEURE, 0).unwrap().unwrap());
    assert_eq!(cles[&format!("textes_verses|{E3}/format")], Simple::Chaine("chordpro".into()));
}

#[test]
fn verser_une_prise_en_stems_ecrit_son_manifeste_ses_stems_et_leurs_midi() {
    // deux stems, le premier accompagné d'un MIDI ; la prise prend le nom de son dossier
    let op = verser_une_prise_en_stems(&ouvert(), P, Some("Jeu A"), &stems_d_essai(), None, &DejaDeLaPrise::default(), IDEE, HEURE, 0).unwrap().unwrap();
    assert_eq!(op.message, "Idée du 14 novembre : prise en stems versée");
    let cles = cles_posees(&op);
    assert_eq!(cles[&format!("prises|{P}/manifeste")], Simple::Vrai);
    assert_eq!(cles[&format!("prises|{P}/nom/Jeu A")], Simple::Chaine(I.into()));
    assert!(!cles.contains_key(&format!("prises|{P}/format")) && !cles.contains_key(&format!("prises|{P}/duree")));
    assert_eq!(cles[&format!("prises|{P}/stem/{S1}/duree")], Simple::Naturel(1_000));
    assert_eq!(cles[&format!("prises|{P}/stem/{S1}/midi/{M1}")], Simple::Vrai);
    assert_eq!(cles[&format!("prises|{P}/stem/{S1}/nom/Jeu A/voix.wav")], Simple::Chaine(I.into()));
    assert_eq!(cles[&format!("midi_verses|{M1}/division")], Simple::Naturel(480));
    assert!(!cles.contains_key(&format!("midi_verses|{M1}/idee")));
    assert!(!cles.keys().any(|c| c.contains("/type")), "aucun type décidé à l'import (CAP-03 · 7)");
    assert_eq!(verser_une_prise_en_stems(&ouvert(), P, Some("Jeu A"), &[], None, &DejaDeLaPrise::default(), IDEE, HEURE, 0).unwrap_err(), Impossible::JeuSansAudio);
}

#[test]
fn une_prise_en_stems_sans_nom_garde_les_noms_de_ses_stems() {
    // un jeu versé seul dont le dossier n'a pas de nom en UTF-8 (R23)
    let op = verser_une_prise_en_stems(&ouvert(), P, None, &stems_d_essai(), None, &DejaDeLaPrise::default(), IDEE, HEURE, 0).unwrap().unwrap();
    let cles = cles_posees(&op);
    assert!(!cles.keys().any(|c| c.starts_with(&format!("prises|{P}/nom/"))));
    assert_eq!(cles[&format!("prises|{P}/stem/{S1}/nom/Jeu A/voix.wav")], Simple::Chaine(I.into()));
    assert_eq!(cles[&format!("prises|{P}/manifeste")], Simple::Vrai);
}

#[test]
fn clore_ecrit_l_empreinte_du_rapport() {
    let op = clore(&ouvert(), R, HEURE + 3_600, 0);
    assert_eq!(op.message, "Import du 14 novembre clos");
    assert_eq!(cles_posees(&op)[&format!("imports|{I}/rapport")], Simple::Chaine(R.into()));
}

#[test]
fn declarer_une_origine_apres_l_import() {
    let generee = OriginesLues { main: false, generees: BTreeSet::from(["Atelier".into()]) };
    let rien = OriginesLues::default();
    let cibles = [Cible { empreinte: E1, origines: &generee, nommee: "« a.wav »" }, Cible { empreinte: E2, origines: &rien, nommee: "« b.wav »" }];
    let faite = declarer_une_origine(&cibles, Some(&Ajout::Main), "« Démos/ »", HEURE).unwrap();
    assert_eq!((faite.deja_portees, faite.generees_gardees), (0, 1));
    let op = faite.operation.unwrap();
    assert_eq!(op.message, "Origine déclarée pour « Démos/ » : de la main de Sacha");
    assert_eq!(cles_posees(&op).len(), 1);
    assert_eq!(declarer_une_origine(&cibles, None, "x", HEURE).unwrap_err(), Impossible::OrigineNonRetiree);
    assert_eq!(declarer_une_origine(&cibles, Some(&Ajout::Generee(String::new())), "x", HEURE).unwrap_err(), Impossible::NomDeGenerateurVide);
    let seule = declarer_une_origine(&cibles[..1], Some(&Ajout::Main), "cette prise", HEURE).unwrap();
    assert_eq!((seule.operation, seule.deja_portees, seule.generees_gardees), (None, 0, 1));
    let op = declarer_une_origine(&cibles[1..], Some(&Ajout::Generee("Forge".into())), "« b.wav »", HEURE).unwrap().operation.unwrap();
    assert_eq!(op.message, "Origine déclarée pour « b.wav » : générée par Forge");
    // déjà portée : rien ne change, et rien ne vaut un avertissement (R16)
    let main = OriginesLues { main: true, generees: BTreeSet::new() };
    let deja = declarer_une_origine(&[Cible { empreinte: E3, origines: &main, nommee: "« c.wav »" }], Some(&Ajout::Main), "« c.wav »", HEURE).unwrap();
    assert_eq!((deja.operation, deja.deja_portees, deja.generees_gardees), (None, 1, 0));
    let atelier = declarer_une_origine(&cibles[..1], Some(&Ajout::Generee("Atelier".into())), "« a.wav »", HEURE).unwrap();
    assert_eq!((atelier.operation, atelier.deja_portees, atelier.generees_gardees), (None, 1, 0));
}

#[test]
fn garder_puis_choisir_le_type() {
    let gardee = cles_posees(&choisir_le_type(P, S1, true, TypeDeStem::Voix, ChoixDuType::Garder, Some("voix.wav"), HEURE).unwrap());
    assert_eq!(gardee[&format!("prises|{P}/stem/{S1}/type")], Simple::Chaine("voix".into()));
    assert_eq!(gardee[&format!("prises|{P}/stem/{S1}/type/provenance")], Simple::Chaine("importee".into()));
    assert_eq!(gardee[&format!("prises|{P}/stem/{S1}/type/par")], Simple::Chaine("nom".into()));
    assert_eq!(gardee[&format!("prises|{P}/stem/{S1}/type/depuis")], Simple::Chaine(S1.into()));
    let op = choisir_le_type(P, S1, true, TypeDeStem::Voix, ChoixDuType::Choisir(TypeDeStem::Choeurs), None, HEURE).unwrap();
    assert_eq!(op.message, "Type de ce stem : chœurs");
    let choisie = cles_posees(&op);
    assert_eq!(choisie[&format!("prises|{P}/stem/{S1}/type/provenance")], Simple::Chaine("saisie".into()));
    assert!(!choisie.contains_key(&format!("prises|{P}/stem/{S1}/type/par")), "saisie : ni par ni depuis (R5)");
    for ote in ["/type/par", "/type/depuis"] {
        assert!(op.changements.iter().any(|c| matches!(c, Changement::Oter { chemin } if chemin.last().is_some_and(|s| s.ends_with(ote)))), "{ote}");
    }
    assert_eq!(choisir_le_type(P, S1, false, TypeDeStem::Voix, ChoixDuType::Garder, None, HEURE).unwrap_err(), Impossible::StemAbsent);
}

#[test]
fn le_nom_d_une_copie_se_forme_et_se_relit() {
    assert_eq!(nom_de_la_copie(I, 12), "import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12");
    assert_eq!(copie_de("import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12.4242.partiel"), Some((I, 12)));
    assert_eq!(copie_de("import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12.4242-2.partiel"), Some((I, 12)));
    for nom in [&format!("{E1}.4242.partiel"), "import-court-1.1.partiel", "import-01HF7YAT00ZZZZZZZZZZZZZZZZ-x.1.partiel",
                "import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12", "import-01HF7YAT00ZZZZZZZZZZZZZZZu-1.1.partiel",
                "import-01HF7YAT00ZZZZZZZZZZZZZZZI-1.1.partiel"] {
        assert_eq!(copie_de(nom), None, "{nom}");
    }
}

// crates/noyau/tests/gestes.rs : un test de plus (balayage P7)
#[test]
fn les_refus_de_l_import_se_disent_a_l_octet() {
    let e = "e100000000000000000000000000000000000000000000000000000000000001".to_string();
    for (refus, attendu) in [
        (Impossible::JeuSansAudio, "Ce dossier n'a aucun fichier audio : il ne forme pas un jeu de stems."),
        (Impossible::OrigineNonRetiree, "Une origine déclarée ne se retire pas."),
        (Impossible::NomDeGenerateurVide, "Le nom du générateur est vide."),
        (Impossible::NomDeGenerateurIllisible, "Le nom du générateur tient sur une ligne, sans caractère de contrôle."),
        (Impossible::StemAbsent, "Ce stem n'est pas dans cette prise."),
        (Impossible::AucunImportOuvert, "Aucun import n'est ouvert."),
        (Impossible::ImportClos, "Cet import est clos."),
        (Impossible::FichierChange { nom: Some("Démo 1.wav".into()), rang: 7 },
         "« Démo 1.wav » a changé depuis l'inspection : l'import s'arrête, et rechoisir le dossier le reprend."),
        (Impossible::FichierChange { nom: None, rang: 7 },
         "Le fichier n° 7 a changé depuis l'inspection : l'import s'arrête, et rechoisir le dossier le reprend."),
        (Impossible::CaptationAbsente { empreinte: e.clone() }, "Cette captation n'est pas dans le fonds."),
        (Impossible::CaptationDejaVersee { empreinte: e }, "Cette captation est déjà versée."),
    ] {
        assert_eq!(refus.to_string(), attendu);
    }
}
```

  `stems_d_essai()` rend deux `StemAVerser<'static>` : `Jeu A/voix.wav` (`S1`, `&FAITS_D_UN_STEM`, 1 000 ms), avec
  `Jeu A/voix.mid` (`M1`, `&FAITS_DU_MIDI`, `smf1`, 480) ; puis `Jeu A/basse.wav` (`S2`, `&FAITS_D_UN_STEM`), sans MIDI.
  `ULID` de Crockford : `copie_de` refuse une minuscule et les lettres `I`, `L`, `O`, `U`.

  Dans `crates/noyau/tests/gestes.rs`, `une_captation_absente_se_dit_par_son_empreinte` (`:838`) devient
  `une_captation_absente_ne_nomme_pas_son_empreinte`, et attend « Cette captation n'est pas dans le fonds. » ;
  `verser_une_fiche_deja_versee_est_refuse` (`:1039`) attend « Cette captation est déjà versée. » à `:1064` (balayage
  P20). Dans
  `crates/services/tests/suite/mod.rs`, `une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change` attend
  `impossible("Cette captation est déjà versée.".to_string())` (`:1903-1906`), et
  `regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse`,
  `impossible("Cette captation n'est pas dans le fonds.".to_string())` (`:1986-1990`) : leurs messages viennent de
  `Display` d'`Impossible`, par `refus_impossible` (`crates/services/src/lib.rs:939`), sur les deux paires (balayage
  P1). `crates/services/tests/application.rs:430` et `crates/stockage/tests/messages.rs:172` ne bougent pas : ils
  lisent les messages des adaptateurs, que la tâche 9 récrit. Dans `crates/stockage/tests/messages.rs`, la liste
  d'`aucun_message_n_est_un_ordre` (`:301`) gagne les huit variantes nouvelles, `FichierChange` avec et sans nom
  (balayage P7).
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les opérations nommées de l'import, au noyau`), pousser. Attendu à
  « Lint » : `unresolved import noyau::donnees::import`.
- [ ] **Étape 3 : écrire le code.** Chaque opération pose ses changements par `poser`, `oter` et `chemin`
  (`gestes.rs`), les clés à plat `format!("{e}/{NOM}/{chemin}")` ; un nom de générateur se contrôle par
  `char::is_control`. Le bloc de la naissance d'une idée (`crates/noyau/src/donnees/gestes.rs:704-721`) devient
  `naissance_de_l_idee`, que `gestes::verser` appelle sans changer ce qu'il écrit, et que les deux versements de
  l'import appellent au lieu de le recopier (balayage P8). La doc de `verser_un_fichier` dit qu'il ne passe pas par
  `gestes::verser`, qui refuse un contenu connu et écrit toujours `format`, et qu'il partage avec lui la naissance de
  l'idée ; celle de `copie_de`, qu'il ne reconnaît que les noms que `nom_de_la_copie` et `voisin` forment. Les deux
  versements portent leur `#[allow(clippy::too_many_arguments)]` raisonné (balayage P9). Aucune boucle `while` ni
  `loop`. Les tests réécrits (les deux de `crates/noyau/tests/gestes.rs`, les deux de la suite) sont dits dans la PR.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `import.rs` `ok. 11 passed`, `gestes.rs` du
  noyau vert, avec un test de plus (`les_refus_de_l_import_se_disent_a_l_octet`), `commandes_en_memoire.rs`,
  `commandes_sur_fichiers.rs` et `messages.rs` verts ; dans `windows`, `commandes_sur_fichiers.rs` et `messages.rs`
  verts ; `mutants` sans survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/import.rs crates/noyau/src/donnees/fonds.rs
  crates/noyau/src/donnees/gestes.rs crates/noyau/src/donnees/mod.rs crates/noyau/tests/import.rs
  crates/noyau/tests/gestes.rs crates/services/tests/suite/mod.rs crates/stockage/tests/messages.rs .cargo/mutants.toml`
  (`feat: les opérations nommées de l'import, au noyau`).

### Tâche 7 : les empreintes que nomme le fonds, étendues ; les copies de l'import à retirer

**Fichiers :** modifier `crates/noyau/src/donnees/idee.rs`, `crates/noyau/src/donnees/demarrage.rs`,
`crates/noyau/tests/idee.rs`, `crates/noyau/tests/demarrage.rs`, `crates/services/src/lecture.rs` (l'appel seul, les
rapports à vide jusqu'à la tâche 10), `crates/stockage/tests/h07.rs` (l'appel seul ; balayage P2).

**`.cargo/mutants.toml`** (balayage P11) : aucun module nouveau, `idee.rs` et `demarrage.rs` y sont déjà, avec leurs
tests ; le compte reste à vingt-sept, et le fichier ne change pas. Aucune boucle `while` ni `loop` dans `src/` : un
parcours se borne par un itérateur fini.

Les appelants d'`idee::empreintes_nommees`, qui prennent tous `[]` pour les rapports à l'étape 3 :
`crates/services/src/lecture.rs:214`, `crates/stockage/tests/h07.rs:146` et l'ancien test
`les_empreintes_nommees_se_coupent_au_premier_trait` (`crates/noyau/tests/idee.rs:23`).

**Interfaces — produit** :

```rust
// idee.rs
/// Les empreintes que nomme le fonds (décision 12 ; P1) : de chaque clé de `cles` (les conteneurs
/// qui nomment une captation), son premier segment ; si le deuxième vaut `stem`, le troisième ; si,
/// de plus, le quatrième vaut `midi`, le cinquième ; et chaque valeur de `rapports` (les
/// `imports/<i>/rapport`). Les segments se lisent à leur place, jamais dans un nom
/// (`<e>/nom/<chemin>`), quel que soit le chemin.
pub fn empreintes_nommees<'a>(cles: impl IntoIterator<Item = &'a str>, rapports: impl IntoIterator<Item = &'a str>) -> BTreeSet<String>;

// demarrage.rs
/// Les copies de l'import `import` à retirer (question 31) : de `copies`, les noms provisoires de
/// `captations/` que `import::copie_de` dit de cet import ; à la reprise (`a_la_cloture` faux),
/// ceux dont l'empreinte n'est pas dans `nommees` ; à la clôture, tous. Le nom provisoire d'une
/// capture n'y est jamais.
pub fn copies_a_retirer(copies: &[Provisoire], import: &str, nommees: &BTreeSet<String>, a_la_cloture: bool) -> Vec<String>;
```

- [ ] **Étape 1 : écrire les tests.** Les constantes sont des `&str` écrites en dur (balayage P16) : dans
  `crates/noyau/tests/idee.rs`, `P`, `S`, `M`, `E`, `R`, `FAUX` et `FAUX2`, sept chaînes distinctes de 64 caractères
  hexadécimaux ; dans `crates/noyau/tests/demarrage.rs`, `I = "01HF7YAT00ZZZZZZZZZZZZZZZZ"`, `AUTRE =
  "01HF7YAT010000000000000001"`, un ULID de Crockford valide, pour que `copie_de` reconnaisse la copie d'un autre import
  et que le test l'écarte par l'import, non par sa forme, et `E1` à `E4`, quatre chaînes distinctes de 64 caractères
  hexadécimaux. Dans `crates/noyau/tests/idee.rs` :

```rust
#[test]
fn les_empreintes_nommees_lisent_les_stems_leurs_midi_et_les_rapports() {
    let cles = [format!("{P}/idee"), format!("{P}/manifeste"), format!("{P}/stem/{S}/duree"),
                format!("{P}/stem/{S}/midi/{M}"), format!("{P}/stem/{S}/nom/Jeu A/voix.wav"),
                format!("{E}/nom/stem/{FAUX}/midi/{FAUX2}"), format!("{M}/division")];
    let nommees = empreintes_nommees(cles.iter().map(String::as_str), [R]);
    assert_eq!(nommees, BTreeSet::from([P, S, M, E, R].map(String::from)));
    assert!(!nommees.contains(FAUX) && !nommees.contains(FAUX2));
    assert_eq!(empreintes_nommees(["sans-barre"], []), BTreeSet::new());
}
```

  Dans `crates/noyau/tests/demarrage.rs` :

```rust
#[test]
fn la_reprise_retire_les_copies_non_nommees_et_la_cloture_toutes() {
    let copie = |nom: &str, e: &str| Provisoire { nom: nom.into(), empreinte: e.into(), deja_rangee: false };
    let copies = [copie(&format!("import-{I}-1.7.partiel"), E1), copie(&format!("import-{I}-2.7.partiel"), E2),
                  copie(&format!("import-{AUTRE}-1.7.partiel"), E3), copie(&format!("{E4}.7.partiel"), E4)];
    let nommees = BTreeSet::from([E1.to_string()]);
    assert_eq!(copies_a_retirer(&copies, I, &nommees, false), [format!("import-{I}-2.7.partiel")]);
    assert_eq!(copies_a_retirer(&copies, I, &nommees, true), [format!("import-{I}-1.7.partiel"), format!("import-{I}-2.7.partiel")]);
}

#[test]
fn une_copie_de_capture_n_est_jamais_retiree() {
    let capture = Provisoire { nom: format!("{E4}.7.partiel"), empreinte: E4.into(), deja_rangee: false };
    assert!(copies_a_retirer(&[capture.clone()], I, &BTreeSet::new(), true).is_empty());
    assert!(copies_a_retirer(&[capture], I, &BTreeSet::new(), false).is_empty());
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: les stems, leurs MIDI et le rapport parmi les empreintes
  nommées`), pousser. Le commit ne porte que les tests, qui appellent à deux arguments une fonction qui n'en prend
  encore qu'un. Attendu à « Lint » : `this function takes 1 argument but 2 arguments were supplied`
  (`crates/noyau/tests/idee.rs`), et `unresolved import` de `copies_a_retirer` (`crates/noyau/tests/demarrage.rs`)
  (balayage P10).
- [ ] **Étape 3 : écrire le code.** Les segments par `split('/')`, lus par leur rang (`nth`), sans boucle `while` ni
  `loop` ; `copies_a_retirer` filtre par `import::copie_de`. Dans le même commit, `crates/services/src/lecture.rs:214`,
  `crates/stockage/tests/h07.rs:146` et `les_empreintes_nommees_se_coupent_au_premier_trait` passent `[]` pour les
  rapports, `[]` partout et non `std::iter::empty()` (balayage P16) ; la tâche 10 met les rapports dans l'appel de
  `lecture`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `idee.rs` et `demarrage.rs` verts, la suite des
  commandes inchangée depuis la tâche 6 ; dans `windows`, `h07.rs` vert ; `mutants` sans survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/idee.rs crates/noyau/src/donnees/demarrage.rs
  crates/noyau/tests/idee.rs crates/noyau/tests/demarrage.rs crates/services/src/lecture.rs crates/stockage/tests/h07.rs`
  (`feat: les empreintes nommées lisent les stems et le rapport ; les copies de l'import à retirer`).

### Tâche 8 : les documents de la PR (a), la mutation, les délais, et la revue de la PR (a)

**Fichiers :** `docs/30-modele-de-domaine.md` (§ 8.1, § 9.2), `docs/31-persistance-et-synchronisation.md` (§ 2 ; § 10,
le test 11), `docs/70-qualite-et-verification.md` (§ 1), `docs/80-glossaire.md`, `.cargo/mutants.toml` (sa phrase de
tête, relue), `CONVENTIONS.md`, `CLAUDE.md`, `.github/workflows/ci.yml` et `.github/workflows/mutants-complet.yml`
(les délais).

**`.cargo/mutants.toml`** (balayage P11) : les tâches 1 à 6 y ont mis les six modules, leurs tests, le compte
« Vingt-sept » à `:2` et dans « ces vingt-sept modules », et chacune son thème dans l'énumération des lignes 2 à 7 ;
la tâche 8 relit la phrase entière et en corrige la langue, sans module nouveau.

- [ ] **Étape 1 : la recherche de LE-2**, citée dans la description de la PR : `illisible`, `illisibles`, `verser`,
  `versé`, `versée`, `rassembler`, `manifeste`, `rapport`, `signature`, `en-tête`, `proposition`, `origine`, chacun
  aussi sans accent et sur la majuscule accentuée ; chaque emploi antérieur se requalifie (question 33 : regrouper
  « rassemble »).
- [ ] **Étape 2 : le modèle.** § 30 · 9.2 : la table des signatures et des extensions (tâche 1), sa ligne MP3 avec
  l'exception de l'octet 1 `0xFE`, le BOM d'un UTF-16 petit-boutiste, qu'admet la question 10 (balayage P3) ; les faits d'un WAV
  (format, durée, codage, canaux, fréquence, bits) et d'un MIDI (format, division brute) ; « illisible » ; les types de
  stem, la liste fermée et leurs libellés ; le manifeste à l'octet, avec son témoin ; le rapport d'import à l'octet,
  avec son témoin ; « stem » et « MIDI d'un stem » ne sont la captation d'aucune idée ; un fichier illisible n'a aucun
  fait, son format compris, et ne s'écoute pas (R17). § 30 · 8.1 : la ligne du type d'un stem, gardé (`importee`,
  `par` = `nom`, `depuis` = l'empreinte du stem, question 18, quand la table dit `par` = format pour un élément
  importé) et choisi (`saisie`, sans `par` ni `depuis`) (R5).
- [ ] **Étape 3 : la persistance.** § 31 · 2 : sous `imports`, `<i>/ouvert`, `<i>/rapport`, `<i>/declaration/<chemin>` ;
  sous `origines_declarees`, `<e>/main` et `<e>/generee/<nom>`, à `true`, l'ensemble dont le noyau tire la plus forte ;
  sous `prises`, les faits d'en-tête, `<p>/manifeste`, `<e>/nom/<chemin>` à l'ULID de l'import qui l'a vu le premier
  (question 20, au lieu de `true`), l'entrée d'un stem et son type avec sa provenance ; sous `midi_verses`, la division,
  et le MIDI d'un stem sans `idee` ; aucune migration, la genèse inchangée, son empreinte de référence aussi ; le texte
  dit que ces clés se gèlent avec la `v0.3.0`. § 31 · 10 : la ligne du test 11 nomme aussi le manifeste et le rapport
  témoins, à leurs empreintes de référence, que `crates/stockage/tests/references.rs` est seul à écrire (balayage P17).
- [ ] **Étape 4 : la qualité, le glossaire.** § 70 · 1 : « Les faits d'un fichier versé » récrit (la table fermée, les
  en-têtes, le texte lisible, illisible) ; « Un import rejoué ne crée aucun doublon » récrit d'après la question 9 ;
  une ligne de plus, « L'import d'un fonds », pour la place libre, l'origine la plus forte, le premier nom vu, la portée,
  le type proposé, le jeu proposé, la durée d'une prise en stems et les comptes du rapport (C24). Le glossaire :
  « Illisible (fichier versé) », distinct de ses deux autres sens (question 10) ; « Verser » réservé à l'import, et
  « Rassembler » pour regrouper (question 33) ; « Manifeste (d'une prise en stems) » et « Rapport d'import » à
  l'octet ; « Proposition » nomme l'import comme troisième source au lot 1 (question 18).
- [ ] **Étape 5 : les portes.** `.cargo/mutants.toml` : la phrase des lignes 2 à 7, relue (« Vingt-sept modules ») ;
  `CONVENTIONS.md` : `mutants` (les six modules) ; `CLAUDE.md` : « Les crates » (le noyau : les signatures, les
  en-têtes, le rapport, l'origine, les stems, l'import). Le délai de `mutants` (et de `mutants-complet.yml`) vaut deux
  fois le plus long passage complet (`.github/workflows/ci.yml:268`), que le job d'une PR, qui ne mute que le diff, ne
  fait pas : lancer `gh workflow run mutants-complet.yml --ref t5a-les-regles-de-l-import`, lire sa durée
  (`gh run list --workflow mutants-complet.yml --branch t5a-les-regles-de-l-import --limit 1`, puis `gh run view`),
  et dater le commentaire de cette exécution (balayage P23) ; ceux de `verifier` et de `windows`, remesurés sur
  l'exécution de la tête.
- [ ] **Étape 6 : committer**, en deux commits, par chemins explicites (balayage P23). Les documents : `git add
  docs/30-modele-de-domaine.md docs/31-persistance-et-synchronisation.md docs/70-qualite-et-verification.md
  docs/80-glossaire.md .cargo/mutants.toml CONVENTIONS.md CLAUDE.md` (`docs: les règles de l'import au modèle, au
  schéma, aux règles du noyau et au glossaire`). Les délais : `git add .github/workflows/ci.yml
  .github/workflows/mutants-complet.yml` (`ci: les délais de mutants, de verifier et de windows remesurés`).
- [ ] **Étape 7 : la revue de la PR (a).** La revue de la PR entière, sur sa tête, par le modèle le plus capable : chaque
  valeur témoin recalculée (le manifeste `340999f9…`, le rapport `6d161268…`, les Go, les durées, la taille *syncsafe*
  2 130 318), chaque règle de LE-6
  recopiée, chaque mécanisme annoncé que la PR touche relu, la recherche de LE-2. Puis les corrections, leur relecture
  sur la tête qui part. Le corps dit : les tests réécrits, et pourquoi (les messages de `CaptationAbsente` et de
  `CaptationDejaVersee`, qui ne nomment plus l'empreinte : `une_captation_absente_ne_nomme_pas_son_empreinte` et
  `verser_une_fiche_deja_versee_est_refuse` au noyau, `une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change` et
  `regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse` dans la suite des commandes ; la naissance d'une
  idée extraite de `gestes::verser`, ses tests inchangés ; l'appel de `h07.rs` à deux arguments) ; que la table fermée
  de la question 11 est accordée à la question 10 : la ligne MP3 excepte l'octet 1 `0xFE`, car `FF FE` est le BOM d'un
  UTF-16 petit-boutiste, texte que la question 10 admet et qu'un « Unicode » de Windows écrit, et que la signature,
  l'emportant sur l'extension, aurait versé en prise `mp3` ; la question 11 garde sa lettre, la table de la tâche 1 et
  le § 30 · 9.2 portent l'exception (balayage P3) ; qu'aucune règle n'est encore appelée par les services (PR (b)) ;
  que le WAV dont l'en-tête
  passe 65 536 octets est illisible, limite dite, et qu'un fichier illisible, sans format, ne s'écoute pas : le
  protocole lui rendra 404, même à un WAV que l'appareil saurait jouer (R17) ; qu'une photo HEIC ou AVIF est ignorée,
  par le code de format aux octets 8 à 11 de son `ftyp`, une vidéo versée en prise `m4a` (question 36) ; que le nom d'un générateur tient sur une ligne ; « Écarts à la spec :
  Aucun. ». `gh pr ready`, puis la fusion par squash, `--match-head-commit` nommant la tête relue.

## PR (b) · l'import, ses déclarations, son rapport, le test 13

**Branche :** `t5b-l-import`. **Titre :** `feat: l'import d'un fonds, ses déclarations, son rapport, la copie en flux et
le test 13` (88). **Documents :** CAP-02, CAP-03, CAP-07 ; SYN-07 ; § 30 · 2 ; § 31 · 2.1, 2.4, 6.1 et 10 ; § 40 · 3 ;
§ 41 · 6 ; § 70 · 3, 4.1 et 8 ; le glossaire ; `docs/pistes.md` ; `scripts/verif-confidentialite.sh` ;
`CONVENTIONS.md` ; `CLAUDE.md`. **Fin :** la tâche 17.

### Tâche 9 : la source à verser et la copie en flux, au stockage et en mémoire

**Fichiers :** modifier `crates/services/src/contrats.rs`, `crates/services/src/en_memoire.rs`,
`crates/services/src/lib.rs` (les exports), `crates/stockage/src/captations.rs`, `crates/stockage/src/fichier.rs`,
`crates/stockage/src/pannes.rs`, `crates/stockage/src/adaptateurs.rs`, `crates/stockage/src/lib.rs` ; créer
`crates/services/src/source_en_memoire.rs`, `crates/stockage/src/source.rs`, `crates/stockage/tests/source.rs` ;
modifier `crates/stockage/tests/captations.rs`, `crates/stockage/tests/demarrage.rs`,
`crates/services/tests/commandes_en_memoire.rs`.

**Interfaces — consomme :** `identifiants::Hacheur` (tâche 2), `import::{nom_de_la_copie, copie_de}` (tâche 6),
`demarrage::{Provisoire, a_ranger}`.

**Interfaces — produit** (`services::contrats`, `services::source_en_memoire`, `stockage::source`) :

```rust
/// La sorte d'une entrée du dossier versé, lue sans suivre un lien.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SorteDEntree { Fichier, Lien, Autre }

/// Une entrée du dossier versé : son rang (dès 1), son chemin relatif, segments joints par `/`, s'il
/// est de l'UTF-8 ; l'extension de son dernier segment, si elle en est ; ses octets ; sa sorte ; sa
/// marque de dernière écriture, telle que le système de fichiers la tient, `None` s'il ne la tient
/// pas (sous Windows, `last_write_time` rend alors 0, que la source lit `None` ; V4) (question 14 ;
/// R8). La marque de dernière écriture n'est pas une heure : elle ne se compare qu'à elle-même.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entree { pub rang: u64, pub chemin: Option<String>, pub extension: Option<String>, pub octets: u64, pub sorte: SorteDEntree, pub marque: Option<u64> }

/// La source à verser (§ 40 · 3, le troisième contrat) : un dossier parcouru une fois.
pub trait Source {
    /// Le dernier segment du dossier versé, s'il est de l'UTF-8 : le nom d'un jeu versé seul.
    fn nom_du_dossier(&self) -> Option<&str>;
    /// Ses entrées, dans l'ordre de leurs rangs, rangées par leurs segments, comparés un à un ;
    /// les dossiers n'y sont pas, leurs entrées si.
    fn entrees(&self) -> &[Entree];
    /// Au plus `longueur` octets de l'entrée `rang`, depuis `debut`.
    fn lire(&self, rang: u64, debut: u64, longueur: usize) -> Result<Vec<u8>, Refus>;
    /// L'entrée `rang`, en flux, de bout en bout.
    fn lire_en_flux(&self, rang: u64) -> Result<Box<dyn io::Read + '_>, Refus>;
    /// Les octets de l'entrée `rang` à cet instant.
    fn octets_actuels(&self, rang: u64) -> Result<u64, Refus>;
    /// La marque de dernière écriture de l'entrée `rang` à cet instant.
    fn marque_actuelle(&self, rang: u64) -> Result<Option<u64>, Refus>;
    /// Les octets lus de l'entrée `rang` depuis le parcours, par `lire` et `lire_en_flux` (décision 19).
    fn octets_lus(&self, rang: u64) -> u64;
}

/// Ce que rend une copie en flux : l'empreinte calculée, les octets copiés, si une captation la
/// portait déjà, et si la copie est rangée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versement { pub empreinte: String, pub octets: u64, pub deja_rangee: bool, pub rangee: bool }

pub trait Captations {
    /// Copie `flux` sous un nom provisoire formé sur `base` (`voisin`), l'empreinte calculée en
    /// chemin, le synchronise, le relit en flux contre elle, puis le range ; une captation déjà
    /// rangée sous cette empreinte : le nom provisoire se retire, `deja_rangee` vrai (question 8).
    /// `attendue` donnée et autre que l'empreinte calculée : le nom provisoire se retire avant le
    /// renommage, rien n'est rangé, `rangee` faux (un fichier changé depuis l'inspection ; R20).
    fn verser_en_flux(&self, base: &str, flux: &mut dyn io::Read, attendue: Option<&str>) -> Result<Versement, Refus>;
    /// La voie par octets : `verser_en_flux`, `base` l'empreinte des octets, calculée d'abord, `attendue` à `None`.
    fn verser(&self, octets: &[u8]) -> Result<String, Refus> { /* fournie par le contrat */ }
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus>;
    /// Au plus `longueur` octets de la captation `empreinte`, depuis `debut` ; le fichier refermé.
    fn relire_une_plage(&self, empreinte: &str, debut: u64, longueur: u64) -> Result<Vec<u8>, Refus>;
    /// Les octets de la captation `empreinte`, `None` si aucune ne la porte.
    fn taille(&self, empreinte: &str) -> Result<Option<u64>, Refus>;
    /// Les tailles des captations rangées, chacune une fois.
    fn tailles(&self) -> Result<BTreeSet<u64>, Refus>;
    /// Les noms provisoires de `captations/` que `import::copie_de` dit de l'import `import`,
    /// chacun lu en flux (son empreinte, et si elle est rangée).
    fn copies_de_l_import(&self, import: &str) -> Result<Vec<Provisoire>, Refus>;
    /// Retire le nom provisoire `nom` de `captations/`.
    fn retirer_une_copie(&self, nom: &str) -> Result<(), Refus>;
}

// services::source_en_memoire (hors d'`en-memoire` : le générateur du test 13 et la tranche 7 l'emploient)
pub struct SourceEnMemoire { /* nom, entrées, contenus, octets lus par rang (`RefCell`) */ }
impl SourceEnMemoire {
    /// `fichiers` : (chemin, contenu), un chemin `None` étant un nom sans forme UTF-8 ; `liens` :
    /// des chemins ; rangés par leurs segments, puis numérotés dès 1.
    pub fn new(nom_du_dossier: Option<&str>, fichiers: Vec<(Option<String>, Vec<u8>)>, liens: Vec<String>) -> Self;
    /// Change le contenu de l'entrée `rang`, et avance sa marque de dernière écriture d'une unité
    /// (question 14).
    pub fn changer(&mut self, rang: u64, contenu: Vec<u8>);
    /// L'entrée `rang` refuse toute lecture, de `Raison::Disque`, tant que `refusee` est vrai
    /// (question 38 ; V2).
    pub fn refuser(&mut self, rang: u64, refusee: bool);
}

// stockage::source
pub struct SourceSurFichiers { /* le dossier, ses entrées et leurs chemins, les octets lus (`AtomicU64`) */ }
impl SourceSurFichiers {
    /// Parcourt `dossier` une fois, par une pile, sans suivre un lien (`fs::symlink_metadata`,
    /// `read_dir` sur les seuls dossiers) ; une jonction compte pour un lien. Un dossier qui ne se
    /// liste pas : `Erreur::Disque`, sur lui. La marque de dernière écriture de chaque fichier : sous
    /// Windows, `std::os::windows::fs::MetadataExt::last_write_time`, `None` s'il vaut 0 (le système
    /// de fichiers ne la tient pas) ; ailleurs, `std::os::unix::fs::MetadataExt::mtime` × 10⁹ +
    /// `mtime_nsec`, `None` si elle est négative ou déborde ; jamais `Metadata::modified`, dont la
    /// valeur est une heure de l'horloge (I30).
    pub fn parcourir(dossier: &Path) -> Result<SourceSurFichiers, Erreur>;
}

// stockage::fichier
/// Les octets d'un tronçon d'une copie en flux, au plus.
pub(crate) const TRONCON: usize = 1_048_576;

// stockage::pannes
// Etape gagne `Troncon` : « Avant l'écriture de chaque tronçon d'une copie en flux : la `rang`-ième
// coupe après `rang - 1` tronçons écrits. » La lecture en flux passe par `Relecture` à son début, et
// `alterer_la_relecture` compte chacune de ses lectures comme une lecture de `fichier::lire`, son
// premier tronçon altéré.
```

  La copie en flux du stockage : lister `captations/` (un refus rend `Erreur::Disque`, sur lui, avant tout nom
  provisoire) ; `voisin(captations/<base>)` ; `Etape::VoisinEcrit` ; chaque tronçon, `Etape::Troncon` puis `write_all`
  et `Hacheur::ajouter` ; `Etape::SyncAll` ; la relecture en flux contre l'empreinte ; une empreinte autre
  qu'`attendue`, le retrait, `rangee` faux ; déjà rangée, le retrait ; sinon le sous-dossier, `Etape::Renommage`,
  `Etape::RendreDurable`. Un refus avant la fin du renommage retire le nom provisoire (`captations.rs:67`, par
  `Etape::Retrait`) : un test qui simule une coupure du processus, après laquelle le nom provisoire reste, coupe aussi
  `Etape::Retrait` le temps du seul versement coupé, comme `demarrage.rs:356` (R1). Une erreur du flux (le fichier à verser ne se lit plus) rend `Raison::Disque`, son message « La lecture
  du fichier à verser a échoué : <erreur>. ». La voie par octets garde son raccourci : une captation déjà rangée n'est
  ni réécrite ni rouverte. Le démarrage lit chaque nom provisoire en flux (`lire_un_nom_provisoire`), sans le tenir en
  mémoire (vigilance 3). Les messages : `CaptationAbsente` « Cette captation n'est pas dans le fonds. »,
  `CaptationAlteree` « Une captation relue ne donne pas son empreinte. » (question 26).

- [ ] **Étape 1 : écrire les tests.** Dans `crates/stockage/tests/captations.rs` (un contenu de 2 621 440 octets,
  trois tronçons, engendré par le test) :
  - `verser_par_octets_passe_par_le_flux` : sous `couper(Etape::Troncon, 1)`, `Captations::verser(octets)` est refusé,
    et aucune captation n'est rangée ;
  - `i14_la_copie_en_flux_coupee_a_chaque_etape_ne_laisse_aucune_captation_alteree` : pour chaque coupure,
    `VoisinEcrit`, `Troncon` aux rangs 1, 2 et 3, `SyncAll`, `Relecture`, `Renommage`, `RendreDurable`, la copie est
    refusée (ou faite pour `RendreDurable`, la captation rangée entière) ; `Captations::verifier()` est vide ; aucune
    captation n'est réécrite ;
  - `une_copie_en_flux_alteree_a_la_relecture_est_refusee` : sous `alterer_la_relecture(1)`,
    `Erreur::CaptationAlteree`, aucun nom provisoire ne reste ;
  - `une_copie_deja_rangee_retire_son_nom_provisoire` : deux copies du même contenu, la seconde `deja_rangee`, aucun
    `.partiel` dans `captations/` ;
  - `une_copie_qui_ne_donne_pas_l_empreinte_attendue_ne_range_rien` (R20) : `verser_en_flux` d'un contenu, `attendue`
    l'empreinte d'un autre ; `rangee` faux, l'empreinte calculée rendue, aucune captation ne la porte, aucun `.partiel` ;
  - `relire_une_plage_rend_ses_octets` : sur un contenu `(0..=255).cycle()` de 1 000 octets, la plage (10, 5) rend
    `[10, 11, 12, 13, 14]`, la plage (998, 10) rend `[230, 231]`, la plage (1 000, 4) rend `[]` ; `taille` rend
    `Some(1_000)`, et `None` d'une empreinte absente ; `tailles` rend `{1_000}`.

  Dans `crates/services/tests/commandes_en_memoire.rs` :
  - `une_copie_en_memoire_qui_ne_donne_pas_l_empreinte_attendue_ne_range_rien` (V7) : le même cas sur
    `CaptationsEnMemoire` ; `rangee` faux, l'empreinte calculée rendue, `taille` de cette empreinte à `None`.

  Dans `crates/stockage/tests/demarrage.rs`, chaque coupure étant celle du processus : l'étape visée coupée, et
  `couper(Etape::Retrait, 1)` armé avec elle, le temps du seul versement coupé (R1) :
  - `la_copie_en_flux_coupee_entre_deux_troncons_ne_se_range_pas` : une copie `import-<I>-3` coupée à `Troncon` rang 2,
    le retrait coupé aussi, laisse un nom provisoire de 1 048 576 octets ; un fonds qui nomme l'empreinte du contenu
    entier se rouvre ; le nom provisoire reste, non rangé, et aucune captation ne porte l'empreinte ;
  - `une_copie_en_flux_entiere_et_nommee_se_range_au_demarrage` : coupée à `Renommage`, le retrait coupé aussi,
    entière, nommée par le fonds, elle se range à la réouverture ;
  - `un_stem_nomme_prend_son_nom_au_demarrage` : un fonds dont une prise en stems nomme le stem `S` (les changements de
    `import::verser_une_prise_en_stems`, appliqués) ; la copie de `S` coupée à `Renommage`, le retrait coupé aussi, se
    range à la réouverture (P1, le cas du test 4).

  Dans `crates/stockage/tests/source.rs` :
  - `le_parcours_range_les_entrees_et_compte_les_octets_lus` : un dossier `b.txt`, `a/c.wav`, `a/d.mid` (octets du
    test) rend les rangs 1 `a/c.wav`, 2 `a/d.mid`, 3 `b.txt`, leurs extensions et leurs octets ; `lire(1, 0, 16)` rend
    16 octets et `octets_lus(1)` vaut 16 ; une lecture en flux entière y ajoute la taille du fichier ;
  - `un_dossier_absent_ne_se_parcourt_pas` : `Erreur::Disque` ;
  - `un_lien_symbolique_compte_sans_etre_suivi` (`#[cfg(unix)]`, `std::os::unix::fs::symlink` vers un dossier) : une
    entrée `SorteDEntree::Lien`, et rien de ce qu'il vise ;
  - `la_marque_suit_la_derniere_ecriture` (R8) : un fichier daté par le test (`File::set_modified`, dans le test
    seul, hors de `src/`) ; sa marque de dernière écriture, présente, égale `marque_actuelle` ; le même fichier récrit à la même taille et
    daté d'une seconde plus tard : `marque_actuelle` diffère de celle du parcours.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la copie en flux, la source à verser et leurs coupures`),
  pousser, ouvrir la PR en brouillon. Attendu à « Lint » de `verifier` : `no variant named Troncon` et `cannot find
  type SourceSurFichiers`.
- [ ] **Étape 3 : écrire le code.** Les quatre sites du contrat ensemble (P6) : `contrats.rs`, `adaptateurs.rs`,
  `en_memoire.rs`, `captations.rs` ; `CaptationsEnMemoire` copie le flux dans un `Vec`, calcule l'empreinte par
  `Hacheur`, rend `deja_rangee` d'une empreinte présente, et, `attendue` autre que l'empreinte calculée, ne range rien,
  `rangee` faux (V7). `SourceEnMemoire` emploie `signature::extension`, tient une marque de dernière écriture par entrée
  (1 à la création, avancée par `changer`) et les entrées refusées par `refuser`, tant qu'elles le sont. Aucun mot des
  contrôles statiques des services (`File`, `fs::`, `spawn`, `time::`, `SystemTime`) dans `contrats.rs`,
  `source_en_memoire.rs` ni `en_memoire.rs` ; aucun mot d'`HORLOGE` dans `stockage/src/source.rs`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `captations.rs`, `demarrage.rs` et `source.rs`
  du stockage verts, la suite des commandes inchangée, le test neuf de `commandes_en_memoire.rs` vert, `statique.rs`
  des services vert ; dans `windows`, les mêmes
  fichiers du stockage verts.
- [ ] **Étape 5 : committer** `git add crates/services/src/contrats.rs crates/services/src/en_memoire.rs
  crates/services/src/source_en_memoire.rs crates/services/src/lib.rs crates/stockage/src/source.rs
  crates/stockage/src/captations.rs crates/stockage/src/fichier.rs crates/stockage/src/pannes.rs
  crates/stockage/src/adaptateurs.rs crates/stockage/src/lib.rs crates/stockage/tests/captations.rs
  crates/stockage/tests/demarrage.rs crates/stockage/tests/source.rs crates/services/tests/commandes_en_memoire.rs`
  (`feat: la copie en flux des captations et la source à verser, sur fichiers et en mémoire`).

### Tâche 10 : la lecture du fonds importé ; I01 étendu

**Fichiers :** modifier `crates/services/src/lecture.rs`, `crates/services/tests/suite/mod.rs` (`permis`,
`interdit`), `crates/stockage/tests/demarrage.rs`.

**Interfaces — consomme :** `idee::empreintes_nommees` (tâche 7), `origine::{OriginesLues, Declaration,
premier_nom_vu}`, `stems::{TypeDeStem, duree_de_la_prise_en_stems}`, `import::{DejaLu, DejaDeLaPrise}`.

**Interfaces — produit** (`services::lecture`) :

```rust
/// Les noms d'origine du contenu `empreinte` sous `conteneur` (ou, `stem` donné, sous l'entrée de
/// ce stem dans la prise `empreinte`) : chaque couple (chemin, ULID), chaque valeur d'une clé lue
/// par `get_all` (question 20).
pub fn noms_de(doc: &Automerge, conteneur: &str, empreinte: &str, stem: Option<&str>) -> Vec<(String, String)>;
/// Le premier nom vu d'un contenu, d'après `origine::premier_nom_vu`, cherché sous les trois
/// conteneurs, puis sous l'entrée de chaque stem.
pub fn premier_nom_vu(doc: &Automerge, empreinte: &str) -> Option<String>;
/// L'ensemble des origines du contenu `empreinte`.
pub fn origines_de(doc: &Automerge, empreinte: &str) -> OriginesLues;
/// La même lecture à la tête `tete` (`keys_at`, `get_at`).
pub fn origines_a(doc: &Automerge, empreinte: &str, tete: &[ChangeHash]) -> OriginesLues;
/// Ce que le fonds porte du contenu `empreinte` : nommé par le fonds, d'après `nommees`, les
/// empreintes nommées que l'état de l'import lit une fois à l'ouverture et tient à jour à chaque
/// versement (R26) ; ses noms sous le conteneur de `sorte` ; ses origines.
pub fn deja_lu(doc: &Automerge, sorte: Sorte, empreinte: &str, nommees: &BTreeSet<String>) -> DejaLu;
/// Les noms qu'a posés l'import `import` (la valeur de la clé, son ULID ; question 20) : ceux des
/// fichiers, `<e>/nom/<chemin>` sous `prises` (hors d'une prise marquée `manifeste`), `midi_verses`
/// et `textes_verses`, et `<p>/stem/<s>/nom/<chemin>` ; et, à part, ceux de ses prises en stems
/// (R2). Chaque nom de fichier va avec son empreinte, et si sa captation n'a pas de `format` (un
/// illisible, R7).
pub fn noms_poses_par(doc: &Automerge, import: &str) -> NomsPoses;
/// Ce que le fonds porte d'une prise en stems, de ses stems et de leurs MIDI.
pub fn deja_de_la_prise(doc: &Automerge, manifeste: &str, stems: &[&str], midis: &[&str]) -> DejaDeLaPrise;
/// Les stems de la prise `prise`, dans l'ordre de leurs empreintes : leurs faits, leurs noms, leur
/// type gardé ou choisi, avec sa provenance, et leur MIDI.
pub fn stems_de(doc: &Automerge, prise: &str) -> Vec<StemLu>;
/// Les imports, chacun avec l'heure de son ouverture et, clos, l'empreinte de son rapport.
pub fn imports(doc: &Automerge) -> Vec<ImportLu>;
/// Les déclarations de l'import `import` ; les dernières de chaque chemin, d'après l'import le plus
/// récent qui en a fait une (question 9).
pub fn declarations_de(doc: &Automerge, import: &str) -> BTreeMap<String, Declaration>;
pub fn dernieres_declarations(doc: &Automerge) -> BTreeMap<String, Declaration>;
/// Les noms de générateurs que porte le fonds, chacun une fois, dans l'ordre de leurs octets.
pub fn generateurs(doc: &Automerge) -> BTreeSet<String>;
/// Les empreintes nommées à la tête `tete` : les captations connues à l'ouverture d'un import.
pub fn empreintes_nommees_a(doc: &Automerge, tete: &[ChangeHash]) -> BTreeSet<String>;
// empreintes_nommees(doc) passe désormais les valeurs de `imports/<i>/rapport` à la règle du noyau.
// faits_de_la_sorte : d'une prise en stems (`<p>/manifeste`), la durée est
// `duree_de_la_prise_en_stems` des durées de ses stems ; une entrée de `midi_verses` sans `idee`
// (le MIDI d'un stem) n'est la captation d'aucune idée.
```

  `StemLu`, `ImportLu`, `NomsPoses` (`fichiers: BTreeMap<String, (String, bool)>`, chemin vers (empreinte,
  illisible) ; `jeux: BTreeSet<String>`) : des structures de `lecture`, aux champs nommés par leurs docs. `permis` (I01) admet, sous une
  captation : `idee`, `format`, `duree`, `codage`, `canaux`, `frequence`, `bits`, `division`, `manifeste`,
  `nom/<chemin>`, `stem/<s>/format`, `stem/<s>/duree`, `stem/<s>/nom/<chemin>`, `stem/<s>/midi/<m>`, `stem/<s>/type`,
  `stem/<s>/type/provenance`, `stem/<s>/type/par`, `stem/<s>/type/depuis`, `stem/<s>/type/le` ; sous `imports` :
  `<i>/ouvert`, `<i>/rapport`, `<i>/declaration/<chemin>` ; sous `origines_declarees` : `<e>/main`,
  `<e>/generee/<nom>`. `interdit` ne lit ni le chemin qui suit `nom`, ni celui qui suit `declaration`, ni le nom qui
  suit `generee`.

- [ ] **Étape 1 : écrire les tests.** Dans la suite (`crates/services/tests/suite/mod.rs`), rejoués par les deux
  paires, les noms dans `commandes_en_memoire.rs` et `commandes_sur_fichiers.rs` :
  - `i01_un_fonds_importe_ne_garde_ni_compte_ni_total` : un fonds où les opérations de la tâche 6 ont écrit un fichier
    de chaque sorte, une prise en stems, un type gardé, une origine générée sous un nom `total`, un nom d'origine
    `compte/a_envoyer.wav`, un rapport ; `parcourir` ne refuse rien, et une clé `prises/<e>/total` plantée est refusée ;
  - `la_duree_d_une_prise_en_stems_vient_de_ses_stems` : la vue de l'idée d'une prise en stems de deux stems (1 000 et
    1 500 ms) porte 1 500 ms ; le MIDI d'un stem n'est pas une captation de l'idée ; une requête au contenu « MIDI
    versé » ne rend pas l'idée de cette prise en stems, dont un stem porte un MIDI, et rend celle d'un MIDI versé seul
    (question 21 ; R11) ;
  - `les_noms_d_un_contenu_et_son_premier_nom_vu` : deux imports, trois noms, le premier vu rendu ;
  - `les_noms_poses_par_un_import` : un import d'un WAV, d'un WAV illisible, d'un texte et d'un jeu de deux stems ;
    `noms_poses_par` rend les quatre fichiers et leurs stems, l'illisible marqué, et le jeu à part ; un second import du
    même dossier n'en pose aucun.
  Dans `crates/stockage/tests/demarrage.rs` : `un_rapport_nomme_prend_son_nom_au_demarrage` (le cas du test 4 sur un
  rapport, coupé à `Renommage`, le retrait coupé aussi, comme `un_stem_nomme_prend_son_nom_au_demarrage` ; R1).
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la lecture d'un fonds importé, et I01 sur ses clés`), pousser.
  Attendu à « Tests » de `verifier` : `i01_un_fonds_importe_ne_garde_ni_compte_ni_total` `panicked at`, la clé
  `prises/<e>/codage` refusée par `permis`.
- [ ] **Étape 3 : écrire le code.** Les lectures par `map_range` en un passage, comme `faits_de_la_sorte` ; les clés à
  plat se coupent par `splitn` à leur place (le chemin d'un nom garde ses `/`).
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : la suite des commandes, sur les deux paires,
  verte ; `demarrage.rs` vert ; dans `windows`, `commandes_sur_fichiers.rs` et `demarrage.rs` verts.
- [ ] **Étape 5 : committer** `git add crates/services/src/lecture.rs crates/services/tests/suite/mod.rs
  crates/services/tests/commandes_en_memoire.rs crates/stockage/tests/commandes_sur_fichiers.rs
  crates/stockage/tests/demarrage.rs` (`feat: la lecture d'un fonds importé ; I01 sur les clés de l'import`).

### Tâche 11 : les commandes de l'import

**Fichiers :** modifier `crates/commandes/src/lib.rs`, `crates/commandes/tests/serialisation.rs`.

**Interfaces — produit** (`commandes`), chaque type `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`, chaque
entier borné par `ENTIER_SUR` comme ceux de la tranche 2 :

```rust
pub const INSPECTER_UN_DOSSIER: &str = "inspecter_un_dossier";
pub const OUVRIR_L_IMPORT: &str = "ouvrir_l_import";
pub const VERSER_UN_FICHIER: &str = "verser_un_fichier";
pub const VERSER_UNE_PRISE_EN_STEMS: &str = "verser_une_prise_en_stems";
pub const CLORE_L_IMPORT: &str = "clore_l_import";
pub const DECLARER_UNE_ORIGINE: &str = "declarer_une_origine";
pub const CHOISIR_LE_TYPE_D_UN_STEM: &str = "choisir_le_type_d_un_stem";
pub const LIRE_UN_TEXTE_VERSE: &str = "lire_un_texte_verse";
pub const LIRE_UN_RAPPORT: &str = "lire_un_rapport";
pub const GENERATEURS: &str = "generateurs";
// NOMS : ces dix après SUPPRIMER_UNE_REQUETE, avant EXPORTER_CAHIER ; 31 noms.

pub enum NatureDuFichier { Audio { format: String }, Midi, Texte { format: String }, Ignore }
pub struct FichierInspecte { pub rang: u64, pub chemin: Option<String>, pub octets: u64, pub nature: NatureDuFichier, pub connu: bool }
pub enum Declaration { Stems, Main, Generee { nom: String } }
pub struct DossierInspecte { pub chemin: String, pub audio: u64, pub jeu_propose: bool, pub reproposee: Option<Declaration> }
pub struct Inspection {
    pub reprise: Option<String>, pub nom_du_dossier: Option<String>, pub fichiers: Vec<FichierInspecte>,
    pub dossiers: Vec<DossierInspecte>, pub a_verser: u64, pub libre: u64, pub manque: Option<String>, pub generateurs: Vec<String>,
    /// Les déclarations de l'import repris, que l'écran montre sans choix (R2).
    pub declarations_reprises: Vec<DeclarationDUnDossier>,
    /// Un import est ouvert, que ce dossier ne reprend pas : l'écran le dit (« Un import est resté
    /// ouvert, que ce dossier ne reprend pas : verser celui-ci le clôt, avec ce qu'il a versé. »), et
    /// l'ouverture le clora (question 35 ; V12).
    pub ouvert_ailleurs: bool,
}
pub struct DeclarationDUnDossier { pub chemin: String, pub declaration: Declaration }
pub struct OuvrirLImport { pub declarations: Vec<DeclarationDUnDossier>, pub decalage_minutes: i32 }
pub struct JeuAVerser { pub dossier: String, pub rangs: Vec<u64> }
pub struct ImportOuvert { pub import: String, pub repris: bool, pub rangs: Vec<u64>, pub jeux: Vec<JeuAVerser>, pub avertissement: Option<String> }
pub struct VerserUnFichier { pub import: String, pub rang: u64, pub decalage_minutes: i32 }
pub enum Issue { Verse, Dedoublonne, Stem, DejaFait }
pub struct FichierVerse { pub rang: u64, pub issue: Issue, pub avertissement: Option<String> }
pub struct VerserUnePriseEnStems { pub import: String, pub dossier: String, pub decalage_minutes: i32 }
pub struct CloreLImport { pub import: String, pub decalage_minutes: i32 }
pub struct Compte { pub fichiers: u64, pub octets: u64 }
pub struct RapportDImport {
    pub import: String, pub vus: Compte, pub verses: Compte, pub dedoublonnes: Compte, pub illisibles: Compte,
    pub ignores: Compte, pub sans_nom: Compte, pub origines_changees: u64, pub fichiers_sans_nom: Vec<u64>,
}
pub struct ImportClos { pub rapport: RapportDImport, pub avertissement: Option<String> }
pub enum Portee { Captation(Captation), Prefixe { import: String, prefixe: String } }
pub enum OrigineADeclarer { Aucune, Main, Generee { nom: String } }
pub struct DeclarerUneOrigine { pub portee: Portee, pub origine: OrigineADeclarer }
pub enum TypeDeStem { Voix, Choeurs, Batterie, Percussions, Basse, Guitare, Clavier, Cordes, Cuivres, Synthe, Effets, Autre }
pub enum ChoixDuType { Garder, Choisir(TypeDeStem) }
pub struct ChoisirLeType { pub prise: String, pub stem: String, pub choix: ChoixDuType }
pub struct TexteLu { pub texte: Option<String> }   // None : un texte illisible
pub struct LireUnRapport { pub import: String }
pub enum OrigineDeclaree { Aucune, Main, Generee { noms: Vec<String> } }
pub struct FaitsDUneCaptation { pub format: Option<String>, pub duree_ms: Option<u64>, pub codage: Option<u16>, pub canaux: Option<u16>, pub frequence: Option<u32>, pub bits: Option<u16>, pub division: Option<u16> }
pub enum TypeVu { Propose(TypeDeStem), Garde(TypeDeStem), Choisi(TypeDeStem) }
pub struct MidiVu { pub empreinte: String, pub faits: FaitsDUneCaptation, pub noms: Vec<String> }
pub struct StemVu { pub empreinte: String, pub faits: FaitsDUneCaptation, pub noms: Vec<String>, pub type_de_stem: TypeVu, pub midi: Option<MidiVu> }
pub struct CaptationVue { pub captation: Captation, pub faits: FaitsDUneCaptation, pub noms: Vec<String>, pub origine: OrigineDeclaree, pub stems: Vec<StemVu> }
// VueDUneIdee gagne `captations_vues: Vec<CaptationVue>`, une par captation, dans l'ordre de `captations` ;
// LigneDeLaBoite gagne `nom_de_fichier: Option<String>` : sans texte, le premier nom vu de sa
// première captation (question 4).
```

  Les noms d'une `CaptationVue` et d'un `StemVu` : le premier vu d'abord, puis les autres dans l'ordre de leurs octets.

- [ ] **Étape 1 : écrire les tests** (`crates/commandes/tests/serialisation.rs`) : `les_noms_sont_uniques_et_en_snake_case`
  passe à `assert_eq!(NOMS.len(), 31)`, et `NOMS[15..]` nomme `ETAT_DU_FONDS`, les quatre commandes des requêtes
  enregistrées, les dix de l'import dans l'ordre ci-dessus, puis `EXPORTER_CAHIER` ; un aller-retour JSON de chaque type
  neuf (`les_types_de_l_import_font_l_aller_retour`), dont un chemin `None`, un nom de générateur à apostrophe et
  `TypeVu::Propose(TypeDeStem::Choeurs)` ; les entiers au-delà d'`ENTIER_SUR` refusés à la relecture, comme les autres
  (`un_compte_au_dela_d_entier_sur_ne_se_relit_pas`).
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les commandes de l'import`), pousser. Attendu à « Lint » :
  `cannot find value INSPECTER_UN_DOSSIER`.
- [ ] **Étape 3 : écrire le code.** Les types et les dix constantes ; `NOMS`. Les services, l'établi et l'interface
  qui construisent `VueDUneIdee` et `LigneDeLaBoite` passent `captations_vues: Vec::new()` et `nom_de_fichier: None` jusqu'aux
  tâches 12 et 24 ; les droits de Windows et `generate_handler!` suivent à la tâche 15 : `windows` reste rouge sur I31
  entre les deux, ce que la PR dit.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `serialisation.rs` vert ; dans `windows`, les
  tests d'I31 rouges (`allow-inspecter-un-dossier` manque), jusqu'à la tâche 15.
- [ ] **Étape 5 : committer** `git add crates/commandes/src/lib.rs crates/commandes/tests/serialisation.rs` et les
  constructeurs touchés (`feat: les types et les dix commandes de l'import`).

### Tâche 12 : les services de l'import : inspecter, ouvrir, verser, clore

**Fichiers :** créer `crates/services/src/etat_de_l_import.rs` (l'état de l'import et ses conversions) ; modifier
`crates/services/src/lib.rs`, `crates/services/tests/suite/mod.rs`, `crates/services/tests/commandes_en_memoire.rs`,
`crates/stockage/tests/commandes_sur_fichiers.rs`.

Le module des services se nomme `etat_de_l_import`, non `import` : `mod import;` et `use noyau::donnees::import;` ne
coexisteraient pas dans `crates/services/src/lib.rs` (E0255), et ce plan écrit `import::ouvrir`,
`import::verser_un_fichier`, `import::declarer_une_origine`… pour les opérations du noyau (balayage P13). Les
homonymes de `commandes` (`ImportOuvert`, `ChoixDuType`, `Compte`) et ceux du noyau se nomment par leur module, comme
`gestes::Sorte` aujourd'hui.

**Interfaces — consomme :** les règles de la PR (a) ; `Source`, `Captations::verser_en_flux` (tâche 9) ; les lectures
de la tâche 10 ; les commandes de la tâche 11.

**Interfaces — produit** (`services::Services`) :

```rust
impl<D: Documents, C: Captations> Services<D, C> {
    /// Inspecte `source` sans rien écrire : refusée en lecture seule, du refus de `lecture_seule()`.
    /// Chaque fichier lu de `SIGNATURE_AU_PLUS` octets (`reconnaitre`) ; une étiquette ID3, de quatre
    /// de plus à sa suite (`apres_id3`) ; un WAV, de `ENTETE_AU_PLUS` (`faits_wav`) ; un lien, un
    /// ignoré, jamais au-delà. Une lecture refusée arrête l'inspection et nomme le fichier, de
    /// `Raison::Disque` : « « x.wav » ne se lit pas : <erreur>. », sans nom « Le fichier n° <rang> ne
    /// se lit pas : <erreur>. » (question 38). L'empreinte d'un fichier audio, MIDI ou texte ne se
    /// calcule que si sa taille est dans `Captations::tailles` (question 8) ; connu, si le fonds le
    /// nomme. La taille à verser, ses fichiers, le poids du fichier complet (`Automerge::save`, 0 sans
    /// fonds) et `place_libre` : `rapport::place` ; le manque, son texte. Chaque dossier : ses fichiers
    /// audio, le jeu proposé (`jeu_propose`, sur les tailles et les en-têtes lus, sans empreinte ;
    /// question 37), la déclaration reproposée (`dernieres_declarations`). La reprise (question 2 ;
    /// R2) : l'import ouvert dont au moins un nom de fichier est posé (`noms_poses_par`), et dont chaque
    /// nom de fichier est le chemin d'une entrée de la source, et chaque nom de prise en stems le
    /// préfixe, par segments entiers (`<nom>/`), d'une entrée, ou `nom_du_dossier` ; elle rend les
    /// déclarations de son ouverture (`declarations_de`). Un import ouvert que la source ne reprend
    /// pas : `ouvert_ailleurs` (question 35). L'inspection se garde dans les services, avec la marque de dernière écriture
    /// de chaque entrée et chaque empreinte calculée, jusqu'à l'ouverture ou à une autre.
    pub fn inspecter<S: Source>(&mut self, source: &S, place_libre: u64) -> Reponse<Inspection>;
    /// Ouvre l'import inspecté (question 15) : en lecture seule, le refus ; sans inspection,
    /// « Aucun dossier n'est inspecté. » ; la place, de nouveau comparée, son manque en refus ; un
    /// dossier déclaré `stems` sans fichier audio, `JeuSansAudio`. Un import ouvert que la source ne
    /// reprend pas se clôt d'abord (question 35) : son rapport compte les fichiers nommés à son ULID
    /// (`noms_poses_par`), vus et versés, leurs octets ceux de leurs captations
    /// (`Captations::taille`), les illisibles parmi eux, dédoublonnés, ignorés et sans nom à zéro, sans
    /// ligne `origine=` ; « Import du <jour de son ouverture> clos » ; ses copies retirées. Neuf :
    /// l'ULID tiré, puis `import::ouvrir`, par `faire_naitre` dans un Pupitre neuf (la naissance tirée
    /// d'abord, comme `noter`), sinon `appliquer` ; la tête qui suit, gardée. Repris : rien ne s'écrit ;
    /// les déclarations sont celles de l'ouverture, et une commande qui en porte d'autres est refusée,
    /// « Cet import reprend ses déclarations : elles ne changent pas. » ; la tête de l'ouverture,
    /// retrouvée par l'opération qui a posé `imports/<i>/ouvert` (`get_all`, puis `hash_for_opid`) ;
    /// chaque fichier dont un nom porte l'ULID de l'import, compté de nouveau sans être relu,
    /// illisible si sa captation n'a pas de `format`, ses octets ceux de son entrée (R7) ; puis ses
    /// copies non nommées, retirées (`copies_a_retirer`, à la reprise). Les empreintes nommées, lues une
    /// fois et gardées dans l'état de l'import (R26). Rend les rangs à verser, hors des jeux, et chaque
    /// jeu avec les siens.
    pub fn ouvrir_l_import<S: Source>(&mut self, source: &S, commande: &OuvrirLImport, place_libre: u64, contexte: &mut Contexte<'_>) -> Reponse<ImportOuvert>;
    /// Verse le fichier `rang` de l'import ouvert : un autre import, `AucunImportOuvert` ; en lecture
    /// seule, le refus ; déjà compté, `Issue::DejaFait`. Ses octets ou sa marque de dernière écriture changés depuis
    /// l'inspection (`octets_actuels`, `marque_actuelle`, lus avant la copie, puis `marque_actuelle` relue
    /// après), ou son empreinte autre que celle qu'a calculée l'inspection (passée en `attendue` à
    /// `verser_en_flux`, qui ne range rien en désaccord ; R20) : `FichierChange` (question 14 ; R8).
    /// Une marque de dernière écriture absente ne compare rien. La copie en flux, sous `nom_de_la_copie`, le flux contrôlé
    /// (ses octets comptés, un texte par `ControleDuTexte`) ; ses faits, ceux de l'en-tête lu à
    /// l'inspection, ou aucun (illisible). Un fichier audio d'un jeu, et le MIDI d'un de ses stems
    /// (`stem_du_midi`) : la captation seule, `Issue::Stem`. Sinon `import::verser_un_fichier`, l'ULID
    /// de l'idée tiré pour un contenu neuf seulement ; `Issue::Verse` ou `Issue::Dedoublonne` ; les
    /// empreintes nommées de l'état, tenues à jour.
    pub fn verser_un_fichier<S: Source>(&mut self, source: &S, commande: &VerserUnFichier, contexte: &mut Contexte<'_>) -> Reponse<FichierVerse>;
    /// Verse la prise en stems du jeu `dossier`, chacun de ses stems déjà copié (sinon « Les stems de
    /// ce jeu ne sont pas tous versés. ») : le manifeste (`rapport::manifeste`) par
    /// `Captations::verser`, puis `import::verser_une_prise_en_stems`, son nom le chemin du dossier,
    /// ou `Source::nom_du_dossier` pour le dossier versé ; sans nom en UTF-8, aucun, et aucune ligne
    /// `fichier-sans-nom` au rapport, qui ne nomme que le rang d'un fichier (R23).
    pub fn verser_une_prise_en_stems<S: Source>(&mut self, source: &S, commande: &VerserUnePriseEnStems, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
    /// Clôt l'import, chaque fichier et chaque jeu versés (sinon « Des fichiers restent à verser. ») :
    /// le rapport (`rapport::rapport`), les connues lues à la tête de l'ouverture
    /// (`empreintes_nommees_a`), chaque origine changée par comparaison d'`origines_a` et
    /// d'`origines_de` ; versé par `Captations::verser` ; `import::clore` ; chaque copie de l'import
    /// retirée (`copies_a_retirer`, à la clôture). L'état de l'import oublié, les services gardent la
    /// table (chemin → empreinte) de chaque fichier versé ou dédoublonné de l'import clos, jusqu'à
    /// l'inspection suivante : `Portee::Prefixe` se résout sur elle (R6 ; tâche 13).
    pub fn clore_l_import<S: Source>(&mut self, source: &S, commande: &CloreLImport, contexte: &mut Contexte<'_>) -> Reponse<ImportClos>;
}
// La vue d'une idée porte ses captations vues (`CaptationVue`), et chaque ligne de la boîte, sans
// texte, le premier nom vu de sa première captation.
```

  La suite gagne, dans `Paire`, `type S: Source`, `fn source(&mut self, nom_du_dossier: Option<&str>, fichiers:
  &[(Option<&str>, Vec<u8>)], liens: &[&str]) -> Self::S`, `fn changer(&mut self, source: &mut Self::S, rang: u64,
  contenu: &[u8])` et `fn refuser_la_lecture(&mut self, source: &mut Self::S, rang: u64, refusee: bool)` (V2) : en
  mémoire, `SourceEnMemoire` (`changer`, `refuser`) ; sur fichiers, un dossier jetable écrit par le test
  (`commun::dossier_jetable`), un chemin `None` écrit sous un nom sans forme UTF-8 (`OsString::from_wide` d'une
  demi-paire seule sous Windows, `OsStr::from_bytes` d'un octet `0xFF` ailleurs), puis `SourceSurFichiers::parcourir` ;
  `changer` y récrit le fichier, puis le date d'une seconde après sa marque de dernière écriture (`File::set_modified`,
  dans le test), pour ne pas dépendre de la finesse de l'horloge du système de fichiers ; `refuser_la_lecture(…, true)`
  garde dans un champ de la paire le fichier ouvert sans partage (`OpenOptionsExt::share_mode(0)`) sous Windows, ou ôte
  ses droits (`0o000`) ailleurs, puis vérifie qu'une ouverture du fichier échoue, et sinon panique en le disant (« les
  droits ôtés ne refusent pas la lecture : le test tourne-t-il sous root ? » ; V14) ; `refuser_la_lecture(…, false)`
  referme ce fichier, ou rend ses droits.
  Les aides `wav(frequence, canaux, bits, trames)`, `midi(format, pistes, division)`, `m4a()`, `zip(octets)`,
  `pdf(octets)` écrivent leurs octets à l'exécution (P12). Une aide `importer(paire, services, source, declarations)`
  conduit un import entier, unité par unité, comme l'interface (tâche 22). Chaque test de la suite a son `#[test]` dans
  `commandes_en_memoire.rs` et dans `commandes_sur_fichiers.rs`.

- [ ] **Étape 1 : écrire les tests** (la suite), chacun calculant l'exemple du critère :
  - `cap03_1_quatre_vingts_fichiers_dont_trois_doublons_donnent_soixante_dix_sept_captations` : 77 WAV de 8 000 Hz,
    mono, 8 bits, de 1 à 77 trames (rang r : r trames), puis trois copies des trois premiers dans `Copies/` ; importés,
    77 idées, chacune `Importee { depuis: <l'import> }` ; 78 captations (77 et le rapport) ; le rapport : vus 80, versés
    77, dédoublonnés 3, et les octets de chaque catégorie, calculés par le test d'après les tailles ;
  - `cap03_2_le_meme_dossier_importe_deux_fois_n_ajoute_que_son_rapport` : le même dossier, un second import ; ni idée
    ni captation de plus que son rapport ; toute clé neuve du document est sous `imports/<second import>/` ; le second
    rapport : vus 80, versés 0, dédoublonnés 80 ;
  - `cap03_3_un_nom_d_origine_se_relit_intact` : `Démo d'hiver 2 - 01_06_2015 18.42.m4a` (une boîte `ftyp`), versé en
    prise `m4a` ; son premier nom vu, relu, égale ce nom octet pour octet ;
  - `cap03_4_des_noms_semblables_ne_creent_ni_lien_ni_regroupement` (part sans morceaux, non inscrite) : `Pont v1.wav`,
    `Pont v2.wav`, `Pont.mid`, `Pont.txt` donnent quatre idées, une captation chacune, aucun lien ;
  - `cap03_7_un_jeu_de_dix_wav_et_neuf_midi_donne_une_prise_en_stems` : `Jeu A/` déclaré `stems`, dix WAV de 8 000
    trames à 8 000 Hz aux contenus distincts, neuf MIDI aux noms de neuf d'entre eux ; une prise, une idée, dix stems
    dont neuf avec leur MIDI, chaque type `TypeVu::Propose`, aucune clé `type` ; 21 captations (dix, neuf, le
    manifeste, le rapport) ; le rapport compte 19 versés ; un second import ne verse que son rapport ;
  - `cap03_8_trois_wav_de_meme_duree_proposent_un_jeu_et_sans_declaration_trois_prises` (livré par part, inscrit dans
    la PR (d) ; R24) : l'inspection dit `jeu_propose` du dossier ; ouvert sans déclaration, trois prises, trois idées ;
  - `un_premier_import_ne_lit_que_signatures_et_en_tetes` (R4 ; question 37 ; V1) : dans un Pupitre neuf, dans
    `Jeu A/`, trois WAV de 100 000 trames, 8 000 Hz, mono, 8 bits (100 044 octets chacun), leurs données distinctes dès
    leur premier octet (remplies de 1, de 2 et de 3), un texte et un MIDI ; après l'inspection, `Source::octets_lus` de
    chaque WAV vaut au plus `SIGNATURE_AU_PLUS + ENTETE_AU_PLUS`, moins que 100 044 (aucun WAV n'est lu en entier),
    celui du texte et du MIDI au plus `SIGNATURE_AU_PLUS`, et le jeu est proposé ; puis, dans `Jeu B/`, deux WAV de
    100 044 octets aux 65 536 premiers octets égaux, qui ne diffèrent qu'au-delà : aucun jeu n'est proposé (la lettre de
    la question 37), et leurs `octets_lus` restent sous la même borne ;
  - `un_fichier_qui_ne_se_lit_pas_arrete_l_inspection` (question 38) : un WAV `Démos/a.wav` dont la lecture est
    refusée (`refuser_la_lecture(…, true)`) ; l'inspection rend `Raison::Disque`, son message commence par
    « « Démos/a.wav » ne se lit pas : » ; rien ne s'écrit ; puis `refuser_la_lecture(…, false)`, et la même inspection
    passe (V2) ;
  - `un_jeu_d_un_seul_contenu_donne_une_prise_en_stems_distincte` (question 19 ; R11) : un WAV versé en prise, puis
    son dossier déclaré `stems` avec ce seul WAV ; deux prises, l'empreinte du manifeste distincte de celle du WAV ;
  - `cap03_10_un_contenu_sous_deux_noms_dans_deux_sous_dossiers_est_un_objet_a_deux_noms` : `A/x.wav` et `B/y.wav` du
    même contenu ; une captation, ses noms `["A/x.wav", "B/y.wav"]` ;
  - `cap03_11_une_archive_et_un_pdf_sont_ignores_sans_etre_ouverts` : un zip (`PK\x03\x04` puis 5 000 octets), un PDF
    (`%PDF-1.7` puis 2 000 octets) et un WAV ; le rapport compte deux ignorés, 5 004 + 2 008 octets ;
    `Source::octets_lus` de chacun vaut 16 au plus ;
  - `cap03_13_une_taille_a_verser_plus_grande_que_la_place_libre_ne_verse_rien` : un WAV de 8 044 octets, une place
    libre de 4 000 ; l'inspection dit « La taille à verser, 0,1 Go, passe la place libre, 0,0 Go : il manque 0,1 Go. » ;
    l'ouverture est refusée de ce message ; le fonds n'est pas né, et aucune captation n'est versée ;
  - `deux_copies_d_un_contenu_versees_dans_le_desordre_comptent_un_verse` (point de revue 1) : les rangs versés à
    rebours ; un versé, un dédoublonné, deux noms ;
  - `un_zip_nomme_wav_est_ignore_et_lu_de_seize_octets` (point de revue 2) ;
  - `un_stem_deja_prise_reste_la_prise_de_son_idee` (point de revue 3) : un WAV versé en prise, puis le même contenu
    stem d'un jeu déclaré ; la prise garde son idée, la prise en stems nomme le stem, le rapport le compte dédoublonné ;
  - `un_texte_illisible_est_verse_et_compte_a_part` : un `.txt` en Latin-1 ; un texte versé sans format ; illisibles 1 ;
  - `un_fichier_sans_nom_est_verse_et_compte_a_part` : un WAV de chemin `None` ; versé, sans clé `nom` ; sans-nom 1,
    et la ligne `fichier-sans-nom=<rang>` dans le rapport relu ;
  - `la_place_compte_deux_fois_le_poids_du_fonds` : sur un fonds né, la place libre égale à la taille, plus deux fois
    `save().len()`, plus 4 096, passe ; un octet de moins manque ;
  - `l_import_dans_un_pupitre_neuf_fait_naitre_le_fonds_par_son_ouverture` : le journal, dans l'ordre : la genèse, la
    naissance, « Import du 14 novembre ouvert », les versements, « Import du 14 novembre clos ».
- [ ] **Étape 2 : voir le rouge.** Committer (`test: l'import, CAP-03 · 1 à 3, 4 et 8 en part, 7, 10, 11 et 13`),
  pousser. Attendu à « Lint » : `no method named inspecter found`.
- [ ] **Étape 3 : écrire le code.** `services::etat_de_l_import` tient `EtatDeLImport` (l'ULID, l'heure de l'ouverture, la tête,
  les déclarations, l'inspection, ce que chaque rang a donné, les empreintes nommées, lues une fois puis tenues à
  jour ; R26) ; `Services` le garde dans un champ, `None` hors d'un import, et garde à part la table (chemin →
  empreinte) du dernier import clos, oubliée à l'inspection suivante (R6). `verser_une_fiche` ne change pas : la voie d'une capture, qui verse toujours ; sa doc le dit, et ne dit plus
  « l'import la recevra ». Aucune lecture d'un fichier hors du contrat `Source`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : la suite verte sur les deux paires, `statique.rs`
  des services vert ; dans `windows`, `commandes_sur_fichiers.rs` vert.
- [ ] **Étape 5 : committer** `git add crates/services/src/etat_de_l_import.rs crates/services/src/lib.rs
  crates/services/tests/suite/mod.rs crates/services/tests/commandes_en_memoire.rs
  crates/stockage/tests/commandes_sur_fichiers.rs` (`feat: l'import, conduit unité par unité, ses comptes et son
  rapport`).

### Tâche 13 : après l'import : déclarer une origine, choisir le type d'un stem, lire un texte et un rapport

**Fichiers :** modifier `crates/services/src/lib.rs`, `crates/services/tests/suite/mod.rs`,
`crates/services/tests/commandes_en_memoire.rs`, `crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — consomme :** `rapport::{lire_le_rapport, RapportLu}` (tâche 3 ; balayage P14), `import::{declarer_une_origine,
choisir_le_type, Cible, Declaree, ChoixDuType}` (tâche 6), `stems::type_propose` (tâche 5), `origine::sous_le_prefixe`
(tâche 4).

**Interfaces — produit** (`services::Services`) :

```rust
/// Déclare une origine (question 22) sur une captation, ou sur les captations dont un chemin de la
/// table du dernier import clos (chemin → empreinte, gardée par `clore_l_import` jusqu'à
/// l'inspection suivante ; R6) est sous `prefixe` (`sous_le_prefixe`), l'import `import` étant
/// celui de la table, sinon « Ce dossier n'est plus inspecté. » : en lecture seule, le refus ; sans
/// cible, `CaptationAbsente` ; `import::declarer_une_origine`. L'avertissement « L'origine reste
/// générée : une origine déclarée ne se retire pas. » dès que `generees_gardees` > 0, l'opération
/// faite pour les autres cibles, s'il y en a (V9) ; une origine déjà portée par chaque cible :
/// `Faite { avertissement: None }`, sans opération (R16). Ne touche pas au dernier geste de tri.
pub fn declarer_une_origine(&mut self, commande: &DeclarerUneOrigine, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
/// Choisit le type d'un stem (question 18) : `import::choisir_le_type`, la proposition tirée du
/// premier nom vu du stem (`type_propose`). Ne touche pas au dernier geste de tri.
pub fn choisir_le_type_d_un_stem(&mut self, commande: &ChoisirLeType, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
/// Le texte versé `captation`, relu contre son empreinte, décodé : UTF-8 (son BOM ôté) ou UTF-16
/// d'après son BOM ; illisible, `None`. Une lecture : en lecture seule comme ailleurs.
pub fn lire_un_texte_verse(&self, commande: &Captation) -> Reponse<TexteLu>;
/// Le rapport de l'import `import`, relu contre son empreinte, ses comptes lus par la règle du
/// noyau, `rapport::lire_le_rapport` ; `None` : `Raison::Illisible`.
pub fn lire_un_rapport(&self, commande: &LireUnRapport) -> Reponse<RapportDImport>;
/// Les noms de générateurs du fonds (question 22 : l'écran les offre, sans normaliser).
pub fn generateurs(&self) -> Reponse<Vec<String>>;
```

- [ ] **Étape 1 : écrire les tests** (la suite, sur les deux paires, sauf mention) :
  - `cap03_9_une_origine_generee_se_garde_et_ne_se_retire_pas` : un dossier déclaré `generee:Atelier inventé` ; chaque
    prise porte `OrigineDeclaree::Generee { noms: ["Atelier inventé"] }` ; la déclarer `Aucune` est refusé, « Une
    origine déclarée ne se retire pas. » ; le reverser, le dossier déclaré `main`, la laisse générée ; déclarer `Main`
    sur l'une rend l'avertissement « L'origine reste générée : une origine déclarée ne se retire pas. » ; déclarer
    `Generee { nom: "Atelier inventé" }` sur elle, déjà portée, rend `Faite { avertissement: None }` et n'ajoute rien
    au journal ; une prise d'un dossier déclaré `main`, déclarée `Main` de nouveau, de même (R16) ; `Main` déclarée par
    le préfixe d'un dossier qui porte une prise générée et une prise sans origine pose `main` sur la seconde, une
    opération au journal, et rend l'avertissement (V9) ; un dossier non déclaré : `OrigineDeclaree::Aucune` ;
  - `declarer_une_origine_par_prefixe_ne_prend_que_les_segments_entiers` : `Démos/a.wav` et `Démos 2015/b.wav` ; le
    préfixe `Démos` ne prend que `a.wav` ;
  - `declarer_par_prefixe_apres_un_import_rejoue` (R6) : un dossier `Démos/` de deux WAV importé deux fois ; depuis le
    second rapport, la déclaration `Main` par le préfixe `Démos` prend les deux captations, quand aucun nom n'est à
    l'ULID du second import ; après une nouvelle inspection, la même commande est refusée, « Ce dossier n'est plus
    inspecté. » ;
  - `garder_puis_choisir_le_type_d_un_stem` : `voix.wav` dans un jeu ; `TypeVu::Propose(Voix)`, puis
    `Garde(Voix)`, puis, choisi, `Choisi(Choeurs)` ;
  - `lire_un_texte_verse_le_rend_tel_quel` : un ChordPro UTF-8 avec BOM, un UTF-16 petit-boutiste, dont le BOM
    `FF FE` le fait reconnaître en texte, non en prise `mp3` (balayage P3), un illisible (`None`) ; un texte qui porte
    `<script>` et `&amp;` revient octet pour octet (I31) ;
  - `lire_un_rapport_rend_ses_comptes` : ceux que `clore_l_import` a rendus ;
  - `les_generateurs_du_fonds_chacun_une_fois`.
  Sur la seule paire en mémoire (`commandes_en_memoire.rs`), à deux répliques (vigilance 1) :
  - `le_premier_nom_vu_de_deux_repliques_est_le_plus_petit` : le même fichier importé sur deux répliques, à deux heures,
    puis réunies ; `get_all` rend deux valeurs de `<e>/nom/<chemin>`, et le premier vu est celui de l'import le plus
    ancien ;
  - `deux_imports_du_meme_fichier_sur_deux_repliques_laissent_une_idee_sans_captation` : réunies, deux idées, dont une
    seule porte la prise ; la piste « Ce que laissent deux regroupements croisés » le dit (tâche 17) ;
  - `deux_origines_nees_sur_deux_repliques_se_reunissent` : `main` d'un côté, `generee/Atelier` de l'autre ; la plus
    forte, générée.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: les déclarations après l'import ; CAP-03 · 9`), pousser. Attendu à
  « Lint » : `no method named declarer_une_origine found`.
- [ ] **Étape 3 : écrire le code.** Le rapport, relu contre son empreinte, se lit par `rapport::lire_le_rapport`
  (tâche 3), jamais par un second lecteur de ses lignes dans les services (balayage P14) ; `None` (une autre version,
  une ligne manquante ou mal formée), refusé de `Raison::Illisible`, « Ce rapport vient d'une version plus récente. ».
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : la suite verte sur les deux paires ; dans
  `windows`, `commandes_sur_fichiers.rs` vert.
- [ ] **Étape 5 : committer** `git add crates/services/src/lib.rs crates/services/tests/suite/mod.rs
  crates/services/tests/commandes_en_memoire.rs crates/stockage/tests/commandes_sur_fichiers.rs` (`feat: déclarer une
  origine, choisir le type d'un stem, lire un texte versé et un rapport`).

### Tâche 14 : la reprise, le fichier qui change, la lecture seule, les copies

**Fichiers :** modifier `crates/services/src/lib.rs`, `crates/services/src/etat_de_l_import.rs`,
`crates/services/tests/suite/mod.rs`, `crates/services/tests/commandes_en_memoire.rs`,
`crates/stockage/tests/commandes_sur_fichiers.rs`.

- [ ] **Étape 1 : écrire les tests.** Dans la suite, sur les deux paires :
  - `une_reprise_rend_le_rapport_d_une_traite` (point de revue 4) : huit fichiers, dont deux du même contenu, un sans
    nom, un WAV coupé à 30 octets, avant l'en-tête de `data` (illisible ; V8), et un texte en Latin-1 ; les trois
    premiers versés, dont ce WAV et le texte, puis
    `Paire::rouvrir` (les services perdent leur état) ; l'inspection du même dossier dit `reprise: Some(<l'import>)` ;
    l'ouverture rend `repris: true` et les seuls rangs restants ; la clôture rend les comptes d'un import fait d'une
    traite du même dossier, sur une paire neuve, illisibles et octets compris (R7) ;
  - `un_import_coupe_apres_une_prise_en_stems_se_reprend` (R2) : un jeu `Jeu A/` déclaré `stems` et un dossier
    `Atelier/` déclaré `generee:Atelier inventé` ; la prise en stems versée, puis `Paire::rouvrir` ; l'inspection du
    même dossier dit la reprise, et la clôture rend les comptes d'une traite ;
  - `un_import_ouvert_sans_nom_ne_se_reprend_pas` (R2) : un import ouvert, rien versé, puis `Paire::rouvrir` ;
    l'inspection du même dossier ne dit aucune reprise, et `ouvert_ailleurs` ;
  - `une_reprise_garde_les_declarations_de_l_ouverture` (R2) : un import ouvert, `Démos/` déclaré `main`, un fichier
    versé, puis `Paire::rouvrir` ; l'inspection rend `declarations_reprises` (`Démos/`, `main`) ; une ouverture qui
    déclare `Démos/` `stems` est refusée, « Cet import reprend ses déclarations : elles ne changent pas. » ; sans
    déclaration ou avec les mêmes, elle passe ;
  - `un_fichier_change_depuis_l_inspection_arrete_l_import` : `Paire::changer` du rang 2 après l'inspection, à une
    autre taille ; son versement est refusé, « « b.wav » a changé depuis l'inspection : l'import s'arrête, et
    rechoisir le dossier le reprend. » ; une nouvelle inspection le reprend, et le fichier changé se verse ;
  - `un_contenu_change_a_taille_egale_arrete_l_import` (question 14 ; R8) : `Paire::changer` du rang 2 après
    l'inspection, à la même taille, sa marque de dernière écriture avancée ; son versement est refusé du même message, aucune captation ne
    porte l'empreinte du contenu changé, aucun nom provisoire ne reste ; une nouvelle inspection le reprend ;
  - `un_import_ouvert_ailleurs_se_clot_a_l_ouverture_du_suivant` (question 35) : un import ouvert sur `A`, un fichier
    versé ; l'inspection de `B` dit `ouvert_ailleurs` ; l'ouverture de `B` clôt d'abord l'import de `A` : le journal
    porte « Import du 14 novembre clos » avant « Import du 14 novembre ouvert », le rapport de `A` se relit (vus 1,
    versés 1, dédoublonnés et ignorés 0), et l'état du fonds ne dit plus d'import resté ouvert une fois `B` clos ;
  - `en_lecture_seule_l_import_est_refuse_avant_toute_captation` : un fonds en lecture seule (la migration qui attend
    ses dépendances de la suite) ; l'inspection est refusée du refus de `lecture_seule()` ; aucune captation n'est
    versée ;
  - `l_etat_du_fonds_dit_qu_un_import_est_reste_ouvert` : un import ouvert, puis `Paire::rouvrir` ; les avertissements
    de l'état du fonds portent « Un import est resté ouvert : rechoisir son dossier le reprend. » ; clos, plus ;
  - `rechoisir_un_autre_dossier_ne_reprend_pas_l_import_ouvert` : un import ouvert sur `A`, deux noms écrits ; un
    dossier `B` sans ces noms ; l'inspection ne dit aucune reprise, dit `ouvert_ailleurs`, et l'import de `A` reste
    ouvert tant que `B` n'est pas ouvert (question 35).
  Dans `crates/stockage/tests/commandes_sur_fichiers.rs` seul, sous `pannes` :
  - `une_reprise_apres_une_coupure_entre_deux_troncons_ne_laisse_aucune_copie` : la copie du rang 4 (2 621 440
    octets) coupée à `Troncon` rang 2, une coupure du processus : le retrait coupé aussi (R1) ; le nom provisoire
    reste ; rouvert, puis repris, il est retiré à l'ouverture de la reprise ; à la clôture, aucun nom provisoire de
    l'import ne reste, et le rapport égale celui d'un import fait d'une traite du même dossier ;
  - `la_cloture_retire_chaque_copie_de_l_import` : un nom provisoire `import-<I>-9.1.partiel` posé avant la clôture
    disparaît ; un nom provisoire de capture, `<e>.1.partiel`, reste.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la reprise d'un import, un fichier changé, les copies`), pousser.
  Attendu à « Tests » de `verifier` : `une_reprise_rend_le_rapport_d_une_traite` `panicked at` (l'inspection ne dit pas
  la reprise).
- [ ] **Étape 3 : écrire le code.** L'avertissement de l'import resté ouvert s'ajoute dans `etat_du_fonds`, d'après
  `lecture::imports` ; la reprise, la clôture d'un import ouvert ailleurs, la marque de dernière écriture et le retrait
  des copies, comme
  les docs de la tâche 12 le disent.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : la suite verte sur les deux paires,
  `commandes_sur_fichiers.rs` vert ; dans `windows`, de même.
- [ ] **Étape 5 : committer** `git add crates/services/src/lib.rs crates/services/src/etat_de_l_import.rs
  crates/services/tests/suite/mod.rs crates/services/tests/commandes_en_memoire.rs
  crates/stockage/tests/commandes_sur_fichiers.rs` (`feat: la reprise d'un import, le fichier changé, le retrait des
  copies`).

### Tâche 15 : l'établi : le dossier choisi, la place libre, les commandes servies ; I30 et I31

**Fichiers :** modifier `app/src-tauri/src/systeme.rs`, `app/src-tauri/src/lib.rs`,
`app/src-tauri/src/commandes_de_l_etabli.rs`, `app/src-tauri/capabilities/windows.json`,
`app/src-tauri/tests/statique.rs` ; créer, dans `crates/stockage/tests/source.rs`, les tests de Windows.

**Interfaces — produit** (`etabli`) :

```rust
// systeme.rs
/// La place offerte à l'appelant sur le volume de `dossier` (question 8 ; P10) : sous Windows,
/// `GetDiskFreeSpaceExW`, déclaré dans un bloc `unsafe extern "system"`, le seul du dépôt, sur le
/// chemin canonique de `dossier` (`fs::canonicalize`, qui rend un `\\?\`), en UTF-16 fini par un
/// zéro, `lpFreeBytesAvailableToCaller` seul lu ; un échec rend l'erreur du système. Hors de
/// Windows : `io::ErrorKind::Unsupported`.
pub fn place_libre(dossier: &Path) -> io::Result<u64>;
/// Le dossier de l'application, que `run()` garde dans l'état de Tauri.
pub struct DossierDeLApplication(pub PathBuf);
/// Admet le dossier choisi (question 14) : refusé s'il est le dossier de l'application, le contient
/// ou s'y trouve, leurs chemins canoniques comparés par segments (`Path::starts_with`), « Ce
/// dossier est celui de Pupitre, le contient ou s'y trouve : il ne se verse pas. » ; rend son chemin.
pub fn admettre_le_dossier(choisi: &Path, application: &Path) -> Result<PathBuf, String>;

// lib.rs
/// Le dossier choisi, parcouru, que les commandes de l'import se partagent ; oublié à la clôture.
pub(crate) struct DossierChoisi(pub Mutex<Option<Arc<SourceSurFichiers>>>);

// commandes_de_l_etabli.rs : chaque commande comme celles de la tranche 2, par `Ouverture::demander`
pub async fn inspecter_un_dossier<R: tauri::Runtime>(app: tauri::AppHandle<R>, ouverture: tauri::State<'_, Ouverture>,
    choisi: tauri::State<'_, DossierChoisi>, application: tauri::State<'_, DossierDeLApplication>) -> Reponse<Option<Inspection>>;
pub async fn ouvrir_l_import(…, commande: OuvrirLImport) -> Reponse<ImportOuvert>;
pub async fn verser_un_fichier(…, commande: VerserUnFichier) -> Reponse<FichierVerse>;
pub async fn verser_une_prise_en_stems(…, commande: VerserUnePriseEnStems) -> Reponse<Faite>;
pub async fn clore_l_import(…, commande: CloreLImport) -> Reponse<ImportClos>;
pub async fn declarer_une_origine(ouverture: tauri::State<'_, Ouverture>, commande: DeclarerUneOrigine) -> Reponse<Faite>;
pub async fn choisir_le_type_d_un_stem(ouverture: tauri::State<'_, Ouverture>, commande: ChoisirLeType) -> Reponse<Faite>;
pub async fn lire_un_texte_verse(ouverture: tauri::State<'_, Ouverture>, commande: Captation) -> Reponse<TexteLu>;
pub async fn lire_un_rapport(ouverture: tauri::State<'_, Ouverture>, commande: LireUnRapport) -> Reponse<RapportDImport>;
pub async fn generateurs(ouverture: tauri::State<'_, Ouverture>) -> Reponse<Vec<String>>;
```

  `inspecter_un_dossier` : dans `tauri::async_runtime::spawn_blocking`, la boîte de dossier (`app.dialog().file()
  .set_title("Importer un dossier")`, `set_parent` de la fenêtre `main`, `blocking_pick_folder()`), fermée sans choix :
  `Ok(None)` ; puis `admettre_le_dossier`, puis `SourceSurFichiers::parcourir`, hors du fil du fonds ; la source gardée
  dans `DossierChoisi` ; la place libre, `systeme::place_libre(&application.0)`, le volume du dossier de
  l'application, où vit la racine des données, que la commande ne connaît pas (R19) ; puis `inspecter`, au fil.
  `ouvrir_l_import` prend aussi `DossierDeLApplication` et relit la place libre de même avant d'ouvrir (« la place, de
  nouveau comparée »). Les autres commandes de l'import prennent la source gardée ; sans elle, « Aucun dossier n'est
  choisi. », de `Raison::Impossible`. Aucune
  méthode ne se nomme `message` (I29). Le contrôle d'I30 (`tests/statique.rs`) gagne `PLACE_LIBRE: &[&str] =
  &["GetDiskFreeSpaceEx", "statvfs", "available_space", "fs2::", "sysinfo"]` et le mot `unsafe`, admis dans `SYSTEME`
  seul.

- [ ] **Étape 1 : écrire les tests.** Dans `app/src-tauri/src/systeme.rs` (`mod tests`, `windows`) :
  - `la_place_libre_se_lit_sur_un_chemin_long` (`#[cfg(windows)]`) : un dossier jetable, imbriqué jusqu'à plus de 300
    unités UTF-16, rend une place non nulle ;
  - `la_place_libre_d_un_dossier_absent_est_refusee` (`#[cfg(windows)]`) : une erreur ;
  - `un_dossier_dans_celui_de_l_application_est_refuse` : sous un dossier jetable, `app/` pour l'application ;
    `app/`, `app/x` et le dossier jetable lui-même sont refusés, du message ; `autre/` est admis.
  Dans `app/src-tauri/tests/statique.rs`, `i30_chaque_controle_refuse_ce_qu_on_y_plante` gagne deux plantations : un
  `GetDiskFreeSpaceExW` et un `unsafe extern "system"` dans un fichier autre que `systeme.rs`, chacun refusé ; dans
  `systeme.rs`, admis.
  Dans `crates/stockage/tests/source.rs` (`verifier` et `windows`) :
  - `un_chemin_de_300_unites_utf16_se_parcourt_et_garde_son_nom` : des dossiers imbriqués, le chemin relatif du
    fichier fait plus de 300 unités UTF-16 ; parcouru, son chemin rendu égale celui du test, et sa lecture en flux rend
    ses octets ;
  - `les_formes_composee_et_decomposee_d_un_nom_restent_deux_noms` : `\u{e9}t\u{e9}.txt` et `e\u{301}te\u{301}.txt` ;
    deux entrées, deux chemins distincts, chacun tel qu'écrit ;
  - `une_jonction_compte_parmi_les_ignores_sans_etre_suivie` (`#[cfg(windows)]`) : `cmd /C mklink /J lien <cible>` vers
    un dossier qui porte un fichier ; une entrée `SorteDEntree::Lien`, et le fichier de la cible n'est vu qu'une fois,
    par son propre chemin ;
  - `un_nom_sans_forme_utf8_se_parcourt_sans_chemin` : un fichier au nom sans forme UTF-8 ; `chemin: None`, son
    extension lisible ;
  - `un_fichier_ouvert_sans_partage_ne_se_lit_pas` (`#[cfg(windows)]` ; question 38) : un fichier que le test tient
    ouvert par `OpenOptionsExt::share_mode(0)` ; `lire` rend `Raison::Disque` ; le fichier refermé, il se lit.
  Les tests d'I31 (`app/src-tauri/tests/i31.rs`) ne changent pas : ils comparent les droits et `generate_handler!` à
  `commandes::NOMS`, que la tâche 11 a étendu.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la place libre, le dossier admis, I30 et les noms de Windows`),
  pousser. Attendu à « Lint de l'établi » de `windows` : `cannot find function place_libre in module systeme`.
- [ ] **Étape 3 : écrire le code.** `GetDiskFreeSpaceExW` déclaré à la main (`#[link(name = "kernel32")] unsafe extern
  "system"`, édition 2024), sans dépendance nouvelle ; le bloc `unsafe` commenté de ce qu'il suppose (le chemin fini
  par un zéro, trois sorties) ; `windows.json` gagne les dix `allow-<nom>`, `generate_handler!` les dix commandes ;
  `run()` gère `DossierDeLApplication` et `DossierChoisi`.
- [ ] **Étape 4 : voir le vert.** Attendu dans `windows` : « Lint de l'établi » vert, les tests de l'établi verts
  (`statique.rs`, `i31.rs`, les tests de `systeme`), la sentinelle d'I30 verte, `source.rs` du stockage vert ; dans
  `verifier`, `clippy -p etabli` vert hors de Windows, `source.rs` vert.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/systeme.rs app/src-tauri/src/lib.rs
  app/src-tauri/src/commandes_de_l_etabli.rs app/src-tauri/capabilities/windows.json app/src-tauri/tests/statique.rs
  crates/stockage/tests/source.rs` (`feat: l'établi choisit le dossier, lit la place libre et sert l'import`).

### Tâche 16 : le test 13 entier

**Fichiers :** modifier `crates/services/src/mesure.rs`, `crates/services/src/en_memoire.rs`,
`crates/services/tests/commandes_en_memoire.rs`, `crates/stockage/tests/mesure.rs`, `app/src-tauri/tests/statique.rs`
(`ACTEUR`), `.github/workflows/ci.yml` (l'étape et son commentaire).

**Interfaces — produit :**

```rust
// services::mesure (hors d'`en-memoire`)
/// L'aléa du générateur, déterministe (splitmix64, d'une graine) : `Splitmix64` (`mesure.rs:253`),
/// renommé et rendu public, avec son `impl Alea`, pour que l'adaptateur en mémoire en tire ses
/// acteurs ; aucun second générateur (R18).
pub struct AleaDuGenerateur { /* l'état, `etat` */ }
impl AleaDuGenerateur { pub fn de(graine: u64) -> Self; }
/// `GRAINE` (`mesure.rs:44`), rendue publique.
pub const GRAINE: u64 = 13;
/// Le test 13 entier (question 28), par les opérations de l'import : vingt imports, chacun d'une
/// source en mémoire de 300 idées à faire naître (100 groupes de trois prises), dont, aux groupes
/// 1 à 100, une prise en stems de douze WAV et de douze MIDI au lieu de la première prise ; chaque
/// WAV d'un en-tête de 44 octets et de 16 octets de données, tirés de l'aléa, chaque MIDI de son
/// en-tête de 14 octets ; chaque chemin de 80 octets en moyenne ; les imports 1 à 10 déclarent leur
/// dossier `generee:<nom>`, l'un de trois noms inventés, les imports 11 à 20 `main` ; après eux,
/// cinquante captations générées déclarées sous un second nom ; puis les gestes de la tranche 2 :
/// 4 000 regroupements, trois textes et trois étiquettes par idée visible, 1 000 liens.
pub fn engendrer_le_fonds_de_mesure<D: Documents, C: Captations>(services: &mut Services<D, C>, heure: i64) -> Result<Engendre, Refus>;

// services::en_memoire (sous `en-memoire`)
impl DocumentsEnMemoire {
    /// Change d'acteur après chaque `operations` opérations appliquées, l'acteur fait de seize
    /// octets tirés de `alea` (`set_actor`, `ActorId::from`) (question 29).
    pub fn changer_d_acteur_tous_les(&mut self, operations: usize, alea: Box<dyn Alea>);
}
```

  Le compte : 20 ouvertures, 20 clôtures, 5 900 versements de prises, 100 prises en stems, 50 déclarations d'origine,
  4 000 regroupements, 6 000 textes, 6 000 étiquettes et 1 000 liens, soit 23 090 opérations ; les acteurs, un tous les
  sept, quelque 3 298, au moins les 3 000 du § 31 · 10 (« un acteur par ouverture, 3 000 en tout » ; R10). Un import du fonds visé fait 6 002 opérations (6 000 versements, l'ouverture et la clôture) :
  une compaction toutes les 101 opérations (`AJOUTS_AU_PLUS` = 100, au sens strict), soit 59 compactions ; le test
  imprime la durée d'une compaction de 101 fichiers d'ajout, et ce temps multiplié par 59 (question 30).

- [ ] **Étape 1 : écrire les tests.** `crates/stockage/tests/mesure.rs` :
  `le_fonds_de_mesure_pese_moins_de_4_500_000_octets` attend `Engendre { idees_visibles: 2_000, idees_retirees: 4_000,
  operations: 23_090 }`, engendre sur `DocumentsEnMemoire` aux acteurs tournants (`changer_d_acteur_tous_les(7,
  Box::new(AleaDuGenerateur::de(GRAINE + 1)))`), compte les acteurs du document (au moins 3 000), rouvre sur fichiers
  comme avant, puis applique 101 notes au fonds sur fichiers et mesure celle qui compacte ; il imprime « Le fonds de
  mesure : <poids> octets ; <acteurs> acteurs ; une compaction de 101 fichiers d'ajout : <durée> ; 59 compactions pour
  un import du fonds visé : <59 × durée> » ; il échoue s'il pèse 4 500 000 octets ou plus (`POIDS_MAX` inchangé, la
  comparaison stricte), ou si la compaction passe 2 s. `crates/services/tests/commandes_en_memoire.rs` : le test du
  générateur réduit (`engendrer_le_fonds_de_mesure_selon`) suit les nombres neufs. Le contrôle d'I30 gagne `set_actor`
  et `ActorId::from` dans `ACTEUR`, et une plantation dans `i30_chaque_controle_refuse_ce_qu_on_y_plante`.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: le test 13 entier, ses acteurs et sa compaction mesurée`),
  pousser. Attendu à l'étape « Le fonds de mesure » de `verifier` : `panicked at`, `operations: 23000` contre `23090`.
- [ ] **Étape 3 : écrire le code.** Le générateur conduit chaque import par les méthodes de la tâche 12, sur une
  `SourceEnMemoire`, la place libre à `u64::MAX` ; sa doc récrit le compte, et ne dit plus « Il y manque encore » ; la
  date, l'appareil et le diapason mesuré restent absents, et la doc le dit. L'étape de `ci.yml` se nomme « Le fonds de
  mesure (test 13) », son commentaire récrit d'après ce compte.
- [ ] **Étape 4 : voir le vert.** Attendu à l'étape « Le fonds de mesure » de `verifier` : `test result: ok.`, la ligne
  imprimée relevée (poids, acteurs, durée) ; dans `windows`, `statique.rs` de l'établi vert. Si le fonds passe 4,5 Mo ou
  la compaction 2 s, la tâche s'arrête : le schéma des tranches 2 et 5 se reprend avant la tranche 6, et la maîtrise
  d'ouvrage est consultée (ADR-024).
- [ ] **Étape 5 : committer** `git add crates/services/src/mesure.rs crates/services/src/en_memoire.rs
  crates/services/tests/commandes_en_memoire.rs crates/stockage/tests/mesure.rs app/src-tauri/tests/statique.rs
  .github/workflows/ci.yml` (`feat: le test 13 entier, par l'import, ses acteurs et sa compaction`).

### Tâche 17 : les documents de la PR (b), les délais, et la revue de la PR (b)

**Fichiers :** `docs/20-sfd/21-capter-et-organiser.md` (CAP-02, CAP-03, CAP-07), `docs/20-sfd/29-socle-donnees-appareils.md`
(SYN-07), `docs/30-modele-de-domaine.md` (§ 2), `docs/31-persistance-et-synchronisation.md` (§ 2.1, § 2.4, § 6.1,
§ 10), `docs/40-architecture.md` (§ 3), `docs/41-pile-technique.md` (§ 6), `docs/70-qualite-et-verification.md` (§ 3,
§ 4.1, § 8), `docs/80-glossaire.md`, `docs/pistes.md`, `scripts/verif-confidentialite.sh`, `CONVENTIONS.md`,
`CLAUDE.md`, `ci.yml` (les délais).

- [ ] **Étape 1 : la recherche de LE-2**, citée dans la description de la PR : `voisin`, `nom provisoire`, `source`,
  `contrat`, `contrats`, `import`, `importer`, `rapport`, `déclarer`, `déclaration`, `reprise`, `lecture seule`,
  chacun aussi sans accent et sur la majuscule accentuée.
- [ ] **Étape 2 : la SFD.** CAP-03 : « Lit · écrit » devient « Écrit des captations, des idées importées et ce que
  Sacha déclare ; ses propositions s'engendrent en INC-2 » (question 32) ; la règle des formats devient « Un fichier se
  reconnaît audio ou MIDI à sa signature, texte à son extension, et est versé tel quel » ; « son rôle nouveau » se
  récrit d'après la question 9 (l. 199-202), sa lettre gardée : la déclaration reproposée suit le seul chemin relatif,
  celle de la racine ou d'un nom courant (« Démos ») d'un import sans rapport avec le dossier choisi comprise, et elle
  ne s'applique jamais seule (R22) ; la portée par préfixe reprend la phrase de l'ADR-024 (question 23) ; le rapport
  compte aussi les fichiers « sans nom » (question 13) ; le cas limite « Fichier illisible » se récrit (question 32,
  (d)) et dit qu'un fichier illisible, sans format, ne s'écoute pas (R17) ; le critère 13 se lit « Étant donné une
  taille à verser plus grande que la place libre » (question 8) ; trois règles de plus : un fonds en lecture seule
  refuse l'import entier, avant toute captation (question 15) ; un fichier qui ne se lit pas arrête l'inspection, nommé
  (question 38) ; un import resté ouvert, que le dossier choisi ne reprend pas, se clôt à l'ouverture du suivant,
  avec ce qu'il a versé, et l'écran le dit : « Un import est resté ouvert, que ce dossier ne reprend pas : verser
  celui-ci le clôt, avec ce qu'il a versé. » (question 35 ; V12). SYN-07, ligne « Lit · écrit » : « Lit ; un import n'écrit aucune décision musicale », et sa règle
  (question 32, (e)). CAP-02 : regrouper « rassemble » (question 33) ; et la règle « Une idée importée sans texte se
  montre « Fichier : » suivi du premier nom vu de sa première captation, écrit comme texte ; la recherche ne lit pas ce
  nom. » (question 4 ; R12). CAP-07 (`:533-535`) : importer, refusé en lecture seule, et la page d'un fonds vide le
  propose.
- [ ] **Étape 3 : le modèle, la persistance, l'architecture.** § 30 · 2 : la table gagne l'ULID d'un import. § 31 · 2.1
  et 2.4 : « Une captation se verse toujours » est la règle d'une capture (CAP-01), non d'un import (question 15).
  § 31 · 6.1 : la copie en flux, son nom provisoire `import-<ULID>-<rang>.<processus>.partiel`, rangé au démarrage s'il
  est entier et nommé, retiré à la reprise s'il ne l'est pas, et à la clôture (question 31) ; la capture garde sa règle.
  § 31 · 10 : le test 4 rejoué sur la copie en flux et sur un stem ; le test 11 gagne le manifeste et le rapport
  témoins ; la ligne du test 13, récrite : « 2 000 idées visibles, 200 octets de texte en moyenne, chaque texte écrit
  trois fois, trois étiquettes et un lien par idée ; 6 000 idées importées, dont 4 000 retirées par regroupement, par
  vingt imports, chacun avec son rapport ; dont 100 prises en stems de douze stems, chacun avec son MIDI, et leur
  manifeste ; chaque fait d'en-tête, chaque nom d'origine de 80 octets en moyenne, chemin compris ; des origines
  déclarées, une moitié générée, dont cinquante sous deux noms ; un acteur toutes les sept opérations, 3 000 au moins ;
  la date,
  l'appareil et le diapason mesuré absents, jusqu'à l'opération qui les écrit ; engendrées à l'exécution | Son fichier
  complet pèse 4 500 000 octets ou plus, ou une compaction de 101 fichiers d'ajout passe 2 s en `--release` ». § 40 · 3 :
  trois contrats, la source à verser comprise. § 41 · 6 : aucune dépendance nouvelle ; les trois usages de la boîte de
  dialogue (le cahier, le dossier à verser, d'avance la destination de l'archive) (C28).
- [ ] **Étape 4 : la qualité, le glossaire, les pistes, la porte `contenu`.** § 70 · 3 : I14 éprouvé sur la copie en
  flux ; I30 : la place libre et le bloc `unsafe`, admis dans `systeme` seul ; I01 sur les clés de l'import. § 70 · 4.1 :
  la ligne « CAP-03 | 1 à 3, 7, 9 à 11, 13 | INC-1, lot 1 », et le 8 dit livré par part (`jeu_propose` de
  l'inspection, sans écran), selon la règle des parts du § 70 · 4.1 (R24). § 70 · 8. Le glossaire : « Voisin » et « Nom provisoire »
  (la copie en flux), « Source à verser ». `docs/pistes.md` : « L'empreinte d'une captation dans un message » sort
  (question 26) ; « Chercher par origine déclarée » se déclenche à la PR de conception d'INC-2 (question 24) ; « Ce
  que laissent deux regroupements croisés » dit le sort de l'idée qu'un même fichier importé sur deux appareils laisse
  (tâche 13). `scripts/verif-confidentialite.sh:47` gagne `aifc|opus|oga|wave|bwf|m4b|caf|wma|smf|kar|rmi|cho|crd|chopro|chord|chordpro`,
  et son commentaire dit pourquoi `pro` n'y est pas (trop commun) ; la porte, lancée sur la tête, passe.
- [ ] **Étape 5 : les portes.** `CONVENTIONS.md` : `contenu` (les extensions), `verifier` (le test 13 entier, la
  compaction), `windows` (les tests de la source et de la place libre) ; `CLAUDE.md` : « Les crates » (le stockage : la
  source sur fichiers, la copie en flux ; les services : l'import, la source en mémoire ; `commandes` : `NOTER, …,
  GENERATEURS, EXPORTER_CAHIER` ; l'établi : la place libre et le dossier admis dans `systeme`, les commandes de
  l'import) et « Commandes » (la ligne du fonds de mesure). Les délais de `verifier` et de `windows`, remesurés sur
  l'exécution de la tête, au double de la mesure, leur commentaire daté.
- [ ] **Étape 6 : committer** les documents (`docs: l'import à la SFD, au schéma, à l'architecture et aux portes`).
- [ ] **Étape 7 : la revue de la PR (b).** La revue de la PR entière, sur sa tête, par le modèle le plus capable : le
  tableau du § 70 · 4.1 contre les critères de la tranche (CAP-03 · 4, 8 et 12 non inscrits), le job de chaque test nommé
  (la suite sur les deux paires dans `verifier`, sur fichiers dans `windows`), la recherche de LE-2, chaque mécanisme
  annoncé que la PR touche relu, les textes de la question 32 recopiés. Puis les corrections, leur relecture sur la tête
  qui part. Le corps dit : les tests réécrits (`le_protocole…` reste jusqu'à la PR (d) ; le générateur ; les messages) ;
  que `windows` a été rouge sur I31 entre les tâches 11 et 15 ; la règle de reprise (les seuls noms de fichiers à
  l'ULID de l'import, une prise en stems par le préfixe de son dossier ; sans nom, aucune reprise ; les déclarations
  de l'ouverture gardées ; R2) ; qu'un import resté ouvert, que le dossier choisi ne reprend pas, se clôt à
  l'ouverture du suivant
  (question 35) ; que l'empreinte d'un fichier n'est calculée à l'inspection qu'à la taille d'une captation, et jamais
  pour proposer un jeu (question 37) ; qu'un contenu changé se voit à sa taille ou à sa marque de dernière écriture,
  et qu'un système de fichiers sans marque de dernière écriture ne compare que la taille (R8) ; qu'un réimport, qui relit tout à
  l'inspection, reste une seule demande longue, le fil tenu, sans avancée à l'écran (R4) ; qu'un fichier qui ne se lit
  pas arrête l'inspection, et qu'un fichier de OneDrive présent seulement en ligne se télécharge en entier à sa
  première lecture, ou échoue hors connexion, limite dite sans test (question 38) ; le poids, les acteurs et la durée
  de compaction relevés ; « Écarts à la spec : Aucun. ». `gh pr ready`, puis la fusion par squash,
  `--match-head-commit` nommant la tête relue.

## PR (c) · la boîte d'idées de l'interface en un module

**Branche :** `t5c-la-boite-en-module`. **Titre :** `refactor: la boîte d'idées de l'interface en un module, sans
changement de comportement` (87). **Documents :** `docs/pistes.md` ; `CLAUDE.md`. **Fin :** la tâche 19.

Rien ne change de ce que fait l'interface : ni texte, ni signal, ni commande, ni règle de style ; chaque test garde son
nom et son corps, à ses `use` près. Une limite, que le corps de la PR dit : la fusion par squash fait de la scission un
seul commit, où aucune part ne ressemble à plus de la moitié de `boite.rs` ; `git log --follow` sur une part s'arrête
donc à cette PR. L'histoire d'avant se lit par `git log -- app/ui/src/boite.rs`, et celle d'une ligne par
`git blame -C -C`, qui suit une ligne d'un fichier à l'autre dans un même commit.

### Tâche 18 : la scission de `boite.rs` en `boite/`

**Fichiers :** supprimer `app/ui/src/boite.rs` ; créer `app/ui/src/boite/mod.rs`, `fenetre.rs`, `textes.rs`,
`gestes.rs`, `requetes.rs`, `etat.rs`, `focus.rs`, `ecran.rs`, `vue.rs`, `volets.rs`.

Les parts, d'après les éléments de `boite.rs` à `fe918aa` (l'élément nommé fait foi, non le numéro) :

| Part | Ce qu'elle reçoit |
|---|---|
| `mod.rs` | la doc du module (les lignes `//!` de `boite.rs`), la déclaration des parts, et les seuls noms lus hors du module, réexportés : `pub(crate) use` de `BoiteDIdees`, `EtatDeLaBoite`, `dans_un_champ_de_texte` (lus par `app.rs:14`) et `CONTENUS` (lu par `recherche.rs:7`) ; les aides de test que deux parts partagent, sous `#[cfg(test)]` |
| `fenetre.rs` | `MARGE`, `LIGNES_DE_L_ETALON`, `Etalon`, `en_u32`, `lisible`, `fenetre`, `couvre`, `fenetre_a_demander`, `Bord`, `ligne_d_entree`, `idee_de_rang`, `mesures` |
| `textes.rs` | `CASE_VIDE`, `ORDRES`, `CONTENUS`, `libelle_d_annulation`, `page_du_fonds_vide`, `texte_ou_tiret`, `mot_de_l_etat`, `mot_de_la_provenance`, `texte_de_la_ligne`, `etiquettes_de_la_ligne`, `etiquette_ecrite`, `etiquette_du_choix`, `etiquettes_a_choisir`, `etat_de_la_ligne`, `provenance_de_la_ligne`, `nom_de_la_ligne`, `date_locale`, `nom_d_une_idee_liee`, `etat_de_l_idee`, `cle_de_la_ligne`, `ligne_du_journal`, `entete_du_journal` |
| `gestes.rs` | `ctrl_z_annule`, `note_a_envoyer`, `commande_de_lien`, `relier_les_cochees`, `deliaison`, `envoi_au_clavier`, `dans_un_champ_de_texte`, `argument` |
| `requetes.rs` | `en_usize`, `au_noyau`, `sans_au_noyau`, `pose_un_critere`, `NOTE_D_UNE_REQUETE_PLUS_RECENTE`, `PRECISION_D_UNE_REQUETE_PLUS_RECENTE`, `constat_des_requetes`, `ce_que_dit_la_requete`, `lancer`, `enregistrer_possible`, `texte_de_la_requete`, `commande_de_la_boite`, `requete_en_cours`, `GesteDeRequete` |
| `etat.rs` | `BoiteLue`, `RetireesLues`, `JournalLu`, `Volet`, `cible_de_fermeture`, `IdeeOuverte`, `Demandes`, `EtatDeLaBoite` et son `impl`, `fonds_vide_dans` |
| `focus.rs` | `Liste`, `Commande`, `Gardee`, `Actif`, `Rendu`, `focus_a_rendre`, `cible_du_rendu`, `cible_apres_le_volet`, `defilement_a_remettre`, `Offerts`, `ce_que_le_volet_offre`, `prendre`, `actif`, `rendre_le_focus_perdu`, `Defilement`, `Cible`, `cible_repliee`, `cible_quand_le_fonds_se_vide`, `focus_de_la_ligne` |
| `ecran.rs` | `Ecran` et son `impl` |
| `vue.rs` | `choix_d_un_critere`, `champ_de_date`, `BoiteDIdees`, `ligne_de_la_boite` |
| `volets.rs` | `ligne_retiree`, `VoletDeLIdee`, `VoletDesRetirees`, `VoletDuJournal` |

  La visibilité : un élément que sa seule part emploie reste privé ; un élément, ou un champ, qu'une autre part lit
  devient `pub(super)` ; rien ne devient plus visible hors de `boite/` qu'avant. Chaque test va au pied de la part qui
  porte la première fonction qu'il éprouve, dans un `#[cfg(test)] mod tests` qui commence par `use super::*;` ; ses
  aides le suivent ; une aide que deux parts emploient (`ligne`, `ligne_de`, `ecran`…) va dans `mod.rs`, en
  `pub(super)`, sous `#[cfg(test)]`.

- [ ] **Étape 1 : relever avant.** Depuis le worktree de la branche, à jour d'`origin/main` :
  `git show origin/main:app/ui/src/boite.rs | grep -A1 '#\[test\]' | grep -o 'fn [a-z0-9_]*' | sort` dans
  `<scratchpad>/t5c-tests-avant.txt`, 47 lignes ; et, sur la dernière exécution verte de `main`, la ligne de
  « Tests » de `verifier` pour l'interface (`cargo test -p interface`) : `test result: ok. 86 passed` (47 de
  `boite.rs`, 9 d'`app.rs`, 9 de `bandeau.rs`, 8 de `vue_table.rs`, 7 de `recherche.rs`, 4 d'`export.rs`, 2 de
  `natif.rs`). Le script `<scratchpad>/t5c-memes-lignes.py` lit le multiensemble des lignes de l'ancien fichier et des
  dix nouveaux, après en avoir retiré les lignes vides, les `use`, les `mod`, les `//!`, `#[cfg(test)]`, `mod tests {`
  et la seule accolade qui le ferme, et les mots `pub(super) ` et `pub(crate) ` : il imprime toute ligne de plus ou de
  moins, et sort en 1 s'il en trouve.
- [ ] **Étape 2 : scinder.** Les parts, dans l'ordre du tableau ; `rustfmt --edition 2024` sur chaque fichier, hors du
  dépôt, comme les contraintes le disent ; le script de l'étape 1 sort en 0 ; la même commande `grep` sur
  `app/ui/src/boite/*.rs` rend un fichier identique à `t5c-tests-avant.txt` (`diff` vide).
- [ ] **Étape 3 : committer, pousser, voir le vert.** `git add app/ui/src/boite.rs app/ui/src/boite/mod.rs
  app/ui/src/boite/fenetre.rs app/ui/src/boite/textes.rs app/ui/src/boite/gestes.rs app/ui/src/boite/requetes.rs
  app/ui/src/boite/etat.rs app/ui/src/boite/focus.rs app/ui/src/boite/ecran.rs app/ui/src/boite/vue.rs
  app/ui/src/boite/volets.rs` (`refactor: la boîte d'idées en un module boite/, une part par fichier`) ; ouvrir la PR
  en brouillon. Attendu à « Tests » de `verifier` : `test result: ok. 86 passed` pour l'interface, `statique.rs` et
  `encre_statique.rs` de l'interface verts (ils lisent `app/ui/src/` en descendant dans ses dossiers) ; `clippy -p
  interface`, sur l'hôte et sur `wasm32-unknown-unknown`, vert ; dans `windows`, la construction de l'interface et de
  l'établi verte. Aucun rouge attendu : la PR ne change aucun comportement, et ses tests existent déjà.

### Tâche 19 : les documents de la PR (c), et la revue de la PR (c)

**Fichiers :** `docs/pistes.md`, `CLAUDE.md`.

- [ ] **Étape 1 : les documents.** `docs/pistes.md` : « La boîte de l'interface en un module » sort de la liste.
  `CLAUDE.md`, « Les crates », `interface` : « la boîte d'idées (`boite/`, une part par fichier : la fenêtre, les
  textes, les gestes du clavier, les requêtes, l'état, le focus, l'écran, la vue et les volets) ». Committer (`docs: la
  boîte de l'interface en module, aux pistes et à CLAUDE.md`).
- [ ] **Étape 2 : la revue de la PR (c).** La revue de la PR entière, sur sa tête : le script de la tâche 18 et la
  liste des 47 tests rejoués sur la tête ; aucun élément plus visible qu'il ne le faut ; aucun texte montré changé
  (`git diff origin/main -M -- app/ui/src` ne montre que des lignes déplacées, à la visibilité près). Puis les
  corrections, leur relecture sur la tête qui part. Le corps dit : la limite de `git log --follow` et ses deux
  détours ; qu'aucun test n'est réécrit ; « Écarts à la spec : Aucun. ». `gh pr ready`, puis la fusion par squash,
  `--match-head-commit` nommant la tête relue.

## PR (d) · le protocole des captations, la lecture simple et l'écran de l'import

**Branche :** `t5d-l-ecoute-et-l-ecran`. **Titre :** `feat: le protocole des captations, la lecture simple et l'écran
de l'import` (75). **Documents :** CAP-03 ; § 40 · 7.5 ; § 50 · 2, 3.1 et 4.2, et la maquette du fonds ; § 70 · 3,
4.1, 7 et 8 ; le glossaire ; `docs/pistes.md` ; `docs/registre.md` (LE-17) ; `.cargo/mutants.toml` ;
`CONVENTIONS.md` ; `CLAUDE.md`. **Fin :** la tâche 26.

### Tâche 20 : les règles du protocole, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/protocole.rs`, `crates/noyau/tests/protocole.rs` ; modifier
`crates/noyau/src/donnees/mod.rs`, `.cargo/mutants.toml`.

**Interfaces — produit** (`noyau::donnees::protocole`) :

```rust
/// Les octets d'une réponse du protocole, au plus (question 27).
pub const REPONSE_AU_PLUS: u64 = 1_048_576;

/// Le type d'un format audio versé, que le protocole envoie et que la lecture simple demande à
/// l'appareil, la table fermée de la question 27, à six lignes : `wav` `audio/wav`, `flac`
/// `audio/flac`, `mp3` `audio/mpeg`, `m4a` `audio/mp4`, `ogg` `audio/ogg`, `aiff` `audio/aiff` ;
/// `None` pour tout autre format, `caf` et `wma` compris, qui ne sont pas servis (R3), un MIDI et
/// un texte aussi.
pub fn type_du_format(format: &str) -> Option<&'static str>;

/// Une plage d'octets, ses deux bornes comprises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plage { pub debut: u64, pub fin: u64 }
impl Plage { pub fn longueur(&self) -> u64; }

/// Ce que demande l'en-tête `Range` d'une requête.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demande { Entier, Plage(Plage), Impossible }

/// Lit `range` pour un fichier de `taille` octets. Absent, ou d'une autre unité que `bytes`
/// (comparée sans égard à la casse) : `Entier`. `bytes=a-b` et `bytes=a-` : de `a` à `b`, ou à la
/// fin, bornés à `taille − 1` et à `a + REPONSE_AU_PLUS − 1` ; `bytes=-n` : les `min(n,
/// REPONSE_AU_PLUS, taille)` derniers octets. `Impossible` : `a` au-delà de la fin, `b` avant `a`,
/// `-0`, un fichier vide, plusieurs plages, ou toute autre forme (des chiffres seuls, sans signe ni
/// espace, autour d'un seul `-`).
pub fn plage(range: Option<&str>, taille: u64) -> Demande;

/// Une réponse du protocole : son statut, ses en-têtes, et la plage du fichier à lire, `None` sans
/// corps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Servie { pub statut: u16, pub entetes: Vec<(&'static str, String)>, pub plage: Option<Plage> }

/// La réponse à `methode` sur un fichier de `taille` octets, de type `type_mime`. `GET` ou `HEAD`
/// seuls, sinon 405 et `Allow: GET, HEAD`. `Entier` : 200 et le fichier s'il tient dans
/// `REPONSE_AU_PLUS`, 206 et ses `REPONSE_AU_PLUS` premiers octets sinon ; `Plage` : 206 ;
/// `Impossible` : 416 et `Content-Range: bytes */<taille>`. Chaque réponse porte
/// `Accept-Ranges: bytes` et `X-Content-Type-Options: nosniff` ; 200 et 206, `Content-Type` et
/// `Content-Length` ; 206, `Content-Range: bytes <début>-<fin>/<taille>`. Un fichier vide, sans
/// `Range` : 200, sans plage. `HEAD` : les en-têtes de `GET`, sans plage.
pub fn servir(methode: &str, range: Option<&str>, taille: u64, type_mime: &'static str) -> Servie;

/// 404, avec les deux en-têtes communs : une empreinte inconnue ou mal formée, ou un contenu qui
/// n'est l'audio ni d'une prise ni d'un stem, d'un format qui a un type.
pub fn introuvable() -> Servie;
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/protocole.rs`), une aide `entete(&Servie, nom) ->
  Option<&str>` :

```rust
const TROIS_MILLIONS: u64 = 3_000_000;

#[test]
fn chaque_format_a_son_type() {
    for (format, attendu) in [("wav", "audio/wav"), ("flac", "audio/flac"), ("mp3", "audio/mpeg"), ("m4a", "audio/mp4"),
                              ("ogg", "audio/ogg"), ("aiff", "audio/aiff")] {
        assert_eq!(type_du_format(format), Some(attendu), "{format}");
    }
    for format in ["caf", "wma", "smf1", "chordpro", "brut", "", "WAV"] { assert_eq!(type_du_format(format), None, "{format}"); }
}

#[test]
fn une_requete_sans_plage_qui_tient_rend_200() {
    let s = servir("GET", None, 1_000, "audio/wav");
    assert_eq!((s.statut, s.plage), (200, Some(Plage { debut: 0, fin: 999 })));
    assert_eq!((entete(&s, "Content-Length"), entete(&s, "Content-Type")), (Some("1000"), Some("audio/wav")));
    assert_eq!(entete(&s, "Content-Range"), None);
}

#[test]
fn une_requete_sans_plage_au_dela_de_la_borne_rend_206() {
    let s = servir("GET", None, TROIS_MILLIONS, "audio/wav");
    assert_eq!((s.statut, s.plage), (206, Some(Plage { debut: 0, fin: 1_048_575 })));
    assert_eq!(entete(&s, "Content-Range"), Some("bytes 0-1048575/3000000"));
    assert_eq!(entete(&s, "Content-Length"), Some("1048576"));
}

#[test]
fn les_plages_se_bornent() {
    let p = |range: &str, taille| plage(Some(range), taille);
    let pl = |debut, fin| Demande::Plage(Plage { debut, fin });
    assert_eq!(p("bytes=0-", TROIS_MILLIONS), pl(0, 1_048_575));
    assert_eq!(p("bytes=100-199", TROIS_MILLIONS), pl(100, 199));
    assert_eq!(p("bytes=2999990-", TROIS_MILLIONS), pl(2_999_990, 2_999_999));
    assert_eq!(p("bytes=2999990-4000000", TROIS_MILLIONS), pl(2_999_990, 2_999_999));
    assert_eq!(p("bytes=-500", TROIS_MILLIONS), pl(2_999_500, 2_999_999));
    assert_eq!(p("bytes=-5000000", TROIS_MILLIONS), pl(1_951_424, 2_999_999));
    assert_eq!(p("bytes=-5000", 1_000), pl(0, 999));
    assert_eq!(p("Bytes=0-9", 1_000), pl(0, 9));
    let s = servir("GET", Some("bytes=100-199"), TROIS_MILLIONS, "audio/wav");
    assert_eq!((s.statut, entete(&s, "Content-Range"), entete(&s, "Content-Length")), (206, Some("bytes 100-199/3000000"), Some("100")));
}

#[test]
fn une_plage_impossible_rend_416() {
    for range in ["bytes=3000000-", "bytes=-0", "bytes=5-4", "bytes=0-1,5-6", "bytes=abc", "bytes=", "bytes= 1-2", "bytes=+1-2", "bytes=1--2"] {
        assert_eq!(plage(Some(range), TROIS_MILLIONS), Demande::Impossible, "{range}");
    }
    assert_eq!(plage(Some("bytes=0-"), 0), Demande::Impossible);
    let s = servir("GET", Some("bytes=3000000-"), TROIS_MILLIONS, "audio/wav");
    assert_eq!((s.statut, s.plage, entete(&s, "Content-Range")), (416, None, Some("bytes */3000000")));
}

#[test]
fn une_autre_unite_vaut_une_requete_sans_plage() {
    assert_eq!(plage(Some("items=0-5"), 1_000), Demande::Entier);
    assert_eq!(plage(None, 1_000), Demande::Entier);
    assert_eq!(servir("GET", None, 0, "audio/wav").statut, 200);
}

#[test]
fn head_rend_les_en_tetes_de_get_sans_corps() {
    let (get, head) = (servir("GET", None, TROIS_MILLIONS, "audio/wav"), servir("HEAD", None, TROIS_MILLIONS, "audio/wav"));
    assert_eq!((head.statut, &head.entetes, head.plage), (get.statut, &get.entetes, None));
}

#[test]
fn une_autre_methode_rend_405() {
    for methode in ["POST", "PUT", "DELETE", "get", "OPTIONS"] {
        let s = servir(methode, None, 1_000, "audio/wav");
        assert_eq!((s.statut, entete(&s, "Allow"), s.plage), (405, Some("GET, HEAD"), None), "{methode}");
    }
}

#[test]
fn chaque_reponse_porte_nosniff_et_accept_ranges() {
    for s in [servir("GET", None, 1_000, "audio/wav"), servir("GET", Some("bytes=-0"), 1_000, "audio/wav"),
              servir("POST", None, 1_000, "audio/wav"), introuvable()] {
        assert_eq!((entete(&s, "X-Content-Type-Options"), entete(&s, "Accept-Ranges")), (Some("nosniff"), Some("bytes")), "{}", s.statut);
    }
    assert_eq!((introuvable().statut, introuvable().plage), (404, None));
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: les règles du protocole des captations, au noyau`), pousser,
  ouvrir la PR en brouillon. Attendu à « Lint » : `unresolved import noyau::donnees::protocole`.
- [ ] **Étape 3 : écrire le code.** La lecture de l'en-tête par `split_once('=')`, `split_once('-')` et
  `bytes().all(|o| o.is_ascii_digit())`, sans boucle écrite à la main ; les produits et les sommes en `checked_*`, un
  débordement valant `Impossible`.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `protocole.rs` `ok. 9 passed` ; `mutants` sans
  survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/protocole.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/protocole.rs .cargo/mutants.toml` (`feat: les règles du protocole des captations, au noyau`).

### Tâche 21 : le protocole servi par l'établi, en asynchrone

**Fichiers :** créer `app/src-tauri/src/protocole.rs` ; modifier `app/src-tauri/src/lib.rs`,
`app/src-tauri/tests/i31.rs`, `crates/services/src/lib.rs`, `crates/services/src/lecture.rs`,
`crates/services/tests/suite/mod.rs`, `crates/services/tests/commandes_en_memoire.rs`,
`crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — produit :**

```rust
// services::Services
/// Ce que le protocole sert de la captation `empreinte` : le type de son format
/// (`protocole::type_du_format`) et sa taille, si c'est l'audio d'une prise (son format sous
/// `prises`) ou d'un stem (son format sous l'entrée du stem) ; `None` pour une empreinte inconnue ou
/// mal formée, un MIDI, un texte, un manifeste, un rapport, un illisible. Les stems se lisent dans
/// une table (empreinte → format), rebâtie à la première demande qui suit un changement des têtes,
/// comme l'index de la tranche 4. Une lecture : servie aussi en lecture seule.
pub fn captation_a_servir(&mut self, empreinte: &str) -> Result<Option<(&'static str, u64)>, Refus>;
/// La plage `plage` de la captation `empreinte`, par `Captations::relire_une_plage`.
pub fn plage_d_une_captation(&self, empreinte: &str, plage: Plage) -> Result<Vec<u8>, Refus>;

// etabli::protocole
/// Sert une requête du protocole `captation` (§ 40 · 7.5 ; question 27) : le chemin, `/` puis 64
/// chiffres hexadécimaux en minuscules, nomme l'empreinte, sinon 404 ; au fil, par
/// `Ouverture::demander`, `captation_a_servir`, puis `protocole::servir`, puis, s'il y a une plage,
/// `plage_d_une_captation`. Un refus du fil ou du stockage : 500, sans corps, les deux en-têtes
/// communs ; jamais un message.
pub async fn servir_une_captation(ouverture: &Ouverture, requete: &http::Request<Vec<u8>>) -> http::Response<Vec<u8>>;
```

  `run()` enregistre le protocole par `register_asynchronous_uri_scheme_protocol("captation", |contexte, requete,
  repondeur| …)` : le gestionnaire prend `contexte.app_handle()`, lance `servir_une_captation` par
  `tauri::async_runtime::spawn`, l'`Ouverture` lue dans l'état de l'application, et répond une fois par
  `repondeur.respond`. La CSP ne change pas (`media-src … http://captation.localhost`).

- [ ] **Étape 1 : écrire les tests.** Dans la suite, sur les deux paires :
  - `le_protocole_ne_sert_que_l_audio_d_une_prise_ou_d_un_stem` : après un import d'un WAV de 8 044 octets, d'un jeu de
    deux WAV et d'un MIDI de stem, d'un MIDI versé seul, d'un texte et d'un WAV illisible : la prise rend
    `Some(("audio/wav", 8_044))`, chaque stem le sien ; le MIDI de stem, le MIDI versé seul, le texte, le manifeste, le
    rapport, l'illisible, une prise `caf` (une boîte `caff` de la table de la tâche 1 ; question 27 ; R3), une empreinte
    inconnue et `"../x"` rendent `None`, que le protocole sert en 404 ;
  - `la_plage_d_une_captation_rend_ses_octets` : la plage (40, 47) de la prise rend ses octets 40 à 47.
  Dans `app/src-tauri/tests/i31.rs` :
  - `i31_le_protocole_s_enregistre_en_asynchrone` : `controler_le_protocole(source)` passe sur `src/lib.rs`, qui porte
    `register_asynchronous_uri_scheme_protocol("captation"` et aucun `register_uri_scheme_protocol(` ; deux plantations,
    l'enregistrement synchrone et l'absence d'enregistrement, sont refusées.
  Dans `app/src-tauri/src/protocole.rs` (`mod tests`, dans `windows`), sur un fil ouvert sur un dossier jetable, comme
  les tests de `fil.rs` :
  - `une_requete_du_protocole_passe_entre_deux_unites` (point de revue 5) : un dossier de trois WAV, inspecté et
    ouvert ; le premier versé par `demander` ; une requête `GET` du premier, `Range: bytes=0-15`, rend 206 et ses seize
    premiers octets ; les deux autres versés ; la clôture compte trois versés ;
  - `une_requete_sur_un_chemin_mal_forme_rend_404` : `/../x`, l'empreinte en majuscules, 63 chiffres, `/<empreinte>/x` ;
  - `la_lecture_d_une_plage_ne_garde_aucun_fichier_ouvert` (vigilance 5) : un témoin d'abord, un fichier tenu ouvert
    sous la racine des données, qui fait échouer le renommage de la racine ; puis, le témoin refermé, une plage servie,
    et la racine se renomme, puis reprend son nom. Si le témoin ne fait pas échouer le renommage sur le runner, la
    prémisse ne tient pas : la tâche le dit dans son commit, et le test se récrit sur ce que le runner permet.
  `le_protocole_des_captations_refuse_toute_requete` (`app/src-tauri/src/lib.rs:197`) se retire, remplacé par ces
  tests ; la PR le dit.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: le protocole servi, en asynchrone, entre deux unités de
  l'import`), pousser. Attendu à « Lint » de `verifier` : `no method named captation_a_servir found` ; à « Lint de
  l'établi » de `windows` : `cannot find function servir_une_captation`.
- [ ] **Étape 3 : écrire le code.** La table des stems dans les services, comme l'index (§ 31 · 7) ; le module
  `protocole` de l'établi n'emploie aucun mot d'I29 ni d'I30 ; aucun chemin deviné : la racine reste au fil.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : la suite verte sur les deux paires ; dans
  `windows`, `i31.rs`, les tests de `protocole` et la sentinelle d'I30 verts, `commandes_sur_fichiers.rs` vert.
- [ ] **Étape 5 : committer** `git add app/src-tauri/src/protocole.rs app/src-tauri/src/lib.rs
  app/src-tauri/tests/i31.rs crates/services/src/lib.rs crates/services/src/lecture.rs
  crates/services/tests/suite/mod.rs crates/services/tests/commandes_en_memoire.rs
  crates/stockage/tests/commandes_sur_fichiers.rs` (`feat: le protocole des captations, servi en asynchrone`).

### Tâche 22 : l'écran de l'import

**Fichiers :** créer `app/ui/src/import.rs` ; modifier `app/ui/src/main.rs`, `app/ui/src/boite/etat.rs`
(`Volet::Import`), `app/ui/src/boite/ecran.rs`, `app/ui/src/boite/vue.rs`, `app/ui/src/boite/volets.rs`,
`app/ui/src/boite/textes.rs`, `app/ui/src/boite/focus.rs` (`Cible::BoutonDeLImport`), `app/ui/style.css`.

**Interfaces — consomme :** les commandes de la tâche 11 ; `natif::invoquer`, `natif::decalage_du_geste`.

**Interfaces — produit** (`interface::import`) :

```rust
/// L'unité suivante d'un import (question 1) : chaque fichier hors des jeux, dans l'ordre des rangs ;
/// puis, pour chaque jeu, ses fichiers, puis sa prise en stems ; puis la clôture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unite { Fichier(u64), PriseEnStems(String), Clore }

/// La conduite d'un import ouvert : ce qui reste, ce qui est fait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Conduite { /* les rangs, les jeux, les fichiers faits, l'arrêt */ }
impl Conduite {
    pub(crate) fn de(ouvert: &ImportOuvert) -> Self;
    /// L'unité à demander ; `None` après la clôture, après un refus, ou l'import arrêté.
    pub(crate) fn suivante(&self) -> Option<Unite>;
    /// Note la réponse à `unite` : faite, elle passe à la suivante ; refusée, la conduite s'arrête.
    pub(crate) fn noter(&mut self, unite: &Unite, faite: bool);
    /// « Arrêter » : la conduite s'arrête entre deux unités ; l'import reste ouvert (question 2).
    pub(crate) fn arreter(&mut self);
    /// « 12 fichiers versés sur 80 », d'après les réponses ; dit toutes les dix unités et à la fin.
    pub(crate) fn avancee(&self) -> Option<String>;
}

/// Le choix de l'écran pour un dossier inspecté.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ChoixDuDossier { Aucune, Stems, Main, Generee(String) }

/// Les déclarations à envoyer : chaque choix autre qu'`Aucune` ; un nom de générateur vide refuse
/// avant l'envoi, « Le nom du générateur est vide. », comme le noyau.
pub(crate) fn declarations_choisies(choix: &[(String, ChoixDuDossier)]) -> Result<Vec<DeclarationDUnDossier>, &'static str>;

/// Ce que dit un dossier inspecté : son chemin (« Le dossier versé » pour `""`), ses fichiers audio,
/// et, en attente, « ~ jeu de stems proposé », ou la déclaration de l'import précédent, reproposée.
pub(crate) fn lignes_d_un_dossier(dossier: &DossierInspecte) -> Vec<String>;

/// Le texte du volet quand un import est resté ouvert, que ce dossier ne reprend pas, le même
/// dossier rechoisi compris (question 35 ; V12).
pub(crate) const IMPORT_OUVERT_AILLEURS: &str =
    "Un import est resté ouvert, que ce dossier ne reprend pas : verser celui-ci le clôt, avec ce qu'il a versé.";

/// Ce que dit une déclaration reprise, sans choix (R2) : « Jeu de stems, déclaré à l'ouverture de
/// cet import », « De la main de Sacha, … », « Générée par <nom>, … ».
pub(crate) fn ligne_d_une_declaration_reprise(declaration: &DeclarationDUnDossier) -> String;

/// Le rapport en lignes, les nombres groupés par trois chiffres, une espace insécable (U+00A0) entre
/// deux groupes : « Vus : 80 fichiers, 83 009 octets », « Versés : … », « Dédoublonnés : … »,
/// « Illisibles, parmi les versés : … », « Ignorés : … », « Sans nom, parmi les versés : … (les
/// fichiers n° 4 et 9) », « Origines déclarées : 2 ».
pub(crate) fn lignes_du_rapport(rapport: &RapportDImport) -> Vec<String>;

// boite::ecran : `Ecran::apres_une_unite(self, unite: &Unite, issue: Result<Option<String>, Refus>)`
// note la réponse d'une unité dans la conduite, passe un refus ou un avertissement au bandeau, et ne
// relit la boîte qu'après la réponse de `Unite::Clore`.

// boite::textes : la page d'un fonds vide propose aussi d'importer, hors de la lecture seule.
pub(crate) struct PageDuFondsVide { pub(crate) constats: [&'static str; 2], pub(crate) importer: bool }
pub(crate) fn page_du_fonds_vide(considerees: u64, lecture_seule: bool) -> Option<PageDuFondsVide>;
```

- [ ] **Étape 1 : écrire les tests** (`app/ui/src/import.rs`, et les parts de `boite/` qui portent ce qu'ils
  éprouvent) :
  - `la_conduite_verse_les_fichiers_puis_chaque_jeu_puis_clot` : un `ImportOuvert` aux rangs 1, 4 et deux jeux, `Jeu
    A` (2, 3) et `Jeu B` (5) ; les unités, dans l'ordre : `Fichier(1)`, `Fichier(4)`, `Fichier(2)`, `Fichier(3)`,
    `PriseEnStems("Jeu A")`, `Fichier(5)`, `PriseEnStems("Jeu B")`, `Clore`, puis `None` ;
  - `un_refus_arrete_la_conduite` et `arreter_laisse_l_import_ouvert` : après un refus, ou `arreter`, `suivante` rend
    `None`, et `Clore` n'a pas été demandé ;
  - `l_avancee_se_dit_toutes_les_dix_unites_et_a_la_fin` : sur 25 fichiers, « 10 fichiers versés sur 25 », « 20
    fichiers versés sur 25 », puis « 25 fichiers versés sur 25 », et `None` entre deux ;
  - `les_declarations_choisies_et_un_generateur_sans_nom` ;
  - `cap03_8_un_dossier_dit_son_jeu_propose_en_attente` (la part de l'écran ; V13) : « ~ jeu de stems proposé, en
    attente » ; une déclaration
    reproposée : « ~ Jeu de stems, déclaré à l'import précédent, en attente » ;
  - `une_reprise_montre_ses_declarations_sans_choix` (R2) : `ligne_d_une_declaration_reprise` de `Jeu A` (`Stems`)
    rend « Jeu de stems, déclaré à l'ouverture de cet import », de `Atelier` (`Generee { nom: "Atelier inventé" }`)
    « Générée par Atelier inventé, déclaré à l'ouverture de cet import » ; `IMPORT_OUVERT_AILLEURS` égale le texte de
    la question 35 ;
  - `le_rapport_se_dit_en_lignes` : les comptes de `les_comptes_de_cap03_1` (tâche 3), « Vus : 80 fichiers,
    83\u{a0}009 octets »,
    « Versés : 77 fichiers, 80\u{a0}003 octets », « Dédoublonnés : 3 fichiers, 3\u{a0}006 octets » ;
  - `la_page_d_un_fonds_vide_propose_d_importer` : `importer` vrai hors de la lecture seule, faux en lecture seule, les
    deux constats inchangés ; `le_fonds_vide_se_dit_par_des_constats` se récrit sur le type neuf, la PR le dit ;
  - `l_import_ne_relit_la_boite_qu_a_la_fin` (point de revue 6), au pied de `boite/ecran.rs`, sur l'aide `ecran()` : une
    requête en cours posée (« pont ») ; le rang des demandes de la boîte (`Demandes::rang`) relevé ; trois réponses
    `verser_un_fichier` notées par `Ecran::apres_une_unite` : le rang et la requête en cours inchangés ; la réponse de
    `Clore` : le rang monte d'un, et d'un seul, la requête en cours inchangée.
- [ ] **Étape 2 : voir le rouge** à « Lint », committer (`test: la conduite de l'import, ses déclarations et son
  rapport à l'écran`).
- [ ] **Étape 3 : écrire le code.** Le bouton « Importer un dossier… » dans l'en-tête de la boîte, à côté de « Journal
  du fonds », et dans la page d'un fonds vide quand elle le propose ; il demande `inspecter_un_dossier` ; un refus va
  au bandeau, `None` ne fait rien. Le volet de l'import, comme les trois autres : « Importer un dossier » en titre ;
  le nom du dossier ; ses fichiers par nature, « 80 fichiers : 77 audio, 2 MIDI, 1 texte, 0 ignoré » ; « À verser :
  0,1 Go ; place libre : 12,3 Go », ou le texte du manque, sans « Verser » ; la reprise, « Cet import reprend celui
  resté ouvert : les fichiers déjà versés ne se relisent pas. », puis ses déclarations, une ligne chacune
  (`ligne_d_une_declaration_reprise`), sans bouton ni champ : « Verser » les renvoie telles quelles (R2) ; un import
  ouvert ailleurs, `IMPORT_OUVERT_AILLEURS` (question 35) ; hors d'une reprise, chaque dossier en `fieldset`, sa légende son chemin,
  ses choix en boutons radio, « Aucune déclaration », « Jeu de stems » (s'il a un fichier audio), « De la main de
  Sacha », « Générée par », un champ « Nom du générateur » à `datalist` des générateurs du fonds ; une proposition en
  italique avec `~`, dite « en attente » dans son texte, et « Reprendre cette déclaration » pour une déclaration
  reproposée, qui ne s'applique jamais seule. « Verser » envoie `ouvrir_l_import`, puis chaque unité de la conduite,
  une à la fois, chaque réponse attendue avant la suivante, sans minuterie (I18) ; l'avancée dans une région
  `role="status"` ; « Arrêter » ; un refus et un avertissement vont au bandeau (I29). La clôture : les lignes du
  rapport, puis la boîte relue, une fois ; sous le rapport, tant que le volet reste ouvert, chaque dossier inspecté
  offre « Déclarer l'origine de ce dossier… » (`Portee::Prefixe`, l'import et le chemin du dossier, question 23),
  résolu sur la table que les services gardent de l'import clos, jusqu'à l'inspection suivante (R6) ; ensuite, une
  origine se déclare captation par captation, depuis sa ligne dans le volet de l'idée (tâche 23 ; R25). Pendant
  l'import, Échap ne ferme pas le volet ; après, il le ferme et rend le
  focus au bouton « Importer un dossier… » (`Cible::BoutonDeLImport`). Aucun identifiant ni empreinte à l'écran. Les
  règles de style : les mêmes que les volets, sans couleur, à 200 %.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : l'interface verte, `statique.rs` et
  `encre_statique.rs` de l'interface verts ; `clippy -p interface` sur les deux cibles vert.
- [ ] **Étape 5 : committer** `git add app/ui/src/import.rs app/ui/src/main.rs app/ui/src/boite/etat.rs
  app/ui/src/boite/ecran.rs app/ui/src/boite/vue.rs app/ui/src/boite/volets.rs app/ui/src/boite/textes.rs
  app/ui/src/boite/focus.rs app/ui/style.css` (`feat: l'écran de l'import, ses déclarations, son avancée et son
  rapport`).

### Tâche 23 : la vue d'une idée importée et la lecture simple

**Fichiers :** créer `app/ui/src/ecoute.rs` ; modifier `app/ui/src/main.rs`, `app/ui/src/boite/volets.rs`
(`VoletDeLIdee`), `app/ui/src/boite/ecran.rs`, `app/ui/style.css`.

**Interfaces — consomme :** `VueDUneIdee::captations_vues`, `CaptationVue`, `StemVu`, `TypeVu` (tâche 11) ;
`noyau::donnees::protocole::type_du_format` (tâche 20) ; `stems::TypeDeStem::libelle`.

**Interfaces — produit** (`interface::ecoute`) :

```rust
/// L'adresse d'une captation servie par le protocole : `http://captation.localhost/<empreinte>`
/// (§ 40 · 7.5).
pub(crate) fn adresse(empreinte: &str) -> String;
/// Vrai si l'appareil dit pouvoir lire : `canPlayType` rend `"maybe"` ou `"probably"` ; `""`, faux.
pub(crate) fn lisible_par_l_appareil(reponse: &str) -> bool;
/// « Lecture impossible sur cet appareil. » (CAP-03 · 12).
pub(crate) const LECTURE_IMPOSSIBLE: &str = "Lecture impossible sur cet appareil.";
/// Ce que montre une captation, sa ligne dans le volet de l'idée (R25) : ses lignes (« Nom
/// d'origine : … », « Autres noms : … », l'origine, puis chaque fait, un fait absent écrit `—`),
/// l'adresse à écouter, ou le constat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CaptationMontree { pub(crate) lignes: Vec<String>, pub(crate) ecoute: Option<String>, pub(crate) constat: Option<&'static str> }
/// La captation `vue` montrée, `reponse` ce que `canPlayType` a rendu pour son type ; `None` sans
/// type (`type_du_format`, un `caf` et un `wma` compris) : le constat, sans le demander (R3).
pub(crate) fn captation_montree(vue: &CaptationVue, reponse: Option<&str>) -> CaptationMontree;
/// La captation montrée après l'évènement `error` de son élément audio : le constat, sans adresse.
pub(crate) fn apres_une_erreur(montree: CaptationMontree) -> CaptationMontree;
/// L'origine en mots : « Origine : de la main de Sacha », « Origine : générée par Atelier inventé,
/// Forge », « Origine : — ».
pub(crate) fn origine_en_mots(origine: &OrigineDeclaree) -> String;
/// Le type d'un stem : son texte, son nom accessible, et s'il s'écrit en italique (question 18 ;
/// § 50 · 4.1 et 4.2).
pub(crate) fn type_en_mots(type_vu: &TypeVu) -> (String, String, bool);
/// Le texte versé à montrer : le texte tel quel, ou « Ce texte versé est illisible. ».
pub(crate) fn texte_a_montrer(lu: &TexteLu) -> String;
```

- [ ] **Étape 1 : écrire les tests** (`app/ui/src/ecoute.rs`) :
  - `cap03_12_une_prise_que_l_appareil_ne_sait_pas_jouer_montre_ses_faits` : la vue d'une prise `aiff`, nom
    `Démos/Pont.aiff`, sans durée ; `captation_montree(&vue, Some(""))` rend ses lignes, dont « Nom d'origine :
    Démos/Pont.aiff », « Format : aiff » et « Durée : — », `ecoute: None` et `constat: Some(LECTURE_IMPOSSIBLE)` ; avec
    `Some("maybe")`, l'adresse et aucun constat ; puis `apres_une_erreur` : le constat, sans adresse, les mêmes lignes ;
    une prise `wma`, `captation_montree(&vue, None)` (son format n'a pas de type, et l'écran ne demande pas
    `canPlayType`) : `constat: Some(LECTURE_IMPOSSIBLE)`, sans adresse (question 27 ; R3). Aucune voie ne rend
    d'erreur : la captation montrée est une valeur ;
  - `lisible_par_l_appareil_ne_vaut_que_maybe_ou_probably` : `""`, `"no"`, `"MAYBE"` faux ;
  - `les_faits_se_disent_en_mots_et_un_absent_par_un_tiret` : une prise WAV, « Format : wav », « Durée : 0:01 »,
    « Canaux : 2 », « Fréquence : 44 100 Hz », « Bits : 16 » ; un MIDI, « Format : smf1 », « Division : 480 » ; un
    illisible, `—` à chaque fait ;
  - `l_origine_se_dit_en_mots` ;
  - `le_type_d_un_stem_ne_se_dit_jamais_par_le_seul_glyphe` : proposé, (« ~ voix », « voix, en attente », vrai) ;
    gardé, (« ⇣ voix », « voix, venu d'un fichier », faux) ; choisi, (« voix », « voix », faux) ; `Choeurs` s'écrit
    « chœurs » ;
  - `un_texte_verse_piege_se_montre_tel_quel` : `<script>alert(1)</script> &amp;` revient octet pour octet ; un
    illisible, « Ce texte versé est illisible. » ;
  - `l_adresse_d_une_captation`.
- [ ] **Étape 2 : voir le rouge** à « Lint », committer (`test: la captation importée dans le volet de l'idée et la
  lecture simple`).
- [ ] **Étape 3 : écrire le code.** Le volet de l'idée montre, sous ses champs, une ligne par captation : « Prise ·
  wav », « MIDI versé · smf1 », « Texte versé · chordpro », « Prise en stems · 10 stems » en titre ; ses lignes ; une
  prise, son élément `<audio controls preload="none">`, nommé « Écouter « Démo 1.wav » », ou le constat ; un texte
  versé, « Lire le texte », qui le montre dans un `<pre>`, en nœud de texte ; une prise en stems, chaque stem, son nom,
  son type, « Garder » pour un type en attente, une liste « Choisir le type » (les douze libellés), son écoute, son
  MIDI ; « Déclarer l'origine… » (« De la main de Sacha », « Générée par » et son nom), dont l'avertissement va au
  bandeau. `canPlayType` par `HtmlMediaElement::can_play_type`, que le `web_sys` de Leptos active (vérifié par la
  relecture), sans dépendance de plus, et demandé seulement d'un format qui a un type ; l'évènement `error` passe la
  captation montrée à `apres_une_erreur`.
  Ni lecture automatique, ni minuterie, ni animation.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : l'interface verte, ses deux contrôles statiques
  verts ; `clippy -p interface` sur les deux cibles vert.
- [ ] **Étape 5 : committer** `git add app/ui/src/ecoute.rs app/ui/src/main.rs app/ui/src/boite/volets.rs
  app/ui/src/boite/ecran.rs app/ui/style.css` (`feat: la vue d'une idée importée, ses captations et la lecture
  simple`).

### Tâche 24 : la boîte : la ligne d'une idée sans texte, le regroupement, les titres à 200 %

**Fichiers :** modifier `app/ui/src/boite/textes.rs`, `app/ui/src/boite/etat.rs`, `app/ui/src/boite/ecran.rs`,
`app/ui/src/boite/volets.rs`, `app/ui/src/boite/gestes.rs`, `app/ui/src/boite/vue.rs`, `app/ui/style.css`,
`crates/services/tests/suite/mod.rs`,
`crates/services/tests/commandes_en_memoire.rs`, `crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — produit :**

```rust
// boite::textes : une ligne sans texte se nomme par le nom de son fichier (question 4).
// texte_de_la_ligne : le texte ; sinon « Fichier : <nom_de_fichier> » ; sinon `—`.

// boite::gestes
/// Les commandes d'un regroupement (question 3) : une par captation choisie, dans l'ordre du choix,
/// vers `cible`, née à `le_de_la_cible` ; une captation que la cible porte déjà n'en donne aucune ;
/// chaque décalage lu par `decalage_a` à l'instant qu'il date.
pub(crate) fn commandes_de_regroupement(choisies: &[(Captation, i64)], cible: &str, le_de_la_cible: i64,
    captations_de_la_cible: &[Captation], decalage_a: impl Fn(i64) -> i32) -> Vec<Regrouper>;
// EtatDeLaBoite gagne `a_regrouper: RwSignal<Vec<(Captation, i64)>>` : chaque captation choisie, et
// la naissance de l'idée qu'elle quitte.
```

- [ ] **Étape 1 : écrire les tests.** Au pied de `boite/textes.rs` :
  - `une_idee_sans_texte_se_nomme_par_le_nom_de_son_fichier` : sans texte, `nom_de_fichier` à `Démos/Démo d'hiver 2 -
    01_06_2015 18.42.m4a`, la case rend « Fichier : Démos/Démo d'hiver 2 - 01_06_2015 18.42.m4a », et le nom de la
    ligne le porte ; avec un texte, le texte ; sans l'un ni l'autre, `—` ;
  - `l_annulation_d_un_regroupement_se_dit` (R21 ; V10) : la constante du volet, `ANNULER_LE_DERNIER_REGROUPEMENT`,
    égale « Annuler ne défait que le dernier regroupement : celui de la dernière captation. ».
  Au pied de `boite/gestes.rs` :
  - `regrouper_envoie_une_commande_par_captation_choisie` : deux prises choisies, de deux idées nées à deux instants
    du 25 octobre 2026 (le décalage de Paris, comme `relier_les_cochees_date_chaque_idee_a_sa_naissance`) ; deux
    `Regrouper`, dans l'ordre, chacun avec ses deux décalages ;
  - `une_captation_deja_dans_la_cible_ne_se_regroupe_pas`.
  Au pied de `boite/vue.rs`, sur la feuille de style (l'aide `declaree`) :
  - `les_titres_des_colonnes_passent_a_la_ligne` : `.colonnes .cellules > span` et `.colonnes .cocher` déclarent
    `white-space: normal` et `overflow-wrap: anywhere`.
  Dans la suite, sur les deux paires :
  - `la_recherche_ne_lit_pas_le_nom_d_origine` : une idée importée sans texte, de nom `Démo 1.wav` ; la requête de texte
    « demo » ne la trouve pas (question 4) ;
  - `apres_deux_regroupements_annuler_ne_defait_que_le_dernier` (R21) : deux prises regroupées dans une même idée, deux
    `Regrouper` ; le geste à annuler, que `libelle_d_annulation` reçoit, est le message du second ; annuler ne rend que
    la seconde prise à son idée, la première reste regroupée.
- [ ] **Étape 2 : voir le rouge.** Committer (`test: la ligne d'une idée sans texte, le regroupement, les titres à
  200 %`), pousser. Attendu à « Lint » : `cannot find function commandes_de_regroupement`.
- [ ] **Étape 3 : écrire le code.** Sur la ligne de chaque captation dans le volet de l'idée, une case « À regrouper » ;
  dans le volet d'une autre idée, « Regrouper ici (2) », qui envoie chaque `Regrouper`, un à la fois ; un refus
  s'arrête au bandeau ; la boîte se relit une fois, à la fin, et les choix se vident après un succès. « Annuler » ne
  défait que le dernier regroupement : son libellé le nomme (`libelle_d_annulation`), et le volet le dit sous
  « Regrouper ici (N) » : `ANNULER_LE_DERNIER_REGROUPEMENT` (R21 ; V10). Les titres des colonnes passent à la
  ligne (`white-space: normal`, `overflow-wrap: anywhere`, `hyphens: auto`), la rangée des titres grandissant avec
  eux ; les cases d'une ligne gardent leur coupe (`text-overflow: ellipsis`).
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : l'interface et la suite vertes ; dans `windows`,
  `commandes_sur_fichiers.rs` vert.
- [ ] **Étape 5 : committer** `git add app/ui/src/boite/textes.rs app/ui/src/boite/etat.rs app/ui/src/boite/ecran.rs
  app/ui/src/boite/volets.rs app/ui/src/boite/gestes.rs app/ui/src/boite/vue.rs app/ui/style.css
  crates/services/tests/suite/mod.rs crates/services/tests/commandes_en_memoire.rs
  crates/stockage/tests/commandes_sur_fichiers.rs` (`feat: la ligne d'une idée sans texte, le regroupement à l'écran,
  les titres à 200 %`).

### Tâche 25 : les documents de la PR (d), les délais, et la revue de la PR (d)

**Fichiers :** `docs/20-sfd/21-capter-et-organiser.md` (CAP-03), `docs/40-architecture.md` (§ 7.5),
`docs/50-interfaces-et-appareils.md` (§ 2, § 3.1, § 4.2), `docs/assets/maquettes/fonds.svg`,
`docs/70-qualite-et-verification.md` (§ 3, § 4.1, § 8), `docs/80-glossaire.md`, `docs/pistes.md`,
`docs/registre.md`, `.cargo/mutants.toml`, `CONVENTIONS.md`, `CLAUDE.md`, `ci.yml` (les délais).

- [ ] **Étape 1 : la recherche de LE-2**, citée dans la description de la PR : `lecture simple`, `écouter`,
  `protocole`, `plage`, `regrouper`, `rassembler`, `fichier`, `avancée`, chacun aussi sans accent et sur la
  majuscule accentuée ; et, pour LE-3, `refuse toute requête` dans tout le dépôt, maquettes et commentaires SVG
  compris.
- [ ] **Étape 2 : la SFD et l'architecture.** CAP-03 : la lecture simple, une prise ou un stem à la fois, et « lecture
  impossible sur cet appareil », sans erreur ; l'écran de l'import, ses déclarations, la proposition d'un jeu (CAP-03 ·
  8), son avancée et son rapport. § 40 · 7.5 : le protocole servi, enregistré en asynchrone, chaque requête une
  demande au fil ; le chemin ; la table des types de la question 27, à six lignes (`wav`, `flac`, `mp3`, `m4a`, `ogg`,
  `aiff`), un `caf` et un `wma` non servis (R3) ; `nosniff`, `Accept-Ranges` ; 1 048 576 octets ; 200, 206, 404, 405, 416 ; `GET` et `HEAD` ; un texte
  versé et un rapport lus par une commande ; la CSP inchangée.
- [ ] **Étape 3 : les écrans.** § 50 · 2 : l'import, depuis l'en-tête de la boîte et depuis la page d'un fonds vide.
  § 50 · 3.1 : le volet de l'import, la reprise et ses déclarations sans choix, l'import resté ouvert ailleurs
  (question 35) ; chaque captation, sa ligne dans le volet de l'idée (R25) ; le regroupement, et « Annuler » qui ne
  défait que le dernier regroupement, celui de la dernière captation (R21 ; V10) ; « Fichier : ». § 50 · 4.2 : le type d'un
  stem en exemple d'un élément importé gardé. La maquette du fonds : le bouton « Importer un dossier… » et le volet de
  l'import, ses commentaires SVG compris.
- [ ] **Étape 4 : la qualité, le glossaire, les pistes, le registre.** § 70 · 3 : I31, le protocole (ses tests). § 70 ·
  4.1 : « CAP-03 | 1 à 3, 7 à 13 | INC-1, lot 1 », le 8 et le 12 ajoutés, que l'écran livre (R24). § 70 · 8 : `mutants` (le protocole). Le glossaire : « Lecture
  simple », d'après la recherche. `docs/pistes.md` : « Les titres des colonnes de la boîte à 200 % » sort. LE-17 :
  l'état devient « clos le <date>, à la fusion de la PR (d) #<n>, qui le clôt », suivi des PR (a), (b), (c) et (d) et
  de leurs numéros, et des sections corrigées par chacune, comme LE-16.
- [ ] **Étape 5 : les portes.** `.cargo/mutants.toml` : « Vingt-huit modules », la phrase qui les énumère ;
  `CONVENTIONS.md` : `mutants` (le protocole), `windows` (les tests du protocole) ; `CLAUDE.md` : « Les crates » (le
  noyau : le protocole ; les services : la captation à servir ; l'établi : le protocole servi en asynchrone ;
  l'interface : l'import, l'écoute). Les délais de `verifier`, de `windows` et de `mutants`, remesurés sur l'exécution
  de la tête, au double de la mesure, leur commentaire daté.
- [ ] **Étape 6 : committer** les documents (`docs: le protocole, l'écran de l'import et la lecture simple à la SFD, à
  l'architecture et aux écrans`).
- [ ] **Étape 7 : la revue de la PR (d).** La revue de la PR entière, sur sa tête, par le modèle le plus capable : le
  tableau du § 70 · 4.1 contre les critères de la tranche (CAP-03 · 8 et 12 inscrits, chacun avec son test nommé :
  `cap03_8_trois_wav_de_meme_duree_proposent_un_jeu_et_sans_declaration_trois_prises` et
  `cap03_8_un_dossier_dit_son_jeu_propose_en_attente`, `cap03_12_une_prise_que_l_appareil_ne_sait_pas_jouer_montre_ses_faits`) ;
  chaque section
  que nomme LE-17 contre les diffs des quatre PR ; chaque mécanisme annoncé que la PR touche relu ; la recherche de
  LE-2 ; chaque écran mené au clavier, chaque information dite autrement que par la couleur. Puis les corrections,
  leur relecture sur la tête qui part. Le corps dit que « Annuler » ne défait que le dernier regroupement, celui de la dernière captation (R21 ; V10). La PR
  ne se fusionne qu'après l'essai installé (tâche 26).

### Tâche 26 : l'essai installé et son relevé

**Fichiers :** créer `crates/stockage/examples/dossier_a_verser.rs` ; modifier `.github/workflows/ci.yml` (job
`windows`), `CONVENTIONS.md` (la ligne `windows`), `docs/70-qualite-et-verification.md` (§ 7).

- [ ] **Étape 1 : le programme de l'essai.** `dossier_a_verser <dossier>` refuse un dossier qui existe et n'est pas
  vide ; il écrit des fichiers inventés, aux octets calculés : `Démos/Démo 1.wav` (8 000 Hz, mono, 8 bits, deux
  secondes d'un son audible), `Démos/Démo 2.wav`, `Démos/Copie de Démo 1.wav` (le contenu de `Démo 1.wav`),
  `Démos/Démo d'hiver 2 - 01_06_2015 18.42.m4a` (une boîte `ftyp` seule), `Démos/Pont.aiff` (un en-tête `FORM AIFF`) ;
  `Jeu A/voix.wav`, `Jeu A/basse.wav`, `Jeu A/batterie.wav` (les mêmes trames, des contenus distincts) et
  `Jeu A/voix.mid` (`smf1`) ; `Textes/Pont du soir.cho` (UTF-8, un ChordPro inventé) et `Textes/Ancien.txt` (Latin-1) ;
  `Divers/archive.zip` et `Divers/notice.pdf` ; `Générées/Boucle 1.wav` et `Générées/Boucle 2.wav`. Le job `windows`
  le construit (`cargo build -p stockage --release --example dossier_a_verser`) et le dépose en artefact
  `dossier-a-verser`, à côté de l'installateur ; `clippy -p stockage --all-targets` de `verifier` le passe déjà.
  `CONVENTIONS.md` le dit. Committer `git add crates/stockage/examples/dossier_a_verser.rs .github/workflows/ci.yml
  CONVENTIONS.md` (`feat: le programme qui engendre le dossier de l'essai installé`), pousser ; attendu : `verifier` et
  `windows` verts, les deux artefacts présents.
- [ ] **Étape 2 : préparer l'essai.** L'installateur `etabli-installateur` et le programme de l'exécution verte de la
  tête (étape 1) ; la liste de l'essai, dans le dossier temporaire, à remettre à la maîtrise d'ouvrage. Le fonds
  d'essai de la tranche 4 se garde : la genèse n'a pas changé.
- [ ] **Étape 3 (👤) : l'essai.** La maîtrise d'ouvrage installe l'établi par-dessus celui de la tranche 4, sous
  `app.pupitre.etabli`, sur son fonds d'essai, lance le programme vers un dossier neuf hors du dossier de
  l'application, puis, chaque point noté « vu » ou « écart » :
  - « Importer un dossier… » ouvre la boîte de dossier au-dessus de l'établi ; la fermer sans choisir ne fait rien ;
    choisir le dossier de l'application est refusé, le bandeau dit pourquoi ;
  - l'inspection : les fichiers par nature, la taille à verser et la place libre ; `Jeu A` « ~ jeu de stems proposé,
    en attente » ; `Jeu A` déclaré jeu de stems, `Générées` « Générée par Atelier inventé » ;
  - « Verser » : l'avancée se dit, la fenêtre répond pendant l'import ; « Arrêter » au milieu, puis le même dossier
    rechoisi : la reprise se dit, ses déclarations montrées sans choix, l'import reprend et se clôt ; le rapport : un
    dédoublonné, un illisible, deux ignorés ;
  - la boîte : une idée sans texte se lit « Fichier : Démos/Démo 1.wav » ; dans le volet de l'idée, sa captation dit
    ses faits et son origine ;
    « Écouter » joue la prise, et la lecture se déplace dans la prise ; `Pont.aiff` dit « Lecture impossible sur cet
    appareil. » et ses faits, sans fenêtre d'erreur ; le `.m4a` aussi, après l'échec de sa lecture ;
  - la prise en stems : trois stems, « ~ voix » en italique, « Garder » donne « ⇣ voix » ; « chœurs » choisi ; chaque
    stem écouté seul ; le MIDI de `voix.wav` dit ses faits ;
  - « Lire le texte » montre le ChordPro tel quel ; `Ancien.txt` se dit illisible ;
  - « De la main de Sacha » déclaré sur une prise de `Générées` : l'avertissement au bandeau, l'origine reste générée ;
  - deux prises regroupées dans une idée : les deux idées qu'elles quittent se retirent, le journal du fonds le dit ;
  - le même dossier importé une seconde fois : les déclarations de l'import précédent reproposées, en attente ; le
    rapport ne compte aucun versé ;
  - à l'équivalent de 200 % : « Provenance » et « Relier » passent à la ligne sans se chevaucher ; le volet de l'import
    et les captations du volet de l'idée servent ;
  - tout l'import au clavier, le focus visible à chaque pas ;
  - un Pupitre neuf (un second dossier de données, si la maîtrise d'ouvrage le veut) : la page d'un fonds vide propose
    d'importer, et l'import fait naître le fonds.

  Un écart se corrige dans la PR (d), revu, puis l'essai reprend.
- [ ] **Étape 4 : le relevé.** Au § 70 · 7, une ligne : la date, le SHA de la tête essayée, chaque point et ce qu'il a
  montré, sur la parole de la maîtrise d'ouvrage, sans donnée personnelle ni chemin de la machine ; CAP-03 · 12 vu.
  Committer (`docs: le relevé de l'essai installé de la tranche 5`).
- [ ] **Étape 5 : la relecture** du programme, de son étape de CI et du relevé, sur la tête qui part ; le corps final
  (les tests réécrits : `le_protocole_des_captations_refuse_toute_requete` retiré, `le_fonds_vide_se_dit_par_des_constats`
  récrit ; la table des stems ; « Écarts à la spec : Aucun. ») ; `gh pr ready` ; la fusion par squash,
  `--match-head-commit` nommant cette tête.

### Tâche 27 : la clôture de la tranche

- [ ] Le clone principal à jour (`git fetch`, `main..origin/main` vide) ; les worktrees de la tranche retirés
  (`git worktree remove`, puis le dossier s'il reste) ; l'espace de travail SDD de la tranche supprimé ; la mémoire du
  lot mise à jour : la tranche 5 close, ses PR, ce qu'elle laisse aux suivantes (la source à verser et la copie en flux
  pour la tranche 6, l'archive et la réimportation ; le générateur par l'import pour la tranche 7, le mode
  démonstration ; les clés de l'import et de l'origine, à geler avec la `v0.3.0` et à reporter dans « Ce qui doit
  survivre » à la clôture du lot ; les doutes de ce plan restés ouverts) ; les leçons de la tranche, notées pour le
  registre, que la clôture du lot porte. Ce plan reste dans `.claude/plans/` jusqu'à la clôture du lot, que la
  tranche 9 conduit.

---

## Auto-relecture

- **La couverture.** Chaque puce de « Elle livre » du plan maître a sa tâche : au noyau, les faits d'un fichier versé,
  l'import rejoué, le manifeste à l'octet, l'origine la plus forte, le premier nom vu, la durée d'une prise en stems
  (1 à 6) ; l'import en deux temps, la place libre de `systeme` sous I30, la copie en flux, la première opération
  dans un Pupitre neuf, l'ULID, les idées `importee`, le rapport en captation, la reprise, l'import rejoué, la boîte de
  dossier, la page d'un fonds vide (9, 12, 14, 15, 22) ; les empreintes nommées étendues et le démarrage (7, 9, 10) ;
  les déclarations, l'origine en ensemble, le type d'un stem (4, 6, 13) ; le protocole et la lecture simple (20, 21,
  23) ; l'écran, `⇣` et `~`, les noms et les textes écrits comme texte, I01, les fichiers engendrés (10, 22 à 24) ; le
  test 13 entier et la compaction (16) ; ce que son plan décide (« Les questions tranchées »). Chaque critère de sortie
  a son test : CAP-03 · 1 à 3, 7, 10, 11, 13 (12), 9 (13), 8 (12, inscrit avec l'écran qui le propose, 22 et 25 ;
  R24), 12 (23, et l'essai, 26), 4 en part (12) ; I14 et le test 4 sur la copie en flux (9) ; I31 pour le protocole
  (21) ; I01 (10) ; le test 13 (16) ; `mutants` (1 à 6, 20 ; R14) ;
  les délais (8, 17, 25) ; le chemin long et les deux formes d'un nom dans `windows` (15).
- **Les types.** `Reconnu`, `FaitsWav`, `FaitsMidi`, `ControleDuTexte`, `Hacheur` (1, 2) servent aux tâches 9 et 12 ;
  `FichierDeLImport`, `Comptes`, `rapport`, `place`, `go` (3) à la tâche 12, et `lire_le_rapport`, `RapportLu` (3) à
  la tâche 13 ; `OriginesLues`, `Ajout`, `Declaration`
  (4) aux tâches 6, 10, 12 et 13 ; `TypeDeStem`, `stem_du_midi`, `jeu_propose` (5) aux tâches 12, 13 et 23 ; les
  opérations et `Faits` (6) aux tâches 12 à 14 et 16 ; `copies_a_retirer` (7) aux tâches 12 et 14 ; `Source`,
  `Versement` (9) aux tâches 12, 15 et 16 ; les commandes (11) aux tâches 12 à 15 et 22 à 24 ; `Plage`, `servir` (20)
  à la tâche 21 ; `Conduite` (22) au point de revue 6.
- **Les points de revue** ont chacun leur test : 1, 2, 3 (12), 4 (14, avec et sans coupure ; R1), 5 (21), 6 (22) ;
  les quatre limites sans test (la marque de dernière écriture absente, le réimport d'une seule demande, OneDrive, l'illisible qui ne
  s'écoute pas) sont dites dans le corps des PR (a) et (b).
- **Les nombres recopiés** l'ont été des passages, recalculés par la relecture : le manifeste témoin (195
  octets, `340999f9…`), le rapport témoin (259 octets, `6d161268…`), les empreintes des trois stems inventés ; les
  durées (8 000 trames à 8 000 Hz, 1 000 ms ; 1 000 trames à 44 100 Hz, 22 ms ; 2 × 10¹⁶ trames à 48 000 Hz,
  416 666 666 666 666 ms) ; les tailles des WAV (8 044 et 4 044 octets) ; les comptes de `cap03_1` (83 009, 80 003,
  3 006 octets) ; le manque de place (3 000 008 192 octets, « 3,1 Go ») ; les syncsafe d'ID3 (210, 220, et
  2 130 318, ses quatre octets non nuls) ; la borne d'`ENTIER_SUR` d'une durée (`ENTIER_SUR` ms lisible, un de plus
  illisible) ; la plage
  suffixe bornée (1 951 424) ; les 23 090 opérations, les quelque 3 298 acteurs (un tous les sept ; R10) et les 59
  compactions du test 13.
- **Ce que la rédaction n'avait pas pu vérifier**, faute de lancer cargo, la relecture l'a vérifié sur docs.rs :
  chaque nom d'`automerge` 0.12.0 (`hash_for_opid`, `get_all`, les lectures `*_at`, `set_actor`, `ActorId::from`), de
  `tauri` 2.12.5 et de `tauri-plugin-dialog` 2.8.4 (`register_asynchronous_uri_scheme_protocol`, la boîte de dossier),
  et `HtmlMediaElement::can_play_type`, que le `web_sys` de Leptos active ; la contrainte générale veut encore qu'un
  nom se relise sur docs.rs avant d'écrire, et qu'un nom absent se remplace par son équivalent documenté. Les API de
  la marque de dernière écriture (`MetadataExt::last_write_time`, `mtime`, `mtime_nsec`) sont celles de la bibliothèque standard, hors
  d'`HORLOGE`.
