use noyau::hauteur::{Lettre, NomDeNote};
use noyau::table::{CoucheSombre, Famille, Fonction, Position, Rangee, Table, table};
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

fn la_mineur() -> Tonalite {
    tonalite(Lettre::A, 0, Mode::Mineur)
}

fn pos(rangee: Rangee, rang: u8) -> Position {
    Position { rangee, rang }
}

/// Les cases d'une rangée : (rang, chiffre, nom).
fn lignes(t: Tonalite, rangee: Rangee) -> Vec<(u8, String, String)> {
    let tb = table(t, CoucheSombre::Affichee);
    tb.rangee(rangee)
        .into_iter()
        .map(|c| (c.position.rang, c.chiffre.clone(), tb.nom(c)))
        .collect()
}

fn attendu(v: &[(u8, &str, &str)]) -> Vec<(u8, String, String)> {
    v.iter()
        .map(|(r, c, n)| (*r, c.to_string(), n.to_string()))
        .collect()
}

#[test]
fn mic12_1_do_majeur() {
    let t = do_majeur();
    assert_eq!(
        lignes(t, Rangee::Centre),
        attendu(&[
            (0, "I", "C"),
            (1, "ii", "Dm"),
            (2, "iii", "Em"),
            (3, "IV", "F"),
            (4, "V", "G"),
            (5, "vi", "Am"),
            (6, "vii°", "B°"),
        ])
    );
    assert_eq!(
        lignes(t, Rangee::Dessus),
        attendu(&[
            (1, "V7/ii", "A7"),
            (2, "V7/iii", "B7"),
            (3, "V7/IV", "C7"),
            (4, "V7/V", "D7"),
            (5, "V7/vi", "E7"),
        ])
    );
    let tb = table(t, CoucheSombre::Masquee);
    for c in tb.rangee(Rangee::Dessus) {
        assert_eq!(c.famille, Famille::DominanteSecondaire);
        assert_eq!(c.cible, Some(pos(Rangee::Centre, c.position.rang)));
        assert_eq!(tb.cible_de(c.position), c.cible);
        assert_eq!(c.fonction, None);
    }
    for c in tb.rangee(Rangee::Centre) {
        assert_eq!(c.famille, Famille::Diatonique);
    }
}

#[test]
fn mic12_2_la_mineur() {
    let t = la_mineur();
    assert_eq!(
        lignes(t, Rangee::Centre),
        attendu(&[
            (0, "i", "Am"),
            (1, "ii°", "B°"),
            (2, "III", "C"),
            (3, "iv", "Dm"),
            (4, "v", "Em"),
            (5, "VI", "F"),
            (6, "VII", "G"),
        ])
    );
    assert_eq!(
        lignes(t, Rangee::Dessus),
        attendu(&[
            (0, "V7/i", "E7"),
            (2, "V7/III", "G7"),
            (3, "V7/iv", "A7"),
            (4, "V7/v", "B7"),
            (5, "V7/VI", "C7"),
            (6, "V7/VII", "D7"),
        ])
    );
    let tb = table(t, CoucheSombre::Masquee);
    let e7 = tb.case(pos(Rangee::Dessus, 0)).unwrap();
    assert_eq!(e7.cible, Some(pos(Rangee::Centre, 0)));
}

#[test]
fn mic12_3_de_do_a_mi_bemol() {
    let mi_bemol = tonalite(Lettre::E, -1, Mode::Majeur);
    let a = table(do_majeur(), CoucheSombre::Affichee);
    let b = table(mi_bemol, CoucheSombre::Affichee);
    let cle = |tb: &Table| {
        tb.cases
            .iter()
            .map(|c| (c.position, c.chiffre.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(cle(&a), cle(&b));
    let nom = |tb: &Table, r, n| tb.nom(tb.case(pos(r, n)).unwrap());
    assert_eq!(nom(&a, Rangee::Centre, 0), "C");
    assert_eq!(nom(&b, Rangee::Centre, 0), "E♭");
    assert_eq!(nom(&a, Rangee::Dessus, 1), "A7");
    assert_eq!(nom(&b, Rangee::Dessus, 1), "C7");
    assert_eq!(nom(&a, Rangee::Centre, 6), "B°");
    assert_eq!(nom(&b, Rangee::Centre, 6), "D°");
}

#[test]
fn mic12_4_fa_diese_et_mi_bemol_mineur() {
    let fa_diese = tonalite(Lettre::F, 1, Mode::Majeur);
    let tb = table(fa_diese, CoucheSombre::Masquee);
    let iii = tb.case(pos(Rangee::Centre, 2)).unwrap();
    assert_eq!((iii.chiffre.as_str(), tb.nom(iii).as_str()), ("iii", "A♯m"));

    let mi_bemol_mineur = tonalite(Lettre::E, -1, Mode::Mineur);
    let tb = table(mi_bemol_mineur, CoucheSombre::Masquee);
    let v7 = tb.case(pos(Rangee::Dessus, 0)).unwrap();
    assert_eq!((v7.chiffre.as_str(), tb.nom(v7).as_str()), ("V7/i", "B♭7"));
}

#[test]
fn mic12_5_couche_sombre_masquee_par_defaut() {
    assert_eq!(CoucheSombre::default(), CoucheSombre::Masquee);
    for t in Tonalite::les_24() {
        let masquee = table(t, CoucheSombre::default());
        assert_eq!(masquee.couche_sombre, CoucheSombre::Masquee);
        assert!(masquee.rangee(Rangee::Sombre).is_empty());
        assert!(masquee.cases.iter().all(|c| c.famille != Famille::Sombre));
        let affichee = table(t, CoucheSombre::Affichee);
        assert_eq!(affichee.couche_sombre, CoucheSombre::Affichee);
        assert!(!affichee.rangee(Rangee::Sombre).is_empty());
        // Hors couche sombre, les deux tables sont identiques.
        let reste: Vec<_> = affichee
            .cases
            .iter()
            .filter(|c| c.position.rangee != Rangee::Sombre)
            .cloned()
            .collect();
        assert_eq!(reste, masquee.cases);
    }
}

#[test]
fn mic12_9_re_bemol_majeur() {
    let t = tonalite(Lettre::D, -1, Mode::Majeur);
    let tb = table(t, CoucheSombre::Masquee);
    let vi = tb.case(pos(Rangee::Dessous, 5)).unwrap();
    assert_eq!(vi.chiffre, "♭VI");
    assert_eq!(tb.nom(vi), "A");
}

#[test]
fn emprunts_par_colonne() {
    assert_eq!(
        lignes(do_majeur(), Rangee::Dessous),
        attendu(&[
            (0, "i", "Cm"),
            (1, "ii°", "D°"),
            (2, "♭III", "E♭"),
            (3, "iv", "Fm"),
            (4, "v", "Gm"),
            (5, "♭VI", "A♭"),
            (6, "♭VII", "B♭"),
        ])
    );
    assert_eq!(
        lignes(la_mineur(), Rangee::Dessous),
        attendu(&[
            (0, "I", "A"),
            (1, "ii", "Bm"),
            (2, "iii", "C♯m"),
            (3, "IV", "D"),
            (4, "V", "E"),
            (5, "vi", "F♯m"),
            (6, "vii°", "G♯°"),
        ])
    );
    let tb = table(do_majeur(), CoucheSombre::Masquee);
    assert!(
        tb.rangee(Rangee::Dessous)
            .iter()
            .all(|c| c.famille == Famille::Emprunt && c.cible.is_none() && c.fonction.is_none())
    );
}

type LigneSombre = (u8, String, String, Option<Position>);

fn sombres(t: Tonalite) -> Vec<LigneSombre> {
    let tb = table(t, CoucheSombre::Affichee);
    tb.rangee(Rangee::Sombre)
        .into_iter()
        .map(|c| (c.position.rang, c.chiffre.clone(), tb.nom(c), c.cible))
        .collect()
}

fn s(r: u8, c: &str, n: &str, cible: Option<Position>) -> LigneSombre {
    (r, c.to_string(), n.to_string(), cible)
}

#[test]
fn couche_sombre_majeure() {
    let centre = |n| Some(pos(Rangee::Centre, n));
    assert_eq!(
        sombres(do_majeur()),
        vec![
            s(0, "vii°7/I", "B°7", centre(0)),
            s(1, "vii°7/ii", "C♯°7", centre(1)),
            s(2, "vii°7/iii", "D♯°7", centre(2)),
            s(3, "vii°7/IV", "E°7", centre(3)),
            s(4, "vii°7/V", "F♯°7", centre(4)),
            s(5, "♭II6", "D♭/F", centre(4)),
            s(6, "vii°7/vi", "G♯°7", centre(5)),
        ]
    );
    let tb = table(do_majeur(), CoucheSombre::Affichee);
    assert!(
        tb.rangee(Rangee::Sombre)
            .iter()
            .all(|c| c.famille == Famille::Sombre && c.fonction.is_none())
    );
}

#[test]
fn couche_sombre_mineure() {
    let centre = |n| Some(pos(Rangee::Centre, n));
    assert_eq!(
        sombres(la_mineur()),
        vec![
            s(0, "vii°7/i", "G♯°7", centre(0)),
            s(1, "III+", "C+", None),
            s(2, "vii°7/III", "B°7", centre(2)),
            s(3, "vii°7/iv", "C♯°7", centre(3)),
            s(4, "vii°7/v", "D♯°7", centre(4)),
            s(5, "♭II6", "B♭/D", Some(pos(Rangee::Dessous, 4))),
            s(6, "vii°7/VI", "E°7", centre(5)),
            s(7, "vii°7/VII", "F♯°7", centre(6)),
        ]
    );
}

fn effet(t: Tonalite, r: Rangee, n: u8) -> String {
    let tb = table(t, CoucheSombre::Affichee);
    tb.ligne_d_effet(tb.case(pos(r, n)).unwrap())
}

fn fonctions(t: Tonalite) -> Vec<Option<Fonction>> {
    let tb = table(t, CoucheSombre::Masquee);
    tb.rangee(Rangee::Centre)
        .iter()
        .map(|c| c.fonction)
        .collect()
}

#[test]
fn lignes_d_effet() {
    let (m, n) = (do_majeur(), la_mineur());
    assert_eq!(effet(m, Rangee::Dessus, 1), "tire vers Dm");
    assert_eq!(effet(n, Rangee::Dessus, 0), "tire vers Am");
    assert_eq!(effet(m, Rangee::Dessous, 0), "assombrit");
    assert_eq!(effet(n, Rangee::Dessous, 0), "éclaire");
    assert_eq!(effet(m, Rangee::Sombre, 1), "glisse vers Dm");
    assert_eq!(effet(n, Rangee::Sombre, 7), "glisse vers G");
    assert_eq!(
        effet(m, Rangee::Sombre, 5),
        "tension sombre avant la dominante"
    );
    assert_eq!(
        effet(n, Rangee::Sombre, 5),
        "tension sombre avant la dominante"
    );
    assert_eq!(effet(n, Rangee::Sombre, 1), "suspend");

    let majeur = [
        "repos",
        "éloignement",
        "repos",
        "éloignement",
        "tension",
        "repos",
        "tension",
    ];
    let mineur = [
        "repos",
        "éloignement",
        "repos",
        "éloignement",
        "tension",
        "éloignement",
        "tension",
    ];
    for i in 0..7u8 {
        assert_eq!(effet(m, Rangee::Centre, i), majeur[i as usize]);
        assert_eq!(effet(n, Rangee::Centre, i), mineur[i as usize]);
    }

    use Fonction::{Eloignement as E, Repos as R, Tension as T};
    assert_eq!(fonctions(m), [R, E, R, E, T, R, T].map(Some).to_vec());
    assert_eq!(fonctions(n), [R, E, R, E, T, E, T].map(Some).to_vec());
}

#[test]
fn une_case_sans_cible() {
    let tb = table(la_mineur(), CoucheSombre::Affichee);
    assert_eq!(tb.cible_de(pos(Rangee::Centre, 0)), None);
    assert_eq!(tb.cible_de(pos(Rangee::Dessous, 3)), None);
    assert_eq!(tb.cible_de(pos(Rangee::Sombre, 1)), None);
    assert_eq!(
        tb.cible_de(pos(Rangee::Dessus, 0)),
        Some(pos(Rangee::Centre, 0))
    );
    assert_eq!(tb.cible_de(pos(Rangee::Dessus, 1)), None);
    assert!(tb.case(pos(Rangee::Dessus, 1)).is_none());
    assert!(tb.case(pos(Rangee::Centre, 7)).is_none());
}

fn etage(r: Rangee) -> i8 {
    match r {
        Rangee::Dessus => 0,
        Rangee::Centre => 1,
        Rangee::Sombre => 2,
        Rangee::Dessous => 3,
    }
}

#[test]
fn mic12_7_chaque_fleche_relie_deux_cases() {
    for t in Tonalite::les_24() {
        let tb = table(t, CoucheSombre::Affichee);
        for c in &tb.cases {
            let Some(cible) = c.cible else { continue };
            let arrivee = tb
                .case(cible)
                .unwrap_or_else(|| panic!("{} : cible absente", t.nom()));
            assert_ne!(cible, c.position);
            assert_eq!((etage(cible.rangee) - etage(c.position.rangee)).abs(), 1);
            assert_eq!(tb.cible_de(c.position), Some(arrivee.position));
        }
    }
}

#[test]
fn aucun_nom_de_la_table_n_a_de_double_alteration() {
    for t in Tonalite::les_24() {
        let tb = table(t, CoucheSombre::Affichee);
        for c in &tb.cases {
            let nom = tb.nom(c);
            assert!(
                !nom.contains('𝄫') && !nom.contains('𝄪'),
                "{} : {nom}",
                t.nom()
            );
        }
    }
}

#[test]
fn i26_translation_des_noms() {
    let les_24 = Tonalite::les_24();
    let mut paires = 0;
    for t1 in les_24 {
        for t2 in les_24 {
            if t1.mode != t2.mode {
                continue;
            }
            paires += 1;
            let a = table(t1, CoucheSombre::Affichee);
            let b = table(t2, CoucheSombre::Affichee);
            assert_eq!(a.cases.len(), b.cases.len());
            let ecart_t = t2.tonique.classe() as i8 - t1.tonique.classe() as i8;
            let ecart_lettre = (t2.tonique.lettre as i8 - t1.tonique.lettre as i8).rem_euclid(7);
            for (c1, c2) in a.cases.iter().zip(&b.cases) {
                assert_eq!((c1.position, &c1.chiffre), (c2.position, &c2.chiffre));
                let f1 = c1.accord.fondamentale_juste(t1).classe() as i8;
                let f2 = c2.accord.fondamentale_juste(t2).classe() as i8;
                assert_eq!(f2, (f1 + ecart_t).rem_euclid(12));
                if c1.accord.ecart_d_ecriture(t1) == 0 && c2.accord.ecart_d_ecriture(t2) == 0 {
                    let l1 = c1.accord.noms_des_notes(t1)[0].lettre as i8;
                    let l2 = c2.accord.noms_des_notes(t2)[0].lettre as i8;
                    assert_eq!(l2, (l1 + ecart_lettre).rem_euclid(7));
                }
            }
        }
    }
    assert_eq!(paires, 288);
}
