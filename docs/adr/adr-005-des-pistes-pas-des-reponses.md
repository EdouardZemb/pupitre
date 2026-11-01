# ADR-005 — Des pistes, pas des réponses

| | |
|---|---|
| **Statut** | 🟡 proposée — le principe de suggestions de contenu a été validé au cadrage ; les règles ci-dessous en sont la mise en forme |
| **Date** | 2026-10-11 |
| **Sert** | B09, B10 · principe P3 · contre le risque R5 |
| **S'appuie sur** | [ADR-004](adr-004-proposer-n-est-pas-decider.md) |

## Contexte

Au cadrage, la place de l'IA a été fixée : Pupitre peut **proposer du contenu** — accords
suivants, rimes, variantes de mélodie ou de ligne — **toujours sous décision humaine**.

Le risque est réel et a un nom : **un outil qui écrit à la place** (R5). Sacha ne veut ni
qu'on écrive à sa place (N1), ni une recommandation unique qui serait une réponse déguisée
(N6). Et un modèle génératif produit des sorties plausibles mais souvent non conformes :
une ligne au mauvais compte syllabique, une rime que la doctrine refuse.

## Décision

Toute piste, de règles ou générative, obéit au même contrat :

| Règle | Pourquoi |
|---|---|
| **À la demande**, sur un élément précis | L'outil se tait quand on compose |
| **Au moins deux**, ou l'aveu qu'il n'y en a pas | Une seule piste est une réponse |
| **Expliquée** : son effet ou sa raison | On choisit en comprenant |
| **Vérifiée** par une validation déterministe (théorie, doctrine, profil d'outil) | C'est ce qui permet d'employer un modèle sans lui faire confiance |
| **Écoutable** quand elle est musicale | Un accord se juge à l'oreille |
| **Groupée par effet**, jamais classée par qualité | Pas de « meilleure piste » ; un ordre objectif et nommé (le coût en mesures ajoutées, par exemple) est admis, et dit |
| **Tracée** à l'acceptation | La part d'origine reste lisible |
| **Bornée** : jamais une section, une mélodie ou un texte entiers | La frontière avec l'écriture à la place |

S'y ajoutent un **mode sans suggestion** (global ou par morceau) et la **part d'origine**
visible à la demande.

## Conséquences

### Ce que ça permet

- Débloquer sans perdre la main.
- Changer de fournisseur ou de modèle sans risque : la validation garantit ce qui
  s'affiche.
- Des pistes de règles disponibles partout, hors ligne.

### Ce que ça coûte

- Certaines demandes répondent « pas de piste valable ».
- Un code de validation à entretenir, par langue et par règle.
- Une interface plus exigeante qu'un bouton « suggérer ».

## Options écartées

| Option | Pourquoi |
|---|---|
| Un assistant toujours actif qui suggère en continu | Parle quand on compose ; pousse vers l'écriture à la place |
| Une recommandation unique | Anti-besoin N6 |
| Engendrer des sections, des mélodies ou des textes entiers | C'est écrire le morceau |
| Afficher les sorties du modèle sans les vérifier | Des pistes non conformes à la doctrine de Sacha s'afficheraient |

## Comment on vérifie

- **I06** : toute piste affichée est validée ; au moins deux, ou l'aveu.
- Critères de [PRO-03](../20-sfd/26-propositions-suggestions-provenance.md#pro-03--demander-des-pistes) et
  [MIC-10](../20-sfd/22-micro-motif-et-phrase.md#mic-10--demander-des-pistes-de-détail).
