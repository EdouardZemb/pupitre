# ADR-019 — Rust pour tout le code écrit par le projet, Python pour la seule analyse lourde

| | |
|---|---|
| **Statut** | ✅ acceptée — amendée par [ADR-022](adr-022-documents-cote-natif.md) le 2026-11-04 |
| **Date** | 2026-10-27 |
| **Sert** | B01, B05 · principe P5 · contre le risque R7 |
| **S'appuie sur** | [ADR-001](adr-001-un-seul-materiau.md) et [ADR-002](adr-002-local-first.md) |

## Contexte

Elle remplace l'[ADR-012](adr-012-typescript-et-python.md).

Le noyau métier — le modèle, la théorie, les diagnostics, les pistes de règles, la mise en
page — doit tourner sur chaque appareil, hors ligne ([ADR-002](adr-002-local-first.md)). Le
nœud doit employer **le même** noyau pour écrire le miroir, produire les PDF et valider les
pistes génératives : deux implémentations de la musique finiraient par diverger, ce que
l'[ADR-001](adr-001-un-seul-materiau.md) interdit à l'échelle du code.

L'ADR-012 avait choisi son langage parce que le noyau devait tourner dans un navigateur. Les
applications deviennent natives ([ADR-020](adr-020-leptos-et-tauri.md)) : cette contrainte
tombe. Il reste à tenir un seul langage, du noyau à l'écran et au nœud, pour qu'une personne
puisse tout entretenir (R7), et à tester le noyau par mutation, comme la méthode l'exige dès
qu'il y a une logique de domaine
([M-0001](../modele/decisions/M-0001-ecosysteme-des-applications-personnelles.md)).

L'analyse audio lourde, elle, vit dans l'écosystème Python, sans équivalent mûr ailleurs.

## Décision

- **Rust** pour tout le code écrit par le projet : le noyau, l'interface
  ([ADR-020](adr-020-leptos-et-tauri.md)), les applications, le service du nœud, la passerelle
  IA, l'analyse légère et les tests.
- Le **noyau** est une crate Rust pure, sans entrées-sorties : ni interface, ni stockage, ni
  réseau. Il est partagé par l'interface, compilée en WebAssembly, par le nœud et par les
  tests.
- **Quatre exceptions, nommées** :
  1. les garde-fous du modèle, en Python, gérés par `modele-projet` ;
  2. Verovio, en C++, compilé en WebAssembly ;
  3. un **module Android natif en Kotlin**, aussi mince que possible, pour la seule capture
     depuis l'écran verrouillé (service au premier plan de type micro, tuile ou raccourci) :
     Tauri ne donne pas ce point d'entrée ;
  4. l'analyse lourde, en Python, dans son conteneur, derrière un contrat d'entrées-sorties
     ([ADR-014](adr-014-analyse-en-deux-etages.md)).

  Une cinquième exception demande une ADR.

## Conséquences

### Ce que ça permet

- Une seule implémentation de la musique, sur les appareils, au nœud et dans les tests.
- Un seul langage à lire et à entretenir, des types partagés de bout en bout, et un
  compilateur qui refuse beaucoup avant l'exécution.
- Le test de mutation du noyau par `cargo-mutants`.
- L'écosystème Python pour ce qu'il fait de mieux, sans qu'il se répande ailleurs.

### Ce que ça coûte

- Rust s'écrit plus lentement qu'un langage à ramasse-miettes, et s'apprend plus lentement
  seul ; la compilation est plus longue.
- Les bibliothèques de théorie musicale en Rust sont minces : la théorie s'écrit dans le
  noyau (degrés, familles, orthographe, transposition).
- Trois frontières à tenir : le noyau en WebAssembly sous l'interface, le module Kotlin sous
  Tauri, l'analyseur Python derrière son contrat.
- Le module Kotlin, même mince, est un langage de plus à lire.

## Options écartées

| Option | Pourquoi |
|---|---|
| TypeScript partout, la pile de l'[ADR-012](adr-012-typescript-et-python.md) | Elle tenait à un noyau exécuté dans le navigateur, que les applications natives ne demandent plus ; elle laissait la capture depuis l'écran verrouillé et la durée du stockage aux limites du navigateur (H01, R6) |
| Un noyau Rust sous une interface TypeScript | Deux langages pour le code du projet, et une frontière de types à franchir à chaque appel du noyau : le gain d'un seul langage disparaît |

## Comment on vérifie

- La CI `verifier` : `cargo fmt --check`, `cargo clippy -D warnings` et `cargo test`, sur tout
  le code Rust ([§ 70 · 8](../70-qualite-et-verification.md#8-les-portes-de-ci)).
- `cargo-mutants` passe sur le noyau : une mutation qui survit à la suite est un test qui
  manque.
- Le graphe de dépendances de la crate du noyau ne contient aucune crate d'interface, de
  stockage ou de réseau.
