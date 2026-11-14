//! Le schéma du fonds, provisoire jusqu'à la `v0.3.0` (§ 31 · 2.3, § 31 · 2.4) : les clés de
//! la racine du document et de ses objets, les messages de la genèse et de la naissance, la
//! genèse elle-même, en octets, et les migrations (`MIGRATIONS`). Le noyau, qui calcule les
//! modifications des opérations nommées, et les adaptateurs `Documents` les trouvent ici ; le
//! noyau ne lit pas ces octets.

use crate::donnees::version::Migration;

/// À la racine du document : la version de schéma, un entier non signé, 1 à la genèse.
pub const VERSION_DE_SCHEMA: &str = "version_de_schema";
/// À la racine du document : l'objet à clés des idées, de l'identifiant d'idée à l'idée.
pub const IDEES: &str = "idees";
/// À la racine du document : l'objet à clés des prises, les clés à plat, de `<empreinte>/<clé>`
/// à sa valeur.
pub const PRISES: &str = "prises";
/// À la racine du document : l'objet à clés des MIDI versés seuls, les clés à plat, de
/// `<empreinte>/<clé>` à sa valeur.
pub const MIDI_VERSES: &str = "midi_verses";
/// À la racine du document : l'objet à clés des textes versés, les clés à plat, de
/// `<empreinte>/<clé>` à sa valeur.
pub const TEXTES_VERSES: &str = "textes_verses";
/// À la racine du document : l'objet à clés des liens, de la clé d'un lien à `true`.
pub const LIENS: &str = "liens";
/// À la racine du document : l'objet à clés des requêtes enregistrées.
pub const REQUETES_ENREGISTREES: &str = "requetes_enregistrees";
/// À la racine du document : l'objet à clés des imports.
pub const IMPORTS: &str = "imports";
/// À la racine du document : l'objet à clés des origines déclarées, rangées par empreinte
/// (§ 30 · 8.1).
pub const ORIGINES_DECLAREES: &str = "origines_declarees";
/// À la racine du document : l'objet à clés des photos.
pub const PHOTOS: &str = "photos";
/// Les conteneurs de la racine du document que crée la genèse (§ 31 · 2.3), dans l'ordre de sa
/// recette, chacun un objet à clés, après la version de schéma.
pub const CONTENEURS_DE_LA_GENESE: [&str; 9] = [
    IDEES,
    PRISES,
    MIDI_VERSES,
    TEXTES_VERSES,
    LIENS,
    REQUETES_ENREGISTREES,
    IMPORTS,
    ORIGINES_DECLAREES,
    PHOTOS,
];
/// Les conteneurs dont les clés nomment une captation, `<empreinte>/<clé>` (§ 31 · 6.1) ; le
/// lot 2 y ajoute les photos, avec le cas du test 4.
pub const CONTENEURS_NOMMANT_UNE_CAPTATION: [&str; 3] = [PRISES, MIDI_VERSES, TEXTES_VERSES];
/// À la racine du document : l'identifiant du document, une chaîne écrite à la naissance.
pub const IDENTIFIANT: &str = "identifiant";
/// À la racine du document : la lignée (§ 31 · 2.5), une chaîne écrite à la naissance.
pub const LIGNEE: &str = "lignee";
/// À la racine du document : les lignées que remplace la lignée (§ 31 · 2.5), que seule la
/// naissance d'un document reconstruit pose, au lot 4. La teneur l'exclut.
pub const LIGNEES_REMPLACEES: &str = "lignees_remplacees";
/// À la racine du document : l'identifiant de propriétaire, une chaîne écrite à la naissance.
pub const PROPRIETAIRE: &str = "proprietaire";
/// D'une idée : son texte, une chaîne.
pub const TEXTE: &str = "texte";
/// D'une idée : l'objet à clés de ses étiquettes, de l'étiquette à `true`.
pub const ETIQUETTES: &str = "etiquettes";
/// D'une idée : l'origine de sa provenance, `captee` ou `importee` (§ 30 · 8.1), une chaîne.
pub const PROVENANCE: &str = "provenance";
/// D'une idée : le champ `par` de sa provenance, le rôle de l'appareil où elle est née,
/// `etabli` ou `carnet`, une chaîne.
pub const PAR: &str = "par";
/// D'une idée : la date de sa naissance, en secondes, un entier. Une captation n'a pas cette clé,
/// ni `par` : sa date et son appareil, des faits que rien ne lit encore sur elle, restent `—`
/// (§ 30 · 9.2).
pub const LE: &str = "le";
/// D'une idée importée : le champ `depuis` de sa provenance, l'identifiant de l'import, un ULID,
/// une chaîne.
pub const DEPUIS: &str = "depuis";
/// D'une idée retirée : la date de son retrait, en secondes, un entier (§ 30 · 9.1).
pub const RETIREE: &str = "retiree";
/// La valeur de `provenance` d'une idée née d'une capture.
pub const CAPTEE: &str = "captee";
/// La valeur de `provenance` d'une idée née d'un fichier versé.
pub const IMPORTEE: &str = "importee";
/// D'une captation, sous `<empreinte>/idee` : l'identifiant de son idée, une chaîne.
pub const IDEE: &str = "idee";
/// D'une captation, sous `<empreinte>/format` : son format, une chaîne.
pub const FORMAT: &str = "format";
/// D'une prise seulement (§ 30 · 9.2), sous `<empreinte>/duree` : sa durée, en millisecondes, un
/// entier non signé.
pub const DUREE: &str = "duree";
/// D'un lien : le type « apparenté à », en tête de sa clé, `apparente:<a>:<b>`, les deux
/// identifiants rangés dans l'ordre de leurs octets (§ 30 · 9.3).
pub const APPARENTE: &str = "apparente";

/// Le message de la genèse.
pub const MESSAGE_DE_LA_GENESE: &str = "Genèse du fonds";
/// Le message de la naissance, la modification qui écrit l'identifiant, la lignée et le
/// propriétaire sur la genèse.
pub const MESSAGE_DE_LA_NAISSANCE: &str = "Fonds créé";
/// L'acteur de la genèse, le même dans chaque installation : 16 octets ASCII.
pub const ACTEUR_DE_LA_GENESE: [u8; 16] = *b"pupitre-genese-1";

/// La genèse du fonds (§ 31 · 2.3) : une modification Automerge, en octets, sur une ligne,
/// telle que le test l'imprime.
#[rustfmt::skip]
pub const GENESE: &[u8] = &[133, 111, 74, 131, 169, 221, 153, 100, 1, 184, 1, 0, 16, 112, 117, 112, 105, 116, 114, 101, 45, 103, 101, 110, 101, 115, 101, 45, 49, 1, 1, 0, 16, 71, 101, 110, 195, 168, 115, 101, 32, 100, 117, 32, 102, 111, 110, 100, 115, 0, 6, 21, 120, 52, 1, 66, 4, 86, 4, 87, 1, 112, 2, 118, 17, 118, 101, 114, 115, 105, 111, 110, 95, 100, 101, 95, 115, 99, 104, 101, 109, 97, 5, 105, 100, 101, 101, 115, 6, 112, 114, 105, 115, 101, 115, 11, 109, 105, 100, 105, 95, 118, 101, 114, 115, 101, 115, 13, 116, 101, 120, 116, 101, 115, 95, 118, 101, 114, 115, 101, 115, 5, 108, 105, 101, 110, 115, 21, 114, 101, 113, 117, 101, 116, 101, 115, 95, 101, 110, 114, 101, 103, 105, 115, 116, 114, 101, 101, 115, 7, 105, 109, 112, 111, 114, 116, 115, 18, 111, 114, 105, 103, 105, 110, 101, 115, 95, 100, 101, 99, 108, 97, 114, 101, 101, 115, 6, 112, 104, 111, 116, 111, 115, 10, 127, 1, 9, 0, 127, 19, 9, 0, 1, 10, 0];

/// Les migrations du fonds (§ 31 · 2.4), dans l'ordre : la n-ième mène à la version de schéma
/// n + 1. Aucune : l'application ne connaît que la version de schéma 1, celle de la genèse.
/// Une migration publiée, portée par une étiquette `vX.Y.Z`, ne change plus.
pub const MIGRATIONS: &[Migration<'static>] = &[];
