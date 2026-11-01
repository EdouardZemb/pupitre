# ADR-020 — Leptos pour l'interface, Tauri 2 pour les applications

| | |
|---|---|
| **Statut** | ✅ acceptée |
| **Date** | 2026-10-27 |
| **Sert** | B01, B05, B12 · principes P5, P6 |
| **S'appuie sur** | [ADR-019](adr-019-rust-et-python.md) |

## Contexte

Elle remplace l'[ADR-013](adr-013-svelte-pour-l-interface.md).

L'interface de Pupitre est faite d'**éditeurs riches en SVG** (loupe, grille, frise, plan
d'album), qui doivent rester **sobres** sur une dalle à encre : chaque mise à jour de
l'affichage peut provoquer un rafraîchissement visible. Elle doit être légère sur un
téléphone, et entretenable par une personne. Depuis l'[ADR-019](adr-019-rust-et-python.md),
elle s'écrit en Rust.

Trois appareils la portent : un ordinateur sous Windows pour l'établi, un téléphone et une
tablette à encre sous Android pour le carnet et le lutrin. La capture depuis l'écran
verrouillé demande un point d'entrée natif (H01), et la sûreté des captations un stockage
que le système s'interdit d'évincer (R6).

Les tests de bout en bout, eux, ont besoin d'un navigateur, avec les trois profils
d'appareil du [§ 70 · 2](../70-qualite-et-verification.md#2-les-trois-profils-dappareil-en-test).

Comparaison détaillée : [§ 41 · 5.1](../41-pile-technique.md#51-linterface).

## Décision

- **Leptos**, rendu côté client, compilé en WebAssembly : **une seule base d'interface**.
- **Tauri 2** pour les applications. L'interface tourne dans la WebView du système : Windows
  pour l'établi, Android pour le carnet et le lutrin. Les APK sont construits en CI et
  installés à la main sur le téléphone et la tablette à encre.
- L'interface se construit aussi pour le **navigateur**, où les tests de bout en bout gardent
  leurs trois profils.
- L'interface reste **mince** : aucune logique métier dans les composants ; tout passe par
  les services d'application et le noyau ([§ 40 · 3](../40-architecture.md#3-composants-de-lapplication)).

## Conséquences

### Ce que ça permet

- Un seul langage de l'écran au noyau : les types du noyau arrivent dans l'interface sans
  traduction.
- Des mises à jour fines de l'affichage, par signaux : moins de zones redessinées sur l'encre.
- Les éditeurs SVG, la portée de Verovio et les pages de la mise en page maison s'affichent
  dans la WebView, comme dans un navigateur.
- Des applications installées : le module Kotlin y branche la capture depuis l'écran
  verrouillé ([ADR-019](adr-019-rust-et-python.md)), et les données vivent dans le dossier de
  l'application.

### Ce que ça coûte

- Un écosystème bien plus petit que ceux des cadriciels JavaScript, et des versions qui
  bougent encore.
- Tauri 2 sur Android est récent : sa construction en CI est à prouver (H10).
- Deux WebView à tenir, celle de Windows et celle d'Android : les rendus se vérifient sur les
  deux.
- Le poids du WebAssembly au premier affichage, à surveiller sur la tablette à encre.
- Une installation à la main à chaque version, sur le téléphone et la tablette à encre.

## Options écartées

| Option | Pourquoi |
|---|---|
| Dioxus | Une interface en Rust aussi, mais son support d'Android est expérimental |
| Slint | Une interface native, sans web : ni WebView ni construction navigateur pour les tests de bout en bout, et des éditeurs SVG à réécrire dans son propre langage de vues |
| Une application web installée, la voie de l'[ADR-013](adr-013-svelte-pour-l-interface.md) | Aucun point d'entrée depuis l'écran verrouillé (H01) ; un stockage que le navigateur peut évincer (R6) ; un contexte sécurisé qui exige un certificat de domaine ([ADR-016](adr-016-reseau-domestique.md)) |

## Comment on vérifie

- **H10** : la construction Tauri et Leptos est reproductible en CI, pour Windows et pour
  Android.
- Les parcours de bout en bout, trois profils, tournent sur la construction navigateur ;
  ceux du profil lutrin ne relèvent aucune animation (I18).
