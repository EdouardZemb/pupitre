//! Les jours civils et les bornes d'une période : ce qu'un critère de date compare (question 9 ;
//! CAP-06). Rien ici ne lit l'horloge : le décalage local se reçoit, pour un instant.

/// Un jour civil (question 9) : l'année, le mois de 1 à 12, le jour de 1 à 31.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jour {
    pub annee: i32,
    pub mois: u8,
    pub jour: u8,
}

/// Une année bissextile : divisible par 4, sauf les siècles non divisibles par 400.
fn est_bissextile(annee: i32) -> bool {
    (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0
}

/// Le nombre de jours du mois `mois` (de 1 à 12) de l'année `annee`.
fn jours_du_mois(annee: i32, mois: u8) -> u8 {
    match mois {
        2 if est_bissextile(annee) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `chiffres` n'est fait que de chiffres ASCII : sa valeur, sinon `None`.
fn nombre(chiffres: &str) -> Option<u32> {
    if chiffres.bytes().all(|c| c.is_ascii_digit()) {
        chiffres.parse().ok()
    } else {
        None
    }
}

impl Jour {
    /// « AAAA-MM-JJ », la forme d'un champ de date et du document : quatre chiffres, deux, deux ; un
    /// jour qui n'existe pas (le 30 février, un mois 13), ou toute autre forme : `None`.
    pub fn depuis_iso(texte: &str) -> Option<Jour> {
        let octets = texte.as_bytes();
        if octets.len() != 10 || octets[4] != b'-' || octets[7] != b'-' {
            return None;
        }
        let annee = nombre(texte.get(0..4)?)?;
        let mois = nombre(texte.get(5..7)?)?;
        let jour = nombre(texte.get(8..10)?)?;
        let annee = i32::try_from(annee).ok()?;
        let mois = u8::try_from(mois).ok()?;
        let jour = u8::try_from(jour).ok()?;
        if !(1..=12).contains(&mois) || jour < 1 || jour > jours_du_mois(annee, mois) {
            return None;
        }
        Some(Jour { annee, mois, jour })
    }

    /// « AAAA-MM-JJ ».
    pub fn en_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.annee, self.mois, self.jour)
    }

    /// Minuit UTC de ce jour, en secondes depuis 1970 (« days from civil », H. Hinnant).
    pub fn minuit_utc(self) -> i64 {
        let mois = i64::from(self.mois);
        // L'année commence en mars : janvier et février comptent pour l'année d'avant.
        let annee = i64::from(self.annee) - i64::from(mois <= 2);
        let ere = annee.div_euclid(400);
        let annee_de_l_ere = annee - ere * 400;
        // `mp`, le mois depuis mars (0) jusqu'à février (11).
        let mp = (mois + 9) % 12;
        let jour_de_l_annee = (153 * mp + 2) / 5 + i64::from(self.jour) - 1;
        let jour_de_l_ere =
            annee_de_l_ere * 365 + annee_de_l_ere / 4 - annee_de_l_ere / 100 + jour_de_l_annee;
        // 719 468 jours vont du 1er mars de l'an 0 au 1er janvier 1970.
        (ere * 146_097 + jour_de_l_ere - 719_468) * 86_400
    }
}

/// Les bornes d'une date, en secondes : `des` comprise, `avant` exclue ; une borne absente ne borne pas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bornes {
    pub des: Option<i64>,
    pub avant: Option<i64>,
}

impl Bornes {
    /// `heure` est dans les bornes.
    pub fn contient(self, heure: i64) -> bool {
        self.des.is_none_or(|des| heure >= des) && self.avant.is_none_or(|avant| heure < avant)
    }
}

/// Le début du jour civil `jour` à l'heure locale que dit `decalage` (en minutes, pour un instant) :
/// `m` le minuit UTC du jour, `d = m − decalage(m) × 60`, puis `m − decalage(d) × 60`.
pub fn debut_du_jour(jour: Jour, decalage: impl Fn(i64) -> i32) -> i64 {
    debut_depuis_minuit(jour.minuit_utc(), decalage)
}

fn debut_depuis_minuit(minuit: i64, decalage: impl Fn(i64) -> i32) -> i64 {
    let approche = minuit - i64::from(decalage(minuit)) * 60;
    minuit - i64::from(decalage(approche)) * 60
}

/// La période « du … au … », bornes comprises (question 9) : `des`, le début du jour `du` ; `avant`,
/// le début du jour qui suit `au` (son minuit UTC plus 86 400, puis le même calcul).
pub fn bornes_de_la_periode(
    du: Option<Jour>,
    au: Option<Jour>,
    decalage: impl Fn(i64) -> i32,
) -> Bornes {
    Bornes {
        des: du.map(|jour| debut_du_jour(jour, &decalage)),
        avant: au.map(|jour| debut_depuis_minuit(jour.minuit_utc() + 86_400, &decalage)),
    }
}
