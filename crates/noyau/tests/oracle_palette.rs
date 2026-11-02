//! L'oracle de la palette (MIC-13, I27), écrit à la main : pour chaque case, les appuis, les
//! tensions et les notes qui frottent, en solfège. Chaque ligne rappelle la phrase de MIC-06 ou
//! de MIC-13 qui la fonde.

use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::palette::{Role, role};
use noyau::table::{CoucheSombre, table};
use noyau::tonalite::{Mode, Tonalite};

fn do_majeur() -> Tonalite {
    Tonalite {
        tonique: NomDeNote {
            lettre: Lettre::C,
            alteration: 0,
        },
        mode: Mode::Majeur,
    }
}

fn la_mineur() -> Tonalite {
    Tonalite {
        tonique: NomDeNote {
            lettre: Lettre::A,
            alteration: 0,
        },
        mode: Mode::Mineur,
    }
}

/// Un nom en solfège (`mi♭`, `do♯`, `sol`) lu en `NomDeNote`.
fn nom(s: &str) -> NomDeNote {
    let lettre = [
        ("do", Lettre::C),
        ("ré", Lettre::D),
        ("mi", Lettre::E),
        ("fa", Lettre::F),
        ("sol", Lettre::G),
        ("la", Lettre::A),
        ("si", Lettre::B),
    ]
    .into_iter()
    .find(|(n, _)| s.starts_with(n))
    .expect("une lettre en solfège")
    .1;
    let signe = s.trim_start_matches(|c: char| c.is_alphabetic());
    let alteration = match signe {
        "" => 0,
        "♭" => -1,
        "♯" => 1,
        autre => panic!("altération inconnue : {autre}"),
    };
    NomDeNote { lettre, alteration }
}

struct Ligne {
    t: Tonalite,
    chiffre: &'static str,
    appuis: &'static str,
    tensions: &'static str,
    frottent: &'static str,
}

fn l(
    t: Tonalite,
    chiffre: &'static str,
    appuis: &'static str,
    tensions: &'static str,
    frottent: &'static str,
) -> Ligne {
    Ligne {
        t,
        chiffre,
        appuis,
        tensions,
        frottent,
    }
}

fn oracle() -> Vec<Ligne> {
    let (c, a) = (do_majeur(), la_mineur());
    vec![
        // MIC-13 : « tension, une 9, 11 ou 13 diatonique qui se tient » ; la 11 juste sur un
        // accord majeur frotte, la septième d'un accord de trois sons frotte (Q17).
        l(c, "I", "do mi sol", "ré la", "fa si"),
        // MIC-13 : 9, 11, 13 tenues sur un accord mineur ; la septième frotte.
        l(c, "ii", "ré fa la", "mi sol si", "do"),
        // MIC-13 : la ♭9 (fa sur mi) et la ♭13 sur un accord mineur frottent ; la septième aussi.
        l(c, "iii", "mi sol si", "la", "fa do ré"),
        // MIC-13 : la ♯11 (si sur fa) est une tension ; la septième frotte.
        l(c, "IV", "fa la do", "sol si ré", "mi"),
        // MIC-13 : 9 et 13 tenues ; la 11 juste sur un accord majeur frotte ; la septième aussi.
        l(c, "V", "sol si ré", "la mi", "do fa"),
        // MIC-13 : la ♭13 sur un accord mineur frotte (fa) ; la septième (sol) aussi.
        l(c, "vi", "la do mi", "si ré", "fa sol"),
        // MIC-13 : la ♭9 (do sur si) et la ♭13 sur un accord mineur frottent ; la septième aussi.
        l(c, "vii°", "si ré fa", "mi", "do sol la"),
        // MIC-13 : les appuis s'affichent même hors de la tonalité (do♯) ; la ♭13 sur un accord
        // majeur (fa sur la) est une tension ; la tierce altérée (do) et la 11 juste sur un
        // accord majeur (ré) frottent.
        l(c, "V7/ii", "la do♯ mi sol", "si fa", "do ré"),
        // MIC-13 : 9, ♯11 et 13 tenues ; la septième frotte ; l'unisson et la quinte altérés
        // contre l'accord frottent (Q18).
        l(c, "♭III", "mi♭ sol si♭", "fa la do", "ré mi si"),
        // MIC-13 : mi est une ♯9 (tension) et sol une ♯11 (tension) ; la septième, l'unisson
        // altéré, la quinte altérée et la sixte augmentée frottent.
        l(c, "♭II6", "ré♭ fa la♭", "mi sol", "do ré la si"),
        // Le mode mineur suit les mêmes phrases, par les degrés (la mineur, i, comme vi de do).
        l(a, "i", "la do mi", "si ré", "fa sol"),
        l(a, "ii°", "si ré fa", "mi", "do sol la"),
        l(a, "III", "do mi sol", "ré la", "fa si"),
        l(a, "iv", "ré fa la", "mi sol si", "do"),
        l(a, "v", "mi sol si", "la", "fa do ré"),
        l(a, "VI", "fa la do", "sol si ré", "mi"),
        l(a, "VII", "sol si ré", "la mi", "do fa"),
        // MIC-13 : la ♭13 sur un accord majeur est une tension (do sur mi) ; la 11 juste sur un
        // accord majeur, la ♭9 et la tierce altérée (sol contre sol♯) frottent.
        l(a, "V7/i", "mi sol♯ si ré", "do", "la fa sol"),
        // MIC-13 : 9 et 13 tenues ; la 11 juste sur un accord majeur, la septième et la tierce
        // altérée (fa contre fa♯) frottent.
        l(a, "IV", "ré fa♯ la", "mi si", "sol do fa"),
        // MIC-13 : un accord diminué de septième n'a aucune tension ; la ♭9 (la), la quarte
        // diminuée (do), la ♭13 sur un accord mineur (mi) et l'octave diminuée (sol) frottent.
        l(a, "vii°7/i", "sol♯ si ré fa", "", "la do mi sol"),
    ]
}

fn depuis_la_tonique(t: Tonalite, s: &str) -> Intervalle {
    t.tonique.intervalle_vers(nom(s))
}

#[test]
fn l_oracle_de_la_palette() {
    for ligne in oracle() {
        let t = ligne.t;
        let tb = table(t, CoucheSombre::Affichee);
        let cases: Vec<_> = tb
            .cases
            .iter()
            .filter(|c| c.chiffre == ligne.chiffre)
            .collect();
        assert_eq!(
            cases.len(),
            1,
            "{} {} : une seule case",
            t.nom(),
            ligne.chiffre
        );
        let accord = cases[0].accord;

        let attendu: Vec<(&str, Role)> = [
            (ligne.appuis, Role::Appui),
            (ligne.tensions, Role::Tension),
            (ligne.frottent, Role::Frotte),
        ]
        .into_iter()
        .flat_map(|(noms, r)| noms.split_whitespace().map(move |n| (n, r)))
        .collect();

        for (n, r) in &attendu {
            assert_eq!(
                role(depuis_la_tonique(t, n), &accord, t),
                Some(*r),
                "{} {} : {n}",
                t.nom(),
                ligne.chiffre
            );
        }

        // Aucune note de la gamme n'est oubliée : chacune est listée.
        let classes: Vec<u8> = attendu.iter().map(|(n, _)| nom(n).classe()).collect();
        for degre in t.gamme() {
            let classe = t.note(degre).classe();
            assert!(
                classes.contains(&classe),
                "{} {} : la note {} de la gamme n'est pas listée",
                t.nom(),
                ligne.chiffre,
                t.note(degre).solfege()
            );
        }
    }
}
