# ADR-009 — Écouter pour vérifier, jamais pour produire

| | |
|---|---|
| **Statut** | ✅ acceptée — validée au cadrage du 2026-10-11 |
| **Date** | 2026-10-11 |
| **Sert** | B07, B08 · principe P7 · contre le risque R4 |
| **S'appuie sur** | — |

## Contexte

Au cadrage, la question « l'outil doit-il pouvoir jouer ce qui est écrit ? » a reçu cette
réponse : **oui, une écoute de contrôle** — un rendu simple, piano ou guitare échantillonnés
et métronome, pour vérifier une grille ou une ligne loin de l'instrument ; **ni mixage, ni
effets, ni export audio comme produit**.

Le risque inverse a aussi un nom : **glisser vers le studio** (R4). Chaque pas y est petit et
raisonnable — « juste une réverbération », « juste une piste de batterie » — et la somme
fait une station audio, que Sacha n'a pas demandée et que d'autres outils font mieux.

## Décision

**Pupitre joue ce qui est écrit, pour le vérifier :**

- les accords, en voicing de piano ou selon le **doigté de guitare** retenu, et la mélodie ;
- un clic, un décompte ;
- au tempo décidé, ralentissable et bouclable, sans toucher au tempo du morceau ;
- au **diapason de lecture** réglé, pour sonner avec le piano de la maison ;
- et les **prises** elles-mêmes, alignées sur la forme, ralentissables pour travailler.

**Pupitre ne produit pas de son :** pas de mixage, pas d'effet, pas de rendu instrument par
instrument de l'arrangement, pas de batterie, pas d'interprétation « humanisée », **aucune
exportation audio**. Une maquette s'enregistre, ou se génère hors de Pupitre
([MOR-07](../20-sfd/24-morceau.md#mor-07--maquettes-et-brief-de-génération)).

## Conséquences

### Ce que ça permet

- Vérifier un enchaînement d'accords dans le train, au casque.
- Entendre exactement le doigté que montre le diagramme.
- Comparer en un geste une prise et l'écrit, à la même mesure.

### Ce que ça coûte

- Quelques mégaoctets d'échantillons en cache sur chaque appareil.
- Une discipline : chaque demande de « petit plus sonore » passe le test de périmètre du
  [§ 10](../10-vision.md#le-test-de-périmètre).

## Options écartées

| Option | Pourquoi |
|---|---|
| Aucun son synthétisé, seulement les prises | Contredit le besoin validé : entendre l'écrit loin de l'instrument |
| Une maquette audio exportable, arrangée par section | C'est de la production ; la frontière avec le séquenceur disparaît |

## Comment on vérifie

- **I21** : le graphe audio ne contient qu'échantillonneur, lecteur de prise (ralenti à
  hauteur constante), volume et sortie ; aucune exportation audio.
- Critères d'[ECO-01](../20-sfd/27-ecoute-et-compagnon.md#eco-01--écoute-de-contrôle).
