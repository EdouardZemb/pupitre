//! Le voicing au clavier (REN-08) : la disposition des notes d'un accord sous la main.
//!
//! Un candidat par renversement, en position serrée, la basse entre do 3 et si 3 (do 3 = MIDI
//! 60). Hors contexte, l'accord est dans son renversement propre ; dans une suite, le candidat qui
//! déplace le moins la main l'emporte, et à égalité la basse la plus grave.

use crate::accord::Accord;
use crate::tonalite::Tonalite;

/// Le do 3, la note la plus grave d'une basse (la 3 = 440 Hz).
const DO_3: u8 = 60;

/// Les notes d'un voicing en MIDI, ascendantes, la basse d'abord.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Voicing {
    pub notes: Vec<u8>,
}

/// La classe de hauteur absolue d'une note de l'accord, do = 0.
fn classe_absolue(t: Tonalite, demi_tons: i8) -> u8 {
    (t.tonique.classe() as i8 + demi_tons).rem_euclid(12) as u8
}

/// La hauteur la plus proche strictement au-dessus de `base` dont la classe est `classe`.
fn au_dessus(base: u8, classe: u8) -> u8 {
    base + 1 + (classe as i16 - base as i16 - 1).rem_euclid(12) as u8
}

fn voicing_du_renversement(accord: &Accord, t: Tonalite, r: usize) -> Voicing {
    let classes: Vec<u8> = accord
        .notes()
        .iter()
        .map(|n| classe_absolue(t, n.demi_tons))
        .collect();
    let n = classes.len();
    // La basse est entre 60 et 71 : sa classe donne sa hauteur, 60 étant do.
    let mut notes = vec![DO_3 + classes[r]];
    for k in 1..n {
        let precedente = notes[k - 1];
        notes.push(au_dessus(precedente, classes[(r + k) % n]));
    }
    Voicing { notes }
}

/// Les candidats : un par renversement ; un accord à basse imposée ne garde que le sien.
pub fn candidats(accord: &Accord, t: Tonalite) -> Vec<Voicing> {
    let n = accord.notes().len();
    let rangs: Vec<usize> = if accord.renversement > 0 {
        vec![accord.renversement as usize]
    } else {
        (0..n).collect()
    };
    rangs
        .into_iter()
        .map(|r| voicing_du_renversement(accord, t, r))
        .collect()
}

/// Le voicing dont la basse est celle de l'accord.
pub fn hors_contexte(accord: &Accord, t: Tonalite) -> Voicing {
    voicing_du_renversement(accord, t, accord.renversement as usize)
}

fn distance_a_la_plus_proche(note: u8, autres: &[u8]) -> u32 {
    autres
        .iter()
        .map(|&o| note.abs_diff(o) as u32)
        .min()
        .unwrap_or(0)
}

/// Le déplacement de la main : pour chaque note de l'un, la distance en demi-tons à la note la
/// plus proche de l'autre, dans les deux sens.
pub fn deplacement(a: &Voicing, b: &Voicing) -> u32 {
    let aller: u32 = a
        .notes
        .iter()
        .map(|&n| distance_a_la_plus_proche(n, &b.notes))
        .sum();
    let retour: u32 = b
        .notes
        .iter()
        .map(|&n| distance_a_la_plus_proche(n, &a.notes))
        .sum();
    aller + retour
}

/// Sans voicing précédent, hors contexte ; sinon le candidat de plus petit déplacement, et à
/// égalité la basse la plus grave.
pub fn en_contexte(precedent: Option<&Voicing>, accord: &Accord, t: Tonalite) -> Voicing {
    let Some(precedent) = precedent else {
        return hors_contexte(accord, t);
    };
    candidats(accord, t)
        .into_iter()
        .min_by_key(|v| (deplacement(precedent, v), v.notes[0]))
        .expect("un accord a au moins un candidat")
}
