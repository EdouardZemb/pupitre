# 90 — Questions ouvertes et arbitrages

Ce document rassemble ce qui **n'est pas tranché**, trié par ce que chaque question
débloque, avec son enjeu et une recommandation. Puis les **arbitrages** pris pendant la
conception, avec ce que coûterait leur renversement : aucun n'est irréversible.

---

## Ce qui a été tranché au cadrage du 2026-10-11

| Sujet | Décision | Où elle s'applique |
|---|---|---|
| Pour qui | Un **profil** — le persona Sacha —, pas une personne nommée | [§ 11](11-persona-et-situations.md) |
| Point de départ | La conception part **du besoin** — travailler une composition du motif à l'album, avec toutes les étapes entre —, pas de l'existant | [§ 12](12-continuum-de-composition.md) |
| Mobilité | **Tout fonctionne hors ligne**, sauf l'analyse lourde et l'IA générative, qui attendent le retour au foyer ; synchronisation au retour, **aucune exposition publique** | [ADR-002](adr/adr-002-local-first.md), [ADR-021](adr/adr-021-cle-du-noeud-epinglee.md) |
| Son | **Écoute de contrôle** : vérifier l'écrit, sans production | [ADR-009](adr/adr-009-ecoute-de-controle.md) |
| IA | Détection, **et pistes de contenu** (accords suivants, rimes, variantes), toujours sous décision humaine. Lu comme incluant l'explication des constats par un modèle — à confirmer (A15) | [ADR-004](adr/adr-004-proposer-n-est-pas-decider.md), [ADR-005](adr/adr-005-des-pistes-pas-des-reponses.md) |
| Ancienne documentation | **Remplacée** ; elle reste dans l'historique git | [README](../README.md) |

---

## 🔴 Bloque le début de l'implémentation

Aucune question ouverte ne bloque plus le début de l'implémentation : Q02 est tranchée
(plus bas). Les trois ADR qu'INC-0 met en jeu et qui restaient proposées,
[001](adr/adr-001-un-seul-materiau.md), [008](adr/adr-008-hauteurs-relatives.md) et
[010](adr/adr-010-trois-appareils-mode-encre.md), sont acceptées depuis le 2026-10-28, et
les questions sur ce qu'il calcule sont tranchées le même jour : le plan d'INC-0 peut
s'écrire ([§ 40 · 9](40-architecture.md#9-trajectoire-de-construction)).

---

## 🟠 Bloque une décision d'architecture

### Q04 — Quel chemin d'annotation par défaut au lutrin ?

**Enjeu.** Annoter dans Pupitre (simple, intégré) ou par aller-retour PDF dans le lecteur
natif (plus rapide au stylet) — 🔶 H02, [ADR-011](adr/adr-011-encre-ancree-rendu-fige.md).

**Recommandation.** Essayer les deux sur la tablette avec une fiche de test, et retenir par
défaut celui dont le tracé suit le geste. Garder l'autre en option.

### Q05 — Quel fournisseur d'IA, et quel consentement par défaut ?

**Enjeu.** Confidentialité des paroles, qualité des pistes, coût.

**Recommandation.** **Local par défaut** si Q01 le permet ; **distant sur consentement par
demande**, avec l'API Claude et `claude-opus-5-5` par défaut, revu après le banc des pistes
([§ 70 · 6](70-qualite-et-verification.md#6-le-banc-des-pistes-génératives)).

### Q07 — Synchroniser hors du foyer par un réseau privé ?

**Enjeu.** Sans synchronisation dehors, une captation n'existe que sur le téléphone jusqu'au
retour (risque R6). Un réseau privé virtuel, si la maîtrise d'ouvrage en emploie déjà un,
permettrait de la synchroniser plus tôt, sans rien exposer.

**Recommandation.** **L'autoriser comme option**, sans qu'aucune fonction n'en dépende ni
qu'aucun test ne l'exige. La règle « rien d'exposé sur Internet » ne bouge pas.

---

## 🟡 Bloque un domaine fonctionnel

### Q01 — Quelle est la capacité du nœud maison ?

**Enjeu.** Processeur, mémoire, présence d'une carte graphique : de là dépendent la durée
des analyses lourdes (🔶 H03) et la possibilité d'un modèle de langage local (🔶 H06). Ni
l'une ni l'autre ne change l'architecture ; les deux changent l'expérience. La question
débloque les incréments INC-4 et INC-6, pas le début : synchroniser et sauvegarder ne
demandent presque rien.

**Recommandation.** Mesurer plutôt que supposer : trois prises du fonds (séparation,
accords, transcription) et dix demandes de variantes en local, sur la machine réelle, pour
mesurer les durées. Une heure, et les choix de modèles se font sur pièce ; la qualité des
pistes se juge ensuite au banc du [§ 70 · 6](70-qualite-et-verification.md#6-le-banc-des-pistes-génératives).

### Q08 — Quel support pour l'album ?

**Enjeu.** Un album pensé pour deux faces impose une coupure et une durée par face
([ALB-01](20-sfd/25-album.md#alb-01--créer-un-album), DG-A9).

**Recommandation.** **Numérique** par défaut ; déclarer les faces si un pressage est
envisagé. C'est une intention, modifiable à tout moment.

### Q09 — Quelle doctrine transcrire d'abord ?

**Enjeu.** Les règles personnelles de Sacha contraignent les pistes et nourrissent des
constats ([§ 60 · 8](60-moteur-musical.md#8-la-doctrine-de-sacha-en-exemples)). Sans elles,
seule la théorie parle.

**Recommandation.** Transcrire **dix à vingt règles**, en commençant par celles qui
contraignent les pistes (rimes, réécriture des lignes décidées), puis celles des constats.

### Q10 — Qui annote le jeu de référence, et quand ?

**Enjeu.** Sans lui, aucun analyseur n'entre et aucune confiance n'est calibrée
([ADR-014](adr/adr-014-analyse-en-deux-etages.md)).

**Recommandation.** La maîtrise d'ouvrage, dès que le tapotage (MOR-03) est livré dans
l'incrément INC-4, **avec Pupitre lui-même** (tapotage, saisie de grille). Une journée,
de six à dix heures, étalée ; chronométrer les trois premières prises pour ajuster.

### Q11 — Quel générateur de maquettes, et quel profil d'outil ?

**Enjeu.** Le brief de génération se conforme au **profil** de l'outil visé : limites de
longueur, syntaxe des balises, nombre d'exclusions ([MOR-07](20-sfd/24-morceau.md#mor-07--maquettes-et-brief-de-génération)).

**Recommandation.** Décrire le profil **en données**, depuis la documentation de l'outil
employé ; le mettre à jour quand l'outil change.

### Q16 — Quelle brique pour l'écoute de contrôle ?

**Enjeu.** La pile Rust ne la tranche pas encore
([§ 41 · 5.4](41-pile-technique.md#54-lécoute-de-contrôle)), et elle bloque
[ECO-01](20-sfd/27-ecoute-et-compagnon.md#eco-01--écoute-de-contrôle), livrée avec INC-3.
Trois exigences la bornent : un graphe fait d'échantillonneurs, d'un lecteur de prise, de
volumes et d'une sortie, sans exportation audio (I21) ; des échantillons servis par le
nœud, jamais par un hôte public, et gardés en cache pour l'écoute hors ligne (I24) ; aucun
environnement de production sonore ([ADR-009](adr/adr-009-ecoute-de-controle.md)).

**Recommandation.** Trancher avant INC-3, au gabarit des couches du § 41 : ses concurrents
écartés, et la fonction de la SFD qui la justifie.

---

## 🟢 Utile, pas bloquant

| # | Question | Recommandation |
|---|---|---|
| Q12 | L'anglais et le français suffisent-ils pour les paroles ? | Oui pour commencer ; ajouter une langue, c'est ajouter un dictionnaire de prononciation et des règles |
| Q13 | Où afficher la part d'origine ? | À la demande seulement, dans la vue de provenance ; jamais en tête de fiche ([PRO-02](20-sfd/26-propositions-suggestions-provenance.md#pro-02--provenance-et-part-dorigine)) |
| Q14 | Le fonds existant contient-il des analyses ou des fichiers MIDI déjà produits ? | Si oui, les importer **en propositions**, avec un mappage déclaré ; ne pas recalculer ce qui est déjà mesuré tant que le banc n'a pas montré mieux ([CAP-03](20-sfd/21-capter-et-organiser.md#cap-03--importer-un-fonds-existant)) |
| Q15 | La tessiture : déclarée ou mesurée ? | Mesurée : chanter sa note la plus grave et la plus aiguë tenables, trois minutes |

---

## ✅ Tranchée le 2026-10-27

### Q02 — Quel est le système du téléphone ?

**Tranchée le 2026-10-27 : Android.** Le téléphone, comme la tablette à encre, est sous
Android.

**Ce que ça règle.** Le carnet est une application Tauri 2 pour Android, comme le lutrin
([ADR-020](adr/adr-020-leptos-et-tauri.md)) ; son APK est construit en CI et installé à la
main. Ce que Tauri ne donne pas, démarrer un enregistrement depuis l'écran verrouillé,
passe par un **module Android natif en Kotlin**, aussi mince que possible : c'est l'une des
quatre exceptions de l'[ADR-019](adr/adr-019-rust-et-python.md). L'ancienne
recommandation, une enveloppe native légère pour la seule capture si un prototype ne
tenait pas les 3 s, est devenue la règle : cette enveloppe, c'est ce module.

**Ce qui reste à vérifier.** 🔶 H01 : la capture démarre depuis l'écran verrouillé, par le
module Kotlin, en 3 s au plus, en mode avion et après une veille, sur le téléphone réel
([§ 40 · 11](40-architecture.md#11-hypothèses-à-confronter)). Elle porte la fonction la
plus importante, capter, livrée avec INC-1 ; INC-0 n'en dépend pas.

---

## ✅ Tranchées le 2026-10-28

Six questions sur la table et la palette, tranchées selon leur recommandation. Q17, Q18,
Q20 et Q21 portent sur ce que calcule INC-0.

### Q17 — Quel rôle a la septième d'un accord sans septième ?

**Tranchée le 2026-10-28 : elle frotte.** Le si de `C` et le sol de `Am` frottent : c'est
ce que disent déjà la définition de « frotte » et
[MIC-06](20-sfd/22-micro-motif-et-phrase.md#mic-06--lire-la-relation-mélodie-harmonie), et
I27 tient sans autre changement. Pour en faire un appui, on choisit l'accord de septième
(`Cmaj7`, `Am7`).

**Ce que ça règle.** Les règles et les critères 1 et 3 de
[MIC-13](20-sfd/22-micro-motif-et-phrase.md#mic-13--lire-la-palette-dun-accord) nomment
ces notes.

### Q18 — Que montre la palette d'un accord hors du centre de la table ?

**Tranchée le 2026-10-28 : tous ses appuis.** Les appuis s'affichent tous, même hors de la
tonalité : un accord se lit d'abord par ses notes. Les tensions et les notes qui frottent
restent prises parmi les notes de la tonalité. En do majeur, les appuis de `E♭` sont mi♭,
sol et si♭.

**Ce que ça règle.** La règle de
[MIC-13](20-sfd/22-micro-motif-et-phrase.md#mic-13--lire-la-palette-dun-accord), qui gagne
un critère, et l'entrée « Palette » du [glossaire](80-glossaire.md) : « les notes d'un
accord et de sa tonalité ».

### Q19 — Que devient un renversement épinglé après une transposition ?

**Tranchée le 2026-10-28 : inactif, comme un doigté.** Après une transposition, un
renversement épinglé qui ne correspond plus à aucun accord est listé comme inactif. C'est
un état engendré : rien n'est réécrit, et il redevient actif si l'accord revient.
L'arbitrage A08 vaut désormais pour les renversements.

**Ce que ça règle.** Le [§ 30 · 12](30-modele-de-domaine.md#12-règles-et-réglages), le
[§ 60 · 6.3](60-moteur-musical.md#63-le-voicing-au-clavier) et
[REN-08](20-sfd/28-rendus-encre-impression.md#ren-08--diagrammes-de-clavier) le disent.

### Q20 — Quelle fonction porte chaque degré du centre de la table ?

**Tranchée le 2026-10-28 : le regroupement courant en harmonie de chanson**, celui des
maquettes :
- en majeur, I, iii et vi : `repos` ; ii et IV : `éloignement` ; V et vii° : `tension` ;
- en mineur, i et III : `repos` ; ii°, iv et VI : `éloignement` ; v et VII : `tension`.

**Ce que ça règle.** Les lignes d'effet du centre, que calcule le noyau d'INC-0
([§ 60](60-moteur-musical.md#12-les-familles-daccords)), et la maquette de la table
([§ 50 · 3.7](50-interfaces-et-appareils.md#37-la-table-de-la-tonalité)), qui les suivait
déjà.

### Q21 — Comment s'écrivent les cases de la table ?

**Tranchée le 2026-10-28 : selon A17, sans double altération.** Dans la table et dans le
cahier des tables, l'orthographe est celle de l'arbitrage A17, et la préférence
d'orthographe de [SYN-09](20-sfd/29-socle-donnees-appareils.md#syn-09--régler-ses-préférences)
ne s'y applique pas : la table est la même pour tout morceau
([ADR-018](adr/adr-018-la-table-montre-tout.md)). Une case dont le nom porterait une double
altération s'écrit à l'enharmonique simple, sous le même chiffre romain : `A` pour le ♭VI de
ré♭ majeur, `G°7` pour `F𝄪°7`.

**Ce que ça règle.** [MIC-12](20-sfd/22-micro-motif-et-phrase.md#mic-12--parcourir-la-table-de-la-tonalité),
SYN-09, le [§ 60](60-moteur-musical.md#12-les-familles-daccords) et l'invariant
[I26](70-qualite-et-verification.md#3-les-invariants-et-leurs-tests), qui translate les noms
« à l'enharmonie près pour ces cases ».

### Q22 — Peut-on prendre un accord de la table au lutrin ?

**Tranchée le 2026-10-28 : non.** Au lutrin, page fixe, on ne prend rien ; on prend à
l'établi ou au carnet.

**Ce que ça règle.** La grammaire de la table
([§ 50 · 3.7](50-interfaces-et-appareils.md#37-la-table-de-la-tonalité)) et
[MIC-12](20-sfd/22-micro-motif-et-phrase.md#mic-12--parcourir-la-table-de-la-tonalité) le
disent.

---

## ⚪ Sans objet depuis le 2026-10-27

La pile Rust ([§ 41](41-pile-technique.md)) a retiré leur objet à deux questions. Elles
gardent leur numéro.

### Q03 — Un nom de domaine est-il disponible pour le certificat ?

**Sans objet.** Les applications natives épinglent la clé du nœud
([ADR-021](adr/adr-021-cle-du-noeud-epinglee.md)) : aucun certificat de domaine n'est
nécessaire.

**Si un certificat de domaine redevient utile**, la voie était établie le 2026-10-10 : la
sous-zone d'un domaine existant, par défi DNS-01, avec un certificat générique ; le nom
`_acme-challenge` de la sous-zone est délégué par CNAME à une zone de validation dédiée,
seule modifiable par le jeton d'API ([ADR-016](adr/adr-016-reseau-domestique.md)).

### Q06 — Quel cadriciel pour l'interface ?

**Sans objet.** La question opposait deux cadriciels JavaScript, sous
l'[ADR-013](adr/adr-013-svelte-pour-l-interface.md). L'interface s'écrit désormais en Rust,
avec Leptos ([ADR-020](adr/adr-020-leptos-et-tauri.md)).

---

## Les arbitrages pris dans cette conception

Chacun répond à une absence d'information, pas à un goût. Chacun est réversible, et son
coût est dit.

| # | Arbitrage | Pourquoi | Coût d'un renversement |
|---|---|---|---|
| A01 | Hauteurs stockées **relatives à une tonique de référence** | Transposition légère, refrains modulés, tonalités indécises ([ADR-008](adr/adr-008-hauteurs-relatives.md)) | Élevé : une migration du modèle, à faire avant toute donnée réelle |
| A02 | Un motif placé est **copié avec suivi**, pas lié | Un lien vivant entre documents serait fragile ([§ 30 · 5.4](30-modele-de-domaine.md#54-motifs-placés)) | Moyen : ajouter un type de lien |
| A03 | Une ligne de texte est une **valeur atomique** ; un conflit devient une variante | Une ligne fusionnée caractère par caractère est une ligne que personne n'a écrite | Faible : un réglage de fusion |
| A04 | **Pas de détection automatique** de la langue d'une ligne | Trop peu fiable sur une ligne ; appliquerait les mauvaises règles en silence | Faible : une proposition de langue, à accepter |
| A05 | Au-delà de **50 % de mesures douteuses**, une liste d'indices plutôt qu'une grille | Une grille majoritairement douteuse présentée comme une grille est une affirmation | Faible : un seuil |
| A06 | **Au moins deux pistes**, ou l'aveu | Une seule piste est une réponse déguisée | Faible : un paramètre |
| A07 | **Seuils de départ** des constats (6 BPM, trois voisins, 20 BPM, intensité ≤ 2) | Valeurs courantes de pratique, à régler sur le fonds réel | Nul : des paramètres |
| A08 | À la transposition, un doigté ou un renversement épinglé qui ne correspond plus à aucun accord devient **inactif et listé** — un état engendré, rien n'est réécrit. Étendu aux renversements le 2026-10-28 ([Q19](#q19--que-devient-un-renversement-épinglé-après-une-transposition-)) | Un doigté n'a pas de sens dans une autre tonalité ; un renversement épinglé suit la même règle, puisqu'il est lié comme lui à un accord absolu | Faible : transposer la forme serait une option |
| A09 | Une **mise en page maison**, sans bibliothèque de pagination | La pagination doit être identique au lutrin et sur papier, et déterministe | Moyen : adopter une bibliothèque si elle tient ces deux exigences |
| A10 | **Aucune intégration** d'un générateur de maquettes par son API | Dépendance à un tiers sans bénéfice ; l'export et l'import suffisent | Moyen : un connecteur par outil |
| A11 | Modèle distant par défaut : `claude-opus-5-5` | Le meilleur compromis capacité/prix de la gamme courante ; Claude Fable 5.1, plus capable, coûte 2,5 fois plus. Réglable dans la passerelle, revu après le banc des pistes | Nul : un réglage |
| A12 | Le **lutrin peut accepter ou écarter** une proposition, une à une | Geste non destructif et réversible ; relire au calme est le bon moment pour trancher | Faible : masquer deux boutons |
| A13 | En-têtes du miroir en **TOML**, corps en « un enregistrement par ligne » | Lisibilité des diffs, absence d'ambiguïté de types | Faible : l'exportateur seul change |
| A14 | **Aucune notification**, y compris pour une sauvegarde en échec : un bandeau dans l'application | Anti-besoin N2 ; un bandeau suffit à ne pas l'ignorer | Faible : ajouter un canal |
| A15 | Le choix « pistes de contenu » est lu comme **incluant** l'explication des constats par un modèle | Les options du cadrage étaient cumulatives | Nul : retirer une tâche de la passerelle |
| A16 | **Prendre** un accord dans la table, c'est le **saisir** : provenance `saisie`, part d'origine inchangée | La table montre tout sans choisir : elle ne propose rien ([ADR-018](adr/adr-018-la-table-montre-tout.md)). La carte des chemins, qui part de ce qui précède réellement, reste une piste | Faible : changer la provenance qu'écrit le geste de prendre, et cacher la table en mode sans suggestion ; l'ADR-018 serait remplacée |
| A17 | L'**orthographe des tonalités** : les 24 tonalités, 12 majeures et 12 mineures, s'écrivent chacune avec le moins d'altérations ; à égalité, fa♯ majeur et mi♭ mineur. Chaque case de la table se nomme ensuite dans sa tonalité : en fa♯ majeur, le iii s'écrit `A♯m` ; en mi♭ mineur, le V7/i s'écrit `B♭7` | L'orthographe la plus légère est la plus facile à lire au pupitre. À six altérations, les deux orthographes se valent, et le [cahier des tables](20-sfd/28-rendus-encre-impression.md#ren-01--fiche-de-travail) n'en garde qu'une | Faible : un réglage d'orthographe ; la table se calcule en degrés ([ADR-008](adr/adr-008-hauteurs-relatives.md)), et seuls ses noms changeraient |
| A18 | **INC-0 sans serveur** : la table interactive dans une application de bureau Tauri sur l'établi, le cahier des tables en PDF au lutrin ; Android à partir d'INC-1 | La table ne demande ni stockage ni synchronisation : elle rend un service réel dès le premier incrément, sans attendre le nœud ([§ 40 · 9](40-architecture.md#9-trajectoire-de-construction)) | Faible : le noyau ne dépend pas du nœud ; ajouter le serveur, ce serait avancer une part d'INC-1 |
| A19 | **L'écoute au lutrin, sans curseur** : on y écoute un accord ou une suite au haut-parleur de la tablette, et rien ne bouge à l'écran | La dalle à encre ne suit pas le temps réel, et le mode encre n'a ni animation ni horloge (I18). Le micro existe, mais le carnet reste le compagnon d'instrument ([§ 50 · 1](50-interfaces-et-appareils.md#ce-qui-est-impossible-et-pourquoi)) | Faible : retirer l'écoute du lutrin ; un curseur animé, lui, contredirait I18 |
| A20 | Le PDF par **svg2pdf** plutôt que par Typst | Le PDF n'est que la conversion des pages SVG que l'écran affiche : une seule mise en page, donc la même pagination au lutrin et sur papier (I19). Typst mettrait les pages en page une seconde fois ([§ 41 · 5.7](41-pile-technique.md#57-les-pages-et-le-pdf)) | Moyen : une seconde mise en page, à tenir identique à la première ; H09 la vérifierait |
| A21 | Une **couche de synchronisation maison** plutôt que samod | L'étoile autour du nœud ne demande, pour les documents, que le protocole de synchronisation d'Automerge et le relevé des conflits ; les captations passent par l'adaptateur `Captations` ([§ 31 · 4.4](31-persistance-et-synchronisation.md#44-la-couche-de-synchronisation)) ; samod, le dépôt Automerge en Rust, est jeune et en fait davantage | Moyen : la couche est isolée derrière l'adaptateur `Documents`, et adopter samod ne touche que la part des documents ; H07 en juge |
| A22 | **La page de la table en paysage** : 209,5 × 157 mm, l'écran du lutrin tourné, pour la table, son verso et tout le cahier | En portrait, à 11 pt, « éloignement » (24,4 mm en DejaVu Sans) ne tient pas dans un septième de 157 mm ; en paysage, une colonne fait 27,2 mm ([§ 50 · 3.7](50-interfaces-et-appareils.md#37-la-table-de-la-tonalité)). H09 l'a vérifié au lutrin le 2026-11-02 ; le papier est reporté (A23) | Moyen : revenir au portrait demande des lignes d'effet plus courtes ou une police plus étroite, et une nouvelle fiche témoin |
| A23 | **Le papier reporté après INC-0** : l'impression en noir et blanc du cahier des tables et des diagrammes, et la mesure à la règle sur papier, sortent des critères d'INC-0. Elles se font au premier incrément où une imprimante est disponible. Arbitré par la maîtrise d'ouvrage le 2026-11-02 | Aucune imprimante n'était disponible, ni pour H09 ni pour la revue finale d'INC-0 (EM-1, au [registre](registre.md)). La Note Air affiche la page à sa taille, 209,5 × 157 mm, l'écran tourné : H09, puis la revue finale, y ont vérifié les pages, les glyphes, les débordements, les cadres et les flèches ([§ 70 · 7](70-qualite-et-verification.md#7-les-contrôles-à-lœil-sur-lappareil)). Le papier ne vérifiera plus que ce que l'imprimante ajoute : l'échelle à 100 % et la tenue des traits fins | Faible : un défaut que le papier montrerait se corrige dans le rendu, au lot qui le trouve ; les PDF déjà produits se réengendrent |
