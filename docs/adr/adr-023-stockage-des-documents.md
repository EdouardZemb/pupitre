# ADR-023 — Le stockage des documents

| | |
|---|---|
| **Statut** | ✅ acceptée |
| **Date** | 2026-11-04 |
| **Sert** | B01, B13, B14 · principes P5, P8 · contre le risque R6 |
| **S'appuie sur** | [ADR-002](adr-002-local-first.md), [ADR-003](adr-003-crdt-et-miroir-durable.md), [ADR-006](adr-006-le-partiel-est-normal.md) et [ADR-022](adr-022-documents-cote-natif.md) |

## Contexte

Elle amende l'[ADR-002](adr-002-local-first.md), qui reste acceptée et dont le texte ne
change pas. L'ADR-002 dit du nœud maison que, « sans lui, Pupitre fonctionne sur un seul
appareil », et qu'« aucun appareil n'a besoin d'un autre pour travailler ». Or l'établi et
le carnet travaillent sans nœud jusqu'au lot 3 d'INC-1
([§ 40 · 9](../40-architecture.md#9-trajectoire-de-construction)), chacun sur sa copie du
fonds, et le second part d'une copie du premier.

Les décisions de Sacha vivent dans des documents Automerge
([ADR-003](adr-003-crdt-et-miroir-durable.md)), tenus sur fichiers, du côté natif de chaque
application ([ADR-022](adr-022-documents-cote-natif.md)). Rien ne disait encore comment un
document se range sur le disque, comment son historique reste entier sans grossir sans fin,
ni comment il suit les versions de Pupitre : le
[§ 31](../31-persistance-et-synchronisation.md) le laissait « à définir avant INC-1 ». INC-1
fait entrer les premières données réelles : ce qui s'écrit alors devra se relire dans des
années.

Ce que fait Automerge, lu dans la documentation et le code de la crate `automerge` 0.12.0,
dans la spécification de son format binaire et dans son guide de modélisation :

- `save` enregistre tout le document ; `save_after(heads)`, les seules modifications qui ne
  sont ni dans `heads` ni parmi leurs dépendances, directes ou transitives : celles faites
  depuis `heads`, ou à côté. `save_incremental`, d'un `AutoCommit`, rend ce qui suit le
  dernier `save` ou `save_incremental` de cette instance : une instance que `load` vient de
  lire n'en a aucun, et il rend alors tout l'historique ; `load_incremental` ne l'avance pas.
  `load` et `load_incremental` relisent l'un et l'autre.
- Un fichier Automerge est une suite de tronçons (*chunks*), chacun avec sa somme de
  contrôle : un tronçon altéré arrête la lecture. `load` refuse alors le fichier ;
  `load_incremental`, sur un document déjà lu, s'arrête au premier tronçon altéré ou
  incomplet sans le dire, et rend ce qu'il a lu. Un tronçon de document « always contains a
  complete history of changes ».
- Une modification dont les dépendances manquent attend, sans être appliquée ; une
  modification déjà appliquée est ignorée.
- Chaque modification porte son acteur, son heure et son message. Les modifications d'un
  même acteur doivent se suivre sans trou.
- Initialiser un document sur chaque appareil « will not work » : deux copies sans ancêtre
  commun ne fusionnent pas proprement. Le guide recommande une première modification écrite
  d'avance, la même partout, et une migration par version, écrite d'avance elle aussi, qui
  ne dépend que de la précédente.

Quatre faits du projet pèsent :

- Une écriture peut être coupée : une batterie vide, une application tuée
  ([CAP-01](../20-sfd/21-capter-et-organiser.md#cap-01--capter-en-un-geste) · 2).
- Le journal et le retour avant une opération demandent toute l'histoire
  ([§ 31 · 3](../31-persistance-et-synchronisation.md#3-les-opérations-nommées)).
- Les applications s'installent à la main, appareil par appareil
  ([§ 40 · 8](../40-architecture.md#8-déploiement)) : deux versions de Pupitre se croisent un
  temps.
- Un document reconstruit depuis le miroir a une autre histoire : fusionné avec une copie
  restée sur un appareil, chacun de ses éléments s'y heurterait à son double : dans un
  objet à clés, l'un cacherait l'autre, avec ce qu'on lui a écrit depuis ; dans une liste,
  les deux paraîtraient ([§ 31 · 8.1](../31-persistance-et-synchronisation.md#81-ce-que-cest)).

## Décision

Le détail est au [§ 31 · 2](../31-persistance-et-synchronisation.md#2-les-documents) et au
[§ 31 · 9](../31-persistance-et-synchronisation.md#9-sauvegarde-et-restauration).

1. **Un dossier par document**, sous la racine des données du fonds : un fichier complet,
   tout le document, historique compris ; puis un fichier d'ajout par écriture faite depuis,
   nommé par son empreinte.
2. **Chaque fichier s'écrit à côté, puis se renomme**, comme l'export du cahier des tables.
   Une opération nommée n'est faite qu'une fois son fichier d'ajout en place. Si l'écriture
   échoue, l'opération est refusée, et le document se relit du disque : la mémoire ne garde
   jamais ce que le disque n'a pas. Un fichier resté provisoire mais entier ne se perd pas au
   démarrage : un fichier d'ajout se reconnaît à son empreinte et prend sa place ; un
   fichier complet, que `load` lit sans erreur, se lit avec le document, et la compaction de
   l'ouverture le fond. Une opération refusée retire le sien. Un document n'est lu par la synchronisation qu'entre deux opérations : une
   modification ne part vers le nœud qu'une fois son fichier d'ajout en place.
3. **La compaction réécrit le fichier complet**, historique compris, le relit, puis retire
   les seuls fichiers d'ajout qu'il absorbe. Elle ne retire jamais d'histoire.
4. **Chaque sorte de document part de sa genèse**, une première modification rangée en
   octets dans le code. Le fonds de Sacha naît une seule fois ; tout autre appareil part
   d'une copie de ce fonds, historique compris.
5. **Chaque document porte sa version de schéma.** Une version de schéma monte par une
   migration écrite d'avance, qui ne dépend que de la précédente, et ne touche que la racine
   du document.
   Une application lit et écrit les versions de schéma qu'elle connaît ; devant une version
   de schéma plus récente, elle lit sans écrire, et le dit. Une captation n'est jamais
   refusée pour autant.
6. **Chaque document porte sa lignée.** Reconstruit hors de son histoire, il en ouvre une
   nouvelle ; restitué avec son histoire, il garde la sienne. Deux copies de lignées
   différentes ne fusionnent jamais : la synchronisation les compare avant tout échange.
7. **La teneur d'un document** est ce qu'il dit, sans son histoire, écrite de façon
   déterministe. C'est elle que comparent l'aller-retour du miroir, le test de restauration
   et la réimportation de l'archive.
8. **Une captation se range dans le sous-dossier de son premier octet** : deux caractères
   hexadécimaux, comme les objets de git.
9. **Sans nœud, la sauvegarde est l'archive complète** : documents, historique compris,
   captations et, dès que le miroir existe, sa projection, dans un dossier qui se vérifie
   sans Pupitre. La réimporter, dans un Pupitre neuf, c'est repartir après un sinistre.
10. **Au nœud, l'instantané de sauvegarde se prend une fois le miroir à jour**, et chaque
    commit du miroir dit le point d'histoire qu'il projette.

**Ce qu'elle amende.** [ADR-002](adr-002-local-first.md), « Décision » : les phrases citées
plus haut se lisent désormais ainsi. Le nœud maison est recommandé, pas indispensable. Sans
lui, rien ne se synchronise : chaque appareil travaille sur sa copie, et la sauvegarde passe
par l'archive complète. Un appareil de plus part d'une copie du fonds d'un autre, historique
compris ; une fois cette copie reçue, il n'a plus besoin de lui pour travailler. Leurs
copies fusionnent quand le nœud arrive
([§ 31 · 4.1](../31-persistance-et-synchronisation.md#41-la-topologie)). Le reste de
l'ADR-002 ne change pas.

## Conséquences

### Ce que ça permet

- Une écriture coupée ne coûte que l'opération en cours, jamais un document : un fichier est
  entier, ou absent.
- Un geste n'écrit que ses propres modifications, pas tout le document.
- Le journal et le retour avant une opération survivent à la compaction, à l'archive et à la
  restauration.
- L'établi et le carnet travaillent sans nœud, puis fusionnent quand il arrive : ils partent
  du même fonds.
- Deux versions de Pupitre se croisent sans rien abîmer.
- Une reconstruction depuis le miroir ne fusionne jamais avec l'ancienne lignée : rien ne
  s'y double ni ne s'y cache.
- Une seule dépendance de plus pour le stockage, `getrandom`, pour l'aléa des identifiants,
  la lignée comprise
  ([§ 41 · 6](../41-pile-technique.md#6-le-budget-de-dépendances)) ; Automerge, `sha2` et le
  système de fichiers suffisent au reste, renommage compris.

### Ce que ça coûte

- La genèse et chaque migration se figent à leur publication, c'est-à-dire à la première
  étiquette `vX.Y.Z` qui les porte : `v0.3.0` pour la genèse du fonds. Avant elle, aucune
  construction ne reçoit de donnée réelle, ni l'essai H07, ni les constructions du lot 1,
  qui gardent l'identifiant `app.pupitre.etabli` : rien de ce qu'elles écrivent n'entre
  dans le dossier de la `v0.3.0`
  ([§ 40 · 8](../40-architecture.md#8-déploiement)). Une erreur s'y corrige
  ensuite par une migration de plus, jamais en réécrivant.
- Une migration ne touche que la racine du document. Changer la structure d'objets
  existants, au-delà d'un ajout, demandera une décision écrite, et sans doute une
  reconstruction dans une lignée nouvelle.
- Un document migré n'accepte plus d'écriture d'une application plus ancienne : chaque
  appareil se met à jour, le nœud d'abord.
- Le fonds ne naît qu'une fois : un second appareil en reçoit une copie avant d'y écrire.
- Ce qui s'écrit dans un document reste dans son historique. Retirer une idée ne l'ôte ni de
  la teneur ni de l'histoire : elle est seulement marquée. Une purge définitive, conçue plus
  tard, ne pourra effacer l'histoire qu'en reconstruisant le document depuis sa teneur
  purgée, dans une lignée nouvelle.
- Une copie d'une lignée remplacée garde, à part, ce qu'elle seule avait : rien ne la fusionne
  d'office.
- Un fichier par écriture : de petits fichiers, que la compaction tient en nombre.
- L'écriture de la teneur ne change jamais, pour qu'une archive ancienne se compare encore.

## Options écartées

| Option | Pourquoi |
|---|---|
| Un seul fichier, réécrit en entier à chaque écriture | Chaque geste réécrit tout le document, historique compris, pour quelques modifications ; sur le téléphone, c'est de l'usure et de l'attente pour rien |
| Un seul fichier, où chaque écriture s'ajoute à la fin | Une écriture coupée laisse un tronçon incomplet, et un tronçon altéré arrête la lecture : il faudrait réparer le fichier à l'ouverture. Un fichier par écriture, renommé une fois entier, n'a rien à réparer |
| Une base SQLite | Les documents sont des octets et les captations des fichiers : une base n'y ajoute rien ([§ 41 · 4](../41-pile-technique.md#4-les-choix-couche-par-couche)) |
| Une compaction qui oublie l'histoire ancienne | Le journal et le retour avant une opération en dépendent ; et seules deux copies d'une même histoire fusionnent |
| Chaque appareil crée son fonds, et la synchronisation les réunit | Deux naissances font deux documents, d'identifiants différents, que la synchronisation n'apparie jamais ([§ 31 · 2.3](../31-persistance-et-synchronisation.md#23-la-genèse-et-le-point-de-départ-du-fonds)) ; réunis de force, leur identifiant, leur lignée et leur identifiant de propriétaire se heurteraient, et l'appareil ne saurait plus quel fonds il ouvre. Sans genèse commune, leurs conteneurs de la racine du document se disputeraient en plus les mêmes clés, et l'un cacherait l'autre |
| Une migration calculée sur chaque appareil | Deux appareils qui migrent le même document de leur côté écrivent deux modifications différentes, qui se heurtent ; écrite d'avance, la migration est la même partout |
| Aucune version de schéma : chaque champ nouveau se lit absent | Suffit pour ajouter, pas pour changer une structure ; et une application ancienne écrirait sans le savoir dans un document qu'elle ne comprend plus |
| Remplacer toute copie à chaque réimportation | Un document restitué avec son histoire fusionne sans rien doubler ; le remplacer mettrait à part, sans raison, ce qu'un autre appareil a écrit depuis |
| Deux octets d'empreinte pour le sous-dossier | 65 536 sous-dossiers possibles, pour quelques milliers de captations au fonds visé : presque chacune aurait le sien, à créer, copier et sauvegarder pour rien |
| Une archive en un seul fichier compressé | Une dépendance de plus, et une archive qu'il faut ouvrir avant de la vérifier ; un dossier se copie, se relit et se vérifie sans outil de plus |

## Comment on vérifie

- Les tests 8 à 14 du
  [§ 31 · 10](../31-persistance-et-synchronisation.md#10-les-tests-qui-verrouillent-ce-document) :
  l'écriture coupée, la compaction coupée, un document de version de schéma N + 1 ouvert par
  une application qui ne connaît que la version de schéma N et deux migrations séparées, la
  genèse et la teneur figées, la lignée, le fonds de mesure, à la taille visée du
  [§ 31 · 2](../31-persistance-et-synchronisation.md#2-les-documents), l'archive réimportée.
- **I12** et **I14** ; **I28**, pour lequel la genèse et chaque migration portent leur
  message ([§ 70 · 3](../70-qualite-et-verification.md#3-les-invariants-et-leurs-tests)).
- Les règles qui décident — quelle lignée l'emporte, quand une version de schéma oblige à
  lire sans écrire, quels fichiers une compaction retire, l'écriture de la teneur — sont des fonctions
  pures du noyau : la mutation les éprouve.
- **H07** se joue sur ce format
  ([§ 40 · 11](../40-architecture.md#11-hypothèses-à-confronter)).
