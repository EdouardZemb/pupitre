# 12 — Le continuum de composition

Composer, c'est zoomer et dézoomer sans cesse : on écoute l'album, on descend sur un
refrain, on corrige une syllabe, on remonte voir si la transition tient. Ce document
fixe **les échelles** (sur quoi l'on travaille), **les moments** (ce que l'on fait), puis
montre comment Pupitre passe de l'un à l'autre **sans jamais recopier**.

C'est la colonne vertébrale de la conception : la [SFD](20-sfd/README.md) est rangée par
échelle, et chaque fonction dit à quel moment elle sert.

---

## 1. Six échelles dans un fonds

```mermaid
flowchart TB
  subgraph FONDS["Fonds — toute la matière de Sacha"]
    direction TB
    I["É0 · Idées<br/>prises, notes, photos"]
    MO["É1 · Motifs<br/>cellules libres ou placées"]
    subgraph MOR["É4 · Morceau"]
      direction TB
      FO["Forme<br/>suite d'occurrences"]
      subgraph SEC["É3 · Section"]
        BL["Bloc<br/>matériau partagé"]
        OC["Occurrences<br/>paroles · écarts · intensité"]
      end
      PH["É2 · Phrases<br/>plage du bloc + ligne de texte"]
    end
    subgraph ALB["É5 · Album"]
      OR["Sélection · ordres · transitions · arc"]
    end
  end
  I -->|promouvoir| MO
  I -->|promouvoir| MOR
  MO -->|placer| BL
  FO --> OC
  OC -->|joue| BL
  BL --> PH
  OC --> PH
  MOR -->|sélectionner| ALB
```

Le **fonds** n'est pas une échelle de travail : c'est le réservoir. On y cherche, on y
rapproche, et c'est là qu'un album puise ses morceaux. Chaque échelle a son propre jeu de
questions, et Pupitre y répond avec une vue, des propositions et des diagnostics.

| Échelle | Taille typique | Les questions de Sacha | Ce que Pupitre montre | Ce que Pupitre propose | Ce que Pupitre diagnostique |
|---|---|---|---|---|---|
| **Fonds** | tout | « Qu'est-ce que j'ai en mi mineur ? Où ai-je déjà fait IV→iv→I ? Quelles idées dorment ? » | Catalogue, recherche, maturité de chaque morceau | Des rapprochements : « ces deux prises semblent la même idée » | Squelettes partagés, incohérences entre sources |
| **É0 · Idée** | quelques secondes à quelques minutes | « Qu'est-ce que c'était ? Est-ce que je l'ai déjà ? » | Les prises et les photos, le texte, les étiquettes, les liens | Mélodie depuis un fredonnement, tempo, tonalité, texte chanté transcrit | — |
| **É1 · Motif** | 1 à 4 mesures | « Comment s'appelle ce que je joue ? Où ça peut aller ? Que donne-t-il renversé, décalé ? » | La loupe : accords, mélodie, rythme, accroche ; la carte des chemins | Accords suivants, substitutions, transformations, rimes pour l'accroche | Notes d'accord, tensions, frottements |
| **É2 · Phrase** | 2 à 8 mesures | « Ma ligne tombe-t-elle sur la mélodie ? Ça se chante ? » | Les syllabes sous les notes, les accents, les voyelles tenues, la tessiture | Découpe syllabique, variantes de ligne au même compte, rimes et quasi-rimes | Accent sur temps faible, voyelle réduite tenue, densité syllabique, sortie de tessiture |
| **É3 · Section** | 4 à 16 mesures, plusieurs occurrences | « Le refrain décolle-t-il ? Le couplet 2 doit-il varier ? Le pont contraste-t-il ? » | Le bloc et ses occurrences côte à côte, leurs écarts, le rythme harmonique, la cadence, le contour, l'intensité | Variantes de cadence, pistes de contraste et de relance | Cadence, rythme harmonique, contour, répétition à l'identique, ambitus |
| **É4 · Morceau** | 1 à 6 minutes | « Quelle forme ? Où est l'énergie ? Ma démo dit-elle la même chose que l'écrit ? » | La frise proportionnelle et sa courbe d'intensité, l'arrangement, les prises alignées, les variantes | Frontières de sections sur une prise, formes alternatives, gestes d'allongement, modulation | Inventaire factuel, puis symptômes de forme |
| **É5 · Album** | 8 à 15 morceaux, 30 à 60 minutes | « Quel ordre ? Les transitions tiennent-elles ? Trop de ballades au milieu ? » | Le plan d'album : tonalités sur le cercle, tempos, intensités, durées, transitions, faces | Ordres candidats expliqués, relations tonales pour les transitions | Adjacences, grappes, redondances, équilibre — toujours avec leur assiette |

---

## 2. Six moments, et des allers-retours

Les moments ne forment pas une chaîne de production : on y revient sans cesse. Le
chemin principal existe, mais **les flèches de retour sont l'usage normal**.

```mermaid
flowchart LR
  M1(["M1 · Capter"]) --> M2(["M2 · Explorer"])
  M2 --> M3(["M3 · Construire"])
  M3 --> M4(["M4 · Arranger<br/>et maquetter"])
  M4 --> M5(["M5 · Relire"])
  M5 --> M6(["M6 · Assembler"])
  M3 -.->|"blocage : des pistes"| M2
  M4 -.->|"la maquette révèle"| M3
  M5 -.->|"annotations à traiter"| M3
  M6 -.->|"l'album demande"| M3
  M6 -.->|"il manque un morceau"| M1
```

| Moment | Quand | Appareil | Échelles | Ce qu'on y fait |
|---|---|---|---|---|
| **M1 · Capter** | Une idée arrive ; une démo existe | 📱 surtout | Fonds, É0, É1, É4 | Enregistrer, noter, photographier, fredonner ; trier la boîte ; importer, rattacher |
| **M2 · Explorer** | On cherche, on hésite | 📱 💻 | Fonds, É0 à É5 | Essayer des accords, des transformations ; demander des pistes ; écouter pour vérifier |
| **M3 · Construire** | On fixe | 💻 surtout | É0 à É5 | Grille, mélodie, texte, alignement, blocs, occurrences, forme, tonalité, tempo |
| **M4 · Arranger et maquetter** | La chanson tient debout | 💻 📱 | É3, É4 | Qui joue quoi, densité, poussée ; prises de maquette ; brief de génération ; confronter la maquette à l'écrit |
| **M5 · Relire** | On prend du recul | 📖 surtout | Fonds, É1 à É5 | Diagnostics, fiche de travail, annotations, accepter ou écarter, consigner |
| **M6 · Assembler** | Assez de morceaux mûrissent | 💻 📖 | Fonds, É4, É5 | Sélectionner, ordonner, régler les transitions, documenter, exporter |

---

## 3. La matrice échelles × moments

Chaque case porte les fonctions de la SFD qui y servent. C'est la carte de lecture de
la SFD, et la preuve qu'aucune case utile n'est vide.

| | M1 · Capter | M2 · Explorer | M3 · Construire | M4 · Arranger et maquetter | M5 · Relire | M6 · Assembler |
|---|---|---|---|---|---|---|
| **Fonds** | CAP-03 · CAP-07 | CAP-05 · CAP-06 | — | — | PRO-07 | CAP-06 |
| **É0 · Idée** | CAP-01 · CAP-02 · ECO-06 | CAP-05 | CAP-02 | — | — | — |
| **É1 · Motif** | ECO-03 · ECO-06 | MIC-08 · MIC-09 · MIC-10 · MIC-12 · MIC-13 · ECO-01 · ECO-03 | MIC-01 · MIC-02 · MIC-03 · MIC-11 · MIC-12 · MIC-13 | — | MIC-06 | — |
| **É2 · Phrase** | — | MIC-10 · ECO-01 | MIC-01 · MIC-04 · MIC-05 · SEC-03 | — | MIC-06 · MIC-07 | — |
| **É3 · Section** | — | SEC-07 · ECO-01 | SEC-01 · SEC-02 · SEC-03 · SEC-04 | SEC-05 · MOR-06 | SEC-04 · SEC-06 | — |
| **É4 · Morceau** | CAP-04 · MOR-03 | MOR-10 · MOR-11 · MOR-14 | MOR-01 · MOR-02 · MOR-04 | MOR-06 · MOR-07 · MOR-08 | MOR-05 · MOR-09 · MOR-12 · MOR-13 · MOR-14 | SYN-07 |
| **É5 · Album** | — | ALB-08 · ALB-09 | ALB-01 · ALB-02 | — | ALB-04 · ALB-07 | ALB-03 · ALB-05 · ALB-06 · ALB-10 · SYN-07 |

Les modules **transverses** — propositions et provenance (`PRO`), écoute et compagnon
d'instrument (`ECO`), rendus et encre (`REN`), données, appareils et réglages (`SYN`) —
servent toutes les cases. La matrice ne montre que celles de leurs fonctions qui tiennent
à un geste propre à une case : ECO-01, ECO-03, ECO-06, PRO-07 et SYN-07.

---

## 4. Zoomer sans recopier

La promesse du [§ 10](10-vision.md#2-la-proposition--un-seul-matériau-toutes-les-échelles)
se vérifie sur un cas : Sacha change **un** accord dans le bloc du refrain.

```mermaid
sequenceDiagram
  autonumber
  actor S as Sacha
  participant B as Bloc « Refrain »
  participant O as Occurrences R1 · R2 · R3
  participant M as Morceau
  participant A as Album
  S->>B: remplace l'avant-dernier accord : V devient IV
  Note over B: une seule écriture, un message de journal<br/>« Refrain : cadence V–I devenue IV–I »
  B-->>O: R1 et R3 changent
  Note over O: R2 porte un écart sur cette mesure :<br/>il garde son accord, et Pupitre le signale
  O-->>M: frise, fiche de travail, diagnostic de forme : recalculés
  M-->>A: la fin du morceau change
  A-->>S: la transition vers le morceau suivant est réévaluée
```

Rien n'a été recopié : les trois refrains, la fiche, la frise et le plan d'album sont
des **vues** du même bloc ([ADR-001](adr/adr-001-un-seul-materiau.md)). Et le cas de R2
montre la nuance qui rend le modèle utile : une occurrence peut s'écarter de son bloc,
et l'écart reste visible ([ADR-007](adr/adr-007-blocs-et-occurrences.md)).

---

## 5. Les gestes qui font passer d'une échelle à l'autre

```mermaid
flowchart LR
  PR["Prise"] -->|"découper à l'oreille<br/>MOR-03"| OC["Occurrences"]
  ID["Idée"] -->|"promouvoir<br/>CAP-02"| MO["Motif"]
  ID -->|"promouvoir<br/>CAP-02"| MR["Morceau"]
  MO -->|"placer, suivi ou détaché<br/>MIC-11"| BL["Bloc"]
  BL -->|"extraire une plage<br/>MIC-11"| MO
  BL -->|"découper en phrases<br/>SEC-03"| PH["Phrases"]
  MR -->|"sélectionner<br/>ALB-02"| AL["Album"]
  AL -->|"ouvrir à la mesure en cause<br/>ALB-07"| MR
```

| Geste | De → vers | Ce qui se passe | Ce qui ne se passe jamais |
|---|---|---|---|
| **Promouvoir** | Idée → motif ou morceau | L'idée devient la première matière de l'objet ; sa prise devient la première prise | L'idée n'est pas supprimée : elle reste, reliée |
| **Placer** | Motif → bloc | Le matériau du motif est **copié** dans le bloc, en mode **suivi** (si le motif change, Pupitre **propose** le report) ou **détaché** (aucun suivi) ; la provenance « issu du motif » reste | Un placement ne réécrit pas les autres mesures du bloc ; aucun lien vivant entre documents |
| **Extraire** | Plage d'un bloc → motif | Une plage devient un motif libre, réutilisable ailleurs | Le bloc n'est pas modifié |
| **Découper à l'oreille** | Prise → occurrences | Sacha tape les frontières en écoutant ; Pupitre les aligne sur les mesures | Pupitre ne nomme pas les sections à la place de Sacha |
| **Sélectionner** | Morceau → album | Le morceau devient candidat de l'album | Le morceau n'est pas figé : il continue de vivre |
| **Redescendre** | Constat d'album → morceau | Un constat ouvre le morceau à la mesure en cause | Aucun constat ne modifie un morceau |

---

## 6. Cycle de vie : de l'idée à l'album

Une idée, un motif et un morceau sont des objets distincts, reliés par les gestes du § 5 :
promouvoir une idée ou placer un motif **crée** un objet relié, sans que la source change
d'état ni disparaisse. Deux cycles, eux, sont de vrais états d'un morceau : son statut, et
sa place dans un album.

```mermaid
stateDiagram-v2
  direction LR
  state "Statut d'un morceau — décidé par Sacha" as Statut {
    [*] --> Actif
    Actif --> EnPause: mettre en pause
    EnPause --> Actif: reprendre
    Actif --> Abouti: déclarer abouti
    Abouti --> Actif: rouvrir
    Actif --> Abandonne: abandonner
    EnPause --> Abandonne: abandonner
    Abandonne --> Actif: reprendre
  }
  state "Place dans un album" as Place {
    [*] --> HorsAlbum
    HorsAlbum --> Candidat: sélectionner
    Candidat --> Retenu: placer dans l'ordre retenu
    Retenu --> Candidat: retirer de l'ordre
    Candidat --> HorsAlbum: désélectionner
  }
  EnPause: En pause
  Abandonne: Abandonné
  HorsAlbum: Hors album
```

Deux choses distinctes, à ne pas confondre :

- le **statut** d'un morceau (actif, en pause, abouti, abandonné) est une **décision**
  de Sacha, datée ;
- la **maturité** d'un morceau est **engendrée** depuis ce qu'il contient, dimension par
  dimension, et ne se saisit jamais ([ADR-006](adr/adr-006-le-partiel-est-normal.md)).

Exemple de maturité engendrée, telle que la vue du fonds l'affiche (0 absent · 1 esquissé ·
2 complet · 3 confirmé) :

```mermaid
xychart-beta
  title "Maturité d'un morceau, par dimension"
  x-axis ["Forme", "Harmonie", "Mélodie", "Texte", "Arrangement", "Maquette"]
  y-axis "Niveau" 0 --> 3
  bar [3, 2, 1, 2, 0, 1]
```

---

## 7. Trois parcours, de bout en bout

### 7.1 Du fredonnement au refrain — É0 → É3

Une mélodie arrive dans le train ; elle finit en refrain.

```mermaid
sequenceDiagram
  autonumber
  actor S as Sacha
  participant C as 📱 Carnet
  participant N as 🏠 Nœud maison
  participant E as 💻 Établi
  S->>C: fredonne 12 secondes, hors ligne
  C->>C: capte la prise, propose des notes (analyse légère)
  Note over C: CAP-01 · ECO-06<br/>la mélodie est une proposition
  S->>C: écoute, accepte 9 notes sur 11, corrige 2
  S->>C: demande des accords possibles pour la première mesure
  C-->>S: 3 pistes avec leur effet (règles, hors ligne)
  Note over C,N: retour à la maison : synchronisation
  C->>N: la prise et les décisions
  N->>E: tout est sur l'établi
  S->>E: promeut l'idée en motif, puis place le motif dans le bloc « Refrain »
  S->>E: écrit la ligne d'accroche, aligne les syllabes
  E-->>S: diagnostic de phrase : un accent sur temps faible
  S->>E: demande des variantes de ligne au même compte syllabique
  E-->>S: 4 variantes, chacune vérifiée sur les syllabes et les accents
  S->>E: en garde une, la retouche
  Note over E: la ligne porte sa provenance :<br/>suggérée, puis retouchée
```

### 7.2 De la vieille démo à la fiche de travail — Fonds → É4

Une démo de quatre minutes, jamais transcrite, devient un morceau lisible au lutrin.

```mermaid
sequenceDiagram
  autonumber
  actor S as Sacha
  participant E as 💻 Établi
  participant N as 🏠 Nœud maison
  participant C as 📱 Carnet
  participant L as 📖 Lutrin
  S->>E: importe la démo (CAP-03)
  E->>N: demande une analyse lourde
  N-->>E: propositions : tempo, tonalité et 2e hypothèse, mesures, accords, frontières, texte chanté
  Note over E: rien n'est accepté d'office,<br/>chaque proposition porte sa confiance
  S->>C: écoute la démo et tape les frontières de sections (MOR-03)
  C-->>E: frontières alignées sur les mesures
  S->>E: nomme les sections, accepte le tempo, garde les deux tonalités candidates
  S->>E: accepte la grille mesure par mesure, en corrige le tiers
  E-->>S: diagnostic de forme : inventaire, puis constats chiffrés (MOR-09)
  S->>L: ouvre la fiche de travail (REN-01)
  L-->>S: fiche paginée, propositions restantes marquées « ~ »
  S->>L: annote au stylet : « pont trop long ? »
```

### 7.3 De douze morceaux à un album — É4 → É5

```mermaid
sequenceDiagram
  autonumber
  actor S as Sacha
  participant E as 💻 Établi
  participant L as 📖 Lutrin
  S->>E: crée l'album, sélectionne 12 candidats (ALB-01, ALB-02)
  E-->>S: plan d'album : tonalités, tempos, intensités, durées<br/>assiette : 12 morceaux, 9 tonalités décidées
  S->>E: pose un premier ordre à la main (ALB-03)
  E-->>S: constats : deux voisins en mi mineur, trois ballades d'affilée
  S->>E: demande des pistes d'ordre (ALB-08)
  E-->>S: 3 ordres, chacun avec ses critères et ses compromis
  S->>E: compare l'ordre manuel et une piste côte à côte (ALB-09)
  S->>E: retient un ordre hybride, décrit deux transitions (ALB-05)
  S->>E: depuis un constat, ouvre un morceau et le transpose d'un ton
  E-->>S: la transition redevient compatible, le journal le consigne
  S->>L: lit la feuille d'album au calme (ALB-10)
```

---

## 8. La vie d'un morceau, en six semaines

```mermaid
timeline
  title Un morceau, de l'idée à l'album
  Semaine 1 : Fredonnement capté dans le train : Mélodie proposée puis corrigée
  Semaine 2 : Motif placé dans un bloc Refrain : Accroche écrite et alignée
  Semaine 3 : Couplet tiré d'une vieille démo : Forme posée, intensités esquissées
  Semaine 4 : Lecture au lutrin : Pont jugé trop long, variante courte essayée
  Semaine 5 : Arrangement par section : Maquette générée puis confrontée à l'écrit
  Semaine 6 : Sélectionné pour l'album : Transposé d'un ton pour la transition
```

Chaque modification de cette frise est une **opération nommée** d'un journal — du fonds,
du morceau ou de l'album : on peut la relire, la comparer, et revenir avant elle ([MOR-12](20-sfd/24-morceau.md#mor-12--journal-décisions-et-questions-du-morceau)).
