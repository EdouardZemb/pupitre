# ADR-012 — TypeScript partout où l'on travaille hors ligne, Python pour l'analyse lourde

| | |
|---|---|
| **Statut** | 🔄 remplacée par [ADR-019](adr-019-rust-et-python.md) |
| **Date** | 2026-10-11 |
| **Sert** | B01, B05 · principe P5 · contre le risque R7 |
| **S'appuie sur** | [ADR-001](adr-001-un-seul-materiau.md) et [ADR-002](adr-002-local-first.md) |

## Contexte

Le noyau métier — le modèle, la théorie, les diagnostics, les pistes de règles, la mise en
page — doit tourner **dans le navigateur** de chaque appareil, hors ligne
([ADR-002](adr-002-local-first.md)). Le nœud doit employer **le même** noyau pour écrire le
miroir, produire les PDF et valider les pistes génératives : deux implémentations de la
musique finiraient par diverger, ce que l'[ADR-001](adr-001-un-seul-materiau.md) interdit à
l'échelle du code.

L'analyse audio lourde, elle, vit dans l'écosystème Python, sans équivalent mûr ailleurs.

## Décision

- **TypeScript**, en mode strict, pour l'application, le noyau et le service du nœud.
- Le **noyau** est du TypeScript pur : sans DOM, sans stockage, sans réseau. Il tourne à
  l'identique dans le navigateur et sous Node.js.
- **Python**, seulement dans l'**analyseur**, isolé dans son conteneur. Il communique par une
  file de travaux, et ses résultats sont des captations immuables.

## Conséquences

### Ce que ça permet

- Une seule implémentation de la musique, sur les appareils et sur le nœud.
- Des types partagés de bout en bout.
- L'écosystème Python pour ce qu'il fait de mieux, sans qu'il se répande ailleurs.

### Ce que ça coûte

- Deux langages dans le projet, avec une frontière nette.
- Les bibliothèques de théorie en TypeScript sont moins riches que leurs équivalents Python :
  `tonal` couvre le socle, le reste s'écrit dans le noyau.

## Options écartées

| Option | Pourquoi |
|---|---|
| Python partout, pages rendues côté serveur | Rien ne tourne hors ligne |
| Un noyau en Rust compilé en WebAssembly | Sûr et rapide, mais coûteux à écrire et à faire évoluer seul |
| L'analyse lourde en JavaScript | Les modèles et outils d'analyse audio n'y sont pas mûrs |

## Comment on vérifie

- Une règle d'analyse statique interdit au paquet du noyau toute dépendance à l'interface, au
  stockage ou au réseau.
- La suite du noyau s'exécute deux fois : sous Node.js et dans un navigateur sans tête, avec
  les mêmes résultats.
