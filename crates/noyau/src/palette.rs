//! La palette d'un accord (MIC-13) : ses notes par rôle. `role` est la seule fonction qui classe ;
//! la palette l'appelle, et MIC-06 l'appellera pour les notes d'une mélodie (I27).

use crate::accord::Accord;
use crate::hauteur::{Intervalle, NomDeNote};
use crate::tonalite::Tonalite;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Appui,
    Tension,
    Frotte,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NoteDePalette {
    /// La note depuis la tonique.
    pub note: Intervalle,
    /// Son nom, écrit comme le nom de l'accord.
    pub nom: NomDeNote,
    pub role: Role,
}

/// Le rôle d'une note (depuis la tonique) contre un accord, calculé en degrés sur l'orthographe
/// juste. `None` pour une note hors de l'accord et hors de la tonalité.
pub fn role(note: Intervalle, accord: &Accord, t: Tonalite) -> Option<Role> {
    let nom = t.note(note);
    let classe = nom.classe();
    if accord.notes().iter().any(|&n| t.note(n).classe() == classe) {
        return Some(Role::Appui);
    }
    t.degre_de_classe(classe)?;

    // L'intervalle orthographié au-dessus de la fondamentale, ramené à l'octave.
    let au_dessus = accord.fondamentale_juste(t).intervalle_vers(nom);
    let tierce_majeure = accord.qualite.tierce_majeure();
    let tension = match (au_dessus.degres, au_dessus.alteration()) {
        // Une 9 : la 9 et la ♯9 sont des tensions ; la ♭9 frotte, comme toute autre note.
        (1, 0 | 1) => true,
        // Une 11 : la 11 juste frotte sur un accord majeur ; la ♯11 est une tension.
        (3, 0) => !tierce_majeure,
        (3, 1) => true,
        // Une 13 : la ♭13 frotte sur un accord mineur ; la 13 est une tension.
        (5, -1) => tierce_majeure,
        (5, 0) => true,
        // Une septième, ou une note altérée contre l'accord, frotte.
        _ => false,
    };
    Some(if tension { Role::Tension } else { Role::Frotte })
}

/// Les notes de l'accord et de la gamme, une par classe de hauteur, de la fondamentale vers l'aigu.
/// Pour une classe partagée, la note de l'accord.
pub fn palette(accord: &Accord, t: Tonalite) -> Vec<NoteDePalette> {
    let mut notes: Vec<Intervalle> = Vec::new();
    for note in accord.notes().into_iter().chain(t.gamme()) {
        if notes
            .iter()
            .all(|&n| t.note(n).classe() != t.note(note).classe())
        {
            notes.push(note);
        }
    }

    let fondamentale = t.note(accord.fondamentale).classe() as i8;
    notes.sort_by_key(|&n| (t.note(n).classe() as i8 - fondamentale).rem_euclid(12));

    let ecart = accord.ecart_d_ecriture(t);
    notes
        .into_iter()
        .map(|note| NoteDePalette {
            note,
            nom: t.note(note).meme_son(ecart).enharmonique_simple(),
            role: role(note, accord, t).expect("toute note de la palette a un rôle"),
        })
        .collect()
}
