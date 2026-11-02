use noyau::accord::{Accord, Qualite};
use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::tonalite::{Mode, Tonalite};

fn tonalite(lettre: Lettre, alteration: i8, mode: Mode) -> Tonalite {
    Tonalite {
        tonique: NomDeNote { lettre, alteration },
        mode,
    }
}

fn accord(degres: i8, demi_tons: i8, qualite: Qualite, renversement: u8) -> Accord {
    Accord {
        fondamentale: Intervalle::new(degres, demi_tons),
        qualite,
        renversement,
    }
}

fn solfege(notes: Vec<NomDeNote>) -> Vec<String> {
    notes.iter().map(|n| n.solfege()).collect()
}

#[test]
fn noms_dans_la_table() {
    let do_maj = tonalite(Lettre::C, 0, Mode::Majeur);
    assert_eq!(accord(5, 8, Qualite::Majeur, 0).nom(do_maj), "A♭");
    assert_eq!(accord(0, 1, Qualite::Diminue7, 0).nom(do_maj), "C♯°7");
    assert_eq!(accord(1, 1, Qualite::Majeur, 1).nom(do_maj), "D♭/F");
    assert_eq!(accord(5, 9, Qualite::Septieme, 0).nom(do_maj), "A7");

    let la_min = tonalite(Lettre::A, 0, Mode::Mineur);
    assert_eq!(accord(2, 3, Qualite::Augmente, 0).nom(la_min), "C+");

    let re_bemol_maj = tonalite(Lettre::D, -1, Mode::Majeur);
    assert_eq!(accord(5, 8, Qualite::Majeur, 0).nom(re_bemol_maj), "A");

    let sol_diese_min = tonalite(Lettre::G, 1, Mode::Mineur);
    assert_eq!(
        accord(6, 11, Qualite::Diminue7, 0).nom(sol_diese_min),
        "G°7"
    );

    let mi_bemol_min = tonalite(Lettre::E, -1, Mode::Mineur);
    assert_eq!(accord(4, 7, Qualite::Septieme, 0).nom(mi_bemol_min), "B♭7");

    let fa_diese_maj = tonalite(Lettre::F, 1, Mode::Majeur);
    assert_eq!(accord(2, 4, Qualite::Mineur, 0).nom(fa_diese_maj), "A♯m");
}

#[test]
fn notes_ecrites_comme_le_nom() {
    let re_bemol_maj = tonalite(Lettre::D, -1, Mode::Majeur);
    assert_eq!(
        solfege(accord(5, 8, Qualite::Majeur, 0).noms_des_notes(re_bemol_maj)),
        ["la", "do♯", "mi"]
    );

    let sol_diese_min = tonalite(Lettre::G, 1, Mode::Mineur);
    assert_eq!(
        solfege(accord(6, 11, Qualite::Diminue7, 0).noms_des_notes(sol_diese_min)),
        ["sol", "si♭", "ré♭", "fa♭"]
    );
    assert_eq!(
        solfege(accord(4, 7, Qualite::Septieme, 0).noms_des_notes(sol_diese_min)),
        ["ré♯", "sol", "la♯", "do♯"]
    );

    let do_maj = tonalite(Lettre::C, 0, Mode::Majeur);
    assert_eq!(
        solfege(accord(5, 9, Qualite::Septieme, 0).noms_des_notes(do_maj)),
        ["la", "do♯", "mi", "sol"]
    );
}

#[test]
fn tierces() {
    assert!(Qualite::Majeur.tierce_majeure());
    assert!(Qualite::Augmente.tierce_majeure());
    assert!(Qualite::Septieme.tierce_majeure());
    assert!(!Qualite::Mineur.tierce_majeure());
    assert!(!Qualite::Diminue.tierce_majeure());
    assert!(!Qualite::Diminue7.tierce_majeure());
}

#[test]
fn trois_sons_et_suffixes() {
    assert!(Qualite::Majeur.trois_sons());
    assert!(Qualite::Mineur.trois_sons());
    assert!(Qualite::Diminue.trois_sons());
    assert!(Qualite::Augmente.trois_sons());
    assert!(!Qualite::Septieme.trois_sons());
    assert!(!Qualite::Diminue7.trois_sons());

    assert_eq!(Qualite::Majeur.suffixe(), "");
    assert_eq!(Qualite::Mineur.suffixe(), "m");
    assert_eq!(Qualite::Diminue.suffixe(), "°");
    assert_eq!(Qualite::Augmente.suffixe(), "+");
    assert_eq!(Qualite::Septieme.suffixe(), "7");
    assert_eq!(Qualite::Diminue7.suffixe(), "°7");
}

#[test]
fn intervalles_des_qualites() {
    let i = Intervalle::new;
    assert_eq!(Qualite::Majeur.intervalles(), [i(0, 0), i(2, 4), i(4, 7)]);
    assert_eq!(Qualite::Mineur.intervalles(), [i(0, 0), i(2, 3), i(4, 7)]);
    assert_eq!(Qualite::Diminue.intervalles(), [i(0, 0), i(2, 3), i(4, 6)]);
    assert_eq!(Qualite::Augmente.intervalles(), [i(0, 0), i(2, 4), i(4, 8)]);
    assert_eq!(
        Qualite::Septieme.intervalles(),
        [i(0, 0), i(2, 4), i(4, 7), i(6, 10)]
    );
    assert_eq!(
        Qualite::Diminue7.intervalles(),
        [i(0, 0), i(2, 3), i(4, 6), i(6, 9)]
    );
}

#[test]
fn notes_basse_et_fondamentale() {
    let do_maj = tonalite(Lettre::C, 0, Mode::Majeur);
    let napolitaine = accord(1, 1, Qualite::Majeur, 1);
    assert_eq!(
        napolitaine.notes(),
        [
            Intervalle::new(1, 1),
            Intervalle::new(3, 5),
            Intervalle::new(5, 8)
        ]
    );
    assert_eq!(napolitaine.basse(), Intervalle::new(3, 5));
    assert_eq!(do_maj.note(napolitaine.basse()).solfege(), "fa");
    assert_eq!(napolitaine.fondamentale_juste(do_maj).solfege(), "ré♭");
    assert_eq!(
        accord(1, 1, Qualite::Majeur, 0).basse(),
        Intervalle::new(1, 1)
    );

    // Les degrés sont ramenés dans l'octave : la septième d'un accord sur le sixième degré.
    let septieme = accord(5, 9, Qualite::Septieme, 0);
    assert_eq!(septieme.notes()[3], Intervalle::new(4, 7));
}

#[test]
fn ecart_d_ecriture_q21() {
    let re_bemol_maj = tonalite(Lettre::D, -1, Mode::Majeur);
    let vi_bemol = accord(5, 8, Qualite::Majeur, 0);
    assert_eq!(vi_bemol.fondamentale_juste(re_bemol_maj).anglais(), "B𝄫");
    assert_eq!(vi_bemol.ecart_d_ecriture(re_bemol_maj), -1);

    let sol_diese_min = tonalite(Lettre::G, 1, Mode::Mineur);
    let vii = accord(6, 11, Qualite::Diminue7, 0);
    assert_eq!(vii.fondamentale_juste(sol_diese_min).anglais(), "F𝄪");
    assert_eq!(vii.ecart_d_ecriture(sol_diese_min), 1);

    let do_maj = tonalite(Lettre::C, 0, Mode::Majeur);
    assert_eq!(vi_bemol.ecart_d_ecriture(do_maj), 0);
}
