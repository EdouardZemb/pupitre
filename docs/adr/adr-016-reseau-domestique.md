# ADR-016 — Réseau domestique seulement, HTTPS par certificat de domaine, appairage par code

| | |
|---|---|
| **Statut** | 🔄 remplacée par [ADR-021](adr-021-cle-du-noeud-epinglee.md) |
| **Date** | 2026-10-11 |
| **Sert** | B13 · principe P8 · contre le risque R3 |
| **S'appuie sur** | [ADR-002](adr-002-local-first.md) |

## Contexte

- Au cadrage, la règle a été posée : **aucune exposition publique sur Internet**.
- L'installation de l'application, son cache hors ligne, l'accès au **micro** et le **stockage
  persistant** exigent un *contexte sécurisé*, c'est-à-dire HTTPS ; une adresse du réseau local
  n'en est pas un.
- Sur une tablette à encre, taper un secret long est pénible au point de faire abandonner
  l'outil (N4) ; un avertissement de certificat aussi.
- Un motif de panne connu : quand l'identité est portée par un intermédiaire (un proxy, un
  service d'accès), un chemin qui le contourne — le réseau local, justement — répond sans
  identité.

## Décision

| Sujet | Décision |
|---|---|
| **Exposition** | Le nœud écoute sur le réseau du foyer, et sur la plage d'un réseau privé virtuel s'il existe. Pare-feu de l'hôte en refus par défaut, que des conteneurs sans racine ne contournent pas ([ADR-017](adr-017-noeud-en-conteneurs.md)). Aucune redirection de port, aucun tunnel entrant, aucun relais public |
| **HTTPS** | Un certificat public pour un nom sous un domaine possédé, obtenu par **défi DNS-01** ; **générique sur une sous-zone dédiée**, pour que les journaux publics de transparence ne révèlent aucun nom d'hôte ; nom résolu **localement**. Repli : une autorité locale installée sur les appareils |
| **Appairage** | Un **code à six chiffres**, valable deux minutes, invalidé après cinq essais faux ; un seul appairage ouvert à la fois ; l'appareil qui a émis le code — un appareil déjà appairé, ou la console du nœud pour le premier — nomme le nouvel appareil et confirme son arrivée |
| **Sessions** | Un **jeton par appareil**, nommé, long, **révocable seul** ; « tout révoquer » en dernier recours |
| **Identité** | Vérifiée **par l'application**, sur chaque requête et par chaque chemin : hors la route d'appairage et les ressources statiques de l'application, aucune requête n'aboutit sans jeton ; **aucun en-tête d'identité** venu d'un intermédiaire n'est cru |
| **Opérations destructrices** | Confirmation renforcée sur l'établi ou le carnet ; **absentes du lutrin** |
| **Indépendance** | Pupitre **fonctionne Internet coupé** : résolution locale, certificat déjà émis, aucune dépendance extérieure pour servir ou synchroniser |

## Conséquences

### Ce que ça permet

- Un HTTPS valide, sans avertissement, sans rien exposer.
- Un appareil perdu se révoque seul ; les autres continuent.
- Rien à taper de long, nulle part.

### Ce que ça coûte

- Posséder un domaine et un jeton d'API DNS. Les jetons se cloisonnent souvent **par zone** :
  si la zone sert aussi à autre chose, un jeton sur cette zone pourrait la modifier. Le nom
  `_acme-challenge` de la sous-zone est donc délégué par CNAME à une zone de validation
  dédiée : le jeton ne peut modifier qu'elle. Jeton dédié, stocké hors de tout dépôt.
- Une adresse stable pour le nœud (réservation sur la box).
- Le vol d'un appareil donne sa copie locale à qui sait le déverrouiller : verrouillage
  automatique et chiffrement du système en sont la parade ; la révocation coupe la
  synchronisation, pas la copie déjà présente.

## Options écartées

| Option | Pourquoi |
|---|---|
| Certificat auto-signé | Un avertissement à chaque ouverture : sur une dalle à encre, l'outil est abandonné |
| Exposition publique, même authentifiée | Contraire à la règle posée au cadrage |
| Identité portée par un proxy ou un service d'accès | Le chemin local le contourne |
| Un secret long saisi sur chaque appareil | Anti-besoin N4 |

## Comment on vérifie

- **I22** : hors la route d'appairage et les ressources statiques de l'application, aucune
  requête n'aboutit sans jeton, par aucun chemin ; aucun en-tête d'identité injecté n'est
  cru.
- Un test d'appairage : un code est refusé après deux minutes et après cinq essais faux ;
  un second appairage ne s'ouvre pas tant qu'un premier l'est ; sans la confirmation de
  l'appareil émetteur, aucun jeton n'est délivré.
- **I24** : Pupitre démarre, sert l'application, charge les échantillons (servis par le nœud, jamais par un hôte public), synchronise et appaire avec Internet coupé.
- Un contrôle lit le certificat servi : seuls des noms génériques de la sous-zone y figurent.
- Critères de [SYN-08](../20-sfd/29-socle-donnees-appareils.md#syn-08--appairer-et-révoquer-des-appareils).
