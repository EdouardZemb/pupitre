use std::collections::HashSet;

use noyau::accord::{Accord, Qualite};
use noyau::guitare::{
    ACCORDAGE, Barre, Corde, Doigte, FORMES, Maillon, NotesVoulues, PAS_DE_DOIGTE, candidats,
    en_contexte, hors_contexte,
};
use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
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

/// La classe d'une note écrite `B♭`, `C♯`, `F` : une lettre, puis au besoin une altération.
fn classe_de_note(nom: &str) -> u8 {
    let mut lettres = nom.chars();
    let base: i8 = match lettres.next() {
        Some('C') => 0,
        Some('D') => 2,
        Some('E') => 4,
        Some('F') => 5,
        Some('G') => 7,
        Some('A') => 9,
        Some('B') => 11,
        autre => panic!("lettre illisible : {autre:?} dans {nom}"),
    };
    let alteration: i8 = match lettres.as_str() {
        "" => 0,
        "♭" => -1,
        "♯" => 1,
        reste => panic!("altération illisible : {reste} dans {nom}"),
    };
    (base + alteration).rem_euclid(12) as u8
}

/// Les notes voulues d'un nom d'accord : `B♭/D`, `C♯°7`, `F+`. Un oracle indépendant de
/// `NotesVoulues::de` : il part des intervalles de la qualité, sans tonalité.
fn voulues(nom: &str) -> NotesVoulues {
    let (accord, basse) = match nom.split_once('/') {
        Some((accord, basse)) => (accord, Some(basse)),
        None => (nom, None),
    };
    let fin_de_note = accord
        .char_indices()
        .nth(1)
        .map(|(k, c)| {
            if c == '♭' || c == '♯' {
                k + c.len_utf8()
            } else {
                k
            }
        })
        .unwrap_or(accord.len());
    let (note, suffixe) = accord.split_at(fin_de_note);
    let qualite = match suffixe {
        "" => Qualite::Majeur,
        "m" => Qualite::Mineur,
        "°" => Qualite::Diminue,
        "+" => Qualite::Augmente,
        "7" => Qualite::Septieme,
        "°7" => Qualite::Diminue7,
        autre => panic!("suffixe illisible : {autre} dans {nom}"),
    };
    let fondamentale = classe_de_note(note) as i8;
    let classes: Vec<u8> = qualite
        .intervalles()
        .iter()
        .map(|i| (fondamentale + i.demi_tons).rem_euclid(12) as u8)
        .collect();
    let basse = basse.map_or(classes[0], classe_de_note);
    assert!(classes.contains(&basse), "basse hors de l'accord : {nom}");
    let facultative = Some(classes[2]).filter(|&q| q != basse);
    NotesVoulues {
        basse,
        classes,
        facultative,
    }
}

fn doigte(notation: &str) -> Doigte {
    Doigte::depuis(notation).unwrap_or_else(|| panic!("doigté illisible : {notation}"))
}

/// Les classes qui sonnent, de la corde grave à l'aiguë.
fn sons(d: &Doigte) -> Vec<u8> {
    d.cordes
        .iter()
        .zip(ACCORDAGE)
        .filter_map(|(c, a)| match c {
            Corde::Etouffee => None,
            Corde::AVide => Some(a % 12),
            Corde::Frette(n) => Some((a + n) % 12),
        })
        .collect()
}

/// Les frettes appuyées, sans les cordes à vide.
fn appuyees(d: &Doigte) -> Vec<u8> {
    d.cordes
        .iter()
        .filter_map(|c| match c {
            Corde::Frette(n) => Some(*n),
            _ => None,
        })
        .collect()
}

/// Les cinq contraintes de REN-04, vérifiées sans le module.
fn tient_les_contraintes(d: &Doigte, v: &NotesVoulues) -> Result<(), String> {
    let s = sons(d);
    if let Some(n) = s.iter().find(|n| !v.classes.contains(n)) {
        return Err(format!("{} sonne {n}, hors de l'accord", d.notation()));
    }
    if s.first() != Some(&v.basse) {
        return Err(format!("{} : la basse n'est pas {}", d.notation(), v.basse));
    }
    for c in &v.classes {
        if Some(*c) != v.facultative && !s.contains(c) {
            return Err(format!("{} : il manque {c}", d.notation()));
        }
    }
    let f = appuyees(d);
    if let (Some(bas), Some(haut)) = (f.iter().min(), f.iter().max())
        && haut - bas > 3
    {
        return Err(format!("{} : étendue {}", d.notation(), haut - bas));
    }
    if d.nombre_de_doigts() > 4 {
        return Err(format!(
            "{} : {} doigts",
            d.notation(),
            d.nombre_de_doigts()
        ));
    }
    Ok(())
}

#[test]
fn numeros_de_doigts() {
    let c = doigte("x32010");
    assert_eq!(
        c.cordes,
        [
            Corde::Etouffee,
            Corde::Frette(3),
            Corde::Frette(2),
            Corde::AVide,
            Corde::Frette(1),
            Corde::AVide,
        ]
    );
    assert_eq!(c.barre, None);
    assert_eq!(c.doigts, [None, Some(3), Some(2), None, Some(1), None]);
    assert_eq!(c.nombre_de_doigts(), 3);

    let a = doigte("x02220");
    assert_eq!(a.barre, None);
    assert_eq!(a.doigts, [None, None, Some(1), Some(2), Some(3), None]);
    assert_eq!(a.nombre_de_doigts(), 3);

    let f = doigte("133211");
    assert_eq!(
        f.barre,
        Some(Barre {
            frette: 1,
            de: 0,
            a: 5
        })
    );
    assert_eq!(
        f.doigts,
        [Some(1), Some(3), Some(4), Some(2), Some(1), Some(1)]
    );
    assert_eq!(f.nombre_de_doigts(), 4);

    let bm = doigte("x24432");
    assert_eq!(
        bm.barre,
        Some(Barre {
            frette: 2,
            de: 1,
            a: 5
        })
    );
    assert_eq!(
        bm.doigts,
        [None, Some(1), Some(3), Some(4), Some(2), Some(1)]
    );
    assert_eq!(bm.nombre_de_doigts(), 4);
}

#[test]
fn quatre_cordes_appuyees_sans_barre() {
    // E♭ : quatre notes appuyées, quatre doigts, aucun barré même si deux sont à la même frette.
    let mib = doigte("xx1343");
    assert_eq!(mib.barre, None);
    assert_eq!(mib.doigts, [None, None, Some(1), Some(2), Some(4), Some(3)]);
    assert_eq!(mib.nombre_de_doigts(), 4);
    // Le barré couvre aussi les cordes à sa frette entre ses bornes.
    let sib7 = doigte("x13131");
    assert_eq!(
        sib7.barre,
        Some(Barre {
            frette: 1,
            de: 1,
            a: 5
        })
    );
    assert_eq!(
        sib7.doigts,
        [None, Some(1), Some(2), Some(1), Some(3), Some(1)]
    );
    assert_eq!(sib7.nombre_de_doigts(), 3);
    // Un barré qui ne couvre pas tout le manche : de la corde 2 à la corde 4.
    let reb_fa = doigte("x-8-6-6-6-9");
    assert_eq!(
        reb_fa.barre,
        Some(Barre {
            frette: 6,
            de: 2,
            a: 4
        })
    );
    assert_eq!(
        reb_fa.doigts,
        [None, Some(2), Some(1), Some(1), Some(1), Some(3)]
    );
    assert_eq!(reb_fa.nombre_de_doigts(), 3);
}

#[test]
fn un_doigte_irrealisable_est_refuse() {
    // Cinq cordes appuyées : le barré est obligé, mais une corde étouffée ou à vide entre ses
    // bornes l'interdit ; une seule corde à la frette la plus basse aussi.
    for n in ["1x1111", "102111", "123456", "11x111", "2x2232"] {
        assert_eq!(Doigte::depuis(n), None, "{n}");
    }
    // Un barré et quatre autres doigts : cinq doigts.
    assert_eq!(Doigte::depuis("134221"), None);
    // Un barré et trois autres doigts : quatre doigts, réalisable.
    assert_eq!(doigte("134211").nombre_de_doigts(), 4);
}

#[test]
fn une_notation_illisible_est_refusee() {
    for n in [
        "",
        "x3201",
        "x320100",
        "y32010",
        "X32010",
        "x-8-10",
        "x-8-a-10-10-8",
        "x-8-10-10-10-8-0",
        "x--10-10-10-8",
        "x-8-10-10-10-300",
    ] {
        assert_eq!(Doigte::depuis(n), None, "{n:?}");
    }
}

#[test]
fn la_notation_aller_retour() {
    let haut = doigte("x-8-10-10-10-8");
    assert_eq!(
        haut.cordes,
        [
            Corde::Etouffee,
            Corde::Frette(8),
            Corde::Frette(10),
            Corde::Frette(10),
            Corde::Frette(10),
            Corde::Frette(8),
        ]
    );
    assert_eq!(haut.notation(), "x-8-10-10-10-8");
    assert_eq!(doigte("x-3-2-0-1-0").notation(), "x32010");
    assert_eq!(doigte("x-3-2-0-1-0"), doigte("x32010"));
    assert_eq!(doigte("9-11-11-10-9-9").notation(), "9-11-11-10-9-9");
    assert_eq!(doigte("xx7565").notation(), "xx7565");
    assert_eq!(doigte("x-9-9-x-x-x").notation(), "x99xxx");
}

#[test]
fn la_zone() {
    assert_eq!(doigte("x32010").zone(), 0);
    assert_eq!(doigte("xx0232").zone(), 0);
    assert_eq!(doigte("x-8-10-10-10-8").zone(), 8);
    assert_eq!(doigte("133211").zone(), 1);
    assert_eq!(doigte("xx7565").zone(), 5);
    assert_eq!(doigte("xxx000").zone(), 0);
}

#[test]
fn les_notes_voulues_d_un_accord() {
    let t = do_majeur();
    let sol7 = Accord {
        fondamentale: Intervalle::new(4, 7),
        qualite: Qualite::Septieme,
        renversement: 0,
    };
    assert_eq!(
        NotesVoulues::de(&sol7, t),
        NotesVoulues {
            basse: 7,
            classes: vec![7, 11, 2, 5],
            facultative: Some(2),
        }
    );
    // La napolitaine, D♭/F : la tierce à la basse, la quinte reste facultative.
    let napolitaine = Accord {
        fondamentale: Intervalle::new(1, 1),
        qualite: Qualite::Majeur,
        renversement: 1,
    };
    assert_eq!(NotesVoulues::de(&napolitaine, t), voulues("D♭/F"));
    assert_eq!(voulues("D♭/F").facultative, Some(8));
    // La quinte à la basse n'est plus facultative.
    let quarte_et_sixte = Accord {
        fondamentale: Intervalle::new(0, 0),
        qualite: Qualite::Majeur,
        renversement: 2,
    };
    assert_eq!(
        NotesVoulues::de(&quarte_et_sixte, t),
        NotesVoulues {
            basse: 7,
            classes: vec![0, 4, 7],
            facultative: None,
        }
    );
    // Dans une autre tonalité, les classes sont absolues : le V de mi♭ mineur est B♭m.
    let mib_mineur = Tonalite::les_24()[13];
    let cinquieme = Accord {
        fondamentale: Intervalle::new(4, 7),
        qualite: Qualite::Mineur,
        renversement: 0,
    };
    assert_eq!(NotesVoulues::de(&cinquieme, mib_mineur), voulues("B♭m"));
}

#[test]
fn la_table_des_formes() {
    assert_eq!(FORMES.len(), 27);
    let mobiles = FORMES.iter().filter(|f| f.mobile).count();
    assert_eq!(mobiles, 18);
    let e = FORMES.iter().find(|f| f.nom == "E").expect("la forme E");
    assert_eq!(
        e.frettes,
        [Some(0), Some(2), Some(2), Some(1), Some(0), Some(0)]
    );
    assert!(e.mobile);
    let c = FORMES.iter().find(|f| f.nom == "C").expect("la forme C");
    assert_eq!(
        c.frettes,
        [None, Some(3), Some(2), Some(0), Some(1), Some(0)]
    );
    assert!(!c.mobile);
}

#[test]
fn doigtes_de_reference() {
    for (accord, attendu) in [
        ("C", "x32010"),
        ("A", "x02220"),
        ("G", "320003"),
        ("E", "022100"),
        ("D", "xx0232"),
        ("F", "133211"),
        ("B♭", "x13331"),
        ("B", "x24442"),
        ("E♭", "xx1343"),
        ("A♭", "466544"),
        ("D♭", "x46664"),
        ("F♯", "244322"),
        ("Am", "x02210"),
        ("Em", "022000"),
        ("Dm", "xx0231"),
        ("Bm", "x24432"),
        ("F♯m", "244222"),
        ("C♯m", "x46654"),
        ("G♯m", "466444"),
        ("Cm", "x35543"),
        ("Gm", "355333"),
        ("Fm", "133111"),
        ("B♭m", "x13321"),
        ("E♭m", "xx1342"),
        ("C7", "x32310"),
        ("A7", "x02020"),
        ("G7", "320001"),
        ("E7", "020100"),
        ("D7", "xx0212"),
        ("B7", "x21202"),
        ("F7", "131211"),
        ("B♭7", "x13131"),
        ("F♯7", "242322"),
        ("C♯7", "x46464"),
        ("G♯7", "464544"),
        ("D♯7", "xx1323"),
        ("B°", "x2343x"),
        ("C♯°", "x4565x"),
        ("F♯°", "xx4575"),
        ("E°", "xx2353"),
        ("C♯°7", "x4535x"),
        ("G♯°7", "456464"),
        ("B°7", "x2313x"),
        ("F°7", "123131"),
        ("C+", "x32110"),
        ("E+", "032110"),
        ("F+", "xx3665"),
        ("D♭/F", "xx3121"),
        ("B♭/D", "xx0331"),
        ("F/A", "x03211"),
        ("C/E", "xx2010"),
        ("G/B", "x20003"),
    ] {
        let choisi = hors_contexte(&voulues(accord)).unwrap_or_else(|| panic!("{accord} : aucun"));
        assert_eq!(choisi.notation(), attendu, "{accord}");
    }
}

#[test]
fn les_candidats_sont_ranges_par_le_departage() {
    let c = voulues("C");
    let tous = candidats(&c);
    let notations: Vec<String> = tous.iter().map(Doigte::notation).collect();
    assert_eq!(
        notations,
        ["x32010", "x35553", "8-10-10-9-8-8", "x-x-10-12-13-12"]
    );
    assert_eq!(hors_contexte(&c), tous.first().cloned());
    // Chaque candidat tient les contraintes, et aucun n'est en double.
    for v in [
        voulues("C"),
        voulues("G♯°7"),
        voulues("D♭/F"),
        voulues("E+"),
    ] {
        let tous = candidats(&v);
        assert!(!tous.is_empty());
        for d in &tous {
            tient_les_contraintes(d, &v).unwrap();
        }
        let distincts: HashSet<&Doigte> = tous.iter().collect();
        assert_eq!(distincts.len(), tous.len());
    }
}

#[test]
fn ren04_1_le_contexte_change_le_doigte() {
    let (g, c, d) = (voulues("G"), voulues("C"), voulues("D"));
    let premier = en_contexte(&[
        Maillon::Libre(g),
        Maillon::Libre(c.clone()),
        Maillon::Libre(d),
    ]);
    let notations: Vec<String> = premier
        .iter()
        .map(|x| x.as_ref().expect("un doigté").notation())
        .collect();
    assert_eq!(notations, ["320003", "x32010", "xx0232"]);

    let fa_haut = doigte("x-8-10-10-10-8");
    let sol_haut = doigte("x-10-12-12-12-10");
    let second = en_contexte(&[
        Maillon::Fixe(fa_haut.clone()),
        Maillon::Libre(c.clone()),
        Maillon::Fixe(sol_haut.clone()),
    ]);
    assert_eq!(second[0], Some(fa_haut.clone()));
    assert_eq!(second[2], Some(sol_haut.clone()));
    let do_haut = second[1].clone().expect("un doigté pour C");
    assert_eq!(do_haut.notation(), "8-10-10-9-8-8");
    assert_ne!(Some(&do_haut), premier[1].as_ref());

    // Recherche exhaustive : aucun autre candidat de C ne déplace moins la main.
    let total = |x: &Doigte| fa_haut.zone().abs_diff(x.zone()) + x.zone().abs_diff(sol_haut.zone());
    let tous = candidats(&c);
    assert!(tous.contains(&do_haut));
    for autre in &tous {
        assert!(
            total(&do_haut) <= total(autre),
            "{} déplace moins",
            autre.notation()
        );
    }
    assert_eq!(total(&do_haut), 2);
}

#[test]
fn ren04_2_tout_doigte_propose_tient_les_contraintes() {
    for t in Tonalite::les_24() {
        let tab = table(t, CoucheSombre::Affichee);
        for case in &tab.cases {
            let v = NotesVoulues::de(&case.accord, t);
            let d = hors_contexte(&v).unwrap_or_else(|| panic!("{} : aucun", tab.nom(case)));
            if let Err(e) = tient_les_contraintes(&d, &v) {
                panic!("{} dans {} : {e}", tab.nom(case), t.nom());
            }
        }
        // En contexte aussi, sur toute la table lue dans l'ordre.
        let suite: Vec<Maillon> = tab
            .cases
            .iter()
            .map(|case| Maillon::Libre(NotesVoulues::de(&case.accord, t)))
            .collect();
        let chemin = en_contexte(&suite);
        assert_eq!(chemin.len(), suite.len());
        for (case, d) in tab.cases.iter().zip(&chemin) {
            let v = NotesVoulues::de(&case.accord, t);
            let d = d
                .as_ref()
                .unwrap_or_else(|| panic!("{} : aucun", tab.nom(case)));
            if let Err(e) = tient_les_contraintes(d, &v) {
                panic!("{} dans {}, en contexte : {e}", tab.nom(case), t.nom());
            }
        }
    }
}

#[test]
fn chaque_accord_des_24_tables_a_un_doigte() {
    let mut distincts: HashSet<NotesVoulues> = HashSet::new();
    for t in Tonalite::les_24() {
        let tab = table(t, CoucheSombre::Affichee);
        for case in &tab.cases {
            distincts.insert(NotesVoulues::de(&case.accord, t));
        }
    }
    assert_eq!(distincts.len(), 84);
    for v in &distincts {
        assert!(hors_contexte(v).is_some(), "{v:?} : pas de doigté");
    }
}

#[test]
fn ren04_5_un_agregat_serre_n_a_pas_de_doigte() {
    let agregat = NotesVoulues {
        basse: 0,
        classes: vec![0, 1, 2, 3, 4],
        facultative: None,
    };
    assert!(candidats(&agregat).is_empty());
    assert_eq!(hors_contexte(&agregat), None);
    assert_eq!(PAS_DE_DOIGTE, "pas de doigté jouable");
}

#[test]
fn le_chemin_saute_un_accord_sans_doigte() {
    let agregat = NotesVoulues {
        basse: 0,
        classes: vec![0, 1, 2, 3, 4],
        facultative: None,
    };
    let fa_haut = doigte("x-8-10-10-10-8");
    let chemin = en_contexte(&[
        Maillon::Fixe(fa_haut.clone()),
        Maillon::Libre(agregat.clone()),
        Maillon::Libre(voulues("C")),
    ]);
    assert_eq!(chemin[0], Some(fa_haut));
    assert_eq!(chemin[1], None);
    // C se règle sur le fa haut, par-dessus l'agrégat.
    assert_eq!(chemin[2].as_ref().unwrap().notation(), "8-10-10-9-8-8");
    // En tête de suite, l'agrégat n'empêche pas le doigté hors contexte.
    let chemin = en_contexte(&[Maillon::Libre(agregat), Maillon::Libre(voulues("C"))]);
    assert_eq!(chemin[0], None);
    assert_eq!(chemin[1].as_ref().unwrap().notation(), "x32010");
    assert!(en_contexte(&[]).is_empty());
}

#[test]
fn seul_hors_contexte_sans_voisin() {
    for nom in ["C", "F", "B°", "G♯°7", "D♭/F"] {
        let v = voulues(nom);
        assert_eq!(
            en_contexte(&[Maillon::Libre(v.clone())]),
            vec![hors_contexte(&v)],
            "{nom}"
        );
    }
}
