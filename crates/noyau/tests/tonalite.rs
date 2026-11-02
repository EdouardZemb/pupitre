use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::tonalite::{Mode, Tonalite};

fn notes_en_anglais(t: Tonalite) -> Vec<String> {
    t.gamme().iter().map(|&i| t.note(i).anglais()).collect()
}

fn tonalite(lettre: Lettre, alteration: i8, mode: Mode) -> Tonalite {
    Tonalite {
        tonique: NomDeNote { lettre, alteration },
        mode,
    }
}

#[test]
fn les_24_dans_l_ordre_du_cahier() {
    let noms: Vec<_> = Tonalite::les_24().iter().map(|t| t.nom()).collect();
    assert_eq!(
        noms,
        [
            "do majeur",
            "la mineur",
            "sol majeur",
            "mi mineur",
            "ré majeur",
            "si mineur",
            "la majeur",
            "fa♯ mineur",
            "mi majeur",
            "do♯ mineur",
            "si majeur",
            "sol♯ mineur",
            "fa♯ majeur",
            "mi♭ mineur",
            "ré♭ majeur",
            "si♭ mineur",
            "la♭ majeur",
            "fa mineur",
            "mi♭ majeur",
            "do mineur",
            "si♭ majeur",
            "sol mineur",
            "fa majeur",
            "ré mineur"
        ]
    );
}

#[test]
fn chaque_majeure_est_suivie_de_sa_relative() {
    let toutes = Tonalite::les_24();
    for paire in toutes.chunks(2) {
        let (majeure, relative) = (paire[0], paire[1]);
        assert_eq!(majeure.mode, Mode::Majeur);
        assert_eq!(relative.mode, Mode::Mineur);
        assert_eq!(
            relative.tonique.classe(),
            (majeure.tonique.classe() + 9) % 12,
            "{} puis {}",
            majeure.nom(),
            relative.nom()
        );
    }
}

#[test]
fn gammes() {
    assert_eq!(
        notes_en_anglais(tonalite(Lettre::C, 0, Mode::Majeur)),
        ["C", "D", "E", "F", "G", "A", "B"]
    );
    assert_eq!(
        notes_en_anglais(tonalite(Lettre::A, 0, Mode::Mineur)),
        ["A", "B", "C", "D", "E", "F", "G"]
    );
    assert_eq!(
        notes_en_anglais(tonalite(Lettre::F, 1, Mode::Majeur)),
        ["F♯", "G♯", "A♯", "B", "C♯", "D♯", "E♯"]
    );
    assert_eq!(
        notes_en_anglais(tonalite(Lettre::E, -1, Mode::Mineur)),
        ["E♭", "F", "G♭", "A♭", "B♭", "C♭", "D♭"]
    );
}

#[test]
fn les_gammes_elles_memes() {
    let majeur: Vec<_> = tonalite(Lettre::C, 0, Mode::Majeur)
        .gamme()
        .iter()
        .map(|i| i.demi_tons)
        .collect();
    assert_eq!(majeur, [0, 2, 4, 5, 7, 9, 11]);
    let mineur: Vec<_> = tonalite(Lettre::A, 0, Mode::Mineur)
        .gamme()
        .iter()
        .map(|i| i.demi_tons)
        .collect();
    assert_eq!(mineur, [0, 2, 3, 5, 7, 8, 10]);
    for (k, i) in tonalite(Lettre::C, 0, Mode::Majeur)
        .gamme()
        .iter()
        .enumerate()
    {
        assert_eq!(i.degres, k as i8);
    }
}

#[test]
fn aucune_gamme_n_a_de_double_alteration() {
    for t in Tonalite::les_24() {
        for i in t.gamme() {
            assert!(t.note(i).alteration.abs() <= 1, "{} : {:?}", t.nom(), i);
        }
    }
}

#[test]
fn degre_de_classe() {
    let fa = tonalite(Lettre::F, 0, Mode::Majeur);
    assert_eq!(fa.degre_de_classe(10), Some(Intervalle::new(3, 5)));
    assert_eq!(
        fa.note(Intervalle::new(3, 5)),
        NomDeNote {
            lettre: Lettre::B,
            alteration: -1
        }
    );
    let do_majeur = tonalite(Lettre::C, 0, Mode::Majeur);
    assert_eq!(do_majeur.degre_de_classe(1), None);
}
