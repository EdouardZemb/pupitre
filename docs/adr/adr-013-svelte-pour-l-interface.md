# ADR-013 — Svelte 5 pour l'interface

| | |
|---|---|
| **Statut** | 🔄 remplacée par [ADR-020](adr-020-leptos-et-tauri.md) |
| **Date** | 2026-10-11 |
| **Sert** | B05, B12 · principe P6 |
| **S'appuie sur** | [ADR-012](adr-012-typescript-et-python.md) |

## Contexte

L'interface de Pupitre est faite d'**éditeurs riches en SVG** (loupe, grille, frise, plan
d'album), qui doivent rester **sobres** sur une dalle à encre : chaque mise à jour du DOM
peut provoquer un rafraîchissement visible. Elle doit être légère sur un téléphone, et
entretenable par une personne. Les documents sont des documents Automerge, qui ont des
liaisons officielles pour certains cadriciels seulement.

Comparaison détaillée : [§ 41 · 5.1](../41-pile-technique.md#51-linterface).

## Décision

- **Svelte 5**, avec **SvelteKit** en mode application statique et **Vite**.
- L'interface reste **mince** : aucune logique métier dans les composants ; tout passe par
  les services d'application et le noyau ([§ 40 · 3](../40-architecture.md#3-composants-de-lapplication)).
- Démarrer sur la ligne de version que l'écosystème suit au début de l'implémentation
  (SvelteKit 3 est sorti le 2026-10-01 ; sa version 3.0.1, le 2026-10-06).

## Conséquences

### Ce que ça permet

- Des mises à jour fines du DOM : moins de zones redessinées sur l'encre.
- Des paquets légers, et des gabarits SVG naturels à écrire.
- La liaison officielle d'Automerge pour Svelte.

### Ce que ça coûte

- Un écosystème plus petit que celui de React.
- Une version majeure toute récente de SvelteKit, d'où l'épinglage prudent.

## Options écartées

| Option | Pourquoi |
|---|---|
| React 19 | Plus lourd à l'exécution, réconciliation moins fine ; **alternative crédible** si l'habitude ou l'écosystème priment |
| SolidJS | Mises à jour aussi fines et liaison Automerge officielle (`automerge-repo-solid-primitives`) ; écosystème plus petit, et une réactivité plus délicate à manier seul (des props qu'il ne faut pas déstructurer) |
| Lit | Standards du web ; ergonomie moindre pour un état applicatif riche |
| Pages serveur + HTMX | Rien ne tourne hors ligne |

## Comment on vérifie

- Un **budget** de JavaScript initial — 300 Ko compressés, hors Verovio et échantillons
  chargés à la demande, et hors WASM d'Automerge (≈ 1,1 Mo en gzip), suivi par un budget
  à part — vérifié à chaque construction.
- Les parcours de bout en bout du profil lutrin ne relèvent aucune animation (I18).
