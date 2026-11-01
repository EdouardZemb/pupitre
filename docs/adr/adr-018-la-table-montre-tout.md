# ADR-018 — La table montre tout sans choisir : y prendre un accord, c'est le saisir

| | |
|---|---|
| **Statut** | ✅ acceptée |
| **Date** | 2026-10-27 |
| **Sert** | B08, B09, B10 · principes P2 et P3 · contre le risque R5 |
| **S'appuie sur** | [ADR-004](adr-004-proposer-n-est-pas-decider.md) |

## Contexte

- La table de la tonalité
  ([MIC-12](../20-sfd/22-micro-motif-et-phrase.md#mic-12--parcourir-la-table-de-la-tonalité))
  montre d'un coup d'œil tous les accords qu'une tonalité met à portée, une rangée par
  famille, chacun avec l'accord où il mène
  ([§ 60](../60-moteur-musical.md#les-familles-daccords)). Depuis un morceau ouvert, Sacha
  peut y **prendre** un accord, qui s'insère à la position courante.
- Une vue qui offre des accords ressemble à une piste. Or l'[ADR-004](adr-004-proposer-n-est-pas-decider.md)
  fait de tout ce que la machine mesure ou imagine une proposition, et la part d'origine
  ([PRO-02](../20-sfd/26-propositions-suggestions-provenance.md#pro-02--provenance-et-part-dorigine))
  compte ce qui vient de pistes acceptées.
- La table n'imagine rien. Elle se calcule à partir de la tonalité seule, en degrés
  ([ADR-008](adr-008-hauteurs-relatives.md)), et elle est la même pour tout morceau dans
  cette tonalité : c'est un traité d'harmonie ouvert sur le pupitre, pas une réponse au
  morceau.
- La carte des chemins
  ([MIC-08](../20-sfd/22-micro-motif-et-phrase.md#mic-08--parcourir-la-carte-des-chemins)),
  elle, part de ce qui précède réellement et groupe ses chemins par effet : c'est une piste
  de règles, et un accord qu'on y ajoute porte la provenance `suggeree`. Les deux vues
  puisent dans les mêmes familles d'accords.
- Si un accord pris dans la table était une suggestion acceptée, la part d'origine
  compterait comme venu de la machine un accord que Sacha a choisi parmi tous les
  possibles. Et le mode sans suggestion
  ([PRO-05](../20-sfd/26-propositions-suggestions-provenance.md#pro-05--travailler-sans-suggestion))
  devrait cacher la table.

## Décision

**La table montre tout, sans choisir : elle n'est pas une proposition. Y prendre un accord,
c'est le saisir.**

1. La table montre **tous** les accords de ses rangées. Elle n'en classe, n'en recommande
   ni n'en retire aucun selon le morceau, et elle ne compose pas de suite.
2. Elle se calcule à partir de la tonalité et des réglages (instrument, couche sombre),
   jamais à partir du contenu du morceau : ses accords, sa mélodie, ses paroles. Ce n'est
   pas une piste au sens de l'[ADR-005](adr-005-des-pistes-pas-des-reponses.md).
3. **Prendre** un accord dans la table l'insère à la position courante du morceau ouvert,
   avec la provenance `saisie`. La part d'origine ne change pas.
4. La table seule n'écrit rien, et rien ne s'écrit sans le geste de prendre.
5. En mode sans suggestion, la table reste affichée, avec son geste de prendre.
6. La carte des chemins reste une piste de règles : un accord qu'on y ajoute porte
   `suggeree`. La ligne de partage entre les deux vues est celle-ci : une vue qui part de
   ce que Sacha a écrit propose ; une vue qui ne part que de la tonalité montre.

## Conséquences

### Ce que ça permet

- Une référence complète, à consulter sans arrière-pensée : la table ne choisit rien à la
  place de Sacha (P3).
- Une part d'origine qui ne compte que ce que la machine a choisi à partir du morceau
  (P2).
- La table reste à portée en mode sans suggestion.
- La table et la carte partagent leurs données de théorie sans se confondre.

### Ce que ça coûte

- Une frontière à tenir. Une table qui filtrerait ou ordonnerait ses cases selon le
  morceau deviendrait une piste : elle suivrait alors l'ADR-005 et écrirait `suggeree`.
  Marquer les accords employés
  ([MOR-14](../20-sfd/24-morceau.md#mor-14--poser-le-morceau-sur-la-table)) ne filtre ni
  n'ordonne rien : c'est un relevé du morceau.
- Un même accord porte `saisie` s'il est pris dans la table, `suggeree` s'il est ajouté
  depuis la carte. Le glossaire nomme les deux gestes différemment : prendre, et accepter.
- Le rapport de provenance ne dit pas qu'un accord saisi a été lu dans la table.

## Options écartées

| Option | Pourquoi |
|---|---|
| Prendre dans la table, c'est accepter une suggestion (`suggeree`) | La table ne choisit rien : la part d'origine compterait comme venu de la machine ce que Sacha a choisi parmi tous les possibles, et le mode sans suggestion devrait la cacher |
| Une provenance de plus, réservée à la table | Une sixième provenance pour un geste qui est une saisie ; le glossaire veut un mot pour une chose |
| Ordonner ou filtrer les cases selon le morceau, « les plus probables d'abord » | Une réponse déguisée (N6) ; c'est le rôle de la carte des chemins, qui part de ce qui précède et suit l'ADR-005 |

## Comment on vérifie

- Critère 6 de [MIC-12](../20-sfd/22-micro-motif-et-phrase.md#mic-12--parcourir-la-table-de-la-tonalité) :
  un accord pris dans la table porte la provenance `saisie`, et la part d'origine est
  inchangée.
- Critère 4 de [MIC-08](../20-sfd/22-micro-motif-et-phrase.md#mic-08--parcourir-la-carte-des-chemins) :
  un accord ajouté depuis la carte porte `suggeree`, jamais `saisie`.
- Critère 4 de [PRO-05](../20-sfd/26-propositions-suggestions-provenance.md#pro-05--travailler-sans-suggestion) :
  en mode sans suggestion, la table reste affichée, et un accord qu'on y prend porte
  `saisie`.
- Un test du noyau : la table d'une tonalité est la même pour deux morceaux différents dans
  cette tonalité, cases et ordre compris.
