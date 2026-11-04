//! Le schéma du fonds, provisoire jusqu'à la `v0.3.0` (§ 31 · 2.3, § 31 · 2.4) : les clés de
//! la racine du document et de ses objets, les messages de la genèse et de la naissance, et la
//! genèse elle-même, en octets. Le noyau, qui calculera les modifications des opérations
//! nommées, et les adaptateurs `Documents` les trouvent ici ; le noyau ne lit pas ces octets.

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
pub const GENESE: &[u8] = &[];
