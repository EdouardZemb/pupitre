# ADR-024 — L'import d'un fonds existant

| | |
|---|---|
| **Statut** | ✅ acceptée |
| **Date** | 2026-11-09 |
| **Sert** | B02, B04, B10 · principes P2, P4 |
| **S'appuie sur** | [ADR-004](adr-004-proposer-n-est-pas-decider.md), [ADR-006](adr-006-le-partiel-est-normal.md) et [ADR-022](adr-022-documents-cote-natif.md) |

## Contexte

Sacha n'arrive pas les mains vides ([§ 11](../11-persona-et-situations.md#1-fiche)), et le
lot 1 d'INC-1 livre l'import de son fonds
([§ 40 · 9](../40-architecture.md#9-trajectoire-de-construction)).
[CAP-03](../20-sfd/21-capter-et-organiser.md#cap-03--importer-un-fonds-existant) en tenait
déjà l'essentiel : verser tel quel, garder le nom d'origine, dédoublonner par empreinte, ne
rien rattacher d'office, vérifier un import par ses comptes. Il ne disait pas ce que
[Q23](../90-questions-ouvertes.md#q23--comment-verser-un-fonds-existant-) relevait : comment
les stems d'un même morceau restent ensemble, ce que deviennent un MIDI et un texte avant
que les morceaux existent, quelle provenance porte un matériau généré, comment un contenu
garde deux noms, ce que l'établi lit et joue sans le nœud, et ce que pèse un tel fonds.

Un relevé des formats et des volumes d'un fonds réel, fait par la maîtrise d'ouvrage, a
précédé cette décision. Il reste hors du dépôt. Les ordres de grandeur qu'on en retient pour
Sacha :

- un **jeu de stems** compte huit à douze stems, chacun typé par son nom : voix, batterie,
  basse… ;
- en WAV, 48 kHz, 16 bits, stéréo, un stem pèse **environ 11 Mo par minute** : autour de
  400 Mo pour les douze stems d'une version de trois minutes ;
- presque chaque stem a **son fichier MIDI**, au tempo variable ; quelques-uns n'en ont
  pas ;
- un même morceau peut exister en **plusieurs versions**, chacune avec son jeu de stems : un
  original, et des versions qu'un générateur de maquettes externe en a tirées ;
- un tel fonds peut ne porter aucun tableau d'analyses ; des archives y doublent souvent des
  dossiers déjà extraits.

## Décision

1. **Un jeu de stems versé est une seule prise, la prise en stems.** Chaque stem en est une
   captation audio. La prise est nommée par l'empreinte de son **manifeste** : l'ensemble
   des empreintes de ses stems, rangées dans l'ordre de leurs octets, une par ligne, figé en
   captation. Verser deux fois le même jeu de stems donne la même prise. Ce que Pupitre sait
   d'un stem se range **sous sa prise** : ses noms d'origine, son format, sa durée, son
   type, et les MIDI qui l'accompagnent. Un même contenu peut ainsi être stem de deux
   prises, avec une entrée sous chacune ; deux stems identiques d'un même jeu n'en font
   qu'un, avec ses deux noms. La durée d'une prise en stems s'engendre : celle de son stem
   le plus long, `—` dès qu'un stem n'a pas la sienne. Les prises d'une idée restent ses
   versions : regrouper déplace la prise entière, jamais un stem seul. Le type d'un stem est
   une décision de Sacha ; l'import le propose d'après le nom d'origine, sans le décider.
2. **Sacha dit ce qui est un jeu de stems**, dossier par dossier, pendant les déclarations
   de l'import, avant que rien ne soit versé. Pupitre le propose pour un dossier dont au
   moins deux fichiers WAV ont la même durée, et ne l'applique jamais d'office. Sans cette
   déclaration, chaque fichier audio est une prise.
3. **L'origine déclarée.** Sacha peut déclarer, à l'import ou plus tard, l'origine d'un
   dossier versé : de sa main, ou produite par un générateur externe, qu'il nomme en texte
   libre. La déclaration vaut pour ses sous-dossiers, sauf ceux qu'il déclare à part, et
   pour chaque captation qui en est versée : prise, stem, MIDI, texte. Plus tard, elle porte
   sur une prise, un MIDI versé seul, un texte versé, ou sur tous les fichiers versés dont
   un chemin d'origine commence par un dossier donné. Non déclarée, elle vaut `—`. Elle se
   range par empreinte : un même contenu n'en a qu'une, la plus forte de celles qu'il a
   reçues, à quelque rôle que ce soit, générée, puis de la main de Sacha, puis `—` ; générée
   sous deux noms, elle garde les deux. Déclarée générée, elle se garde à vie : elle ne se
   retire pas. Le rapport nomme chaque origine qu'un import change. Pupitre ne la déduit
   jamais d'un nom ni d'une métadonnée. Le contenu qu'on accepte d'un fichier déclaré généré
   compte dans la part d'origine, avec les pistes
   ([PRO-02](../20-sfd/26-propositions-suggestions-provenance.md#pro-02--provenance-et-part-dorigine)) ;
   celui d'un fichier d'origine `—` forme le groupe « versé sans origine déclarée ». Ni l'un
   ni l'autre ne compte jamais de la main de Sacha.
4. **Le MIDI et le texte entrent dès le lot 1, comme captations.** Un fichier MIDI qui porte
   le nom d'un stem du même dossier, à l'extension près, accompagne ce stem ; un autre, le
   **MIDI versé seul**, appartient à une idée, comme une prise. Un texte (brut, Markdown,
   ChordPro), le **texte versé**, appartient à une idée. Ils se regroupent comme une prise
   ([CAP-02](../20-sfd/21-capter-et-organiser.md#cap-02--trier-la-boîte-didées)). L'import
   en lit les faits : pour un MIDI, son format SMF et sa division. Leur contenu ne devient
   une proposition qu'en INC-2, dans un morceau : une mélodie, des lignes. D'ici là,
   l'établi montre un texte tel quel, et un MIDI par ses faits. Rien ne s'en stocke qui
   s'engendre, comme le nombre de notes
   ([§ 30 · 13](../30-modele-de-domaine.md#13-ce-que-le-modèle-interdit-de-stocker)), et son
   tempo nominal n'est jamais cru d'office (CAP-03).
5. **Un fichier versé garde tous ses noms d'origine**, chacun avec son chemin relatif au
   dossier versé, séparé par `/` : l'ensemble ne fait que croître. Verser le même contenu
   sous un autre nom ajoute ce nom. Le premier vu, par date d'import, puis dans l'ordre de
   ses octets, s'affiche ; les autres se lisent à la demande. Le nom d'origine d'une prise
   en stems est celui de son dossier.
6. **Ce que l'établi lit seul.** Sans nœud qui décode, l'établi lit l'en-tête d'un WAV
   (codage, canaux, fréquence, bits, durée) et celui d'un MIDI (format, division), par du
   code natif, sans dépendance nouvelle. Les autres formats audio sont versés avec leurs
   faits à `—` ([ADR-006](adr-006-le-partiel-est-normal.md)) : le nœud les lira quand il
   saura les décoder, et marquera « non décodable » ce qu'il ne sait pas décoder. Un fichier
   se reconnaît audio ou MIDI à sa signature, texte à son extension.
7. **Ce que joue la lecture simple** : ce que la WebView de l'appareil sait jouer. Une prise
   qu'elle ne joue pas montre ses faits et « lecture impossible sur cet appareil », sans
   erreur. Une prise en stems s'y joue stem par stem. Les jouer ensemble, ou en n'en gardant
   que certains, relève de l'écoute de contrôle, qui le dit : ce n'est pas un mixage,
   puisque aucun volume ni effet ne s'y règle par stem, et rien ne s'exporte
   ([ECO-01](../20-sfd/27-ecoute-et-compagnon.md#eco-01--écoute-de-contrôle),
   [Q16](../90-questions-ouvertes.md#q16--quelle-brique-pour-lécoute-de-contrôle-)).
8. **Ce qui n'est ni audio, ni MIDI, ni texte est ignoré** : une archive, un PDF, une image,
   un tableau, les métadonnées d'un générateur. Rien n'en est ouvert, aucune archive n'est
   dépliée, et le rapport le compte parmi les ignorés, avec ses octets. Les analyses
   existantes, que CAP-03 importe en propositions selon un mappage déclaré, entrent en
   INC-2, quand une proposition a un morceau où se lire.
9. **Une idée par prise, par MIDI versé seul et par texte versé** : l'import n'en regroupe
   aucune d'office, même d'un même dossier. Chaque import a un identifiant, un ULID : la
   provenance `importee` des idées qu'il crée porte cet identifiant en `depuis`, et le rôle
   de l'appareil en `par`. Un import interrompu reprend sous le même identifiant. Son
   **rapport** compte les fichiers vus, versés, dédoublonnés, illisibles et ignorés, avec
   leurs octets, et se fige en captation à la fin de l'import ; l'opération nommée qui clôt
   l'import ([§ 30 · 11](../30-modele-de-domaine.md#11-journal-décisions-questions)) porte
   l'identifiant et l'empreinte de ce rapport. Rejouer un import ne verse rien de plus :
   seul s'ajoute son rapport.
10. **Les volumes.** Avant de verser, l'import compare la taille à verser à la place libre,
    et ne commence pas sans elle. La taille visée du fonds compte, parmi ses 6 000 prises,
    100 prises en stems de douze stems, chaque stem avec son MIDI : 2 400 captations de
    plus, environ 40 Go
    ([§ 31 · 2](../31-persistance-et-synchronisation.md#2-les-documents)). Le fonds de
    mesure en porte les métadonnées, sans audio, et son fichier complet reste sous 4,5 Mo.
    L'archive complète annonce sa taille avant de commencer, et ne commence pas sans la
    place libre
    ([§ 31 · 9.1](../31-persistance-et-synchronisation.md#91-sans-nœud-larchive-complète)).

## Conséquences

### Ce que ça permet

- Les stems d'une version restent ensemble sans rien inventer : un regroupement, une
  écoute, une archive les prennent tous. Les versions d'un morceau, original et versions
  générées, restent des prises distinctes que Sacha regroupe en une idée s'il le veut.
- Un matériau généré ne passe jamais pour de la main de Sacha, qu'il soit versé avec ou sans
  déclaration : la paternité que garde l'[ADR-004](adr-004-proposer-n-est-pas-decider.md)
  (D5) tient.
- Rien ne se perd avant INC-2 : un MIDI et un texte sont versés, vus, regroupés et
  sauvegardés, et attendent un morceau.
- Un import se vérifie par ses comptes, ignorés compris, et se rejoue sans rien verser.

### Ce que ça coûte

- Quatre objets de plus au modèle : le stem, le MIDI versé, le texte versé, et le manifeste
  d'une prise en stems.
- Des gestes à l'import, dossier par dossier, pour déclarer un jeu de stems et une origine.
- Tant que le nœud ne décode pas, la durée d'un mémo compressé (M4A, MP3) vaut `—`, et la
  durée de son idée aussi : une requête sur la durée ne les voit pas encore.
- Une archive complète de 40 Go prend du temps et de la place ; elle le dit avant de
  commencer.
- Si le fonds de mesure dépasse 4,5 Mo avec ces métadonnées, le budget du fonds se revoit
  ([§ 31 · 2](../31-persistance-et-synchronisation.md#2-les-documents)).

## Options écartées

| Option | Pourquoi |
|---|---|
| Chaque stem, une prise de l'idée | Elle confond les parties d'un enregistrement et les versions d'une idée, et regrouper déplacerait les stems un à un |
| Un lien « stem de » entre des prises | Un lien ne regroupe rien ([§ 30 · 9.3](../30-modele-de-domaine.md#93-regrouper-et-relier)) |
| Reconnaître un jeu de stems d'office, par les noms ou les durées | L'import ne décide aucune valeur, et deux fichiers de même durée ne font pas toujours un jeu de stems |
| Déduire l'origine générée d'un nom ou d'une métadonnée | PRO-02 ne déduit jamais une provenance |
| Lire les métadonnées d'un générateur (prompt, style, filiation) | Leur format est propre à chaque outil, et [MOR-07](../20-sfd/24-morceau.md#mor-07--maquettes-et-brief-de-génération) refuse toute intégration ; ce qui en vaut la peine passe par un texte exporté |
| Une bibliothèque de décodage dès le lot 1, pour lire la durée des formats compressés | Une couche de plus à la pile ([§ 41](../41-pile-technique.md)), pour une durée que le nœud lira |
| Déplier les archives | Elles doublent souvent un dossier déjà extrait ; on verse le dossier |
| Stocker le nombre de notes d'un MIDI, ou son tempo nominal | Le premier s'engendre de son fichier ; le second n'est jamais cru d'office (CAP-03) |

## Comment on vérifie

Par les critères 7 à 13 de
[CAP-03](../20-sfd/21-capter-et-organiser.md#cap-03--importer-un-fonds-existant), le critère
6 de
[PRO-02](../20-sfd/26-propositions-suggestions-provenance.md#pro-02--provenance-et-part-dorigine),
le critère 8 d'[ECO-01](../20-sfd/27-ecoute-et-compagnon.md#eco-01--écoute-de-contrôle) et
le critère 6 de
[SYN-07](../20-sfd/29-socle-donnees-appareils.md#syn-07--exporter-et-importer), par les
tests 13 et 14 du
[§ 31 · 10](../31-persistance-et-synchronisation.md#10-les-tests-qui-verrouillent-ce-document),
et par la règle « Les faits d'un fichier versé » du noyau
([§ 70 · 1](../70-qualite-et-verification.md#1-la-stratégie)), que la mutation éprouve.
