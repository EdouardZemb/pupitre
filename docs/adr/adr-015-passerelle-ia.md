# ADR-015 — Une passerelle IA unique, locale par défaut, distante sur consentement

| | |
|---|---|
| **Statut** | 🟡 proposée |
| **Date** | 2026-10-11 |
| **Sert** | B09, B13 · principes P3, P8 |
| **S'appuie sur** | [ADR-005](adr-005-des-pistes-pas-des-reponses.md) |

## Contexte

Certaines pistes demandent un modèle de langage : des variantes d'une ligne sous
contraintes, des idées de titres, l'explication d'un constat en mots simples. Or :

- les paroles sont **personnelles** ; les envoyer à un tiers est une décision ;
- Pupitre travaille **hors ligne d'abord** ;
- un modèle **se trompe** : compte syllabique faux, rime refusée par la doctrine ;
- un fournisseur distant **coûte**, et ses conditions de conservation des données varient.

## Décision

**Toute demande générative passe par une passerelle unique, sur le nœud.**

| Exigence | Mise en œuvre |
|---|---|
| Tâches typées | Gabarit d'invite versionné, entrée minimale, **schéma de sortie structuré** ; pas de conversation libre |
| Fournisseur | **Local** par défaut si le nœud le permet ; **distant** sur choix explicite ; ou **aucun** |
| Fournisseur distant | L'API Claude, modèle `claude-opus-5-5` par défaut, le meilleur compromis capacité/prix de la gamme courante (réglable) ; sortie structurée, effort fixé explicitement par tâche, gestion explicite de l'arrêt `refusal` |
| Consentement | Par demande par défaut, avec l'**aperçu exact** de l'envoi ; ou par morceau, révocable. Ne part que ce dont l'empreinte a reçu le consentement |
| Traçabilité | **Journal des envois** : date, tâche, fournisseur, modèle, taille, empreinte |
| Sûreté | **Validation déterministe** par le noyau ; variantes et titres : au moins deux pistes valides, ou aucune ([ADR-005](adr-005-des-pistes-pas-des-reponses.md)) ; explication d'un constat : une seule, validée sur sa forme (elle ne cite que des constats et des valeurs existants) |
| Sobriété | Cache des demandes identiques ; cumul des coûts affiché |
| Robustesse | Refus, délai dépassé ou sortie mal formée : « pas de piste », jamais une erreur bloquante |
| Limite | **Aucune captation audio** ne quitte le nœud par la passerelle |

Ordre de grandeur et hypothèses de coût : [§ 40 · 6](../40-architecture.md#6-la-passerelle-ia).

## Conséquences

### Ce que ça permet

- La confidentialité par défaut ; le choix éclairé quand on va plus loin.
- Changer de fournisseur ou de modèle sans toucher à l'application.
- Des sorties sûres à afficher, quel que soit le modèle.

### Ce que ça coûte

- Le code de la passerelle et des gabarits, à entretenir.
- Une qualité locale incertaine (🔶 H06), mesurée au banc des pistes.
- Pas de pistes génératives hors du foyer, sauf réseau privé.

## Options écartées

| Option | Pourquoi |
|---|---|
| Appeler un fournisseur directement depuis les appareils | Des clés sur chaque appareil ; ni consentement central, ni journal, ni validation commune |
| Fournisseur distant par défaut | Les paroles partiraient sans décision explicite |
| Un modèle embarqué sur le téléphone | Qualité insuffisante aujourd'hui pour ces tâches ; à réévaluer |

## Comment on vérifie

- **I23** : sans consentement, aucun appel sortant ; avec, une ligne de journal par appel.
- Critères de [PRO-04](../20-sfd/26-propositions-suggestions-provenance.md#pro-04--passerelle-ia-et-consentement).
- Le banc des pistes du [§ 70 · 6](../70-qualite-et-verification.md#6-le-banc-des-pistes-génératives).
