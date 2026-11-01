# ADR-017 — Le nœud maison en conteneurs

| | |
|---|---|
| **Statut** | 🟡 proposée |
| **Date** | 2026-10-11 |
| **Sert** | B13 · principe P8 · contre le risque R7 |
| **S'appuie sur** | [ADR-021](adr-021-cle-du-noeud-epinglee.md) |

## Contexte

Le nœud fait tourner trois choses de natures très différentes : le service en Rust, un
analyseur Python aux dépendances lourdes et fragiles (bibliothèques d'apprentissage,
versions de Python, codecs), et éventuellement un serveur d'inférence. Il doit être
**reproductible**, **déplaçable** sur une autre machine le jour où celle-ci lâche, et
tourner avec le **moindre privilège**.

## Décision

- **Compose** en mode sans racine (Docker rootless ou Podman) : en mode racine, les ports
  publiés par Docker contournent le pare-feu de l'hôte. Trois services : `pupitre`,
  `analyse`, et `modele-local` facultatif.
- Des **volumes** pour les documents, les captations et le miroir.
- Les tâches planifiées restent sur l'**hôte** : sauvegarde (restic, et une copie hors site
  en ajout seul si elle existe), test de restauration, renouvellement du certificat du
  nœud, sur la même clé ([ADR-021](adr-021-cle-du-noeud-epinglee.md)).
- Des images **épinglées** par leur empreinte.

## Conséquences

### Ce que ça permet

- Une pile d'analyse reproductible, à l'identique d'une machine à l'autre.
- Un nœud qui se reconstruit sur une machine neuve depuis la sauvegarde et un fichier de
  composition.
- Un service compromis reste confiné à son conteneur.

### Ce que ça coûte

- Une couche de plus à apprendre et à déboguer.
- La place disque des images.

## Options écartées

| Option | Pourquoi |
|---|---|
| Des services nus sur l'hôte | Les dépendances Python s'y entremêlent ; le nœud devient difficile à déplacer |
| Un orchestrateur de conteneurs | Disproportionné pour une machine et trois services |

## Comment on vérifie

- La reconstruction d'un nœud neuf, depuis la sauvegarde et le fichier de composition, est
  rejouée et chronométrée avant la mise en service
  ([SYN-06](../20-sfd/29-socle-donnees-appareils.md#syn-06--sauvegarder-et-restaurer)).
- Les images sont référencées par empreinte dans le fichier de composition.
- Un contrôle vérifie que le moteur de conteneurs tourne sans racine, et qu'un port publié
  reste fermé depuis une adresse hors des plages autorisées.
