//! Les jours civils et les bornes d'une période (question 9), la requête sans critère, et le texte
//! de la réponse d'une requête (question 7) ; CAP-06.

use std::collections::BTreeMap;

use noyau::donnees::boite::Contenu;
use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::gestes::{ENTIER_SUR, Impossible};
use noyau::donnees::idee::Assiette;
use noyau::donnees::requete::{
    Bornes, Etat, Jour, ProvenanceDeLIdee, Requete, RequeteEnregistreeLue, Sans,
    bornes_de_la_periode, debut_du_jour, enregistrer_une_requete, lire_une_requete,
    ranger_les_requetes, renommer_une_requete, supprimer_une_requete, texte_de_la_reponse,
};

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

#[test]
fn le_debut_du_jour_relit_le_decalage_a_l_instant_approche() {
    // Un décalage de 60 minutes dès M − 1 800 s : à minuit UTC (M) il vaut 60, et l'instant
    // approché, M − 3 600 s, tombe avant le changement, où il vaut 0 : le début du jour est M.
    let m = j("2026-10-25").minuit_utc();
    let decalage = |t: i64| if t >= m - 1_800 { 60 } else { 0 };
    assert_eq!(debut_du_jour(j("2026-10-25"), decalage), m);
}

#[test]
fn le_texte_de_la_reponse_ordonne_ses_parts_et_s_accorde() {
    let a = |c, p| Assiette {
        considerees: c,
        portant: p,
        decidees: p,
    };
    assert_eq!(
        texte_de_la_reponse(0, a(4, 4), 0, Sans::default()),
        "0 idée · 4 interrogées"
    );
    assert_eq!(
        texte_de_la_reponse(1, a(1, 1), 0, Sans::default()),
        "1 idée · 1 interrogée"
    );
    assert_eq!(
        texte_de_la_reponse(1, a(4, 4), 0, Sans::default()),
        "1 idée · 4 interrogées"
    );
    assert_eq!(
        texte_de_la_reponse(2, a(5, 4), 1, Sans::default()),
        "2 idées · 1 illisible · 5 interrogées"
    );
    let toutes = Sans {
        etiquette: Some(1),
        duree: Some(1),
        texte: Some(2),
    };
    assert_eq!(
        texte_de_la_reponse(1, a(6, 1), 1, toutes),
        "1 idée · 1 sans étiquette · 1 sans durée · 2 sans texte · 1 illisible · 6 interrogées"
    );
    assert_eq!(
        texte_de_la_reponse(
            2,
            a(5, 3),
            2,
            Sans {
                duree: Some(0),
                ..Sans::default()
            }
        ),
        "2 idées · 2 illisibles · 5 interrogées"
    );
}

#[test]
fn chaque_critere_pose_seul_rend_la_requete_non_vide() {
    // Un critère, un seul, posé sur la requête par défaut : un mot, un jour de début, un jour de
    // fin, une étiquette, un seuil, un état, une provenance, un contenu.
    let jour = Jour::depuis_iso("2026-10-25");
    let seuls = [
        Requete {
            texte: "eglise".to_string(),
            ..Requete::default()
        },
        Requete {
            du: jour,
            ..Requete::default()
        },
        Requete {
            au: jour,
            ..Requete::default()
        },
        Requete {
            etiquette: Some("pont".to_string()),
            ..Requete::default()
        },
        Requete {
            plus_de_secondes: Some(60),
            ..Requete::default()
        },
        Requete {
            etat: Some(Etat::NonReliee),
            ..Requete::default()
        },
        Requete {
            provenance: Some(ProvenanceDeLIdee::Captee),
            ..Requete::default()
        },
        Requete {
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        },
    ];
    for requete in seuls {
        assert!(!requete.est_vide(), "{requete:?}");
    }
    assert!(Requete::default().est_vide());
}

fn requete_complete() -> Requete {
    Requete {
        texte: "eglise".to_string(),
        du: Some(j("2026-10-25")),
        au: Some(j("2026-11-02")),
        etiquette: Some("pont".to_string()),
        plus_de_secondes: Some(60),
        etat: Some(Etat::NonReliee),
        provenance: Some(ProvenanceDeLIdee::Importee),
        contenu: Some(Contenu::MidiVerse),
    }
}

const ID: &str = "01JREQUETE0000000000000001";

fn poser(cle: &str, valeur: Simple) -> Changement {
    Changement::Poser {
        chemin: vec![
            "requetes_enregistrees".to_string(),
            ID.to_string(),
            cle.to_string(),
        ],
        valeur,
    }
}

fn chaine(texte: &str) -> Simple {
    Simple::Chaine(texte.to_string())
}

/// Les `Poser` d'une opération d'enregistrement, mis en `champs` comme le fait la lecture du document.
fn champs_de(operation: &Operation) -> BTreeMap<String, Option<Simple>> {
    operation
        .changements
        .iter()
        .filter_map(|changement| match changement {
            Changement::Poser { chemin, valeur } => Some((chemin[2].clone(), Some(valeur.clone()))),
            _ => None,
        })
        .collect()
}

fn lue(nom: &str) -> RequeteEnregistreeLue {
    RequeteEnregistreeLue {
        id: ID.to_string(),
        nom: nom.to_string(),
        requete: None,
    }
}

#[test]
fn enregistrer_cree_l_objet_avec_son_nom_et_une_cle_par_critere() {
    let requete = Requete {
        texte: "eglise".to_string(),
        du: Some(j("2026-10-25")),
        plus_de_secondes: Some(60),
        contenu: Some(Contenu::Prise),
        ..Requete::default()
    };
    let operation = enregistrer_une_requete(ID, "Ponts lents", &requete, 1_794_906_000).unwrap();
    assert_eq!(
        operation.changements,
        vec![
            Changement::Creer {
                chemin: vec!["requetes_enregistrees".to_string(), ID.to_string()],
                conteneurs: Vec::new(),
            },
            poser("nom", chaine("Ponts lents")),
            poser("texte", chaine("eglise")),
            poser("du", chaine("2026-10-25")),
            poser("plus_de_secondes", Simple::Naturel(60)),
            poser("contenu", chaine("prise")),
        ]
    );
    assert_eq!(operation.message, "Requête « Ponts lents » enregistrée");
    assert_eq!(operation.heure, 1_794_906_000);
    // Le nom se range sans ses blancs ; un texte sans mot ne pose pas de clé.
    let sans_mot = Requete {
        texte: "'".to_string(),
        au: Some(j("2026-11-02")),
        ..Requete::default()
    };
    let operation = enregistrer_une_requete(ID, "  Ponts lents\t", &sans_mot, 1).unwrap();
    assert_eq!(
        operation.changements[1..],
        [
            poser("nom", chaine("Ponts lents")),
            poser("au", chaine("2026-11-02")),
        ]
    );
    assert_eq!(operation.message, "Requête « Ponts lents » enregistrée");
}

#[test]
fn un_nom_vide_ou_une_requete_sans_critere_est_refuse() {
    let avec_texte = Requete {
        texte: "eglise".to_string(),
        ..Requete::default()
    };
    assert_eq!(
        enregistrer_une_requete(ID, "\u{a0}", &avec_texte, 1),
        Err(Impossible::NomVide)
    );
    assert_eq!(
        enregistrer_une_requete(ID, "", &Requete::default(), 1),
        Err(Impossible::NomVide)
    );
    for sans_critere in [
        Requete::default(),
        Requete {
            texte: "'".to_string(),
            ..Requete::default()
        },
    ] {
        assert_eq!(
            enregistrer_une_requete(ID, "Ponts", &sans_critere, 1),
            Err(Impossible::RequeteSansCritere)
        );
    }
    let trop_long = Requete {
        plus_de_secondes: Some(ENTIER_SUR + 1),
        ..Requete::default()
    };
    assert_eq!(
        enregistrer_une_requete(ID, "Ponts", &trop_long, 1),
        Err(Impossible::SeuilTropLong {
            secondes: ENTIER_SUR + 1
        })
    );
    let au_plus_long = Requete {
        plus_de_secondes: Some(ENTIER_SUR),
        ..Requete::default()
    };
    assert!(enregistrer_une_requete(ID, "Ponts", &au_plus_long, 1).is_ok());
    assert_eq!(Impossible::NomVide.to_string(), "Le nom est vide.");
    assert_eq!(Impossible::MemeNom.to_string(), "Le nom est déjà celui-ci.");
    assert_eq!(
        Impossible::RequeteSansCritere.to_string(),
        "La requête n'a aucun critère."
    );
    assert_eq!(
        Impossible::RequeteAbsente {
            requete: ID.to_string()
        }
        .to_string(),
        "Aucune requête enregistrée ne porte cet identifiant."
    );
    assert_eq!(
        Impossible::SeuilTropLong { secondes: 1 }.to_string(),
        "Ce seuil de durée est trop long."
    );
}

#[test]
fn renommer_et_supprimer_disent_le_nom() {
    let ponts = lue("Ponts lents");
    let renommee = renommer_une_requete(&ponts, "Ponts du soir", 5).unwrap();
    assert_eq!(
        renommee.changements,
        vec![poser("nom", chaine("Ponts du soir"))]
    );
    assert_eq!(
        renommee.message,
        "Requête « Ponts lents » renommée « Ponts du soir »"
    );
    assert_eq!(renommee.heure, 5);
    let blancs = renommer_une_requete(&ponts, " Ponts du soir ", 5).unwrap();
    assert_eq!(blancs, renommee);
    assert_eq!(
        renommer_une_requete(&ponts, "Ponts lents", 5),
        Err(Impossible::MemeNom)
    );
    assert_eq!(
        renommer_une_requete(&ponts, " Ponts lents ", 5),
        Err(Impossible::MemeNom)
    );
    assert_eq!(
        renommer_une_requete(&ponts, "", 5),
        Err(Impossible::NomVide)
    );
    let supprimee = supprimer_une_requete(&ponts, 6);
    assert_eq!(
        supprimee.changements,
        vec![Changement::Oter {
            chemin: vec!["requetes_enregistrees".to_string(), ID.to_string()],
        }]
    );
    assert_eq!(supprimee.message, "Requête « Ponts lents » supprimée");
    assert_eq!(supprimee.heure, 6);
}

#[test]
fn une_requete_se_relit_telle_qu_elle_s_est_enregistree() {
    let complete = requete_complete();
    let mut requetes = vec![complete.clone()];
    // Chaque critère seul.
    requetes.push(Requete {
        texte: complete.texte.clone(),
        ..Requete::default()
    });
    for seul in [
        Requete {
            du: complete.du,
            ..Requete::default()
        },
        Requete {
            au: complete.au,
            ..Requete::default()
        },
        Requete {
            etiquette: complete.etiquette.clone(),
            ..Requete::default()
        },
        Requete {
            plus_de_secondes: complete.plus_de_secondes,
            ..Requete::default()
        },
        Requete {
            plus_de_secondes: Some(ENTIER_SUR),
            ..Requete::default()
        },
    ] {
        requetes.push(seul);
    }
    // Chaque valeur d'énumération.
    for etat in [Etat::Reliee, Etat::NonReliee] {
        requetes.push(Requete {
            etat: Some(etat),
            ..Requete::default()
        });
    }
    for provenance in [ProvenanceDeLIdee::Captee, ProvenanceDeLIdee::Importee] {
        requetes.push(Requete {
            provenance: Some(provenance),
            ..Requete::default()
        });
    }
    for contenu in [
        Contenu::TexteDeLIdee,
        Contenu::Prise,
        Contenu::Photo,
        Contenu::MidiVerse,
        Contenu::TexteVerse,
    ] {
        requetes.push(Requete {
            contenu: Some(contenu),
            ..Requete::default()
        });
    }
    for requete in requetes {
        let operation = enregistrer_une_requete(ID, "Ponts lents", &requete, 1).unwrap();
        let relue = lire_une_requete(ID, &champs_de(&operation)).unwrap();
        assert_eq!(relue.requete, Some(requete.clone()), "{requete:?}");
        assert_eq!(relue.nom, "Ponts lents");
        assert_eq!(relue.id, ID);
    }
    // Les chaînes que le document porte, valeur par valeur.
    let operation = enregistrer_une_requete(ID, "Ponts lents", &complete, 1).unwrap();
    for attendu in [
        poser("etat", chaine("non_reliee")),
        poser("provenance", chaine("importee")),
        poser("contenu", chaine("midi_verse")),
        poser("etiquette", chaine("pont")),
    ] {
        assert!(operation.changements.contains(&attendu), "{attendu:?}");
    }
    for (etat, chaine_attendue) in [(Etat::Reliee, "reliee"), (Etat::NonReliee, "non_reliee")] {
        let requete = Requete {
            etat: Some(etat),
            ..Requete::default()
        };
        let operation = enregistrer_une_requete(ID, "x", &requete, 1).unwrap();
        assert!(
            operation
                .changements
                .contains(&poser("etat", chaine(chaine_attendue)))
        );
    }
    for (provenance, chaine_attendue) in [
        (ProvenanceDeLIdee::Captee, "captee"),
        (ProvenanceDeLIdee::Importee, "importee"),
    ] {
        let requete = Requete {
            provenance: Some(provenance),
            ..Requete::default()
        };
        let operation = enregistrer_une_requete(ID, "x", &requete, 1).unwrap();
        assert!(
            operation
                .changements
                .contains(&poser("provenance", chaine(chaine_attendue)))
        );
    }
    for (contenu, chaine_attendue) in [
        (Contenu::TexteDeLIdee, "texte_de_l_idee"),
        (Contenu::Prise, "prise"),
        (Contenu::Photo, "photo"),
        (Contenu::MidiVerse, "midi_verse"),
        (Contenu::TexteVerse, "texte_verse"),
    ] {
        let requete = Requete {
            contenu: Some(contenu),
            ..Requete::default()
        };
        let operation = enregistrer_une_requete(ID, "x", &requete, 1).unwrap();
        assert!(
            operation
                .changements
                .contains(&poser("contenu", chaine(chaine_attendue)))
        );
    }
}

#[test]
fn une_cle_inconnue_rend_la_requete_d_une_version_plus_recente() {
    let base = || -> BTreeMap<String, Option<Simple>> {
        BTreeMap::from([
            ("nom".to_string(), Some(chaine("Ponts lents"))),
            ("texte".to_string(), Some(chaine("eglise"))),
        ])
    };
    let lue_ok = lire_une_requete(ID, &base()).unwrap();
    assert_eq!(
        lue_ok.requete,
        Some(Requete {
            texte: "eglise".to_string(),
            ..Requete::default()
        })
    );
    let variantes: Vec<(&str, Option<Simple>)> = vec![
        ("tonalite", Some(chaine("do"))),
        ("contenu", Some(chaine("video"))),
        ("etat", Some(chaine("autre"))),
        ("provenance", Some(chaine("autre"))),
        ("etat", Some(Simple::Naturel(1))),
        ("texte", Some(Simple::Naturel(1))),
        ("du", Some(chaine("2026-02-30"))),
        ("au", Some(chaine("2026-13-01"))),
        ("du", Some(Simple::Naturel(1))),
        ("etiquette", Some(Simple::Naturel(1))),
        ("plus_de_secondes", Some(Simple::Naturel(ENTIER_SUR + 1))),
        ("plus_de_secondes", Some(chaine("60"))),
        ("plus_de_secondes", None),
    ];
    for (cle, valeur) in variantes {
        let mut champs = base();
        champs.insert(cle.to_string(), valeur.clone());
        let relue = lire_une_requete(ID, &champs).unwrap();
        assert_eq!(relue.requete, None, "{cle} {valeur:?}");
        assert_eq!(relue.nom, "Ponts lents");
    }
    let mut sans_nom = base();
    sans_nom.remove("nom");
    assert_eq!(lire_une_requete(ID, &sans_nom), None);
    let mut nom_naturel = base();
    nom_naturel.insert("nom".to_string(), Some(Simple::Naturel(3)));
    assert_eq!(lire_une_requete(ID, &nom_naturel), None);
    let mut nom_autre_type = base();
    nom_autre_type.insert("nom".to_string(), None);
    assert_eq!(lire_une_requete(ID, &nom_autre_type), None);
}

#[test]
fn les_requetes_se_rangent_par_nom_replie() {
    let avec = |id: &str, nom: &str| RequeteEnregistreeLue {
        id: format!("01JREQUETE00000000000000{id}"),
        nom: nom.to_string(),
        requete: None,
    };
    let mut requetes = vec![
        avec("03", "ponts lents"),
        avec("01", "Église"),
        avec("02", "Ponts lents"),
        avec("04", "Ponts lents"),
    ];
    ranger_les_requetes(&mut requetes);
    let ids: Vec<&str> = requetes.iter().map(|r| &r.id[24..]).collect();
    assert_eq!(ids, ["01", "02", "04", "03"]);
}
