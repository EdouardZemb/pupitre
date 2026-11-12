//! Le schéma du fonds, provisoire jusqu'à la `v0.3.0` (§ 31 · 2.3, § 31 · 2.4) : les clés de
//! la racine du document et de ses objets, les messages de la genèse et de la naissance, la
//! genèse elle-même, en octets, et la table des migrations. Le noyau, qui calculera les
//! modifications des opérations nommées, et les adaptateurs `Documents` les trouvent ici ; le
//! noyau ne lit pas ces octets.

use crate::donnees::version::Migration;

/// À la racine du document : la version de schéma, un entier non signé, 1 à la genèse.
pub const VERSION_DE_SCHEMA: &str = "version_de_schema";
/// À la racine du document : l'objet à clés des idées, de l'identifiant d'idée à l'idée.
pub const IDEES: &str = "idees";
/// À la racine du document : l'objet à clés des prises, de l'empreinte à la prise.
pub const PRISES: &str = "prises";
/// À la racine du document : l'identifiant du document, une chaîne écrite à la naissance.
pub const IDENTIFIANT: &str = "identifiant";
/// À la racine du document : la lignée (§ 31 · 2.5), une chaîne écrite à la naissance.
pub const LIGNEE: &str = "lignee";
/// À la racine du document : les lignées que remplace la lignée (§ 31 · 2.5), que seule la naissance
/// d'un document reconstruit pose, au lot 4. La teneur l'exclut.
pub const LIGNEES_REMPLACEES: &str = "lignees_remplacees";
/// À la racine du document : l'identifiant de propriétaire, une chaîne écrite à la naissance.
pub const PROPRIETAIRE: &str = "proprietaire";
/// D'une idée : son texte, une chaîne.
pub const TEXTE: &str = "texte";
/// D'une idée : l'objet à clés de ses étiquettes, de l'étiquette à `true`.
pub const ETIQUETTES: &str = "etiquettes";
/// D'une prise : l'identifiant de son idée, une chaîne.
pub const IDEE: &str = "idee";
/// D'une prise : son format, une chaîne.
pub const FORMAT: &str = "format";

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
pub const GENESE: &[u8] = &[133, 111, 74, 131, 232, 226, 103, 51, 1, 96, 0, 16, 112, 117, 112, 105, 116, 114, 101, 45, 103, 101, 110, 101, 115, 101, 45, 49, 1, 1, 0, 16, 71, 101, 110, 195, 168, 115, 101, 32, 100, 117, 32, 102, 111, 110, 100, 115, 0, 6, 21, 32, 52, 1, 66, 4, 86, 4, 87, 1, 112, 2, 125, 17, 118, 101, 114, 115, 105, 111, 110, 95, 100, 101, 95, 115, 99, 104, 101, 109, 97, 5, 105, 100, 101, 101, 115, 6, 112, 114, 105, 115, 101, 115, 3, 127, 1, 2, 0, 127, 19, 2, 0, 1, 3, 0];

/// La table des migrations du fonds (§ 31 · 2.4), dans l'ordre : la n-ième mène à la version de
/// schéma n + 1. Vide : l'application ne connaît que la version de schéma 1, celle de la genèse.
/// Une migration publiée, portée par une étiquette, ne change plus.
pub const MIGRATIONS: &[Migration<'static>] = &[];
