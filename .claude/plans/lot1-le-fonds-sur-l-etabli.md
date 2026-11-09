# Lot 1 d'INC-1 · le fonds sur l'établi — plan maître

> **Pour les agents :** ce plan n'est pas un plan tâche par tâche. Il fixe les huit tranches du lot, leur ordre, ce
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
2. **Huit tranches**, dans l'ordre des sections qui suivent.
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

### Tranche 8 · la revue finale et la clôture

**But :** éprouver le lot dans l'application installée, puis le clore avant la première donnée réelle.

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

**Elle dépend des** tranches 1 à 7. **Critères de sortie :** l'issue se clôt avec la PR de clôture ; sur une branche
jetable, une PR d'essai qui change une référence du gel est refusée, et une qui n'ajoute qu'une ligne passe ;
`verif-liens.mjs` et `verif-faits.mjs` verts. **Documents :** § 70 · 7 et 8 ; § 40 · 9 ; `CONVENTIONS.md` ;
`CLAUDE.md` ; README.

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
tranche 8 ; **les aides de tests recopiées**, en tranche 1.

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
| Le gel | § 31 · 10 (test 11) ; `CONVENTIONS.md` (`verifier`) | 8 |
| Les relevés à l'œil, et la façon de vérifier « le même journal » | § 70 · 7 | 3, 8 |
| Les reports aux lots 2 à 4 qui n'y sont pas encore | § 40 · 9, « Avant chacun des lots suivants » | 8 |
| Les reports à INC-2 et au-delà qui n'y sont pas encore | `docs/pistes.md`, une piste par report, avec son incrément et sa raison ; pour PRO-02 · 6, que cite l'ADR-024, une phrase de plus au § 40 · 9 (INC-1, « Il ne contient pas ») : il s'éprouve en INC-2 | 8 |
