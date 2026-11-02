//! Les hauteurs : une note est un intervalle orthographié depuis la tonique (ADR-008).
//!
//! Un intervalle compte des degrés et des demi-tons ; un nom de note est une lettre et une
//! altération. Le nom n'est qu'un rendu : en anglais pour les accords, en solfège pour la palette.

/// Les demi-tons depuis do de chaque lettre naturelle.
const NATURELS: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Un intervalle orthographié : un nombre de degrés et un nombre de demi-tons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Intervalle {
    pub degres: i8,
    pub demi_tons: i8,
}

impl Intervalle {
    pub const fn new(degres: i8, demi_tons: i8) -> Self {
        Self { degres, demi_tons }
    }

    pub fn plus(self, autre: Intervalle) -> Intervalle {
        Intervalle::new(self.degres + autre.degres, self.demi_tons + autre.demi_tons)
    }

    pub fn moins(self, autre: Intervalle) -> Intervalle {
        Intervalle::new(self.degres - autre.degres, self.demi_tons - autre.demi_tons)
    }

    /// Les degrés ramenés dans `0..7`, les demi-tons décalés de 12 par octave retirée.
    pub fn reduit(self) -> Intervalle {
        let octaves = self.degres.div_euclid(7);
        Intervalle::new(self.degres.rem_euclid(7), self.demi_tons - 12 * octaves)
    }

    /// L'écart des demi-tons à ceux de l'intervalle naturel du même degré, sur l'intervalle réduit.
    pub fn alteration(self) -> i8 {
        let r = self.reduit();
        r.demi_tons - NATURELS[r.degres as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lettre {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Lettre {
    const TOUTES: [Lettre; 7] = [
        Lettre::C,
        Lettre::D,
        Lettre::E,
        Lettre::F,
        Lettre::G,
        Lettre::A,
        Lettre::B,
    ];

    fn rang(self) -> i8 {
        self as i8
    }

    fn naturel(self) -> i8 {
        NATURELS[self as usize]
    }

    fn anglais(self) -> &'static str {
        ["C", "D", "E", "F", "G", "A", "B"][self as usize]
    }

    fn solfege(self) -> &'static str {
        ["do", "ré", "mi", "fa", "sol", "la", "si"][self as usize]
    }
}

/// Un nom de note : une lettre et une altération (−1 pour ♭, 1 pour ♯, ±2 pour 𝄫 et 𝄪).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NomDeNote {
    pub lettre: Lettre,
    pub alteration: i8,
}

impl NomDeNote {
    /// La note d'arrivée, orthographiée exactement : l'altération n'est jamais réduite modulo 12.
    pub fn plus(self, i: Intervalle) -> NomDeNote {
        let total = self.lettre.rang() + i.degres;
        let octaves = total.div_euclid(7);
        let lettre = Lettre::TOUTES[total.rem_euclid(7) as usize];
        let ecart_naturel = lettre.naturel() + 12 * octaves - self.lettre.naturel();
        NomDeNote {
            lettre,
            alteration: self.alteration + i.demi_tons - ecart_naturel,
        }
    }

    /// L'intervalle ascendant, réduit, de `self` vers `autre`.
    pub fn intervalle_vers(self, autre: NomDeNote) -> Intervalle {
        let degres = (autre.lettre.rang() - self.lettre.rang()).rem_euclid(7);
        let naturels = (autre.lettre.naturel() - self.lettre.naturel()).rem_euclid(12);
        Intervalle::new(degres, naturels + autre.alteration - self.alteration)
    }

    /// La classe de hauteur, do = 0.
    pub fn classe(self) -> u8 {
        (self.lettre.naturel() + self.alteration).rem_euclid(12) as u8
    }

    /// Le même son écrit avec au plus un dièse ou un bémol : si𝄫 devient la, fa𝄪 devient sol.
    pub fn enharmonique_simple(self) -> NomDeNote {
        let mut note = self;
        while note.alteration.abs() > 1 {
            note = note.meme_son(note.alteration.signum());
        }
        note
    }

    /// Le même son, la lettre décalée de `ecart_de_lettre` : ré♭ décalé de −1 donne do♯.
    pub fn meme_son(self, ecart_de_lettre: i8) -> NomDeNote {
        let total = self.lettre.rang() + ecart_de_lettre;
        let octaves = total.div_euclid(7);
        let lettre = Lettre::TOUTES[total.rem_euclid(7) as usize];
        NomDeNote {
            lettre,
            alteration: self.alteration + self.lettre.naturel() - lettre.naturel() - 12 * octaves,
        }
    }

    /// Le nom des accords : `E♭`, `F♯`, `B𝄫`.
    pub fn anglais(self) -> String {
        format!("{}{}", self.lettre.anglais(), signe(self.alteration))
    }

    /// Le nom de la palette : `mi♭`, `fa♯`, `si𝄫`.
    pub fn solfege(self) -> String {
        format!("{}{}", self.lettre.solfege(), signe(self.alteration))
    }
}

/// Le signe d'une altération : des doubles (𝄫, 𝄪), puis au besoin un simple (♭, ♯).
fn signe(alteration: i8) -> String {
    let (double, simple) = if alteration < 0 {
        ('\u{1D12B}', '\u{266D}')
    } else {
        ('\u{1D12A}', '\u{266F}')
    };
    let n = alteration.unsigned_abs() as usize;
    let mut s = double.to_string().repeat(n / 2);
    if n % 2 == 1 {
        s.push(simple);
    }
    s
}
