//! L'écriture des pages : un sous-ensemble de SVG, des glyphes en tracés.

use std::fmt;

pub mod case;
pub mod pages;
pub mod sous_ensemble;
pub mod svg;
pub mod table;
pub mod texte;

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
