//! Les dimensions de la page, les rectangles, les dates et les liens.

/// Largeur de la page, en mm (paysage).
pub const LARGEUR: f32 = 209.5;
/// Hauteur de la page, en mm (paysage).
pub const HAUTEUR: f32 = 157.0;

/// Un rectangle, en mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub l: f32,
    pub h: f32,
}

/// Une date du calendrier grégorien : la date du rendu est une entrée (I25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub annee: u16,
    pub mois: u8,
    pub jour: u8,
}

const MOIS: [&str; 12] = [
    "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
    "déc.",
];

fn jours_du_mois(annee: u16, mois: u8) -> u8 {
    match mois {
        2 if annee.is_multiple_of(4)
            && (!annee.is_multiple_of(100) || annee.is_multiple_of(400)) =>
        {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// « 1er nov. 2026 », « 11 oct. 2026 ».
    ///
    /// # Panics
    /// Si le mois n'est pas compris entre 1 et 12 : une `Date` venue de
    /// [`Date::depuis_iso`] est toujours valide.
    pub fn en_francais(&self) -> String {
        let mois = MOIS[usize::from(self.mois) - 1];
        if self.jour == 1 {
            format!("1er {mois} {}", self.annee)
        } else {
            format!("{} {mois} {}", self.jour, self.annee)
        }
    }

    /// Lit `AAAA-MM-JJ` ; refuse toute autre forme et toute date inexistante.
    pub fn depuis_iso(s: &str) -> Option<Date> {
        let mut parties = s.split('-');
        let (a, m, j) = (parties.next()?, parties.next()?, parties.next()?);
        if parties.next().is_some() || a.len() != 4 || m.len() != 2 || j.len() != 2 {
            return None;
        }
        if ![a, m, j]
            .iter()
            .all(|p| p.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let (annee, mois, jour): (u16, u8, u8) =
            (a.parse().ok()?, m.parse().ok()?, j.parse().ok()?);
        if !(1..=12).contains(&mois) || jour == 0 || jour > jours_du_mois(annee, mois) {
            return None;
        }
        Some(Date { annee, mois, jour })
    }
}

/// Une zone cliquable de la page, qui mène à une autre page.
#[derive(Debug, Clone, PartialEq)]
pub struct Lien {
    pub zone: Rect,
    /// Index de la page visée, depuis 0.
    pub vers: usize,
}

/// Une page du cahier : son SVG et ses liens.
#[derive(Debug, Clone, PartialEq)]
pub struct PageDuCahier {
    pub svg: String,
    pub liens: Vec<Lien>,
}
