# INC-0 · la table : le plan d'implémentation

> **Pour les agents :** compétence requise : `superpowers:subagent-driven-development` (recommandée) ou
> `superpowers:executing-plans`, tâche par tâche. Les étapes se cochent (`- [ ]`).

**But :** livrer le noyau de théorie en Rust, les rendus SVG de la table, des diagrammes et des palettes,
la table interactive sur l'établi (Tauri, Windows) et le cahier des tables en PDF, lu au lutrin et
imprimable.

**Architecture :** une crate pure, `noyau`, calcule la théorie en degrés (ADR-008) et dessine les pages
en SVG, dans un sous-ensemble que `svg2pdf` convertit fidèlement. La crate `cahier` assemble ces pages en
un PDF. L'application de l'établi, en Leptos dans Tauri 2, reste mince : elle affiche les pages du
noyau, et rien ne s'y calcule (ADR-020).

**Pile :** Rust 1.99.1, édition 2024 ; `ttf-parser`, `sha2` ; `svg2pdf`, `pdf-writer` ; `leptos`,
`tauri` 2 et `tauri-plugin-dialog` ; `trunk` ; tests : `proptest`, `resvg`, `lopdf` ; `cargo-mutants`.

**Spec :** la spec d'INC-0 du 01/11/2026, validée par la maîtrise d'ouvrage. Elle vit hors du dépôt :
ce plan en recopie toutes les décisions qui touchent au code (§ « Les règles »). La conception du
dépôt fait foi pour le reste : MIC-12, MIC-13, REN-01, REN-04, REN-07, REN-08, § 40 · 9, § 41,
§ 50 · 3.7 et 6, § 60, § 70. L'issue du lot : #1.

---

## Les décisions prises en écrivant le plan

La spec a été recalculée avant d'écrire ce plan. Quatre écarts ont été trouvés et soumis à la maîtrise
d'ouvrage, qui a tranché le 01/11 :

| # | Écart trouvé | Décision |
|---|---|---|
| 1 | En portrait, à 11 pt en DejaVu Sans, « éloignement » mesure 24,4 mm. Un septième des 157 mm de la page en fait 22,4, marges nulles. « dominante » (20,9 mm) et « assombrit » (19,2 mm) débordent aussi. La maquette `table.svg` ne tient que parce qu'elle compose ces lignes à 8,5 pt | **La page en paysage**, 209,5 × 157 mm : l'écran du lutrin tourné. Cela vaut pour tout le cahier. H09 vérifie que la hauteur tient |
| 2 | La fiche témoin d'H09 devait être « la page la plus dense », et la spec nommait do majeur. Le mineur est plus dense : 6 dominantes et 8 cases sombres, contre 5 et 7 | **La fiche témoin est la table de la mineur**, couche sombre affichée |
| 3 | Le départage des doigtés de la spec donnait x3x010 pour C. L'énumération libre des candidats, même corrigée, produit des formes qu'aucun guitariste ne joue (D x50232, A 502220, F 10321x, Bm x20402) | **Les candidats sont les formes du recueil** (§ « La guitare »). Le départage gagne « le plus de cordes jouées » avant « le moins de doigts » |
| 4 | La palette nomme ses notes en solfège, mais la spec ne disait rien d'une case réécrite à l'enharmonique simple (`A` pour B𝄫), ni d'une note double-altérée (le fa𝄪 de `D♯7`). DejaVu Sans n'a pas de glyphe 𝄪 | **Comme le nom** : une case réécrite l'est entière, toute autre double altération passe à l'enharmonique simple |

Ce plan prend aussi seul quatre décisions, que la spec ne tranchait pas. Toutes ont été calculées sur
ses exemples :

- **Le barré** ne s'emploie que s'il faudrait plus de quatre doigts sans lui. Sans cette condition, la
  règle de la spec mettait `x02220` en barré, contre les numéros 1, 2, 3 qu'elle lui donne.
- **La zone d'un doigté**, pour le calcul en contexte : 0 s'il fait sonner une corde à vide, sinon sa
  frette appuyée la plus basse. Avec la seule frette la plus basse, la suite G–C–D passait en barrés.
- **L'ancre d'une suite** : le calcul en contexte accepte un doigté fixé (`Maillon::Fixe`). Sans lui,
  le critère 1 de REN-04 (« l'autre grille en haut du manche ») est impossible à construire, puisque
  tout accord a une forme basse. Il servira à l'épinglage, plus tard.
- **La portée de la palette** n'a pas de clé : chaque note porte son nom dessous. DejaVu Sans n'a pas
  de clé de sol, et la portée gravée arrive avec Verovio, en INC-2.

## Ce qui doit survivre à la clôture

Le plan est supprimé à la clôture (PR 8). Ce qu'il décide s'écrit avant, dans la conception, par la PR
qui le livre :

| Quoi | Où | Par quelle PR |
|---|---|---|
| La page en paysage | § 50 · 3.7, et l'arbitrage A22 au § 90 | PR 2, si H09 passe |
| La règle de la palette et l'écriture de ses notes | MIC-13 (Règles, Cas limites) | PR 4 |
| Le clavier : candidats, déplacement, égalités, la convention d'octave | § 60 · 6.3 | PR 4 |
| La guitare : les formes, le barré, les numéros de doigts, le départage, la zone | § 60 · 6.1 | PR 4 |
| Le sous-ensemble de SVG, les glyphes en tracés | § 41 · 5.7 | PR 5 |
| Le cahier : son ordre, son sommaire, son pied | REN-01 | PR 5 |
| Le budget de dépendances | § 41 · 6 | Chaque PR qui ajoute une dépendance |
| Les portes de CI | `CONVENTIONS.md`, à chaque porte ajoutée ; puis le § 70 · 8 | PR 2, PR 3 ; PR 8 |
| Les contrôles à l'œil, datés | § 70 · 7, un tableau « Relevés » | PR 7 |
| L'état, les commandes de Rust | `CLAUDE.md` | PR 8 |

## Contraintes globales

Chaque tâche les inclut implicitement.

- **La chaîne** : `rust-toolchain.toml` épingle `channel = "1.99.1"`, les composants `rustfmt` et
  `clippy`, la cible `wasm32-unknown-unknown`. L'espace de travail est en édition 2024.
- **Le noyau** n'a aucune entrée-sortie : aucune dépendance d'interface, de stockage ni de réseau
  (ADR-019). La police y entre par `include_bytes!`.
- **Les dépendances d'exécution** : chacune entre par la PR qui en a besoin. Cette PR la nomme dans sa
  description et amende le § 41 · 6. On prend la dernière version publiée ; `Cargo.lock` est versionné.
  Les dépendances de test ne comptent pas.
- **Les actions** sont épinglées par SHA, avec leur version en commentaire, comme dans `ci.yml`.
- **Le dépôt est public** : ni chemin de machine, ni prénom, ni nom d'une personne réelle. Les
  exemples sont inventés. Les gestes de la maîtrise d'ouvrage sont marqués 👤.
- **Les mots** sont ceux du glossaire : case, cible, famille, rangée, couche sombre, ligne d'effet,
  flèche de résolution, palette, appui, tension, frotte, doigté, voicing, cahier des tables.
- **Le mode encre** : les pages n'ont que du noir et du blanc, et aucune information n'est portée par
  la seule couleur, ni dans les pages ni dans l'interface.
- **Les textes exacts** : les lignes d'effet du § 60, mot pour mot (`tire vers <cible>`,
  `assombrit`, `éclaire`, `glisse vers <cible>`, `tension sombre avant la dominante`, `suspend`,
  `repos`, `éloignement`, `tension`) ; « pas de doigté jouable » ; « couche sombre : masquée » ou
  « affichée ».
- **La page** : 209,5 × 157 mm, en paysage ; une unité du SVG vaut 1 mm.
- **Les minimums physiques** (§ 50 · 6.3) : un texte d'au moins 11 pt, un diagramme de manche d'au
  moins 18 mm de large, une touche blanche d'au moins 3 mm, un trait d'au moins 0,25 pt, une pastille
  de fonction d'au moins 3 mm, une zone de lien d'au moins 12 mm de haut. Aucune trame.
- **Le déterminisme (I25)** : la date du rendu est une entrée. Deux engendrements à la même date
  donnent des pages et un PDF identiques à l'octet, métadonnées comprises.
- **La convention d'octave** : do 3 = MIDI 60 (la 3 = 440 Hz).
- **Les commits** : Conventional Commits en français, en-tête de 100 caractères au plus ;
  `git add` par chemins explicites.
- **Les PR** : une branche par PR, dans un worktree `pupitre.worktrees/<sujet>`. La description dit
  « Refs #1 » ; seule la PR de clôture dit « Closes #1 ». La session fusionne par
  `gh pr merge <n> --squash`, une fois tous les contrôles verts.
- **Les vérifications du projet** restent vertes à chaque PR : `bash scripts/verif-confidentialite.sh`,
  `node scripts/verif-liens.mjs`, `node scripts/verif-diagrammes.mjs`.

## Les règles

Recopiées de la spec, avec les décisions ci-dessus. Elles font foi pour l'exécution.

### Les hauteurs et les tonalités

- Une note est un **intervalle orthographié** depuis la tonique : un nombre de degrés et un nombre de
  demi-tons (ADR-008). Son nom n'est qu'un rendu, en anglais pour les accords (`E♭`), en solfège pour
  la palette (mi♭).
- Les 24 tonalités d'A17 sont, dans l'**ordre du cahier** (le cycle des quintes, chaque majeure suivie
  de sa relative) : do majeur, la mineur, sol majeur, mi mineur, ré majeur, si mineur, la majeur,
  fa♯ mineur, mi majeur, do♯ mineur, si majeur, sol♯ mineur, fa♯ majeur, **mi♭ mineur** (la relative
  de fa♯, écrite selon A17), ré♭ majeur, si♭ mineur, la♭ majeur, fa mineur, mi♭ majeur, do mineur,
  si♭ majeur, sol mineur, fa majeur, ré mineur.
- La gamme d'une tonalité mineure est le mineur naturel.

### La table

Elle se calcule en degrés, puis se nomme dans la tonalité, rangée par rangée :

| Rangée | Majeur (en do) | Mineur (en la) |
|---|---|---|
| **Dessus**, chacune au-dessus de sa cible | V7/ii `A7`, V7/iii `B7`, V7/IV `C7`, V7/V `D7`, V7/vi `E7`, aux colonnes 1 à 5 | V7/i `E7`, V7/III `G7`, V7/iv `A7`, V7/v `B7`, V7/VI `C7`, V7/VII `D7`, aux colonnes 0, 2, 3, 4, 5, 6 |
| **Centre** | I `C`, ii `Dm`, iii `Em`, IV `F`, V `G`, vi `Am`, vii° `B°` | i `Am`, ii° `B°`, III `C`, iv `Dm`, v `Em`, VI `F`, VII `G` |
| **Couche sombre**, masquée par défaut | vii°7/I `B°7`→C · vii°7/ii `C♯°7`→Dm · vii°7/iii `D♯°7`→Em · vii°7/IV `E°7`→F · vii°7/V `F♯°7`→G · ♭II6 `D♭/F`→G · vii°7/vi `G♯°7`→Am | vii°7/i `G♯°7`→Am · III+ `C+`, sans flèche · vii°7/III `B°7`→C · vii°7/iv `C♯°7`→Dm · vii°7/v `D♯°7`→Em · ♭II6 `B♭/D`→`E` des emprunts · vii°7/VI `E°7`→F · vii°7/VII `F♯°7`→G |
| **Dessous**, chacune sous son degré | i `Cm`, ii° `D°`, ♭III `E♭`, iv `Fm`, v `Gm`, ♭VI `A♭`, ♭VII `B♭` | I `A`, ii `Bm`, iii `C♯m`, IV `D`, V `E`, vi `F♯m`, vii° `G♯°` |

- Les flèches vont toujours d'une rangée à sa voisine : du dessus vers le centre ; de la couche sombre
  vers le centre, sauf la napolitaine du mineur, qui mène aux emprunts.
- Les fonctions du centre : en majeur, I, iii, vi → `repos` ; ii, IV → `éloignement` ; V, vii° →
  `tension`. En mineur, i, III → `repos` ; ii°, iv, VI → `éloignement` ; v, VII → `tension`.
- Un nom qui porterait une double altération s'écrit à l'enharmonique simple, sous le même chiffre
  (Q21) : `A` pour le ♭VI de ré♭ majeur, `G°7` pour `F𝄪°7`.

### La palette (MIC-13)

Une seule fonction, `role(note, accord, tonalite)`, que la palette appelle et que MIC-06 appellera en
INC-3 (ADR-001). Elle calcule en degrés, sur l'orthographe juste :

- **appui** : une note dont la classe de hauteur est dans l'accord, même hors de la tonalité ;
- une note hors de l'accord et hors de la tonalité n'a pas de rôle (`None`) ;
- une note de la tonalité hors de l'accord se nomme par son **intervalle orthographié** au-dessus de
  la fondamentale, ramené à l'octave. Une seconde est une 9 (♭9, 9, ♯9), une quarte une 11 (11 juste,
  ♯11), une sixte une 13 (♭13, 13) ;
- **frotte** : la ♭9 ; la 11 juste sur un accord majeur ; la ♭13 sur un accord mineur ; toute
  septième ; et toute note qui n'est ni l'une des 9, 11 ou 13 nommées ci-dessus ni une septième
  (unisson, tierce ou quinte altérés contre ceux de l'accord, quarte diminuée, sixte augmentée) ;
- **tension** : toute autre 9, 11 ou 13 ;
- un accord est **majeur** si sa tierce est majeure (`maj`, `aug`, `7`), **mineur** si elle est
  mineure (`m`, `dim`, `dim7`).

**L'écriture des notes** : une case dont le nom est réécrit à l'enharmonique simple a toutes ses notes
réécrites avec lui, du même écart de lettre (`A` pour B𝄫 : la, do♯, mi). Toute autre note qui
porterait une double altération s'écrit à l'enharmonique simple (le fa𝄪 de `D♯7` : sol).

### Le clavier (REN-08)

- Les **candidats** : un par renversement, en position serrée (chaque note au plus près au-dessus de la
  précédente), la basse entre do 3 et si 3 (MIDI 60 à 71). Un accord à basse imposée (la napolitaine)
  ne garde que les candidats dont la basse est la sienne.
- **Hors contexte** : le renversement dont la basse est celle de l'accord, donc la position
  fondamentale, sauf pour la napolitaine.
- **Dans une suite** : le candidat de plus petit **déplacement** depuis le voicing précédent. Le
  déplacement additionne, sur les notes du candidat, la distance en demi-tons à la note la plus proche
  du voicing précédent, et, sur les notes du voicing précédent, la distance à la note la plus proche du
  candidat. À égalité, la basse la plus grave l'emporte. Sans voicing précédent : hors contexte.
- Exemple chiffré (critère 2 de REN-08) : après `G` [67, 71, 74], `C` prend [67, 72, 76],
  déplacement 6, contre 7 pour [64, 67, 72] et 21 pour [60, 64, 67].

### La guitare (REN-04, § 60 · 6.1)

L'accordage est `EADGBE`, MIDI [40, 45, 50, 55, 59, 64], sans réglage ni capodastre.

**Les candidats sont les formes du recueil.** Une forme se décrit par ses frettes en position ouverte,
de la corde grave à l'aiguë (`x` : étouffée). Une forme **mobile** se déplace d'un même nombre de
frettes sur toutes ses cordes, tant qu'aucune frette ne dépasse 15. Une forme **fixe** ne sert qu'à sa
place. Une forme placée sert tout accord dont elle tient les contraintes, quel que soit son nom.

| Accord | Formes mobiles | Formes fixes |
|---|---|---|
| Majeur | E `022100`, A `x02220`, D `xx0232` | C `x32010`, G `320003` |
| Mineur | Em `022000`, Am `x02210`, Dm `xx0231` | — |
| Septième | E7 `020100`, A7 `x02020`, D7 `xx0212` | C7 `x32310`, G7 `320001`, B7 `x21202` |
| Diminué | B° `x2343x`, D° `xx0131` | — |
| Diminué 7 | D°7 `xx0101`, A°7 `x01212`, A♯°7 `x1202x`, E°7 `012020` | — |
| Augmenté | D+ `xx0332` | C+ `x32110`, E+ `032110` |
| Premier renversement (la napolitaine) | C/E `xx2010`, G/B `x20003` | B♭/D `xx0331`, F/A `x03211` |

**Les contraintes**, toutes obligatoires. Une forme placée n'est candidate que si elle les tient toutes :
- elle ne fait sonner que des notes de l'accord ;
- la basse est juste : la corde la plus grave jouée sonne la basse de l'accord ;
- elle a toutes les notes, sauf éventuellement la quinte ;
- l'étendue entre la frette appuyée la plus basse et la plus haute est d'au plus trois frettes ;
- elle demande au plus quatre doigts, un barré comptant pour un.

**Le barré** ne s'emploie que s'il faudrait plus de quatre doigts sans lui. L'index couvre alors la
frette appuyée la plus basse, de la corde la plus grave qui y est appuyée à la plus aiguë. Il faut que
ces deux cordes soient distinctes et que toute corde entre elles soit jouée, à cette frette ou plus
haut. Une corde étouffée ou à vide entre les bornes interdit le barré.

**Les numéros de doigts** : l'index (1) prend le barré, ou sinon la note appuyée la plus basse en
frette (la plus grave, à frette égale). Les autres notes appuyées prennent 2, 3 et 4, dans l'ordre des
frettes croissantes, puis des cordes de la plus grave à la plus aiguë. Lus de la corde grave à
l'aiguë : `x32010` donne 3, 2, 1 ; `x02220` donne 1, 2, 3 ; `133211` donne un barré, puis 3, 4, 2 ;
`x24432` donne un barré, puis 3, 4, 2.

**Hors contexte**, dans la table, on départage ainsi les candidats :
1. la frette appuyée la plus basse, la plus basse (0 si aucune corde n'est appuyée) ;
2. le plus de cordes à vide ;
3. le plus de cordes jouées ;
4. le moins de doigts ;
5. le premier dans l'ordre des frettes lues de la grave à l'aiguë, une corde étouffée avant 0.

**En contexte**, c'est le plus court chemin sur la suite. La **zone** d'un doigté vaut 0 s'il fait
sonner une corde à vide, et sinon sa frette appuyée la plus basse. Le déplacement entre deux doigtés est
l'écart de leurs zones. On minimise le déplacement total. À égalité, on prend le chemin qui fait sonner
le plus de cordes à vide, puis, accord par accord dans l'ordre de la suite, l'ordre hors contexte. Un
maillon fixé n'a que son doigté. Un accord sans candidat reste sans doigté, et le chemin le saute.

**Sans candidat** : « pas de doigté jouable », et les notes.

### Les rendus

- **Les pages** : un SVG par page, 209,5 × 157 mm, en paysage. Elles s'impriment à 100 % sur A4
  paysage.
- **Le sous-ensemble de SVG** : les éléments `svg`, `g`, `path`, `rect`, `circle`, `line`, et les
  attributs de géométrie, de trait (`stroke`, `stroke-width`, `stroke-linecap`, `stroke-linejoin`),
  `fill`, `fill-rule`, `transform` et `id`. Les couleurs sont `#000`, `#fff` ou `none`. Les pages n'ont
  ni texte, ni dégradé, ni filtre, ni opacité, ni feuille de style, ni script, ni animation.
- **Les identifiants** sont uniques dans une page : un préfixe et un numéro d'ordre (`t-12`,
  `manche-3`, `touche-blanche-3-5`, `pastille-2`). Les tests mesurent par eux.
- **Les glyphes en tracés** : chaque caractère devient le tracé de son glyphe, lu par `ttf-parser`, en
  unités de police. Un texte est un groupe `<g id="t-n" transform="translate(x y) scale(k -k)">`, où
  k = corps en mm / 2048 (les unités par cadratin de DejaVu Sans). Un glyphe avance de sa chasse. Ni
  crénage, ni ligature, ni césure : une ligne se coupe entre deux mots, et un mot plus large que sa
  place est une erreur (`MotTropLong`), jamais un mot coupé.
- **La police** : DejaVu Sans 2.37, sa licence dans le dépôt. Elle couvre le français, ♭, ♯ et °.
- **La page de la table** : une colonne par degré, les rangées dans l'ordre (dessus, centre, couche
  sombre affichée, dessous), chacune avec son nom. Les quatre cadres sont au même trait fin : **droit**
  (diatoniques), **en pointe** vers la cible (dominantes secondaires), **aux coins arrondis**
  (emprunts), **aux coins coupés** (harmonie sombre). L'arrondi et la coupe prennent au moins le quart
  du côté de la case. Chaque flèche est un trait plein, et sa pointe pleine se pose sur la cible. La
  pastille de fonction (○ ◐ ●, 3 mm) est au centre. On trouve encore la légende des quatre cadres et le
  pied. La ligne d'effet se coupe sur plusieurs lignes dans sa case. Les huit cases sombres du mineur
  se répartissent sur la largeur, dans l'ordre des degrés de leurs cibles, et leurs flèches peuvent
  être obliques. Masquée, la couche sombre est dite à l'endroit où elle s'insère : « couche sombre
  masquée ».
- **L'en-tête de la table** : « la mineur · table de la tonalité · tonalité choisie · instrument :
  guitare et clavier · couche sombre : affichée ». L'instrument s'écrit « guitare », « clavier » ou
  « guitare et clavier ».
- **La case cible**, à l'établi : en inverse, fond noir et texte blanc, redoublée d'un double cadre
  fin. Le noyau la dessine.
- **Le verso** : pour les sept accords du centre, dans leur ordre, une colonne par accord, alignée sur
  les colonnes de la table. Chaque colonne porte le nom et le chiffre de l'accord. Avec la guitare
  viennent le diagramme de manche et la palette sur le manche, dans la zone du doigté à ± 2 frettes ;
  avec le clavier, le diagramme de clavier et la palette sur le clavier. La palette sur une portée
  d'une octave figure toujours. L'instrument est un paramètre : guitare, clavier, ou les deux, **les
  deux par défaut**.
- **Le diagramme de manche** : six cordes, quatre ou cinq frettes, le numéro de la frette de départ
  s'il dépasse 1. Un point plein marque un doigt, un cercle une corde à vide, une croix une corde
  étouffée, un trait épais le barré, et les numéros de doigts sont dessous. Il mesure au moins 18 mm de
  large.
- **Le diagramme de clavier** : de la touche blanche qui précède la note la plus grave à celle qui suit
  la plus aiguë, deux octaves au plus. Une note jouée est un point plein, blanc sur une touche noire. La
  basse est une lettre sous le clavier.
- **Les formes de la palette** (celles de MIC-06) : appui plein, tension creuse, frotte en croix. Les
  notes se nomment en solfège. La palette sur le clavier couvre une octave à partir de la fondamentale,
  avec une touche blanche de marge de chaque côté.
- **Le cahier** : un sommaire, puis les 24 tonalités dans l'ordre du cahier, chacune sur une page
  suivie de son verso. Il compte 49 pages. Le sommaire porte un lien vers chaque table, dont la zone
  mesure au moins 12 mm de haut.
- **Le pied** : « page 2 / 49 · rendu le 1er nov. 2026 · empreinte 7c1e4a ». Sur une page de table,
  il ajoute « · verso : diagrammes et palettes des accords du centre ». L'empreinte est faite des six
  premiers caractères hexadécimaux du SHA-256 du SVG de la page, pied exclu. Les mois s'abrègent janv.,
  févr., mars, avr., mai, juin, juil., août, sept., oct., nov., déc. ; le premier jour s'écrit « 1er ».

### L'application de l'établi

- **L'en-tête** : la tonalité (les 24, d'un geste), la couche sombre (affichée ou masquée),
  l'instrument (guitare, clavier, les deux), et « Exporter le cahier ».
- **La table** est la page SVG du noyau. Chaque case est recouverte d'un bouton transparent, qu'on
  atteint au clavier, et dont le libellé dit le nom et le chiffre de la case. Toucher une case met sa
  cible en inverse ; une case sans cible ne met rien en inverse. Le bouton « Palette » s'affiche alors
  à côté de la case. « Prendre » n'existe pas : il n'y a pas de morceau.
- **La palette** est un panneau qui montre la colonne de l'accord, comme au verso, selon l'instrument.
- **Les réglages** ne sont pas gardés d'une ouverture à l'autre.

## Points de vigilance

Ce que la spec implique sans qu'aucun critère ne l'éprouve. Chaque ligne a son test dans la tâche qui
porte le code.

1. **Une case qui déborde.** Mi♭ mineur a des noms longs (`F♭/A♭`, `B♭°7`) et une ligne
   `glisse vers C♭` ; en la mineur, la napolitaine tombe dans une case d'un huitième de largeur. Toute
   page des 24 tonalités, couche sombre affichée ou masquée, doit tenir, ou échouer en le disant
   (`Debordement`), jamais couper un mot ni passer sous 11 pt (tâche 13).
2. **La relative de fa♯ majeur** s'écrit mi♭ mineur (A17), pas ré♯ mineur, dans l'ordre du cahier et au
   sommaire (tâche 6).
3. **Une case sans cible** (diatonique, emprunt, `III+`), touchée à l'établi, ne met rien en inverse,
   mais « Palette » s'affiche (tâches 8 et 16).
4. **Deux flèches sur la même cible** : en do majeur, `F♯°7` et `D♭/F` mènent à `G`. Leurs pointes ne
   se confondent pas : leurs points d'arrivée sont distants d'au moins 3 mm (tâche 13).
5. **L'export abandonné** : si l'on ferme la boîte d'enregistrement, rien n'est écrit et aucune erreur ne
   s'affiche. Si l'écriture échoue, un message en français le dit (tâche 16).

## Les fichiers

```
Cargo.toml                       espace de travail : membres, édition 2024, profils
Cargo.lock                       versionné
rust-toolchain.toml              la chaîne épinglée
.cargo/mutants.toml              les modules examinés, les mutants équivalents nommés
crates/noyau/                    la crate pure
  polices/DejaVuSans.ttf         la police embarquée
  polices/LICENCE-DejaVu.txt     sa licence et sa provenance
  src/lib.rs
  src/hauteur.rs                 Intervalle, Lettre, NomDeNote
  src/tonalite.rs                Mode, Tonalite, l'ordre du cahier
  src/accord.rs                  Qualite, Accord, les noms
  src/table.rs                   Famille, Rangee, Position, Case, Table, CoucheSombre
  src/palette.rs                 Role, role(), palette()
  src/clavier.rs                 Voicing, candidats, déplacement
  src/guitare.rs                 Forme, Doigte, contraintes, départage, chemin
  src/rendu/mod.rs               Instrument, Debordement
  src/rendu/svg.rs               Element, Page, l'écriture du SVG
  src/rendu/sous_ensemble.rs     verifier()
  src/rendu/texte.rs             Corps, chasse, composer, couper
  src/rendu/pages.rs             dimensions, Rect, Date, Lien, PageDuCahier
  src/rendu/case.rs              cadres, états, pastilles, flèches
  src/rendu/table.rs             la page de la table, sa disposition
  src/rendu/manche.rs            le diagramme de manche, la palette sur le manche
  src/rendu/clavier.rs           le diagramme de clavier, la palette sur le clavier
  src/rendu/portee.rs            la palette sur la portée
  src/rendu/verso.rs             la colonne d'un accord, le verso
  src/rendu/cahier.rs            le sommaire, le pied, les 49 pages
  tests/                         un fichier par module, plus commun/mod.rs (mesures, pixels)
crates/cahier/
  src/lib.rs                     assembler(), engendrer()
  src/main.rs                    la commande : --date AAAA-MM-JJ --instrument … --sortie …
  src/temoin.rs                  la fiche témoin d'H09 (retirée en PR 5)
  examples/temoin.rs             écrit fiche-temoin.pdf (retiré en PR 5)
  tests/                         h09.rs (retiré en PR 5), pdf.rs
app/ui/                          l'interface Leptos (crate `interface`)
  Cargo.toml, Trunk.toml, index.html, style.css
  src/main.rs, src/app.rs, src/entete.rs, src/vue_table.rs, src/panneau_palette.rs, src/export.rs
  tests/encre_statique.rs        I17 et I18, statiques
app/src-tauri/                   l'application de bureau (crate `etabli`)
  Cargo.toml, build.rs, tauri.conf.json, capabilities/default.json, icons/
  src/main.rs, src/lib.rs        la commande exporter_cahier
.github/workflows/ci.yml         + verifier, windows, mutants
.github/workflows/mutants-complet.yml
```

---

## PR 2 · Les essais : H09 et H10

Branche `inc-0-essais`. Elle apporte l'espace de travail, la chaîne, les jobs `verifier` et `windows`,
et les deux essais. Si l'un échoue, la PR n'est pas fusionnée et la suite attend une conception revue.

### Tâche 1 : l'espace de travail et l'écriture des pages

**Fichiers :**
- Créer : `Cargo.toml`, `rust-toolchain.toml`, `crates/noyau/Cargo.toml`, `crates/noyau/src/lib.rs`,
  `crates/noyau/src/rendu/{mod,svg,sous_ensemble,texte,pages}.rs`, `crates/noyau/polices/DejaVuSans.ttf`,
  `crates/noyau/polices/LICENCE-DejaVu.txt`, `crates/noyau/tests/pages.rs`
- Modifier : `.gitignore` (+ `target/`), `.github/workflows/ci.yml` (+ job `verifier`),
  `CONVENTIONS.md` (« Portes de CI » : la porte `verifier`, et ses commandes dans « Les lancer avant de
  pousser »), `docs/41-pile-technique.md` § 6 (`ttf-parser` au noyau)

**Interfaces :**
- Produit :

```rust
// rendu::pages
pub const LARGEUR: f32 = 209.5; pub const HAUTEUR: f32 = 157.0;      // mm, paysage
pub struct Rect { pub x: f32, pub y: f32, pub l: f32, pub h: f32 }  // mm
pub struct Date { pub annee: u16, pub mois: u8, pub jour: u8 }
impl Date { pub fn en_francais(&self) -> String; pub fn depuis_iso(s: &str) -> Option<Date> }
pub struct Lien { pub zone: Rect, pub vers: usize }                 // index de page, depuis 0
pub struct PageDuCahier { pub svg: String, pub liens: Vec<Lien> }
// rendu::svg
pub enum Couleur { Noir, Blanc }
pub enum Element {
    Groupe { id: Option<&'static str>, translation: (f32, f32), echelle: Option<(f32, f32)>, enfants: Vec<Element> },
    Chemin { d: String, plein: Option<Couleur>, trait_: Option<(Couleur, f32)> },
    Rect { r: Rect, id: Option<&'static str>, plein: Option<Couleur>, trait_: Option<(Couleur, f32)> },
    Cercle { cx: f32, cy: f32, r: f32, id: Option<&'static str>, plein: Option<Couleur>, trait_: Option<(Couleur, f32)> },
    Ligne { de: (f32, f32), a: (f32, f32), trait_: (Couleur, f32) },
}
pub const TRAIT_FIN: f32 = 0.25 * 25.4 / 72.0;                      // 0,25 pt en mm
pub struct Page { pub largeur: f32, pub hauteur: f32, pub elements: Vec<Element> }
impl Page { pub fn svg(&self) -> String }
// rendu::sous_ensemble
pub fn verifier(svg: &str) -> Result<(), Vec<String>>;
// rendu::texte
pub struct Corps(f32);
impl Corps { pub const MIN: Corps; pub fn pt(pt: f32) -> Option<Corps>; pub fn mm(&self) -> f32 }
pub struct MotTropLong { pub mot: String, pub chasse: f32, pub largeur: f32 }
pub fn chasse(texte: &str, corps: &Corps) -> f32;
pub fn composer(texte: &str, corps: &Corps, x: f32, ligne_de_base: f32) -> Element;
pub fn couper(texte: &str, corps: &Corps, largeur: f32) -> Result<Vec<String>, MotTropLong>;
pub fn glyphe_present(c: char) -> bool;
```

- [ ] **Étape 1 : écrire les tests qui échouent** (`crates/noyau/tests/pages.rs`)

```rust
#[test] fn une_page_vide_est_dans_le_sous_ensemble() {
    let svg = Page { largeur: LARGEUR, hauteur: HAUTEUR, elements: vec![] }.svg();
    assert!(svg.contains(r#"width="209.5mm""#) && svg.contains(r#"viewBox="0 0 209.5 157""#));
    assert_eq!(verifier(&svg), Ok(()));
}
#[test] fn le_sous_ensemble_refuse_ce_qui_sort() {
    for (svg, motif) in [(SVG_AVEC_TEXT, "text"), (SVG_ROUGE, "#f00"), (SVG_OPACITE, "opacity"),
                         (SVG_STYLE, "style"), (SVG_ANIMATE, "animate"), (SVG_DEGRADE, "linearGradient")] {
        let erreurs = verifier(svg).unwrap_err();
        assert!(erreurs.iter().any(|e| e.contains(motif)), "{motif}");
    }
}
#[test] fn les_caracteres_exiges_ont_un_glyphe() {
    for c in "àâäçéèêëîïôöùûüÿœæÀÂÄÇÉÈÊËÎÏÔÖÙÛÜŸŒÆ♭♯°IViv0123456789/+·×".chars() {
        assert!(glyphe_present(c), "{c}");
    }
}
#[test] fn un_texte_compose_est_un_groupe_de_traces_a_son_corps() {
    let page = Page { largeur: LARGEUR, hauteur: HAUTEUR,
                      elements: vec![composer("glisse vers G♯°7", &Corps::MIN, 10.0, 20.0)] };
    let svg = page.svg();
    assert_eq!(verifier(&svg), Ok(()));
    assert!(svg.contains(r#"id="t-1""#) && svg.contains("scale(0.001895 -0.001895)"));
}
#[test] fn la_chasse_d_eloignement_a_11_pt() {
    assert!((chasse("éloignement", &Corps::MIN) - 24.38).abs() < 0.05);
}
#[test] fn couper_coupe_entre_deux_mots_et_refuse_un_mot_trop_long() {
    let lignes = couper("tension sombre avant la dominante", &Corps::MIN, 30.0).unwrap();
    assert_eq!(lignes.join(" "), "tension sombre avant la dominante");
    assert!(lignes.iter().all(|l| chasse(l, &Corps::MIN) <= 30.0));
    assert_eq!(couper("éloignement", &Corps::MIN, 20.0).unwrap_err().mot, "éloignement");
}
#[test] fn le_corps_refuse_moins_de_11_pt() {
    assert!(Corps::pt(10.9).is_none() && Corps::pt(11.0).is_some());
}
#[test] fn la_date_en_francais() {
    assert_eq!(Date { annee: 2026, mois: 11, jour: 1 }.en_francais(), "1er nov. 2026");
    assert_eq!(Date { annee: 2026, mois: 10, jour: 11 }.en_francais(), "11 oct. 2026");
    assert_eq!(Date { annee: 2027, mois: 2, jour: 3 }.en_francais(), "3 févr. 2027");
    assert_eq!(Date::depuis_iso("2026-08-15").unwrap().en_francais(), "15 août 2026");
}
```

Les constantes `SVG_*` sont six petits SVG écrits à la main dans le fichier de test, chacun avec un seul
défaut. Le facteur d'échelle vaut 11 × 25,4 / 72 / 2048 = 0,001895, écrit à six décimales.

- [ ] **Étape 2 : lancer les tests et les voir échouer**

Lancer : `cargo test -p noyau --test pages`
Attendu : échec de compilation (`Page`, `verifier`… introuvables).

- [ ] **Étape 3 : écrire l'espace de travail et les modules**

`Cargo.toml` déclare les membres `crates/noyau` (et `crates/cahier`, `app/ui`, `app/src-tauri` à
mesure qu'ils naissent), `resolver = "3"` et `[workspace.package] edition = "2024"`. La police vient de
la publication officielle de DejaVu 2.37. `LICENCE-DejaVu.txt` recopie sa licence et dit d'où vient le
fichier, avec son SHA-256. Les nombres s'écrivent avec au plus trois décimales (six pour l'échelle des
glyphes), sans zéro final, pour un SVG identique à l'octet. Le SVG commence par un `rect` blanc de toute
la page. `Page::svg()` numérote les identifiants dans l'ordre d'écriture. `verifier` relève sans
dépendance les noms d'éléments, d'attributs et de couleurs, et nomme chaque écart.
`couper` remplit chaque ligne de mots tant que sa chasse tient dans la largeur.

- [ ] **Étape 4 : lancer les tests et les voir passer**

Lancer : `cargo test -p noyau --test pages`
Attendu : 8 tests réussis.

- [ ] **Étape 5 : brancher le job `verifier`**

Dans `ci.yml`, un job `verifier` sur `ubuntu-latest` : l'extraction, `rustup show` (qui installe la
chaîne de `rust-toolchain.toml`), le cache `Swatinem/rust-cache`, puis `cargo fmt --all --check`,
`cargo clippy -p noyau --all-targets -- -D warnings` et `cargo test -p noyau`. Chaque action est
épinglée par SHA. Le commentaire de fin de `ci.yml` (« s'ajoutent ici, dans un job `verifier` ») est
retiré.

- [ ] **Étape 6 : vérifier en local, puis faire le commit**

Lancer : `cargo fmt --all --check && cargo clippy -p noyau --all-targets -- -D warnings && cargo test -p noyau`, puis les trois vérifications du projet.
Attendu : tout passe.

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml .gitignore crates/noyau .github/workflows/ci.yml CONVENTIONS.md docs/41-pile-technique.md
git commit -m "feat(noyau): l'espace de travail et l'écriture des pages en SVG"
```

### Tâche 2 : la fiche témoin et son PDF (H09, partie automatique)

**Fichiers :**
- Créer : `crates/cahier/Cargo.toml`, `crates/cahier/src/{lib,temoin}.rs`,
  `crates/cahier/examples/temoin.rs`, `crates/cahier/tests/h09.rs`
- Modifier : `Cargo.toml` (membre), `.github/workflows/ci.yml` (`verifier` : clippy et tests de
  `cahier`, la fiche témoin en artefact `fiche-temoin`), `docs/41-pile-technique.md` § 6 (`pdf-writer`
  au cahier)

**Interfaces :**
- Consomme : `noyau::rendu::{pages, svg, texte, sous_ensemble}` (tâche 1).
- Produit :

```rust
pub enum ErreurPdf { Svg { page: usize, message: String }, Conversion { page: usize, message: String } }
pub fn assembler(pages: &[PageDuCahier], date: &Date) -> Result<Vec<u8>, ErreurPdf>;
// temoin.rs
pub enum ErreurTemoin { MotTropLong(MotTropLong), Debordement { quoi: String, hauteur: f32 } }
pub fn pages_temoin(date: &Date) -> Result<Vec<PageDuCahier>, ErreurTemoin>;
```

La fiche témoin est **écrite à la main** : ses textes et ses positions sont des constantes de
`temoin.rs`. Elle n'emploie que la couche générique de `noyau::rendu`, sans la théorie. Ses trois
pages :
1. **le sommaire** : un titre ; une ligne « Lettres : à â ç é è ê ë î ï ô ù û ü ÿ œ æ · ♭ ♯ ° » ; une
   entrée « la mineur · table de la tonalité », dont le lien de 12 mm de haut mène à la page 2 ;
2. **la table de la mineur**, couche sombre affichée, en paysage, avec les noms, chiffres et lignes
   d'effet de « Les règles ». On y trouve les quatre cadres, les flèches (dont les obliques de la couche
   sombre et `B♭/D`→`E`), les pastilles ○ ◐ ● de 3 mm, la légende, et un pied à empreinte fictive
   `000000` ;
3. **le verso de la mineur** : sept colonnes. Chacune porte le nom, un diagramme de manche de 18 mm
   (Am `x02210`, B° `x2343x`, C `x32010`, Dm `xx0231`, Em `022000`, F `133211`, G `320003`), un
   clavier à touches blanches de 3 mm, la palette sur le manche, la palette sur le clavier et une
   portée d'une octave. Les rôles sont ceux de l'oracle (tâche 10).

La grille de départ : marges de 6 mm, gouttières de 1,5 mm, une colonne de 26,93 mm (24,93 mm utiles
après 1 mm de retrait de chaque côté). Une case sombre mineure fait 23,375 mm (21,375 mm utiles). Le
nom d'accord est à 14 pt ; chiffres, lignes d'effet, en-tête et pied à 11 pt, avec un interligne de
1,2 cadratin. La hauteur de chaque rangée suit sa case la plus haute. `pages_temoin` additionne
les hauteurs et rend `Debordement` si elles dépassent les 145 mm utiles. Elle rend `MotTropLong` si
un mot ne tient pas dans sa case.

- [ ] **Étape 1 : écrire les tests qui échouent** (`crates/cahier/tests/h09.rs`)

```rust
const DATE: Date = Date { annee: 2026, mois: 11, jour: 1 };
const L_PT: f32 = 209.5 * 72.0 / 25.4;  // 593,858
const H_PT: f32 = 157.0 * 72.0 / 25.4;  // 445,039

#[test] fn la_fiche_temoin_tient_a_11_pt_en_paysage() { assert!(pages_temoin(&DATE).is_ok()); }
#[test] fn les_pages_temoin_restent_dans_le_sous_ensemble() {
    for p in pages_temoin(&DATE).unwrap() { assert_eq!(verifier(&p.svg), Ok(())); }
}
#[test] fn la_fiche_temoin_respecte_les_minimums() {
    for p in pages_temoin(&DATE).unwrap() {
        let m = mesurer(&p.svg);            // aide du fichier : usvg, par les identifiants
        assert!(m.corps_min_pt >= 11.0 && m.trait_min_pt >= 0.25 - 1e-3);
        assert!(m.manches.iter().all(|l| *l >= 18.0));
        assert!(m.touches_blanches.iter().all(|l| *l >= 3.0));
        assert!(m.pastilles.iter().all(|d| *d >= 3.0));
    }
}
#[test] fn le_pdf_temoin_a_trois_pages_de_la_taille_des_svg() {
    let doc = lopdf::Document::load_mem(&assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap()).unwrap();
    assert_eq!(doc.get_pages().len(), 3);
    for (_, id) in doc.get_pages() { assert_media_box(&doc, id, L_PT, H_PT, 0.01); }
}
#[test] fn le_pdf_temoin_pose_les_pages_sans_mise_a_l_echelle() {
    // pour chaque page : la BBox du XObject, transformée par sa /Matrix et le `cm` du flux, égale la MediaBox à 0,01 pt
}
#[test] fn le_lien_du_sommaire_mene_a_la_table() {
    // page 1 : une seule annotation /Link ; sa /Dest désigne l'objet de la page 2 ; la hauteur de son /Rect ≥ 34,016 pt (12 mm)
}
#[test] fn deux_engendrements_du_temoin_sont_identiques() {
    let a = assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap();
    let b = assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap();
    assert_eq!(a, b);
}
```

`mesurer(svg) -> Mesures` lit le SVG par `resvg::usvg` et rend `corps_min_pt` (lu sur l'échelle des
groupes `t-*`), `trait_min_pt` (le plus petit `stroke-width`), et les largeurs des groupes `manche-*`,
des rectangles `touche-blanche-*` et les diamètres des cercles `pastille-*`.

- [ ] **Étape 2 : lancer les tests et les voir échouer**

Lancer : `cargo test -p cahier --test h09`
Attendu : échec de compilation.

- [ ] **Étape 3 : écrire `assembler` et `pages_temoin`**

`assembler` lit chaque SVG par `svg2pdf::usvg::Tree::from_str`, le convertit par `svg2pdf::to_chunk`,
puis assemble par `pdf-writer` : une page par SVG, de MediaBox `[0 0 L_PT H_PT]`, le XObject posé pour
que 1 mm du SVG fasse 1 mm de la page. Chaque `Lien` devient une annotation `/Link` à destination
`/Fit` de sa page ; sa zone, en mm depuis le haut, est convertie en points depuis le bas. `/Info`
porte `/Producer (pupitre)` et `/CreationDate` à la date du rendu, à minuit UTC. Il n'y a pas d'`/ID`.
L'exemple `temoin` écrit le PDF au chemin reçu en argument.

- [ ] **Étape 4 : lancer les tests et les voir passer**

Lancer : `cargo test -p cahier --test h09`
Attendu : 7 tests réussis. **Si `la_fiche_temoin_tient_a_11_pt_en_paysage` échoue sur un `Debordement`
ou un `MotTropLong` qu'aucun réglage de la grille dans les minimums ne lève, H09 a échoué** : on
s'arrête et on le consigne (tâche 4).

- [ ] **Étape 5 : l'artefact en CI, puis le commit**

Le job `verifier` ajoute `-p cahier` à clippy et aux tests, lance
`cargo run -p cahier --example temoin -- fiche-temoin.pdf`, et dépose `fiche-temoin.pdf` en artefact
`fiche-temoin` par `actions/upload-artifact`.

```bash
git add Cargo.toml Cargo.lock crates/cahier .github/workflows/ci.yml docs/41-pile-technique.md
git commit -m "feat(cahier): la fiche témoin d'H09 et son PDF"
```

### Tâche 3 : l'écran minimal et le job `windows` (H10, partie automatique)

**Fichiers :**
- Créer : `app/ui/{Cargo.toml,Trunk.toml,index.html,style.css}`, `app/ui/src/main.rs`,
  `app/src-tauri/{Cargo.toml,build.rs,tauri.conf.json}`, `app/src-tauri/capabilities/default.json`,
  `app/src-tauri/icons/` (engendrées par `cargo tauri icon` depuis `icons/source.svg`, dessiné à la
  main, noir sur blanc), `app/src-tauri/src/{main,lib}.rs`
- Modifier : `Cargo.toml` (membres), `.gitignore` (+ `app/ui/dist/`, `app/src-tauri/gen/`),
  `.github/workflows/ci.yml` (`verifier` : `interface` ; job `windows`), `CONVENTIONS.md` (porte
  `windows`), `docs/41-pile-technique.md` § 6 (les crates de liaison imposées par la pile :
  `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `serde`, `serde-wasm-bindgen`, sous la ligne de
  `leptos` et de `tauri`)

**Interfaces :**
- Consomme : `noyau::rendu::{svg, texte}` (tâche 1).
- Produit : les crates `interface` (`app/ui`, Leptos en mode client) et `etabli` (`app/src-tauri`) ;
  dans `interface`, `fn page_d_essai() -> noyau::rendu::svg::Page`, une page en paysage où « pupitre ·
  essai H10 » est composé en tracés. La PR 6 la retire.

- [ ] **Étape 1 : écrire le test qui échoue** (`app/ui/src/main.rs`, module de test)

```rust
#[test] fn la_page_d_essai_est_dans_le_sous_ensemble() {
    assert_eq!(noyau::rendu::sous_ensemble::verifier(&page_d_essai().svg()), Ok(()));
}
```

- [ ] **Étape 2 : lancer le test et le voir échouer**

Lancer : `cargo test -p interface`
Attendu : échec de compilation.

- [ ] **Étape 3 : écrire l'écran et l'application**

`main` monte un composant qui insère `page_d_essai().svg()` dans la page (`inner_html`).
`tauri.conf.json` porte : `productName` « pupitre », `identifier` « app.pupitre.etabli »,
`build.beforeBuildCommand` `{ "script": "trunk build --release", "cwd": "../ui" }`,
`build.frontendDist` « ../ui/dist », `build.devUrl` « http://localhost:1420 », `app.withGlobalTauri`
vrai, une fenêtre « pupitre » de 1440 × 900, `bundle.targets` `["nsis"]`. `style.css` ne déclare
aucune animation ni transition.

- [ ] **Étape 4 : lancer le test, puis la construction locale**

Lancer : `cargo test -p interface`, puis, dans `app/src-tauri`, `cargo tauri build`.
Attendu : 1 test réussi ; un installateur dans `target/release/bundle/nsis/`.

- [ ] **Étape 5 : les jobs en CI, puis le commit**

Le job `verifier` ajoute `cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings` et
`cargo test -p interface`. Le job `windows`, sur `windows-latest`, enchaîne : l'extraction,
`rustup show`, le cache, l'installation de `trunk` et de `tauri-cli` à version épinglée, puis
`cargo tauri build` dans `app/src-tauri`. Viennent ensuite `cargo clippy -p etabli --all-targets -- -D warnings`,
qui demande `app/ui/dist`, donc après la construction, et le dépôt de l'installateur en artefact
`etabli-installateur`. Le `timeout-minutes` du job vaut deux fois la durée du premier passage mesuré,
et la PR l'écrit (LE-29 de `modele-projet`).

```bash
git add Cargo.toml Cargo.lock .gitignore app .github/workflows/ci.yml CONVENTIONS.md docs/41-pile-technique.md
git commit -m "feat(etabli): l'écran minimal de l'établi et sa construction Windows en CI"
```

### Tâche 4 : les gestes 👤 d'H09 et d'H10, l'arbitrage A22, la fusion

**Fichiers :**
- Modifier : `docs/50-interfaces-et-appareils.md` § 3.7, `docs/90-questions-ouvertes.md` (arbitrage
  A22), `README.md` et `docs/80-glossaire.md` (la plage A01–A21 devient A01–A22), `docs/registre.md`
  (si un essai échoue)

- [ ] **Étape 1 : ouvrir la PR**, titre `feat: les essais d'INC-0, H09 et H10`, corps par
  `--body-file`. Attendre que tous les contrôles soient verts.
- [ ] **Étape 2 : 👤 H09.** Récupérer l'artefact (`gh run download <run> -n fiche-temoin`), l'ouvrir
  sur la Note Air en paysage, et l'imprimer à 100 % sur A4 paysage. Regarder : mêmes pages, mêmes
  glyphes, rien qui déborde ; le trait fin visible ; le texte lisible à bout de bras ; les quatre
  cadres distincts sans lire le nom des rangées ; le lien du sommaire mène à la table, au doigt. Mesurer
  à la règle, sur papier, le diagramme de manche (au moins 18 mm) et une touche blanche (au moins 3 mm).
- [ ] **Étape 3 : 👤 H10.** Récupérer `etabli-installateur`, l'installer sur l'établi. Windows affiche
  « Windows a protégé votre ordinateur » : « Informations complémentaires », puis « Exécuter quand
  même ». L'application s'ouvre et montre la page d'essai.
- [ ] **Étape 4 : si un essai échoue**, la PR reste ouverte. `/consigner` écrit un constat EM avec ce
  qui a échoué, et la suite revient à la conception (§ 40 · 11).
- [ ] **Étape 5 : si les deux passent**, ajouter l'arbitrage A22 au § 90 :
  « **La page de la table en paysage** : 209,5 × 157 mm, l'écran du lutrin tourné, pour la table, son
  verso et tout le cahier | En portrait, à 11 pt, « éloignement » (24,4 mm en DejaVu Sans) ne tient pas
  dans un septième de 157 mm ; en paysage, une colonne fait 26,9 mm. H09 l'a vérifié le <date> | Revenir
  au portrait demande des lignes d'effet plus courtes ou une police plus étroite, et une nouvelle fiche
  témoin ». Le § 3.7 dit que la page est en paysage ([A22]) et que la maquette, dessinée en portrait
  avant l'essai, compose ses lignes d'effet sous 11 pt. Faire passer la plage A01–A21 à A01–A22 partout
  (`git grep -n "A21"`). Écrire les dates des deux essais dans le corps de la PR.
- [ ] **Étape 6 : vérifier, faire le commit, fusionner**

Lancer : les trois vérifications du projet. Attendu : vertes.

```bash
git add docs/50-interfaces-et-appareils.md docs/90-questions-ouvertes.md README.md docs/80-glossaire.md
git commit -m "docs: la page de la table en paysage, d'après H09 (A22)"
gh pr merge <n> --squash
```

- [ ] **Étape 7 : 👤** ajouter `verifier` et `windows` aux contrôles requis de la règle
  `branche-principale`, par `gh api -X PUT repos/{owner}/{repo}/rulesets/<id>`. La session
  prépare le JSON.

---

## PR 3 · La théorie

Branche `inc-0-theorie`. Les hauteurs, les tonalités, les accords, la table ; MIC-12, I26 ; le job
`mutants`.

### Tâche 5 : les hauteurs

**Fichiers :** créer `crates/noyau/src/hauteur.rs` et `crates/noyau/tests/hauteur.rs` ; modifier
`crates/noyau/src/lib.rs`. `proptest` entre en dépendance de test.

**Interfaces :**
- Produit :

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Intervalle { pub degres: i8, pub demi_tons: i8 }
impl Intervalle {
    pub const fn new(degres: i8, demi_tons: i8) -> Self;
    pub fn plus(self, autre: Intervalle) -> Intervalle;
    pub fn moins(self, autre: Intervalle) -> Intervalle;
    pub fn reduit(self) -> Intervalle;      // degrés dans 0..7, demi-tons décalés de 12 par octave
    pub fn alteration(self) -> i8;          // demi-tons − [0, 2, 4, 5, 7, 9, 11][degrés], sur l'intervalle réduit
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lettre { C, D, E, F, G, A, B }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NomDeNote { pub lettre: Lettre, pub alteration: i8 }
impl NomDeNote {
    pub fn plus(self, i: Intervalle) -> NomDeNote;
    pub fn intervalle_vers(self, autre: NomDeNote) -> Intervalle; // ascendant : degrés (b − a) mod 7, demi-tons naturels mod 12 + écart d'altération
    pub fn classe(self) -> u8;                                     // do = 0
    pub fn enharmonique_simple(self) -> NomDeNote;                 // identité si |altération| ≤ 1
    pub fn meme_son(self, ecart_de_lettre: i8) -> NomDeNote;       // même classe, lettre décalée
    pub fn anglais(self) -> String;                                // "E♭", "F♯", "B𝄫"
    pub fn solfege(self) -> String;                                // "mi♭", "fa♯", "si𝄫"
}
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
#[test] fn reduire() { assert_eq!(Intervalle::new(9, 16).reduit(), Intervalle::new(2, 4));
                       assert_eq!(Intervalle::new(-5, -9).reduit(), Intervalle::new(2, 3)); }
#[test] fn alterations() { assert_eq!(Intervalle::new(1, 1).alteration(), -1);
                           assert_eq!(Intervalle::new(3, 6).alteration(), 1);
                           assert_eq!(Intervalle::new(5, 8).alteration(), -1);
                           assert_eq!(Intervalle::new(6, 11).alteration(), 0); }
#[test] fn nommer() { // C + tierce majeure = E ; D + quinte = A ; F♯ + tierce majeure = A♯ ;
                      // D♭ + sixte mineure = B𝄫 ; G♯ + septième majeure = F𝄪
}
#[test] fn intervalle_vers() { // G♯→G = (0, −1) ; D♭→E = (1, 3) ; E♭→B = (4, 8) ; A→C = (2, 3)
}
#[test] fn classes() { // C = 0, B♯ = 0, C♭ = 11, F𝄪 = 7
}
#[test] fn enharmoniques() { // B𝄫 → A ; F𝄪 → G ; E♯ reste E♯ ; D♭.meme_son(−1) = C♯ ; F♭.meme_son(−1) = E ; A♯.meme_son(1) = B♭
}
#[test] fn noms() { // E♭ : "E♭", "mi♭" ; F♯ : "F♯", "fa♯" ; C : "C", "do"
}
proptest! {
    #[test] fn transposer_puis_revenir(n in note_simple(), i in intervalle_simple()) {
        prop_assert_eq!(n.plus(i).plus(Intervalle::new(0, 0).moins(i)), n);
        prop_assert_eq!(n.intervalle_vers(n.plus(i)), i.reduit());
    }
}
```

`note_simple()` tire une lettre et une altération dans −1..=1 ; `intervalle_simple()` tire un
intervalle de degrés 0 à 6 et d'altération −1 à 1.

- [ ] **Étape 2 : lancer les tests et les voir échouer** — `cargo test -p noyau --test hauteur` :
  échec de compilation.
- [ ] **Étape 3 : écrire `hauteur.rs`.**
- [ ] **Étape 4 : lancer les tests et les voir passer** — `cargo test -p noyau --test hauteur` :
  tous réussis.
- [ ] **Étape 5 : faire le commit**

```bash
git add crates/noyau/Cargo.toml Cargo.lock crates/noyau/src/lib.rs crates/noyau/src/hauteur.rs crates/noyau/tests/hauteur.rs
git commit -m "feat(noyau): les hauteurs, des intervalles orthographiés"
```

### Tâche 6 : les tonalités

**Fichiers :** créer `crates/noyau/src/tonalite.rs`, `crates/noyau/tests/tonalite.rs`.

**Interfaces :**
- Consomme : `Intervalle`, `NomDeNote` (tâche 5).
- Produit :

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum Mode { Majeur, Mineur }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub struct Tonalite { pub tonique: NomDeNote, pub mode: Mode }
impl Tonalite {
    pub fn les_24() -> [Tonalite; 24];                 // dans l'ordre du cahier
    pub fn gamme(self) -> [Intervalle; 7];             // majeur 0 2 4 5 7 9 11 ; mineur naturel 0 2 3 5 7 8 10
    pub fn note(self, i: Intervalle) -> NomDeNote;
    pub fn degre_de_classe(self, classe: u8) -> Option<Intervalle>; // le degré de la gamme qui a cette classe
    pub fn nom(self) -> String;                        // "do majeur", "mi♭ mineur"
}
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
#[test] fn les_24_dans_l_ordre_du_cahier() {
    let noms: Vec<_> = Tonalite::les_24().iter().map(|t| t.nom()).collect();
    assert_eq!(noms, ["do majeur", "la mineur", "sol majeur", "mi mineur", "ré majeur", "si mineur",
        "la majeur", "fa♯ mineur", "mi majeur", "do♯ mineur", "si majeur", "sol♯ mineur",
        "fa♯ majeur", "mi♭ mineur", "ré♭ majeur", "si♭ mineur", "la♭ majeur", "fa mineur",
        "mi♭ majeur", "do mineur", "si♭ majeur", "sol mineur", "fa majeur", "ré mineur"]);
}
#[test] fn chaque_majeure_est_suivie_de_sa_relative() { // tonique[k+1].classe = tonique[k].classe + 9 mod 12, mode mineur
}
#[test] fn gammes() { // do majeur : C D E F G A B ; la mineur : A B C D E F G ;
                      // fa♯ majeur : F♯ G♯ A♯ B C♯ D♯ E♯ ; mi♭ mineur : E♭ F G♭ A♭ B♭ C♭ D♭
}
#[test] fn aucune_gamme_n_a_de_double_alteration() { // les 24 × 7 notes : |altération| ≤ 1
}
#[test] fn degre_de_classe() { // fa majeur, classe 10 → la quarte (3, 5), soit B♭ ; do majeur, classe 1 → None
}
```

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test tonalite` échoue à la compilation, puis
  `tonalite.rs` est écrit, puis les tests passent.
- [ ] **Étape 5 : faire le commit** — `feat(noyau): les 24 tonalités, dans l'ordre du cahier`.

### Tâche 7 : les accords et leurs noms

**Fichiers :** créer `crates/noyau/src/accord.rs`, `crates/noyau/tests/accord.rs`.

**Interfaces :**
- Consomme : tâches 5 et 6.
- Produit :

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Qualite { Majeur, Mineur, Diminue, Augmente, Septieme, Diminue7 }
impl Qualite {
    pub fn intervalles(self) -> &'static [Intervalle]; // depuis la fondamentale
    pub fn tierce_majeure(self) -> bool;               // Majeur, Augmente, Septieme
    pub fn trois_sons(self) -> bool;
    pub fn suffixe(self) -> &'static str;              // "", "m", "°", "+", "7", "°7"
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Accord { pub fondamentale: Intervalle, pub qualite: Qualite, pub renversement: u8 } // fondamentale depuis la tonique ; 1 = tierce à la basse
impl Accord {
    pub fn notes(&self) -> Vec<Intervalle>;              // depuis la tonique, fondamentale d'abord
    pub fn basse(&self) -> Intervalle;
    pub fn fondamentale_juste(&self, t: Tonalite) -> NomDeNote;
    pub fn ecart_d_ecriture(&self, t: Tonalite) -> i8;   // 0, ou l'écart de lettre de Q21
    pub fn nom(&self, t: Tonalite) -> String;            // "A7", "D♭/F", "A" pour B𝄫, "G°7" pour F𝄪°7
    pub fn noms_des_notes(&self, t: Tonalite) -> Vec<NomDeNote>; // l'écriture « comme le nom »
}
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
#[test] fn noms_dans_la_table() {
    // do majeur : ♭VI (5, 8) majeur → "A♭" ; (0, 1) diminué 7 → "C♯°7" ; ♭II (1, 1) majeur, renversement 1 → "D♭/F"
    // la mineur : III (2, 3) augmenté → "C+"
    // ré♭ majeur : ♭VI → "A" ; sol♯ mineur : vii°7/i, (6, 11) diminué 7 → "G°7"
    // mi♭ mineur : V7/i, (4, 7) septième → "B♭7" ; fa♯ majeur : iii (2, 4) mineur → "A♯m"
}
#[test] fn notes_ecrites_comme_le_nom() {
    // ré♭ majeur, ♭VI → la, do♯, mi ; sol♯ mineur, vii°7/i → sol, si♭, ré♭, fa♭
    // sol♯ mineur, V7/i (D♯7) → ré♯, sol, la♯, do♯ ; do majeur, V7/ii (A7) → la, do♯, mi, sol
}
#[test] fn tierces() { // majeur, augmenté, septième : tierce majeure ; mineur, diminué, diminué 7 : non
}
```

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test accord`.
- [ ] **Étape 5 : faire le commit** — `feat(noyau): les accords, leurs notes et leurs noms`.

### Tâche 8 : la table (MIC-12, I26)

**Fichiers :** créer `crates/noyau/src/table.rs`, `crates/noyau/tests/table.rs`.

**Interfaces :**
- Consomme : tâches 5 à 7.
- Produit :

```rust
pub enum Famille { Diatonique, DominanteSecondaire, Emprunt, Sombre }
pub enum Rangee { Dessus, Centre, Sombre, Dessous }      // de haut en bas
pub enum Fonction { Repos, Eloignement, Tension }
#[derive(Default)] pub enum CoucheSombre { #[default] Masquee, Affichee }
pub struct Position { pub rangee: Rangee, pub rang: u8 } // rang : colonne 0..7 ; dans la couche sombre, l'ordre 0..n
pub struct Case { pub position: Position, pub famille: Famille, pub chiffre: String, pub accord: Accord,
                  pub cible: Option<Position>, pub fonction: Option<Fonction> }
pub struct Table { pub tonalite: Tonalite, pub couche_sombre: CoucheSombre, pub cases: Vec<Case> }
pub fn table(t: Tonalite, couche: CoucheSombre) -> Table;
impl Table {
    pub fn case(&self, p: Position) -> Option<&Case>;
    pub fn rangee(&self, r: Rangee) -> Vec<&Case>;            // dans l'ordre des rangs
    pub fn nom(&self, c: &Case) -> String;
    pub fn ligne_d_effet(&self, c: &Case) -> String;
    pub fn cible_de(&self, p: Position) -> Option<Position>;
}
```

- [ ] **Étape 1 : écrire les tests qui échouent** — les valeurs sont celles de « Les règles · La
  table ».

```rust
#[test] fn mic12_1_do_majeur()            // centre, chiffres ; dessus A7..E7 aux rangs 1..5, V7/ii..V7/vi, cibles au centre 1..5
#[test] fn mic12_2_la_mineur()            // centre Am..G ; E7 au rang 0, cible centre 0 ; dessus aux rangs 0, 2, 3, 4, 5, 6
#[test] fn mic12_3_de_do_a_mi_bemol()     // mêmes positions et chiffres ; C→E♭, A7→C7, B°→D°
#[test] fn mic12_4_fa_diese_et_mi_bemol_mineur() // iii de fa♯ majeur : A♯m ; V7/i de mi♭ mineur : B♭7
#[test] fn mic12_5_couche_sombre_masquee_par_defaut() // CoucheSombre::default() == Masquee ; aucune case Sombre
#[test] fn mic12_9_re_bemol_majeur()      // ♭VI : "A"
#[test] fn emprunts_par_colonne()         // les deux rangées Dessous, noms et chiffres
#[test] fn couche_sombre_majeure()        // do majeur : noms, chiffres, cibles centre 0, 1, 2, 3, 4, 4, 5
#[test] fn couche_sombre_mineure()        // la mineur : 8 cases ; III+ sans cible ; ♭II6 → Dessous 4 ; les autres au centre 0, 2, 3, 4, 5, 6
#[test] fn lignes_d_effet()               // "tire vers Dm", "assombrit", "éclaire", "glisse vers Dm",
                                          // "tension sombre avant la dominante", "suspend", les fonctions du centre
#[test] fn une_case_sans_cible()          // cible_de : None pour un diatonique, un emprunt et III+ (vigilance 3)
#[test] fn mic12_7_chaque_fleche_relie_deux_cases() // 24 tonalités, Affichee : la cible existe, dans la rangée voisine, distincte de la source
#[test] fn aucun_nom_de_la_table_n_a_de_double_alteration() // 24 tonalités, Affichee
#[test] fn i26_translation_des_noms() {
    // les 288 paires de tonalités de même mode, Affichee : mêmes (position, chiffre) ;
    // classe(fondamentale t2) = classe(fondamentale t1) + classe(tonique t2) − classe(tonique t1) mod 12 ;
    // si aucun des deux noms n'est réécrit (ecart_d_ecriture == 0) : lettres décalées de l'écart de lettre des toniques
}
```

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test table`.
- [ ] **Étape 5 : faire le commit** — `feat(noyau): la table de la tonalité (MIC-12, I26)`.

### Tâche 9 : le job `mutants`, et la fusion de la PR 3

**Fichiers :** créer `.cargo/mutants.toml`, `.github/workflows/mutants-complet.yml` ; modifier
`.github/workflows/ci.yml` (job `mutants`), `CONVENTIONS.md` (porte `mutants`).

- [ ] **Étape 1 : lancer la mutation et relever les survivants**

`.cargo/mutants.toml` examine `crates/noyau/src/{hauteur,tonalite,accord,table,palette,clavier,guitare}.rs`.
Lancer : `cargo mutants -p noyau`.
Attendu au premier passage : la liste des mutants manqués (`missed`).

- [ ] **Étape 2 : tuer chaque survivant par un test**, dans le fichier de test du module. Un mutant
  équivalent s'exclut nommément dans `exclude_re`, avec sa raison en commentaire.
- [ ] **Étape 3 : relancer** — `cargo mutants -p noyau`. Attendu : `0 missed`.
- [ ] **Étape 4 : les jobs.** Dans `ci.yml`, un job `mutants`, sur une PR seulement
  (`if: github.event_name == 'pull_request'`). Il extrait tout l'historique, écrit
  `git diff origin/${{ github.base_ref }}...HEAD > pr.diff`, puis lance
  `cargo mutants -p noyau --in-diff pr.diff`. `mutants-complet.yml` fait le passage complet chaque
  lundi et à la demande (`workflow_dispatch`). Le délai de chaque job vaut deux fois la durée mesurée.
- [ ] **Étape 5 : faire le commit, ouvrir la PR, fusionner** quand tout est vert.

```bash
git add .cargo/mutants.toml .github/workflows/ci.yml .github/workflows/mutants-complet.yml CONVENTIONS.md crates/noyau/tests
git commit -m "ci: le test de mutation du noyau, sur chaque PR et chaque semaine"
```

- [ ] **Étape 6 : 👤** ajouter `mutants` aux contrôles requis.

---

## PR 4 · Palette, clavier, guitare

Branche `inc-0-palette-clavier-guitare`. MIC-13, REN-08, REN-04, I27.

### Tâche 10 : la palette (MIC-13, I27)

**Fichiers :** créer `crates/noyau/src/palette.rs`, `crates/noyau/tests/palette.rs`,
`crates/noyau/tests/oracle_palette.rs` ; modifier `docs/20-sfd/22-micro-motif-et-phrase.md` (MIC-13 :
la règle de calcul dans « Règles » ; l'écriture des notes dans « Cas limites »).

**Interfaces :**
- Consomme : tâches 5 à 8.
- Produit :

```rust
pub enum Role { Appui, Tension, Frotte }
pub fn role(note: Intervalle, accord: &Accord, t: Tonalite) -> Option<Role>; // note depuis la tonique
pub struct NoteDePalette { pub note: Intervalle, pub nom: NomDeNote, pub role: Role }
pub fn palette(accord: &Accord, t: Tonalite) -> Vec<NoteDePalette>; // de la fondamentale vers l'aigu, dans l'octave, une note par classe
```

- [ ] **Étape 1 : écrire l'oracle, à la main** (`tests/oracle_palette.rs`). Chaque ligne donne les
  appuis, les tensions et les notes qui frottent, en solfège. Sa raison est la phrase de MIC-06 ou
  de MIC-13 qui la fonde, rappelée en commentaire : 9, 11, 13 tenues ; ♭9 ; 11 sur majeur ; ♭13 sur
  mineur ; septième (Q17) ; note altérée.

| Tonalité | Case | Appuis | Tensions | Frottent |
|---|---|---|---|---|
| do majeur | I `C` | do mi sol | ré (9), la (13) | fa (11 sur majeur), si (7e) |
| do majeur | ii `Dm` | ré fa la | mi (9), sol (11), si (13) | do (7e) |
| do majeur | iii `Em` | mi sol si | la (11) | fa (♭9), do (♭13 sur mineur), ré (7e) |
| do majeur | IV `F` | fa la do | sol (9), si (♯11), ré (13) | mi (7e) |
| do majeur | V `G` | sol si ré | la (9), mi (13) | do (11 sur majeur), fa (7e) |
| do majeur | vi `Am` | la do mi | si (9), ré (11) | fa (♭13 sur mineur), sol (7e) |
| do majeur | vii° `B°` | si ré fa | mi (11) | do (♭9), sol (♭13 sur mineur), la (7e) |
| do majeur | V7/ii `A7` | la do♯ mi sol | si (9), fa (♭13 sur majeur) | do (tierce altérée), ré (11 sur majeur) |
| do majeur | ♭III `E♭` | mi♭ sol si♭ | fa (9), la (♯11), do (13) | ré (7e), mi (unisson altéré), si (quinte altérée) |
| do majeur | ♭II6 `D♭/F` | ré♭ fa la♭ | mi (♯9), sol (♯11) | do (7e), ré (unisson altéré), la (quinte altérée), si (sixte augmentée) |
| la mineur | i `Am` | la do mi | si (9), ré (11) | fa (♭13 sur mineur), sol (7e) |
| la mineur | ii° `B°` | si ré fa | mi (11) | do (♭9), sol (♭13 sur mineur), la (7e) |
| la mineur | III `C` | do mi sol | ré (9), la (13) | fa (11 sur majeur), si (7e) |
| la mineur | iv `Dm` | ré fa la | mi (9), sol (11), si (13) | do (7e) |
| la mineur | v `Em` | mi sol si | la (11) | fa (♭9), do (♭13 sur mineur), ré (7e) |
| la mineur | VI `F` | fa la do | sol (9), si (♯11), ré (13) | mi (7e) |
| la mineur | VII `G` | sol si ré | la (9), mi (13) | do (11 sur majeur), fa (7e) |
| la mineur | V7/i `E7` | mi sol♯ si ré | do (♭13 sur majeur) | la (11 sur majeur), fa (♭9), sol (tierce altérée) |
| la mineur | IV `D` | ré fa♯ la | mi (9), si (13) | sol (11 sur majeur), do (7e), fa (tierce altérée) |
| la mineur | vii°7/i `G♯°7` | sol♯ si ré fa | — | la (♭9), do (quarte diminuée), mi (♭13 sur mineur), sol (octave diminuée) |

Le test parcourt l'oracle et compare, note par note, `role()` au rôle attendu.

- [ ] **Étape 2 : écrire les autres tests** (`tests/palette.rs`)

```rust
#[test] fn mic13_1_do_en_do_majeur()   // appuis do mi sol ; tensions ré la ; frottent fa si
#[test] fn mic13_2_sol7_en_do_majeur() // appuis sol si ré fa ; tensions la mi ; frotte do
#[test] fn mic13_3_la_m_en_do_majeur() // appuis la do mi ; tensions si ré ; frottent fa sol
#[test] fn mic13_5_mi_bemol_en_do_majeur() // appuis mi♭ sol si♭ ; aucune autre note hors de la tonalité
#[test] fn les_appuis_sont_exactement_les_notes_de_l_accord() {
    // 24 tonalités, chaque case, Affichee : classes des appuis = classes de l'accord ;
    // toute autre note de la palette est de la gamme ; une note par classe de (accord ∪ gamme)
}
#[test] fn la_palette_ne_classe_que_par_role() // chaque NoteDePalette.role == role(note, …)
#[test] fn les_notes_s_ecrivent_comme_le_nom() // ♭VI de ré♭ majeur : la, do♯, mi ; D♯7 en sol♯ mineur : sol, pas fa𝄪
```

- [ ] **Étape 3 : lancer les tests et les voir échouer** — `cargo test -p noyau --test palette --test oracle_palette`.
- [ ] **Étape 4 : écrire `role` et `palette`.** `role` est la seule fonction qui classe. `palette`
  l'appelle pour chaque note de l'accord et de la gamme.
- [ ] **Étape 5 : lancer les tests et les voir passer.**
- [ ] **Étape 6 : amender MIC-13** avec la règle et l'écriture (« Les règles · La palette »). Lancer
  les trois vérifications du projet, puis faire le commit :
  `feat(noyau): la palette d'un accord, par une seule fonction role() (MIC-13, I27)`.

### Tâche 11 : le clavier (REN-08)

**Fichiers :** créer `crates/noyau/src/clavier.rs`, `crates/noyau/tests/clavier.rs` ; modifier
`docs/60-moteur-musical.md` § 6.3.

**Interfaces :**
- Consomme : tâches 5 à 8.
- Produit :

```rust
pub struct Voicing { pub notes: Vec<u8> }   // MIDI, ascendant, la basse d'abord
pub fn candidats(accord: &Accord, t: Tonalite) -> Vec<Voicing>;
pub fn hors_contexte(accord: &Accord, t: Tonalite) -> Voicing;
pub fn deplacement(a: &Voicing, b: &Voicing) -> u32;
pub fn en_contexte(precedent: Option<&Voicing>, accord: &Accord, t: Tonalite) -> Voicing;
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
#[test] fn ren08_1_do_hors_contexte()  // [60, 64, 67]
#[test] fn ren08_2_do_apres_sol() {
    // après [67, 71, 74] : [67, 72, 76] ; déplacements 6, 7 ([64, 67, 72]) et 21 ([60, 64, 67])
}
#[test] fn premier_accord_sans_precedent() // en_contexte(None, …) == hors_contexte(…)
#[test] fn la_napolitaine_garde_sa_basse() // D♭/F en do majeur : [65, 68, 73] ; tous ses candidats ont la basse 65
#[test] fn candidats_en_position_serree()  // table des 24 tonalités : basse dans 60..=71, notes ascendantes au plus près, une par note de l'accord
#[test] fn en_contexte_minimal_et_basse_la_plus_grave() // centres des 24 tonalités, accords consécutifs : déplacement minimal ; à égalité, aucune basse plus grave
```

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test clavier`.
- [ ] **Étape 5 : amender le § 60 · 6.3** (candidats, déplacement, égalités, do 3 = MIDI 60), puis
  faire le commit : `feat(noyau): le voicing au clavier, hors contexte et en contexte (REN-08)`.

### Tâche 12 : la guitare (REN-04)

**Fichiers :** créer `crates/noyau/src/guitare.rs`, `crates/noyau/tests/guitare.rs` ; modifier
`docs/60-moteur-musical.md` § 6.1.

**Interfaces :**
- Consomme : tâches 5 à 8.
- Produit :

```rust
pub const ACCORDAGE: [u8; 6] = [40, 45, 50, 55, 59, 64];
pub const PAS_DE_DOIGTE: &str = "pas de doigté jouable";
pub enum Corde { Etouffee, AVide, Frette(u8) }
pub struct Barre { pub frette: u8, pub de: usize, pub a: usize }       // indices de cordes, grave = 0
pub struct Doigte { pub cordes: [Corde; 6], pub barre: Option<Barre>, pub doigts: [Option<u8>; 6] }
impl Doigte {
    pub fn depuis(notation: &str) -> Option<Doigte>;   // "x32010", ou "x-8-10-10-10-8" ; calcule barré et doigts
    pub fn notation(&self) -> String;                  // un caractère par corde si toutes les frettes < 10, sinon des tirets
    pub fn zone(&self) -> u8;
    pub fn nombre_de_doigts(&self) -> u8;
}
pub struct NotesVoulues { pub basse: u8, pub classes: Vec<u8>, pub facultative: Option<u8> } // facultative : la quinte
impl NotesVoulues { pub fn de(accord: &Accord, t: Tonalite) -> NotesVoulues }
pub struct Forme { pub nom: &'static str, pub frettes: [Option<u8>; 6], pub mobile: bool }
pub const FORMES: &[Forme];                            // la table de « Les règles · La guitare »
pub fn candidats(v: &NotesVoulues) -> Vec<Doigte>;
pub fn hors_contexte(v: &NotesVoulues) -> Option<Doigte>;
pub enum Maillon { Libre(NotesVoulues), Fixe(Doigte) }
pub fn en_contexte(suite: &[Maillon]) -> Vec<Option<Doigte>>;
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
#[test] fn numeros_de_doigts() {
    // x32010 → 3, 2, 1 ; x02220 → 1, 2, 3 sans barré ; 133211 → barré 1 de la corde 0 à 5, puis 3, 4, 2 ; x24432 → barré, puis 3, 4, 2
}
#[test] fn doigtes_de_reference() {
    for (accord, attendu) in [("C", "x32010"), ("A", "x02220"), ("G", "320003"), ("E", "022100"),
        ("D", "xx0232"), ("F", "133211"), ("B♭", "x13331"), ("B", "x24442"), ("E♭", "xx1343"),
        ("A♭", "466544"), ("D♭", "x46664"), ("F♯", "244322"), ("Am", "x02210"), ("Em", "022000"),
        ("Dm", "xx0231"), ("Bm", "x24432"), ("F♯m", "244222"), ("C♯m", "x46654"), ("G♯m", "466444"),
        ("Cm", "x35543"), ("Gm", "355333"), ("Fm", "133111"), ("B♭m", "x13321"), ("E♭m", "xx1342"),
        ("C7", "x32310"), ("A7", "x02020"), ("G7", "320001"), ("E7", "020100"), ("D7", "xx0212"),
        ("B7", "x21202"), ("F7", "131211"), ("B♭7", "x13131"), ("F♯7", "242322"), ("C♯7", "x46464"),
        ("G♯7", "464544"), ("D♯7", "xx1323"), ("B°", "x2343x"), ("C♯°", "x4565x"), ("F♯°", "xx4575"),
        ("E°", "xx2353"), ("C♯°7", "x4535x"), ("G♯°7", "456464"), ("B°7", "x2313x"), ("F°7", "123131"),
        ("C+", "x32110"), ("E+", "032110"), ("F+", "xx3665"), ("D♭/F", "xx3121"), ("B♭/D", "xx0331"),
        ("F/A", "x03211"), ("C/E", "xx2010"), ("G/B", "x20003")] {
        assert_eq!(hors_contexte(&voulues(accord)).unwrap().notation(), attendu, "{accord}");
    }
}
#[test] fn ren04_1_le_contexte_change_le_doigte() {
    // [Libre(G), Libre(C), Libre(D)] → C = x32010
    // [Fixe(x-8-10-10-10-8), Libre(C), Fixe(x-10-12-12-12-10)] → C = 8-10-10-9-8-8 ; il diffère du premier ;
    // aucun autre candidat de C ne donne un déplacement total plus petit (recherche exhaustive)
}
#[test] fn ren04_2_tout_doigte_propose_tient_les_contraintes() {
    // 24 tonalités, toutes les cases, Affichee : chaque doigté proposé ne sonne que des notes de l'accord,
    // a la basse juste, toutes les notes sauf peut-être la quinte, une étendue ≤ 3, ≤ 4 doigts
}
#[test] fn chaque_accord_des_24_tables_a_un_doigte() // 24 tonalités, Affichee : hors_contexte est Some (84 accords distincts)
#[test] fn ren04_5_un_agregat_serre_n_a_pas_de_doigte() {
    // classes do, do♯, ré, ré♯, mi, basse do, aucune facultative → None ; le rendu écrira PAS_DE_DOIGTE et les notes
}
```

`voulues("B♭/D")` est une aide du test, qui construit les `NotesVoulues` d'un nom d'accord.

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test guitare`. Si un doigté de référence diffère,
  c'est la table des formes ou le départage qu'on corrige, jamais l'attendu : un attendu ne change
  qu'avec l'accord de la maîtrise d'ouvrage, consigné au registre.
- [ ] **Étape 5 : amender le § 60 · 6.1.** Y écrire les formes du recueil, les contraintes, le barré,
  les numéros de doigts, le départage hors contexte, la zone, le chemin et le maillon fixé. Cette
  retouche du § 60 déclenche la piste « Ranger les familles d'accords dans la théorie » : la faire
  dans la même PR, telle que `docs/pistes.md` la décrit, et la retirer de ce fichier.
  Relancer `cargo mutants -p noyau` (attendu : `0 missed`), puis les trois vérifications. Faire le
  commit : `feat(noyau): les doigtés de guitare par les formes du recueil (REN-04)`.
- [ ] **Étape 6 : ouvrir la PR 4, fusionner** quand tout est vert.

---

## PR 5 · Les rendus

Branche `inc-0-rendus`. La page de la table, le verso, le cahier et son PDF ; I17, I19, I25, les
minimums. `sha2` entre au noyau ; `resvg` et `lopdf` en dépendances de test.

### Tâche 13 : les cases et la page de la table

**Fichiers :** créer `crates/noyau/src/rendu/{case,table}.rs`, `crates/noyau/tests/{rendu_table,encre}.rs`,
`crates/noyau/tests/commun/mod.rs` ; modifier `crates/noyau/src/rendu/mod.rs`.

**Interfaces :**
- Consomme : `Table`, `Case`, `Position`, `CoucheSombre` (tâche 8) ; `rendu::{svg, texte, pages}`.
- Produit :

```rust
// rendu::mod
#[derive(Default)] pub enum Instrument { Guitare, Clavier, #[default] LesDeux }
pub struct Debordement { pub quoi: String }
// rendu::case
pub enum Etat { Ordinaire, Cible }
pub fn cadre(famille: Famille, r: Rect, etat: Etat) -> Element;        // la pointe en bas pour une dominante secondaire
pub fn pastille(f: Fonction, cx: f32, cy: f32) -> Element;              // 3 mm, ○ ◐ ●
pub fn fleche(de: (f32, f32), a: (f32, f32)) -> Element;                // trait plein, pointe pleine sur la cible
// rendu::table
pub struct Disposition { pub cases: Vec<(Position, Rect)> }
pub struct PageTable { pub page: Page, pub disposition: Disposition, pub arrivees: Vec<(Position, (f32, f32))> }
pub fn page_table(table: &Table, instrument: Instrument, cible: Option<Position>) -> Result<PageTable, Debordement>;
```

La grille est celle de la fiche témoin (tâche 2), si H09 l'a confirmée.

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
// tests/rendu_table.rs
#[test] fn toutes_les_pages_de_table_tiennent() // 24 tonalités × Masquee et Affichee : Ok, et verifier(svg) == Ok (vigilance 1)
#[test] fn les_minimums_de_la_page_de_table()   // corps ≥ 11 pt, trait ≥ 0,25 pt, pastilles ≥ 3 mm, par commun::mesurer
#[test] fn la_couche_masquee_le_dit()           // Masquee : le texte « couche sombre masquée » est composé (chasse connue, groupe présent)
#[test] fn l_en_tete_dit_les_reglages()         // « la mineur · table de la tonalité · tonalité choisie · instrument : guitare et clavier · couche sombre : affichée »
#[test] fn deux_fleches_sur_la_meme_cible_arrivent_a_part() // do majeur, Affichee : arrivées de F♯°7 et D♭/F sur G distantes ≥ 3 mm (vigilance 4)
#[test] fn chaque_fleche_relie_exactement_deux_cases() // ses deux bouts dans les Rect de la source et de la cible (MIC-12 · 7)
#[test] fn la_disposition_couvre_chaque_case()  // une Rect par case, sans chevauchement
// tests/encre.rs : I17 en pixels, couleur supprimée, 300 ppp, ≥ 2 % de la boîte englobante
#[test] fn i17_les_quatre_familles()            // les 6 paires de cadres, même contenu
#[test] fn i17_les_trois_fonctions()            // les 3 paires de pastilles
#[test] fn i17_la_case_cible()                  // Cible contre Ordinaire
```

`commun::mesurer` est l'aide de la tâche 2, déplacée ici. `commun::difference(svg_a, svg_b) -> f32`
rastérise par `resvg` à 300 ppp, force toute couleur non blanche au noir, et rend la part des pixels
qui diffèrent dans l'union des boîtes englobantes.

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test rendu_table --test encre`. Un `Debordement` sur
  une tonalité se corrige dans la grille, dans les minimums ; s'il ne se lève pas, on s'arrête et on
  revient à la conception.
- [ ] **Étape 5 : faire le commit** — `feat(noyau): la page de la table, ses cadres et ses flèches`.

### Tâche 14 : les diagrammes, les palettes dessinées, le verso

**Fichiers :** créer `crates/noyau/src/rendu/{manche,clavier,portee,verso}.rs`,
`crates/noyau/tests/rendu_verso.rs` ; compléter `crates/noyau/tests/encre.rs`.

**Interfaces :**
- Consomme : `guitare`, `clavier`, `palette` (PR 4) ; `rendu::case`, `rendu::texte`.
- Produit :

```rust
pub fn diagramme_de_manche(d: Option<&Doigte>, notes: &[NomDeNote], origine: (f32, f32)) -> Element; // None : PAS_DE_DOIGTE et les notes
pub fn palette_sur_le_manche(d: &Doigte, p: &[NoteDePalette], origine: (f32, f32)) -> Element;    // zone du doigté ± 2 frettes
pub fn diagramme_de_clavier(v: &Voicing, basse: NomDeNote, origine: (f32, f32)) -> Element;
pub fn palette_sur_le_clavier(p: &[NoteDePalette], origine: (f32, f32)) -> Element;
pub fn palette_sur_la_portee(p: &[NoteDePalette], origine: (f32, f32)) -> Element;              // une octave, sans clé, noms dessous
pub fn colonne_d_accord(a: &Accord, t: Tonalite, i: Instrument, r: Rect) -> Result<Element, Debordement>;
pub fn page_verso(table: &Table, i: Instrument) -> Result<Page, Debordement>;
pub fn fiche_d_accord(a: &Accord, t: Tonalite, i: Instrument) -> Result<Page, Debordement>;     // une colonne seule, pour l'établi
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
#[test] fn tous_les_versos_tiennent()           // 24 tonalités × 3 instruments : Ok, sous-ensemble respecté
#[test] fn les_minimums_du_verso()              // manches ≥ 18 mm, touches blanches ≥ 3 mm, corps ≥ 11 pt, trait ≥ 0,25 pt
#[test] fn ren08_1_do_marque_do_mi_sol()        // le clavier de C marque les touches 60, 64, 67 ; la basse « C » dessous
#[test] fn un_accord_injouable_dit_pas_de_doigte() // diagramme_de_manche(None, …) compose PAS_DE_DOIGTE et les notes, sans grille de manche
#[test] fn toute_case_a_sa_fiche()              // 24 tonalités, toutes les cases : fiche_d_accord est Ok pour les 3 instruments
// tests/encre.rs
#[test] fn i17_les_marques_du_manche()          // doigt, corde à vide, corde étouffée, barré : les 6 paires
#[test] fn i17_le_clavier()                     // touche jouée contre touche libre, blanche et noire
#[test] fn i17_les_roles_de_la_palette()        // appui, tension, frotte : les 3 paires
```

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test rendu_verso --test encre`.
- [ ] **Étape 5 : faire le commit** — `feat(noyau): les diagrammes, les palettes dessinées et le verso`.

### Tâche 15 : le cahier et son PDF (REN-01, REN-07, I19, I25)

**Fichiers :** créer `crates/noyau/src/rendu/cahier.rs`, `crates/noyau/tests/cahier.rs`,
`crates/cahier/src/main.rs`, `crates/cahier/tests/pdf.rs` ; modifier `crates/cahier/src/lib.rs`
(`engendrer`), `.github/workflows/ci.yml` (`windows` : le cahier en artefact `cahier-des-tables`),
`docs/41-pile-technique.md` § 5.7 (le sous-ensemble, les glyphes) et § 6 (`sha2`),
`docs/20-sfd/28-rendus-encre-impression.md` (REN-01 : l'ordre, le sommaire, le pied),
`docs/50-interfaces-et-appareils.md` § 3.7 (la maquette cède la place à la page du noyau : do majeur,
instrument guitare, couche sombre masquée, écrite dans `docs/assets/maquettes/table.svg`). Retirer `crates/cahier/src/temoin.rs`,
`crates/cahier/examples/temoin.rs`, `crates/cahier/tests/h09.rs`, et l'artefact `fiche-temoin`.

**Interfaces :**
- Consomme : tâches 13 et 14 ; `assembler` (tâche 2).
- Produit :

```rust
// noyau::rendu::cahier
pub fn empreinte(svg_sans_pied: &str) -> String;                           // 6 caractères hexadécimaux du SHA-256
pub fn pages_du_cahier(i: Instrument, date: &Date) -> Result<Vec<PageDuCahier>, Debordement>; // 49 pages, pied posé
// cahier
pub enum ErreurCahier { Debordement(Debordement), Pdf(ErreurPdf) }
pub fn engendrer(i: Instrument, date: &Date) -> Result<Vec<u8>, ErreurCahier>;
// commande : cahier --date AAAA-MM-JJ [--instrument guitare|clavier|les-deux] --sortie <fichier>
```

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
// noyau/tests/cahier.rs
#[test] fn le_cahier_compte_49_pages()          // sommaire, puis table et verso de chaque tonalité, dans l'ordre du cahier
#[test] fn le_sommaire_mene_a_chaque_table()    // 24 liens ; le k-ième mène à la page 1 + 2k ; zones ≥ 12 mm de haut ; « mi♭ mineur » après « fa♯ majeur » (vigilance 2)
#[test] fn le_pied_porte_page_date_et_empreinte() // « page 2 / 49 · rendu le 1er nov. 2026 · empreinte <6 hex> · verso : … »
#[test] fn l_empreinte_est_celle_de_la_page_sans_pied()
#[test] fn les_minimums_sur_les_49_pages()      // commun::mesurer sur chaque page
// cahier/tests/pdf.rs
#[test] fn ren07_1_le_pdf_a_les_pages_des_svg() // 49 pages, MediaBox 593,858 × 445,039 pt à 0,01 près (I19)
#[test] fn le_pdf_pose_ses_pages_sans_mise_a_l_echelle()
#[test] fn chaque_lien_du_pdf_mene_a_sa_table()
#[test] fn ren07_2_deux_engendrements_identiques() // I25 : mêmes octets, à la même date
#[test] fn la_commande_ecrit_le_cahier()         // cargo run -p cahier -- --date 2026-11-01 --sortie <temp> : le fichier existe et vaut engendrer(…)
```

REN-07 · 3 (une fiche imprimée en noir et blanc garde ses distinctions) est tenu par les tests I17 des
tâches 13 et 14, puisque le PDF pose les pages telles quelles.

- [ ] **Étapes 2 à 4** : `cargo test -p noyau --test cahier` et `cargo test -p cahier`.
- [ ] **Étape 5 : les documents et l'artefact.** Amender le § 41 · 5.7 (la liste du sous-ensemble,
  les glyphes en tracés), REN-01 (l'ordre, le sommaire, le pied) et le § 50 · 3.7. Le job `windows`
  lance `cargo run -p cahier --release -- --date $(git log -1 --format=%cs) --sortie cahier-des-tables.pdf`
  et le dépose en artefact.
- [ ] **Étape 6 : vérifier, faire le commit, ouvrir la PR 5, fusionner**

Lancer : `cargo test -p noyau -p cahier`, `cargo mutants -p noyau` (attendu : `0 missed`), puis les
trois vérifications du projet.

```bash
git add crates .github/workflows/ci.yml docs
git rm crates/cahier/src/temoin.rs crates/cahier/examples/temoin.rs crates/cahier/tests/h09.rs
git commit -m "feat(cahier): le cahier des tables en PDF, 49 pages, déterministe"
```

---

## PR 6 · L'établi

Branche `inc-0-etabli`. L'application ; l'export du cahier ; les contrôles statiques de l'interface.
`tauri-plugin-dialog` entre à l'application.

### Tâche 16 : l'application

**Fichiers :** créer `app/ui/src/{app,entete,vue_table,panneau_palette,export}.rs` ; modifier
`app/ui/src/main.rs` (retirer `page_d_essai`), `app/ui/style.css`, `app/src-tauri/src/lib.rs`,
`app/src-tauri/Cargo.toml`, `app/src-tauri/capabilities/default.json` (le dialogue d'enregistrement),
`docs/41-pile-technique.md` § 6.

**Interfaces :**
- Consomme : `Tonalite::les_24`, `table`, `Table::cible_de`, `Table::nom` (PR 3) ; `page_table`,
  `fiche_d_accord`, `Instrument` (PR 5) ; `cahier::engendrer`, `Date::depuis_iso`.
- Produit, côté `etabli` :

```rust
#[tauri::command]
async fn exporter_cahier(app: tauri::AppHandle, date: String, instrument: String) -> Result<Option<String>, String>;
// Ok(None) : boîte fermée, rien d'écrit ; Ok(Some(nom du fichier)) ; Err(message en français)
fn lire_instrument(s: &str) -> Option<Instrument>;     // "guitare", "clavier", "les-deux"
```

Les composants : `App` tient quatre signaux, `tonalite` (do majeur), `couche` (masquée), `instrument`
(les deux) et `touchee` (aucune). `VueTable` insère `page_table(...).page.svg()`, avec la cible de la
case touchée. Elle la couvre de boutons transparents, placés en pour-cent de la page d'après la
`Disposition` ; chaque libellé dit « <nom> · <chiffre> ». Le bouton « Palette » s'affiche à côté de la
case touchée. `PanneauPalette` insère `fiche_d_accord(...).svg()`. `export.rs` appelle
`window.__TAURI__.core.invoke("exporter_cahier", …)` avec la date du jour (`js_sys::Date`, au format
AAAA-MM-JJ). Il affiche « Cahier exporté : <nom> », ou le message d'erreur, et rien si la boîte a été
fermée. Hors de Tauri, le bouton dit « Export indisponible hors de l'application ». Le focus se voit par
un contour, une forme et non une couleur.

- [ ] **Étape 1 : écrire les tests qui échouent** (`app/src-tauri/src/lib.rs`, module de test)

```rust
#[test] fn lire_les_instruments() // "guitare", "clavier", "les-deux" → Some ; "piano" → None
#[test] fn le_nom_propose_du_fichier() // date 2026-11-01 → "cahier-des-tables-2026-11-01.pdf"
```

- [ ] **Étape 2 : lancer les tests et les voir échouer** — `cargo test -p etabli`, sous Windows.
- [ ] **Étape 3 : écrire les composants et la commande.**
- [ ] **Étape 4 : lancer les tests, construire, essayer**

Lancer : `cargo test -p etabli`, `cargo clippy -p interface --target wasm32-unknown-unknown -- -D warnings`,
puis `cargo tauri dev` dans `app/src-tauri`.
Attendu : les tests passent. À l'essai : la tonalité change d'un geste. La couche sombre s'affiche et
se masque. Toucher `A7` en do majeur met `Dm` en inverse. Toucher `C+` en la mineur ne met rien en
inverse, et « Palette » s'affiche (vigilance 3). Tab atteint chaque case. L'export écrit le cahier.
Fermer la boîte d'enregistrement n'écrit rien et n'affiche rien (vigilance 5).

- [ ] **Étape 5 : faire le commit** — `feat(etabli): la table interactive et l'export du cahier`. Le job
  `windows` ajoute `cargo test -p etabli`.

### Tâche 17 : les contrôles statiques de l'interface (I17, I18)

**Fichiers :** créer `app/ui/tests/encre_statique.rs`.

- [ ] **Étape 1 : écrire les tests**

```rust
#[test] fn i18_aucune_animation_ni_minuterie() {
    // dans app/ui/src/**/*.rs, app/ui/style.css et app/ui/index.html : aucun de
    // "animation", "transition", "@keyframes", "set_timeout", "set_interval", "setTimeout",
    // "setInterval", "request_animation_frame", "requestAnimationFrame", "<animate", "scroll-behavior: smooth"
}
#[test] fn i17_toute_classe_sem_qui_colore_a_un_indice_non_colorimetrique() {
    // chaque règle CSS dont un sélecteur contient ".sem-" et qui déclare color, background, background-color,
    // fill, stroke, border-color ou outline-color déclare aussi l'un de : border, border-style, border-width,
    // outline, text-decoration, font-style, font-weight, background-image, content, stroke-dasharray,
    // stroke-width, margin-top, top, transform ; le test échoue en nommant la classe
}
#[test] fn i17_le_controle_refuse_une_classe_coloree_seule() {
    // la fonction de contrôle, appliquée à ".sem-x { color: #c00 }", rend une erreur qui nomme ".sem-x"
}
```

- [ ] **Étape 2 : lancer les tests** — `cargo test -p interface --test encre_statique`. Le troisième
  échoue tant que la fonction de contrôle n'existe pas ; les deux premiers passent si l'interface est
  sobre.
- [ ] **Étape 3 : écrire la fonction de contrôle**, dans le fichier de test, et relancer : tous
  réussis.
- [ ] **Étape 4 : faire le commit, ouvrir la PR 6, fusionner** quand tout est vert —
  `test(interface): aucune animation, aucune couleur seule (I17, I18)`.

---

## PR 7 · La revue finale

### Tâche 18 : la revue de toute la branche, et les contrôles à l'œil

- [ ] **Étape 1 :** une revue de tout ce qu'INC-0 a fusionné, par un relecteur du modèle le plus
  capable, qui n'a pas écrit le code : `git diff d062af4..origin/main`, le plan, l'issue #1. Elle
  cherche les mots que les PR ont périmés dans tout le dépôt (LE-3) : « portrait », « 157 × 209,5 »,
  « Quatre, aujourd'hui », la plage A01–A21. Chaque constat retenu se corrige dans cette PR, et
  `/consigner` écrit au registre ceux qui le méritent.
- [ ] **Étape 2 : 👤 les contrôles à l'œil**, sur la version candidate (les artefacts du dernier
  `main`) : une page du cahier à bout de bras, couche sombre affichée, sur la Note Air et sur papier ;
  l'impression en noir et blanc des diagrammes ; l'application installée sur l'établi. La session les
  consigne au § 70 · 7, dans un tableau « Relevés » (date, version, contrôle, résultat).
- [ ] **Étape 3 :** la PR de revue, verte, fusionnée — `docs: la revue finale d'INC-0`.

## PR 8 · La clôture

### Tâche 19 : supprimer le plan, reporter ce qui survit

- [ ] **Étape 1 :** vérifier que chaque ligne de « Ce qui doit survivre » est écrite à sa place.
- [ ] **Étape 2 :** `CLAUDE.md` : l'état (INC-0 livré, les crates, les jobs) et les commandes de Rust
  (`cargo fmt --all --check`, `cargo clippy …`, `cargo test …`, `cargo mutants -p noyau`,
  `cargo tauri build`, la commande du cahier). Le § 70 · 8 : les portes ajoutées par INC-0.
  `docs/pistes.md` : ce qu'INC-0 a fait naître.
- [ ] **Étape 3 :** `git rm .claude/plans/inc-0-la-table.md` ; les trois vérifications du projet ; la PR
  « Closes #1 », verte, fusionnée — `chore: clôture d'INC-0`.
- [ ] **Étape 4 : 👤** l'étiquette `v0.2.0`. La session la prépare en local, sans l'envoyer.
