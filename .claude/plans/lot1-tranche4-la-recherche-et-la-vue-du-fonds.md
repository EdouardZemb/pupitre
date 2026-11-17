# La tranche 4 du lot 1 d'INC-1 : la recherche et la vue du fonds — plan d'implémentation

> **Pour les agents :** exécuter ce plan avec `superpowers:subagent-driven-development`, tâche par tâche ;
> les étapes sont des cases à cocher (`- [ ]`).

Plan maître : `.claude/plans/lot1-le-fonds-sur-l-etabli.md`

**Objectif :** Sacha retrouve une idée par ce qu'elle porte ou par son texte, enregistre une requête, voit son fonds
et lit son journal. Chaque réponse déclare son assiette ; chercher n'écrit rien.

**Architecture :** deux PR. (a) pose au noyau la table de repli, les mots cherchés, les jours civils et leurs bornes,
les critères, l'index et sa réponse, les requêtes enregistrées et le journal rangé, tous sous mutation ; `commandes`
étend la boîte en requête, `services` tient l'index et le rebâtit quand les têtes du document changent, sert les
requêtes enregistrées et le journal en fenêtre ; l'établi sert les commandes nouvelles ; le message d'un geste date
chaque idée à son instant. (b) bâtit la vue du fonds : l'en-tête de l'établi (la recherche, l'état de
synchronisation, « Afficher » : Fonds ou Table), `/` et Échap, les critères à l'écran, la page d'un fonds vide, les
requêtes enregistrées et le journal à l'écran ; puis l'essai installé.

**Pile :** Rust 1.99.1, édition 2024 ; `tauri` 2.12.5 ; Leptos 0.8.23 ; `automerge` 0.12.0 ; nouvelle : `serde_json`
1.0.154, en dépendance de test des services et du stockage (déjà au `Cargo.lock`, par l'interface et l'établi) ; tests : `proptest`
1.12.0 ; en CI, `cargo-mutants` 27.1.0, trunk 0.22.1, tauri-cli 2.12.4.

**Spécification :** le plan maître, section « Tranche 4 · la recherche et la vue du fonds », ses décisions 9, 10, 17,
18 et 21 ; ce qu'il cite : CAP-02, CAP-06, CAP-07 ; § 30 · 9.2, 11 et 13 ; § 31 · 2, 2.3, 4.3, 7 et 10 ; § 40 · 7.5
et 9 ; § 50 · 2, 3.1, 5 et 7 ; § 70 · 3 (I01, I09, I13, I31), 4.1, 7 et 8 ; ADR-006, ADR-022 ; et le plan de la
tranche 3, `.claude/plans/lot1-tranche3-la-boite-a-l-ecran.md`, pour les noms qu'il a posés.

Refs #24

---

## Contraintes générales

Recopiées du plan maître (« Contraintes générales », « Le déroulé »), comme l'ont fait les tranches 1 à 3 ; chaque
tâche les suppose.

- « **En CI seulement.** Ni cargo, ni trunk, ni npm sur le poste : chaque vérification de Rust se lit dans le job que nomme le critère, le rouge puis le vert dans `verifier`, le vert aussi dans `windows` si le critère l'y met. » Les tests d'`etabli` ne tournent que dans `windows` ; ceux du noyau, de `commandes`, des services et de l'interface, dans `verifier` ; la suite des commandes sur fichiers (`crates/stockage/tests/commandes_sur_fichiers.rs`), dans les deux.
- « Seul `rustfmt --edition 2024` formate les `.rs` touchés ; les vérifications de documentation tournent sur le poste, après le `git add` (LE-7). » `rustfmt` se lance hors du dépôt, sur une copie dans le dossier temporaire, recopiée ensuite.
- « Les commandes de Rust prennent `-p <crate>`, et `ci.yml` nomme chaque crate qu'il passe à `clippy` et aux tests. »
- « **Le noyau** reste sans entrée-sortie, ni `automerge`, ni horloge, ni aléa ; chaque règle qui décide y vit, sous mutation (décision 4). » Le repli, les mots, les critères, l'assiette, l'index, les jours, les requêtes enregistrées et le journal rangé y vivent ; `services` ne fait que lire le document et tenir l'index.
- « **Les chemins se reçoivent** (I30) : hors de `systeme`, aucun code ne devine un chemin ; chaque test prend son dossier jetable par une seule fonction de sa crate. »
- « **Aucune donnée réelle** : textes et noms inventés ; identifiants, documents et captations engendrés à l'exécution ; aucun audio, MIDI ni `.automerge` suivi. »
- « **Les mots du glossaire**, dits en entier. » Les mots que la tranche fait entrer : requête, critère, recherche, requête enregistrée, table de repli, opération destructrice, état de synchronisation, archive complète, Pupitre neuf, en-tête de l'établi (questions 19, 22, 31, 32).
- « Une PR qui ajoute un mot au glossaire le cherche d'abord (`git grep -n -i -w <mot> -- docs README.md CONVENTIONS.md`), requalifie chaque emploi antérieur et cite la recherche (LE-2) ; une PR qui périme un mot ou un identifiant le cherche dans tout le dépôt, maquettes et commentaires SVG compris (LE-3). » La recherche se refait aussi sans accent et sur la majuscule accentuée (`-i` ne replie pas les accents).
- « **Chaque écran** qu'une tranche ajoute se mène au clavier, focus visible, avec ses libellés, au vocabulaire du glossaire, sert à 200 % (§ 50 · 7), et ses feuilles passent I17 et I18 (`encre_statique.rs`). » L'en-tête, les critères, la page d'un fonds vide, les requêtes enregistrées et le journal sont les écrans de cette tranche ; I18 refuse `set_timeout` et `request_animation_frame`, jusque dans les commentaires : aucune minuterie, aucun anti-rebond (question 16).
- « Conventional Commits en français, en-tête de 100 caractères au plus, titre de 93 au plus ; « Refs #24 » » ; `git add` par chemins explicites, jamais `git add .` ; `git commit -F <scratchpad>/msg-t4-<tâche>-<étape>.txt` (un texte accentué passe par un fichier).
- « Chaque PR qui livre un critère de la SFD en entier l'inscrit au tableau du § 70 · 4.1, et un test le nomme (`cap06_7_…`) et en calcule l'exemple ; `verif-faits.mjs` refuse un critère inscrit sans test qui le nomme. La porte ne voit pas un critère livré que la PR n'inscrit pas : la revue de chaque PR compare le tableau aux critères que la tranche dit livrer. » La PR (a) inscrit CAP-06 · 7 à 10 ; CAP-02 · 5 reçoit sa part « requêtes », nommée, sans s'inscrire (les photos viennent au lot 2) ; CAP-07 n'a aucun critère livrable ici (1 à 3 visent les morceaux, 4 est inscrit).
- « fusion par squash, une fois tout vert » ; mode `session` : la PR s'est ouverte en brouillon, donc `gh pr ready <n>`, puis, depuis le clone principal `pupitre`, sans `cd` ni `-R`, `gh pr merge <n> --squash --match-head-commit <SHA complet de la tête relue>`.
- « une tranche qui alourdit les tests remesure les délais de `verifier` et de `windows` ; l'installateur NSIS reste en artefact. »
- Le corps de chaque PR porte la section `## Écarts à la spec`, avec « Aucun. » : un écart à la spec se lève en corrigeant la spec dans la PR même, jamais en le déclarant. Le corps final s'écrit avant l'envoi qui le rend final (une retouche du corps ne relance que `texte-pr`, et `texte-pr` ne se relance jamais par `gh run rerun`).
- Chaque PR part de `main` à jour, depuis le clone principal (`git fetch`, puis `git worktree add ../pupitre.worktrees/<branche> -b <branche> origin/main`), s'ouvre en brouillon au premier envoi (`ci.yml` ne tourne que sur une PR), et dit dans sa description les tests réécrits et pourquoi.
- **Lire la CI** : `gh run list --branch <branche> --workflow ci --limit 1`, `gh run watch <n> --exit-status`, `gh run view <n> --log-failed`. Un rouge attendu se lit tel que la CI le rend : une erreur de compilation qui nomme ce qui manque, à « Lint » de `verifier` (ou à « Lint de l'établi » de `windows`) ; ou un test, à « Tests » (`panicked at`, avec le nom du test). `cargo test` s'arrête au premier fichier de tests rouge, pris dans l'ordre de leurs noms. Un vert se lit à « Tests » de `verifier`, et, pour `etabli` ou `stockage`, à leur étape de `windows` : `test result: ok.` pour chaque fichier touché, relevé par une file.
- Chaque nom de Leptos 0.8.23, de `web-sys`, d'`automerge` 0.12.0 (dont `Automerge::get_heads`, `fork`, `merge`) et de `serde_json` que le code emploie se vérifie sur docs.rs avant d'écrire ; un commentaire dans un bloc de test de ce plan est une spécification, qui devient des assertions.
- Chaque test qui prouve un critère de la SFD, inscrit ou livré par parts, se nomme `capNN_n_…`, précédé de `#[test]` ; chaque test d'un invariant se nomme `i01_…`, `i09_…`, `i13_…` ou `i31_…`.
- Les heures sont des secondes, partout où le code les reçoit ; seul `systeme` lit l'horloge ; l'interface ne lit le décalage local que par `natif::decalage_du_geste` et `natif::decalage_a`, les seules fonctions que son contrôle statique admet (`app/ui/tests/statique.rs`).
- Un numéro de ligne qu'une tâche cite vaut sur `main` à `bc0daca` : une tâche antérieure du plan peut le décaler, et l'élément nommé (la fonction, l'import, la doc) fait foi ; l'implémenteur le cherche par son nom.
- Les identifiants d'Automerge, d'ULID et de propriétaire ne s'écrivent jamais dans un message montré : un refus nomme une requête par son nom, jamais par son ULID.
- Les contrôles statiques du noyau et des services cherchent en sous-chaîne, commentaires compris (`crates/noyau/tests/statique.rs:22-48`, `crates/services/tests/statique.rs`) : les noms et les docs de la tranche évitent `Instant` (« Instantané » aussi), `File` (« Filet »), `io::`, `time::`, `HashMap`, `HashSet`, `random`, `spawn` ; l'index prend `BTreeMap` et `BTreeSet`.

---

## Ce que les passages ont levé

Trois passages, faits avant ce plan, ne sont pas dans le dépôt : la carte du code (pièges P1 à P20), le passage de
LE-6 (L1 à L17, et vingt et une règles calculées sans écart) et celui d'AM-2 (C1 à C15, dont C15 relève neuf voies où
les deux autres divergent). Leur détail a servi à écrire ce plan ; la description de sa PR porte la table de
couverture. Trente-six questions en sont nées, posées en un bloc à la maîtrise d'ouvrage, qui les a tranchées le
2026-11-17, chacune selon son défaut (« ok pour les défauts ») ; elles sont recopiées plus bas. La PR de ce plan
consigne (`/consigner`), au motif « Une spec qui se contredit » (septième constat ; la porte existe depuis LE-11), une
ligne, LE-16, qui nomme L4, L12, L13, L17, C1, C2, C3, C4, C7, C9, C11, C12 et C13 ; la tâche 17 la clôt.

### La carte du code

| Piège | Décision de ce plan | Tâche |
|---|---|---|
| P1. L'assiette à trois nombres ne sait pas dire « sans durée » | `commandes::Assiette` gagne `illisibles` ; la réponse porte ses parts « sans » ; un texte neuf au noyau, `texte_de_la_reponse` ; questions 6 et 7 | 3, 4 |
| P2. Aucun type de critère ni de requête | `Requete` au noyau et dans `commandes`, une clé par sorte de critère ; la commande `boite` s'étend en requête ; question 21 | 3, 4 |
| P3. Aucune forme d'une requête enregistrée que les gardes laissent passer | Un objet par requête, ses critères posés à sa naissance ; `permis`, `message_connu` et l'historique étendus ; question 17 | 6, 7 |
| P4. Aucune crate de repli | Une table au noyau, rangée, cherchée par dichotomie, gardée entrée par entrée par un test ; question 1 | 1 |
| P5. L'index sans invalidation, sans lieu, sans empreinte | Au noyau, tenu par `services`, rebâti à la lecture qui suit un changement des têtes ; l'empreinte d'une réponse, SHA-256 de son JSON ; questions 13 et 14 | 3, 4, 5 |
| P6. Le journal sans fenêtre ni heure | `journal` en fenêtre (`VueDuJournal`), le plus récent d'abord, l'heure au décalage de l'instant, `—` pour la genèse ; question 29 | 9 |
| P7. « Bandeau », deux sens | L'en-tête de l'établi porte la recherche et l'état de synchronisation ; « bandeau » reste le message d'I29 ; question 22 | 12, 17 |
| P8. `/` sans lieu, Maj+: au clavier AZERTY | `slash_cherche`, la touche produite ; l'écoute dans `App` ; question 24 | 11, 12 |
| P9. Aucune minuterie | Une commande par frappe, la réponse devancée ignorée ; question 16 | 13 |
| P10. Aucune page de fonds vide | Deux constats, sans impératif ; question 26 | 14 |
| P11. La date dépend du décalage | Des jours civils ; les bornes calculées par l'interface au décalage de chaque jour ; question 9 | 2, 11 |
| P12. Le seuil de durée | À la seconde affichée, strict ; question 8 | 3 |
| P13. L'étiquette comparée octet pour octet | Le critère le garde, l'écran la choisit parmi celles du fonds ; question 10 | 3, 13 |
| P14. La ligne tronquée à 200 caractères | La recherche lit tout le texte ; un résultat ouvre l'idée entière ; question 5 | 3, 13 |
| P15. `boite.rs` fait 2 517 lignes | La recherche dans un module neuf, `recherche.rs` ; la scission reste une piste ; question 35 | 11 |
| P16. CAP-06 absent du tableau | « CAP-06 \| 7 à 10 » ; CAP-02 · 5 reste en part (C15) | 5, 10 |
| P17. La mutation s'étend | Quatre modules neufs du noyau, chacun dans `examine_globs` et `--test` ; les délais remesurés | 1 à 3, 6, 9, 10 |
| P18. Les gardes statiques lisent les commentaires | `BTreeMap` ; les mots interdits relus avant chaque doc | 1 à 9 |
| P19. Le compte annoncé à chaque frappe | La région du compte, polie, la dernière réponse remplace l'annonce ; question 27 | 13 |
| P20. Chercher sert en lecture seule, enregistrer non | Une lecture sans `Contexte` ; trois opérations nommées avec lui ; question 20 | 4, 7 |

### Le passage de LE-6

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| L1. La table de repli n'est écrite nulle part | Question 1 : 120 entrées, recopiées dans le test | 1 |
| L2. Ce qu'est un mot cherché | Les blancs coupent, la ponctuation des bords s'ôte ; question 2 | 1 |
| L3. Fragment sans longueur minimale, requête sans mot | Aucune borne ; sans mot, aucun critère de texte ; question 3 | 1, 3 |
| L4. L'assiette tient « considérées − portant » pour les illisibles (écart) | `illisibles` à part ; question 6 | 3, 4 |
| L5. Plusieurs critères | « Et » ; « sans … » compté parmi les candidates ; question 7 | 3 |
| L6. Le seuil de durée | À la seconde affichée ; question 8 | 3 |
| L7. La date, un jour civil | Des bornes au décalage de chaque jour ; question 9 | 2, 11 |
| L8. L'étiquette | Octet pour octet, choisie parmi celles du fonds ; question 10 | 3, 13 |
| L9. Le texte long | Tout le texte, la ligne inchangée ; question 5 | 3 |
| L10. I13 et « les mêmes résultats » | La même empreinte de la réponse entière, sur un jeu fixé ; question 13 | 5 |
| L11. La forme d'une requête enregistrée | Un objet, effacé d'un bloc ; une sorte inconnue la dit d'une version plus récente ; questions 17 et 18 | 6, 7 |
| L12. Le journal sans fenêtre, et l'écart d'un jour (écart) | Questions 29 et 30 | 9 |
| L13. « Bandeau » au plan maître (écart) | Question 22 | 12, 17 |
| L14. « Toujours visible », `/`, la page d'ouverture | Questions 23 et 24 | 11, 12 |
| L15. Un Pupitre neuf n'est pas le seul fonds vide | Aucune idée interrogée ; question 26 | 14 |
| L16. CAP-06 · 6 et le fonds de mesure | L'index compte dans l'ouverture ; question 15 | effets |
| L17. Une proposition trouvée, `~` ou « pas décidée » (écart) | Rien au lot 1, une piste pour INC-2 ; question 12 | 10 |

Les vingt et une règles sans écart sont recopiées dans les tâches qui les livrent : « eglise » trouve « Église »
(règle 1) et « oe » trouve « œ » (règle 2) à la tâche 1 ; CAP-06 · 7 à 9 (règles 3 à 5), la provenance, l'état et le
contenu sans part « sans » (règle 6) et l'ordre total (règle 7) aux tâches 3 et 5 ; le rangement par étiquette octet
pour octet (règle 8) à la tâche 3 ; les requêtes enregistrées hors de ce qu'annuler vise (règle 9) à la tâche 7 ;
chercher n'écrit rien (règle 10) aux tâches 5 et 7 ; « Sans nœud · aucune archive » (règle 11) à la tâche 12 ; la
genèse au journal et la date d'une ligne (règles 12 et 13) à la tâche 9 ; l'index sans base (règles 14 et 15) à la
tâche 3 ; le même repli partout (règle 16) à la tâche 1 ; les exemples d'assiette (règle 17) aux tâches 3 et 17 ; le
nom d'origine (règle 18) à la tâche 10 ; les critères de sortie (règle 19) aux tâches 5 et 10 ; les droits (règle 20)
à la tâche 8 ; une réponse petite (règle 21) à la tâche 4.

### Le passage d'AM-2

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| C1. Ce que la tranche 5 verse, la recherche ne le lit pas | Le texte de l'idée seul ; le § 30 · 9.2 se corrige ; la piste « Chercher par nom d'origine » s'étend au texte versé ; question 4 | 10 |
| C2. « Provenance », trois sens | Le critère est la provenance de l'idée, captée ou importée ; une piste pour l'origine déclarée ; question 11 | 3, 10 |
| C3. Une requête enregistrée dans un Pupitre neuf | Refusée tant que le fonds n'est pas né ; le § 31 · 2.3 le dit ; question 20 | 7, 10 |
| C4. « Supprimer » une requête | Pas une opération destructrice ; sans confirmation ; le glossaire gagne l'entrée ; question 19 | 7, 10 |
| C5. Cinq mots pour deux choses | Requête, critère, recherche, requête enregistrée au glossaire ; `Requete`, `Critere` au code, jamais `Sorte` ; question 32 | 3, 10 |
| C6. Une liste ou deux, une commande ou deux | Une liste, une commande ; noter pendant une requête la laisse en place ; question 21 | 4, 13 |
| C7. La zone du haut | L'en-tête de l'établi ; question 22 | 12, 17 |
| C8. Échap, le fil d'Ariane, les raccourcis documentés | Échap ferme les volets ; pas de fil d'Ariane au lot 1 ; `aria-keyshortcuts` ; question 25 | 11, 14, 17 |
| C9. « Le même journal » sans définition | `empreinte_du_journal` au noyau ; la vue montre le nombre d'entrées et l'empreinte courte ; question 29 | 9, 16 |
| C10. Les textes piégés ailleurs que dans une idée | Une étiquette et le nom d'une requête aussi ; question 31 | 7, 18 |
| C11. La vue du fonds contre CAP-07 et la maquette | Ce que la vue laisse, part par part ; la santé du nœud aux reports ; la maquette aux dates absolues ; question 28 | 17 |
| C12. Les documents au-delà de la ligne « Documents » | Question 33 | 10, 17 |
| C13. « Ce qui doit survivre » n'a aucune ligne de la tranche 4 | Question 34 : « Les écarts au plan maître », écart 9 | 10, 17 |
| C14. Ce que reçoivent les autres tranches et lots | « Effets cherchés » | effets |
| C15. Neuf voies divergentes | Chaque voie suit la colonne « Ce qui tranche » ; la commande `boite` étendue, sans `chercher` ; question 21 | toutes |

### La relecture par une session neuve

Une session neuve a relu ce plan le 2026-11-17, selon « La revue du plan » (`docs/modele/methode.md`), chaque
mécanisme annoncé relu dans le code de `bc0daca`, chaque valeur recalculée (la table de repli, les instants, les
réponses des jeux, l'empreinte du journal) : dix constats (un bloquant, deux majeurs, sept mineurs), tous retenus.
Chacun a été cherché aussi ailleurs dans le plan, et la correction suit partout où la même croyance paraissait.

| Constat | Décision de ce plan | Tâche |
|---|---|---|
| R1. Bloquant : la suite des commandes, que trois tests du stockage incluent, appelle `serde_json`, absent des dépendances de test du stockage (`crates/stockage/Cargo.toml`) | `serde_json` en dépendance de test du stockage aussi ; la ligne d'« Effets cherchés » nomme le stockage ; le § 41 · 6 ne change pas, son budget ne comptant pas les dépendances de test | 4, 10 |
| R2. Majeur : délier part du volet, qui ne garde pas la naissance de l'autre bout, et rétablir part de la liste des retirées (`app/ui/src/boite.rs:417-420`, `:1212`, `:1260`) | `commande_de_lien` ; le volet garde la naissance de chaque idée liée ; rétablir depuis la liste prend le `le` de sa ligne ; un bout illisible prend le décalage du geste, limite dite | 9 |
| R3. Majeur : « Contenu : » restait sur un signal de la boîte que la commande ne lirait plus, et `CONTENUS` est privé (`boite.rs:53`, `:463`) | `CONTENUS` en `pub(crate)` ; « Contenu : » passe à la requête ; `BoiteLue` garde la requête lue | 11, 13 |
| R4. Mineur : `mesure.rs` écrit `Relier` et `Regrouper` (`crates/services/src/mesure.rs:94`, `:144`) | Ajouté aux fichiers de la tâche, le second décalage à 0 | 9 |
| R5. Mineur : sous WebView2, Échap vide un `<input type="search">` | Dans la recherche, Échap garde ce sens et ne ferme aucun volet ; une ligne d'outil ; un point de l'essai | 11, 18 |
| R6. Mineur : le point de revue 1 n'éprouvait pas la note que la requête trouve | Une seconde note, « l’église du soir », que la liste montre | 5 |
| R7. Mineur : `EtatDeLaRequete` créé par `App` et par `Etat::nouveau` | Par `Etat::nouveau` seul | 12 |
| R8. Mineur : la doc de `Requete::texte` mêlait le texte saisi et ses mots | Le texte saisi, tel quel ; ses mots par `repli::mots` | 3 |
| R9. Mineur : « l'évaluation sans index » est un index neuf | La doc de `boite_sans_index` le dit, et pourquoi le test tient ; la PR (a) le dit | 4 |
| R10. Mineur : `aucun_compteur_d_anciennete` défait la vue par un motif exhaustif, et `assiette_de` bâtit l'assiette (`suite/mod.rs:2555`, `:2473`) | Les deux champs neufs nommés dans le motif ; `illisibles: 0` | 4 |

Un relecteur neuf a vérifié les corrections : R5 restait incomplet (l'écoute ne savait pas dire si la cible est la
recherche), et trois défauts nouveaux paraissaient ; tous corrigés : `est_la_recherche` et `ID_DE_LA_RECHERCHE`, testés
(tâche 11) ; la doc d'`EtatDeLaRequete`, tenue par `Etat` ; ce que dit le corps de la PR (a), R2 et R9 compris (tâche 10).

## Les écarts au plan maître

Le plan maître le veut : un plan de tranche qui le contredit le dit dans sa PR, et la maîtrise d'ouvrage relit ce
point. Ce plan ne touche pas le plan maître ; chaque écart est dit ici, et la maîtrise d'ouvrage les a tranchés avec
les questions, le 2026-11-17, chacun selon son défaut.

1. **Deux PR** (question 35) : (a) le noyau, les commandes, les services et l'établi qui les sert, avec l'adaptation
   de l'interface que le changement des types demande et l'envoi des décalages de l'écart d'un jour ; (b) la vue du
   fonds et l'essai installé.
2. **L'en-tête de l'établi, non un bandeau** (question 22) : la recherche et « Sans nœud · aucune archive » vivent
   dans le composant `Entete`, qui porte déjà « Afficher » ; le mot « bandeau » reste le message d'I29. La ligne de la
   tranche 6 (« Le bandeau « Sans nœud · archive du … » ») se lit : l'état de synchronisation, dans l'en-tête.
3. **L'établi s'ouvre sur le fonds** (question 23), et « Afficher » choisit « Fonds » ou « Table ».
4. **La commande `journal` passe en fenêtre** (question 29) : elle rend une `VueDuJournal` (le nombre d'entrées,
   l'empreinte du journal, une fenêtre de lignes), contre la forme de la tranche 2, qui rendait tout (ADR-022 : des vues
   petites). `Services::journal` reste, pour les tests.
5. **Le message d'un geste date chaque idée à son instant** (question 30) : `Relier` gagne `decalage_b_minutes`,
   `Regrouper` `decalage_source_minutes`, et chaque décalage d'une commande de geste devient celui de l'instant de
   l'idée qu'il date. La piste « L'écart d'un jour » sort de `docs/pistes.md`.
6. **Enregistrer une requête est refusé avant la naissance du fonds** (question 20) : le § 31 · 2.3 le dit.
7. **Supprimer une requête n'est pas une opération destructrice** (question 19) : le lot 1 n'en a toujours aucune ;
   le glossaire définit le mot.
8. **Documents** (question 33) : CAP-02, CAP-06, CAP-07 ; § 30 · 9.2 et 11 ; § 31 · 2, 2.3, 7 et 10 ; § 40 · 7.5, relu (ses « requêtes du
   fonds » couvrent les quatre commandes nouvelles) ; § 40 · 9 ;
   § 41 · 6 ; § 50 · 2, 3.1, 5 et 7 ; la maquette du fonds ; § 70 · 3, 4.1, 7 et 8 ; le glossaire ; `docs/pistes.md` ;
   le registre (LE-16) ; `.cargo/mutants.toml` ; `CONVENTIONS.md` ; `CLAUDE.md`. Le plan maître n'en nommait que quatre.
9. **Ce qui doit survivre** (question 34) : la table de repli et les règles de la recherche (CAP-06, le glossaire) ;
   l'index et l'empreinte d'une réponse (§ 31 · 7 et 10, § 70 · 3) ; la forme d'une requête enregistrée (§ 31 · 2,
   § 30 · 11) ; la vue du journal et « le même journal » (§ 50 · 3.1, § 70 · 7) ; l'en-tête, ses mots, `/` et Échap
   (§ 50 · 2 et 3.1, le glossaire). Chaque PR les écrit à leur place ; la clôture du lot les relit.
10. **Un essai installé** (question 36), court, avant la fusion de la PR (b) ; le plan maître n'en prévoit qu'aux
    tranches 3 et 8.
11. **Les reports** (questions 11, 12, 28) : la santé du nœud au lot 4 ; l'origine déclarée, une piste qui se
    déclenche en tranche 5 ; une proposition trouvée, une piste pour INC-2. La ligne des reports « Le journal du fonds
    à l'écran, hors ce que retient la décision 21 » se lit : la vue est livrée ici ; la version de Pupitre d'une
    migration reste à la première migration publiée.
12. **CAP-02 · 5 reste en part** (C15) : sa part « requêtes » a son test nommé, sans s'inscrire, puisque les photos
    viennent au lot 2.

## Les questions tranchées

Posées le 2026-11-17 en un bloc, tranchées le même jour, chacune selon son défaut.

1. **Le repli** : minuscules (`char::to_lowercase`), puis une table au noyau, sans crate : les marques combinantes
   ôtées (formes composées et décomposées), les lettres de Latin-1 et de Latin étendu A réduites à leur base, œ → oe,
   æ → ae, ß → ss, les ligatures récrites, les invisibles ôtés ; aucune forme de compatibilité ; chaque entrée gardée
   par un test.
2. **Le mot** : séparé par les blancs, insécables compris ; apostrophes et tirets repliés dans un mot ; la ponctuation
   des bords ôtée.
3. **Fragments et requête vide** : aucune longueur minimale ; une requête sans mot ne pose aucun critère de texte.
4. **Ce que lit la recherche** : le texte de l'idée seul ; la piste « Chercher par nom d'origine » s'étend au texte
   versé ; la PR dit qu'un fonds importé répondra surtout « sans texte ».
5. **Le texte long** : la recherche lit tout le texte ; la ligne garde ses 200 premiers caractères, sans marque ; un
   résultat ouvre l'idée entière.
6. **L'assiette** : elle compte les illisibles à part (`illisibles`) ; la boîte ne change pas à l'écran.
7. **Plusieurs critères** : « et » ; un contenu par requête ; « sans … » compté parmi les idées qui satisfont les
   autres critères qu'elles portent ; les parts dans l'ordre date, étiquette, durée, état, provenance, contenu, texte,
   puis illisibles, puis interrogées.
8. **La durée** : à la seconde affichée, strictement ; une idée affichée « 1:00 » n'est jamais « plus d'une minute » ;
   une idée dont une prise n'a pas de durée reste « sans durée ».
9. **La date** : une période de jours civils, « du … au … », bornes comprises, chacune facultative ; une requête
   enregistrée garde des jours civils ; l'interface calcule les bornes au décalage de chaque jour ; aucune date
   relative.
10. **L'étiquette** : choisie parmi celles du fonds, comparée octet pour octet.
11. **La provenance** : celle de l'idée, captée ou importée ; une piste pour l'origine déclarée.
12. **Une proposition trouvée** : rien au lot 1 ; une piste pour INC-2.
13. **I13 et le test 1** : la même empreinte (SHA-256 du JSON) de la réponse entière de chaque requête d'un jeu fixé ;
    le test joue des gestes, une annulation et une écriture refusée, puis compare l'index tenu, l'index rebâti du
    document rouvert et une évaluation sans index ; la boîte devient une requête sans critère sur le même index.
14. **Quand l'index se refait** : à la lecture qui suit un changement des têtes du document, jamais pendant une
    écriture ; la recherche est une seule commande.
15. **CAP-06 · 6** : la construction de l'index compte dans l'ouverture, pas dans la recherche ; la tranche 7
    chronomètre un mot absent, « re » et une combinaison ; le fonds de mesure ne change pas.
16. **Sans anti-rebond** : une commande par frappe, aucune minuterie ; une réponse devancée par une plus récente
    s'ignore.
17. **La forme d'une requête enregistrée** : un objet par requête, né avec son nom et une clé par sorte de critère,
    les critères jamais modifiés ; supprimer efface l'objet ; un test à deux répliques ; une sorte inconnue : la requête
    paraît nommée, ne se lance pas et dit « Cette requête vient d'une version plus récente de Pupitre. » ; I01 s'étend
    à cette forme seule.
18. **Le nom** : vide refusé ; doublons admis ; sans borne de longueur ; la liste rangée par nom replié ; « Requête
    « … » enregistrée », « Requête « … » renommée « … » », « Requête « … » supprimée ».
19. **Supprimer** : sans confirmation, ce n'est pas une opération destructrice ; le glossaire gagne l'entrée.
20. **Pupitre neuf et lecture seule** : enregistrer y est refusé, le bandeau dit chaque refus ; chercher reste permis.
21. **Une liste, une commande** : la liste de la boîte est la réponse de la requête en cours ; `boite` s'étend en
    requête ; noter pendant une requête la laisse en place, et le compte des non reliées monte d'un.
22. **La zone du haut** : l'en-tête de l'établi (l'écart 2).
23. **Afficher et l'ouverture** : « Fonds » ou « Table » ; l'établi s'ouvre sur le fonds ; la recherche paraît sur
    les deux, et ses résultats font passer au fonds.
24. **`/`** : la touche produite (Maj+: au clavier AZERTY), refusée avec Ctrl, Alt, AltGr, Méta, en répétition ou en
    composition ; hors d'un champ, elle porte le focus dans la recherche sans s'y écrire ; dans un champ, elle s'écrit.
25. **Échap** : il ferme le volet ouvert, le focus rendu comme par « Fermer » ; pas de fil d'Ariane au lot 1 ; `/` et
    `Ctrl+Z` documentés par `aria-keyshortcuts` et l'aide du champ.
26. **Le fonds vide** : aucune idée interrogée ; un Pupitre neuf montre la même page ; elle propose noter, sauf en
    lecture seule ; les idées retirées restent à un geste.
27. **Les annonces** : la région du compte, polie ; la dernière réponse remplace l'annonce d'avant.
28. **La vue en parts** : ce que la vue du lot 1 laisse de CAP-07, part par part ; la santé du nœud au lot 4 ; la
    maquette aux dates absolues.
29. **Le journal à l'écran** : une commande à fenêtre, le plus récent d'abord ; « 25 oct. 2026 · 02:30 · message », à
    l'heure locale de l'instant, `—` pour la genèse ; le nombre d'entrées et l'empreinte courte, six caractères ;
    « le même journal », une fonction du noyau.
30. **L'écart d'un jour** : corrigé (l'écart 5).
31. **Les textes piégés** : dans une étiquette et dans le nom d'une requête aussi ; le glossaire gagne « Archive
    complète », « État de synchronisation », « Pupitre neuf ».
32. **Les mots** : « Requête », « Critère », « Recherche », « Requête enregistrée » ; le « filtre » du code devient le
    contenu d'une requête ; « sorte » reste aux captations.
33. **Les documents** : l'écart 8.
34. **Ce qui doit survivre** : l'écart 9.
35. **Les PR** : l'écart 1 ; la recherche dans un module neuf de l'interface ; la scission de `boite.rs` reste une
    piste.
36. **L'essai installé** (👤) : l'écart 10.

Les choix techniques que ce plan tranche sans question : une requête sans aucun critère ne s'enregistre pas
(`Impossible::RequeteSansCritere`), pour qu'aucune requête enregistrée ne rende tout le fonds ; renommer en le même nom
est refusé (`Impossible::MemeNom`), comme compléter par le même texte ; un nom se garde sans ses blancs autour ; une part
« sans … » nulle ne s'écrit pas, comme les illisibles depuis la tranche 3 ; l'écran offre trois seuils de durée (30 s,
une minute, 5 min) ; un seuil au-delà de 2⁵³ − 1 secondes se refuse (`Impossible::SeuilTropLong`) ; les étiquettes
du fonds viennent avec la vue de la boîte ; la commande `boite` garde son nom ; `retenues` garde le sien ;
`serde_json` en dépendance de test des services et du stockage, pour l'empreinte d'I13 : la suite des commandes, qui la
calcule, est incluse par trois tests du stockage (`commandes_sur_fichiers.rs`, `references.rs`, `repliques.rs`), et un
test ne voit que les dépendances de son crate.

---

## Points de revue

Cinq conditions que la spec implique sans qu'aucune de ses sources ne demande de les éprouver ; chacune a son test,
dans la tâche qui possède le code.

1. **Noter pendant une requête** : la note est faite, la requête reste, la liste la montre si elle la satisfait, et
   le compte des non reliées monte d'un. `noter_pendant_une_requete_la_laisse_en_place`, tâche 5 (côté services) ;
   `noter_ne_touche_pas_la_requete_en_cours`, tâche 13 (côté écran).
2. **Un fonds en lecture seule** : la recherche sert, enregistrer, renommer et supprimer se refusent du refus de la
   lecture seule. `en_lecture_seule_chercher_sert_et_enregistrer_est_refuse`, tâche 7.
3. **Une requête enregistrée par une version plus récente** (une clé inconnue, une valeur inconnue) : elle paraît
   nommée et ne se lance pas. `une_cle_inconnue_rend_la_requete_d_une_version_plus_recente`, tâche 6 ;
   `une_requete_plus_recente_se_nomme_et_ne_se_lance_pas`, tâche 15.
4. **Supprimer la requête en cours** : ses critères restent à l'écran, seule la liste des requêtes enregistrées change.
   `supprimer_la_requete_en_cours_laisse_ses_criteres`, tâche 15.
5. **Échap sans volet ouvert**, dans un champ pendant une composition, ou dans la recherche : rien ne se ferme, le
   focus ne bouge pas.
   `echap_ne_ferme_que_le_volet_ouvert`, tâche 11.

## Mécanismes annoncés

- `crates/noyau/src/donnees/boite.rs:145` — `pub fn boite(` — une seule assiette, un seul contenu : `Index::repondre` la remplace, et la tâche 4 la retire
- `crates/noyau/src/donnees/boite.rs:156` — `assiette(non_retirees.iter().map(|e| &e.idee), |_| true)` — l'assiette du compte, que la réponse de l'index reprend sans critère
- `crates/noyau/src/donnees/boite.rs:194` — `let illisibles = assiette.considerees - assiette.portant;` — juste pour le seul compte de la boîte ; la réponse d'une requête porte ses illisibles à part (tâche 3)
- `crates/noyau/src/donnees/boite.rs:216` — `pub fn texte_du_filtre(` — remplacée par `texte_de_la_reponse` (tâches 3 et 4)
- `crates/noyau/src/donnees/boite.rs:122` — `pub fn ordonner(` — l'ordre total que la réponse d'une requête reprend, jamais la pertinence (tâche 3)
- `crates/noyau/src/donnees/boite.rs:248` — `let secondes = duree_ms / 1_000;` — la seconde affichée, que le seuil de durée suit (tâche 3)
- `crates/noyau/src/donnees/boite.rs:232` — `pub fn date_de_la_liste(heure: i64, decalage_minutes: i32) -> String {` — la date d'une ligne du journal (tâche 9)
- `crates/noyau/src/donnees/idee.rs:48` — `pub fn duree(prises: &[Option<u64>]) -> Option<u64> {` — `None` dès qu'une prise n'a pas sa durée : CAP-06 · 9 compte l'idée « sans durée » (tâches 3 et 5)
- `crates/noyau/src/donnees/idee.rs:65` — `pub struct Assiette {` — reste à trois nombres au noyau ; la réponse porte `illisibles` à côté (tâche 3)
- `crates/noyau/src/donnees/gestes.rs:243` — `pub(crate) fn date_civile(heure: i64, decalage_minutes: i32) -> (i64, usize, i64) {` — le calcul civil, que les jours d'une requête et l'heure du journal partagent (tâches 2 et 9)
- `crates/noyau/src/donnees/gestes.rs:351` — `fn jour(idee: &IdeeLue, decalage_minutes: i32) -> String {` — date l'idée au décalage reçu : la commande portera celui de l'instant de l'idée (tâche 9)
- `crates/noyau/src/donnees/gestes.rs:480` — `pub fn relier(` — un seul décalage pour deux idées : il en prend deux (tâche 9)
- `crates/noyau/src/donnees/gestes.rs:548` — `pub fn regrouper(` — de même pour la cible et la source (tâche 9)
- `crates/noyau/src/donnees/gestes.rs:129` — `pub const ENTIER_SUR: u64 = 9_007_199_254_740_991;` — la borne d'un seuil de durée enregistré (tâche 6)
- `crates/noyau/src/donnees/gestes.rs:143` — `pub enum Impossible {` — gagne les refus d'une requête enregistrée (tâche 6)
- `crates/noyau/src/donnees/changement.rs:24` — `Creer {` — un objet à clés, sous lequel naît une requête enregistrée, ses clés posées dans la même modification (tâche 6)
- `crates/noyau/src/donnees/fonds.rs:25` — `pub const REQUETES_ENREGISTREES: &str = "requetes_enregistrees";` — le conteneur, déjà dans la genèse : ni genèse ni migration (tâche 6)
- `crates/noyau/src/donnees/identifiants.rs:45` — `pub fn empreinte(octets: &[u8]) -> String {` — l'empreinte du journal et celle d'une réponse (tâches 5 et 9)
- `crates/commandes/src/lib.rs:56` — `pub const NOMS: &[&str] = &[` — gagne quatre noms (tâche 7)
- `crates/commandes/src/lib.rs:123` — `pub struct Relier {` — gagne `decalage_b_minutes` (tâche 9)
- `crates/commandes/src/lib.rs:155` — `pub struct Regrouper {` — gagne `decalage_source_minutes` (tâche 9)
- `crates/commandes/src/lib.rs:320` — `pub struct Boite {` — `filtre` devient `requete`, et `bornes` s'ajoute (tâche 4)
- `crates/commandes/src/lib.rs:340` — `considérées − portant, au lot 1,` — la doc se corrige : les illisibles se comptent à part (tâche 4)
- `crates/commandes/src/lib.rs:381` — `pub struct VueDeLaBoite {` — l'assiette de la requête, ses parts « sans », les étiquettes du fonds (tâche 4)
- `crates/commandes/tests/serialisation.rs:286` — `assert_eq!(NOMS.len(), 17);` — passe à 21 (tâche 7)
- `crates/services/src/lib.rs:69` — `pub struct Services<D: Documents, C: Captations> {` — gagne l'index tenu (tâche 4)
- `crates/services/src/lib.rs:389` — `pub fn journal(&self) -> Reponse<Vec<EntreeDuJournal>> {` — reste pour les tests ; `journal_en_fenetre` sert la commande (tâche 9)
- `crates/services/src/lib.rs:406` — `pub fn boite(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite> {` — passe par l'index tenu (tâche 4)
- `crates/services/src/lib.rs:415-421` — `assiette_du_filtre: assiette,` — la copie de l'assiette du compte, que remplace celle de la requête (tâche 4)
- `crates/services/src/lib.rs:455` — `fn trier(` — seul à écrire le dernier geste de tri : les opérations sur une requête n'y passent pas (tâche 7)
- `crates/services/src/lib.rs:486` — `fn en_boite(fonds: &Automerge) -> (Vec<EnBoite>, usize) {` — relit tout le fonds : l'index se bâtit d'elle (tâche 4)
- `crates/services/src/lib.rs:604` — `fn ulid_tire(contexte: &mut Contexte<'_>) -> String {` — l'ULID d'une requête enregistrée (tâche 7)
- `crates/services/src/lecture.rs:157` — `pub fn journal(doc: &Automerge) -> Vec<(String, i64)> {` — l'ordre de l'historique, que `ranger_le_journal` reçoit (tâche 9)
- `crates/services/src/application.rs:48` — `pub fn appliquer(doc: &mut Automerge, operation: &Operation)` — le test à deux répliques l'applique à chaque copie (tâche 7)
- `crates/stockage/src/adaptateurs.rs:107` — `Aucun fonds n'est né.` — le texte du refus d'enregistrer avant la naissance (tâche 7)
- `crates/services/tests/suite/mod.rs:699` — `pub fn message_connu(message: &str) -> bool {` — reconnaît les trois messages d'une requête (tâche 7)
- `crates/services/tests/suite/mod.rs:789` — `fn permis(chemin: &[String]) -> bool {` — s'étend à la forme d'une requête enregistrée, et à elle seule (tâche 7)
- `crates/services/tests/suite/mod.rs:768` — `const INTERDITS_PARTOUT: [&str; 3] = ["compte", "total", "a_envoyer"];` — à côté, les noms interdits sous une requête (tâche 7)
- `crates/services/tests/suite/mod.rs:2849` — `pub fn un_texte_piege_revient_tel_quel<P: Paire>(paire: &mut P) {` — s'étend à une étiquette et au nom d'une requête (tâche 7)
- `app/src-tauri/src/commandes_de_l_etabli.rs:125` — `pub async fn journal(ouverture: tauri::State<'_, Ouverture>) -> Reponse<Vec<EntreeDuJournal>> {` — prend une `Fenetre` (tâche 9)
- `app/src-tauri/src/lib.rs:135` — `tauri::generate_handler![` — gagne les quatre commandes (tâche 8)
- `app/src-tauri/capabilities/windows.json:28` — `"allow-exporter-cahier"` — les quatre droits s'ajoutent (tâche 8)
- `app/ui/src/app.rs:22` — `pub enum Vue {` — `Fonds`, par défaut, et `Table` (tâche 12)
- `app/ui/src/entete.rs:84` — `let vues = [(Vue::Table, "Table"), (Vue::Boite, "Boîte d'idées")]` — devient « Fonds » et « Table » (tâche 12)
- `app/ui/src/boite.rs:209` — `pub(crate) fn ctrl_z_annule(touche: &str, ctrl: bool, dans_un_champ: bool) -> bool {` — le modèle de `slash_cherche` (tâche 11)
- `app/ui/src/boite.rs:349` — `fn dans_un_champ_de_texte(ev: &leptos::ev::KeyboardEvent) -> bool {` — passe en `pub(crate)`, pour `/` et Échap (tâche 11)
- `app/ui/src/boite.rs:537` — `async fn lire_la_boite(` — envoie la requête en cours et ses bornes (tâches 4 et 13)
- `app/ui/src/boite.rs:1007` — `fn au_debut(self) {` — un critère changé ramène la liste au début (tâche 13)
- `app/ui/src/boite.rs:449` — `fn est_la_derniere(self, rang: u64) -> bool {` — une réponse devancée s'ignore : chaque frappe demande (tâche 13)
- `app/ui/src/boite.rs:1085` — `decalage_minutes: natif::decalage_du_geste(),` — relier envoie le décalage du geste : il enverra celui de chaque idée (tâche 9)
- `app/ui/src/boite.rs:1324` — `let ecouteur = window_event_listener(leptos::ev::keydown, move |ev| {` — l'écoute de `Ctrl+Z`, à côté de laquelle Échap s'écoute (tâche 14)
- `app/ui/src/boite.rs:1461` — `<div class="comptes" aria-live="polite">` — la région qui annonce chaque réponse (tâche 13)
- `app/ui/src/natif.rs:96` — `pub(crate) fn decalage_a(le: i64) -> i32 {` — le décalage de l'instant d'une idée, et celui qui borne un jour (tâches 9 et 11)
- `app/ui/tests/encre_statique.rs:309` — `"set_timeout"` — I18 refuse les minuteries : une commande par frappe (tâche 13)
- `docs/80-glossaire.md:214` — `| **Bandeau** |` — la zone du haut devient l'en-tête de l'établi (tâche 17)
- `docs/50-interfaces-et-appareils.md:105` — `| Bandeau | Recherche ; état de synchronisation` — la ligne se renomme (tâche 17)
- `docs/50-interfaces-et-appareils.md:88` — `| Remonter d'une échelle | Échap | Retour |` — Échap ferme le volet de l'idée au lot 1 (tâche 17)
- `docs/31-persistance-et-synchronisation.md:198` — `` `requetes_enregistrees`, `imports`, `origines_declarees` et `photos` : vides `` — la forme d'une requête enregistrée s'y écrit (tâche 10)
- `docs/31-persistance-et-synchronisation.md:773` — `Index de recherche (texte, attributs, motifs harmoniques)` — ce que l'index lit au lot 1, et quand (tâche 10)
- `docs/31-persistance-et-synchronisation.md:1004` — `supprimer tous les dérivés, les reconstruire, comparer les empreintes canoniques` — le test 1 nomme l'empreinte d'une réponse (tâche 10)
- `docs/30-modele-de-domaine.md:633` — `pour ranger et chercher sans la` — se corrige : chercher par eux reste la piste (tâche 10)
- `docs/70-qualite-et-verification.md:124` — `| I13 | Les dérivés sont jetables |` — son test se décrit (tâche 10)
- `docs/70-qualite-et-verification.md:194` — `| 1, 3, 7, 8 | INC-1, lot 1 |` — CAP-06 s'inscrit à côté (tâche 10)
- `docs/40-architecture.md:738` — `fonds, sans ses morceaux` — ce que la vue du lot 1 laisse de CAP-07 (tâche 17)
- `docs/pistes.md:15` — `L'écart d'un jour au changement d'heure` — la piste sort (tâche 10)
- `docs/pistes.md:17` — `La zone du haut de l'écran du fonds` — la piste sort (tâche 17)
- `docs/assets/maquettes/fonds.svg:24` — `aujourd'hui 08:14 · carnet` — passe aux dates absolues (tâche 17)
- `.cargo/mutants.toml:2` — `Dix-sept modules sont examinés` — vingt et un (tâches 1 à 3, 9)

## Outils et cas limites

- `char::to_lowercase` — rend deux caractères pour `İ` (`i` puis U+0307) et `ß` pour `ẞ`, sans replier `œ` ni ôter un accent — `oe_trouve_oe_lie_et_l_inverse` (tâche 1)
- `str::split_whitespace` — coupe aussi sur U+00A0 et U+202F, quand `split(' ')` ne le fait pas — `les_mots_se_coupent_aux_blancs_et_perdent_la_ponctuation_des_bords` (tâche 1)
- `char::is_alphanumeric` — tient une lettre accentuée et un chiffre pour des lettres, l'apostrophe et le tiret non : rogner les bords ôte `'`, `-`, `«` et `.` — le même test (tâche 1)
- `cargo-mutants` 27.1.0 — ne mute pas une constante : la table de repli ne se garde que par son test exhaustif — `la_table_est_rangee_et_chaque_entree_se_replie` (tâche 1)
- `slice::binary_search_by_key` — suppose une table rangée, et rate en silence une entrée d'une table qui ne l'est pas — `la_table_est_rangee_et_chaque_entree_se_replie` (tâche 1), qui vérifie l'ordre strict
- un décalage local unique pour tout un jour — perd une heure le jour d'un changement d'heure, jours de 23 et de 25 heures — `les_bornes_d_un_jour_de_25_heures_sont_justes` et `les_bornes_d_un_jour_de_23_heures_sont_justes` (tâche 2)
- `Automerge::get_heads` — change à chaque modification, réception comprise, et non sur une écriture refusée — `i13_l_index_reconstruit_rend_les_memes_reponses` (tâche 5), qui joue une écriture refusée entre deux lectures
- `serde_json::to_vec` — écrit les champs d'une structure dans l'ordre de leur déclaration : deux réponses égales donnent les mêmes octets, que l'empreinte compare — `i13_l_index_reconstruit_rend_les_memes_reponses` (tâche 5)
- l'effacement d'une clé sous Automerge, concurrent d'une écriture dans l'objet qu'elle nomme — l'écriture ne fait pas renaître l'objet — `un_renommage_concurrent_d_une_suppression_ne_laisse_aucune_requete_sans_critere` (tâche 7)
- `KeyboardEvent.key` sous un clavier AZERTY — `/` vient de Maj + `:`, la touche dont `code` vaut `Slash` y donne `!`, et AltGr arrive comme Ctrl + Alt — `slash_porte_le_focus_hors_d_un_champ_seulement` (tâche 11)
- le focus posé pendant `keydown` — sans `preventDefault`, la frappe se poursuit dans le champ qui le reçoit — aucun test sans DOM ne le peut : l'essai installé (tâche 18) tape `/` hors d'un champ et lit la recherche vide
- `<input type="search">` sous WebView2 — Échap y vide le champ, action par défaut de `keydown`, et l'`input` qui suit relance la recherche, vide — `echap_ne_ferme_que_le_volet_ouvert` (tâche 11), et l'essai installé (tâche 18) : Échap dans la recherche, un volet ouvert puis aucun
- `<input type="date">` — rend `AAAA-MM-JJ`, ou une chaîne vide quand le champ est vide ou incomplet — `un_champ_de_date_vide_ne_borne_pas` (tâche 11)
- `aria-live="polite"` — annonce chaque changement de la région, une frappe après l'autre si chaque réponse la change — aucun test sans DOM ne le peut : l'essai installé (tâche 18), au lecteur d'écran
- les contrôles statiques du noyau et des services, en sous-chaîne — refusent `Instant`, `File`, `io::` ou `time::` jusque dans un commentaire en français — `statique.rs` du noyau et des services, verts après chaque tâche qui y écrit
- `git grep -n -i -w`, la recherche de LE-2 — `-i` ne replie pas les accents, et `-w` prend une lettre accentuée pour une borne de mot — aucun test ne le peut, limite que chaque PR dit : sa revue relit la recherche citée, faite aussi sans accent

## Effets cherchés

- La commande `boite` prend une requête et des bornes, et sa vue porte l'assiette de la requête, ses parts « sans » et les étiquettes du fonds — l'interface, et la tranche 7, qui chronomètre la recherche autour de cette seule commande — `la_requete_passe_au_noyau` et les aller-retours de `serialisation.rs` (tâche 4)
- `commandes::Assiette` gagne `illisibles` — chaque vue qui porte une assiette, le compte de la boîte et l'interface — `une_idee_illisible_compte_a_part_dans_l_assiette`, mis à jour (tâche 4)
- L'index tenu se rebâtit à la lecture qui suit un changement des têtes du document — la tranche 5 (un import de milliers d'opérations, l'index rebâti une fois à la lecture suivante), la tranche 6 (le document remplacé) et le lot 3 (les réceptions) — `i13_l_index_reconstruit_rend_les_memes_reponses` (tâche 5)
- La construction de l'index compte dans l'ouverture, la recherche n'en est qu'une lecture — la mesure de CAP-06 · 6 en tranche 7 — la PR (a) le dit, et la tranche 7 le reprend
- Une requête enregistrée vit sous `requetes_enregistrees/<ULID>`, une clé par sorte de critère, ses critères jamais modifiés — le schéma de la `v0.3.0`, que la clôture du lot gèle ; le miroir du lot 4, lisible en texte ; le carnet du lot 2 — `i01_une_requete_enregistree_ne_garde_ni_resultats_ni_nombre` et le test à deux répliques (tâche 7)
- Enregistrer, renommer et supprimer ne touchent pas ce qu'annuler vise — `Services`, le bouton « Annuler » et `Ctrl+Z` — `enregistrer_renommer_et_supprimer_une_requete` (tâche 7)
- Enregistrer est refusé tant que le fonds n'est pas né — la tranche 6, qui ne réimporte que dans un Pupitre neuf — `enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait` (tâche 7)
- `NOMS` gagne quatre noms — les droits de Windows, `generate_handler!` et chaque tranche suivante — les tests d'I31 (tâche 8)
- La commande `journal` prend une fenêtre et rend une `VueDuJournal`, que dit `empreinte_du_journal` — la tranche 6 (le test 14) et la tranche 8 (le contrôle d'installation), qui comparent deux journaux par cette fonction — les tests de `journal.rs` et `le_journal_se_lit_en_fenetre` (tâche 9)
- Chaque décalage d'une commande de geste est celui de l'instant de l'idée qu'il date, et `Relier` et `Regrouper` en portent deux — l'interface, et le carnet du lot 2, qui enverra les mêmes commandes — `le_message_date_chaque_idee_a_son_propre_decalage` et `relier_les_cochees` (tâche 9)
- La table de repli est au noyau, que l'interface compile en WebAssembly — le carnet du lot 2 (Android), où le même repli rendra les mêmes réponses — `la_table_est_rangee_et_chaque_entree_se_replie` (tâche 1)
- Quatre modules neufs du noyau sous mutation — le job `mutants`, son délai, et celui de `mutants-complet.yml` — le délai remesuré (tâches 10 et 17)
- L'établi s'ouvre sur le fonds — la tranche 7 (« jusqu'à la boîte d'idées affichée ») et l'essai de la tranche 8 — `l_etabli_s_ouvre_sur_le_fonds_et_quitter_la_table_ferme_sa_palette` (tâche 12)
- `/` et Échap s'écoutent sur la fenêtre, `/` hors d'un champ de texte, Échap partout sauf dans la recherche — chaque écran qui ajoutera un champ de texte — `slash_porte_le_focus_hors_d_un_champ_seulement` et `echap_ne_ferme_que_le_volet_ouvert` (tâche 11)
- « Sans nœud · aucune archive » est une constante de l'en-tête — la tranche 6, qui le lira dans `appareil.txt` — l'essai installé (tâche 18)
- `serde_json` entre en dépendance de test des services et du stockage — le `Cargo.lock` (les dépendances des paquets `services` et `stockage`), et les trois tests du stockage qui incluent la suite — l'étape « Cargo.lock à jour » de `verifier`, et le job `windows`, qui compile `commandes_sur_fichiers.rs`, `references.rs` et `repliques.rs` (tâche 4)

## Revues

- la revue de chaque tâche : Tâche 1, Tâche 2, Tâche 3, Tâche 4, Tâche 5, Tâche 6, Tâche 7, Tâche 8, Tâche 9, Tâche 10, Tâche 11, Tâche 12, Tâche 13, Tâche 14, Tâche 15, Tâche 16, Tâche 17
- la revue de la PR (a) entière, sur sa tête, le tableau du § 70 · 4.1 comparé aux critères de la tranche et chaque règle de LE-6 recopiée : Tâche 10
- la revue de la PR (b) entière, sur sa tête, LE-16 compris : Tâche 17
- la relecture des corrections de chaque revue de PR, sur la tête qui part, celle que nomme `--match-head-commit` : Tâche 10, Tâche 17
- la relecture du relevé de l'essai installé, sur la tête qui part : Tâche 18

---

## La carte des fichiers

```text
crates/noyau/src/donnees/
  repli.rs                         (a) TABLE, replier, mots, trouve                                    (muté)
  requete.rs                       (a) Jour, Bornes, debut_du_jour, bornes_de_la_periode, Etat,
                                   ProvenanceDeLIdee, Requete, Sans, texte_de_la_reponse,
                                   RequeteEnregistreeLue, enregistrer/renommer/supprimer_une_requete,
                                   lire_une_requete, ranger_les_requetes                               (muté)
  index.rs                         (a) Indexee, Index, Reponse                                         (muté)
  journal.rs                       (a) ranger_le_journal, heure_du_journal, empreinte_du_journal,
                                   empreinte_courte                                                    (muté)
  boite.rs                         (a) boite() et texte_du_filtre retirées (tâche 4)                   (muté)
  gestes.rs                        (a) Impossible : NomVide, MemeNom, RequeteSansCritere, RequeteAbsente,
                                   SeuilTropLong ; relier, delier, regrouper à deux décalages        (muté)
  fonds.rs                         (a) NOM, DU, AU, ETIQUETTE, PLUS_DE_SECONDES, ETAT, CONTENU, et leurs valeurs
  mod.rs                           (a) repli, requete, index, journal
crates/noyau/tests/repli.rs, requete.rs, index.rs, journal.rs    (a)
crates/noyau/tests/boite.rs, gestes.rs                           (a) les tests de boite() retirés ; relier à deux décalages
crates/commandes/src/lib.rs        (a) Jour, Bornes, Etat, ProvenanceDeLIdee, Requete, Sans, Boite, Assiette,
                                   VueDeLaBoite, RequeteEnregistree, Enregistrer, Renommer, Supprimer, VueDuJournal,
                                   Relier, Regrouper ; quatre noms
crates/commandes/tests/serialisation.rs   (a)
crates/services/Cargo.toml         (a) [dev-dependencies] serde_json
crates/stockage/Cargo.toml         (a) [dev-dependencies] serde_json
crates/services/src/lib.rs         (a) l'index tenu, boite, boite_sans_index, requetes_enregistrees, enregistrer,
                                   renommer, supprimer, journal_en_fenetre, les conversions
crates/services/src/lecture.rs     (a) requetes_enregistrees
crates/services/tests/suite/mod.rs (a) cap06_7 à 10, cap02_5 (part), i09, i13, i01, les requêtes enregistrées, le journal
crates/services/tests/commandes_en_memoire.rs, crates/stockage/tests/commandes_sur_fichiers.rs   (a) les tests nommés ;
                                   le test à deux répliques (mémoire seule)
app/src-tauri/capabilities/windows.json, src/lib.rs, src/commandes_de_l_etabli.rs   (a)
app/ui/src/recherche.rs            (b) EtatDeLaRequete, DUREES, ETATS, PROVENANCES, ID_DE_LA_RECHERCHE, est_la_recherche, jour_du_champ,
                                   bornes_de_la_requete, slash_cherche, echap_ferme
app/ui/src/app.rs                  (b) Vue { Fonds, Table }, l'état de la requête, l'écoute de `/`
app/ui/src/entete.rs               (b) la recherche, l'état de synchronisation, « Afficher »
app/ui/src/boite.rs                (a) la requête dans la commande, les décalages ; (b) les critères, Échap, le fonds
                                   vide, les requêtes enregistrées, le journal
app/ui/src/main.rs                 (b) mod recherche
app/ui/style.css                   (b) l'en-tête, les critères, le fonds vide, le journal
docs/…                             (a) tâche 10 ; (b) tâche 17
.cargo/mutants.toml                (a) repli, requete, index, journal
.github/workflows/ci.yml, mutants-complet.yml   (a), (b) délais
Cargo.lock                         (a) celui de l'artefact `cargo-lock`, relu
CONVENTIONS.md, CLAUDE.md          (a), (b)
```

## PR (a) · les requêtes, l'index, les requêtes enregistrées, le journal

**Branche :** `t4a-les-requetes`. **Titre :** `feat: les requêtes du fonds, leur index, les requêtes enregistrées et le
journal à fenêtre` (90). **Documents :** CAP-02, CAP-06 ; § 30 · 9.2 et 11 ; § 31 · 2, 2.3, 7 et 10 ; § 40 · 7.5, relu ; § 41 · 6 ;
§ 70 · 3, 4.1 et 8 ; le glossaire ; `docs/pistes.md` ; `.cargo/mutants.toml` ; `CONVENTIONS.md` ; `CLAUDE.md`. **Fin :**
la tâche 10.

### Tâche 1 : le repli et les mots, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/repli.rs`, `crates/noyau/tests/repli.rs` ; modifier
`crates/noyau/src/donnees/mod.rs`, `.cargo/mutants.toml` (`"crates/noyau/src/donnees/repli.rs"` dans
`examine_globs`, `"--test", "repli"` dans `additional_cargo_test_args`, le compte des modules à `:2` et `:27`).

**Interfaces — produit** (`noyau::donnees::repli`) :

```rust
/// La table de repli (question 1), rangée par caractère, chaque caractère une fois : chaque lettre
/// minuscule de Latin-1 (U+00DF à U+00FF, sans U+00F7) et de Latin étendu A (U+0100 à U+017F) vers sa
/// base, ou sa récriture (œ → oe, æ → ae, ß → ss, þ → th, ĳ → ij, ŉ → 'n) ; les ligatures U+FB00 à
/// U+FB04 ; les apostrophes vers `'`, les tirets vers `-` ; les invisibles vers rien.
pub const TABLE: &[(char, &str)];
/// Le texte replié : chaque caractère en minuscules (`char::to_lowercase`), puis, de chaque
/// caractère obtenu, rien pour une marque combinante (U+0300 à U+036F), sa récriture s'il est dans
/// `TABLE` (`binary_search_by_key`), lui-même sinon.
pub fn replier(texte: &str) -> String;
/// Les mots cherchés : la requête repliée, coupée aux blancs (`split_whitespace`), chaque morceau
/// rogné de ce qui n'est ni lettre ni chiffre à ses bords (`char::is_alphanumeric`), les morceaux
/// vides écartés.
pub fn mots(requete: &str) -> Vec<String>;
/// Chaque mot de `mots` paraît, entier ou en fragment, dans `texte_replie` (`str::contains`) ;
/// vrai d'une liste vide.
pub fn trouve(texte_replie: &str, mots: &[String]) -> bool;
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/repli.rs`). `ATTENDUES` est la table, recopiée dans le
  test (cent vingt entrées, calculées au passage de LE-6 d'après la décomposition d'Unicode) :

```rust
const ATTENDUES: &[(char, &str)] = &[
    ('\u{0060}', "'"), ('\u{00ad}', ""), ('\u{00b4}', "'"), ('ß', "ss"), ('à', "a"), ('á', "a"), ('â', "a"),
    ('ã', "a"), ('ä', "a"), ('å', "a"), ('æ', "ae"), ('ç', "c"), ('è', "e"), ('é', "e"), ('ê', "e"),
    ('ë', "e"), ('ì', "i"), ('í', "i"), ('î', "i"), ('ï', "i"), ('ð', "d"), ('ñ', "n"), ('ò', "o"), ('ó', "o"),
    ('ô', "o"), ('õ', "o"), ('ö', "o"), ('ø', "o"), ('ù', "u"), ('ú', "u"), ('û', "u"), ('ü', "u"), ('ý', "y"),
    ('þ', "th"), ('ÿ', "y"), ('ā', "a"), ('ă', "a"), ('ą', "a"), ('ć', "c"), ('ĉ', "c"), ('ċ', "c"),
    ('č', "c"), ('ď', "d"), ('đ', "d"), ('ē', "e"), ('ĕ', "e"), ('ė', "e"), ('ę', "e"), ('ě', "e"), ('ĝ', "g"),
    ('ğ', "g"), ('ġ', "g"), ('ģ', "g"), ('ĥ', "h"), ('ħ', "h"), ('ĩ', "i"), ('ī', "i"), ('ĭ', "i"), ('į', "i"),
    ('ı', "i"), ('ĳ', "ij"), ('ĵ', "j"), ('ķ', "k"), ('ĸ', "k"), ('ĺ', "l"), ('ļ', "l"), ('ľ', "l"),
    ('ŀ', "l"), ('ł', "l"), ('ń', "n"), ('ņ', "n"), ('ň', "n"), ('ŉ', "'n"), ('ŋ', "n"), ('ō', "o"),
    ('ŏ', "o"), ('ő', "o"), ('œ', "oe"), ('ŕ', "r"), ('ŗ', "r"), ('ř', "r"), ('ś', "s"), ('ŝ', "s"),
    ('ş', "s"), ('š', "s"), ('ţ', "t"), ('ť', "t"), ('ŧ', "t"), ('ũ', "u"), ('ū', "u"), ('ŭ', "u"), ('ů', "u"),
    ('ű', "u"), ('ų', "u"), ('ŵ', "w"), ('ŷ', "y"), ('ź', "z"), ('ż', "z"), ('ž', "z"), ('ſ', "s"),
    ('\u{02bc}', "'"), ('\u{200b}', ""), ('\u{200c}', ""), ('\u{200d}', ""), ('\u{2010}', "-"),
    ('\u{2011}', "-"), ('\u{2012}', "-"), ('\u{2013}', "-"), ('\u{2014}', "-"), ('\u{2018}', "'"),
    ('\u{2019}', "'"), ('\u{2032}', "'"), ('\u{2060}', ""), ('\u{2212}', "-"), ('\u{fb00}', "ff"),
    ('\u{fb01}', "fi"), ('\u{fb02}', "fl"), ('\u{fb03}', "ffi"), ('\u{fb04}', "ffl"), ('\u{feff}', ""),
];

#[test]
fn la_table_est_rangee_et_chaque_entree_se_replie() {
    assert!(TABLE.windows(2).all(|paire| paire[0].0 < paire[1].0));
    assert_eq!(TABLE, ATTENDUES);
    for (caractere, attendu) in ATTENDUES {
        assert_eq!(replier(&caractere.to_string()), *attendu, "{caractere:?}");
    }
}

#[test]
fn eglise_trouve_eglise_composee_ou_decomposee() {
    assert_eq!(replier("Église du port"), "eglise du port"); // É, U+00C9
    assert_eq!(replier("E\u{301}glise du port"), "eglise du port"); // E, puis U+0301
    assert!(trouve(&replier("EGLISE"), &mots("église")));
    assert!(trouve(&replier("église"), &mots("ÉGLISE")));
    assert!(trouve(&replier("Eglise"), &mots("église")));
}

#[test]
fn oe_trouve_oe_lie_et_l_inverse() {
    assert!(trouve(&replier("cœur"), &mots("oe")));
    assert!(trouve(&replier("CŒUR"), &mots("coeur")));
    assert!(trouve(&replier("coeur"), &mots("œ")));
    assert_eq!(replier("Straẞe"), "strasse");
    assert_eq!(replier("İzmir"), "izmir");
    assert_eq!(replier("Łódź"), "lodz");
    assert_eq!(replier("\u{fb01}n"), "fin");
}

#[test]
fn les_apostrophes_et_les_tirets_se_replient_dans_un_mot() {
    assert_eq!(mots("l'eglise"), ["l'eglise"]);
    assert!(trouve(&replier("l’Église"), &mots("l'eglise")));
    assert!(!trouve(&replier("la vieille église"), &mots("l'eglise")));
    assert!(trouve(&replier("aujourd’hui"), &mots("aujourd'hui")));
    assert!(trouve(&replier("arc\u{2011}en\u{2011}ciel"), &mots("arc-en-ciel")));
    assert!(trouve(&replier("arc-en-ciel"), &mots("arc en ciel")));
    assert!(!trouve(&replier("arc en ciel"), &mots("arc-en-ciel")));
    assert!(trouve(&replier("IV\u{2013}iv\u{2013}I"), &mots("IV-iv-I")));
}

#[test]
fn les_mots_se_coupent_aux_blancs_et_perdent_la_ponctuation_des_bords() {
    assert_eq!(mots("mi\u{202f}mineur"), ["mi", "mineur"]);
    assert_eq!(mots("mi\u{a0}mineur"), ["mi", "mineur"]);
    assert_eq!(mots("port."), ["port"]);
    assert_eq!(mots("«eglise»"), ["eglise"]);
    assert_eq!(mots("1/2 IV/V"), ["1/2", "iv/v"]);
    assert_eq!(mots(""), Vec::<String>::new());
    assert_eq!(mots("  \u{a0} "), Vec::<String>::new());
    assert_eq!(mots("' - « »"), Vec::<String>::new());
}

#[test]
fn les_invisibles_s_otent_et_les_formes_de_compatibilite_restent() {
    assert!(trouve(&replier("égl\u{ad}ise"), &mots("eglise")));
    assert!(trouve(&replier("égl\u{200b}ise"), &mots("eglise")));
    assert!(!trouve(&replier("1ᵉʳ couplet"), &mots("1er")));
    assert!(!trouve(&replier("x²"), &mots("x2")));
}

#[test]
fn un_fragment_suffit_sans_longueur_minimale() {
    assert!(trouve(&replier("à 120 bpm"), &mots("12")));
    assert!(trouve(&replier("Noël"), &mots("ë")));
    assert!(trouve(&replier("cœur"), &mots("coe")));
    assert!(trouve(&replier("un texte"), &[]));
    assert!(!trouve(&replier("un texte"), &mots("pont")));
}
```

- [ ] **Étape 2 : voir le rouge.** Committer (`test: le repli et les mots cherchés, au noyau`), pousser, ouvrir la PR
  en brouillon. Attendu à « Lint » de `verifier` : `unresolved import noyau::donnees::repli`.
- [ ] **Étape 3 : écrire le code.** `TABLE` en constante, la recherche par `binary_search_by_key` ; aucune boucle
  `while` écrite à la main (un mutant sans fin fait échouer la porte, `mutants.toml:55-64`) ; la doc dit que le repli ne
  couvre que ce que couvre la table : un grec ou un cyrillique n'est replié qu'en casse.
- [ ] **Étape 4 : voir le vert.** Attendu à « Tests » de `verifier` : `repli.rs` `ok. 7 passed`, `statique.rs` du noyau
  vert ; `mutants` sans survivant sur le diff.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/repli.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/repli.rs .cargo/mutants.toml` (`feat: la table de repli et les mots cherchés, au noyau`).

### Tâche 2 : les jours civils et les bornes d'une période, au noyau

**Fichiers :** créer `crates/noyau/src/donnees/requete.rs`, `crates/noyau/tests/requete.rs` ; modifier
`crates/noyau/src/donnees/mod.rs`, `.cargo/mutants.toml` (`requete.rs`, `"--test", "requete"`, le compte).

**Interfaces — consomme :** `date_civile` (`gestes.rs:243`), pour l'aller du jour vers le texte.

**Interfaces — produit** (`noyau::donnees::requete`) :

```rust
/// Un jour civil (question 9) : l'année, le mois de 1 à 12, le jour de 1 à 31.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jour { pub annee: i32, pub mois: u8, pub jour: u8 }
impl Jour {
    /// « AAAA-MM-JJ », la forme d'un champ de date et du document : quatre chiffres, deux, deux ; un
    /// jour qui n'existe pas (le 30 février, un mois 13), ou toute autre forme : `None`.
    pub fn depuis_iso(texte: &str) -> Option<Jour>;
    /// « AAAA-MM-JJ ».
    pub fn en_iso(self) -> String;
    /// Minuit UTC de ce jour, en secondes depuis 1970 (« days from civil », H. Hinnant).
    pub fn minuit_utc(self) -> i64;
}
/// Les bornes d'une date, en secondes : `des` comprise, `avant` exclue ; une borne absente ne borne pas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bornes { pub des: Option<i64>, pub avant: Option<i64> }
impl Bornes {
    /// `heure` est dans les bornes.
    pub fn contient(self, heure: i64) -> bool;
}
/// Le début du jour civil `jour` à l'heure locale que dit `decalage` (en minutes, pour un instant) :
/// `m` le minuit UTC du jour, `d = m − decalage(m) × 60`, puis `m − decalage(d) × 60`.
pub fn debut_du_jour(jour: Jour, decalage: impl Fn(i64) -> i32) -> i64;
/// La période « du … au … », bornes comprises (question 9) : `des`, le début du jour `du` ; `avant`,
/// le début du jour qui suit `au` (son minuit UTC plus 86 400, puis le même calcul).
pub fn bornes_de_la_periode(du: Option<Jour>, au: Option<Jour>, decalage: impl Fn(i64) -> i32) -> Bornes;
```

- [ ] **Étape 1 : écrire les tests.** `j(texte)` vaut `Jour::depuis_iso(texte).unwrap()`. `paris` rend le décalage de
  Paris en 2026, écrit à la main (l'heure d'été du 29 mars à 1 h UTC au 25 octobre à 1 h UTC) :

```rust
fn paris(t: i64) -> i32 {
    if (1_774_746_000..1_792_890_000).contains(&t) { 120 } else { 60 }
}

#[test]
fn un_jour_se_lit_et_s_ecrit_en_iso() {
    assert_eq!(Jour::depuis_iso("2026-10-25"), Some(Jour { annee: 2026, mois: 10, jour: 25 }));
    assert_eq!(j("2024-02-29").en_iso(), "2024-02-29");
    assert!(Jour::depuis_iso("2000-02-29").is_some());
    for faux in ["2026-02-29", "1900-02-29", "2026-13-01", "2026-00-10", "2026-04-31", "2026-10-00",
        "2026-4-1", "26-10-25", "2026-10-25 ", "2026/10/25", ""] {
        assert_eq!(Jour::depuis_iso(faux), None, "{faux}");
    }
}

#[test]
fn minuit_utc_compte_les_jours_depuis_1970() {
    assert_eq!(j("1970-01-01").minuit_utc(), 0);
    assert_eq!(j("1969-12-31").minuit_utc(), -86_400);
    assert_eq!(j("2000-02-29").minuit_utc(), 951_782_400);
    assert_eq!(j("2000-03-01").minuit_utc(), 951_868_800);
    assert_eq!(j("2026-10-25").minuit_utc(), 1_792_886_400);
    assert_eq!(j("2027-01-01").minuit_utc(), 1_798_761_600);
}

#[test]
fn les_bornes_d_un_jour_de_25_heures_sont_justes() {
    // Le 25 octobre 2026 à Paris : de 0 h, heure d'été (24 octobre, 22 h UTC), à 0 h le 26, heure
    // d'hiver (25 octobre, 23 h UTC) : 25 heures.
    let b = bornes_de_la_periode(Some(j("2026-10-25")), Some(j("2026-10-25")), paris);
    assert_eq!(b, Bornes { des: Some(1_792_879_200), avant: Some(1_792_969_200) });
    // A, notée à 0 h 30, heure d'été ; B, à 23 h 30, heure d'hiver : du 25, comme la liste les date ;
    // C, le 26 à 0 h 30 : non. Un seul décalage pour le jour entier perdrait A ou B (LE-6, L7).
    assert!(b.contient(1_792_881_000));
    assert!(b.contient(1_792_967_400));
    assert!(!b.contient(1_792_971_000));
}

#[test]
fn les_bornes_d_un_jour_de_23_heures_sont_justes() {
    let b = bornes_de_la_periode(Some(j("2026-03-29")), Some(j("2026-03-29")), paris);
    assert_eq!(b, Bornes { des: Some(1_774_738_800), avant: Some(1_774_821_600) });
}

#[test]
fn une_borne_absente_ne_borne_pas() {
    assert_eq!(bornes_de_la_periode(None, None, paris), Bornes::default());
    let des_le_25 = bornes_de_la_periode(Some(j("2026-10-25")), None, paris);
    assert_eq!(des_le_25, Bornes { des: Some(1_792_879_200), avant: None });
    assert!(des_le_25.contient(i64::MAX));
    assert!(!des_le_25.contient(1_792_879_199));
    assert!(Bornes::default().contient(i64::MIN));
}
```

- [ ] **Étape 2 : voir le rouge** à « Lint » (`unresolved import noyau::donnees::requete`), committer (`test: les jours
  civils et les bornes d'une période, au noyau`).
- [ ] **Étape 3 : écrire le code.** `minuit_utc` : le calcul inverse de `date_civile`, sur des entiers, dans une ère de
  400 ans ; `depuis_iso` vérifie le jour par le nombre de jours du mois (février : 29 les années divisibles par 4, sauf
  les siècles non divisibles par 400) ; `en_iso` écrit l'année sur quatre chiffres.
- [ ] **Étape 4 : voir le vert.** `requete.rs` `ok. 5 passed` ; `mutants` sans survivant.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/requete.rs crates/noyau/src/donnees/mod.rs
  crates/noyau/tests/requete.rs .cargo/mutants.toml` (`feat: les jours civils et les bornes d'une période, au noyau`).

### Tâche 3 : les critères, l'index et la réponse, au noyau

**Fichiers :** modifier `crates/noyau/src/donnees/requete.rs` ; créer `crates/noyau/src/donnees/index.rs`,
`crates/noyau/tests/index.rs` ; modifier `crates/noyau/src/donnees/mod.rs`, `.cargo/mutants.toml` (`index.rs`,
`"--test", "index"`, le compte).

**Interfaces — consomme :** `replier`, `mots`, `trouve` (tâche 1) ; `Jour`, `Bornes` (tâche 2) ; `EnBoite`, `Contenu`,
`Ordre`, `ordonner` (`boite.rs`) ; `Assiette` (`idee.rs`) ; `Provenance` (`gestes.rs`).

**Interfaces — produit** (`noyau::donnees::requete`, puis `noyau::donnees::index`) :

```rust
/// L'état d'une idée, pour une requête : reliée ou non.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat { Reliee, NonReliee }
/// La provenance d'une idée (§ 30 · 8.1, question 11) : captée ou importée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceDeLIdee { Captee, Importee }
/// Une requête (CAP-06) : une clé par sorte de critère, chacune facultative ; tous posés, ils se
/// combinent par « et » (question 7). `texte` : le texte saisi, tel quel ; ses mots (`repli::mots`) sont
/// les fragments cherchés, et sans mot, il ne pose aucun critère. `du`, `au` : la période, que la réponse juge sur les bornes qu'on lui donne.
/// `plus_de_secondes` : la durée affichée, en secondes entières, strictement plus grande (question 8).
/// `etiquette` : comparée octet pour octet (question 10).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requete {
    pub texte: String, pub du: Option<Jour>, pub au: Option<Jour>, pub etiquette: Option<String>,
    pub plus_de_secondes: Option<u64>, pub etat: Option<Etat>, pub provenance: Option<ProvenanceDeLIdee>,
    pub contenu: Option<Contenu>,
}
impl Requete {
    /// Aucun critère : ni mot, ni jour, ni étiquette, ni seuil, ni état, ni provenance, ni contenu.
    pub fn est_vide(&self) -> bool;
}
/// Les parts « sans … » d'une réponse (question 7) : `Some` pour chaque critère posé dont une idée
/// peut manquer l'attribut (l'étiquette, la durée, le texte), le nombre d'idées lisibles qui le
/// manquent et satisfont chacun des autres critères qu'elles portent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sans { pub etiquette: Option<usize>, pub duree: Option<usize>, pub texte: Option<usize> }
/// « N idée(s) », puis « · K sans étiquette », « · K sans durée », « · K sans texte », chaque part
/// non nulle, dans cet ordre, puis « · K illisible(s) » si K > 0, puis « · M interrogée(s) »,
/// M = `assiette.considerees` ; singulier pour 0 et 1 (question 7).
pub fn texte_de_la_reponse(retenues: usize, assiette: Assiette, illisibles: usize, sans: Sans) -> String;

// noyau::donnees::index
/// Une idée de l'index : l'idée telle que la boîte la reçoit, et son texte replié.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexee { pub en_boite: EnBoite, pub texte_replie: Option<String> }
/// L'index des idées interrogées : un dérivé jetable (I13), bâti du fonds lu, sans rien écrire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index { /* les idées non retirées, indexées ; le nombre des illisibles */ }
/// La réponse d'une requête : le compte des non reliées et son assiette, qui ne suivent pas la
/// requête ; l'assiette de la requête (considérées : les lisibles et les illisibles ; portant et
/// décidées : les lisibles qui portent chaque attribut que la requête interroge), ses illisibles, ses
/// parts « sans », et les idées trouvées, rangées par `ordonner`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reponse {
    pub non_reliees: usize, pub assiette_du_compte: Assiette, pub assiette: Assiette,
    pub illisibles: usize, pub sans: Sans, pub trouvees: Vec<EnBoite>,
}
impl Index {
    /// L'index de `idees`, les retirées écartées (CAP-06), et de `illisibles` idées qui ne se lisent pas.
    pub fn construire(idees: Vec<EnBoite>, illisibles: usize) -> Index;
    /// La réponse de `requete`, la date jugée sur `bornes`, les trouvées rangées par `ordre`.
    pub fn repondre(&self, requete: &Requete, bornes: Bornes, ordre: Ordre) -> Reponse;
    /// Les étiquettes des idées de l'index, chacune une fois, dans l'ordre de leurs octets.
    pub fn etiquettes(&self) -> Vec<String>;
}
```

Les règles de `repondre`, critère par critère : la date, toujours portée, satisfaite si `bornes.contient(le)` ;
l'étiquette, portée par une idée qui en a une, satisfaite si elle la porte, octet pour octet ; la durée, portée par une
idée qui a une durée, satisfaite si `duree_ms / 1 000 > plus_de_secondes` ; l'état, la provenance et le contenu,
toujours portés ; le texte, sans mot aucun critère, sinon porté par une idée qui a un texte, satisfait si `trouve`.
Une idée est trouvée si elle porte chaque attribut interrogé et satisfait chaque critère ; elle compte dans une part
« sans » si elle manque cet attribut et satisfait chaque autre critère qu'elle porte (une idée qui en manque deux
compte dans les deux, LE-6, L5).

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/index.rs`). `idee(n)` rend l'idée de test `n` : captée,
  `le` valant `T0 + 60 × n` (`T0 = 1_793_000_000`), identifiant `01JIDEE` suivi de `n` sur deux chiffres, sans texte,
  étiquette, prise ni lien. Des fonctions de test la complètent : `avec_texte`, `avec_etiquettes`, `avec_prises` (les
  durées des prises, `None` pour une prise sans durée lue ; la durée par `idee::duree` ; le contenu `Prise`),
  `importee`, `retiree_a`, `le_a`. `reponse(idees, illisibles, &requete)` rend le texte de la réponse
  (`texte_de_la_reponse`) et les identifiants trouvés, par `Ordre::Date` et `Bornes::default()`. Les jeux sont ceux du
  passage de LE-6 (`requetes.py`), le premier refait sans valeur au seuil :

```rust
fn plus_d_une_minute() -> Requete { Requete { plus_de_secondes: Some(60), ..Requete::default() } }
fn cherche(texte: &str) -> Requete { Requete { texte: texte.to_string(), ..Requete::default() } }

/// Douze idées : 1 à 3 sans prise ; 4 (61 000 ms), 5 (90 000 et 12 000), 6 (600 000), 7 (75 000) ;
/// 8 (60 999, affichée « 1:00 »), 9 (59 999), 10 (30 000), 11 (5 000 et 20 000), 12 (999).
fn jeu_de_cap06_7() -> Vec<EnBoite>;

#[test]
fn plus_d_une_minute_se_juge_a_la_seconde_affichee() {
    let index = Index::construire(jeu_de_cap06_7(), 0);
    let r = index.repondre(&plus_d_une_minute(), Bornes::default(), Ordre::Date);
    assert_eq!(r.assiette, Assiette { considerees: 12, portant: 9, decidees: 9 });
    assert_eq!(r.sans, Sans { duree: Some(3), ..Sans::default() });
    assert_eq!(reponse(jeu_de_cap06_7(), 0, &plus_d_une_minute()),
        ("4 idées · 3 sans durée · 12 interrogées".to_string(),
         ids(["07", "06", "05", "04"])));
    // 60 400 ms s'affiche « 1:00 » : elle n'est pas de plus d'une minute ; 61 000, « 1:01 », l'est.
    let seuil = vec![avec_prises(idee(20), &[Some(60_400)]), avec_prises(idee(21), &[Some(61_000)])];
    assert_eq!(reponse(seuil, 0, &plus_d_une_minute()).1, ids(["21"]));
}

#[test]
fn une_prise_sans_duree_compte_parmi_les_sans_duree() {
    // CAP-06 · 9 : 13, une prise de 30 s et une sans durée ; 14, une de 90 s et une sans durée.
    let mut jeu = jeu_de_cap06_7();
    jeu.push(avec_prises(idee(13), &[Some(30_000), None]));
    jeu.push(avec_prises(idee(14), &[Some(90_000), None]));
    assert_eq!(reponse(jeu, 0, &plus_d_une_minute()).0, "4 idées · 5 sans durée · 14 interrogées");
}

#[test]
fn une_idee_illisible_se_compte_a_part() {
    let index = Index::construire(jeu_de_cap06_7(), 1);
    let r = index.repondre(&plus_d_une_minute(), Bornes::default(), Ordre::Date);
    assert_eq!(r.assiette, Assiette { considerees: 13, portant: 9, decidees: 9 });
    assert_eq!(r.illisibles, 1);
    assert_eq!(reponse(jeu_de_cap06_7(), 1, &plus_d_une_minute()).0,
        "4 idées · 3 sans durée · 1 illisible · 13 interrogées");
}

#[test]
fn une_idee_retiree_n_est_ni_trouvee_ni_comptee() {
    // 21 « Église du port » ; 22 « une église retirée », retirée ; 23 « le phare et la jetée » ; 24 et
    // 25 sans texte ; 26 « … », un texte sans mot ; 27 « L’EGLISE, encore ».
    let r = reponse(jeu_des_textes(), 0, &cherche("eglise"));
    assert_eq!(r, ("2 idées · 2 sans texte · 6 interrogées".to_string(), ids(["27", "21"])));
}

#[test]
fn deux_criteres_se_combinent_et_les_sans_se_comptent_parmi_les_candidates() {
    // 31 « Église du port », 90 s ; 32 « Église du port » ; 33 « le phare », 90 s ; 34, 90 s ;
    // 35, rien ; 36 « église, nuit », 30 s et une prise sans durée ; 37 « phare ».
    let requete = Requete { plus_de_secondes: Some(60), ..cherche("eglise") };
    let index = Index::construire(jeu_mixte(), 0);
    let r = index.repondre(&requete, Bornes::default(), Ordre::Date);
    assert_eq!(r.assiette, Assiette { considerees: 7, portant: 2, decidees: 2 });
    assert_eq!(r.sans, Sans { duree: Some(3), texte: Some(2), etiquette: None });
    assert_eq!(reponse(jeu_mixte(), 0, &requete),
        ("1 idée · 3 sans durée · 2 sans texte · 7 interrogées".to_string(), ids(["31"])));
}

#[test]
fn l_etiquette_se_compare_octet_pour_octet_et_le_texte_ne_la_lit_pas() {
    // 41 pont ; 42 Pont ; 43 ponts et nuit ; 44 pont-levis ; 45 #pont ; 46, aucune ; 47 Pönt.
    let pont = Requete { etiquette: Some("pont".to_string()), ..Requete::default() };
    assert_eq!(reponse(jeu_des_etiquettes(), 0, &pont),
        ("1 idée · 1 sans étiquette · 7 interrogées".to_string(), ids(["41"])));
    assert_eq!(reponse(jeu_des_etiquettes(), 0, &cherche("pont")).0, "0 idée · 7 sans texte · 7 interrogées");
    assert_eq!(Index::construire(jeu_des_etiquettes(), 0).etiquettes(),
        ["#pont", "Pont", "Pönt", "nuit", "pont", "pont-levis", "ponts"]);
}

#[test]
fn la_provenance_l_etat_et_le_contenu_n_ont_pas_de_part_sans() {
    // 51 captée ; 52 importée ; une illisible.
    let importees = Requete { provenance: Some(ProvenanceDeLIdee::Importee), ..Requete::default() };
    assert_eq!(reponse(vec![idee(51), importee(idee(52))], 1, &importees),
        ("1 idée · 1 illisible · 3 interrogées".to_string(), ids(["52"])));
}

#[test]
fn la_date_se_juge_sur_les_bornes() {
    // A (0 h 30, heure d'été), B (23 h 30, heure d'hiver), C (le 26, 0 h 30), et les bornes du
    // 25 octobre 2026 à Paris (tâche 2).
    let jeu = vec![le_a(idee(71), 1_792_881_000), le_a(idee(72), 1_792_967_400), le_a(idee(73), 1_792_971_000)];
    let bornes = Bornes { des: Some(1_792_879_200), avant: Some(1_792_969_200) };
    let requete = Requete { du: Jour::depuis_iso("2026-10-25"), au: Jour::depuis_iso("2026-10-25"), ..Requete::default() };
    let r = Index::construire(jeu, 0).repondre(&requete, bornes, Ordre::Date);
    assert_eq!(ids_de(&r.trouvees), ids(["72", "71"]));
    assert_eq!(texte_de_la_reponse(r.trouvees.len(), r.assiette, r.illisibles, r.sans), "2 idées · 3 interrogées");
}

#[test]
fn l_ordre_est_total_et_jamais_la_pertinence() {
    // 61 « eglise a » et 62 « eglise b », de la même seconde ; 63 « eglise c », une seconde après.
    assert_eq!(reponse(jeu_d_une_seconde(), 0, &cherche("eglise")).1, ids(["63", "61", "62"]));
}

#[test]
fn une_requete_sans_mot_ne_pose_aucun_critere() {
    for vide in ["", "  ", "'", " « » "] {
        assert!(cherche(vide).est_vide(), "{vide:?}");
        assert_eq!(reponse(jeu_des_textes(), 0, &cherche(vide)), reponse(jeu_des_textes(), 0, &Requete::default()));
    }
    assert_eq!(reponse(jeu_des_textes(), 0, &Requete::default()).0, "6 idées · 6 interrogées");
    assert!(!Requete { contenu: Some(Contenu::Photo), ..Requete::default() }.est_vide());
}

#[test]
fn sans_critere_l_index_rend_la_boite() {
    // Le jeu de `crates/noyau/tests/boite.rs` (`jeu()`, six idées, F retirée), recopié : les ordres,
    // les contenus et le compte de la tranche 3, inchangés.
    let index = Index::construire(jeu_de_la_boite(), 0);
    let par = |ordre| ids_de(&index.repondre(&Requete::default(), Bornes::default(), ordre).trouvees);
    assert_eq!(par(Ordre::Date), ["01B", "01D", "01C", "01E", "01A"]);
    assert_eq!(par(Ordre::Etiquette), ["01C", "01E", "01A", "01D", "01B"]);
    assert_eq!(par(Ordre::Duree), ["01D", "01A", "01C", "01B", "01E"]);
    assert_eq!(par(Ordre::Reliee), ["01C", "01E", "01A", "01B", "01D"]);
    assert_eq!(par(Ordre::Provenance), ["01D", "01E", "01A", "01B", "01C"]);
    let contenu = |c| ids_de(&index.repondre(&Requete { contenu: Some(c), ..Requete::default() },
        Bornes::default(), Ordre::Date).trouvees);
    assert_eq!(contenu(Contenu::Prise), ["01B", "01D", "01C", "01A"]);
    assert_eq!(contenu(Contenu::Photo), Vec::<&str>::new());
    assert_eq!(contenu(Contenu::MidiVerse), ["01B"]);
    assert_eq!(contenu(Contenu::TexteVerse), ["01C"]);
    assert_eq!(contenu(Contenu::TexteDeLIdee), ["01D", "01E", "01A"]);
    for requete in [Requete::default(), Requete { contenu: Some(Contenu::Prise), ..Requete::default() },
        Requete { contenu: Some(Contenu::Photo), ..Requete::default() }] {
        let r = index.repondre(&requete, Bornes::default(), Ordre::Date);
        assert_eq!(r.non_reliees, 3); // A, C, E ; F, retirée, n'y compte pas
        assert_eq!(r.assiette_du_compte, Assiette { considerees: 5, portant: 5, decidees: 5 });
    }
    let r = Index::construire(jeu_de_la_boite(), 1).repondre(&Requete::default(), Bornes::default(), Ordre::Date);
    assert_eq!(r.non_reliees, 3);
    assert_eq!(r.assiette_du_compte, Assiette { considerees: 6, portant: 5, decidees: 5 });
}
```

  Dans `crates/noyau/tests/requete.rs`, `le_texte_de_la_reponse_ordonne_ses_parts_et_s_accorde` :

```rust
#[test]
fn le_texte_de_la_reponse_ordonne_ses_parts_et_s_accorde() {
    let a = |c, p| Assiette { considerees: c, portant: p, decidees: p };
    assert_eq!(texte_de_la_reponse(0, a(4, 4), 0, Sans::default()), "0 idée · 4 interrogées");
    assert_eq!(texte_de_la_reponse(1, a(1, 1), 0, Sans::default()), "1 idée · 1 interrogée");
    assert_eq!(texte_de_la_reponse(2, a(5, 4), 1, Sans::default()), "2 idées · 1 illisible · 5 interrogées");
    let toutes = Sans { etiquette: Some(1), duree: Some(1), texte: Some(2) };
    assert_eq!(texte_de_la_reponse(1, a(6, 1), 1, toutes),
        "1 idée · 1 sans étiquette · 1 sans durée · 2 sans texte · 1 illisible · 6 interrogées");
    assert_eq!(texte_de_la_reponse(2, a(5, 3), 2, Sans { duree: Some(0), ..Sans::default() }),
        "2 idées · 2 illisibles · 5 interrogées");
}
```

- [ ] **Étape 2 : voir le rouge** à « Lint » (`unresolved import noyau::donnees::index`), committer (`test: les
  critères, l'index et la réponse d'une requête, au noyau`).
- [ ] **Étape 3 : écrire le code.** `construire` replie chaque texte une fois ; `repondre` parcourt l'index, sans index
  inversé (décision 17 ; LE-6, règle 14), puis `ordonner` ; `etiquettes` par un `BTreeSet`. `boite()` et
  `texte_du_filtre` restent jusqu'à la tâche 4, qui les retire.
- [ ] **Étape 4 : voir le vert.** `index.rs` `ok. 11 passed`, `requete.rs` `ok. 6 passed` ; `mutants` sans survivant.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/requete.rs crates/noyau/src/donnees/index.rs
  crates/noyau/src/donnees/mod.rs crates/noyau/tests/index.rs crates/noyau/tests/requete.rs .cargo/mutants.toml`
  (`feat: les critères, l'index et la réponse d'une requête, au noyau`).

### Tâche 4 : la requête dans les commandes et les services ; l'index tenu

**Fichiers :** modifier `crates/commandes/src/lib.rs`, `crates/commandes/tests/serialisation.rs`,
`crates/services/Cargo.toml` (`serde_json = "1.0.154"` dans `[dev-dependencies]`), `crates/stockage/Cargo.toml`
(`serde_json = "1.0.154"` dans `[dev-dependencies]`, son commentaire complété : la suite des commandes, incluse, calcule
l'empreinte d'I13 par `serde_json`), `crates/services/src/lib.rs`,
`crates/noyau/src/donnees/boite.rs` (`boite`, `Boite` et `texte_du_filtre` retirés), `crates/noyau/tests/boite.rs` (les
quatre tests de `boite()` retirés, que `sans_critere_l_index_rend_la_boite` reprend, valeur pour valeur ; dans
`le_compte_et_le_filtre_disent_leur_assiette_et_s_accordent`, renommé `le_compte_dit_son_assiette_et_s_accorde`, les trois
assertions de `texte_du_filtre`, que `le_texte_de_la_reponse_ordonne_ses_parts_et_s_accorde` reprend), `crates/services/tests/suite/mod.rs`,
`crates/services/tests/commandes_en_memoire.rs`, `crates/stockage/tests/commandes_sur_fichiers.rs`,
`app/ui/src/boite.rs` (la commande et le compte), `Cargo.lock` (celui de l'artefact `cargo-lock`, relu).

**Interfaces — consomme :** `Index`, `Reponse`, `Requete`, `Sans`, `Bornes`, `Jour`, `texte_de_la_reponse` (tâches 2 et 3).

**Interfaces — produit** (`commandes`, chaque type doublé de celui du noyau, comme `Ordre` et `Contenu`) :

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)] pub struct Jour { pub annee: i32, pub mois: u8, pub jour: u8 }
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)] pub struct Bornes { pub des: Option<i64>, pub avant: Option<i64> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)] pub enum Etat { Reliee, NonReliee }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)] pub enum ProvenanceDeLIdee { Captee, Importee }
/// Une requête : les champs de `noyau::donnees::requete::Requete`, que la doc de chacun nomme.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requete { pub texte: String, pub du: Option<Jour>, pub au: Option<Jour>, pub etiquette: Option<String>,
    pub plus_de_secondes: Option<u64>, pub etat: Option<Etat>, pub provenance: Option<ProvenanceDeLIdee>,
    pub contenu: Option<Contenu> }
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sans { pub etiquette: Option<u64>, pub duree: Option<u64>, pub texte: Option<u64> }
/// Lire la boîte : l'ordre, la requête en cours, les bornes de sa date (que l'interface tire de `du` et
/// `au`), et la fenêtre demandée.
pub struct Boite { pub ordre: Ordre, pub requete: Requete, pub bornes: Bornes, pub debut: u32, pub nombre: u32 }
/// L'assiette (I09) : considérées, portant, décidées, et les illisibles, comptées à part (question 6).
pub struct Assiette { pub considerees: u64, pub portant: u64, pub decidees: u64, pub illisibles: u64 }
pub struct VueDeLaBoite {
    pub non_reliees: u64, pub assiette_du_compte: Assiette,
    /// Les idées que la requête trouve, toutes sans critère : un compte d'idées.
    pub retenues: u64,
    pub assiette_de_la_requete: Assiette, pub sans: Sans,
    /// Les étiquettes du fonds, chacune une fois, dans l'ordre de leurs octets (question 10).
    pub etiquettes_du_fonds: Vec<String>,
    pub lignes: Vec<LigneDeLaBoite>,
}

// services
impl<D: Documents, C: Captations> Services<D, C> {
    /// La boîte, réponse de la requête en cours (question 21), par l'index tenu.
    pub fn boite(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite>;
    /// La même réponse, d'un index neuf, bâti à l'appel du fonds lu, sans l'index tenu : « l'évaluation sans index »
    /// de la question 13 se lit ainsi, l'index tenu n'étant jamais mis à jour par morceaux, mais rebâti en entier
    /// (question 14) ; un défaut de `construire` ou de `repondre` se voit aux tests de la tâche 3.
    pub fn boite_sans_index(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite>;
}
```

`Services` gagne `index: RefCell<Option<(Vec<ChangeHash>, Index)>>` : à chaque lecture de la boîte, les têtes du
fonds (`Automerge::get_heads`) comparées aux têtes gardées ; différentes, ou aucun index, l'index se rebâtit de
`en_boite` ; sans fonds, un index vide, rien gardé. Aucune écriture ne le touche (question 14) : la lecture qui suit
le rebâtit. La doc de `Services` le dit.

- [ ] **Étape 1 : écrire les tests.** Dans `serialisation.rs`, l'aller-retour de `Jour`, `Bornes`, `Etat`,
  `ProvenanceDeLIdee`, `Requete` (chaque champ posé, puis `Requete::default()`), `Sans`, et la `Boite` et la
  `VueDeLaBoite` nouvelles ; `illisibles`, `sans`, `retenues` sous `ENTIER_SUR` dans le test des entiers. Dans la
  suite, `la_requete_passe_au_noyau` reprend `le_filtre_et_l_ordre_passent_au_noyau` (`suite/mod.rs:3128`), renommé :
  la requête `contenu: Some(Prise)` rend les mêmes lignes que le filtre d'avant, et une requête de texte
  (« pont ») sur trois idées notées rend la seule qui le porte, `sans.texte` à `Some(0)`, `etiquettes_du_fonds` vide ;
  les autres tests de la suite prennent les noms nouveaux (`requete`, `assiette_de_la_requete`), sans changer leurs
  valeurs ; `aucun_compteur_d_anciennete` (`suite/mod.rs:2555`) gagne `sans` et `etiquettes_du_fonds` dans son motif :
  `sans` vaut `Sans::default()`, `etiquettes_du_fonds` est vide, et sa doc dit qu'aucun des deux ne date une idée ;
  `assiette_de` (`:2473`) pose `illisibles: 0` ; `une_idee_illisible_compte_a_part_dans_l_assiette` (`commandes_en_memoire.rs:311`) attend
  `Assiette { considerees: 2, portant: 1, decidees: 1, illisibles: 1 }`.
- [ ] **Étape 2 : voir le rouge** à « Lint » (`no field requete on type Boite`), committer (`test: la requête dans la
  commande de la boîte`).
- [ ] **Étape 3 : écrire le code.** Les conversions (`requete_du_noyau`, `assiette_des_commandes` avec ses
  illisibles, `sans_des_commandes`, `bornes_du_noyau`) à côté de `ordre_du_noyau` ; `boite` et `boite_sans_index`
  partagent une fonction privée qui fait la vue d'une `Reponse`. La doc de `commandes::Assiette` (`:340`) se corrige.
  L'interface : `lire_la_boite` envoie `requete: Requete { contenu: CONTENUS[…], ..Requete::default() }` et
  `bornes: Bornes::default()` ; le compte lit `assiette_de_la_requete` par `texte_de_la_reponse`, et `au_noyau` rend les
  illisibles à part ; rien ne change à l'écran (question 6). Le `Cargo.lock` : l'artefact `cargo-lock`, relu, puis
  committé.
- [ ] **Étape 4 : voir le vert.** À « Tests » de `verifier` : la suite, sur les deux paires, et `serialisation.rs` ;
  dans `windows`, `commandes_sur_fichiers.rs` ; « Cargo.lock à jour » vert ; `boite.rs` du noyau, ses tests de `boite()`
  retirés.
- [ ] **Étape 5 : committer** les fichiers de la tâche, chacun nommé (`feat: la boîte, réponse d'une requête sur
  l'index tenu`).

### Tâche 5 : CAP-06 · 7 à 10, I09, I13 et « chercher n'écrit rien », dans la suite des commandes

**Fichiers :** modifier `crates/services/tests/suite/mod.rs`, `crates/services/tests/commandes_en_memoire.rs`,
`crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — consomme :** `Services::boite`, `boite_sans_index` (tâche 4) ; `verser_une_prise`, `fiche_wav`,
`une_note` (`suite/mod.rs`).

Chaque fonction de la suite est générique sur la paire, et les deux fichiers l'appellent sous un `#[test]` au nom dit
plus bas, comme `cap01_6_…` (`commandes_en_memoire.rs:49`). Une idée sans texte est une prise versée (`verser_une_prise`) ;
une idée de deux prises, deux prises regroupées, comme `deux_prises_regroupees_paraissent_ensemble_dans_la_boite`.

- [ ] **Étape 1 : écrire les tests**, chacun avec ses valeurs, celles de la tâche 3 rejouées par les opérations nommées :
  - `cap06_7_plus_d_une_minute_declare_son_assiette` : trois notes ; neuf prises versées, de 61 000, 90 000 (avec une
    de 12 000 regroupée), 600 000, 75 000, 60 999, 59 999, 30 000, 20 000 (avec une de 5 000 regroupée) et 999 ms ;
    la requête `plus_de_secondes: Some(60)` : `retenues` 4, `assiette_de_la_requete` (12, 9, 9, 0), `sans.duree`
    `Some(3)`, et son texte, par `texte_de_la_reponse`, « 4 idées · 3 sans durée · 12 interrogées » ;
  - `cap06_8_une_idee_retiree_n_est_ni_trouvee_ni_comptee` : les notes « Église du port », « une église retirée »
    (retirée), « le phare et la jetée », « … », « L’EGLISE, encore », deux prises versées ; « eglise » : « 2 idées · 2
    sans texte · 6 interrogées », l'idée retirée absente des lignes ;
  - `cap02_5_une_idee_retiree_quitte_les_requetes` (la part « requêtes ») : une idée étiquetée `pont`, trouvée par le
    critère d'étiquette, puis retirée : elle quitte les lignes, et `assiette_de_la_requete.considerees` baisse d'un ;
  - `cap06_9_une_prise_sans_duree_compte_parmi_les_sans_duree` : une idée de deux prises regroupées, l'une de 30 s,
    l'autre versée sans durée (`Fiche { duree_ms: None, … }`) ; « plus d'une minute » : elle n'est pas dans les lignes,
    et `sans.duree` la compte ;
  - `cap06_10_eglise_trouve_eglise_du_port` : les notes « Église du port » (U+00C9) et « E\u{301}glise du port » ;
    « eglise » les trouve toutes deux ;
  - `i09_chaque_reponse_declare_son_assiette` : sur le fonds de `le_fonds_de_chaque_geste`, chaque requête du jeu fixé
    (`jeu_de_requetes()`) rend une assiette où `considerees` vaut les idées lisibles non retirées plus `illisibles`,
    `decidees` égale `portant`, `retenues ≤ portant`, chaque part « sans » au plus `considerees`, et un texte qui finit
    par « interrogée » ou « interrogées » ;
  - `i13_l_index_reconstruit_rend_les_memes_reponses` : des notes, deux prises versées, une étiquette posée, deux idées
    reliées, l'une retirée (l'autre redevient non reliée), une annulation, puis une écriture refusée (une étiquette de
    blancs) ; après chaque geste, pour chaque requête de `jeu_de_requetes()`, l'empreinte
    (`identifiants::empreinte` de `serde_json::to_vec` de la vue, fenêtre de `FENETRE_MAX`) de `boite` égale celle de
    `boite_sans_index` ; puis le fonds rouvert (`paire.rouvrir`) rend les mêmes empreintes ;
  - `chercher_n_ecrit_rien` : les têtes du fonds (`get_heads`) et le journal, égaux avant et après chaque requête du jeu ;
  - `noter_pendant_une_requete_la_laisse_en_place` (point de revue 1) : sous la requête « eglise », noter « pont du
    soir » : la vue suivante, sous la même requête, ne la montre pas, et `non_reliees` monte d'un ; puis noter
    « l’église du soir » : la vue suivante la montre, `retenues` monte d'un, `non_reliees` d'un encore.

  `jeu_de_requetes()` : la requête vide ; « eglise » ; l'étiquette `pont` ; plus d'une minute ; les non reliées ; les
  importées ; le contenu `Prise` ; « pont » et les non reliées ensemble.
- [ ] **Étape 2 : voir le rouge**, s'il en est un : ces tests éprouvent du code écrit aux tâches 3 et 4. Un vert
  d'emblée se dit dans le rapport de la tâche ; chaque test est alors éprouvé par un mutant à la main (une valeur
  attendue changée le fait rougir), relevé dans le rapport.
- [ ] **Étape 3 : écrire le code** que les tests demandent, s'il en manque.
- [ ] **Étape 4 : voir le vert.** À « Tests » de `verifier` (mémoire) et à l'étape du stockage de `windows` (fichiers) :
  chaque `cap06_…`, `cap02_5_…`, `i09_…`, `i13_…` nommé.
- [ ] **Étape 5 : committer** `git add crates/services/tests/suite/mod.rs crates/services/tests/commandes_en_memoire.rs
  crates/stockage/tests/commandes_sur_fichiers.rs` (`test: CAP-06 · 7 à 10, I09 et I13 sur les deux paires`).

### Tâche 6 : les requêtes enregistrées, au noyau

**Fichiers :** modifier `crates/noyau/src/donnees/requete.rs`, `crates/noyau/src/donnees/gestes.rs` (`Impossible` et son
`Display`), `crates/noyau/src/donnees/fonds.rs` (les clés et les valeurs d'une requête), `crates/noyau/tests/requete.rs`.

**Interfaces — consomme :** `Requete`, `Jour` (tâches 2 et 3) ; `replier` (tâche 1) ; `Operation`, `Changement`,
`Simple` (`changement.rs`) ; `REQUETES_ENREGISTREES`, `TEXTE`, `PROVENANCE`, `CAPTEE`, `IMPORTEE` (`fonds.rs`) ;
`ENTIER_SUR` (`gestes.rs:129`).

**Interfaces — produit :**

```rust
// noyau::donnees::fonds : les clés d'une requête enregistrée, sous requetes_enregistrees/<ULID>
pub const NOM: &str = "nom";
pub const DU: &str = "du";
pub const AU: &str = "au";
pub const ETIQUETTE: &str = "etiquette";
pub const PLUS_DE_SECONDES: &str = "plus_de_secondes";
pub const ETAT: &str = "etat";
pub const CONTENU: &str = "contenu";
// les valeurs : ETAT, "reliee" ou "non_reliee" ; PROVENANCE, CAPTEE ou IMPORTEE ; CONTENU,
// "texte_de_l_idee", "prise", "photo", "midi_verse" ou "texte_verse" ; DU et AU, « AAAA-MM-JJ ».

// noyau::donnees::gestes::Impossible, avec leurs textes
NomVide,                          // « Le nom est vide. »
MemeNom,                          // « Le nom est déjà celui-ci. »
RequeteSansCritere,               // « La requête n'a aucun critère. »
RequeteAbsente { requete: String }, // « Aucune requête enregistrée ne porte cet identifiant. »
SeuilTropLong { secondes: u64 },  // « Ce seuil de durée est trop long. »

// noyau::donnees::requete
/// Une requête enregistrée, lue : son ULID, son nom, et sa requête, `None` si elle porte une clé ou
/// une valeur que cette version ne connaît pas (question 17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequeteEnregistreeLue { pub id: String, pub nom: String, pub requete: Option<Requete> }
/// Enregistrer : un objet créé sous `requetes_enregistrees/<id>` (`Changement::Creer`, sans
/// conteneur), son nom (sans ses blancs autour) et une clé par critère posé, dans la même
/// opération ; le texte n'est posé que s'il a un mot. Message : « Requête « <nom> » enregistrée ».
/// Un nom vide : `NomVide` ; aucun critère : `RequeteSansCritere` ; un seuil au-delà d'`ENTIER_SUR` :
/// `SeuilTropLong`.
pub fn enregistrer_une_requete(id: &str, nom: &str, requete: &Requete, heure: i64) -> Result<Operation, Impossible>;
/// Renommer : le nom posé de nouveau. Message : « Requête « <ancien> » renommée « <nouveau> » ». Un
/// nom vide : `NomVide` ; le même : `MemeNom`.
pub fn renommer_une_requete(lue: &RequeteEnregistreeLue, nom: &str, heure: i64) -> Result<Operation, Impossible>;
/// Supprimer : l'objet ôté d'un bloc. Message : « Requête « <nom> » supprimée ».
pub fn supprimer_une_requete(lue: &RequeteEnregistreeLue, heure: i64) -> Operation;
/// La requête enregistrée `id`, de ses clés : `champs` donne, de chaque clé de son objet, sa valeur
/// simple, ou `None` pour une valeur d'un autre type. Sans `nom` en chaîne : `None` (elle ne se liste
/// pas). Une clé inconnue, une valeur inconnue ou d'un autre type, un jour qui ne se lit pas, un seuil
/// au-delà d'`ENTIER_SUR` : la requête est `None`, le nom gardé.
pub fn lire_une_requete(id: &str, champs: &BTreeMap<String, Option<Simple>>) -> Option<RequeteEnregistreeLue>;
/// Range les requêtes par nom replié, puis par nom, octet pour octet, puis par ULID (question 18).
pub fn ranger_les_requetes(requetes: &mut [RequeteEnregistreeLue]);
```

- [ ] **Étape 1 : écrire les tests** (`crates/noyau/tests/requete.rs`) :
  - `enregistrer_cree_l_objet_avec_son_nom_et_une_cle_par_critere` : la requête { texte « eglise », du 2026-10-25,
    `plus_de_secondes` 60, contenu `Prise` }, le nom « Ponts lents », l'ULID `01JREQUETE0000000000000001`, l'heure
    1 794 906 000 : l'opération vaut, dans cet ordre, `Creer [requetes_enregistrees, <id>]` sans conteneur, puis
    `Poser` de `nom` (« Ponts lents »), `texte` (« eglise »), `du` (« 2026-10-25 »), `plus_de_secondes` (`Naturel(60)`),
    `contenu` (« prise ») ; le message, « Requête « Ponts lents » enregistrée » ;
  - `un_nom_vide_ou_une_requete_sans_critere_est_refuse` : le nom « \u{a0} » rend `NomVide` ; `Requete::default()` et
    la requête de texte « ' » rendent `RequeteSansCritere` ; un seuil de `ENTIER_SUR + 1`, `SeuilTropLong` ;
  - `renommer_et_supprimer_disent_le_nom` : renommer « Ponts lents » en « Ponts du soir » : `Poser` de `nom`, le message
    « Requête « Ponts lents » renommée « Ponts du soir » » ; en « Ponts lents » : `MemeNom` ; en « » : `NomVide` ;
    supprimer : `Oter [requetes_enregistrees, <id>]`, « Requête « Ponts lents » supprimée » ;
  - `une_requete_se_relit_telle_qu_elle_s_est_enregistree` : pour une requête où chaque critère est posé, puis pour
    chacun seul, les `Poser` de l'enregistrement mis en `champs` rendent, par `lire_une_requete`, la même requête et le
    même nom ;
  - `une_cle_inconnue_rend_la_requete_d_une_version_plus_recente` (point de revue 3) : une clé `tonalite` en plus, une
    valeur `contenu` « video », une valeur `etat` en `Naturel`, un `du` « 2026-02-30 », un seuil de `ENTIER_SUR + 1` :
    chaque fois, la requête est `None` et le nom gardé ; sans `nom`, ou un `nom` en `Naturel`, `lire_une_requete` rend
    `None` ;
  - `les_requetes_se_rangent_par_nom_replie` : « ponts lents » (ULID `…03`), « Église » (`…01`), « Ponts lents »
    (`…02`), « Ponts lents » (`…04`) se rangent `…01`, `…02`, `…04`, `…03`.
- [ ] **Étape 2 : voir le rouge** à « Lint » (`cannot find function enregistrer_une_requete`), committer (`test: les
  requêtes enregistrées, au noyau`).
- [ ] **Étape 3 : écrire le code.** La correspondance des critères et des clés, écrite une fois, que l'enregistrement et
  la lecture partagent ; chaque nouveau texte d'`Impossible` dans son `Display`, un constat, jamais un ordre.
- [ ] **Étape 4 : voir le vert.** `requete.rs` `ok. 12 passed` ; `gestes.rs` du noyau vert ; `mutants` sans survivant.
- [ ] **Étape 5 : committer** `git add crates/noyau/src/donnees/requete.rs crates/noyau/src/donnees/gestes.rs
  crates/noyau/src/donnees/fonds.rs crates/noyau/tests/requete.rs` (`feat: les requêtes enregistrées, au noyau`).

### Tâche 7 : les requêtes enregistrées dans les commandes et les services ; I01, I28 et deux répliques

**Fichiers :** modifier `crates/commandes/src/lib.rs`, `crates/commandes/tests/serialisation.rs` (`NOMS.len()` à 21,
`:286`), `crates/services/src/lib.rs`, `crates/services/src/lecture.rs`, `crates/services/tests/suite/mod.rs`,
`crates/services/tests/commandes_en_memoire.rs`, `crates/stockage/tests/commandes_sur_fichiers.rs`.

**Interfaces — consomme :** les fonctions de la tâche 6 ; `ulid_tire` (`lib.rs:604`) ; `Documents::appliquer`.

**Interfaces — produit :**

```rust
// commandes : les noms, entrés dans NOMS après ETAT_DU_FONDS, avant EXPORTER_CAHIER
pub const REQUETES_ENREGISTREES: &str = "requetes_enregistrees";
pub const ENREGISTRER_UNE_REQUETE: &str = "enregistrer_une_requete";
pub const RENOMMER_UNE_REQUETE: &str = "renommer_une_requete";
pub const SUPPRIMER_UNE_REQUETE: &str = "supprimer_une_requete";
/// Une requête enregistrée : son ULID, son nom, et sa requête, `None` si elle vient d'une version
/// plus récente de Pupitre (question 17).
pub struct RequeteEnregistree { pub id: String, pub nom: String, pub requete: Option<Requete> }
pub struct Enregistrer { pub nom: String, pub requete: Requete }
pub struct Renommer { pub requete: String, pub nom: String }
pub struct Supprimer { pub requete: String }

// services::lecture
/// Chaque requête enregistrée de `doc` qui a un nom (`lire_une_requete`), dans l'ordre des clés.
pub fn requetes_enregistrees(doc: &Automerge) -> Vec<RequeteEnregistreeLue>;

// services::Services
pub fn requetes_enregistrees(&self) -> Reponse<Vec<RequeteEnregistree>>;
pub fn enregistrer_une_requete(&mut self, commande: &Enregistrer, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn renommer_une_requete(&mut self, commande: &Renommer, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
pub fn supprimer_une_requete(&mut self, commande: &Supprimer, contexte: &mut Contexte<'_>) -> Reponse<Faite>;
```

Chaque écriture refuse d'abord la lecture seule (le refus de `lecture_seule()`, tel quel), puis un fonds non né
(`Raison::Impossible`, « Aucun fonds n'est né. », sans rien faire naître : question 20), puis une requête absente
(`RequeteAbsente`) ; enregistrer tire l'ULID par `ulid_tire` ; aucune ne touche le dernier geste de tri. La lecture,
en lecture seule comme ailleurs, sans fonds rend une liste vide, rangée par `ranger_les_requetes`. Dans
`lecture::requetes_enregistrees`, une valeur `Str`, `Int`, `Uint` ou `Boolean(true)` devient `Chaine`, `Entier`,
`Naturel` ou `Vrai`, toute autre `None`.

- [ ] **Étape 1 : écrire les tests.** Dans la suite, sur les deux paires :
  - `enregistrer_renommer_et_supprimer_une_requete` : une note, un geste de tri (une étiquette posée) ; enregistrer
    « Ponts lents » (la requête « pont »), la renommer « Ponts du soir », en enregistrer une seconde « Église », puis
    supprimer la première : la liste rend « Église » seule ; le journal finit par les trois messages, puis « Requête
    « Ponts du soir » supprimée » ; `etat_du_fonds().a_annuler` reste le message de l'étiquette posée ;
  - `enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait` : le refus « Aucun fonds n'est né. », de
    `Raison::Impossible` ; `documents().fonds()` reste `None` ;
  - `en_lecture_seule_chercher_sert_et_enregistrer_est_refuse` (point de revue 2) : le fonds d'une version plus récente,
    comme `une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme` (`suite/mod.rs:1137`) : `boite` répond ;
    enregistrer, renommer et supprimer rendent le refus de la lecture seule ;
  - `i01_une_requete_enregistree_ne_garde_ni_resultats_ni_nombre` : la même requête enregistrée dans deux fonds aux idées
    différentes (deux notes, cinq notes) : les clés et les valeurs de son objet sont les mêmes, l'ULID à part ; et
    `parcourir` ne rend aucun chemin fautif ;
  - `rien_d_engendre_n_est_stocke` (`suite/mod.rs:2383`) enregistre une requête de plus ; `permis` admet
    `[requetes_enregistrees, <id>]` et `[requetes_enregistrees, <id>, <clé>]`, la clé parmi `CHAMPS_D_UNE_REQUETE` (les
    neuf de la tâche 6), et rien d'autre dessous ; `INTERDITS_SOUS_UNE_REQUETE` (« resultats », « nombre ») s'ajoute à
    `interdit` ;
  - `l_historique_ne_porte_que_des_operations_connues` (`:2304`) joue aussi les trois opérations ; `message_connu`
    gagne `est_une_operation_sur_une_requete` : « Requête « », un nom non vide, puis « » enregistrée », « » supprimée »,
    ou « » renommée « », un nom non vide, « » » ;
  - `un_texte_piege_revient_tel_quel` (`:2849`) pose aussi `<img src=x onerror=alert(1)>` en étiquette et
    `<script>alert(1)</script>` en nom de requête : la vue de l'idée, `etiquettes_du_fonds` et la liste des requêtes les
    rendent tels quels.

  Dans `commandes_en_memoire.rs`, sur cette paire seule,
  `un_renommage_concurrent_d_une_suppression_ne_laisse_aucune_requete_sans_critere` : un fonds où une requête est
  enregistrée ; deux copies du document (`Automerge::fork`) ; sur l'une, `renommer_une_requete` appliqué par
  `services::application::appliquer`, sur l'autre, `supprimer_une_requete` ; les deux fusionnées (`merge`), dans les
  deux sens : `lecture::requetes_enregistrees` ne rend aucune requête ; puis, à rebours, le renommage seul fusionné avec
  un fonds où rien n'est supprimé rend la requête renommée, ses critères entiers.
- [ ] **Étape 2 : voir le rouge** à « Lint » (`cannot find value ENREGISTRER_UNE_REQUETE`), committer (`test: les
  requêtes enregistrées, servies et gardées`).
- [ ] **Étape 3 : écrire le code.** Le rouge d'I31 dans `windows` (`i31.rs` : un nom de `NOMS` sans droit) est attendu
  jusqu'à la tâche 8, qui sert les commandes : il se dit dans le rapport de la tâche.
- [ ] **Étape 4 : voir le vert** à « Tests » de `verifier` ; dans `windows`, la suite sur fichiers verte, I31 rouge comme
  dit.
- [ ] **Étape 5 : committer** les fichiers de la tâche (`feat: les requêtes enregistrées, servies sur les deux paires`).

### Tâche 8 : l'établi sert les commandes nouvelles

**Fichiers :** modifier `app/src-tauri/capabilities/windows.json`, `app/src-tauri/src/lib.rs` (`gestionnaire`, `:135`),
`app/src-tauri/src/commandes_de_l_etabli.rs`.

**Interfaces — consomme :** les méthodes de la tâche 7 ; `Ouverture::demander` (`fil.rs:171`).

- [ ] **Étape 1 : écrire le code.** Quatre droits, après `allow-etat-du-fonds` : `allow-requetes-enregistrees`,
  `allow-enregistrer-une-requete`, `allow-renommer-une-requete`, `allow-supprimer-une-requete` ; quatre enveloppes
  `#[tauri::command] pub async fn <nom>(ouverture: tauri::State<'_, Ouverture>, commande: <T>) -> Reponse<R>`, sur le
  modèle de `boite` (`:131-138`), la lecture sans argument comme `etat_du_fonds` ; quatre lignes dans
  `generate_handler!`. Les tests d'I31 comparent `NOMS` aux droits et au gestionnaire : ils sont le test de la tâche.
- [ ] **Étape 2 : voir le vert** dans `windows` : « Lint de l'établi », et `i31.rs` `test result: ok.`
- [ ] **Étape 3 : committer** `git add app/src-tauri/capabilities/windows.json app/src-tauri/src/lib.rs
  app/src-tauri/src/commandes_de_l_etabli.rs` (`feat: l'établi sert les requêtes enregistrées`).

### Tâche 9 : le journal à fenêtre, ses heures et son empreinte ; l'écart d'un jour

**Fichiers :** créer `crates/noyau/src/donnees/journal.rs`, `crates/noyau/tests/journal.rs` ; modifier
`crates/noyau/src/donnees/mod.rs`, `crates/noyau/src/donnees/gestes.rs` (`relier`, `delier`, `regrouper`),
`crates/noyau/tests/gestes.rs`, `crates/commandes/src/lib.rs`, `crates/commandes/tests/serialisation.rs`,
`crates/services/src/lib.rs`, `crates/services/src/mesure.rs` (`Relier`, `:94`, et `Regrouper`, `:144` : le second
décalage à 0, comme le premier ; le poids du test 13 ne change pas, les messages gardant leur longueur),
`crates/services/tests/suite/mod.rs` (les commandes de geste, le journal),
`app/src-tauri/src/commandes_de_l_etabli.rs` (`journal`, `:123-127`), `app/ui/src/boite.rs` (les décalages envoyés),
`.cargo/mutants.toml` (`journal.rs`, `"--test", "journal"`, le compte).

**Interfaces — consomme :** `date_de_la_liste` (`boite.rs:232`), `empreinte` (`identifiants.rs:45`) ;
`lecture::journal` (`lecture.rs:157`).

**Interfaces — produit :**

```rust
// noyau::donnees::journal
/// Les entrées (message, heure), reçues dans l'ordre de l'historique, rangées la plus récente
/// d'abord ; à égalité d'heure, la plus tardive dans l'historique d'abord (question 29).
pub fn ranger_le_journal(entrees: Vec<(String, i64)>) -> Vec<(String, i64)>;
/// « 25 oct. 2026 · 02:30 » : la date de la liste et l'heure, sur deux chiffres, au décalage
/// `decalage_minutes` ; « — » pour l'heure 0, celle de la genèse et des migrations (§ 31 · 2.4).
pub fn heure_du_journal(heure: i64, decalage_minutes: i32) -> String;
/// « Le même journal » (question 29) : les entrées rangées par heure, puis par message, octet pour
/// octet, chacune écrite `<heure>`, une tabulation, `<message>`, un saut de ligne ; le SHA-256 de ce
/// texte (`identifiants::empreinte`). L'ordre de l'historique n'y compte pas.
pub fn empreinte_du_journal(entrees: &[(String, i64)]) -> String;
/// Les six premiers caractères d'une empreinte, que la vue montre.
pub fn empreinte_courte(empreinte: &str) -> &str;

// commandes
/// La vue du journal : le nombre d'entrées, l'empreinte du journal, et les lignes de la fenêtre,
/// la plus récente d'abord.
pub struct VueDuJournal { pub total: u64, pub empreinte: String, pub lignes: Vec<EntreeDuJournal> }
// Relier gagne `decalage_b_minutes: i32`, Regrouper `decalage_source_minutes: i32` ; la doc de chaque
// `decalage_minutes` d'une commande de geste : le décalage local à l'instant de l'idée qu'il date
// (`natif::decalage_a(le)`) ; celle de `Noter` : le décalage du geste, qui est celui de la naissance.

// services
pub fn journal_en_fenetre(&self, commande: &Fenetre) -> Reponse<VueDuJournal>;

// noyau::donnees::gestes
pub fn relier(a: &IdeeLue, b: &IdeeLue, heure: i64, decalage_de_a: i32, decalage_de_b: i32) -> Result<GesteDeTri, Impossible>;
pub fn delier(a: &IdeeLue, b: &IdeeLue, heure: i64, decalage_de_a: i32, decalage_de_b: i32) -> Result<GesteDeTri, Impossible>;
pub fn regrouper(captation: &CaptationLue, source: &IdeeLue, captations_de_la_source: usize, cible: &IdeeLue,
    heure: i64, decalage_de_la_cible: i32, decalage_de_la_source: i32) -> Result<GesteDeTri, Impossible>;
```

- [ ] **Étape 1 : écrire les tests.** `crates/noyau/tests/journal.rs` :

```rust
#[test]
fn l_heure_du_journal_se_dit_a_l_instant_et_la_genese_par_un_tiret() {
    assert_eq!(heure_du_journal(0, 60), "—");
    assert_eq!(heure_du_journal(1_792_888_200, 120), "25 oct. 2026 · 02:30"); // 0 h 30 UTC, heure d'été
    assert_eq!(heure_du_journal(1_792_891_800, 60), "25 oct. 2026 · 02:30"); // 1 h 30 UTC, heure d'hiver
    assert_eq!(heure_du_journal(1_798_759_800, 60), "1er janv. 2027 · 00:30");
    assert_eq!(heure_du_journal(1_792_886_460, 0), "25 oct. 2026 · 00:01");
}

#[test]
fn le_journal_se_range_le_plus_recent_d_abord() {
    let recu = vec![("Genèse du fonds".into(), 0), ("A".into(), 10), ("B".into(), 10), ("C".into(), 5)];
    assert_eq!(ranger_le_journal(recu),
        [("B".to_string(), 10), ("A".to_string(), 10), ("C".to_string(), 5), ("Genèse du fonds".to_string(), 0)]);
}

#[test]
fn le_meme_journal_a_la_meme_empreinte() {
    let journal = [("Genèse du fonds".to_string(), 0), ("Fonds créé".to_string(), 1_792_888_200),
        ("Idée du 25 octobre notée".to_string(), 1_792_888_220)];
    let e = empreinte_du_journal(&journal);
    assert_eq!(e, "570cda108d563a266092b9cc5b4e884c691ca3ee7b228c032d594e8219b54dce");
    assert_eq!(empreinte_courte(&e), "570cda");
    let mut a_rebours = journal.clone();
    a_rebours.reverse();
    assert_eq!(empreinte_du_journal(&a_rebours), e);
    let mut autre = journal.clone();
    autre[2].1 += 1;
    assert_ne!(empreinte_du_journal(&autre), e);
}
```

  Dans `crates/noyau/tests/gestes.rs`, `le_message_date_chaque_idee_a_son_propre_decalage` : `a` née à 1 792 881 000
  (24 octobre, 22 h 30 UTC), `b` à 1 794 906 000 (17 novembre, 9 h UTC) ; `relier(&a, &b, heure, 120, 60)` dit « Idée du
  25 octobre reliée à l'idée du 17 novembre » ; `relier(&a, &b, heure, 60, 60)`, « Idée du 24 octobre … » ; de même
  `delier` et `regrouper` (la cible, puis la source retirée). Les tests existants de ces gestes passent le même décalage
  deux fois. Dans la suite, `le_journal_se_lit_en_fenetre` : un fonds de trois notes, la fenêtre (0, 2) rend
  `total` 5 (genèse, naissance, trois notes), deux lignes, la plus récente d'abord, et l'empreinte de
  `empreinte_du_journal` sur `Services::journal` ; la fenêtre (4, 200), la seule genèse. Dans l'interface,
  `relier_les_cochees` : une fonction pure qui fait la commande `Relier` de deux idées cochées, chacune avec sa
  naissance, par un décalage donné ; avec `|le| if le < 1_792_890_000 { 120 } else { 60 }`, deux idées nées à
  1 792 881 000 et 1 794 906 000 donnent `decalage_minutes` 120 et `decalage_b_minutes` 60 ; une ou trois cochées,
  `None`. Elle appelle `commande_de_lien(a: (String, i64), b: (String, i64), decalage: impl Fn(i64) -> i32) -> Relier`,
  que délier appelle aussi ; `delier_depuis_le_volet_date_chaque_bout` : l'idée du volet née à 1 792 881 000, l'autre
  bout à 1 794 906 000, le même décalage : `decalage_minutes` 120, `decalage_b_minutes` 60.
- [ ] **Étape 2 : voir le rouge** à « Lint » (`unresolved import noyau::donnees::journal`), committer (`test: le journal
  rangé, daté et son empreinte ; chaque idée datée à son instant`).
- [ ] **Étape 3 : écrire le code.** `heure_du_journal` partage le calcul civil de `date_civile` pour l'heure (`(heure +
  décalage × 60)` modulo 86 400) ; `journal_en_fenetre` lit `lecture::journal`, range, borne la fenêtre à
  `FENETRE_MAX` ; l'enveloppe `journal` de l'établi prend `commande: Fenetre` et rend `VueDuJournal`. L'interface :
  `cochees` garde la naissance de chaque idée cochée (`Vec<(String, i64)>`) ; `IdeeOuverte.liens` garde celle de chaque
  idée liée (`Vec<(String, String, Option<i64>)>`, de `liee.le` dans `lire_l_idee`), `None` pour un bout qui ne s'est pas
  lu. Compléter, étiqueter, et retirer ou rétablir depuis le volet envoient `natif::decalage_a(le)` de l'idée du volet ;
  rétablir depuis la liste des retirées, celui du `le` de sa ligne ; relier, `relier_les_cochees(…, natif::decalage_a)` ;
  délier, `commande_de_lien` de l'idée du volet et de l'autre bout, un bout sans naissance prenant
  `natif::decalage_du_geste()`, limite que la PR (a) dit ; noter garde `decalage_du_geste`.
- [ ] **Étape 4 : voir le vert.** `journal.rs` `ok. 3 passed`, `gestes.rs` du noyau un test de plus, la suite et
  `serialisation.rs` verts ; dans `windows`, « Lint de l'établi » et `i31.rs` verts ; `mutants` sans survivant.
- [ ] **Étape 5 : committer** les fichiers de la tâche (`feat: le journal à fenêtre, son empreinte, et chaque idée datée
  à son instant`).

### Tâche 10 : les documents de la PR (a), la mutation, les délais, et la revue de la PR (a)

**Fichiers :** `docs/20-sfd/21-capter-et-organiser.md` (CAP-02, CAP-06), `docs/30-modele-de-domaine.md` (§ 9.2, § 11),
`docs/31-persistance-et-synchronisation.md` (§ 2, § 2.3, § 7, § 10), `docs/41-pile-technique.md` (§ 6),
`docs/70-qualite-et-verification.md` (§ 3, § 4.1, § 8), `docs/80-glossaire.md`, `docs/pistes.md`, `CONVENTIONS.md`,
`CLAUDE.md`, `ci.yml` (les délais), `mutants-complet.yml` (son délai).

- [ ] **Étape 1 : la recherche de LE-2**, citée dans la description de la PR, pour chaque mot ajouté ou requalifié :
  `requête`, `requêtes`, `critère`, `recherche`, `chercher`, `filtre`, `filtrer`, `repli`, `replier`, `destructrice`,
  `supprimer`, chacun aussi sans accent et sur la majuscule accentuée ; chaque emploi antérieur se requalifie.
- [ ] **Étape 2 : la SFD.** CAP-06, ses règles : la recherche par texte lit le texte de l'idée seul (question 4) ; la
  table de repli et le mot cherché (questions 1 à 3) ; les critères d'une idée, combinés par « et », la provenance de
  l'idée (captée ou importée), distincte de la sorte « par provenance », qui vise les éléments (question 11) ; la
  durée à la seconde affichée (question 8) ; la date en jours civils (question 9) ; l'étiquette octet pour octet
  (question 10) ; l'assiette d'une requête et ses parts « sans », comptées parmi les candidates, avec l'exemple « 4
  idées · 3 sans durée · 1 illisible · 13 interrogées » (questions 6 et 7). CAP-02 (`:135`) : « Elle se filtre par ce
  que l'idée contient » devient « Une requête la restreint à ce que l'idée contient », suivi d'un lien vers CAP-06.
- [ ] **Étape 3 : le modèle et la persistance.** § 30 · 9.2 (`:633`) : « pour les montrer et les ranger » ; chercher
  par eux reste la piste. § 30 · 11 : les trois messages d'une requête enregistrée, leur nom libre en fin. § 31 · 2
  (`:198`) : `requetes_enregistrees` porte un objet par requête, ses clés et leurs valeurs (tâche 6), écrites à sa
  naissance, sans migration ni genèse changée ; le texte dit qu'il se gèle avec la `v0.3.0`. § 31 · 2.3 : enregistrer
  une requête ne fait pas naître le fonds, et se refuse avant. § 31 · 7 (`:773`) : au lot 1, l'index lit les idées
  interrogées (le texte replié, les attributs), se bâtit à la première lecture qui suit un changement des têtes du
  document, et sa construction compte dans l'ouverture (question 15). § 31 · 10, test 1 (`:1004`) : « les empreintes
  canoniques » sont, pour l'index, celles des réponses entières d'un jeu de requêtes fixé (SHA-256 de leur JSON).
- [ ] **Étape 4 : la qualité, le glossaire, les pistes.** § 70 · 3, I13 (`:124`) : son test, comme écrit à l'étape 3 ;
  § 70 · 4.1 : la ligne « CAP-06 | 7 à 10 | INC-1, lot 1 », après CAP-02 ; § 70 · 8 : I09 et I13 ont leurs tests. Le
  glossaire (questions 19, 32) : « Requête » (des critères sur les idées, combinés par « et », qui rendent des idées
  et une assiette), « Critère » (une condition d'une requête : date, étiquette, durée, état, provenance, contenu,
  texte), « Recherche » (le champ et le geste de chercher par texte ; la requête qu'il pose), « Requête enregistrée »
  (un nom et des critères, jamais de résultats), « Table de repli », « Opération destructrice » (une opération qui
  efface une idée ou une captation ; supprimer une requête enregistrée n'en est pas une). `docs/pistes.md` : « L'écart
  d'un jour » sort ; « Chercher par nom d'origine » s'étend au contenu d'un texte versé ; deux pistes neuves, « Chercher
  par origine déclarée » (se déclenche en tranche 5) et « Une proposition trouvée » (`~` contre « pas décidée », INC-2,
  LE-6 L17). § 41 · 6 : rien, son budget ne comptant pas les dépendances de test ; la PR (a) dit la dépendance de test nouvelle.
- [ ] **Étape 5 : les portes.** Le § 40 · 7.5, relu : son tableau nomme déjà « les opérations nommées et les requêtes
  du fonds », que les quatre commandes nouvelles sont ; il ne change pas, et la PR le dit. `CONVENTIONS.md` : `mutants` (les modules `repli`, `requete`, `index`, `journal`),
  `verifier` (I09, I13 dans la suite) ; `CLAUDE.md` : « Les crates » (le noyau : le repli, les requêtes, l'index, le
  journal rangé ; `commandes` : `NOTER, …, EXPORTER_CAHIER` et les requêtes enregistrées ; l'interface : « son filtre »
  devient « sa requête »). Les délais de `verifier`, `windows` et `mutants` (et de `mutants-complet.yml`), remesurés
  sur l'exécution de la tête, chacun au double de la mesure, son commentaire daté.
- [ ] **Étape 6 : committer** les documents (`docs: les requêtes au glossaire, à la SFD, au schéma et aux portes`).
- [ ] **Étape 7 : la revue de la PR (a).** La revue de la PR entière, sur sa tête, par le modèle le plus capable : le
  tableau du § 70 · 4.1 contre les critères de la tranche, le job de chaque test nommé, la recherche de LE-2, chaque
  règle de LE-6 recopiée, chaque mécanisme annoncé relu. Puis les corrections, leur relecture sur la tête qui part. Le
  corps dit : les tests réécrits (ceux de `boite()`, `le_filtre_et_l_ordre_passent_au_noyau`, les gestes à deux
  décalages) ; la dépendance de test nouvelle ; qu'un fonds importé répondra surtout « sans texte » (question 4) ; que
  le repli ne couvre que sa table ; que la construction de l'index compte dans l'ouverture ; que `boite_sans_index` est
  un index neuf, non une évaluation sans index, et pourquoi le test d'I13 tient (R9) ; qu'un bout de lien illisible se
  délie au décalage du geste (R2) ; « Écarts à la spec : Aucun. ». `gh pr ready`, puis la fusion par squash, `--match-head-commit` nommant la tête relue.

## PR (b) · la vue du fonds

**Branche :** `t4b-la-vue-du-fonds`. **Titre :** `feat: la vue du fonds, sa recherche, ses requêtes enregistrées et son
journal` (77). **Documents :** CAP-07 ; § 40 · 9 ; § 50 · 2, 3.1, 5 et 7 ; la maquette du fonds ; § 70 · 3, 7 et 8 ; le
glossaire ; `docs/pistes.md` ; le registre (LE-16) ; `CONVENTIONS.md` ; `CLAUDE.md`. **Fin :** les tâches 17 et 18.

### Tâche 11 : la requête de l'écran et ses touches, en fonctions pures

**Fichiers :** créer `app/ui/src/recherche.rs` ; modifier `app/ui/src/main.rs` (`mod recherche`), `app/ui/src/boite.rs`
(`dans_un_champ_de_texte`, `:349`, et `CONTENUS`, `:53`, en `pub(crate)`).

**Interfaces — consomme :** `commandes::{Requete, Jour, Bornes, Etat, ProvenanceDeLIdee, Contenu}` ;
`noyau::donnees::requete::{Jour::depuis_iso, bornes_de_la_periode}` (tâche 2) ; `natif::decalage_a`.

**Interfaces — produit** (`app/ui/src/recherche.rs`) :

```rust
/// La requête en cours, que l'en-tête et le fonds partagent, tenue par `Etat` (`Etat::nouveau` la crée) pour
/// survivre au changement de vue : le texte cherché, la période (la valeur de chaque champ de date), l'étiquette,
/// et le rang choisi de chaque réglage.
#[derive(Clone, Copy)]
pub(crate) struct EtatDeLaRequete {
    pub texte: RwSignal<String>, pub du: RwSignal<String>, pub au: RwSignal<String>,
    pub etiquette: RwSignal<Option<String>>, pub duree: RwSignal<usize>, pub etat: RwSignal<usize>,
    pub provenance: RwSignal<usize>, pub contenu: RwSignal<usize>,
}
pub(crate) const DUREES: [(Option<u64>, &str); 4] =
    [(None, "Toutes"), (Some(30), "Plus de 30 s"), (Some(60), "Plus d'une minute"), (Some(300), "Plus de 5 min")];
pub(crate) const ETATS: [(Option<Etat>, &str); 3] =
    [(None, "Toutes"), (Some(Etat::NonReliee), "Non reliées"), (Some(Etat::Reliee), "Reliées")];
pub(crate) const PROVENANCES: [(Option<ProvenanceDeLIdee>, &str); 3] =
    [(None, "Toutes"), (Some(ProvenanceDeLIdee::Captee), "Captées"), (Some(ProvenanceDeLIdee::Importee), "Importées")];
impl EtatDeLaRequete {
    pub(crate) fn nouveau() -> Self;
    /// La requête que disent les réglages (le contenu par `boite::CONTENUS`).
    pub(crate) fn requete(self) -> Requete;
    /// Les réglages qui disent `requete`, ceux d'un jour illisible ou d'un seuil absent de `DUREES` vidés.
    pub(crate) fn poser(self, requete: &Requete);
}
/// L'`id` du champ de recherche de l'en-tête, seul champ où Échap garde son sens du navigateur.
pub(crate) const ID_DE_LA_RECHERCHE: &str = "recherche";
/// La cible d'un événement, dite par son `id`, est le champ de recherche ; tout autre champ de texte, ou une cible sans
/// `id`, ne l'est pas.
pub(crate) fn est_la_recherche(id_de_la_cible: Option<&str>) -> bool;
/// Le jour d'un champ de date : « AAAA-MM-JJ », ou `None` pour un champ vide ou incomplet.
pub(crate) fn jour_du_champ(valeur: &str) -> Option<Jour>;
/// Les bornes de la période de `requete`, au décalage que `decalage` dit de chaque instant.
pub(crate) fn bornes_de_la_requete(requete: &Requete, decalage: impl Fn(i64) -> i32) -> Bornes;
/// `/` cherche (question 24) : la touche produite `/`, sans Ctrl, Alt ni Méta (AltGr arrive comme
/// Ctrl + Alt), ni tenue, ni en composition, hors d'un champ de texte ; Maj admise.
pub(crate) fn slash_cherche(touche: &str, ctrl: bool, alt: bool, meta: bool, tenue: bool,
    en_composition: bool, dans_un_champ: bool) -> bool;
/// Échap ferme le volet ouvert (question 25) : la touche `Escape`, un volet ouvert, hors d'une composition, hors de la
/// recherche, où Échap garde son sens du navigateur : vider le champ, sans fermer de volet.
pub(crate) fn echap_ferme(touche: &str, en_composition: bool, volet_ouvert: bool, dans_la_recherche: bool) -> bool;
```

- [ ] **Étape 1 : écrire les tests** (dans `recherche.rs`, `#[cfg(test)]`) :

```rust
#[test]
fn slash_porte_le_focus_hors_d_un_champ_seulement() {
    assert!(slash_cherche("/", false, false, false, false, false, false)); // AZERTY : Maj + :
    assert!(!slash_cherche("/", false, false, false, false, false, true)); // dans un champ, elle s'écrit
    assert!(!slash_cherche("/", true, false, false, false, false, false));
    assert!(!slash_cherche("/", false, true, false, false, false, false));
    assert!(!slash_cherche("/", true, true, false, false, false, false)); // AltGr
    assert!(!slash_cherche("/", false, false, true, false, false, false));
    assert!(!slash_cherche("/", false, false, false, true, false, false)); // tenue
    assert!(!slash_cherche("/", false, false, false, false, true, false)); // en composition
    assert!(!slash_cherche("!", false, false, false, false, false, false));
    assert!(!slash_cherche(":", false, false, false, false, false, false));
}

#[test]
fn echap_ne_ferme_que_le_volet_ouvert() {
    assert!(echap_ferme("Escape", false, true, false));
    assert!(!echap_ferme("Escape", false, false, false));
    assert!(!echap_ferme("Escape", true, true, false));
    assert!(!echap_ferme("Esc", false, true, false));
    assert!(!echap_ferme("Escape", false, true, true)); // dans la recherche, Échap vide le champ
    assert!(est_la_recherche(Some(ID_DE_LA_RECHERCHE)));
    assert!(!est_la_recherche(Some("note")));
    assert!(!est_la_recherche(Some("")));
    assert!(!est_la_recherche(None));
}

#[test]
fn un_champ_de_date_vide_ne_borne_pas() {
    assert_eq!(jour_du_champ(""), None);
    assert_eq!(jour_du_champ("2026-10"), None);
    assert_eq!(jour_du_champ("2026-10-25"), Some(Jour { annee: 2026, mois: 10, jour: 25 }));
}

#[test]
fn les_bornes_de_la_requete_se_tirent_de_ses_jours() {
    let paris = |t: i64| if (1_774_746_000..1_792_890_000).contains(&t) { 120 } else { 60 };
    let jour = jour_du_champ("2026-10-25");
    let requete = Requete { du: jour.clone(), au: jour, ..Requete::default() };
    assert_eq!(bornes_de_la_requete(&requete, paris),
        Bornes { des: Some(1_792_879_200), avant: Some(1_792_969_200) });
    assert_eq!(bornes_de_la_requete(&Requete::default(), paris), Bornes::default());
}

#[test]
fn la_requete_de_l_ecran_suit_ses_reglages_et_s_y_repose() {
    let owner = Owner::new();
    owner.set();
    let etat = EtatDeLaRequete::nouveau();
    assert_eq!(etat.requete(), Requete::default());
    let requete = Requete { texte: "eglise".into(), du: jour_du_champ("2026-10-25"), etiquette: Some("pont".into()),
        plus_de_secondes: Some(60), etat: Some(Etat::NonReliee), provenance: Some(ProvenanceDeLIdee::Importee),
        contenu: Some(Contenu::Prise), ..Requete::default() };
    etat.poser(&requete);
    assert_eq!(etat.requete(), requete);
    etat.poser(&Requete { plus_de_secondes: Some(45), ..Requete::default() });
    assert_eq!(etat.requete().plus_de_secondes, None);
}
```

- [ ] **Étape 2 : voir le rouge** à « Lint » de l'interface (`unresolved import crate::recherche`), committer (`test:
  la requête de l'écran, /, Échap`).
- [ ] **Étape 3 : écrire le code**, sans signal hors d'`EtatDeLaRequete`, ni appel natif hors de `bornes_de_la_requete`,
  qui reçoit le décalage.
- [ ] **Étape 4 : voir le vert.** Les tests de l'interface, `statique.rs` et `encre_statique.rs` verts.
- [ ] **Étape 5 : committer** `git add app/ui/src/recherche.rs app/ui/src/main.rs app/ui/src/boite.rs` (`feat: la requête
  de l'écran et ses touches, en fonctions pures`).

### Tâche 12 : l'en-tête de l'établi : la recherche, l'état de synchronisation, « Afficher » ; `/`

**Fichiers :** modifier `app/ui/src/app.rs`, `app/ui/src/entete.rs`, `app/ui/style.css`.

**Interfaces — consomme :** `EtatDeLaRequete`, `slash_cherche`, `dans_un_champ_de_texte` (tâche 11).

**Interfaces — produit :** `Vue { Fonds, Table }`, `Fonds` par défaut ; `Etat` gagne `requete: EtatDeLaRequete` et
`chercher(self, texte: String)`, qui pose le texte et passe au fonds ; `entete::ETAT_DE_SYNCHRONISATION: &str = "Sans
nœud · aucune archive"` (26 caractères, `œ` U+0153, le point médian U+00B7 entre deux espaces : LE-6, règle 11).

- [ ] **Étape 1 : écrire les tests** (`app.rs`, `#[cfg(test)]`) :
  - `l_etabli_s_ouvre_sur_le_fonds_et_quitter_la_table_ferme_sa_palette` remplace
    `l_etabli_s_ouvre_sur_la_table_et_la_quitter_ferme_la_palette` (`:246`) : la vue par défaut est `Fonds` ; choisir
    `Table`, ouvrir la palette, revenir au fonds la ferme ;
  - `chercher_depuis_la_table_passe_au_fonds` : vue `Table`, `chercher("eglise")` : la vue est `Fonds`, le texte de la
    requête « eglise » ;
  - `l_etat_de_synchronisation_dit_sans_noeud_aucune_archive` : la constante, 26 caractères, 28 octets.
- [ ] **Étape 2 : voir le rouge** à « Lint » (`no variant Fonds`), committer (`test: l'établi s'ouvre sur le fonds`).
- [ ] **Étape 3 : écrire le code.** `Entete` porte, avant « Afficher », un champ `<input type="search">`, d'`id` `ID_DE_LA_RECHERCHE`, libellé
  « Chercher dans le texte des idées », `aria-keyshortcuts="/"`, et une aide liée par `aria-describedby` (« / porte le
  focus ici, depuis le fonds comme depuis la table ») ; chaque frappe (`on:input`) appelle `etat.chercher`, sans
  minuterie ; l'état de synchronisation, un paragraphe, toujours visible, sur le fonds comme sur la table. « Afficher »
  choisit « Fonds » ou « Table ». `Etat::nouveau` crée `EtatDeLaRequete` (`etat.requete`), qu'`App` ne crée pas à part ;
  `App` écoute `keydown` sur la fenêtre : `slash_cherche` vrai, `prevent_default`, puis le focus dans le champ (un `NodeRef` que l'en-tête reçoit).
  La boîte d'idées devient la zone « Boîte d'idées » de la vue du fonds, qui la porte entière.
- [ ] **Étape 4 : voir le vert**, les tests de l'interface et ses contrôles statiques.
- [ ] **Étape 5 : committer** `git add app/ui/src/app.rs app/ui/src/entete.rs app/ui/style.css` (`feat: l'en-tête de
  l'établi porte la recherche et l'état de synchronisation`).

### Tâche 13 : les critères à l'écran ; la liste, réponse de la requête en cours

**Fichiers :** modifier `app/ui/src/boite.rs`, `app/ui/style.css`.

**Interfaces — consomme :** `EtatDeLaRequete::requete`, `bornes_de_la_requete`, `DUREES`, `ETATS`, `PROVENANCES`
(tâche 11) ; `VueDeLaBoite::{assiette_de_la_requete, sans, etiquettes_du_fonds}` (tâche 4) ; `texte_de_la_reponse`.

- [ ] **Étape 1 : écrire les tests** (`boite.rs`, `#[cfg(test)]`) :
  - `la_commande_porte_la_requete_en_cours_et_ses_bornes` : une fonction pure, `commande_de_la_boite(ordre, requete,
    decalage, fenetre) -> Boite`, que `lire_la_boite` appelle : la requête « eglise » du 25 octobre au 25 octobre, contenu
    `Some(Contenu::Prise)`, avec le décalage de Paris (tâche 11), donne la requête telle quelle dans `Boite.requete`, donne `bornes` (1 792 879 200, 1 792 969 200), et la fenêtre reçue ;
  - `la_reponse_ne_se_dit_que_d_une_requete_posee` : `texte_de_la_requete(vue, requete) -> Option<String>` : `None`
    pour `Requete::default()` ; pour une requête de texte, `texte_de_la_reponse` de la vue (« 2 idées · 2 sans texte ·
    6 interrogées » sur une vue écrite par le test) ;
  - `chaque_frappe_demande_et_seule_la_derniere_reponse_compte` : `Demandes` (`:426`) : trois demandes de suite, les
    deux premières répondent après la troisième, seule la dernière est retenue ;
  - `noter_ne_touche_pas_la_requete_en_cours` (point de revue 1) : la fonction qui conclut une note faite vide le champ
    de note et laisse chaque signal d'`EtatDeLaRequete` tel quel.
- [ ] **Étape 2 : voir le rouge** à « Lint », committer (`test: la boîte, réponse de la requête en cours`).
- [ ] **Étape 3 : écrire le code.** `lire_la_boite` envoie `commande_de_la_boite`, avec `natif::decalage_a`. Sous
  « Ranger par » et « Contenu », chaque critère en `fieldset` à légende : « Date : » (deux `<input type="date">`
  libellés « du » et « au ») ; « Étiquette : » (un `<select>` : « Toutes », puis `etiquettes_du_fonds`, chacune écrite
  comme la liste l'écrit, `#` devant : `#pont`) ; « Durée : », « État : », « Provenance : »
  (des choix en ligne, comme « Ranger par »). Chaque changement d'un réglage ou du texte ramène la liste au début
  (`au_debut`) et redemande ; `demander_la_boite` lit la requête en cours. Le compte : la première ligne, celle des non reliées,
  inchangée ; la seconde, `texte_de_la_requete`. La région du compte reste `aria-live="polite"` : chaque réponse en
  remplace le texte (question 27). Une ligne ouvre l'idée entière, comme avant (question 5). « Contenu : » passe à la requête :
  `EtatDeLaBoite.contenu` (`:463`) se retire, ses boutons lisent et écrivent `etat.requete.contenu` ; `BoiteLue` garde la
  requête lue avec sa vue (`BoiteLue { fenetre, requete, vue }`, `filtree` retiré), que `texte_de_la_requete` lit. Une
  requête posée par `poser` se voit ainsi dans « Contenu : ».
- [ ] **Étape 4 : voir le vert.**
- [ ] **Étape 5 : committer** `git add app/ui/src/boite.rs app/ui/style.css` (`feat: les critères à l'écran, la boîte
  réponse de la requête en cours`).

### Tâche 14 : Échap, et la page d'un fonds vide

**Fichiers :** modifier `app/ui/src/boite.rs`, `app/ui/style.css`.

**Interfaces — consomme :** `echap_ferme`, `est_la_recherche` (tâche 11) ; `Volet`, `fermer_le_volet`, `fermer_l_idee` (`boite.rs:1104`,
`:1111`) ; `EtatDuFonds::lecture_seule`.

- [ ] **Étape 1 : écrire les tests** :
  - `le_fonds_vide_se_dit_par_des_constats` : `page_du_fonds_vide(considerees: u64, lecture_seule: bool) ->
    Option<[&'static str; 2]>` : `None` dès qu'une idée est interrogée ; sinon « Le fonds ne porte encore aucune
    idée. » et « Une idée se note dans le champ « Noter une idée ». », ou, en lecture seule, « Le fonds ne porte encore
    aucune idée. » et « Le fonds est en lecture seule : le bandeau dit pourquoi. » ; aucun des textes ne commence par un
    verbe à l'impératif (« Notez », « Importez », « Créez ») ;
  - `echap_ferme_le_volet_et_rend_le_focus_comme_fermer` : la fonction que l'écoute appelle rend la même cible de focus
    que « Fermer », pour le volet de l'idée (la ligne qui l'a ouvert) et pour celui des retirées (le bouton « Idées
    retirées »).
- [ ] **Étape 2 : voir le rouge**, committer (`test: Échap et le fonds vide`).
- [ ] **Étape 3 : écrire le code.** L'écoute de `keydown` de la boîte (`:1324`) joue aussi Échap par `echap_ferme`, son
  `dans_la_recherche` valant `est_la_recherche` de l'`id` de la cible de l'événement (`Element::id`, une chaîne vide
  pour une cible sans `id`), le
  volet de l'idée, celui des retirées et celui du journal (tâche 16) ; la page du fonds vide prend la place de la liste
  quand `assiette_du_compte.considerees` vaut 0, Pupitre neuf compris, au-dessous du champ « Noter une idée », qui reste
  sauf en lecture seule ; le bouton « Idées retirées » reste (question 26). Une recherche qui ne trouve rien n'est pas
  un fonds vide : la liste vide et son compte (« 0 idée · 12 interrogées »). Pas de fil d'Ariane au lot 1 : la limite se
  dit dans la PR. Le bouton « Annuler » porte `aria-keyshortcuts="Control+Z"`.
- [ ] **Étape 4 : voir le vert.**
- [ ] **Étape 5 : committer** `git add app/ui/src/boite.rs app/ui/style.css` (`feat: Échap ferme les volets, et la page
  d'un fonds vide`).

### Tâche 15 : les requêtes enregistrées à l'écran

**Fichiers :** modifier `app/ui/src/boite.rs`, `app/ui/style.css`.

**Interfaces — consomme :** les commandes `REQUETES_ENREGISTREES`, `ENREGISTRER_UNE_REQUETE`, `RENOMMER_UNE_REQUETE`,
`SUPPRIMER_UNE_REQUETE` (tâche 7) ; `EtatDeLaRequete::poser` (tâche 11) ; `etat.apres_un_geste` (le bandeau d'un refus).

- [ ] **Étape 1 : écrire les tests** :
  - `une_requete_plus_recente_se_nomme_et_ne_se_lance_pas` (point de revue 3) : `ce_que_dit_la_requete(r:
    &RequeteEnregistree) -> (String, Option<&'static str>)` : son nom, et, de `requete: None`, « Cette requête vient
    d'une version plus récente de Pupitre. » ; `lancer(etat, r)` ne change rien d'`EtatDeLaRequete` pour celle-là, et
    pose les réglages d'une requête lisible ;
  - `supprimer_la_requete_en_cours_laisse_ses_criteres` (point de revue 4) : la requête lancée, puis supprimée : chaque
    signal d'`EtatDeLaRequete` garde sa valeur ;
  - `enregistrer_ne_s_offre_qu_a_une_requete_posee` : `enregistrer_possible(requete, nom) -> bool` : faux pour une
    requête vide ou un nom de blancs.
- [ ] **Étape 2 : voir le rouge**, committer (`test: les requêtes enregistrées à l'écran`).
- [ ] **Étape 3 : écrire le code.** Une zone « Requêtes enregistrées » dans la vue du fonds : la liste, rangée par la
  commande, chaque requête un bouton qui la lance (`aria-disabled` et sa note pour une plus récente) ; un champ « Nom »
  et « Enregistrer la requête » (`aria-disabled` tant que `enregistrer_possible` est faux) ; pour la requête choisie,
  « Renommer » (le même champ) et « Supprimer », sans confirmation (question 19). Chaque refus passe par
  `etat.apres_un_geste`, au bandeau (question 20) ; la liste se relit après chaque geste fait.
- [ ] **Étape 4 : voir le vert.**
- [ ] **Étape 5 : committer** `git add app/ui/src/boite.rs app/ui/style.css` (`feat: les requêtes enregistrées à
  l'écran`).

### Tâche 16 : le journal à l'écran

**Fichiers :** modifier `app/ui/src/boite.rs`, `app/ui/style.css`.

**Interfaces — consomme :** la commande `JOURNAL` et `VueDuJournal` (tâche 9) ; `heure_du_journal`,
`empreinte_courte` (tâche 9) ; `natif::decalage_a` ; la fenêtre et la liste de hauteur fixe des retirées.

- [ ] **Étape 1 : écrire les tests** :
  - `chaque_ligne_du_journal_dit_son_heure_et_son_message` : `ligne_du_journal(entree, decalage) -> String` :
    « — · Genèse du fonds » pour l'heure 0 ; « 25 oct. 2026 · 02:30 · Idée du 25 octobre notée » pour 1 792 888 200 au
    décalage 120 ;
  - `l_entete_du_journal_dit_son_nombre_et_son_empreinte` : `entete_du_journal(total, empreinte) -> String` :
    « 1 entrée · empreinte 570cda », « 3 entrées · empreinte 570cda ».
- [ ] **Étape 2 : voir le rouge**, committer (`test: le journal à l'écran`).
- [ ] **Étape 3 : écrire le code.** Un bouton « Journal du fonds » ouvre un volet en lecture seule, comme celui des
  retirées : l'en-tête du journal, puis la liste en fenêtre, des lignes de hauteur fixe, chacune au décalage de son
  instant (`natif::decalage_a(heure)`), la plus récente d'abord ; « Fermer » et Échap le ferment, le focus rendu au
  bouton.
- [ ] **Étape 4 : voir le vert.**
- [ ] **Étape 5 : committer** `git add app/ui/src/boite.rs app/ui/style.css` (`feat: le journal du fonds à l'écran`).

### Tâche 17 : les documents de la PR (b), les délais, et la revue de la PR (b)

**Fichiers :** `docs/20-sfd/21-capter-et-organiser.md` (CAP-07), `docs/40-architecture.md` (§ 9),
`docs/50-interfaces-et-appareils.md` (§ 2, § 3.1, § 5, § 7), `docs/assets/maquettes/fonds.svg`,
`docs/70-qualite-et-verification.md` (§ 2, § 3, § 7, § 8), `docs/80-glossaire.md`, `docs/pistes.md`, `docs/registre.md`
(LE-16), `CONVENTIONS.md`, `CLAUDE.md`, `ci.yml` (les délais).

- [ ] **Étape 1 : la recherche de LE-2**, citée dans la description de la PR : `bandeau`, `en-tête`, `entête`,
  `synchronisation`, `archive`, `Pupitre neuf`, `fonds vide`, `Échap`, `raccourci`, chacun aussi sans accent et sur la
  majuscule accentuée ; maquettes et commentaires SVG compris (LE-3).
- [ ] **Étape 2 : l'écran.** § 50 · 3.1 (`:105`) : la ligne « Bandeau » devient « En-tête de l'établi | Recherche ; état
  de synchronisation … ; « Afficher » : Fonds ou Table » ; la zone « Boîte d'idées » dit la liste, réponse de la requête
  en cours, ses critères, les requêtes enregistrées et le journal du fonds, à un geste. § 50 · 2 (`:88`) : au lot 1,
  Échap ferme le volet de l'idée, des retirées ou du journal ; le fil d'Ariane vient avec les échelles d'INC-2 ; la boîte
  est une zone du fonds, et le diagramme le dit. § 50 · 5 : l'exemple d'assiette d'une requête. § 50 · 7 : les
  raccourcis se documentent par `aria-keyshortcuts` et l'aide du champ. La maquette du fonds (`fonds.svg:24`, `:30`,
  `:35`) : les dates absolues de la liste (« 3 nov. 2026 »), sans heure.
- [ ] **Étape 3 : la SFD, l'architecture, la qualité.** CAP-07 : la page d'un fonds vide (aucune idée interrogée ; un
  Pupitre neuf montre la même ; noter, sauf en lecture seule ; importer et réimporter viennent aux tranches 5 et 6).
  § 40 · 9 (`:738`) : la vue du fonds du lot 1 laisse les morceaux, les albums, les captations à envoyer et la santé du
  nœud ; la santé du nœud entre aux reports, au lot 4. § 70 · 2 et § 70 · 3 (I31) : le texte piégé se saisit aussi dans
  une étiquette et dans le nom d'une requête. § 70 · 7 : « le même journal » se vérifie par le nombre d'entrées et
  l'empreinte courte, que montre la vue, avant et après l'installation. § 70 · 8 : où en est le tableau.
- [ ] **Étape 4 : le glossaire, les pistes, le registre.** Le glossaire (questions 22, 31) : « Bandeau » perd sa
  dernière phrase et dit que l'état de synchronisation n'en est pas un ; « En-tête de l'établi » ; « État de
  synchronisation » ; « Archive complète » ; « Pupitre neuf ». `docs/pistes.md` : « La zone du haut de l'écran du
  fonds » sort ; « La boîte de l'interface en un module » reste, la tranche 4 ayant mis la recherche à part ; « Les
  avertissements nés après l'ouverture » relue, gardée. Le registre : LE-16 se clôt, son critère de sortie tenu.
  `CONVENTIONS.md`, `CLAUDE.md` (« Les crates » : l'interface, l'en-tête, la recherche, les requêtes enregistrées, le
  journal).
- [ ] **Étape 5 : les délais** de `verifier` et de `windows`, comme à la tâche 10.
- [ ] **Étape 6 : committer** les documents (`docs: la vue du fonds au glossaire, à la SFD et aux portes ; LE-16 clos`).
- [ ] **Étape 7 : la revue de la PR (b).** La revue de la PR entière, sur sa tête, par le modèle le plus capable :
  chaque écran au clavier, ses libellés au vocabulaire du glossaire, aucune information portée par la seule couleur,
  la recherche de LE-2. Puis les corrections, leur relecture sur la tête qui part. Le corps dit les limites : pas de
  fil d'Ariane ; la ligne d'un résultat sans marque du mot trouvé ; « Écarts à la spec : Aucun. ».

### Tâche 18 : l'essai installé et son relevé

- [ ] **Étape 1 : préparer l'essai.** L'installateur `etabli-installateur` de l'exécution verte de la tête relue
  (tâche 17) ; la liste de l'essai, dans le dossier temporaire, à remettre à la maîtrise d'ouvrage. Le fonds d'essai de
  la tranche 3 se garde : la genèse n'a pas changé.
- [ ] **Étape 2 (👤) : l'essai.** La maîtrise d'ouvrage installe l'établi par-dessus celui de la tranche 3, sur son
  fonds d'essai, puis, chaque point noté « vu » ou « écart » :
  - l'établi s'ouvre sur le fonds ; « Sans nœud · aucune archive » paraît sur le fonds et sur la table ;
  - `/` hors d'un champ porte le focus dans la recherche, vide ; `/` dans la note s'écrit ;
  - « eglise » trouve « Église du port » ; le compte s'annonce au lecteur d'écran, sans s'empiler à chaque frappe ;
  - chaque critère, puis deux ensemble ; une période sur un jour ;
  - une requête enregistrée, lancée, renommée, supprimée ; un nom piégé et une étiquette piégée s'affichent tels quels ;
  - Échap ferme le volet de l'idée, des retirées, du journal ; dans la recherche, il vide le champ et ne ferme aucun
    volet ;
  - le journal du fonds : la genèse en `—`, le nombre d'entrées et l'empreinte courte, relevés, puis les mêmes après
    une réouverture ;
  - un fonds neuf (un second dossier de données, si la maîtrise d'ouvrage le veut) montre la page du fonds vide.

  Un écart se corrige dans la PR (b), revu, puis l'essai reprend.
- [ ] **Étape 3 : le relevé.** Au § 70 · 7, une ligne : la date, le SHA de la tête essayée, chaque point et ce qu'il a
  montré, sur la parole de la maîtrise d'ouvrage, sans donnée personnelle ni chemin de la machine. Committer (`docs: le
  relevé de l'essai installé de la tranche 4`).
- [ ] **Étape 4 : la relecture du relevé**, sur la tête qui part ; le corps final ; `gh pr ready` ; la fusion par squash,
  `--match-head-commit` nommant cette tête.

### Tâche 19 : la clôture de la tranche

- [ ] Le clone principal à jour (`git fetch`, `main..origin/main` vide) ; les worktrees de la tranche retirés ; l'espace
  de travail SDD de la tranche supprimé ; la mémoire du lot mise à jour (la tranche 4 close, ses PR, ce qu'elle laisse
  aux suivantes : l'index rebâti à la lecture, `empreinte_du_journal` pour les tranches 6 et 8, la commande unique de la
  recherche pour la tranche 7). Ce plan reste dans `.claude/plans/` jusqu'à la clôture du lot, que la tranche 9 conduit.

---

## Auto-relecture

- **La couverture.** Chaque puce de « Elle livre » du plan maître a sa tâche : les requêtes au noyau, par date,
  étiquette, durée, état, provenance, contenu et texte, la table de repli, l'assiette, les retirées jamais interrogées,
  chercher n'écrit rien (1 à 5) ; l'index jetable, I13 (3 à 5) ; les requêtes enregistrées, I01 étendu (6 à 8, 15) ; la
  vue du fonds, la recherche, « Sans nœud · aucune archive », la boîte, le fonds vide, un résultat qui ouvre l'idée, `/`
  (11 à 14) ; le journal en lecture seule (9, 16). Chaque critère de sortie a son test nommé : CAP-06 · 7 à 10 (5, sur
  les deux paires, inscrits en 10), CAP-02 · 5 en part (5), I09 (5), I13 (5), I01 (7), la paire de fichiers dans
  `windows` (5, 7), `mutants` (1 à 3, 6, 9), les délais (10, 17). L'essai 👤 (18).
- **Les types.** `Requete`, `Jour`, `Bornes`, `Sans` du noyau (2, 3) ont leurs doubles dans `commandes` (4), que les
  services convertissent ; `Index` et `Reponse` (3) servent aux tâches 4 et 5 ; `RequeteEnregistreeLue` (6) à la
  tâche 7, `RequeteEnregistree` (7) à la tâche 15 ; `VueDuJournal`, `heure_du_journal`, `empreinte_courte` (9) à la
  tâche 16 ; `EtatDeLaRequete` (11) aux tâches 12 à 15.
- **Les points de revue** ont chacun leur test : 1 (5, 13), 2 (7), 3 (6, 15), 4 (15), 5 (11).
- **Les nombres recopiés** l'ont été du passage de LE-6, recalculés : les instants du 25 octobre et du 29 mars 2026
  (1 792 879 200, 1 792 969 200, 1 774 738 800, 1 774 821 600 ; A, B, C à 1 792 881 000, 1 792 967 400,
  1 792 971 000), les minuits (951 782 400, 951 868 800, 1 792 886 400, 1 798 761 600), l'empreinte du journal d'exemple
  (570cda…), la table de repli (cent vingt entrées, dont quatre-vingt-dix-sept lettres), et chaque réponse des jeux
  inventés, rejouée par un script sous la règle des candidates et de la seconde affichée.
