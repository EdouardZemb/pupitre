# ADR-010 — Trois appareils, trois rôles ; un mode encre sans couleur porteuse de sens

| | |
|---|---|
| **Statut** | ✅ acceptée — confirmée le 2026-10-28 |
| **Date** | 2026-10-11 |
| **Sert** | B08, B12 · principe P6 · contre le risque R3 |
| **S'appuie sur** | — |

## Contexte

Sacha a trois appareils aux capacités **opposées**, pas trois tailles d'écran :

- la **tablette à encre** : pas de couleur exploitable, un rafraîchissement lent avec des
  images rémanentes, mais une lecture reposante en plein jour et un stylet ;
- le **téléphone** : micro, haut-parleur, rapidité, toujours là, mais une surface trop
  petite pour une partition ;
- l'**ordinateur** : clavier, grand écran, sessions longues.

Une interface unique, simplement « adaptative », donnerait la plus mauvaise version des
trois.

## Décision

- **Une application, un modèle, trois rôles** : l'**établi** (construire), le **carnet**
  (capter, écouter, accompagner l'instrument), le **lutrin** (lire, annoter, répéter).
- La conception part des deux appareils **contraints** ; le confort de l'établi en découle.
- Le **mode encre** est un profil de présentation : pages fixes, aucune animation, **aucune
  information portée par la seule couleur** (forme, remplissage, glyphe, position), tailles
  fixées en millimètres. Sur un écran couleur, la couleur ne fait que **redoubler** la
  forme.
- Au lutrin, **aucune opération destructrice** ; accepter ou écarter une proposition se fait
  une à une, d'un geste large.

## Conséquences

### Ce que ça permet

- Chaque appareil fait ce qu'il fait bien, et « cette fonction n'existe pas ici » devient
  une réponse franche.
- Tout s'imprime en noir et blanc sans perte.
- Les personnes qui distinguent mal les couleurs lisent tout.

### Ce que ça coûte

- Un vrai travail d'encodage visuel, et une convention de nommage (`sem-`) pour le tester.
- Trois profils de test, dont un qu'aucun émulateur ne reproduit fidèlement : la lisibilité
  réelle se vérifie aussi à l'œil.

## Options écartées

| Option | Pourquoi |
|---|---|
| Une interface adaptative unique | Le dénominateur commun d'une dalle à encre et d'un téléphone est trop pauvre |
| La tablette réduite à une imprimante de PDF | La lecture hors ligne, la navigation dans un morceau et l'acceptation des propositions y ont leur place |
| La couleur comme encodage principal | Elle disparaît sur l'encre et sur le papier, et elle n'ordonne pas |

## Comment on vérifie

- **I17** : aucune information portée par la seule couleur (analyse statique et différence de
  pixels, [§ 50 · 6.5](../50-interfaces-et-appareils.md#65-comment-on-le-vérifie)).
- **I18** : aucune animation ni horloge en mode encre.
- **I19** : la même pagination au lutrin et à l'impression.
