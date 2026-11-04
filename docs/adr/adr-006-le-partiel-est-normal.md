# ADR-006 — Le partiel est l'état normal

| | |
|---|---|
| **Statut** | ✅ acceptée — confirmée le 2026-11-04, complétée de l'idée non reliée et de la version de schéma |
| **Date** | 2026-10-11 |
| **Sert** | B01, B03, B11 · principe P4 · contre le risque R1 |
| **S'appuie sur** | — |

## Contexte

La matière de Sacha est incomplète et incertaine, **durablement** : des fragments sans
suite, des démos jamais mesurées, des titres provisoires, un album dont le nombre de titres
n'est pas arrêté et qui puise dans un fonds bien plus large. Un outil qui ne deviendrait
utile qu'une fois tout renseigné ne servirait jamais (R1).

## Décision

1. **Un morceau peut n'avoir qu'une identité, et une idée rester une idée.** Aucune saisie
   n'est obligatoire : ni titre, ni tonalité, ni prise. Une idée non reliée est un objet
   légitime et durable, pas une tâche en souffrance.
2. **Aucune fonction ne refuse de s'exécuter pour incomplétude.** Elle travaille sur ce
   qu'elle a et dit sur quoi. Le seul refus admis est l'impossibilité stricte, et elle est
   nommée (« pas de portée sans une seule note »).
3. **Tout agrégat déclare son assiette** : combien d'éléments considérés, combien portant la
   donnée, combien décidés.
4. **Aucun cardinal en dur.** Le nombre de morceaux d'un album est toujours compté ; le
   « cardinal visé » est une intention, lue par aucun diagnostic.
5. **La maturité est engendrée**, dimension par dimension ; le **statut** est décidé. Les deux
   ne se confondent pas.
6. **Indécidable n'est pas absent** : un constat dont la donnée manque le dit.

## Conséquences

### Ce que ça permet

- Commencer tout de suite : une idée de trois notes est déjà dans l'outil.
- Faire grandir le fonds sans migration : un morceau de plus est un cas normal, et un champ
  de plus se lit absent dans un objet plus ancien. Changer une structure, lui, demande une
  version de schéma ([ADR-023](adr-023-stockage-des-documents.md)).
- Le taux de complétude devient une information utile, et non plus un échec.

### Ce que ça coûte

- Chaque fonction doit gérer l'absence de chaque donnée qu'elle consomme.
- Les rendus doivent rester lisibles avec des trous.
- Les tests couvrent trois cas au lieu d'un : le vide, le partiel, le complet.

## Options écartées

| Option | Pourquoi |
|---|---|
| Des états de maturité saisis à la main | Un état global tenu à la main finit toujours par mentir sur un objet à vingt champs |
| Un minimum obligatoire (titre, tonalité) | Exiger une donnée provisoire, c'est la faire passer pour stable |
| Attendre que le nombre de titres soit fixé pour concevoir l'album | Le modèle doit survivre à toutes les réponses |

## Comment on vérifie

- **I07** : un morceau à identité seule se rend et se diagnostique partout, sans erreur.
- **I08** : la suite passe sur des albums de 0, 1, 11 et 57 morceaux ; aucun littéral de
  cardinal dans la logique d'album.
- **I09** : tout agrégat porte son assiette.
- Le critère 1 de [CAP-02](../20-sfd/21-capter-et-organiser.md#cap-02--trier-la-boîte-didées) :
  cinquante idées non reliées depuis des mois, et ni compteur d'ancienneté, ni alerte, ni
  badge.
