//! Les accords : une fondamentale, une qualité, un renversement ; leurs notes et leur nom.
//!
//! Un accord se calcule en intervalles depuis la tonique, puis se nomme dans la tonalité. Un nom
//! qui porterait une double altération s'écrit à l'enharmonique simple, sous le même chiffre (Q21).

use crate::hauteur::{Intervalle, NomDeNote};
use crate::tonalite::Tonalite;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Qualite {
    Majeur,
    Mineur,
    Diminue,
    Augmente,
    Septieme,
    Diminue7,
}

const fn i(degres: i8, demi_tons: i8) -> Intervalle {
    Intervalle::new(degres, demi_tons)
}

const MAJEUR: [Intervalle; 3] = [i(0, 0), i(2, 4), i(4, 7)];
const MINEUR: [Intervalle; 3] = [i(0, 0), i(2, 3), i(4, 7)];
const DIMINUE: [Intervalle; 3] = [i(0, 0), i(2, 3), i(4, 6)];
const AUGMENTE: [Intervalle; 3] = [i(0, 0), i(2, 4), i(4, 8)];
const SEPTIEME: [Intervalle; 4] = [i(0, 0), i(2, 4), i(4, 7), i(6, 10)];
const DIMINUE7: [Intervalle; 4] = [i(0, 0), i(2, 3), i(4, 6), i(6, 9)];

impl Qualite {
    /// Les intervalles depuis la fondamentale : fondamentale, tierce, quinte, puis septième.
    pub fn intervalles(self) -> &'static [Intervalle] {
        match self {
            Qualite::Majeur => &MAJEUR,
            Qualite::Mineur => &MINEUR,
            Qualite::Diminue => &DIMINUE,
            Qualite::Augmente => &AUGMENTE,
            Qualite::Septieme => &SEPTIEME,
            Qualite::Diminue7 => &DIMINUE7,
        }
    }

    pub fn tierce_majeure(self) -> bool {
        matches!(
            self,
            Qualite::Majeur | Qualite::Augmente | Qualite::Septieme
        )
    }

    pub fn trois_sons(self) -> bool {
        self.intervalles().len() == 3
    }

    /// Le suffixe du nom : `""`, `m`, `°`, `+`, `7`, `°7`.
    pub fn suffixe(self) -> &'static str {
        match self {
            Qualite::Majeur => "",
            Qualite::Mineur => "m",
            Qualite::Diminue => "°",
            Qualite::Augmente => "+",
            Qualite::Septieme => "7",
            Qualite::Diminue7 => "°7",
        }
    }
}

/// Un accord. La fondamentale se compte depuis la tonique ; au renversement 1, la tierce est à la
/// basse.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Accord {
    pub fondamentale: Intervalle,
    pub qualite: Qualite,
    pub renversement: u8,
}

impl Accord {
    /// Les notes depuis la tonique, la fondamentale d'abord.
    pub fn notes(&self) -> Vec<Intervalle> {
        self.qualite
            .intervalles()
            .iter()
            .map(|&iv| self.fondamentale.plus(iv).reduit())
            .collect()
    }

    pub fn basse(&self) -> Intervalle {
        self.notes()[self.renversement as usize]
    }

    /// Le nom de la fondamentale avant toute réécriture : si𝄫 pour le ♭VI de ré♭ majeur.
    pub fn fondamentale_juste(&self, t: Tonalite) -> NomDeNote {
        t.note(self.fondamentale)
    }

    /// 0 si la fondamentale juste porte au plus une altération ; sinon l'écart de lettre vers son
    /// enharmonique simple (−1 pour si𝄫 → la, +1 pour fa𝄪 → sol), comme dans `meme_son`.
    pub fn ecart_d_ecriture(&self, t: Tonalite) -> i8 {
        let juste = self.fondamentale_juste(t);
        if juste.alteration.abs() <= 1 {
            return 0;
        }
        let ecart = juste.enharmonique_simple().lettre as i8 - juste.lettre as i8;
        (ecart + 3).rem_euclid(7) - 3
    }

    /// Les notes écrites comme le nom : décalées du même écart de lettre que la fondamentale, puis
    /// ramenées à l'enharmonique simple si une double altération reste.
    pub fn noms_des_notes(&self, t: Tonalite) -> Vec<NomDeNote> {
        let ecart = self.ecart_d_ecriture(t);
        self.notes()
            .into_iter()
            .map(|n| {
                let note = t.note(n).meme_son(ecart);
                if note.alteration.abs() > 1 {
                    note.enharmonique_simple()
                } else {
                    note
                }
            })
            .collect()
    }

    /// `A7`, `D♭/F`, `A` pour si𝄫, `G°7` pour fa𝄪°7.
    pub fn nom(&self, t: Tonalite) -> String {
        let noms = self.noms_des_notes(t);
        let mut nom = format!("{}{}", noms[0].anglais(), self.qualite.suffixe());
        if self.renversement > 0 {
            nom.push('/');
            nom.push_str(&noms[self.renversement as usize].anglais());
        }
        nom
    }
}
