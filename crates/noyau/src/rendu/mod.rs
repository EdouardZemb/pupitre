//! L'écriture des pages : un sous-ensemble de SVG, des glyphes en tracés.

use std::fmt::{self, Display};

pub mod case;
pub mod clavier;
mod fleches;
mod grille;
pub mod manche;
pub mod pages;
pub mod portee;
pub mod sous_ensemble;
pub mod svg;
pub mod table;
pub mod texte;
pub mod verso;

/// L'instrument des pages : il choisit les diagrammes du verso, et l'en-tête le dit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Instrument {
    Guitare,
    Clavier,
    #[default]
    LesDeux,
}

impl Instrument {
    /// « guitare », « clavier » ou « guitare et clavier ».
    pub fn nom(self) -> &'static str {
        match self {
            Instrument::Guitare => "guitare",
            Instrument::Clavier => "clavier",
            Instrument::LesDeux => "guitare et clavier",
        }
    }
}

/// Une page qui ne tient pas : ce qui déborde, et de combien. Jamais un mot coupé, jamais
/// un corps sous 11 pt : la page le dit.
#[derive(Clone, Debug, PartialEq)]
pub struct Debordement {
    pub quoi: String,
}

impl fmt::Display for Debordement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "la page ne tient pas : {}", self.quoi)
    }
}

impl std::error::Error for Debordement {}

/// Une longueur à la française, pour les messages : « 21,6 mm ».
pub(crate) fn en_mm(v: f32) -> String {
    format!("{v:.1} mm").replace('.', ",")
}

/// La page dont on parle, en tête de chaque débordement.
pub(crate) struct Contexte(pub(crate) String);

impl Contexte {
    pub(crate) fn deborde(&self, quoi: impl Display) -> Debordement {
        Debordement {
            quoi: format!("{} : {quoi}", self.0),
        }
    }

    /// Un texte plus large que sa place.
    pub(crate) fn trop_large(&self, texte: &str, largeur: f32, place: f32) -> Debordement {
        self.deborde(format_args!(
            "« {texte} » mesure {}, sa place {}",
            en_mm(largeur),
            en_mm(place)
        ))
    }
}
