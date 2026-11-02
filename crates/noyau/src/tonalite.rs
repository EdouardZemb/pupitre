//! Les 24 tonalités d'A17, dans l'ordre du cahier.

use crate::hauteur::{Intervalle, Lettre, NomDeNote};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Majeur,
    Mineur,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tonalite {
    pub tonique: NomDeNote,
    pub mode: Mode,
}

/// Les demi-tons des sept degrés du majeur.
const MAJEUR: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];
/// Les demi-tons des sept degrés du mineur naturel.
const MINEUR: [i8; 7] = [0, 2, 3, 5, 7, 8, 10];

const fn nom(lettre: Lettre, alteration: i8) -> NomDeNote {
    NomDeNote { lettre, alteration }
}

const fn majeure(lettre: Lettre, alteration: i8) -> Tonalite {
    Tonalite {
        tonique: nom(lettre, alteration),
        mode: Mode::Majeur,
    }
}

const fn mineure(lettre: Lettre, alteration: i8) -> Tonalite {
    Tonalite {
        tonique: nom(lettre, alteration),
        mode: Mode::Mineur,
    }
}

impl Tonalite {
    /// Les 24 tonalités, dans l'ordre du cahier : le cycle des quintes, chaque majeure suivie de
    /// sa relative. La relative de fa♯ majeur s'écrit mi♭ mineur (A17), non ré♯ mineur.
    pub fn les_24() -> [Tonalite; 24] {
        use Lettre::*;
        [
            majeure(C, 0),
            mineure(A, 0),
            majeure(G, 0),
            mineure(E, 0),
            majeure(D, 0),
            mineure(B, 0),
            majeure(A, 0),
            mineure(F, 1),
            majeure(E, 0),
            mineure(C, 1),
            majeure(B, 0),
            mineure(G, 1),
            majeure(F, 1),
            mineure(E, -1),
            majeure(D, -1),
            mineure(B, -1),
            majeure(A, -1),
            mineure(F, 0),
            majeure(E, -1),
            mineure(C, 0),
            majeure(B, -1),
            mineure(G, 0),
            majeure(F, 0),
            mineure(D, 0),
        ]
    }

    /// Les sept degrés, du premier au septième ; le mineur est le mineur naturel.
    pub fn gamme(self) -> [Intervalle; 7] {
        let demi_tons = match self.mode {
            Mode::Majeur => MAJEUR,
            Mode::Mineur => MINEUR,
        };
        std::array::from_fn(|k| Intervalle::new(k as i8, demi_tons[k]))
    }

    pub fn note(self, i: Intervalle) -> NomDeNote {
        self.tonique.plus(i)
    }

    /// Le degré de la gamme qui a cette classe de hauteur, s'il y en a un.
    pub fn degre_de_classe(self, classe: u8) -> Option<Intervalle> {
        self.gamme()
            .into_iter()
            .find(|&i| self.note(i).classe() == classe)
    }

    /// `do majeur`, `mi♭ mineur`.
    pub fn nom(self) -> String {
        let mode = match self.mode {
            Mode::Majeur => "majeur",
            Mode::Mineur => "mineur",
        };
        format!("{} {}", self.tonique.solfege(), mode)
    }
}
