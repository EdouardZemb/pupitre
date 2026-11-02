//! Les doigtés de guitare (REN-04) : les formes du recueil, placées sur le manche.
//!
//! Les candidats d'un accord sont les formes placées qui tiennent ses contraintes. Hors contexte,
//! un départage en cinq critères les range ; dans une suite, le plus court chemin entre les zones
//! des doigtés l'emporte. L'accordage est `EADGBE`, sans réglage ni capodastre.

use std::cmp::Ordering;

use crate::accord::Accord;
use crate::tonalite::Tonalite;

/// Les cordes à vide, de la grave à l'aiguë, en MIDI (do 3 = 60).
pub const ACCORDAGE: [u8; 6] = [40, 45, 50, 55, 59, 64];

/// Le texte d'un accord sans candidat, suivi de ses notes.
pub const PAS_DE_DOIGTE: &str = "pas de doigté jouable";

/// La frette la plus haute où une forme mobile se place.
const FRETTE_MAX: u8 = 15;
/// L'écart le plus grand entre la frette appuyée la plus basse et la plus haute.
const ETENDUE_MAX: u8 = 3;
/// Les doigts de la main gauche, l'index compris.
const DOIGTS_MAX: usize = 4;

/// Une corde : étouffée, à vide, ou appuyée à une frette (1 et plus).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Corde {
    Etouffee,
    AVide,
    Frette(u8),
}

impl Corde {
    fn de_frette(frette: Option<u8>) -> Corde {
        match frette {
            None => Corde::Etouffee,
            Some(0) => Corde::AVide,
            Some(n) => Corde::Frette(n),
        }
    }

    /// La frette jouée, 0 à vide ; rien pour une corde étouffée.
    fn jouee(&self) -> Option<u8> {
        match self {
            Corde::Etouffee => None,
            Corde::AVide => Some(0),
            Corde::Frette(n) => Some(*n),
        }
    }

    /// La frette appuyée ; rien pour une corde à vide ou étouffée.
    fn appuyee(&self) -> Option<u8> {
        match self {
            Corde::Frette(n) => Some(*n),
            _ => None,
        }
    }
}

/// Le barré : l'index couvre une frette, de la corde `de` à la corde `a` (grave = 0).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Barre {
    pub frette: u8,
    pub de: usize,
    pub a: usize,
}

/// Un doigté : six cordes, de la grave à l'aiguë, le barré éventuel et le numéro du doigt de
/// chaque note appuyée (1 l'index, 4 l'auriculaire).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Doigte {
    pub cordes: [Corde; 6],
    pub barre: Option<Barre>,
    pub doigts: [Option<u8>; 6],
}

/// Une corde d'une notation : `x`, ou une frette en chiffres.
fn lire_corde(morceau: &str) -> Option<Option<u8>> {
    if morceau == "x" {
        return Some(None);
    }
    if !morceau.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    morceau.parse().ok().map(Some)
}

/// Le barré, s'il est possible : à la frette appuyée la plus basse, de la corde la plus grave
/// qui y est appuyée à la plus aiguë. Les deux bornes sont distinctes, et toute corde entre elles
/// est appuyée (elle l'est alors à cette frette ou plus haut).
fn barre_de(cordes: &[Corde; 6], appuyees: &[(usize, u8)]) -> Option<Barre> {
    let frette = appuyees.iter().map(|&(_, f)| f).min()?;
    let mut a_la_frette = appuyees
        .iter()
        .filter(|&&(_, f)| f == frette)
        .map(|&(k, _)| k);
    let de = a_la_frette.next()?;
    let a = a_la_frette.next_back()?;
    cordes[de..=a]
        .iter()
        .all(|c| matches!(c, Corde::Frette(_)))
        .then_some(Barre { frette, de, a })
}

impl Doigte {
    /// Lit `x32010` (un caractère par corde) ou `x-8-10-10-10-8` (des tirets), puis calcule le
    /// barré et les doigts. Rien pour une notation illisible ou un doigté irréalisable.
    pub fn depuis(notation: &str) -> Option<Doigte> {
        let morceaux: Vec<&str> = if notation.contains('-') {
            notation.split('-').collect()
        } else {
            notation.split_inclusive(|_: char| true).collect()
        };
        let frettes: Option<Vec<Option<u8>>> = morceaux.into_iter().map(lire_corde).collect();
        let frettes: [Option<u8>; 6] = frettes?.try_into().ok()?;
        Doigte::des_cordes(frettes.map(Corde::de_frette))
    }

    /// Le doigté de ces cordes. Le barré ne s'emploie que si plus de quatre cordes sont
    /// appuyées ; rien s'il faudrait alors plus de quatre doigts.
    fn des_cordes(cordes: [Corde; 6]) -> Option<Doigte> {
        let appuyees: Vec<(usize, u8)> = cordes
            .iter()
            .enumerate()
            .filter_map(|(k, c)| c.appuyee().map(|f| (k, f)))
            .collect();
        let barre = if appuyees.len() > DOIGTS_MAX {
            Some(barre_de(&cordes, &appuyees)?)
        } else {
            None
        };
        let mut doigts = [None; 6];
        // Les notes hors du barré, par frette croissante, puis de la corde grave à l'aiguë.
        let mut autres: Vec<(u8, usize)> = Vec::new();
        for &(k, f) in &appuyees {
            match &barre {
                Some(b) if f == b.frette => doigts[k] = Some(1),
                _ => autres.push((f, k)),
            }
        }
        autres.sort();
        if usize::from(barre.is_some()) + autres.len() > DOIGTS_MAX {
            return None;
        }
        let premier: u8 = if barre.is_some() { 2 } else { 1 };
        for (n, &(_, k)) in (0u8..).zip(&autres) {
            doigts[k] = Some(premier + n);
        }
        Some(Doigte {
            cordes,
            barre,
            doigts,
        })
    }

    /// Un caractère par corde si toutes les frettes sont sous 10, sinon des tirets.
    pub fn notation(&self) -> String {
        let morceaux: Vec<String> = self
            .cordes
            .iter()
            .map(|c| c.jouee().map_or("x".to_string(), |f| f.to_string()))
            .collect();
        let courte = morceaux.iter().all(|m| m.len() == 1);
        morceaux.join(if courte { "" } else { "-" })
    }

    /// La zone, pour le calcul en contexte : 0 si une corde à vide sonne, sinon la frette
    /// appuyée la plus basse.
    pub fn zone(&self) -> u8 {
        if self.cordes.contains(&Corde::AVide) {
            0
        } else {
            self.frette_la_plus_basse()
        }
    }

    /// Un doigt pour le barré, plus un par autre note appuyée.
    pub fn nombre_de_doigts(&self) -> u8 {
        self.doigts.iter().flatten().copied().max().unwrap_or(0)
    }

    /// La frette appuyée la plus basse, 0 si aucune corde n'est appuyée.
    fn frette_la_plus_basse(&self) -> u8 {
        self.cordes
            .iter()
            .filter_map(Corde::appuyee)
            .min()
            .unwrap_or(0)
    }

    fn a_vide(&self) -> usize {
        self.cordes.iter().filter(|c| **c == Corde::AVide).count()
    }

    fn jouees(&self) -> usize {
        self.cordes.iter().filter_map(Corde::jouee).count()
    }

    /// Les classes qui sonnent, de la corde grave à l'aiguë.
    fn sons(&self) -> Vec<u8> {
        self.cordes
            .iter()
            .zip(ACCORDAGE)
            .filter_map(|(c, a)| c.jouee().map(|f| (a + f) % 12))
            .collect()
    }
}

/// Les notes que doit faire sonner un doigté : la basse, toutes les classes, dont la quinte qui
/// peut manquer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NotesVoulues {
    pub basse: u8,
    pub classes: Vec<u8>,
    pub facultative: Option<u8>,
}

impl NotesVoulues {
    /// Les classes absolues des notes de l'accord, la fondamentale d'abord. La quinte est
    /// facultative, sauf à la basse.
    pub fn de(accord: &Accord, t: Tonalite) -> NotesVoulues {
        let classes: Vec<u8> = accord
            .notes()
            .into_iter()
            .map(|n| t.note(n).classe())
            .collect();
        let basse = t.note(accord.basse()).classe();
        let facultative = Some(classes[2]).filter(|&q| q != basse);
        NotesVoulues {
            basse,
            classes,
            facultative,
        }
    }
}

/// Une forme du recueil : ses frettes en position ouverte, de la corde grave à l'aiguë (rien :
/// étouffée). Une forme mobile se déplace sur toutes ses cordes ; une forme fixe ne sert qu'à sa
/// place.
#[derive(Debug)]
pub struct Forme {
    pub nom: &'static str,
    pub frettes: [Option<u8>; 6],
    pub mobile: bool,
}

const fn corde_lue(b: u8) -> Option<u8> {
    if b == b'x' { None } else { Some(b - b'0') }
}

const fn forme(nom: &'static str, notation: &str, mobile: bool) -> Forme {
    let b = notation.as_bytes();
    Forme {
        nom,
        frettes: [
            corde_lue(b[0]),
            corde_lue(b[1]),
            corde_lue(b[2]),
            corde_lue(b[3]),
            corde_lue(b[4]),
            corde_lue(b[5]),
        ],
        mobile,
    }
}

const MOBILE: bool = true;
const FIXE: bool = false;

/// Les formes du recueil (§ 60 · 6.1).
pub const FORMES: &[Forme] = &[
    // Majeur.
    forme("E", "022100", MOBILE),
    forme("A", "x02220", MOBILE),
    forme("D", "xx0232", MOBILE),
    forme("C", "x32010", FIXE),
    forme("G", "320003", FIXE),
    // Mineur.
    forme("Em", "022000", MOBILE),
    forme("Am", "x02210", MOBILE),
    forme("Dm", "xx0231", MOBILE),
    // Septième.
    forme("E7", "020100", MOBILE),
    forme("A7", "x02020", MOBILE),
    forme("D7", "xx0212", MOBILE),
    forme("C7", "x32310", FIXE),
    forme("G7", "320001", FIXE),
    forme("B7", "x21202", FIXE),
    // Diminué.
    forme("B°", "x2343x", MOBILE),
    forme("D°", "xx0131", MOBILE),
    // Diminué 7.
    forme("D°7", "xx0101", MOBILE),
    forme("A°7", "x01212", MOBILE),
    forme("A♯°7", "x1202x", MOBILE),
    forme("E°7", "012020", MOBILE),
    // Augmenté.
    forme("D+", "xx0332", MOBILE),
    forme("C+", "x32110", FIXE),
    forme("E+", "032110", FIXE),
    // Premier renversement (la napolitaine).
    forme("C/E", "xx2010", MOBILE),
    forme("G/B", "x20003", MOBILE),
    forme("B♭/D", "xx0331", FIXE),
    forme("F/A", "x03211", FIXE),
];

/// Les places d'une forme : la sienne seule si elle est fixe ; décalée de 0, 1, 2… frettes si
/// elle est mobile, tant qu'aucune frette ne dépasse 15.
fn places(forme: &Forme) -> impl Iterator<Item = [Corde; 6]> + '_ {
    let haute = forme.frettes.iter().flatten().copied().max().unwrap_or(0);
    let decalages = if forme.mobile {
        0..=FRETTE_MAX - haute
    } else {
        0..=0
    };
    decalages.map(move |s| {
        forme
            .frettes
            .map(|frette| Corde::de_frette(frette.map(|n| n + s)))
    })
}

/// Les contraintes de REN-04 hors du nombre de doigts, que `Doigte::des_cordes` tient déjà.
fn tient(d: &Doigte, v: &NotesVoulues) -> bool {
    let sons = d.sons();
    let appuyees: Vec<u8> = d.cordes.iter().filter_map(Corde::appuyee).collect();
    let etendue = match (appuyees.iter().max(), appuyees.iter().min()) {
        (Some(haute), Some(basse)) => haute - basse,
        _ => 0,
    };
    sons.first() == Some(&v.basse)
        && sons.iter().all(|s| v.classes.contains(s))
        && v.classes
            .iter()
            .all(|c| Some(*c) == v.facultative || sons.contains(c))
        && etendue <= ETENDUE_MAX
}

/// Le départage hors contexte : la frette appuyée la plus basse, le plus de cordes à vide, le
/// plus de cordes jouées, le moins de doigts, puis les frettes lues de la grave à l'aiguë, une
/// corde étouffée avant 0.
fn departage(a: &Doigte, b: &Doigte) -> Ordering {
    a.frette_la_plus_basse()
        .cmp(&b.frette_la_plus_basse())
        .then(b.a_vide().cmp(&a.a_vide()))
        .then(b.jouees().cmp(&a.jouees()))
        .then(a.nombre_de_doigts().cmp(&b.nombre_de_doigts()))
        .then_with(|| {
            let frettes = |d: &Doigte| d.cordes.each_ref().map(Corde::jouee);
            frettes(a).cmp(&frettes(b))
        })
}

/// Les formes placées qui tiennent les contraintes, chacune une fois, rangées par le départage
/// hors contexte.
pub fn candidats(v: &NotesVoulues) -> Vec<Doigte> {
    let mut tous: Vec<Doigte> = Vec::new();
    for cordes in FORMES.iter().flat_map(places) {
        if let Some(d) = Doigte::des_cordes(cordes).filter(|d| tient(d, v))
            && !tous.contains(&d)
        {
            tous.push(d);
        }
    }
    tous.sort_by(departage);
    tous
}

/// Le premier candidat du départage ; rien s'il n'y en a pas (`PAS_DE_DOIGTE`).
pub fn hors_contexte(v: &NotesVoulues) -> Option<Doigte> {
    candidats(v).into_iter().next()
}

/// Un maillon d'une suite : un accord à doigter, ou un doigté fixé.
#[derive(Clone, Debug)]
pub enum Maillon {
    Libre(NotesVoulues),
    Fixe(Doigte),
}

/// Le meilleur chemin qui finit sur un candidat : son déplacement, ses cordes à vide, et le rang
/// hors contexte de chaque doigté choisi.
struct Chemin {
    deplacement: u32,
    a_vide: usize,
    rangs: Vec<usize>,
}

impl Chemin {
    /// Le moindre déplacement d'abord, puis le plus de cordes à vide, puis les rangs dans
    /// l'ordre de la suite.
    fn ordre(&self, autre: &Chemin) -> Ordering {
        self.deplacement
            .cmp(&autre.deplacement)
            .then(autre.a_vide.cmp(&self.a_vide))
            .then_with(|| self.rangs.cmp(&autre.rangs))
    }
}

/// Le plus court chemin sur la suite : le déplacement entre deux doigtés est l'écart de leurs
/// zones. À égalité, le plus de cordes à vide, puis, accord par accord, l'ordre hors contexte.
/// Un maillon fixé n'a que son doigté. Un accord sans candidat reste sans doigté, et le chemin
/// relie ses voisins par-dessus lui.
pub fn en_contexte(suite: &[Maillon]) -> Vec<Option<Doigte>> {
    plus_court_chemin(
        suite
            .iter()
            .map(|m| match m {
                Maillon::Libre(v) => candidats(v),
                Maillon::Fixe(d) => vec![d.clone()],
            })
            .collect(),
    )
}

/// Le chemin d'`en_contexte`, sur les candidats de chaque maillon rangés par l'ordre hors
/// contexte : le rang d'un candidat est sa place dans sa liste.
fn plus_court_chemin(listes: Vec<Vec<Doigte>>) -> Vec<Option<Doigte>> {
    let mut chemins: Vec<Chemin> = Vec::new();
    let mut precedente: Option<&Vec<Doigte>> = None;
    for liste in listes.iter().filter(|l| !l.is_empty()) {
        let suivants: Vec<Chemin> = liste
            .iter()
            .enumerate()
            .map(|(rang, d)| match precedente {
                None => Chemin {
                    deplacement: 0,
                    a_vide: d.a_vide(),
                    rangs: vec![rang],
                },
                Some(avant) => chemins
                    .iter()
                    .zip(avant)
                    .map(|(c, p)| Chemin {
                        deplacement: c.deplacement + u32::from(p.zone().abs_diff(d.zone())),
                        a_vide: c.a_vide + d.a_vide(),
                        rangs: [c.rangs.as_slice(), &[rang]].concat(),
                    })
                    .min_by(Chemin::ordre)
                    .expect("le maillon joué précédent a au moins un chemin"),
            })
            .collect();
        chemins = suivants;
        precedente = Some(liste);
    }
    let mut rangs = chemins
        .into_iter()
        .min_by(Chemin::ordre)
        .map(|c| c.rangs)
        .unwrap_or_default()
        .into_iter();
    listes
        .into_iter()
        .map(|liste| {
            if liste.is_empty() {
                None
            } else {
                rangs.next().map(|r| liste[r].clone())
            }
        })
        .collect()
}

/// Les règles que le recueil ne départage jamais seules. Avec le recueil des 27 formes, une
/// recherche exhaustive sur toutes les notes voulues le montre : retirer un des critères 2 à 4 du
/// départage ne change l'ordre d'aucune liste de candidats, aucun candidat placé n'excède trois
/// frettes d'étendue, et un candidat plus loin dans l'ordre n'a jamais plus de cordes à vide. Ce
/// constat tient au recueil, non aux règles : un changement de `FORMES` le remet en cause. Ces
/// règles s'éprouvent donc ici, sur des doigtés hors du recueil.
#[cfg(test)]
mod tests {
    use super::*;

    fn d(notation: &str) -> Doigte {
        Doigte::depuis(notation).expect(notation)
    }

    #[test]
    fn chaque_critere_du_departage_decide_contre_les_suivants() {
        // 1. La frette appuyée la plus basse, contre tous les autres.
        assert_eq!(departage(&d("x1x3xx"), &d("x00002")), Ordering::Less);
        // 2. Le plus de cordes à vide, contre le plus de cordes jouées.
        assert_eq!(departage(&d("xx0002"), &d("222222")), Ordering::Less);
        // 3. Le plus de cordes jouées, contre le moins de doigts.
        assert_eq!(departage(&d("x2222x"), &d("xx22xx")), Ordering::Less);
        // 4. Le moins de doigts, contre l'ordre des frettes.
        assert_eq!(departage(&d("x32222"), &d("x23332")), Ordering::Less);
        // 5. L'ordre des frettes : une corde étouffée avant 0.
        assert_eq!(departage(&d("x02220"), &d("0x2220")), Ordering::Less);
        assert_eq!(departage(&d("0x2220"), &d("x02220")), Ordering::Greater);
        assert_eq!(departage(&d("x02220"), &d("x02220")), Ordering::Equal);
    }

    #[test]
    fn l_etendue_va_jusqu_a_trois_frettes() {
        // Do et fa à trois frettes d'écart, do et fa♯ à quatre.
        let do_fa = NotesVoulues {
            basse: 0,
            classes: vec![0, 5],
            facultative: None,
        };
        let do_fa_diese = NotesVoulues {
            basse: 0,
            classes: vec![0, 6],
            facultative: None,
        };
        assert!(tient(&d("x3xx6x"), &do_fa));
        assert!(!tient(&d("x3xx7x"), &do_fa_diese));
    }

    #[test]
    fn le_chemin_prefere_les_cordes_a_vide_au_rang() {
        // Le doigté du milieu le mieux rangé est en zone 2, l'autre fait sonner deux cordes à
        // vide : entre la zone 0 et la zone 4, les deux chemins déplacent la main de 4.
        let chemin = plus_court_chemin(vec![
            vec![d("x32010")],
            vec![d("xx2xxx"), d("xx00xx")],
            vec![d("xxxx4x")],
        ]);
        assert_eq!(chemin[1], Some(d("xx00xx")));
        // Le déplacement passe avant les cordes à vide : vers la zone 2, le premier l'emporte.
        let chemin = plus_court_chemin(vec![vec![d("xxxx2x")], vec![d("xx2xxx"), d("xx00xx")]]);
        assert_eq!(chemin[1], Some(d("xx2xxx")));
    }
}
