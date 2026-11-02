use noyau::accord::{Accord, Qualite};
use noyau::clavier::{Voicing, candidats, deplacement, en_contexte, hors_contexte};
use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::table::{CoucheSombre, Rangee, table};
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

fn accord(degres: i8, demi_tons: i8, qualite: Qualite, renversement: u8) -> Accord {
    Accord {
        fondamentale: Intervalle::new(degres, demi_tons),
        qualite,
        renversement,
    }
}

fn voicing(notes: &[u8]) -> Voicing {
    Voicing {
        notes: notes.to_vec(),
    }
}

#[test]
fn ren08_1_do_hors_contexte() {
    let c = accord(0, 0, Qualite::Majeur, 0);
    assert_eq!(hors_contexte(&c, do_majeur()), voicing(&[60, 64, 67]));
}

#[test]
fn ren08_2_do_apres_sol() {
    let c = accord(0, 0, Qualite::Majeur, 0);
    let sol = voicing(&[67, 71, 74]);
    let choisi = en_contexte(Some(&sol), &c, do_majeur());
    assert_eq!(choisi, voicing(&[67, 72, 76]));
    assert_eq!(deplacement(&sol, &choisi), 6);
    assert_eq!(deplacement(&sol, &voicing(&[64, 67, 72])), 7);
    assert_eq!(deplacement(&sol, &voicing(&[60, 64, 67])), 21);
    let tous = candidats(&c, do_majeur());
    assert_eq!(tous.len(), 3);
    for v in [
        voicing(&[60, 64, 67]),
        voicing(&[64, 67, 72]),
        voicing(&[67, 72, 76]),
    ] {
        assert!(tous.contains(&v), "{v:?} manque");
    }
}

#[test]
fn le_deplacement_est_symetrique() {
    let a = voicing(&[60, 64, 67]);
    let b = voicing(&[67, 71, 74, 77]);
    assert_eq!(deplacement(&a, &b), deplacement(&b, &a));
    assert_eq!(deplacement(&a, &a), 0);
}

#[test]
fn premier_accord_sans_precedent() {
    let t = do_majeur();
    let c = accord(4, 7, Qualite::Septieme, 0);
    assert_eq!(en_contexte(None, &c, t), hors_contexte(&c, t));
}

#[test]
fn la_napolitaine_garde_sa_basse() {
    let t = do_majeur();
    let napolitaine = accord(1, 1, Qualite::Majeur, 1);
    assert_eq!(hors_contexte(&napolitaine, t), voicing(&[65, 68, 73]));
    let tous = candidats(&napolitaine, t);
    assert_eq!(tous.len(), 1);
    assert!(tous.iter().all(|v| v.notes[0] == 65));
    let apres = voicing(&[67, 71, 74]);
    assert_eq!(en_contexte(Some(&apres), &napolitaine, t).notes[0], 65);
}

#[test]
fn une_septieme_a_quatre_candidats() {
    let t = do_majeur();
    let g7 = accord(4, 7, Qualite::Septieme, 0);
    let tous = candidats(&g7, t);
    assert_eq!(tous.len(), 4);
    assert_eq!(hors_contexte(&g7, t), voicing(&[67, 71, 74, 77]));
    // Basse fa (r = 3) : fa 65, puis sol 67, si 71, ré 74.
    assert!(tous.contains(&voicing(&[65, 67, 71, 74])));
    // Basse si (r = 1) : si 71, ré 74, fa 77, sol 79.
    assert!(tous.contains(&voicing(&[71, 74, 77, 79])));
}

#[test]
fn candidats_en_position_serree() {
    for t in Tonalite::les_24() {
        let tab = table(t, CoucheSombre::Affichee);
        for case in &tab.cases {
            let a = &case.accord;
            let notes = a.notes();
            let tous = candidats(a, t);
            let attendu = if a.renversement > 0 { 1 } else { notes.len() };
            assert_eq!(tous.len(), attendu, "{} dans {}", tab.nom(case), t.nom());
            for v in &tous {
                assert_eq!(v.notes.len(), notes.len());
                assert!((60..=71).contains(&v.notes[0]));
                for paire in v.notes.windows(2) {
                    let ecart = paire[1] - paire[0];
                    assert!((1..=12).contains(&ecart), "pas au plus près : {v:?}");
                }
                let mut classes: Vec<u8> = v.notes.iter().map(|n| n % 12).collect();
                let mut voulues: Vec<u8> = notes
                    .iter()
                    .map(|&n| (t.tonique.classe() as i8 + n.demi_tons).rem_euclid(12) as u8)
                    .collect();
                classes.sort();
                voulues.sort();
                assert_eq!(classes, voulues, "{} dans {}", tab.nom(case), t.nom());
            }
            let bases: Vec<u8> = tous.iter().map(|v| v.notes[0]).collect();
            let mut sans_doublon = bases.clone();
            sans_doublon.sort();
            sans_doublon.dedup();
            assert_eq!(bases.len(), sans_doublon.len());
            let hors = hors_contexte(a, t);
            let classe_basse =
                (t.tonique.classe() as i8 + a.basse().demi_tons).rem_euclid(12) as u8;
            assert_eq!(hors.notes[0] % 12, classe_basse);
            assert!(tous.contains(&hors));
        }
    }
}

#[test]
fn en_contexte_minimal_et_basse_la_plus_grave() {
    for t in Tonalite::les_24() {
        let tab = table(t, CoucheSombre::Masquee);
        let mut precedent: Option<Voicing> = None;
        for case in tab.rangee(Rangee::Centre) {
            let choisi = en_contexte(precedent.as_ref(), &case.accord, t);
            if let Some(p) = &precedent {
                let tous = candidats(&case.accord, t);
                let d = deplacement(p, &choisi);
                for autre in &tous {
                    let da = deplacement(p, autre);
                    assert!(d <= da, "{} dans {}", tab.nom(case), t.nom());
                    if da == d {
                        assert!(
                            choisi.notes[0] <= autre.notes[0],
                            "{} dans {}",
                            tab.nom(case),
                            t.nom()
                        );
                    }
                }
            } else {
                assert_eq!(choisi, hors_contexte(&case.accord, t));
            }
            precedent = Some(choisi);
        }
    }
}
