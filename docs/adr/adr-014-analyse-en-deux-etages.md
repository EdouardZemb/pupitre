# ADR-014 — L'analyse en deux étages, toujours mesurée

| | |
|---|---|
| **Statut** | 🟡 proposée |
| **Date** | 2026-10-11 |
| **Sert** | B01, B04, B08, B15 · principes P2, P5 |
| **S'appuie sur** | [ADR-002](adr-002-local-first.md) et [ADR-004](adr-004-proposer-n-est-pas-decider.md) |

## Contexte

Certaines analyses doivent être **immédiates et hors ligne** : nommer un accord joué au
piano, transcrire un fredonnement dans le train, accorder une guitare. D'autres sont
**lourdes** : séparer la voix d'un mix, trouver les accords d'une démo dense, transcrire le
texte chanté. La qualité des détecteurs varie beaucoup selon le type de prise, et une
confiance affichée doit vouloir dire quelque chose.

## Décision

| Étage | Où | Ce qu'il fait |
|---|---|---|
| **Léger** | Sur l'appareil, en Rust | Accordeur et diapason, nommer un accord, transcrire un fredonnement, tempo d'une prise courte |
| **Lourd** | Sur le nœud, analyseur Python | Séparation de sources, pulsations et premiers temps, tonalité, accords par mesure, mélodie de la voix, frontières, texte chanté |

- Un résultat d'analyse est une **captation immuable**, nommée par l'empreinte de son
  contenu et indexée par `(prise, analyseur@version, paramètres)` ; deux exécutions de
  même clé qui diffèrent donnent deux résultats, côte à côte. Un analyseur meilleur
  produit un nouveau résultat, à côté.
- **Aucun analyseur n'entre sans mesure** sur un **jeu de référence** de prises réelles,
  annoté par la maîtrise d'ouvrage.
- Les **confiances sont calibrées** sur ce jeu.
- Au-delà de **50 % de mesures douteuses** sur une prise, la grille proposée s'affiche en
  **liste d'indices** par mesure, pas en grille.
- **Aucune analyse de hauteur ne suppose 440** : le diapason est estimé d'abord.

## Conséquences

### Ce que ça permet

- Une réponse immédiate là où les mains sont prises, et dans le train.
- Des confiances honnêtes, donc des propositions utilisables.
- Une provenance reproductible : on sait quel analyseur a dit quoi.

### Ce que ça coûte

- Annoter le jeu de référence : une journée (6 à 10 heures), étalée, par la maîtrise
  d'ouvrage, dès que le tapotage
  ([MOR-03](../20-sfd/24-morceau.md#mor-03--découper-une-prise-à-loreille)) est livré dans
  INC-4 ; chronométrer les trois premières prises pour ajuster
  ([Q10](../90-questions-ouvertes.md)).
- Deux piles d'analyse et un banc d'évaluation à entretenir.

## Options écartées

| Option | Pourquoi |
|---|---|
| Tout analyser sur le nœud | Rien au piano sans réseau, rien dans le train |
| Tout analyser sur l'appareil | Les modèles lourds n'y tiennent pas |
| Afficher les confiances brutes | Un détecteur trop sûr de lui tromperait avec des chiffres |

## Comment on vérifie

- Le banc du [§ 70 · 5](../70-qualite-et-verification.md#5-le-banc-dévaluation-des-analyseurs),
  rejoué avant toute entrée d'analyseur.
- Critères d'[ECO-03](../20-sfd/27-ecoute-et-compagnon.md#eco-03--nommer-un-accord-joué),
  d'[ECO-04](../20-sfd/27-ecoute-et-compagnon.md#eco-04--accordeur-et-diapason) et
  d'[ECO-06](../20-sfd/27-ecoute-et-compagnon.md#eco-06--fredonner-une-mélodie).
