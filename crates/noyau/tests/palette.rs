use noyau::accord::{Accord, Qualite};
use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::palette::{NoteDePalette, Role, palette, role};
use noyau::table::{CoucheSombre, table};
use noyau::tonalite::{Mode, Tonalite};

fn tonalite(lettre: Lettre, alteration: i8, mode: Mode) -> Tonalite {
    Tonalite {
        tonique: NomDeNote { lettre, alteration },
        mode,
    }
}

fn do_majeur() -> Tonalite {
    tonalite(Lettre::C, 0, Mode::Majeur)
}

fn accord(degres: i8, demi_tons: i8, qualite: Qualite) -> Accord {
    Accord {
        fondamentale: Intervalle::new(degres, demi_tons),
        qualite,
        renversement: 0,
    }
}

/// Les noms de la palette qui ont ce rôle, de la fondamentale vers l'aigu.
fn noms(p: &[NoteDePalette], r: Role) -> Vec<String> {
    p.iter()
        .filter(|n| n.role == r)
        .map(|n| n.nom.solfege())
        .collect()
}

fn lire(a: &Accord, t: Tonalite) -> (Vec<String>, Vec<String>, Vec<String>) {
    let p = palette(a, t);
    (
        noms(&p, Role::Appui),
        noms(&p, Role::Tension),
        noms(&p, Role::Frotte),
    )
}

fn v(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| x.to_string()).collect()
}

#[test]
fn mic13_1_do_en_do_majeur() {
    let (appuis, tensions, frottent) = lire(&accord(0, 0, Qualite::Majeur), do_majeur());
    assert_eq!(appuis, v(&["do", "mi", "sol"]));
    assert_eq!(tensions, v(&["ré", "la"]));
    assert_eq!(frottent, v(&["fa", "si"]));
}

#[test]
fn mic13_2_sol7_en_do_majeur() {
    let (appuis, tensions, frottent) = lire(&accord(4, 7, Qualite::Septieme), do_majeur());
    assert_eq!(appuis, v(&["sol", "si", "ré", "fa"]));
    assert_eq!(tensions, v(&["la", "mi"]));
    assert_eq!(frottent, v(&["do"]));
}

#[test]
fn mic13_3_la_m_en_do_majeur() {
    let (appuis, tensions, frottent) = lire(&accord(5, 9, Qualite::Mineur), do_majeur());
    assert_eq!(appuis, v(&["la", "do", "mi"]));
    assert_eq!(tensions, v(&["si", "ré"]));
    assert_eq!(frottent, v(&["fa", "sol"]));
}

#[test]
fn mic13_5_mi_bemol_en_do_majeur() {
    let t = do_majeur();
    let p = palette(&accord(2, 3, Qualite::Majeur), t);
    assert_eq!(noms(&p, Role::Appui), v(&["mi♭", "sol", "si♭"]));
    let gamme: Vec<u8> = t.gamme().iter().map(|&i| t.note(i).classe()).collect();
    for n in p.iter().filter(|n| n.role != Role::Appui) {
        assert!(
            gamme.contains(&n.nom.classe()),
            "{} hors de la gamme",
            n.nom.solfege()
        );
    }
}

#[test]
fn la_palette_va_de_la_fondamentale_vers_l_aigu() {
    let p = palette(&accord(2, 3, Qualite::Majeur), do_majeur());
    let tout: Vec<(String, Role)> = p.iter().map(|n| (n.nom.solfege(), n.role)).collect();
    let attendu = [
        ("mi♭", Role::Appui),
        ("mi", Role::Frotte),
        ("fa", Role::Tension),
        ("sol", Role::Appui),
        ("la", Role::Tension),
        ("si♭", Role::Appui),
        ("si", Role::Frotte),
        ("do", Role::Tension),
        ("ré", Role::Frotte),
    ];
    let attendu: Vec<(String, Role)> = attendu.iter().map(|(n, r)| (n.to_string(), *r)).collect();
    assert_eq!(tout, attendu);
}

#[test]
fn les_appuis_sont_exactement_les_notes_de_l_accord() {
    for t in Tonalite::les_24() {
        let gamme: Vec<u8> = t.gamme().iter().map(|&i| t.note(i).classe()).collect();
        for c in table(t, CoucheSombre::Affichee).cases {
            let a = c.accord;
            let de_l_accord: Vec<u8> = a.notes().iter().map(|&i| t.note(i).classe()).collect();
            let p = palette(&a, t);

            let mut appuis: Vec<u8> = p
                .iter()
                .filter(|n| n.role == Role::Appui)
                .map(|n| n.nom.classe())
                .collect();
            let mut attendus = de_l_accord.clone();
            appuis.sort();
            attendus.sort();
            assert_eq!(appuis, attendus, "{} {}", t.nom(), c.chiffre);

            for n in p.iter().filter(|n| n.role != Role::Appui) {
                assert!(gamme.contains(&n.nom.classe()), "{} {}", t.nom(), c.chiffre);
            }

            let mut classes: Vec<u8> = p.iter().map(|n| n.nom.classe()).collect();
            classes.sort();
            let mut union: Vec<u8> = de_l_accord.into_iter().chain(gamme.clone()).collect();
            union.sort();
            union.dedup();
            assert_eq!(
                classes,
                union,
                "{} {} : une note par classe",
                t.nom(),
                c.chiffre
            );
        }
    }
}

#[test]
fn la_palette_ne_classe_que_par_role() {
    for t in Tonalite::les_24() {
        for c in table(t, CoucheSombre::Affichee).cases {
            for n in palette(&c.accord, t) {
                assert_eq!(
                    role(n.note, &c.accord, t),
                    Some(n.role),
                    "{} {}",
                    t.nom(),
                    c.chiffre
                );
            }
        }
    }
}

#[test]
fn une_note_hors_de_l_accord_et_de_la_tonalite_n_a_pas_de_role() {
    // do♯ contre `C` en do majeur : ni de l'accord ni de la gamme.
    assert_eq!(
        role(
            Intervalle::new(0, 1),
            &accord(0, 0, Qualite::Majeur),
            do_majeur()
        ),
        None
    );
}

#[test]
fn les_notes_s_ecrivent_comme_le_nom() {
    // ♭VI de ré♭ majeur : `A`, écrit la, do♯, mi.
    let t = tonalite(Lettre::D, -1, Mode::Majeur);
    let tb = table(t, CoucheSombre::Affichee);
    let c = tb.cases.iter().find(|c| c.chiffre == "♭VI").unwrap();
    assert_eq!(tb.nom(c), "A");
    assert_eq!(
        noms(&palette(&c.accord, t), Role::Appui),
        v(&["la", "do♯", "mi"])
    );

    // D♯7 en sol♯ mineur : sol, non fa𝄪.
    let t = tonalite(Lettre::G, 1, Mode::Mineur);
    let tb = table(t, CoucheSombre::Affichee);
    let c = tb.cases.iter().find(|c| c.chiffre == "V7/i").unwrap();
    assert_eq!(tb.nom(c), "D♯7");
    assert_eq!(
        noms(&palette(&c.accord, t), Role::Appui),
        v(&["ré♯", "sol", "la♯", "do♯"])
    );

    // Aucune note de aucune palette ne porte de double altération.
    for t in Tonalite::les_24() {
        for c in table(t, CoucheSombre::Affichee).cases {
            for n in palette(&c.accord, t) {
                assert!(n.nom.alteration.abs() <= 1, "{} {}", t.nom(), c.chiffre);
            }
        }
    }
}
