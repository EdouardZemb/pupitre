# Lot 1 d'INC-1 · le fonds sur l'établi — plan maître

> **Pour les agents :** ce plan n'est pas un plan tâche par tâche. Il fixe les neuf tranches du lot, leur ordre, ce
> que chacune livre et prouve, et ce qui est reporté. Chaque tranche a son plan détaillé, écrit juste avant elle par
> `superpowers:writing-plans`, ajouté dans `.claude/plans/` par sa propre PR de plan, relu par une session neuve,
> puis exécuté par `superpowers:subagent-driven-development`. Un plan de tranche qui contredit ce plan maître le dit
> dans sa PR, et la maîtrise d'ouvrage relit ce point. La table de couverture du relevé des exigences est dans la
> description de la PR de ce plan.

**But :** faire du fonds un lieu sûr sur l'établi. À la `v0.3.0`, Sacha y note ses idées en texte, les trie, les
retrouve, importe son fonds existant (prises, jeux de stems, MIDI et textes), réécoute une prise, archive le fonds
et le reprend d'une racine des données abîmée ; le mode démonstration montre Pupitre sur un fonds inventé. Dès cette
étiquette, le fonds sert pour de vrai (§ 40 · 9).

**Architecture :** le noyau décide, en fonctions pures, de ce qu'écrit une opération nommée, des identifiants, de ce
que permettent une version de schéma, une lignée, un démarrage ou un import. `services` conduit opérations et
requêtes derrière deux contrats, `Documents` et `Captations`, réalisés en mémoire pour les tests et sur fichiers par
`stockage`. `commandes` type arguments, réponses et vues. `etabli` assemble, reçoit du système en un seul module
chemins, heure, aléa et place libre, et sert les commandes et le protocole des captations. L'interface montre des
vues et envoie des commandes.

**Pile :** Rust 1.99.1, édition 2024 ; `automerge` 0.12.0 ; `sha2` 0.11.1 ; `tauri` 2.12.5 et `tauri-plugin-dialog`
2.8.4 ; `leptos` 0.8.23. Nouvelles : `serde`, `serde_json`, `ulid` et `getrandom`, chacune à sa dernière version
publiée, que sa PR relève (§ 41 · 6). Tests : `proptest` 1.12.0 ; en CI, `cargo-mutants` 27.1.0, trunk 0.22.1 et
tauri-cli 2.12.4.

**Spec :** § 40 · 3, 7.5, 8 et 9 (lot 1) ; § 41 · 4, 6 et 7 ; § 50 · 2 à 7 ; § 31 · 1, 2.1 à 2.6, 3, 4.3, 6.1, 7,
9.1 et 10 (tests 1, 3, 4, 7 à 11, 13 et 14) ; § 30 · 2, 8.1, 9, 11 et 13 ; CAP-01, CAP-02, CAP-03, CAP-06, CAP-07 ;
SYN-07 · 4 à 6 ; PRO-02, en partie ; ADR-001 à 004, 006, 010, 019 à 024 ; § 70 · 1, 2, 3 (I01, I09, I10, I13, I14,
I17, I18, I28 à I31), 7 et 8 ; M-0001 ; au registre, AM-3, AM-4 et LE-10 ; le plan d'H07, décision 8. L'issue : #24.

---

## Les décisions de ce plan

### Les choix de la maîtrise d'ouvrage (2026-11-09)

1. **Un plan maître, puis un plan par tranche.** La maîtrise d'ouvrage relit ce plan ; un plan de tranche est relu
   par une session neuve, et par elle seulement s'il change ce plan.
2. **Neuf tranches**, dans l'ordre des sections qui suivent. Le plan en comptait huit ; la maîtrise d'ouvrage a
   scindé la huitième, « la revue finale et la clôture », le 2026-11-13 : « la revue finale » (tranche 8) et « la
   clôture » (tranche 9), comme la méthode le veut (`docs/modele/methode.md`, « Un lot en tranches »).
3. **AM-4, point 7, au lot 4**, la sauvegarde ; l'archive complète lit la racine des données depuis Pupitre.
4. **La mutation reste sur le noyau seul**, où vont toutes les règles qui décident. Un module de règle entre dans
   `.cargo/mutants.toml`, avec ses tests, dans la PR qui l'écrit, qui remesure les deux délais à froid (§ 70 · 1) ;
   un plan de tranche regroupe ces extensions quand il le peut.
5. **L'exécution :** une tâche, un agent, une revue ; une ou plusieurs PR par tranche ; la session fusionne quand
   tout est vert (mode `session`), sauf aux étapes 👤.
6. **Avant la `v0.3.0`, rien n'est installé entre-temps**, chaque tranche laisse l'installateur NSIS en artefact, et
   l'étiquette est à la maîtrise d'ouvrage. Ce plan le lit ainsi (décision 21, question 1, tranchée) : aucune installation
   pour un usage réel, aucune donnée réelle. AM-3, I31 et le § 70 · 7 demandent pourtant l'application installée : 👤
   la maîtrise d'ouvrage installe des essais, sur des fonds d'essai, sous `app.pupitre.etabli` à la fin de la
   tranche 3, où seule une installation montre une fenêtre blanche due à la CSP, et en revue finale ; puis celui de
   la PR de clôture, sous `app.pupitre`, sans rien noter hors du mode démonstration. Sans l'essai de la tranche 3,
   la PR de ce plan accepte par écrit qu'une fenêtre blanche ne se voie qu'en revue finale.

### Ce que ce plan tranche

7. **Les crates** (§ 40 · 3). `synchro` ne change pas.

   | Crate | Ce qu'elle porte au lot 1 | Dépend de |
   |---|---|---|
   | `commandes`, nouvelle | Les types des commandes (arguments, réponses, vues) et la liste de leurs noms | `serde` |
   | `services`, nouvelle | Les opérations nommées et les requêtes ; les contrats `Documents` et `Captations` ; leurs adaptateurs en mémoire, sous la fonction de Cargo `en-memoire`, que seuls des tests activent ; le générateur du fonds de mesure et le jeu factice | `noyau`, `commandes`, `automerge` |
   | `stockage` | Les adaptateurs sur fichiers ; la fonction `pannes`, que seuls ses tests activent | `noyau`, `synchro`, `services`, `automerge`, `sha2` |
   | `etabli` | L'assemblage ; le module `systeme`, seul à recevoir du système chemins, heure, aléa et place libre | `stockage`, `services`, `commandes`, `noyau`, `cahier`, `tauri`, `tauri-plugin-dialog`, `getrandom` |
   | `interface` | Des vues et des commandes | `noyau`, `commandes`, `serde_json` |

   Une étape de `verifier` lit `cargo tree` : l'interface, en natif et en `wasm32-unknown-unknown`, ne tire ni
   `automerge`, ni `services`, ni `stockage` (dès la tranche 2) ; `etabli`, hors de ses dépendances de test,
   n'active ni `en-memoire` ni `pannes` (dès la tranche 3).
8. **Le noyau décide, l'adaptateur applique** (§ 31 · 3 ; § 70 · 1). Le noyau, sans `automerge`, lit une vue du
   fonds que construit l'adaptateur `Documents`, vérifie les préconditions d'une opération et rend ses écritures
   (poser une valeur, créer un objet avec ses conteneurs, ôter une clé) et son message ; l'adaptateur les applique
   en une transaction (`commit_with`). Hors des deux adaptateurs `Documents`, aucun code n'écrit dans un document
   (I28). La teneur, sur un arbre de valeurs neutre que `stockage` tire du document, le démarrage, la version de
   schéma et la lignée ont aussi leurs règles au noyau.
9. **Le schéma de la `v0.3.0` s'écrit en entier en tranche 2** (§ 31 · 2, 2.3 et 2.4). La genèse crée un conteneur
   de la racine du document par sorte d'objet qu'INC-1 écrit : idées et prises, déjà là ; MIDI versés seuls, textes
   versés, liens, requêtes enregistrées, imports, origines déclarées ; photos, que le lot 2 écrira ; aucune
   migration n'est donc publiée aux lots 1 et 2. Les conteneurs d'un objet naissent avec lui ; un objet nommé par
   une empreinte suit la vigilance 1. Les lignées remplacées s'écriront à la naissance d'un document reconstruit
   (lot 4), sous une clé que seule elle pose. La genèse change en tranche 2, puis seulement si la tranche 4 ou 5
   trouve un manque, que sa PR dit ; elle se fige à la `v0.3.0`. L'état « à trancher » du lot 3 s'écrira en valeurs
   simples dans l'objet, ou par la première migration publiée ; motifs, morceaux et albums, par une migration, en
   INC-2.
10. **Les identifiants, l'heure, l'aléa** (§ 30 · 2 ; § 41 · 6). Le noyau forme les ULID (idée, requête enregistrée,
    document, lignée, import) par la crate `ulid`, sans ses fonctions par défaut, donc sans horloge ni aléa ; si
    elle ne se compile pas ainsi pour `wasm32-unknown-unknown`, l'encodage s'écrit au noyau, et le § 41 · 6 se
    corrige. Il forme, de seize octets reçus, l'UUID version 4 de l'identifiant de propriétaire, sans crate ; un
    seul service fait naître le fonds et forme son identifiant de propriétaire. L'heure et l'aléa se lisent dans le
    seul module `systeme` (`SystemTime`, `getrandom`), dès la tranche 3 ; avant, les tests les donnent. L'acteur
    neuf de chaque ouverture reste celui d'Automerge (`ActorId::random()`, dans `stockage`), seule exception au
    contrôle statique de l'aléa, qui la cite. Le décalage horaire local d'un message (« Idée du 3 novembre ») vient
    de l'interface, comme la date du cahier.
11. **Les commandes et les droits** (§ 40 · 7.5 ; § 41 · 6). L'interface passe les types de `commandes` à `invoke`
    par `serde_json`, lu par `JSON.parse` : la même sérialisation des deux côtés, que les tests de `commandes`
    éprouvent en natif ; `withGlobalTauri` reste posé. La liste des noms des commandes vit dans `commandes` :
    l'interface y prend le nom qu'elle invoque, le `build.rs` d'`etabli` en tire le manifeste de l'application, le
    test d'I31 la compare à `generate_handler!` et aux droits, qui la recopient (décision 20, (f)).
    `capabilities/windows.json` (`"platforms": ["windows"]`) nomme chaque commande et les seuls droits du noyau de
    Tauri employés ; `default.json` disparaît ; aucune autre plateforme, aucun droit de greffon. Aucun signal
    « document changé » : au lot 1, rien ne change un document hors de l'interface, pas même l'import, commande
    attendue.
12. **Le stockage durci** (§ 31 · 2.1, 2.2, 2.4 ; AM-4). Un fichier illisible (un fichier d'ajout qui trahit son
    empreinte, un fichier complet que `load` refuse, une modification qui attend ses dépendances) ouvre le document
    en lecture seule : le document montre ce qui se lit, nomme le fichier, n'écrit ni ne compacte ; faute de nœud,
    il n'envoie ni ne reçoit (la réparation par le nœud : lot 3). Devant une version de schéma plus récente, le
    document s'ouvre aussi en lecture seule, mais reçoit sans rien transmettre (§ 31 · 2.4), ce qu'un test de
    `synchro` éprouve en tranche 1 ; s'il ne le peut, la contradiction rejoint la décision 20. Une captation se
    verse toujours. Une relecture du disque qui échoue met le document en lecture seule. Une compaction d'ouverture
    infidèle, ou refusée par le disque, n'empêche pas d'ouvrir : avertissement, fichiers en place, compaction
    retentée. Après un fichier d'ajout en place, une compaction qui échoue ou un `rendre_durable` refusé laisse
    l'opération « faite, avec un avertissement » : dite refusée, elle paraîtrait à la relecture, contre le test 8.
    Le démarrage, après le verrou, traite les voisins de `documents/` à l'ouverture du document ; puis chaque nom
    provisoire de `captations/` dont le contenu donne une empreinte que le fonds nomme, et qu'aucun fichier ne porte
    encore, prend ce nom, partout et non sous Windows seulement (la tranche 1 corrige le § 31 · 6.1). Cette règle
    vit au noyau ; chaque tranche qui ajoute une sorte de captation nommée par le fonds l'étend, avec le cas du
    test 4.
13. **WebAssembly.** `services` se compile en WebAssembly, avec la fonction `wasm` d'`automerge`, avec la
    construction navigateur, à la ligne 5 des portes (§ 40 · 9 ; la seconde voie de la décision 1 du plan d'H07). Le
    lot 1 ne pose aucune dépendance de cible `wasm32`, qu'aucune CI n'éprouverait ; un contrôle statique garde
    `services` sans fichier, fil, réseau ni horloge.
14. **Les garde-fous vivent dans les jobs requis**, en tests Rust, comme `encre_statique.rs` ; aucun job nouveau (il
    demanderait un geste 👤). La sentinelle d'I30 est une étape du job `windows` (§ 70 · 3).
15. **Le mode démonstration se choisit par un argument de relance**, `--demonstration` (§ 31 · 1) : un lancement
    ordinaire ouvre toujours `donnees/`, et rien ne s'écrit pour s'en souvenir.
16. **Le gel de la genèse** (test 11 ; ADR-023). Les empreintes de référence (la genèse, la teneur d'un document
    témoin, les octets d'un manifeste témoin) vivent dans un seul fichier de tests. La PR de clôture ajoute à
    `verifier` une étape qui lit la branche de base (`git fetch --depth=1 origin "$BASE_REF"`, `BASE_REF` venant de
    `github.base_ref`, comme dans `mutants`), puis refuse toute ligne de ce fichier changée ou retirée par rapport à
    `origin/$BASE_REF` ; une ligne ajoutée, celle d'une migration nouvelle, passe.
17. **Le budget** (§ 41 · 6). Entrent `serde`, `serde_json` (interface ; en test pour `commandes`), `ulid` (noyau),
    `getrandom` (`etabli`) ; `tauri-plugin-dialog` sert aussi au dossier à verser et à la destination de l'archive,
    ce que la tranche 5 écrit. N'entrent ni décodage audio (ADR-024, décision 6), ni moteur d'index ou base, ni
    crate de place libre (`systeme` la lit par `GetDiskFreeSpaceExW`, déclaré à la main, et les services la
    reçoivent en argument), ni crate de repli des accents (une table au noyau).
18. **Annuler** (§ 30 · 11 ; CAP-02) vise le dernier geste de tri fait sur l'appareil depuis l'ouverture. Une note,
    une requête enregistrée, un import ou une déclaration ne le visent ni ne le remplacent. Une annulation vide la
    place : un second « annuler » est refusé, et le dit.
19. **L'import** (ADR-024 ; CAP-03). Lire la signature n'est pas ouvrir : un fichier ignoré n'est lu que de ses
    premiers octets, ce que prouve un contrat de lecture qui compte les octets lus. Les fichiers illisibles sont
    versés, et comptés aussi à part : vus = versés + dédoublonnés + ignorés ; un fichier que le disque refuse de
    lire arrête l'import, qui le nomme. L'empreinte du rapport, une empreinte connue, vit sous l'import, dans le
    document, comme l'état d'un import en cours. Devant un fonds d'une version de schéma plus récente, l'import
    entier est refusé.
20. **Les contradictions de la spec que ce plan lève**, que sa PR consigne au motif « Une spec qui se contredit »
    (troisième constat, après AM-2 et LE-6 : une porte est due, décision 21, question 3) : (a) § 40 · 3 et 9 sur les
    services en WebAssembly ; (b) AM-4 (4) et § 70 · 1 sur la lignée remplacée ; (c) au § 41 · 6, « sans crate de
    plus » vaut pour l'UUID ; (d) CAP-03 dit « non décodable » ce que l'ADR-024 réserve au nœud ; (e) SYN-07 dit
    qu'un import écrit des propositions, jamais des décisions, quand CAP-03 et l'ADR-024 lui font écrire des idées
    et des captations ; (f) le § 40 · 7.5 met la liste des commandes dans les seuls fichiers de droits, ce plan dans
    `commandes`, que les droits recopient et qu'I31 compare. La tranche 2 corrige (a) au § 40 · 3 et (c) au
    § 41 · 6 ; (b) se lève par le report écrit du point (4) d'AM-4 ; la tranche 3 corrige (f), la tranche 5 (d) et
    (e).
21. **Les questions à la maîtrise d'ouvrage**, tranchées à la relecture de la PR de ce plan, le 2026-11-09, chacune
    selon son défaut :
    1. La lecture du choix 6 (décision 6) : aucune installation pour un usage réel ; les essais installés
       des tranches 3 et 8 ont lieu, sur des fonds d'essai.
    2. La recherche par nom d'origine, que CAP-06 ne prévoit pas, quand CAP-03 dit ce nom souvent la seule clé d'une
       prise importée : rien au lot 1, une piste dans `docs/pistes.md`, que la PR de ce plan écrit, sauf si une PR de
       conception amende CAP-06 avant la tranche 4.
    3. La porte due de la décision 20 : chaque critère que le lot livre a un test qui le nomme
       (`cap06_7_…`, la forme des tests d'INC-0) et en calcule l'exemple (LE-6) ; `verif-faits.mjs` refuse un
       critère de la liste des critères livrés, écrite au § 70 · 4, sans test qui le nomme ; une PR à part, avant
       la tranche 1. Aucune porte ne lit le sens de deux phrases : pour le reste, le passage croisé d'AM-2 demeure.
    4. Le moyen de voir « le même journal du fonds » au contrôle d'installation du § 70 · 7 : la vue du
       fonds montre, à un geste, la liste en lecture seule des messages du journal, avec leurs heures (`—` pour la
       genèse), en tranche 4.

---

## Contraintes générales

- **En CI seulement.** Ni cargo, ni trunk, ni npm sur le poste : chaque vérification de Rust se lit dans le job que
  nomme le critère, le rouge puis le vert dans `verifier`, le vert aussi dans `windows` si le critère l'y met. Seul
  `rustfmt --edition 2024` formate les `.rs` touchés ; les vérifications de documentation tournent sur le poste,
  après le `git add` (LE-7). Les commandes de Rust prennent `-p <crate>`, et `ci.yml` nomme chaque crate qu'il passe
  à `clippy` et aux tests.
- **Le noyau** reste sans entrée-sortie, ni `automerge`, ni horloge, ni aléa ; chaque règle qui décide y vit, sous
  mutation (décision 4). **Les chemins se reçoivent** (I30) : hors de `systeme`, aucun code ne devine un chemin ;
  chaque test prend son dossier jetable par une seule fonction de sa crate. **Aucune donnée réelle** : textes et
  noms inventés ; identifiants, documents et captations engendrés à l'exécution ; aucun audio, MIDI ni `.automerge`
  suivi.
- **Les mots du glossaire**, dits en entier : la racine des données et la racine du document, la version de schéma,
  l'origine déclarée, l'identifiant de propriétaire, le manifeste de l'application. Une PR qui ajoute un mot au
  glossaire le cherche d'abord (`git grep -n -i -w <mot> -- docs README.md CONVENTIONS.md`), requalifie chaque
  emploi antérieur et cite la recherche (LE-2) ; une PR qui périme un mot ou un identifiant le cherche dans tout le
  dépôt, maquettes et commentaires SVG compris (LE-3).
- **Chaque écran** qu'une tranche ajoute se mène au clavier, focus visible, avec ses libellés, au vocabulaire du
  glossaire, sert à 200 % (§ 50 · 7), et ses feuilles passent I17 et I18 (`encre_statique.rs`).
- **Les critères livrés.** Chaque PR qui livre un critère de la SFD en entier l'inscrit au tableau du § 70 · 4.1, et
  un test le nomme (`cap06_7_…`) et en calcule l'exemple ; `verif-faits.mjs` refuse un critère inscrit sans test qui
  le nomme (décision 21, question 3). La porte ne voit pas un critère livré que la PR n'inscrit pas : la revue de
  chaque PR compare le tableau aux critères que la tranche dit livrer.

---

## Mécanismes annoncés

- `rust-toolchain.toml:2` — `channel = "1.99.1"` — la chaîne Rust 1.99.1, épinglée, que chaque job Rust installe
  (`verifier`, `windows`, `mutants`, et `mutants-complet.yml`) ; `socle`, `contenu`, `documentation` et `texte-pr`
  ne l'installent pas (Pile).
- `Cargo.toml:6` — `edition = "2024"` — l'édition 2024, que chaque crate hérite de l'espace de travail (Pile ;
  `rustfmt --edition 2024`, contraintes générales).
- `crates/stockage/Cargo.toml:9` — `automerge = "0.12.0"` — automerge 0.12.0, dans `stockage` et dans `synchro`
  (`crates/synchro/Cargo.toml:9`), jamais au noyau (Pile ; décision 8).
- `crates/noyau/Cargo.toml:9` — `sha2 = { version = "0.11.1", default-features = false }` — sha2 0.11.1, au noyau
  comme au stockage (Pile).
- `app/src-tauri/Cargo.toml:18` — `tauri = { version = "2.12.5"` — tauri 2.12.5 (Pile).
- `app/src-tauri/Cargo.toml:19` — `tauri-plugin-dialog = "2.8.4"` — tauri-plugin-dialog 2.8.4, déjà dépendance de
  l'établi (Pile ; décision 17).
- `app/ui/Cargo.toml:10` — `leptos = { version = "0.8.23"` — leptos 0.8.23, rendu côté client (`csr`) (Pile).
- `crates/noyau/Cargo.toml:13` — `proptest = "1.12.0"` — proptest 1.12.0, dépendance de test du noyau et du stockage
  (Pile).
- `.github/workflows/ci.yml:197` — `cargo-mutants --version 27.1.0` — cargo-mutants 27.1.0, installé par `mutants`, et
  par `mutants-complet.yml` à la même version (Pile).
- `.github/workflows/ci.yml:148` — `trunk --version 0.22.1` — trunk 0.22.1, installé par `windows` avant
  l'installateur (Pile).
- `.github/workflows/ci.yml:150` — `tauri-cli --version 2.12.4` — tauri-cli 2.12.4, de même (Pile).
- `.copier-answers.yml:8` — `fusion: session` — le mode de fusion du dépôt : la session fusionne sa PR quand tout est
  vert (décision 5).
- `scripts/modele/garde_claude.py:127-129` — `envoi d'une étiquette ou écriture de publication` — le garde de Claude
  refuse à la session l'envoi d'une étiquette : `v0.3.0` est à la maîtrise d'ouvrage (décision 6 ; tranche 9).
- `app/src-tauri/tauri.conf.json:21` — `"targets": ["nsis"]` — l'établi se construit en un installateur NSIS, et en
  lui seul (décision 6).
- `.github/workflows/ci.yml:159-163` — `name: etabli-installateur` — le job `windows` dépose l'installateur en
  artefact, et échoue sans lui (`if-no-files-found: error`) : chaque tranche le laisse (décision 6 ; le déroulé,
  point 3).
- `.github/workflows/ci.yml:151-153` — `run: cargo tauri build` — l'installateur se construit sans être lancé, et
  aucun test n'ouvre la fenêtre (`app/ui/tests/encre_statique.rs:2-3`) : une fenêtre blanche due à la CSP ne se voit
  qu'installée (décision 6).
- `app/src-tauri/tauri.conf.json:5` — `"identifier": "app.pupitre.etabli"` — l'identifiant des essais, que Tauri
  ajoute au dossier local de l'utilisateur pour former le dossier de l'application : à la clôture, `app.pupitre` prend
  un autre dossier, et les essais restent intacts (décision 6 ; tranche 9).
- `crates/stockage/Cargo.toml:9-12` — `synchro = { path = "../synchro" }` — le stockage dépend déjà du noyau, de
  `synchro`, d'`automerge` et de `sha2` ; le lot lui ajoute `services` (décision 7).
- `app/src-tauri/Cargo.toml:15-19` — `cahier = { path = "../../crates/cahier" }` — l'établi dépend déjà du cahier, du
  noyau, de `tauri` et de `tauri-plugin-dialog` ; le lot lui ajoute `stockage`, `services`, `commandes` et `getrandom`
  (décision 7).
- `app/ui/Cargo.toml:8-13` — `noyau = { path = "../../crates/noyau" }` — l'interface dépend aujourd'hui du noyau, de
  Leptos et des liaisons de WebAssembly (`js-sys`, `wasm-bindgen`), sans `automerge`, `services` ni `stockage`
  (décision 7, l'étape `cargo tree`).
- `crates/synchro/Cargo.toml:8-10` — `noyau = { path = "../noyau" }` — `synchro` ne dépend que d'`automerge` et du
  noyau, et le lot ne lui ajoute aucune dépendance (décision 7).
- `Cargo.toml:3` — `resolver = "3"` — une fonction qu'active une dépendance de test ne gagne pas la construction
  ordinaire : `etabli` peut activer `en-memoire` ou `pannes` dans ses tests sans les livrer (décision 7).
- `crates/noyau/src/lib.rs:1` — `pur, sans entrée-sortie` — le noyau est pur aujourd'hui, sans fichier, horloge ni
  aléa : il le reste, et le contrôle statique de la tranche 2 le fige (décision 8 ; contraintes générales).
- `crates/noyau/Cargo.toml:8-10` — `ttf-parser = "0.25.1"` — le noyau ne dépend ni d'`automerge` ni d'une crate
  d'horloge ou d'aléa : `sha2`, sans ses fonctions par défaut, et `ttf-parser` (décision 8).
- `crates/stockage/src/documents.rs:259-264` — `tx.commit_with(a_l_heure(message, heure))` — une modification
  s'applique en une transaction, avec son message et son heure : l'adaptateur `Documents` appliquera ainsi les
  écritures du noyau, en une opération nommée (décision 8 ; tranche 2).
- `crates/synchro/src/lib.rs:75-79` — `doc.receive_sync_message(&mut self.etat, message)` — la réception écrit dans un
  document les modifications d'une autre copie, hors des adaptateurs `Documents` : I28 la compte comme une reprise,
  non comme une écriture (décision 8 ; tranche 2).
- `crates/noyau/src/donnees/fonds.rs:11-14` — `pub const PRISES: &str = "prises";` — la genèse d'aujourd'hui crée les
  conteneurs des idées et des prises, « déjà là » (décision 9).
- `crates/noyau/src/donnees/fonds.rs:41-44` — `pub const GENESE: &[u8]` — la genèse vit au noyau, en octets : la
  tranche 2 la récrit, puis elle se fige à la `v0.3.0` (décision 9).
- `crates/stockage/src/documents.rs:585` — `automerge.set_actor(ActorId::random());` — chaque ouverture tire un acteur
  neuf par Automerge, dans `stockage` : l'exception que cite le contrôle statique de l'aléa ; la relecture du disque
  en tire un aussi (`:495`) (décision 10).
- `app/ui/src/export.rs:44-47` — `fn date_du_jour()` — la date du cahier se lit dans l'interface, à l'heure de
  l'ordinateur, et passe en argument à la commande : le décalage horaire d'un message viendra de l'interface de même
  (décision 10).
- `app/src-tauri/tauri.conf.json:13` — `"withGlobalTauri": true` — l'API de Tauri est posée sur `window.__TAURI__`, où
  l'interface prend `invoke` (`app/ui/src/export.rs:51-55`) : elle reste posée (décision 11).
- `app/ui/src/export.rs:86` — `JsValue::from_str("exporter_cahier")` — l'interface écrit en toutes lettres le nom de
  la commande qu'elle invoque : elle le prendra dans `commandes` (décision 11).
- `app/src-tauri/build.rs:2` — `tauri_build::build();` — aucun manifeste de l'application : la tranche 3 l'y tire de
  la liste des noms de `commandes` (décision 11).
- `app/src-tauri/src/lib.rs:130` — `tauri::generate_handler![exporter_cahier]` — les commandes servies, une seule
  aujourd'hui, que le test d'I31 comparera à la liste de `commandes` et aux droits (décision 11).
- `app/src-tauri/src/lib.rs:85-86` — `async fn exporter_cahier(` — la commande d'INC-0, que les droits de la tranche 3
  permettent par son nom (décision 11 ; tranche 3).
- `app/src-tauri/capabilities/default.json:4` — `faute de manifeste dans build.rs` — `exporter_cahier` est permise par
  défaut, sans droit qui la nomme, et la boîte d'enregistrement s'ouvre en Rust, sans droit du greffon (décision 11).
- `app/src-tauri/capabilities/default.json:5-6` — `"permissions": ["core:default"]` — le seul fichier de droits, sans
  `platforms`, donne `core:default` à la fenêtre `main` : `windows.json` le remplace, et `default.json` disparaît
  (décision 11).
- `app/src-tauri/src/lib.rs:113-114` — `blocking_save_file` — `tauri-plugin-dialog` sert déjà, côté natif, à la
  destination du cahier : il servira aussi au dossier à verser et à la destination de l'archive (décision 17 ;
  tranches 5 et 6).
- `.github/workflows/ci.yml:110` — `cargo clippy -p interface --target wasm32-unknown-unknown` — la CI construit en
  `wasm32-unknown-unknown` par deux voies, ce lint de l'interface et `trunk build --release`, que lance
  `cargo tauri build` dans `windows` : l'une et l'autre passent par l'interface seule, qui compile le noyau, donc
  `ulid` dès qu'il y entre (décision 10), mais ne dépend pas de `services` : une dépendance de cible `wasm32` de
  `services` n'y serait pas éprouvée (décision 13).
- `app/src-tauri/tauri.conf.json:7` — `"script": "trunk build --release"` — `cargo tauri build` construit d'abord
  l'interface en WebAssembly par trunk, dans `app/ui` : la seconde construction en `wasm32-unknown-unknown` de la
  CI, dans `windows` (`.github/workflows/ci.yml:151-153`) (décision 13).
- `CONVENTIONS.md:156-161` — `requiert six contrôles` — la règle de protection ne vit que sur GitHub, et requiert
  `verifier`, `windows` et `mutants` avec le socle et `texte-pr` ; `contenu` et `documentation` n'y sont pas requis :
  requérir un job nouveau changerait la règle sur GitHub, un geste de la maîtrise d'ouvrage (décision 14).
- `scripts/verif-faits.mjs:32-33` — `la liste est comparée à la règle de protection` — en CI, la porte `documentation`
  compare la liste de « Protection de `main` » à la règle réelle : un contrôle requis de plus s'y écrirait aussi
  (décision 14).
- `scripts/verif-faits.mjs:24-26` — `CLAUDE.md nomme les jobs` — un job nouveau se nommerait aussi dans le tableau des
  portes de `CONVENTIONS.md` et dans `CLAUDE.md` (décision 14).
- `app/ui/tests/encre_statique.rs:1-2` — `le mode encre tient (I17)` — un garde-fou de l'interface est un test Rust
  qui lit les sources, lancé par `verifier` (`cargo test -p interface`, `.github/workflows/ci.yml:112`) : le modèle
  des garde-fous du lot (décision 14).
- `.github/workflows/ci.yml:199-201` — `BASE_REF: ${{ github.base_ref }}` — `mutants` prend la branche de base de
  `github.base_ref`, par `BASE_REF`, et ne tourne que sur une PR (`:179`) : l'étape du gel prend `BASE_REF` de même
  (décision 16).
- `.cargo/mutants.toml:6-20` — `examine_globs = [` — la mutation examine des modules du noyau nommés un à un : un
  module de règle y entre dans la PR qui l'écrit (décision 4).
- `.cargo/mutants.toml:26-27` — `Un nouveau fichier de tests de ces modules s'ajoute à cette liste` — ses fichiers de
  tests entrent avec lui dans `additional_cargo_test_args` (décision 4).
- `.github/workflows/ci.yml:203` — `cargo mutants -p noyau --in-diff pr.diff` — la mutation porte sur le noyau seul,
  et, sur une PR, sur les seules lignes qu'elle change (décision 4).
- `.github/workflows/ci.yml:181-185` — `À remesurer quand le périmètre de la mutation s'étend` — le délai de `mutants`
  vaut deux fois un passage complet mesuré à froid, comme celui de `mutants-complet.yml` : la PR qui étend
  `.cargo/mutants.toml` les remesure (décision 4).
- `.github/workflows/ci.yml:72-75` — `mesuré à froid` — le délai de `verifier` est à ramener à deux fois un passage
  mesuré à froid, et celui de `windows` vaut deux fois son premier passage (`:123-128`) : une tranche qui alourdit les
  tests les remesure (le déroulé, point 3).
- `.github/workflows/ci.yml:96-97` — `cargo fmt --all --check` — le format se vérifie dans `verifier` ; sans cargo sur
  le poste, `rustfmt --edition 2024` formate les fichiers touchés (contraintes générales).
- `.github/workflows/ci.yml:99` — `cargo clippy -p noyau -p cahier -p stockage -p synchro --all-targets` — le lint de
  `verifier` nomme chaque crate : `-p commandes -p services` s'y ajoutent (contraintes générales ; tranche 2).
- `.github/workflows/ci.yml:112` — `cargo test -p noyau -p cahier -p interface -p stockage -p synchro` — les tests de
  `verifier` nomment de même chaque crate (contraintes générales ; tranche 2).
- `.github/workflows/ci.yml:146` — `cargo test -p stockage -p synchro` — `windows` teste déjà le stockage : la suite
  des commandes sur la paire de fichiers, qui y vit, y tourne sans étape de plus (tranche 2).
- `.github/workflows/ci.yml:156-158` — `cargo test -p etabli` — l'établi ne se teste que dans `windows`, après
  l'installateur, dont ses tests demandent `app/ui/dist` : I31 pour `etabli` et le second test d'I30 y tournent
  (tranches 3 et 7).
- `scripts/verif-confidentialite.sh:19` — `suivis() { git ls-files -z` — la porte `contenu` ne lit que les fichiers
  indexés : les vérifications du poste se lancent après le `git add` (LE-7 ; contraintes générales).
- `scripts/verif-confidentialite.sh:46-48` — `mid|midi` — la porte `contenu` refuse tout fichier suivi dont le nom
  finit, sans égard à la casse, par une extension de sa liste fermée,
  `mp3|m4a|wav|aif|aiff|flac|ogg|mid|midi|musicxml|mxl` : un fichier audio d'une autre extension, ou un WAV sous un
  autre nom, passe ; les fichiers des tests de l'import et les captations du jeu factice s'engendrent à l'exécution,
  et le plan de la tranche 5 ajoute à la liste chaque extension audio ou MIDI que son import reconnaît (contraintes
  générales ; tranches 5 et 7).
- `scripts/verif-faits.mjs:696-697` — `est inscrit, sans test qui le nomme` — un critère inscrit au § 70 · 4.1 sans
  test qui le nomme, marqué `#[test]` et non `#[ignore]`, est refusé ; la porte ne lit que les critères inscrits
  (`:683`) (contraintes générales ; décision 21, question 3).
- `scripts/verif-faits.mjs:643-644` — `export function fonctionsNommeesCritere` — un test de critère se nomme comme
  ceux d'INC-0, `cap06_7_…`, le code de la fonction collé à son numéro (contraintes générales).
- `scripts/verif-faits.mjs:47-48` — `en calcule l'exemple, c'est la revue de la PR qui le vérifie` — la porte vérifie
  qu'un test nomme le critère, non qu'il en calcule l'exemple (contraintes générales ; LE-6).
- `scripts/verif-faits.mjs:35` — `CLAUDE.md nomme les membres de l'espace de travail` — `CLAUDE.md` nomme chaque crate
  de l'espace de travail : `commandes` et `services` y entrent avec leur membre (tranche 2).
- `app/ui/tests/encre_statique.rs:349` — `racine().join("style.css")` — I17 et le contrôle des couleurs ne lisent que
  `style.css`, et I18 `style.css`, `index.html` et `src/` (`:323-331`) : la feuille d'un écran neuf y entre, ou les
  tests la lisent aussi (contraintes générales).
- `app/ui/src/vue_table.rs:65-72` — `pub(crate) fn style_du_recouvrement(r: Rect) -> String` — la place d'une case se
  calcule en une chaîne de quatre propriétés, posée dans l'attribut `style` (`:162`) : la tranche 3 la pose propriété
  par propriété.
- `app/ui/src/vue_table.rs:95-96` — `pub(crate) fn style_de_la_palette(r: Rect) -> String` — de même pour le bouton
  « Palette » (`:176`) (tranche 3).
- `app/ui/src/panneau_palette.rs:47` — `style=format!("width: calc({largeur} * var(--mm))")` — la largeur de la fiche,
  le troisième style calculé, dans l'attribut `style` (tranche 3).
- `app/ui/src/vue_table.rs:321` — `fn le_recouvrement_se_place_en_pour_cent_de_la_page()` — le test compare la chaîne
  entière du style : il se réécrit (tranche 3).
- `app/ui/src/vue_table.rs:335` — `fn palette_se_pose_au_bas_de_la_case_touchee()` — le test compare la chaîne de
  `style_de_la_palette` (`:347-350`) : il se réécrit (tranche 3).
- `app/ui/src/vue_table.rs:126` — `inner_html=svg` — la page du noyau entre par `inner_html`, comme la fiche
  (`app/ui/src/panneau_palette.rs:48`) : ce sont les SVG du noyau qu'admet le contrôle statique de l'interface
  (tranche 3).
- `app/src-tauri/tauri.conf.json:17` — `"csp": null` — l'établi n'a aucune CSP (AM-3) : la tranche 3 pose celle du
  § 40 · 7.5, et la revue finale cherche le mot périmé (tranche 8).
- `crates/stockage/src/racine_des_donnees.rs:66-67` — `TryLockError::WouldBlock => Erreur::RacineDesDonneesTenue` —
  `RacineDesDonnees::ouvrir` rend `Erreur::RacineDesDonneesTenue` quand un autre processus tient le verrou, après
  une ouverture de `verrou` qui ne tronque ni n'écrit rien (`:60-64`), et avant de créer `documents/` et
  `captations/` (`:70-73`) : un second processus devant une racine des données tenue s'arrête sans rien écrire
  (tranche 3).
- `crates/stockage/src/racine_des_donnees.rs:21-23` — `_verrou: File` — le verrou est le fichier `verrou`, tenu
  ouvert : il se lève quand ce fichier se ferme, avec la racine des données ; un `Document` et les `Captations`
  l'empruntent (`crates/stockage/src/documents.rs:63-64`, `crates/stockage/src/captations.rs:21-22`), et se ferment
  donc avant elle : la reprise ferme chaque fichier tenu, le verrou en dernier (vigilance 5 ; tranche 6).
- `crates/stockage/src/captations.rs:47` — `pub fn verser(&self, octets: &[u8])` — une captation se verse depuis des
  octets tenus en mémoire (vigilance 3 ; tranches 5 et 6).
- `crates/stockage/src/captations.rs:48-57` — `voisin(&self.dossier().join(&e))` — son nom provisoire se tire de
  l'empreinte, calculée avant toute écriture (vigilance 3).
- `crates/stockage/src/documents.rs:944-947` — `let mut avec = relu.fork();` — la compaction charge chaque fichier
  d'ajout dans un `fork()` du fichier complet relu (`charger_contre`, appelé par `lu_contre`, `:931-932`) : le coût
  que la tranche 5 mesure sur cent ajouts (mineur d'H07).
- `crates/synchro/src/lib.rs:3` — `lit aussi les conflits et les relève` — le relevé des conflits attend le lot 3 (Ce
  qui est reporté).
- `crates/synchro/src/lib.rs:4` — `transport : le réseau se branche dessus` — `synchro` n'a aucun transport, reporté
  au lot 3 (Ce qui est reporté).
- `crates/synchro/src/lib.rs:40` — `pub fn prochain_message(&mut self, doc: &Automerge)` — l'échange de deux copies
  par le protocole d'Automerge, sans réseau, sur lequel s'appuient les preuves à deux et à trois répliques (I10, test
  3 ; tranche 2).
- `scripts/modele/commits.py:35` — `LONGUEUR_MAX = 100` — l'en-tête d'un commit a 100 caractères au plus (le déroulé,
  point 3).
- `.github/workflows/modele-texte-pr.yml:45` — `commits.py --titre` — `texte-pr` vérifie le titre de la PR à la même
  limite de 100 : rien ne garde le titre de 93 au plus avant la fusion, et, le squash ajoutant « (#NN) », un titre de
  plus de 94 caractères (93 dès la PR #100) donnerait un en-tête de plus de 100, que le socle refuse au push sur `main`
  (`.github/workflows/modele-socle.yml:60`, `commits.py --plage`) (le déroulé, point 3).
- `app/src-tauri/tauri.conf.json:4` — `"version": "0.0.0"` — la version de l'application, que Tauri sert par
  `package_info()` et que l'installateur montre : la clôture y écrit `0.3.0`, seule (tranche 9 ; tranche 6, le
  `LISEZMOI.md`).
- `Cargo.toml:7` — `version = "0.0.0"` — chaque crate hérite de la version `0.0.0` (`version.workspace = true`), que
  la clôture garde : `CARGO_PKG_VERSION` y reste, et ne dit pas la version de Pupitre (§ 40 · 8 ; tranches 6 et 9).

## Outils et cas limites

- `cargo tree` — sans `--target`, seules les dépendances de l'hôte paraissent ; une erreur ne laisse rien sur la
  sortie, qu'un `grep` lirait comme une absence ; la première ligne est le paquet racine, qu'un motif qui le nomme
  trouve toujours — tranche 2 (tâche 17) : l'étape « Le graphe de l'interface », en natif puis avec
  `--target wasm32-unknown-unknown`, la sortie capturée sous `bash -e`, et son essai sur `stockage`, qui doit trouver
  `^automerge v` et `^services v` (décision 7)
- `cargo tree` — il montre par défaut les dépendances de test (`--edges normal,build,dev`), et une fonction activée
  n'y paraît que par `--edges features`, ou par `--format` avec `{f}` — tranche 3 : l'étape `cargo tree` d'`etabli`,
  sans les dépendances de test et avec les fonctions, et son essai : `pannes` activée par une dépendance ordinaire la
  fait rougir, par une dépendance de test non (décision 7)
- les fonctions de `tauri`, lues dans le `Cargo.toml` d'`etabli` — `protocol-asset` peut s'activer par une autre
  dépendance, que ce fichier ne nomme pas — tranche 3 : le test d'I31 qui relit le `Cargo.toml`, et l'étape
  `cargo tree` d'`etabli`, avec les fonctions, qui ne doit pas y trouver `protocol-asset`
- `SHGetKnownFolderPath`, par où Tauri (`dirs`) lit le dossier de l'application — `APPDATA` et `LOCALAPPDATA`
  redirigées ne le déplacent pas ; sous `USERPROFILE` redirigé, il échoue si la sentinelle n'a pas ses sous-dossiers
  `AppData\Roaming` et `AppData\Local` — tranche 3 : le garde-fou d'I30, étape de `windows`, qui vérifie d'abord que
  le dossier que résout Tauri tombe dans la sentinelle, sinon échoue (vigilance 2 ; § 70 · 3)
- `cargo` et `rustup` sous `USERPROFILE` redirigé — sans `CARGO_HOME` ni `RUSTUP_HOME`, ils tirent leurs dossiers de
  `USERPROFILE`, et chercheraient la chaîne dans la sentinelle — tranche 3 : l'étape de la sentinelle, `CARGO_HOME` et
  `RUSTUP_HOME` posés à leurs vrais dossiers, qui trouve la chaîne et lance les tests (vigilance 2)
- `git fetch --depth=1` — une histoire d'un seul commit n'a pas de base commune :
  `git diff "origin/$BASE_REF...HEAD"`, à trois points comme dans `mutants`, échoue — tranche 9 : l'étape du gel
  compare `origin/$BASE_REF` à la tête, à deux points, et la PR d'essai qui n'ajoute qu'une ligne passe (décision 16)
- la comparaison à deux points de `origin/$BASE_REF` et de la tête, sur une PR — la tête est le commit de fusion
  d'essai de l'événement, et `git fetch` prend la pointe actuelle de la base : une ligne qu'une autre PR, fusionnée
  entre-temps, a ajoutée au fichier des références paraît retirée, et le gel refuse à tort ; une relance rejoue le
  même commit de fusion, et le même refus — tranche 9 : son plan choisit entre la comparaison au premier parent du
  commit de fusion (`HEAD^1`, avec `fetch-depth: 2`) et la limite dite, levée par un nouvel envoi sur la branche ;
  l'essai : sur une branche jetable, une ligne ajoutée à la base après l'ouverture de la PR d'essai, puis la relance
  de son `verifier`
- `git diff` sur le fichier des références — une ligne ajoutée ne donne qu'un `+`, une ligne changée un `-` et un
  `+` ; rustfmt, qui recoupe une liste quand une ligne s'y ajoute, change aussi une ligne voisine — tranche 9 : sur
  une branche jetable, une PR d'essai qui change une référence du gel, refusée, et une qui n'ajoute qu'une ligne,
  admise (critères de sortie)
- `github.base_ref` — vide hors d'une PR : `verifier` tourne aussi à l'envoi sur `main` et à la demande, où
  `origin/$BASE_REF` ne nomme aucune branche — tranche 9 : l'étape du gel, sur une PR seulement, comme `mutants`
  (`if: github.event_name == 'pull_request'`), et le passage vert de `verifier` à l'envoi de la fusion
- `sha256sum -c` — sans `--strict`, une ligne mal formée n'est qu'un avertissement ; un fichier de l'archive absent
  d'`empreintes.txt` n'est jamais relu — tranche 6 : l'étape de `verifier` qui relit une archive de test, avec
  `--strict`, et le test du noyau sur la forme d'`empreintes.txt`, une ligne par fichier écrit
- `Get-FileHash`, dans PowerShell, le shell par défaut de `windows` — il rend l'empreinte en majuscules, quand
  `empreintes.txt` l'écrit en minuscules, au format de `sha256sum` : une comparaison sensible à la casse (`-ceq`,
  `.Equals`) refuse tout ; son chemin (`Path`) est entier, à barres obliques inverses, quand l'archive écrit chaque
  chemin relatif, séparé par `/` — tranche 6 : l'étape de `windows` qui suit les lignes du `LISEZMOI.md` sur une
  archive de test, admise intacte, ce qui éprouve la casse et les chemins, et refusée altérée d'un octet
- `GetDiskFreeSpaceExW`, déclaré à la main — un dossier absent échoue, au lieu de rendre 0 ; un chemin de plus de 260
  caractères demande le préfixe `\\?\`, que `std::fs` pose de lui-même et qu'un appel déclaré à la main ne pose pas —
  tranche 5 : un test de `systeme`, dans `windows`, sur la place d'un dossier jetable, positive, puis d'un dossier
  jetable au chemin de plus de 260 caractères, positive aussi, et d'un dossier absent, une erreur dite, jamais 0 ;
  tranche 6, où l'utilisateur choisit la destination de l'archive par la boîte de dialogue : un test de `systeme`,
  dans `windows`, sur la place d'une destination d'archive au chemin de plus de 260 caractères, positive, et d'une
  destination absente, une erreur dite. Seule la place offerte à l'appelant (`lpFreeBytesAvailableToCaller`, quotas
  compris) se lit : aucun job n'a de quota, limite que la PR de la tranche 5 dit. Une destination sur un partage
  réseau, dont le nom UNC doit finir, d'après la documentation de Microsoft (non éprouvé ici), par une barre oblique
  inverse : le plan de la tranche 6 dit ce qu'il en fait (décision 17)
- les droits de Tauri et `generate_handler!` — sans manifeste de l'application dans `build.rs`, toute commande de
  `generate_handler!` est permise ; avec lui, une commande sans droit n'est refusée qu'à l'appel, à l'exécution —
  tranche 3 : le test d'I31, qui compare la liste de `commandes` à `generate_handler!`, au manifeste de `build.rs` et
  aux droits, et l'essai installé (décision 11)
- les fichiers de droits de Tauri — chaque fichier de `capabilities/` s'applique, faute de liste dans
  `app > security > capabilities` : un `default.json` resté en place garderait ses droits — tranche 3 : le test d'I31,
  qui relit chaque fichier de `capabilities/` et n'y admet que `windows.json` (décision 11)
- la CSP de Tauri — à la construction, Tauri ajoute à la CSP de `tauri.conf.json` les nonces et les empreintes de ce
  qu'il sert, dont le chargeur de Trunk : la CSP qu'applique la fenêtre n'est pas celle que relisent les tests —
  tranche 3 : l'essai installé de la maîtrise d'ouvrage, seul à voir une fenêtre blanche, à côté de la comparaison de
  la CSP au § 40 · 7.5 par `verif-faits.mjs` (décision 6)
- la CSP que `verif-faits.mjs` lit dans `tauri.conf.json` — Tauri y admet `csp` sous deux formes, une chaîne ou un
  objet de directives (`tauri-utils` 2.10.2, l'énumération `Csp`), et fusionne par-dessus un
  `tauri.windows.conf.json` s'il existe (JSON Merge Patch) : une comparaison de chaînes sur le seul
  `tauri.conf.json` passe à côté de l'objet comme du fichier de Windows — tranche 3 : un test de
  `scripts/verif-faits.test.mjs` qui plante une CSP en objet, puis un `tauri.windows.conf.json`, refusés tous deux
- `style-src 'self'` — il refuse un attribut `style`, posé par `setAttribute` ou par `cssText`, mais non une propriété
  posée seule (`style.setProperty`, la directive `style:` de Leptos) — tranche 3 : les styles d'INC-0 posés propriété
  par propriété (PR (a)), et le contrôle statique de l'interface, qui refuse un attribut `style` et `cssText`
- le protocole `captation` sous Windows — WebView2 le sert à `http://captation.localhost/`, non à
  `captation://localhost/` — tranche 3 : la comparaison de la CSP au bloc du § 40 · 7.5 ; tranche 5 : le test d'I31
  pour le protocole, dans `windows`
- `JSON.parse`, qui lit dans l'interface ce qu'écrit `serde_json` — un entier au-delà de 2⁵³ − 1 s'y arrondit, sans
  erreur — tranche 2 : `les_entiers_d_une_reponse_restent_sous_2_puissance_53` (tâche 3) et
  `verser_une_duree_au_dela_d_entier_sur_est_refusee` (tâche 10) (décision 11)
- `git ls-files`, par où la porte `contenu` lit les fichiers suivis — un fichier indexé de force (`git add -f`), que le
  `.gitignore` masque, reste suivi et listé ; sans `-z`, git met entre guillemets un nom à accent, et `xargs` coupe un
  nom à espace et s'arrête à une apostrophe (LE-13) — tranche 1 : l'essai de sa tâche 2, `essai.automerge` et
  `pupitre-archive-2026-11-09/LISEZMOI.md` indexés de force, refusés par le contrôle 6
  (`.claude/plans/lot1-tranche1-stockage-durci.md`, « Outils et cas limites »), et celui de LE-13, un nom à accent, à
  espace et à apostrophe suivi d'un autre nom, les deux portant un motif refusé
- `cargo-mutants` — un mutant qui boucle sans fin, pris par le délai (TIMEOUT), le fait sortir en 3, et `mutants`
  échoue — tranche 1 : `le_voisin_prend_le_premier_nom_libre` (tâche 9), sur une recherche bornée ; un mutant de
  boucle sans fin s'exclut nommément dans `.cargo/mutants.toml`, avec sa raison (décision 4)
- `cargo-mutants` — un module examiné dont un fichier de tests manque à `additional_cargo_test_args` laisse ses
  mutants survivre (`MISSED`) ; sur une PR, `--in-diff` ne mute que les lignes changées — la remesure à froid de
  chaque PR qui étend `.cargo/mutants.toml`, par le passage complet (`mutants-complet.yml`, à la demande), qui mute
  chaque module entier : tranche 2 (tâches 7 et 13), puis les tranches 3 à 6 (décision 4)
- `rustdoc` — sur la chaîne stable du dépôt (1.99.1, `rust-toolchain.toml`), un doctest `compile_fail` passe sur
  toute erreur de compilation : le numéro qu'il porte (`compile_fail,E0502`) n'est vérifié que sur nightly —
  tranche 1 : le doctest `compile_fail,E0502` de `Document::ecrire`, que garde son jumeau qui compile (`apres`,
  `crates/stockage/src/documents.rs:245-250`), le même code, l'appel déplacé après l'écriture ; tranche 2 : celui de
  `Document::appliquer` (tâche 6), gardé de même par son second doctest, qui prépare le message après `appliquer`
- la lecture d'une signature (`Read::read`) — elle peut rendre moins d'octets que demandé sans être à la fin ; un
  fichier peut être plus court que la signature ; `fs::read` lit tout — tranche 5 : le contrat de lecture qui compte
  les octets lus, sur un fichier ignoré lu de ses seuls premiers octets, un fichier plus court que la signature et une
  lecture qui ne rend qu'un octet à la fois (décision 19)
- les noms de fichier sous Windows (`std::fs`, `OsStr`) — NTFS garde un nom en Unicode composé et le même décomposé
  pour deux noms distincts ; un nom UTF-16 mal formé n'a pas de `&str` (`to_str` rend `None`, `to_string_lossy`
  l'altère) — tranche 5 : les tests de `windows` sur un chemin de plus de 260 caractères, les deux formes d'un même
  nom et un nom intraduisible, que le rapport compte (vigilance 4)
- les réponses partielles (`Range`) du protocole `captation` — `bytes=N-`, `bytes=-N`, plusieurs plages, une plage
  au-delà de la fin (416) — tranche 5 : les tests du gestionnaire, sur chaque forme, et une empreinte inconnue,
  refusée
- `pannes`, l'injection de la tranche 1 — elle ne coupe qu'aux étapes que nomme `Etape` (voisin écrit, `sync_all`,
  renommage, `rendre_durable`, retrait, `save`, relecture, réception), et un voisin s'écrit en un seul `write_all` ;
  une copie en flux écrit par tronçons, l'empreinte calculée en chemin — tranches 5 et 6 : I14 et le test 4 rejoués
  sur la copie en flux, une coupure après un tronçon écrit comprise, puis la reprise au démarrage
- `cargo test -- --ignored` — une étape dont aucun test ne passe le filtre sort en 0 (« 0 passed ») — tranche 2
  (tâche 18) : l'étape du fonds de mesure, dont la PR recopie la mesure imprimée (`--nocapture`) ; tranche 5 : le test
  13 entier, qui échoue au-delà de 4,5 Mo
- `f64::sin`, que le jeu factice n'appelle jamais, et les fonctions de `f64` qui passent comme lui par la
  bibliothèque mathématique de la plateforme (`cos`, `sin_cos`, `tan`, `powf`, `exp`) — leur résultat peut différer
  d'un bit entre Linux et Windows ; un test d'empreintes ne le voit que si les deux bibliothèques diffèrent sur les
  valeurs calculées, et ne dit rien d'une autre plateforme ni d'une autre chaîne — tranche 7 : un contrôle statique du
  module du jeu factice, qui refuse `sin(`, `cos(`, `sin_cos(`, `tan(`, `powf(` et `exp(`, en méthode comme en chemin
  (`x.sin()`, `f64::sin(x)`), avec une plantation de chaque forme, refusée ; à côté, le test de `stockage`, lancé par
  `verifier` et `windows`, qui compare les empreintes des captations du jeu factice à celles qu'il écrit, garde le
  résultat, aux mêmes octets, sur ces deux plateformes seulement
- `HashMap` et `HashSet` de std, sous leur hacheur par défaut (`RandomState`) — `new()`, `default()` et `collect()`
  tirent leur graine du système sans qu'aucun mot de l'aléa ne paraisse : l'ordre d'itération change d'un passage à
  l'autre, et une règle qui en dépendrait ne serait plus déterministe — tranche 2 (tâche 17) : les contrôles du noyau
  et de `services` refusent `HashMap`, `HashSet` et `RandomState` (`BTreeMap` et `BTreeSet` admis), et
  `chaque_controle_refuse_ce_qu_on_y_plante` y plante un `HashMap::new()`, refusé ; ils restent permis dans les tests,
  hors de `src/` (`crates/noyau/tests/guitare.rs:1` s'en sert), et dans les autres crates, où le contrôle d'I30
  (tranche 3) ne les cherche pas : limite que la PR de la tranche 2 dit
- la recherche en sous-chaîne des contrôles statiques de la tranche 2 (le noyau, `services`, I28) — un import groupé
  (`use std::{fs, path::Path};`) laisse `fs::read` sans `std::fs` ; un mot interdit cité dans un commentaire ou une
  chaîne la fait échouer ; `.insert(` est aussi celui d'un `BTreeSet` — tranche 2 (tâche 17) :
  `chaque_controle_refuse_ce_qu_on_y_plante`, qui y plante un import groupé, les chemins courts (`fs::`, `env::`…)
  cherchés aussi ; le contrôle d'I28 cherche `Transactable` et `automerge::transaction`, non `.insert(`
- `std::fs::read_dir`, par où chaque contrôle statique lit ses sources — un dossier vide, ou qui ne se lit pas,
  laisserait passer le contrôle sans rien lire — tranche 2 (tâche 17) : `assert!(!sources.is_empty(), …)` dans
  chaque contrôle, puis de même dans ceux des tranches 3 et 6
- `proptest` — un cas qui échoue ne se rejoue pas à l'identique : chaque ouverture tire l'acteur d'Automerge au
  hasard, hors de la graine ; sa réduction, sans borne utile par défaut, peut passer le délai du job — tranche 2
  (tâche 15) : `trois_repliques_convergent`, seize cas, la réduction bornée (`max_shrink_iters`, `max_shrink_time`),
  aucun fichier de régression suivi, le commentaire du test disant qu'un échec se rejoue sans garantie (I10, test 3)
- `get` d'Automerge — d'une clé que deux répliques ont changée, il ne présente qu'une valeur, et tait l'autre —
  tranche 2 (tâche 15) : `un_texte_change_des_deux_cotes_garde_ses_deux_valeurs`, qui lit les deux par `get_all`
- `put` d'Automerge — sur une clé en conflit, il écrit et ôte les autres valeurs ; la même valeur posée sur deux
  répliques en fait deux, qu'Automerge ne fusionne pas — tranche 2 (tâche 15) :
  `l_inverse_d_un_texte_en_conflit_rend_la_valeur_presentee`, `une_etiquette_posee_des_deux_cotes_n_en_fait_qu_une`
  et `une_prise_nee_sur_deux_repliques_ne_cache_aucun_de_ses_faits`
- `commit_with` d'Automerge — une transaction dont aucun changement ne change rien rend `None`, sans modification ni
  message, et `Document::ecrire` rend alors `Ok(None)`, le message perdu
  (`crates/stockage/src/documents.rs:264-267`) — tranche 2 (tâche 4) : `une_operation_sans_effet_est_inapplicable`
- le contrôle statique de l'aléa, une recherche en sous-chaîne — Automerge tire aussi un acteur au hasard sans que
  `random` paraisse : `Automerge::new()`, `fork()`, `fork_at` et `load` (automerge 0.12.0) ; le stockage les appelle
  (`crates/stockage/src/documents.rs:696`, `:766`, `:854` et `:1046`, d'où naît le fonds ; `:945` ; `:507` ; `:307`,
  `:758`, `:762` ; `crates/stockage/src/demarrage.rs:103`), et `services` lira des octets par `Automerge::load` dès
  la tranche 2 — tranche 3 : le contrôle d'I30, qui range ces appels avec `ActorId::random()` dans l'exception de
  l'acteur d'Automerge, et sa plantation d'un `Automerge::new()` et d'un `.fork()` dans un module hors de
  l'exception, refusée (décision 10)
- le contrôle statique d'I30, « ni chemin deviné » — le module de tests d'une crate peut vivre dans `src/` : celui
  d'`etabli` prend son dossier jetable par `std::env::temp_dir()` (`app/src-tauri/src/lib.rs:170`) ; un import
  groupé (`use std::{env, path::Path};`) laisse `env::temp_dir()` sans `std::env` — tranche 3 : le contrôle, qui
  n'admet que la fonction de dossier jetable de chaque crate et cherche aussi les chemins courts (`env::`), et ses
  plantations d'un second `temp_dir()` dans le code et d'un import groupé, refusées
- le contrôle statique de l'interface — un stockage du navigateur s'atteint aussi par `Reflect::get` d'un nom écrit en
  chaîne, comme l'interface atteint `__TAURI__` (`app/ui/src/export.rs:52`) ; un `inner_html` ne dit pas d'où vient
  son texte — tranche 3 : le contrôle, qui cherche le nom en chaîne comme l'appel de `web_sys`, et n'admet
  `inner_html` qu'aux deux lieux du SVG du noyau, avec une plantation de chaque forme
- `app.package_info().version` et `CARGO_PKG_VERSION` — avant la clôture, l'un et l'autre valent `0.0.0` : un test ne
  distingue pas lequel le `LISEZMOI.md` a lu — tranche 6 : le test du `LISEZMOI.md`, qui reçoit la version en
  argument, une valeur que ni l'un ni l'autre ne porte, et le contrôle statique qui refuse `CARGO_PKG_VERSION`
- `verif-faits.mjs`, point 8 — un critère livré que la PR n'inscrit pas au § 70 · 4.1 n'est pas lu, ni le calcul de
  son exemple — aucun test ne le peut, limite de la porte que la PR dit : la revue de chaque PR compare le tableau aux
  critères que la tranche dit livrer, et lit le calcul de chaque exemple (contraintes générales)
- `verif-faits.mjs`, point 8 (`fonctionsNommeesCritere`) — un `#[test]` sous `#[cfg(windows)]`, ou dans une crate
  qu'aucun job ne teste encore, compte comme un test qui tourne ; seuls `#[ignore]` et un `#[cfg_attr(…, ignore)]`
  l'écartent, ce dernier même quand le test tourne dans `verifier` — le test « fonctionsNommeesCritere relève les
  fonctions nommées d'après un critère, et leurs attributs » (`scripts/verif-faits.test.mjs`), qui lit
  `cap06_15_ignore_sous_windows`, sous `#[cfg_attr(windows, ignore = …)]`, comme écarté ; pour `#[cfg(windows)]` et
  la crate sans job, aucun test, limite que la PR dit : la revue de chaque PR lit dans quel job tourne le test de
  chaque critère inscrit, contre le job que nomme le critère de sortie
- `git grep -n -i -w`, la recherche de LE-2 — `-i` ne replie pas les accents, et `-w` prend une lettre accentuée pour
  une borne de mot (« tri » trouvé dans « trié ») — aucun test ne le peut, limite que la PR dit : la revue de chaque
  PR qui ajoute un mot au glossaire relit la recherche citée, faite aussi sans accent et sur la majuscule accentuée

## Revues

- la revue de chaque plan de tranche, par une session neuve : Tranche 1, Tranche 2, Tranche 3, Tranche 4, Tranche 5, Tranche 6, Tranche 7, Tranche 8
- la revue de chaque tâche, dans le plan de sa tranche : Tranche 1, Tranche 2, Tranche 3, Tranche 4, Tranche 5, Tranche 6, Tranche 7, Tranche 8
- la revue finale du lot et la relecture de ses corrections : Tranche 8

## Les tranches

### Tranche 1 · le stockage durci

**But :** lever les simplifications d'H07 avant toute fonction : le stockage tient une panne à chaque étape, s'ouvre
en lecture seule au lieu de refuser, reprend ce qu'un arrêt a laissé, et ses règles décident au noyau.

**Elle livre :**
- Les aides de tests recopiées que relève la carte du code (`NAISSANCE`, `HEURE`, `MESSAGE_DU_GESTE`,
  `ouvrir_le_fonds`, `faire_naitre`, `dossier_du_fonds`, `dossier_des_ajouts`, `lire_le_complet`, `poids`, `rangee`,
  `objet`, `copie`, `reunir`), dans le `tests/commun` de leur crate ; une seule fonction de dossier jetable par
  crate, `etabli` compris (I30).
- Le point d'injection `pannes`, sur chaque étape : voisin écrit, `sync_all`, renommage, `rendre_durable`, `save`,
  relecture, réception (LE-10 ; tests 4, 8 et 9).
- La garde d'un seul `Document` du fonds (AM-4 (1)) ; la relecture du disque, avec un acteur neuf (AM-4 (2)) ; la
  lecture seule et les avertissements, un fichier complet illisible nommé (décision 12 ; AM-4 (3), (5) et (6)).
- L'épreuve de la décision 8 d'H07 (une compaction qui échoue après une écriture ; une réception refusée en chemin,
  qui écrit ce qui a changé les heads), et l'échec de `rendre_durable` après le renommage, qu'aucun test d'H07 ne
  provoque.
- Le démarrage (AM-4 (4)) : le voisin conforme d'un fichier d'ajout mis en place ; le voisin du fichier complet lu
  avec le document, même à la naissance, puis fondu (`a_l_ouverture(_, true)`) ; les autres `.partiel` de
  `documents/` retirés ; les noms provisoires de `captations/` repris (décision 12) ; le suffixe `-2`, `-3` de
  `voisin` et d'`ecrire_a_cote`.
- La version de schéma : la règle au noyau ; les migrations, éprouvées par une migration de test engendrée à
  l'exécution depuis la genèse courante, de façon déterministe (acteur fixe, heure 0), avec la table du code qui
  dira la version de Pupitre de chacune ; la lecture sans écriture ; une copie de version de schéma N + 1 qui reçoit
  sans rien transmettre, par `synchro`.
- La teneur au noyau, et le test 11 sur la genèse d'aujourd'hui et un document témoin engendré à l'exécution ; la
  règle de la lignée au noyau, sur des lignées que le test écrit.
- Le fichier reçu comparé à `save_after` des heads d'avant (mineur d'H07) ; un doctest `compile_fail` fige qu'aucun
  message ne se prépare pendant une écriture ; chaque variante d'`Erreur` jamais provoquée (`Genese`, `ChampAbsent`,
  `Synchro`) a son test, ou disparaît.
- La porte `contenu` refuse tout fichier `*.automerge` et tout chemin sous `pupitre-archive-*` suivis, sans
  exemption (§ 40 · 9) ; ce plan ajoute aussi les deux motifs au `.gitignore`.

**Elle dépend de** ce plan, et reprend le code d'H07 et ses 39 tests, réécrits quand un comportement change.
**Critères de sortie :** les tests 4 et 8 à 11 du § 31 · 10 entiers, verts dans `verifier` et `windows` ; `mutants`
sans survivant, délais remesurés ; un `*.automerge` indexé à l'essai refusé par `scripts/verif-confidentialite.sh` ;
LE-10 et AM-4 clos au registre. **PR :** (a) les aides de tests et la porte `contenu` ; (b) l'injection, la garde,
la relecture, la lecture seule, les avertissements ; (c) le démarrage et le suffixe ; (d) la version de schéma, les
migrations, la teneur, la lignée, le test 11. **Documents :** § 31 · 2.1 et 6.1 ; `CONVENTIONS.md` (`contenu`,
`mutants`) ; `.cargo/mutants.toml` ; le registre.

### Tranche 2 · les services, les commandes, les opérations nommées

**But :** faire naître la seule porte d'écriture. Chaque geste du fonds devient une opération nommée, décidée au
noyau, conduite par `services`, appliquée par l'adaptateur `Documents` ; une même suite de tests des commandes passe
sur les adaptateurs en mémoire et sur ceux des fichiers, en natif.

**Elle livre :**
- `commandes` et `services`, que `stockage` réalise ; la suite des tests des commandes, générique sur la paire
  d'adaptateurs, jouée sur les deux paires, que chaque tranche étend (§ 40 · 9 ; ADR-022). `ci.yml` ajoute
  `-p commandes -p services` aux étapes Lint et Tests de `verifier` ; la suite sur la paire de fichiers vit dans
  `stockage`, que `windows` teste déjà.
- Le schéma de la `v0.3.0` et sa genèse, les références du test 11 à jour (décision 9) ; les identifiants
  (décision 10) ; le fonds qui naît au premier geste qui écrit, en un premier fichier complet, un geste refusé
  laissant la racine des données sans document (§ 31 · 2.3).
- Les gestes du fonds au noyau, préconditions, écritures et messages (CAP-01 ; CAP-02 ; § 30 · 8.1, 9 et 11) :
  noter, provenance `captee`, par l'établi, gardée à vie ; compléter le texte, une valeur `Str` atomique ; poser et
  ôter une étiquette ; relier et délier, par une clé qui porte le type et les deux bouts dans l'ordre de leurs
  octets, une idée reliée à elle-même refusée ; regrouper une prise entière, un MIDI versé seul ou un texte versé,
  l'idée laissée vide retirée par le même geste ; retirer et rétablir ; annuler (décision 18). Au noyau aussi :
  l'état « reliée », la durée d'une idée (`—` dès qu'une prise n'a pas la sienne), l'assiette d'un compte ; la règle
  des empreintes que nomme le fonds, et le cas du test 4, étendus aux MIDI versés seuls et aux textes versés.
- L'opération nommée, une modification atomique avec son message et son heure. La commande ne répond « faite »
  qu'une fois le fichier d'ajout en place : une panne injectée avant rend un refus, une panne injectée après rend
  « faite, avec un avertissement » (décision 12).
- Les preuves : I28, statique et sur l'historique ; I01, le test 7 ; ni `Counter` ni objet `Text` ; I10 et le test 3
  à trois répliques, par `synchro`. À deux répliques : un texte changé des deux côtés garde ses deux valeurs,
  intactes ; une étiquette, ou un lien posé dans les deux sens, n'en font qu'un ; deux regroupements d'une même
  prise laissent une seule idée visible.
- Un message en français, au vocabulaire du glossaire, qui dit ce qui est et jamais un ordre (§ 50 · 5), pour chaque
  erreur du stockage, de la synchronisation et des services ; les contrôles statiques du noyau (ni horloge, ni aléa,
  ni fichier) et de `services` (décision 13) ; l'étape `cargo tree` de l'interface.
- Une première mesure du poids : le générateur du fonds de mesure, dans `services`, sur 2 000 idées et 6 000 prises
  sans stems, par un test `#[ignore]` de `stockage`, lancé par `--ignored` dans une étape à part de `verifier`, hors
  de `windows` (test 13, que la tranche 5 complète).
- Ce que le plan de la tranche 1 lui laisse (`.claude/plans/lot1-tranche1-stockage-durci.md`, « Les écarts au plan
  maître », point 1) : la suite des commandes ouvre un fonds de version de schéma N + 1 et voit la commande refusée
  en le nommant, sur les deux paires ; le doctest « une synchronisation lancée pendant l'écriture ne voit pas
  l'opération en cours » se reprouve sur l'API qui applique les écritures du noyau ; le plan de la tranche dit qui
  conduit la migration.

**Elle dépend de** la tranche 1. **Critères de sortie :** la suite verte sur les deux paires dans `verifier`, sur la
paire de fichiers dans `windows` ; CAP-01 · 6, CAP-02 · 3, 4 et 6 à 8, et CAP-02 · 5 pour la boîte, y passent, sur
des prises que les tests engendrent ; I01, I10 et I28 verts dans `verifier` ; la première mesure dans la PR ;
`mutants` sans survivant, délais remesurés ; `documentation` vert, `CLAUDE.md` nommant les deux crates. **PR :** (a)
`commandes`, `services`, contrats, adaptateurs en mémoire, identifiants ; (b) le schéma, la genèse, les gestes,
l'annulation ; (c) les preuves, les messages d'erreur, la première mesure. **Documents :** § 40 · 3 ((a)) ; § 31 · 2
et `noyau::donnees::fonds` ; § 41 · 6 ((c)) ; `CLAUDE.md` (« Les crates », « Commandes ») ; `CONVENTIONS.md`
(`verifier`, `mutants`) ; `.cargo/mutants.toml` ; `ci.yml`.

### Tranche 3 · la boîte d'idées à l'écran

**But :** Sacha note et trie ses idées sur l'établi. `etabli` assemble le stockage et les services ; l'interface ne
voit que des vues, sous la CSP du § 40 · 7.5 et des droits qui nomment chaque commande.

**Elle livre :**
- D'abord, les trois styles calculés d'INC-0 (`style_du_recouvrement`, `style_de_la_palette`, la largeur de la
  fiche) posés propriété par propriété ; les tests `le_recouvrement_se_place_en_pour_cent_de_la_page` et
  `palette_se_pose_au_bas_de_la_case_touchee` se réécrivent.
- L'assemblage : `systeme` reçoit `app_local_data_dir`, jamais `app_data_dir`, ouvre `donnees/`, lit l'heure, tire
  l'aléa ; un second processus devant une racine des données tenue s'arrête en le disant, sans rien écrire
  (§ 40 · 8 ; § 31 · 1 et 2.1).
- I30 : le garde-fou de la sentinelle (`USERPROFILE`, `APPDATA` et `LOCALAPPDATA` redirigés, `CARGO_HOME` et
  `RUSTUP_HOME` à leurs vrais dossiers) ; le contrôle statique : hors de `systeme`, ni chemin deviné, ni horloge, ni
  aléa, sauf l'acteur d'Automerge (décision 10).
- La boîte et ses commandes : le nombre d'idées non reliées, avec son assiette, sans ancienneté ; le tri par date,
  étiquette, durée (une idée sans durée à part, jamais exclue), état et provenance ; le filtre par contenu, où
  « photo » répond « 0 idée », avec son assiette ; les idées retirées, à un geste ; une liste virtualisée (CAP-01 ;
  CAP-02 ; CAP-07 · 4). Le tri et le filtre sont des fonctions du noyau, sous mutation dès cette tranche, que la
  tranche 4 étend en requêtes.
- Noter en un geste, sans demande de titre ; `Ctrl+Z` et un bouton annulent ; un bandeau dit un refus, un
  avertissement, une lecture seule en nommant le fichier, une version de schéma plus récente ; aucune notification
  (I29) ; aucune idée « à envoyer » ; une valeur absente s'écrit `—`.
- La CSP exacte du § 40 · 7.5, protocole des captations compris, sans `dangerousDisableAssetCspModification`,
  `useHttpsScheme` ni `assetProtocol` ; le manifeste de l'application dans `build.rs` et les droits (décision 11),
  `exporter_cahier` permise par son nom.
- I31 : des tests d'`etabli` relisent `tauri.conf.json`, les droits et le `Cargo.toml` d'`etabli` (ni
  `assetProtocol`, ni fonction `protocol-asset`) ; `verif-faits.mjs` compare la CSP au bloc du § 40 · 7.5 ; un texte
  piégé (`<script>`, `<img onerror=…>`) revient tel quel ; un contrôle statique de l'interface refuse un
  `inner_html` hors du SVG du noyau, un attribut `style` ou `cssText`, et tout stockage du navigateur. L'étape
  `cargo tree` d'`etabli`.

**Elle dépend de** la tranche 2. **Critères de sortie :** CAP-01 · 4 et 6, CAP-02 · 1, 4 et 6 à 8, CAP-02 · 5 pour
la boîte et CAP-07 · 4, dans la suite des commandes (`verifier`, deux paires ; `windows`, fichiers) ou les tests du
modèle de vue (`verifier`), CAP-02 · 3 sur des prises de test ; I29 dans `verifier` et `windows` ; I30, son
garde-fou et son contrôle statique, dans `windows` ; I31, hors le protocole des captations (tranche 5) : l'interface
dans `verifier`, `etabli` dans `windows`, la CSP dans `documentation` ; `mutants` sans survivant, délais remesurés.
👤 Si la maîtrise d'ouvrage confirme la décision 6, l'essai installé note, trie, annule et exporte le cahier sans
fenêtre blanche ; le relevé va au § 70 · 7, et AM-3 se clôt. **PR :** (a) les styles en propriétés ; (b)
l'assemblage, `getrandom` et I30 ; (c) la boîte, la CSP, les droits et I31. **Documents :** § 41 · 6 ; § 40 · 7.5
(où vit la liste des commandes, (f) ; les droits du lot 1) ; § 70 · 3 et 8 ; `.cargo/mutants.toml` ;
`CONVENTIONS.md` (`verifier`, `windows`, `documentation`, `mutants`) ; `CLAUDE.md`.

### Tranche 4 · la recherche et la vue du fonds

**But :** Sacha retrouve une idée par ce qu'elle porte ou par son texte, enregistre une requête, et voit son fonds.

**Elle livre :**
- Les requêtes au noyau (CAP-06) : par date, étiquette, durée, état, provenance et contenu ; par texte, chaque mot
  entier ou en fragment, sans égard à la casse ni aux accents, par une table de repli (« eglise » trouve « Église »,
  « oe » trouve « œ »). Chaque réponse déclare son assiette ; une idée dont l'attribut manque, ou sans texte, se
  compte à part ; une idée retirée n'est jamais interrogée ; chercher n'écrit rien.
- L'index, en mémoire, côté natif, sans base : un dérivé jetable qui, reconstruit, rend les mêmes résultats sur un
  jeu de requêtes fixé (I13, test 1).
- Les requêtes enregistrées : un ULID, un nom, des critères sous une forme qui admet des sortes nouvelles sans
  migration ; les enregistrer, renommer, supprimer sont des opérations nommées, non des gestes de tri. I01 étendu :
  ni résultats, ni leur nombre.
- La vue du fonds, sans morceaux ni albums : le bandeau, avec la recherche et « Sans nœud · aucune archive » ; la
  boîte ; la page d'un fonds vide (noter) ; un résultat ouvre l'idée ; `/` porte le focus dans la recherche
  (CAP-07 ; § 50 · 2 et 3.1). Le journal du fonds en lecture seule, à un geste (décision 21, question 4).

**Elle dépend des** tranches 2 et 3. **Critères de sortie :** CAP-06 · 7 à 10 et la part « requêtes » de CAP-02 · 5
(CAP-06 · 8), I09, I13 et I01, dans `verifier` ; la paire de fichiers dans `windows` ; `mutants` sans survivant,
délais remesurés. **PR :** une ou deux. **Documents :** § 40 · 7.5 ; § 50 · 3.1, si le journal s'y ajoute ;
`.cargo/mutants.toml` ; `CONVENTIONS.md`.

### Tranche 5 · l'import

**But :** Sacha verse son fonds existant, prises, jeux de stems, MIDI et textes, avec l'origine déclarée qu'il
donne, et réécoute une prise.

**Elle livre :**
- Au noyau, « Les faits d'un fichier versé » et « Un import rejoué ne crée aucun doublon », tels que le § 70 · 1 les
  décrit (le manifeste d'une prise en stems à l'octet), et l'origine déclarée la plus forte, le premier nom vu, la
  durée d'une prise en stems.
- L'import en deux temps (ADR-024 ; décision 19) : inspecter et déclarer, dossier par dossier, sans rien écrire ;
  comparer la taille à la place libre, que `systeme` lit (`GetDiskFreeSpaceExW`, sous `#[cfg(windows)]`) et passe
  aux services, sous le contrôle statique d'I30 ; verser, en flux (vigilance 3). Dans un Pupitre neuf, l'import est
  le premier geste : sa première opération écrit la genèse, la naissance et elle-même en un premier fichier
  complet ; refusé, il laisse la racine des données sans document (§ 31 · 2.3). Chaque import a son ULID ; ses idées
  sont `importee`, par l'établi, depuis lui, sans regroupement ni lien ; son rapport se fige en captation ;
  interrompu, il reprend sous le même identifiant ; rejoué, il n'ajoute que son rapport. Le dossier se choisit par
  `tauri-plugin-dialog`, côté natif ; la page d'un fonds vide propose aussi d'importer.
- La règle des empreintes que nomme le fonds (décision 12), étendue aux stems, à leurs MIDI, au manifeste d'une
  prise en stems et au rapport d'import ; la reprise au démarrage connaît aussi le nom provisoire des copies en flux
  (vigilance 3).
- Les déclarations de Sacha, des opérations nommées : un jeu de stems ; une origine déclarée, à l'import ou après,
  de sa main ou d'un générateur externe nommé en texte libre ; le type d'un stem. L'origine déclarée se range par
  empreinte : la plus forte l'emporte, générée sous deux noms elle garde les deux, elle ne se retire ni ne se déduit
  jamais.
- Le protocole `captation` : un gestionnaire côté natif, qui ne sert qu'une empreinte connue, en lecture seule, avec
  le type que nomme le document, en réponses partielles (`Range`) bornées. La lecture simple, à sa vitesse, stem par
  stem ; sinon les faits et « lecture impossible sur cet appareil », sans erreur (§ 40 · 7.5 ; ADR-022).
- L'écran : déclarations, rapport, provenance (`⇣` sur un élément importé gardé ; aucun glyphe sur une saisie, ni
  sur une proposition en attente, en italique avec `~`) (§ 50 · 4.1 et 4.2) ; noms d'origine et textes versés écrits
  comme texte. I01 étendu ; les fichiers des tests engendrés à l'exécution.
- Le test 13 entier : le générateur de la tranche 2 complété (stems et leurs MIDI, noms d'origine, origines
  déclarées, 3 000 acteurs), le fichier complet sous 4,5 Mo, et le `fork()` d'`est_absorbe` mesuré sur une
  compaction de cent ajouts (mineur d'H07), dans l'étape à part de `verifier`, en `--release` si elle passe dix
  minutes. Passé 4,5 Mo, ou la compaction trop lente, le schéma des tranches 2 et 5 se reprend avant la tranche 6,
  sinon le budget se revoit (ADR-024).
- Son plan décide, et le dit : la granularité des opérations ; l'état d'un import en cours ; les signatures audio et
  les extensions de texte (ChordPro : `.cho`, `.crd`, `.chopro`, `.chordpro`, `.pro`) ; la « même durée » ; un MIDI
  au nom d'un WAV hors d'un jeu ; les sous-dossiers d'un jeu ; un texte hors UTF-8 ; les types de stem ; les octets
  du manifeste, référence du test 11 ; la portée d'une déclaration par préfixe de chemin.

**Elle dépend des** tranches 1 à 3, et la vue du fonds de la tranche 4 montre ce qu'elle verse. **Critères de
sortie :** CAP-03 · 1 à 3 et 7 à 13, et le 4 pour sa part sans morceaux (aucun lien, aucun regroupement), dans
`verifier` et `windows` ; I14 et le test 4, coupure à chaque étape et reprise au démarrage, rejoués sur la copie en
flux, dans `verifier` et `windows` ; I31 pour le protocole, dans `windows` ; I01 ; le test 13 dans son étape ;
`mutants` sans survivant, délais remesurés ; un chemin de plus de 260 caractères et des noms Unicode composés et
décomposés passent dans `windows`. **PR :** (a) les règles de l'import, au noyau ; (b) l'import, ses déclarations,
son rapport, le test 13 ; (c) le protocole des captations, la lecture simple, l'écran. **Documents :** CAP-03 et
SYN-07 ((d) et (e)) ; § 41 · 6 (les deux usages de la boîte de dialogue) ; § 40 · 7.5 ; `.cargo/mutants.toml` ;
`CONVENTIONS.md`.

### Tranche 6 · l'archive complète et la reprise

**But :** tant qu'il n'y a pas de nœud, l'archive complète sauvegarde le fonds, et une racine des données abîmée en
repart (§ 31 · 2.1 et 9.1 ; SYN-07 · 4 à 6).

**Elle livre :**
- `appareil.txt` : une ligne `clé=valeur` par fait, écrit à côté puis renommé, jamais échangé ni archivé ; au lot 1,
  la date de la dernière archive relue.
- L'archive du § 31 · 9.1, `pupitre-archive-AAAA-MM-JJ/` (`-2`, `-3`… si le nom est pris), sans miroir, ce que dit
  son `LISEZMOI.md`, qui porte la version de Pupitre, lue par `app.package_info().version` et passée en argument,
  jamais `CARGO_PKG_VERSION` (un contrôle statique le garde). Sa destination se choisit par `tauri-plugin-dialog` ;
  sa taille s'annonce face à la place libre du support, que `systeme` lit, et rien ne s'écrit sans la place ; chaque
  fichier est relu et chaque teneur comparée avant le renommage ; son rapport compte documents, captations et
  octets ; seule une archive relue écrit sa date.
- La réimportation, dans un Pupitre neuf seulement, même ouvert sans geste ou après un premier geste refusé :
  histoire et lignée restituées, teneur comparée avant toute migration, captations relues ; un document illisible se
  nomme ; un Pupitre qui a un fonds la refuse sans rien écrire.
- La reprise d'une racine des données abîmée, proposée par le bandeau de la lecture seule : chaque fichier tenu
  fermé (vigilance 5), le verrou en dernier ; la racine des données renommée `donnees-abimee-AAAA-MM-JJ/` par
  `systeme` ; la relance sur une racine des données neuve, puis la réimportation ; un renommage qui échoue ne change
  rien, et le bandeau dit pourquoi.
- Au noyau, sous mutation : l'archive face à la place libre ; le suffixe `-2`, `-3` de `pupitre-archive-…` et de
  `donnees-abimee-…` ; la réimportation admise dans un Pupitre neuf seulement ; la forme d'`appareil.txt` et
  d'`empreintes.txt`.
- Le bandeau « Sans nœud · archive du … » ; la page d'un Pupitre neuf propose la réimportation ; une étape relit une
  archive de test par `sha256sum -c` dans `verifier`, par les lignes du `LISEZMOI.md` dans `windows`.

**Elle dépend des** tranches 1 à 5. **Critères de sortie :** le test 14 entier : ses six cas sans reprise dans
`verifier` et `windows`, ses deux cas de reprise, que conduit `systeme`, dans `windows`, dont le fichier tenu par un
autre processus ; I14 et le test 4 rejoués sur la copie en flux de la réimportation ; SYN-07 · 4 à 6 ; aucune
dépendance nouvelle ; `mutants` sans survivant, délais remesurés. **PR :** (a) `appareil.txt`, l'archive et sa
réimportation ; (b) la reprise et les bandeaux. **Documents :** § 31 · 9.1 si la forme se précise ; § 40 · 7.5 ;
`.cargo/mutants.toml` ; `CONVENTIONS.md`.

### Tranche 7 · le mode démonstration et le fonds de mesure

**But :** montrer Pupitre sur un fonds inventé, dans son dossier, sans jamais atteindre le fonds, et l'ouvrir sur le
fonds de mesure (§ 31 · 1 ; M-0001).

**Elle livre :**
- La relance par argument (décision 15) : `demonstration/` à côté de `donnees/`, jamais dedans ; le bandeau
  « Démonstration · fonds inventé » porte « Quitter la démonstration ».
- Le jeu factice, du code : ses idées par les mêmes opérations nommées, aux textes, étiquettes et liens inventés,
  avec son propre identifiant de propriétaire, écrit à la première ouverture ; ses captations, des WAV PCM de
  quelques secondes, à l'en-tête écrit à la main, d'une onde carrée ou triangulaire ou de valeurs de sinus écrites
  dans le code (jamais `f64::sin`), aux mêmes octets partout ; aucune photo, aucune suivie, aucune dépendance.
- Réécrire le jeu factice et engendrer le fonds de mesure, par le générateur des tranches 2 et 5 : permis par les
  droits, refusés côté natif hors du mode démonstration.
- La durée d'une recherche, lue dans `systeme` (`Instant`) autour de la commande, affichée en mode démonstration
  seulement : la mesure se prend dans la construction installée de la revue finale, en `--release`, sur le fonds de
  mesure ; jamais une porte de CI, aucun cargo sur le poste. Le second test d'I30. Les délais de `verifier` et
  `windows` remesurés.

**Elle dépend des** tranches 2 à 5. **Critères de sortie :** le second test d'I30 vert dans `windows` ; un test de
`stockage`, que lancent `verifier` et `windows`, trouve aux captations du jeu factice les mêmes empreintes sous
Linux et sous Windows ; les délais dans la PR. **PR :** (a) la démonstration et le jeu factice ; (b) le fonds de
mesure en démonstration et la durée d'une recherche. **Documents :** § 31 · 1 ; § 40 · 7.5 ; `CONVENTIONS.md`
(délais).

### Tranche 8 · la revue finale

**But :** éprouver le lot dans l'application installée.

**Elle livre :**
- La PR de revue finale. 👤 La maîtrise d'ouvrage passe sur l'installateur les contrôles à l'œil que le § 70 · 7
  donne à l'établi au lot 1 : chaque écran s'ouvre et sert, CSP et droits posés (un texte piégé tel quel, un refus
  en bandeau, des prises importées regroupées) ; une construction installée par-dessus une autre, sous
  `app.pupitre.etabli`, sur un fonds d'essai, sans rien perdre : deux archives complètes, exportées avant et après,
  ont les mêmes lignes `captations/` dans `empreintes.txt` et le même `teneurs.txt`, l'export relisant chaque
  captation, et le journal du fonds est le même, par le moyen de la décision 21 (question 4) ; l'ouverture en
  démonstration sur le fonds de mesure, chronométrée, qui fixe sa cible ; une idée désignée retrouvée en dix
  secondes au plus, la recherche sous la seconde. Chaque passage se consigne au tableau des relevés, avec la façon
  de vérifier « le même journal » ; AM-3 se clôt s'il ne l'est pas. La revue vérifie qu'il ne reste ni mot périmé
  par le lot (`"csp": null`, `default.json`, « à relire »), ni second sens d'un mot ajouté au glossaire (LE-2,
  LE-3) ; le § 70 · 8 et `CONVENTIONS.md` disent ce que chaque porte refuse.

**Elle dépend des** tranches 1 à 7. **Critères de sortie :** chaque passage à l'œil consigné au tableau des relevés
du § 70 · 7, avec la façon de vérifier « le même journal » ; AM-3 clos ; aucun mot périmé par le lot, ni second sens
d'un mot ajouté au glossaire ; les corrections de la revue relues ; `verif-liens.mjs` et `verif-faits.mjs` verts.
**Documents :** § 70 · 7 et 8 ; `CONVENTIONS.md`.

### Tranche 9 · la clôture

**But :** clore le lot avant la première donnée réelle.

**Elle livre :**
- La PR de clôture : `app.pupitre` et la version `0.3.0`, dans `tauri.conf.json` seul, les `Cargo.toml` gardant
  `0.0.0` (§ 40 · 8), `app.pupitre.etabli` cherché dans tout le dépôt ; l'étape du gel (décision 16) ; le § 40 · 9
  nomme `verifier` et `windows` pour H07 (mineur d'H07) ; ce plan, les plans de tranche et celui d'H07 retirés, une
  fois chaque ligne de « Ce qui doit survivre », ici et dans le plan d'H07, écrite ailleurs ; `CLAUDE.md` et le
  README, relu d'après `docs/modele/documentation.md`. 👤 Avant sa fusion, la maîtrise d'ouvrage installe
  l'installateur qu'elle dépose : il s'ouvre sur un Pupitre neuf, la racine des données née sous
  `%LOCALAPPDATA%\app.pupitre\donnees`, sans document ; Windows montre la version 0.3.0 ; une idée se note sans
  fenêtre blanche, en mode démonstration seulement, pour que `donnees/` reste neuf ; les essais de
  `app.pupitre.etabli` restent intacts. Le relevé va au § 70 · 7, dans cette PR.
- 👤 L'étiquette `v0.3.0`, par la maîtrise d'ouvrage, après la fusion de la clôture. Avant elle, aucune construction
  ne reçoit de donnée réelle.

**Elle dépend des** tranches 1 à 8. **Critères de sortie :** l'issue se clôt avec la PR de clôture ; sur une branche
jetable, une PR d'essai qui change une référence du gel est refusée, et une qui n'ajoute qu'une ligne passe ;
`verif-liens.mjs` et `verif-faits.mjs` verts. **Documents :** § 70 · 7 (le relevé de l'installation) ; § 31 · 10 (le
test 11, le gel) ; § 70 · 8 (la porte du gel) ; § 40 · 9 ; `CONVENTIONS.md` (l'étape du gel) ; `CLAUDE.md` ; README.

---

## AM-4, point par point

| Point | Ce que l'essai simplifie | Levé par | Preuve |
|---|---|---|---|
| (1) | Deux `Document` du fonds peuvent vivre dans un processus | Tranche 1 | Un second `fonds()` refusé, ou un doctest `compile_fail` |
| (2) | Un document à relire le reste jusqu'à sa réouverture | Tranche 1 | Test 8 : relu du disque, acteur neuf, l'ajout suivant ne porte que sa modification |
| (3) | Un ajout altéré, ou une compaction qui échoue, fait refuser l'ouverture | Tranche 1 | Test 9 : lecture seule ; la compaction d'ouverture infidèle ouvre avec un avertissement (décision 12) |
| (4) | Ni reprise des `.partiel`, ni suffixe du voisin, ni lignée remplacée | Tranche 1 ; la copie remplacée, au lot 4, par ce report écrit (§ 70 · 1) | Tests 4, 8 et 9 ; la règle de la lignée au noyau |
| (5) | `ecrire` rend `CompactionInfidele` quand l'écriture est faite | Tranche 1 | Par injection : « faite, avec un avertissement » |
| (6) | Un fichier complet illisible ne nomme pas son chemin ; une erreur du disque à la compaction d'ouverture fait échouer l'ouverture | Tranche 1 | Un fichier complet altéré se nomme ; par injection, l'ouverture réussit |
| (7) | Sous Windows, le verrou peut refuser la copie d'une racine des données ouverte | Reporté au lot 4 (choix 3) | — |

Le reste d'H07 : **LE-10** et **la décision 8 de son plan**, en tranche 1 ; **les trois mineurs de sa revue
finale** : le fichier reçu en tranche 1, le `fork()` d'`est_absorbe` en tranche 5, la phrase du § 40 · 9 en
tranche 9 ; **les aides de tests recopiées**, en tranche 1.

---

## Ce qui est reporté

| Élément | Où il va | Pourquoi |
|---|---|---|
| La compilation de `services` en WebAssembly, la construction navigateur, ses parcours de bout en bout, l'audit axe-core | La ligne 5 des portes | § 40 · 9 ; décision 13 |
| Le transport de `synchro` ; le relevé des conflits (I11, la part « à trancher » du test 3, SYN-03) et la place de l'état « à trancher », en valeurs simples dans l'objet ou par la première migration publiée ; la réparation par le nœud d'un document illisible ; l'annulation refusée après une synchronisation ; l'éviction (test 5, I15), le budget de stockage, la file d'envoi | Lot 3 | Ni synchronisation dans l'application, ni éviction hors de `demonstration/` au lot 1 ; décisions 9 et 12 |
| Le signal « document changé » | Lot 2 ou 3 | Décision 11 |
| La copie remplacée dans `documents/remplaces/`, la lignée d'un document reconstruit, le test 12, I12, le miroir, la restauration (tests 2 et 6) | Lot 4 | § 70 · 1 ; le lot 1 écrit la règle, et la teneur exclut la lignée |
| AM-4 (7), la sauvegarde à chaud ; une sauvegarde en échec au bandeau (I29) | Lot 4 | Choix 3 ; la sauvegarde n'existe qu'alors |
| La photo, dont le conteneur naît dans la genèse ; le carnet, sa capture audio et photo hors ligne, ses commandes, ses droits, son code de version ; CAP-06 · 5, et le 6 sur le téléphone de référence (sur l'établi, tranches 7 et 8) | Lot 2 | § 40 · 9 |
| La capture audio et photo sur l'établi | Après INC-1, à l'incrément que la feuille de route nommera | § 40 · 9, « Il ne contient pas » : l'établi ne note qu'en texte |
| Le journal du fonds à l'écran, hors ce que retient la décision 21 (question 4) ; la version de Pupitre d'une migration, lue dans une table du code | La première vue du journal ; la première migration publiée | CAP-02 · 4 et 6 se lisent dans l'historique ; aucune migration n'est publiée au lot 1 |
| Les opérations sur deux documents, la réparation d'une référence au démarrage | INC-2 | Le fonds est le seul document ; la reprise des voisins ne modifie aucun document |
| Promouvoir, relier à un motif ou à un morceau, rattacher (CAP-04), rapprocher (CAP-05), les zones Morceaux et Albums, CAP-07 · 1 à 3, les requêtes sur les morceaux, les grilles, les mélodies, les lignes, la provenance `suggeree` | INC-2 à INC-6 ; INC-4 pour les analyses, INC-5 pour les albums | Ces objets n'existent pas |
| Une mélodie MIDI en proposition, les lignes proposées d'un texte versé, les analyses importées, la lecture au tempo du morceau ; la vue de provenance et la part d'origine (PRO-02, dont le critère 6) | INC-2 | ADR-024, décisions 4 et 8 : une proposition demande un morceau ; PRO-02 · 6 compte des lignes acceptées d'un texte versé |
| L'écoute de contrôle, les stems joués ensemble (ECO-01 · 8) | INC-3 | Q16 |
| « Non décodable » et la durée des formats compressés, lus par le nœud | Lot 3 au plus tôt | ADR-024, décision 6 |
| L'analyse légère après une capture ; les propositions ; les demandes de pistes | INC-4 ; INC-6 | § 40 · 3 et 9 |
| La suppression définitive d'une idée, sa confirmation renforcée, le refus selon le rôle | Plus tard ; INC-2 pour le lutrin | Le lot 1 n'a aucune opération destructrice |
| Le document de réglages, l'épinglage ; les autres exports et imports de SYN-07 ; SYN-01, SYN-02 hors du test 3, SYN-04 à SYN-06, SYN-08, SYN-09 | INC-2 au plus tôt ; lots 2 à 4 | § 40 · 9 |

---

## Points de vigilance

Cinq cas que la conception implique et qu'aucune tranche n'éprouverait par son seul chemin nominal ; chacun a son
test, dans la tranche qui possède le code.

1. **Un objet nommé par une empreinte, né sur deux répliques**, quand le même fichier est importé sur deux
   appareils : une prise, l'entrée d'un stem, l'origine déclarée ou les noms d'origine d'un contenu. Un conteneur
   créé deux fois sous une clé cache l'autre (§ 31 · 2.3), et une origine déclarée en valeur simple se perdrait au
   hasard d'Automerge : leurs faits s'écrivent en valeurs simples, sous des clés `<empreinte>/<…>` d'un conteneur de
   la racine du document, ou en ensembles dont le noyau tire la valeur. Tranches 2 et 5, à deux répliques.
2. **Une redirection qui ne porte pas.** Tauri lit le dossier de l'application par `SHGetKnownFolderPath`, et
   `cargo` et `rustup` tirent les leurs de `USERPROFILE` : le garde-fou d'I30 vérifie d'abord que le dossier que
   résout Tauri tombe dans la sentinelle, sinon il échoue, et la tranche 3 passe à l'autre voie du § 70 · 3.
3. **Un gros fichier.** `Captations::verser(&[u8])` tient tout en mémoire et tire son nom provisoire de
   l'empreinte ; un jeu de stems pèse quelque 400 Mo, le fonds 40 Go : l'import, l'archive et la réimportation
   copient en flux, l'empreinte calculée en chemin, sous un nom provisoire qui n'en dépend pas et que la reprise au
   démarrage connaît. Tranches 5 et 6.
4. **Un nom que Windows seul connaît** (plus de 260 caractères, Unicode composé ou décomposé, intraduisible en
   UTF-8) : le nom d'origine se garde exactement, sans refus ni altération silencieuse, et le rapport compte ce
   qu'il n'a pu nommer. Tranche 5, `windows`.
5. **Une prise servie pendant la reprise.** Avant de renommer une racine des données abîmée, Pupitre ferme chaque
   fichier qu'il tient, dont la prise que sert le protocole des captations (§ 31 · 2.1) : le test 14 éprouve le
   fichier tenu par un autre processus, et celui que tient Pupitre pendant une lecture s'éprouve aussi. Tranche 6,
   `windows`.

---

## Le déroulé

1. **Ce plan.** L'issue « Lot : INC-1, lot 1 · le fonds sur l'établi » s'ouvre sous la milestone « INC-1 · carnet et
   fonds » ; la PR de ce plan l'ajoute dans `.claude/plans/lot1-le-fonds-sur-l-etabli.md`, dit « Refs #24 », porte
   la table de couverture et consigne les contradictions de la décision 20 (`/consigner`). 👤 La maîtrise d'ouvrage
   la relit et tranche les questions de la décision 21. Elle retient la porte de la question 3 : une PR à part
   l'écrit avant la tranche 1, et la ligne LE-11 du registre la nomme.
2. **Chaque tranche, dans l'ordre.** `git fetch`, puis un worktree dans `pupitre.worktrees/<sujet>`. Le plan de
   tranche s'écrit par `superpowers:writing-plans` après deux passages : ce que la tranche livre, croisé avec ce que
   le lot ne contient pas (AM-2) ; chaque règle chiffrée recopiée, calculée sur ses exemples (LE-6). Une session
   neuve relit sa PR ; l'exécution suit `superpowers:subagent-driven-development` et les contraintes générales.
3. **Chaque PR.** Conventional Commits en français, en-tête de 100 caractères au plus, titre de 93 au plus ;
   « Refs #24 », et « Closes #24 » dans la PR de clôture ; la description nomme toute dépendance nouvelle, toute
   référence du test 11 changée et les recherches de LE-2 et LE-3 ; fusion par squash, une fois tout vert ; une
   tranche qui alourdit les tests remesure les délais de `verifier` et de `windows` ; l'installateur NSIS reste en
   artefact.
4. **Les étapes 👤 :** la relecture de ce plan ; l'essai installé de la tranche 3 ; les contrôles à l'œil de la revue
   finale ; l'installation de la PR de clôture, avant sa fusion ; l'étiquette `v0.3.0`.

---

## Ce qui doit survivre au plan

Le plan est retiré à la clôture du lot. Ce qu'il décide s'écrit avant, par la tranche qui le livre :

| Quoi | Où | Tranche |
|---|---|---|
| Les crates nouvelles, leurs dépendances, `en-memoire` et `pannes` | § 40 · 3 ; `CLAUDE.md` | 1 à 3 |
| La lecture seule, l'avertissement, la reprise au démarrage | § 31 · 2.1 et 6.1 | 1, 2, 5 |
| Le schéma de la `v0.3.0` | § 31 · 2 ; `noyau::donnees::fonds` | 2 |
| Annuler, ce que l'import décide (décisions 18 et 19) | § 30 · 11 ; CAP-02 ; CAP-03 | 2, 5 |
| Les règles mutées | `.cargo/mutants.toml` ; `CONVENTIONS.md` (`mutants`) | 1 à 6 |
| Les dépendances, le choix de `serde_json`, les usages de la boîte de dialogue | § 41 · 6 | 2, 3, 5 |
| La liste des noms dans `commandes`, la CSP et les droits | § 40 · 7.5 | 3 |
| Le module `systeme`, seul endroit qui reçoit les chemins du système ; I30 et les garde-fous | § 70 · 3 et 8 ; `CONVENTIONS.md` | 3 |
| Le mode démonstration par argument | § 31 · 1 | 7 |
| Le gel | § 31 · 10 (test 11) ; `CONVENTIONS.md` (`verifier`) | 9 |
| Les relevés à l'œil, et la façon de vérifier « le même journal » | § 70 · 7 | 3, 8, 9 |
| Les reports aux lots 2 à 4 qui n'y sont pas encore | § 40 · 9, « Avant chacun des lots suivants » | 9 |
| Les reports à INC-2 et au-delà qui n'y sont pas encore | `docs/pistes.md`, une piste par report, avec son incrément et sa raison ; pour PRO-02 · 6, que cite l'ADR-024, une phrase de plus au § 40 · 9 (INC-1, « Il ne contient pas ») : il s'éprouve en INC-2 | 9 |
