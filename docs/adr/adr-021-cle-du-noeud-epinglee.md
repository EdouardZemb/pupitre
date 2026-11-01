# ADR-021 — Réseau domestique seulement ; TLS par la clé du nœud, épinglée à l'appairage

| | |
|---|---|
| **Statut** | ✅ acceptée |
| **Date** | 2026-10-27 |
| **Sert** | B13 · principe P8 · contre le risque R3 |
| **S'appuie sur** | [ADR-002](adr-002-local-first.md) |

## Contexte

Elle remplace l'[ADR-016](adr-016-reseau-domestique.md), dont elle reprend la règle
d'exposition, inchangée.

- Au cadrage du 2026-10-11, la règle a été posée : **aucune exposition publique sur
  Internet**.
- L'ADR-016 obtenait HTTPS par un certificat public, sur un domaine possédé, parce qu'une
  application web exige un *contexte sécurisé*. Les applications sont désormais natives
  ([ADR-020](adr-020-leptos-et-tauri.md)) : elles peuvent reconnaître le nœud à sa clé, sans
  domaine ni autorité de certification.
- L'ADR-016 écartait le certificat auto-signé pour l'avertissement affiché à chaque
  ouverture. Cet avertissement venait du navigateur : une application qui épingle la clé du
  nœud n'en affiche aucun, et le certificat que le nœud émet sur sa propre clé redevient
  utilisable.
- Un code court ne dit pas, à lui seul, à l'appareil qu'il parle au bon nœud : sur le réseau
  du foyer, un intermédiaire pourrait s'interposer pendant l'appairage.
- Sur une tablette à encre, taper un secret long est pénible au point de faire abandonner
  l'outil (N4).
- Un motif de panne connu : quand l'identité est portée par un intermédiaire (un proxy, un
  service d'accès), un chemin qui le contourne — le réseau local, justement — répond sans
  identité.

## Décision

| Sujet | Décision |
|---|---|
| **Exposition** | Le nœud écoute sur le réseau du foyer, et sur la plage d'un réseau privé virtuel s'il existe. Pare-feu de l'hôte en refus par défaut, que des conteneurs sans racine ne contournent pas ([ADR-017](adr-017-noeud-en-conteneurs.md)). Aucune redirection de port, aucun tunnel entrant, aucun relais public |
| **TLS** | Le nœud a sa propre paire de clés, et sert TLS avec un certificat émis sur cette clé. Chaque appareil épingle la **clé publique** du nœud, pas son certificat : le certificat se renouvelle sans nouvel appairage. Aucun domaine, aucune autorité de certification, aucune racine installée sur les appareils |
| **Appairage** | Un **code à six chiffres**, valable deux minutes, invalidé après cinq essais faux ; un seul appairage ouvert à la fois ; l'appareil qui a émis le code — un appareil déjà appairé, ou la console du nœud pour le premier — nomme le nouvel appareil et confirme son arrivée. Le code sert à un **échange à mot de passe authentifié (PAKE)**, qui authentifie la clé du nœud : un intermédiaire ne peut pas s'interposer, et un code deviné ne sert qu'une fois |
| **Rotation** | Une nouvelle clé est annoncée par le nœud et **signée par l'ancienne** ; les appareils appairés l'épinglent à la place. Si l'ancienne clé est perdue, chaque appareil s'appaire de nouveau |
| **Sessions** | Un **jeton par appareil**, nommé, long, **révocable seul** ; « tout révoquer » en dernier recours |
| **Identité** | Vérifiée **par l'application**, sur chaque requête et par chaque chemin : hors la route d'appairage, aucune requête n'aboutit sans jeton ; **aucun en-tête d'identité** venu d'un intermédiaire n'est cru |
| **Opérations destructrices** | Confirmation renforcée sur l'établi ou le carnet ; **absentes du lutrin** |
| **Indépendance** | Pupitre **fonctionne Internet coupé** : la clé épinglée ne dépend d'aucune autorité extérieure, et rien d'extérieur n'est appelé pour servir, synchroniser ou appairer |

## Conséquences

### Ce que ça permet

- Un trafic chiffré et un nœud authentifié, sans domaine, sans jeton d'API DNS, sans rien
  exposer.
- Un appairage qu'un intermédiaire sur le réseau du foyer ne peut pas détourner.
- Un appareil perdu se révoque seul ; les autres continuent.
- Rien à taper de long, nulle part.

### Ce que ça coûte

- Une adresse stable pour le nœud (réservation sur la box).
- La clé privée du nœud est un secret de plus à garder, hors de tout dépôt ; perdue, elle
  impose un nouvel appairage de chaque appareil.
- Un échange PAKE et une rotation signée à mettre en œuvre, sur des bibliothèques éprouvées.
- La construction navigateur de l'interface, qui ne sait pas épingler une clé, reste un outil
  de test.
- Le vol d'un appareil donne sa copie locale à qui sait le déverrouiller : verrouillage
  automatique et chiffrement du système en sont la parade ; la révocation coupe la
  synchronisation, pas la copie déjà présente.

## Options écartées

| Option | Pourquoi |
|---|---|
| Un certificat de domaine, par défi DNS-01 (la voie de l'[ADR-016](adr-016-reseau-domestique.md)) | Un domaine, un jeton d'API DNS cloisonné et un renouvellement à tenir, pour un contexte sécurisé dont les applications natives n'ont plus besoin |
| L'épinglage du certificat | Chaque renouvellement du certificat exigerait un nouvel appairage de chaque appareil |
| Exposition publique, même authentifiée | Contraire à la règle posée au cadrage |
| Identité portée par un proxy ou un service d'accès | Le chemin local le contourne |
| Un secret long saisi sur chaque appareil | Anti-besoin N4 |

## Comment on vérifie

- **H08** : après une rotation de la clé du nœud, un appareil appairé se reconnecte, et un
  appareil non appairé est refusé.
- Un appareil à qui le nœud présente une autre clé que celle épinglée, sans rotation signée,
  refuse la connexion.
- Un test d'appairage : un code est refusé après deux minutes et après cinq essais faux ; un
  second appairage ne s'ouvre pas tant qu'un premier l'est ; sans la confirmation de
  l'appareil émetteur, aucun jeton n'est délivré ; un intermédiaire qui relaie l'échange sans
  connaître le code n'obtient ni jeton ni clé épinglée.
- **I22** : hors la route d'appairage, aucune requête n'aboutit sans jeton, par aucun
  chemin ; aucun en-tête d'identité injecté n'est cru.
- **I24** : Pupitre démarre, charge les échantillons (servis par le nœud, jamais par un hôte
  public), synchronise et appaire avec Internet coupé.
- Critères de [SYN-08](../20-sfd/29-socle-donnees-appareils.md#syn-08--appairer-et-révoquer-des-appareils).
