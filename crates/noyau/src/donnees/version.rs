//! La version de schéma (§ 31 · 2.4) : ce que permet un document selon la version de schéma qu'il
//! porte et celle que connaît l'application, et les migrations écrites d'avance qui le font passer
//! de l'une à l'autre. L'adaptateur `Documents` lit la version de schéma, applique et écrit les
//! migrations ; ces règles ne voient que les numéros, et la suite des migrations.

use std::cmp::Ordering;

/// Une migration écrite d'avance : la modification, en octets, qui fait passer le fonds à `vers`,
/// et la version de Pupitre qui l'a apportée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Migration<'a> {
    /// La version de schéma où elle mène.
    pub vers: u64,
    /// Sa modification Automerge, en octets, qui ne dépend que de la précédente : la genèse, ou la
    /// migration d'avant.
    pub octets: &'a [u8],
    /// La version de Pupitre qui l'a apportée, que le journal nommera.
    pub version_de_pupitre: &'a str,
}

/// Ce que permet un document, selon sa version de schéma et celle que connaît l'application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lecture {
    /// La version de schéma connue : il se lit et s'écrit.
    Ecrire,
    /// Une version de schéma plus ancienne, `de` : il migre à son ouverture, jusqu'à `vers`, la
    /// version de schéma connue.
    Migrer { de: u64, vers: u64 },
    /// Une version de schéma plus récente : il se lit sans écrire.
    LireSansEcrire,
    /// Une version de schéma absente, nulle, ou qui n'est pas un entier non signé : il se lit,
    /// sans écrire ni échanger.
    Illisible,
}

/// `Some(v)`, 1 ≤ v < connue : `Migrer` ; v == connue : `Ecrire` ; v > connue : `LireSansEcrire` ;
/// `None` ou `Some(0)` : `Illisible`. `version` est celle que porte la racine du document, `None`
/// quand elle est absente ou n'est pas un entier non signé ; `connue`, celle de l'application.
pub fn lecture(version: Option<u64>, connue: u64) -> Lecture {
    match version {
        None | Some(0) => Lecture::Illisible,
        Some(v) => match v.cmp(&connue) {
            Ordering::Less => Lecture::Migrer {
                de: v,
                vers: connue,
            },
            Ordering::Equal => Lecture::Ecrire,
            Ordering::Greater => Lecture::LireSansEcrire,
        },
    }
}

/// La version de schéma que connaît l'application des migrations `migrations` : 1 + le nombre de
/// migrations. Elles se suivent : `vers` = 2, 3…
pub fn connue(migrations: &[Migration<'_>]) -> u64 {
    1 + migrations.len() as u64
}

/// Les migrations au-delà de `de`, dans l'ordre de la suite des migrations : celles qui font
/// passer un document de la version de schéma `de` à la version de schéma connue.
pub fn a_appliquer<'m, 'a>(migrations: &'m [Migration<'a>], de: u64) -> &'m [Migration<'a>] {
    let deja = migrations
        .iter()
        .take_while(|migration| migration.vers <= de)
        .count();
    &migrations[deja..]
}

/// Le message de la migration vers `vers` : « Fonds passé à la version de schéma {vers} ».
pub fn message_de_la_migration(vers: u64) -> String {
    format!("Fonds passé à la version de schéma {vers}")
}
