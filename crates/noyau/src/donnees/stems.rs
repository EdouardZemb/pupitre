//! Les stems : le type proposé, le MIDI d'un stem, le jeu proposé et la durée (§ 30 · 9.2 ;
//! questions 16, 17, 18 et 37), sans entrée-sortie.

use std::collections::{BTreeMap, BTreeSet};

use super::repli::replier;
use super::signature::extension;

/// Le type d'un stem, une liste fermée (question 18 ; § 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeDeStem {
    Voix,
    Choeurs,
    Batterie,
    Percussions,
    Basse,
    Guitare,
    Clavier,
    Cordes,
    Cuivres,
    Synthe,
    Effets,
    Autre,
}

impl TypeDeStem {
    pub const TOUS: [TypeDeStem; 12] = [
        TypeDeStem::Voix,
        TypeDeStem::Choeurs,
        TypeDeStem::Batterie,
        TypeDeStem::Percussions,
        TypeDeStem::Basse,
        TypeDeStem::Guitare,
        TypeDeStem::Clavier,
        TypeDeStem::Cordes,
        TypeDeStem::Cuivres,
        TypeDeStem::Synthe,
        TypeDeStem::Effets,
        TypeDeStem::Autre,
    ];

    /// La valeur stockée.
    pub fn nom(self) -> &'static str {
        match self {
            TypeDeStem::Voix => "voix",
            TypeDeStem::Choeurs => "choeurs",
            TypeDeStem::Batterie => "batterie",
            TypeDeStem::Percussions => "percussions",
            TypeDeStem::Basse => "basse",
            TypeDeStem::Guitare => "guitare",
            TypeDeStem::Clavier => "clavier",
            TypeDeStem::Cordes => "cordes",
            TypeDeStem::Cuivres => "cuivres",
            TypeDeStem::Synthe => "synthe",
            TypeDeStem::Effets => "effets",
            TypeDeStem::Autre => "autre",
        }
    }

    /// Ce que l'écran écrit.
    pub fn libelle(self) -> &'static str {
        match self {
            TypeDeStem::Choeurs => "chœurs",
            TypeDeStem::Synthe => "synthé",
            autre => autre.nom(),
        }
    }

    pub fn lire(nom: &str) -> Option<TypeDeStem> {
        TypeDeStem::TOUS.into_iter().find(|t| t.nom() == nom)
    }
}

/// Les mots, français ou anglais, repliés, qui proposent un type ; rangés, chacun une fois.
pub const MOTS_DES_TYPES: &[(&str, TypeDeStem)] = &[
    ("backing", TypeDeStem::Choeurs),
    ("bass", TypeDeStem::Basse),
    ("basse", TypeDeStem::Basse),
    ("batterie", TypeDeStem::Batterie),
    ("brass", TypeDeStem::Cuivres),
    ("cello", TypeDeStem::Cordes),
    ("chant", TypeDeStem::Voix),
    ("choeur", TypeDeStem::Choeurs),
    ("choeurs", TypeDeStem::Choeurs),
    ("choir", TypeDeStem::Choeurs),
    ("clavier", TypeDeStem::Clavier),
    ("claviers", TypeDeStem::Clavier),
    ("cordes", TypeDeStem::Cordes),
    ("cuivres", TypeDeStem::Cuivres),
    ("drum", TypeDeStem::Batterie),
    ("drums", TypeDeStem::Batterie),
    ("effects", TypeDeStem::Effets),
    ("effet", TypeDeStem::Effets),
    ("effets", TypeDeStem::Effets),
    ("fx", TypeDeStem::Effets),
    ("guitar", TypeDeStem::Guitare),
    ("guitare", TypeDeStem::Guitare),
    ("guitares", TypeDeStem::Guitare),
    ("guitars", TypeDeStem::Guitare),
    ("keyboard", TypeDeStem::Clavier),
    ("keys", TypeDeStem::Clavier),
    ("organ", TypeDeStem::Clavier),
    ("orgue", TypeDeStem::Clavier),
    ("pad", TypeDeStem::Synthe),
    ("perc", TypeDeStem::Percussions),
    ("percussion", TypeDeStem::Percussions),
    ("percussions", TypeDeStem::Percussions),
    ("piano", TypeDeStem::Clavier),
    ("strings", TypeDeStem::Cordes),
    ("synth", TypeDeStem::Synthe),
    ("synthe", TypeDeStem::Synthe),
    ("synths", TypeDeStem::Synthe),
    ("trompette", TypeDeStem::Cuivres),
    ("trumpet", TypeDeStem::Cuivres),
    ("violin", TypeDeStem::Cordes),
    ("violon", TypeDeStem::Cordes),
    ("vocal", TypeDeStem::Voix),
    ("vocals", TypeDeStem::Voix),
    ("voice", TypeDeStem::Voix),
    ("voix", TypeDeStem::Voix),
    ("vox", TypeDeStem::Voix),
];

/// Le dernier segment d'un chemin, sans son extension.
fn radical(chemin: &str) -> &str {
    let segment = chemin.rsplit('/').next().unwrap_or(chemin);
    match extension(segment) {
        Some(ext) => &segment[..segment.len() - ext.len() - 1],
        None => segment,
    }
}

/// Les mots d'un nom de fichier : son dernier segment, sans son extension, replié, coupé à chaque
/// caractère qui n'est ni lettre ni chiffre, les morceaux vides écartés.
pub fn mots_du_nom(chemin: &str) -> Vec<String> {
    replier(radical(chemin))
        .split(|c: char| !c.is_alphanumeric())
        .filter(|m| !m.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Le type proposé d'un stem (question 18) : celui du premier mot du nom que la table porte ;
/// `Autre` sans nom ou sans mot qui réponde.
pub fn type_propose(premier_nom_vu: Option<&str>) -> TypeDeStem {
    let Some(nom) = premier_nom_vu else {
        return TypeDeStem::Autre;
    };
    mots_du_nom(nom)
        .iter()
        .find_map(|mot| {
            MOTS_DES_TYPES
                .binary_search_by_key(&mot.as_str(), |(m, _)| *m)
                .ok()
                .map(|i| MOTS_DES_TYPES[i].1)
        })
        .unwrap_or(TypeDeStem::Autre)
}

/// Le stem qu'accompagne le MIDI `midi` (question 17) : le premier de `stems` dont le dernier
/// segment, sans son extension, égale celui du MIDI, octet pour octet.
pub fn stem_du_midi<'a>(midi: &str, stems: &[&'a str]) -> Option<&'a str> {
    let voulu = radical(midi);
    stems.iter().copied().find(|stem| radical(stem) == voulu)
}

/// Un WAV tel que l'inspection l'a lu, sans empreinte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavInspecte<'a> {
    pub taille: u64,
    pub debut: &'a [u8],
    pub trames: u64,
    pub frequence: u32,
}

/// Vrai si au moins deux WAV distincts ont le même nombre de trames à la même fréquence
/// (question 16) ; deux WAV sont distincts si leurs tailles, ou les octets lus de leur début,
/// diffèrent (question 37).
pub fn jeu_propose(wav: &[WavInspecte<'_>]) -> bool {
    let mut groupes: BTreeMap<(u64, u32), BTreeSet<(u64, &[u8])>> = BTreeMap::new();
    for w in wav {
        groupes
            .entry((w.trames, w.frequence))
            .or_default()
            .insert((w.taille, w.debut));
    }
    groupes.values().any(|g| g.len() >= 2)
}

/// La durée d'une prise en stems : celle de son stem le plus long ; `None` dès qu'un stem n'a pas
/// la sienne, ou sans stem.
pub fn duree_de_la_prise_en_stems(stems: &[Option<u64>]) -> Option<u64> {
    let durees: Option<Vec<u64>> = stems.iter().copied().collect();
    durees?.into_iter().max()
}
