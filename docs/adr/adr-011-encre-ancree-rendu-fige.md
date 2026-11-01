# ADR-011 — L'encre s'ancre à un rendu figé

| | |
|---|---|
| **Statut** | 🟡 proposée |
| **Date** | 2026-10-11 |
| **Sert** | B12 · principe P6 |
| **S'appuie sur** | [ADR-001](adr-001-un-seul-materiau.md) et [ADR-010](adr-010-trois-appareils-mode-encre.md) |

## Contexte

Une annotation au stylet est **positionnelle** : un trait entoure trois mesures, une flèche
relie deux notes. Elle n'a de sens que sur la page qui était dessous.

Or toute page de Pupitre est un **rendu engendré** ([ADR-001](adr-001-un-seul-materiau.md)) :
elle change dès que le modèle change. Une encre ancrée à une page vivante finirait par
entourer d'autres mesures, et dire quelque chose de faux.

Et sur une tablette à encre, la **WebView** trace plus lentement que le **lecteur PDF
natif**, optimisé pour le stylet.

## Décision

1. Ouvrir une page pour l'annoter la **fige** : elle devient une captation, nommée par son
   empreinte.
2. Les traits sont capturés en vecteurs et **ancrés à cette empreinte**.
3. Quand le modèle change, le rendu annoté **reste tel quel**, consultable ; un **nouveau
   rendu vierge** apparaît.
4. **Aucune interprétation de l'encre** : ni reconnaissance d'écriture, ni détection de
   symbole, ni décision tirée d'un trait.
5. **Deux chemins** d'annotation : dans Pupitre ; ou par **aller-retour PDF** — le PDF du rendu
   figé s'annote dans le lecteur natif, se réimporte, et ses traits sont extraits et ancrés
   au même rendu.
6. Pour que les annotations ne s'oublient pas : chaque section du rendu courant affiche le
   **nombre d'annotations de ses rendus antérieurs**, rattachées à la section par la boîte
   englobante de leurs traits ; une annotation se marque **traitée**.
7. Les rendus sont **déterministes** : mêmes données, mêmes pages, même empreinte.

## Conséquences

### Ce que ça permet

- Ce qu'on relit est ce qu'on a écrit, sur la page qu'on avait sous les yeux.
- Le stylet garde la vitesse du lecteur natif.
- La suite des rendus annotés forme un journal visuel du morceau.

### Ce que ça coûte

- Après une modification, il faut réannoter la nouvelle page.
- Des annotations orphelines, que les comptes par section et la liste des rendus annotés
  gardent visibles.
- Une contrainte de déterminisme sur tout le moteur de rendu.
- L'extraction de l'encre des PDF se fait sur le nœud.

## Options écartées

| Option | Pourquoi |
|---|---|
| Ancrer l'encre aux objets musicaux plutôt qu'à la page | Il faudrait deviner ce qu'un trait désigne, donc interpréter l'encre |
| Faire suivre l'encre quand la page bouge | Une transformation silencieuse, et fausse sans alerte |
| Reconnaître l'écriture, puis faire valider | Disproportionné ; et l'annotation sert justement à noter ce qui n'est pas encore formulé |

## Comment on vérifie

- **I20** : après une modification, l'empreinte du rendu annoté et la position de ses traits
  sont inchangées ; aucun code n'interprète les traits (ni reconnaissance, ni détection de
  symbole, ni décision) : l'affichage, l'extraction d'un PDF annoté et le rattachement d'un
  trait à une section par sa boîte englobante sont les seules lectures de leur géométrie.
- **I25** : deux engendrements des mêmes données donnent la même empreinte.
- 🔶 **H02** : les deux chemins d'annotation sont essayés sur la tablette, avant de choisir le
  chemin par défaut.
