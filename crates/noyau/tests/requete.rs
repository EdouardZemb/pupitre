//! Les jours civils et les bornes d'une période (question 9 ; CAP-06).

use noyau::donnees::requete::{Bornes, Jour, bornes_de_la_periode, debut_du_jour};

fn j(texte: &str) -> Jour {
    Jour::depuis_iso(texte).unwrap()
}

/// Le décalage de Paris en 2026, écrit à la main : l'heure d'été va du 29 mars à 1 h UTC au
/// 25 octobre à 1 h UTC.
fn paris(t: i64) -> i32 {
    if (1_774_746_000..1_792_890_000).contains(&t) {
        120
    } else {
        60
    }
}

#[test]
fn un_jour_se_lit_et_s_ecrit_en_iso() {
    assert_eq!(
        Jour::depuis_iso("2026-10-25"),
        Some(Jour {
            annee: 2026,
            mois: 10,
            jour: 25
        })
    );
    assert_eq!(j("2024-02-29").en_iso(), "2024-02-29");
    assert!(Jour::depuis_iso("2000-02-29").is_some());
    for faux in [
        "2026-02-29",
        "1900-02-29",
        "2026-13-01",
        "2026-00-10",
        "2026-04-31",
        "2026-10-00",
        "2026-4-1",
        "26-10-25",
        "2026-10-25 ",
        "2026/10/25",
        "",
    ] {
        assert_eq!(Jour::depuis_iso(faux), None, "{faux}");
    }
}

#[test]
fn minuit_utc_compte_les_jours_depuis_1970() {
    assert_eq!(j("1970-01-01").minuit_utc(), 0);
    assert_eq!(j("1969-12-31").minuit_utc(), -86_400);
    assert_eq!(j("2000-02-29").minuit_utc(), 951_782_400);
    assert_eq!(j("2000-03-01").minuit_utc(), 951_868_800);
    assert_eq!(j("2026-10-25").minuit_utc(), 1_792_886_400);
    assert_eq!(j("2027-01-01").minuit_utc(), 1_798_761_600);
}

#[test]
fn les_bornes_d_un_jour_de_25_heures_sont_justes() {
    // Le 25 octobre 2026 à Paris : de 0 h, heure d'été (24 octobre, 22 h UTC), à 0 h le 26, heure
    // d'hiver (25 octobre, 23 h UTC) : 25 heures.
    let b = bornes_de_la_periode(Some(j("2026-10-25")), Some(j("2026-10-25")), paris);
    assert_eq!(
        b,
        Bornes {
            des: Some(1_792_879_200),
            avant: Some(1_792_969_200)
        }
    );
    // A, notée à 0 h 30, heure d'été ; B, à 23 h 30, heure d'hiver : du 25, comme la liste les date ;
    // C, le 26 à 0 h 30 : non. Un seul décalage pour le jour entier perdrait A ou B (LE-6, L7).
    assert!(b.contient(1_792_881_000));
    assert!(b.contient(1_792_967_400));
    assert!(!b.contient(1_792_971_000));
}

#[test]
fn les_bornes_d_un_jour_de_23_heures_sont_justes() {
    let b = bornes_de_la_periode(Some(j("2026-03-29")), Some(j("2026-03-29")), paris);
    assert_eq!(
        b,
        Bornes {
            des: Some(1_774_738_800),
            avant: Some(1_774_821_600)
        }
    );
}

#[test]
fn une_borne_absente_ne_borne_pas() {
    assert_eq!(bornes_de_la_periode(None, None, paris), Bornes::default());
    let des_le_25 = bornes_de_la_periode(Some(j("2026-10-25")), None, paris);
    assert_eq!(
        des_le_25,
        Bornes {
            des: Some(1_792_879_200),
            avant: None
        }
    );
    assert!(des_le_25.contient(i64::MAX));
    assert!(!des_le_25.contient(1_792_879_199));
    assert!(Bornes::default().contient(i64::MIN));
}

#[test]
fn les_bornes_sont_comprises_puis_exclues() {
    let b = Bornes {
        des: Some(100),
        avant: Some(200),
    };
    assert!(!b.contient(99));
    assert!(b.contient(100));
    assert!(b.contient(199));
    assert!(!b.contient(200));
    let jusque = Bornes {
        des: None,
        avant: Some(200),
    };
    assert!(jusque.contient(i64::MIN));
    assert!(jusque.contient(199));
    assert!(!jusque.contient(200));
    let seul_au = bornes_de_la_periode(None, Some(j("2026-10-25")), paris);
    assert_eq!(
        seul_au,
        Bornes {
            des: None,
            avant: Some(1_792_969_200)
        }
    );
}

#[test]
fn chaque_mois_a_son_dernier_jour() {
    let longueurs = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, dernier) in longueurs.iter().enumerate() {
        let mois = i + 1;
        assert!(
            Jour::depuis_iso(&format!("2026-{mois:02}-{dernier:02}")).is_some(),
            "{mois}"
        );
        assert_eq!(
            Jour::depuis_iso(&format!("2026-{mois:02}-{:02}", dernier + 1)),
            None,
            "{mois}"
        );
        assert!(
            Jour::depuis_iso(&format!("2026-{mois:02}-01")).is_some(),
            "{mois}"
        );
        assert_eq!(
            Jour::depuis_iso(&format!("2026-{mois:02}-00")),
            None,
            "{mois}"
        );
    }
}

#[test]
fn les_annees_bissextiles_ont_un_29_fevrier() {
    for oui in ["2024-02-29", "2000-02-29", "1996-02-29", "2400-02-29"] {
        assert!(Jour::depuis_iso(oui).is_some(), "{oui}");
    }
    for non in [
        "2023-02-29",
        "2026-02-29",
        "1900-02-29",
        "2100-02-29",
        "2025-02-29",
        "2022-02-29",
    ] {
        assert_eq!(Jour::depuis_iso(non), None, "{non}");
    }
    assert_eq!(Jour::depuis_iso("2024-02-30"), None);
}

#[test]
fn les_formes_voisines_sont_refusees() {
    for faux in [
        "2026-10-2",
        "2026-1-25",
        "20261-0-25",
        "2026-10-255",
        "+026-10-25",
        "2026-1a-25",
        "2026-10-2x",
        "202a-10-25",
        "2026_10_25",
        "2026-10_25",
        "2026_10-25",
        "-026-10-25",
        "2026-10-25\n",
        " 2026-10-25",
    ] {
        assert_eq!(Jour::depuis_iso(faux), None, "{faux}");
    }
}

#[test]
fn en_iso_ecrit_l_annee_sur_quatre_chiffres() {
    assert_eq!(j("0999-01-05").en_iso(), "0999-01-05");
    assert_eq!(j("0001-12-31").en_iso(), "0001-12-31");
    assert_eq!(j("2026-03-07").en_iso(), "2026-03-07");
}

#[test]
fn minuit_utc_avance_de_86_400_secondes_d_un_jour_a_l_autre() {
    let longueurs = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut precedent: Option<i64> = None;
    for annee in 1896..=2104 {
        let bissextile = (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0;
        for (i, base) in longueurs.iter().enumerate() {
            let dernier = if i == 1 && bissextile { 29 } else { *base };
            for jour in 1..=dernier {
                let minuit = j(&format!("{annee:04}-{:02}-{jour:02}", i + 1)).minuit_utc();
                if let Some(avant) = precedent {
                    assert_eq!(minuit - avant, 86_400, "{annee}-{}-{jour}", i + 1);
                }
                precedent = Some(minuit);
            }
        }
    }
    assert_eq!(j("2000-01-01").minuit_utc(), 946_684_800);
    assert_eq!(j("2000-12-31").minuit_utc(), 978_220_800);
}

#[test]
fn le_debut_du_jour_suit_le_decalage_du_moment() {
    // Sans décalage, le début du jour est son minuit UTC.
    assert_eq!(debut_du_jour(j("2026-10-25"), |_| 0), 1_792_886_400);
    // Un décalage constant de 60 minutes recule le début d'une heure.
    assert_eq!(
        debut_du_jour(j("2026-10-25"), |_| 60),
        1_792_886_400 - 3_600
    );
    assert_eq!(debut_du_jour(j("2026-10-25"), paris), 1_792_879_200);
    assert_eq!(debut_du_jour(j("2026-10-26"), paris), 1_792_969_200);
}
