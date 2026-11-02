//! La table de la tonalité (MIC-12) : quatre rangées de cases, décrites en degrés une fois par
//! mode, puis nommées dans la tonalité.

use crate::accord::{Accord, Qualite};
use crate::hauteur::Intervalle;
use crate::tonalite::{Mode, Tonalite};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Famille {
    Diatonique,
    DominanteSecondaire,
    Emprunt,
    Sombre,
}

/// Les rangées, de haut en bas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rangee {
    Dessus,
    Centre,
    Sombre,
    Dessous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fonction {
    Repos,
    Eloignement,
    Tension,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CoucheSombre {
    #[default]
    Masquee,
    Affichee,
}

/// Le rang est la colonne (0..7) ; dans la couche sombre, l'ordre (0..n).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub rangee: Rangee,
    pub rang: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    pub position: Position,
    pub famille: Famille,
    pub chiffre: String,
    pub accord: Accord,
    pub cible: Option<Position>,
    pub fonction: Option<Fonction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub tonalite: Tonalite,
    pub couche_sombre: CoucheSombre,
    pub cases: Vec<Case>,
}

/// Un degré du centre : sa fondamentale depuis la tonique, sa qualité, son chiffre, sa fonction.
type DegreDuCentre = (Intervalle, Qualite, &'static str, Fonction);

fn centre(mode: Mode) -> [DegreDuCentre; 7] {
    use Fonction::{Eloignement as E, Repos as R, Tension as T};
    use Qualite::{Diminue as D, Majeur as M, Mineur as m};
    let i = Intervalle::new;
    match mode {
        Mode::Majeur => [
            (i(0, 0), M, "I", R),
            (i(1, 2), m, "ii", E),
            (i(2, 4), m, "iii", R),
            (i(3, 5), M, "IV", E),
            (i(4, 7), M, "V", T),
            (i(5, 9), m, "vi", R),
            (i(6, 11), D, "vii°", T),
        ],
        Mode::Mineur => [
            (i(0, 0), m, "i", R),
            (i(1, 2), D, "ii°", E),
            (i(2, 3), M, "III", R),
            (i(3, 5), m, "iv", E),
            (i(4, 7), m, "v", T),
            (i(5, 8), M, "VI", E),
            (i(6, 10), M, "VII", T),
        ],
    }
}

fn accord(fondamentale: Intervalle, qualite: Qualite, renversement: u8) -> Accord {
    Accord {
        fondamentale,
        qualite,
        renversement,
    }
}

fn position(rangee: Rangee, rang: usize) -> Position {
    Position {
        rangee,
        rang: rang as u8,
    }
}

fn au_centre(rang: usize) -> Position {
    position(Rangee::Centre, rang)
}

/// La case d'une rangée, sans cible ni fonction.
fn nouvelle(pos: Position, famille: Famille, chiffre: String, accord: Accord) -> Case {
    Case {
        position: pos,
        famille,
        chiffre,
        accord,
        cible: None,
        fonction: None,
    }
}

fn dessus(mode: Mode) -> Vec<Case> {
    let quinte = Intervalle::new(4, 7);
    centre(mode)
        .iter()
        .enumerate()
        // Un accord diminué n'a pas de dominante secondaire.
        .filter(|(_, (_, qualite, _, _))| *qualite != Qualite::Diminue)
        // En majeur, la tonique n'a pas de dominante secondaire : elle est déjà le repos.
        .filter(|(rang, _)| mode == Mode::Mineur || (1..=5).contains(rang))
        .map(|(rang, (fond, _, chiffre, _))| Case {
            cible: Some(au_centre(rang)),
            ..nouvelle(
                position(Rangee::Dessus, rang),
                Famille::DominanteSecondaire,
                format!("V7/{chiffre}"),
                accord(fond.plus(quinte).reduit(), Qualite::Septieme, 0),
            )
        })
        .collect()
}

fn au_milieu(mode: Mode) -> Vec<Case> {
    centre(mode)
        .iter()
        .enumerate()
        .map(|(rang, (fond, qualite, chiffre, fonction))| Case {
            fonction: Some(*fonction),
            ..nouvelle(
                au_centre(rang),
                Famille::Diatonique,
                chiffre.to_string(),
                accord(*fond, *qualite, 0),
            )
        })
        .collect()
}

/// Les emprunts : le centre de l'autre mode, un ♭ devant les accords majeurs empruntés au mineur.
fn dessous(mode: Mode) -> Vec<Case> {
    let (autre, bemol) = match mode {
        Mode::Majeur => (Mode::Mineur, "♭"),
        Mode::Mineur => (Mode::Majeur, ""),
    };
    centre(autre)
        .iter()
        .enumerate()
        .map(|(rang, (fond, qualite, chiffre, _))| {
            let signe = if mode == Mode::Majeur && *qualite == Qualite::Majeur {
                bemol
            } else {
                ""
            };
            nouvelle(
                position(Rangee::Dessous, rang),
                Famille::Emprunt,
                format!("{signe}{chiffre}"),
                accord(*fond, *qualite, 0),
            )
        })
        .collect()
}

/// La couche sombre, dans l'ordre du tableau des règles.
fn sombre(mode: Mode) -> Vec<Case> {
    let centre = centre(mode);
    let sensible = Intervalle::new(6, 11);
    // Un vii°7 qui glisse vers le degré `rang` du centre.
    let diminue = |rang: usize| {
        let (fond, _, chiffre, _) = centre[rang];
        (
            format!("vii°7/{chiffre}"),
            accord(fond.plus(sensible).reduit(), Qualite::Diminue7, 0),
            Some(au_centre(rang)),
        )
    };
    let napolitaine = |cible: Position| {
        (
            "♭II6".to_string(),
            accord(Intervalle::new(1, 1), Qualite::Majeur, 1),
            Some(cible),
        )
    };
    let entrees = match mode {
        Mode::Majeur => vec![
            diminue(0),
            diminue(1),
            diminue(2),
            diminue(3),
            diminue(4),
            napolitaine(au_centre(4)),
            diminue(5),
        ],
        Mode::Mineur => vec![
            diminue(0),
            (
                "III+".to_string(),
                accord(Intervalle::new(2, 3), Qualite::Augmente, 0),
                None,
            ),
            diminue(2),
            diminue(3),
            diminue(4),
            napolitaine(position(Rangee::Dessous, 4)),
            diminue(5),
            diminue(6),
        ],
    };
    entrees
        .into_iter()
        .enumerate()
        .map(|(rang, (chiffre, accord, cible))| Case {
            cible,
            ..nouvelle(
                position(Rangee::Sombre, rang),
                Famille::Sombre,
                chiffre,
                accord,
            )
        })
        .collect()
}

pub fn table(t: Tonalite, couche: CoucheSombre) -> Table {
    let mut cases = dessus(t.mode);
    cases.extend(au_milieu(t.mode));
    if couche == CoucheSombre::Affichee {
        cases.extend(sombre(t.mode));
    }
    cases.extend(dessous(t.mode));
    Table {
        tonalite: t,
        couche_sombre: couche,
        cases,
    }
}

impl Table {
    pub fn case(&self, p: Position) -> Option<&Case> {
        self.cases.iter().find(|c| c.position == p)
    }

    /// Les cases d'une rangée, dans l'ordre des rangs.
    pub fn rangee(&self, r: Rangee) -> Vec<&Case> {
        let mut cases: Vec<&Case> = self
            .cases
            .iter()
            .filter(|c| c.position.rangee == r)
            .collect();
        cases.sort_by_key(|c| c.position.rang);
        cases
    }

    pub fn nom(&self, c: &Case) -> String {
        c.accord.nom(self.tonalite)
    }

    /// La ligne d'effet du § 60, mot pour mot.
    ///
    /// # Panics
    ///
    /// Si la cible de la case n'est pas dans la table.
    pub fn ligne_d_effet(&self, c: &Case) -> String {
        let vers = |verbe: &str| {
            let cible = c.cible.and_then(|p| self.case(p));
            let cible = cible.expect("la cible d'une case est dans la table");
            format!("{verbe} {}", self.nom(cible))
        };
        match (c.famille, c.fonction) {
            (Famille::Diatonique, Some(Fonction::Repos)) => "repos".to_string(),
            (Famille::Diatonique, Some(Fonction::Eloignement)) => "éloignement".to_string(),
            (Famille::Diatonique, Some(Fonction::Tension)) => "tension".to_string(),
            (Famille::Diatonique, None) => unreachable!("une case du centre a une fonction"),
            (Famille::DominanteSecondaire, _) => vers("tire vers"),
            (Famille::Emprunt, _) => match self.tonalite.mode {
                Mode::Majeur => "assombrit".to_string(),
                Mode::Mineur => "éclaire".to_string(),
            },
            (Famille::Sombre, _) => match c.accord.qualite {
                Qualite::Diminue7 => vers("glisse vers"),
                Qualite::Augmente => "suspend".to_string(),
                _ => "tension sombre avant la dominante".to_string(),
            },
        }
    }

    pub fn cible_de(&self, p: Position) -> Option<Position> {
        self.case(p).and_then(|c| c.cible)
    }
}
