//! L'index des idées interrogées et la réponse d'une requête (CAP-06 ; questions 6 à 10) : la
//! portée et la satisfaction de chaque critère, les parts « sans … », l'assiette et l'ordre, sur
//! des jeux inventés.

use std::collections::BTreeSet;

use noyau::donnees::boite::{Contenu, EnBoite, Ordre};
use noyau::donnees::gestes::{Appareil, IdeeLue, Provenance};
use noyau::donnees::idee::{Assiette, duree};
use noyau::donnees::index::Index;
use noyau::donnees::requete::{
    Bornes, Etat, Jour, ProvenanceDeLIdee, Requete, Sans, texte_de_la_reponse,
};

/// L'heure de référence des idées de test, en secondes.
const T0: i64 = 1_793_000_000;

/// L'idée de test `n` : captée à l'établi, notée à `T0 + 60 × n`, d'identifiant `01JIDEE` suivi
/// de `n` sur deux chiffres, sans texte, étiquette, prise ni lien.
fn idee(n: i64) -> EnBoite {
    EnBoite {
        idee: IdeeLue {
            id: format!("01JIDEE{n:02}"),
            provenance: Provenance::Captee {
                par: Appareil::Etabli,
            },
            le: T0 + 60 * n,
            texte: None,
            etiquettes: BTreeSet::new(),
            retiree: None,
            liens: BTreeSet::new(),
        },
        contenus: BTreeSet::new(),
        duree_ms: None,
        reliee: false,
    }
}

/// `e`, avec le texte `texte` (et le contenu `TexteDeLIdee`).
fn avec_texte(mut e: EnBoite, texte: &str) -> EnBoite {
    e.idee.texte = Some(texte.to_string());
    e.contenus.insert(Contenu::TexteDeLIdee);
    e
}

/// `e`, avec les étiquettes `etiquettes`.
fn avec_etiquettes(mut e: EnBoite, etiquettes: &[&str]) -> EnBoite {
    e.idee.etiquettes = etiquettes
        .iter()
        .map(|etiquette| etiquette.to_string())
        .collect();
    e
}

/// `e`, avec des prises de ces durées (`None` pour une prise sans durée lue) : sa durée par
/// `idee::duree`, et le contenu `Prise`.
fn avec_prises(mut e: EnBoite, prises: &[Option<u64>]) -> EnBoite {
    e.duree_ms = duree(prises);
    e.contenus.insert(Contenu::Prise);
    e
}

/// `e`, importée à l'établi.
fn importee(mut e: EnBoite) -> EnBoite {
    e.idee.provenance = Provenance::Importee {
        par: Appareil::Etabli,
        depuis: "01JIMPORT".into(),
    };
    e
}

/// `e`, retirée à `quand`.
fn retiree_a(mut e: EnBoite, quand: i64) -> EnBoite {
    e.idee.retiree = Some(quand);
    e
}

/// `e`, notée à `le`.
fn le_a(mut e: EnBoite, le: i64) -> EnBoite {
    e.idee.le = le;
    e
}

/// Les identifiants des idées de test dont `suffixes` sont les numéros sur deux chiffres.
fn ids<const N: usize>(suffixes: [&str; N]) -> Vec<String> {
    suffixes.iter().map(|s| format!("01JIDEE{s}")).collect()
}

/// Les identifiants de `trouvees`, dans leur ordre.
fn ids_de(trouvees: &[EnBoite]) -> Vec<String> {
    trouvees.iter().map(|e| e.idee.id.clone()).collect()
}

/// Le texte de la réponse de `requete` sur l'index de `idees` et de `illisibles` idées
/// illisibles, et les identifiants trouvés, par la date, sans borne.
fn reponse(idees: Vec<EnBoite>, illisibles: usize, requete: &Requete) -> (String, Vec<String>) {
    let r = Index::construire(idees, illisibles).repondre(requete, Bornes::default(), Ordre::Date);
    (
        texte_de_la_reponse(r.trouvees.len(), r.assiette, r.illisibles, r.sans),
        ids_de(&r.trouvees),
    )
}

fn plus_d_une_minute() -> Requete {
    Requete {
        plus_de_secondes: Some(60),
        ..Requete::default()
    }
}

fn cherche(texte: &str) -> Requete {
    Requete {
        texte: texte.to_string(),
        ..Requete::default()
    }
}

/// Douze idées : 1 à 3 sans prise ; 4 (61 000 ms), 5 (90 000 et 12 000), 6 (600 000), 7 (75 000) ;
/// 8 (60 999, affichée « 1:00 »), 9 (59 999), 10 (30 000), 11 (5 000 et 20 000), 12 (999).
fn jeu_de_cap06_7() -> Vec<EnBoite> {
    vec![
        idee(1),
        idee(2),
        idee(3),
        avec_prises(idee(4), &[Some(61_000)]),
        avec_prises(idee(5), &[Some(90_000), Some(12_000)]),
        avec_prises(idee(6), &[Some(600_000)]),
        avec_prises(idee(7), &[Some(75_000)]),
        avec_prises(idee(8), &[Some(60_999)]),
        avec_prises(idee(9), &[Some(59_999)]),
        avec_prises(idee(10), &[Some(30_000)]),
        avec_prises(idee(11), &[Some(5_000), Some(20_000)]),
        avec_prises(idee(12), &[Some(999)]),
    ]
}

/// 21 « Église du port » ; 22 « une église retirée », retirée ; 23 « le phare et la jetée » ;
/// 24 et 25 sans texte ; 26 « … », un texte sans mot ; 27 « L’EGLISE, encore ».
fn jeu_des_textes() -> Vec<EnBoite> {
    vec![
        avec_texte(idee(21), "Église du port"),
        retiree_a(avec_texte(idee(22), "une église retirée"), T0 + 6_000),
        avec_texte(idee(23), "le phare et la jetée"),
        idee(24),
        idee(25),
        avec_texte(idee(26), "\u{2026}"),
        avec_texte(idee(27), "L\u{2019}EGLISE, encore"),
    ]
}

/// 31 « Église du port », 90 s ; 32 « Église du port » ; 33 « le phare », 90 s ; 34, 90 s ;
/// 35, rien ; 36 « église, nuit », 30 s et une prise sans durée ; 37 « phare ».
fn jeu_mixte() -> Vec<EnBoite> {
    vec![
        avec_prises(avec_texte(idee(31), "Église du port"), &[Some(90_000)]),
        avec_texte(idee(32), "Église du port"),
        avec_prises(avec_texte(idee(33), "le phare"), &[Some(90_000)]),
        avec_prises(idee(34), &[Some(90_000)]),
        idee(35),
        avec_prises(avec_texte(idee(36), "église, nuit"), &[Some(30_000), None]),
        avec_texte(idee(37), "phare"),
    ]
}

/// 41 pont ; 42 Pont ; 43 ponts et nuit ; 44 pont-levis ; 45 #pont ; 46, aucune ; 47 Pönt.
fn jeu_des_etiquettes() -> Vec<EnBoite> {
    vec![
        avec_etiquettes(idee(41), &["pont"]),
        avec_etiquettes(idee(42), &["Pont"]),
        avec_etiquettes(idee(43), &["ponts", "nuit"]),
        avec_etiquettes(idee(44), &["pont-levis"]),
        avec_etiquettes(idee(45), &["#pont"]),
        idee(46),
        avec_etiquettes(idee(47), &["Pönt"]),
    ]
}

/// 61 « eglise a » et 62 « eglise b », de la même seconde ; 63 « eglise c », une seconde après ;
/// dans un ordre qui n'est pas celui de la réponse.
fn jeu_d_une_seconde() -> Vec<EnBoite> {
    vec![
        le_a(avec_texte(idee(62), "eglise b"), T0),
        le_a(avec_texte(idee(63), "eglise c"), T0 + 1),
        le_a(avec_texte(idee(61), "eglise a"), T0),
    ]
}

/// Une idée `id` du jeu de la boîte, née à `le`, avec ses étiquettes ; captée à l'établi, ou
/// importée (`idee` de `crates/noyau/tests/boite.rs`, recopiée).
fn idee_de_la_boite(id: &str, le: i64, etiquettes: &[&str], captee: bool) -> IdeeLue {
    IdeeLue {
        id: id.to_string(),
        provenance: if captee {
            Provenance::Captee {
                par: Appareil::Etabli,
            }
        } else {
            Provenance::Importee {
                par: Appareil::Etabli,
                depuis: "01IMPORT".into(),
            }
        },
        le,
        texte: None,
        etiquettes: etiquettes.iter().map(|e| e.to_string()).collect(),
        retiree: None,
        liens: BTreeSet::new(),
    }
}

/// `en_boite` de `crates/noyau/tests/boite.rs`, recopiée.
fn en_boite(idee: IdeeLue, contenus: &[Contenu], duree_ms: Option<u64>, reliee: bool) -> EnBoite {
    EnBoite {
        idee,
        contenus: contenus.iter().copied().collect(),
        duree_ms,
        reliee,
    }
}

/// Le jeu de la boîte, celui de la tranche 3 : A à E, non retirées, et F, retirée à 500.
fn jeu_de_la_boite() -> Vec<EnBoite> {
    use Contenu::*;
    vec![
        en_boite(
            idee_de_la_boite("01A", 100, &["pont"], true),
            &[TexteDeLIdee, Prise],
            Some(47_000),
            false,
        ),
        en_boite(
            idee_de_la_boite("01B", 300, &[], false),
            &[Prise, MidiVerse],
            None,
            true,
        ),
        en_boite(
            idee_de_la_boite("01C", 200, &["Pont", "refrain"], false),
            &[Prise, TexteVerse],
            Some(12_000),
            false,
        ),
        en_boite(
            idee_de_la_boite("01D", 300, &["élan"], true),
            &[TexteDeLIdee, Prise],
            Some(47_000),
            true,
        ),
        en_boite(
            idee_de_la_boite("01E", 200, &["pont"], true),
            &[TexteDeLIdee],
            None,
            false,
        ),
        retiree_a(
            en_boite(
                idee_de_la_boite("01F", 400, &["pont"], true),
                &[Prise],
                None,
                false,
            ),
            500,
        ),
    ]
}

#[test]
fn plus_d_une_minute_se_juge_a_la_seconde_affichee() {
    let index = Index::construire(jeu_de_cap06_7(), 0);
    let r = index.repondre(&plus_d_une_minute(), Bornes::default(), Ordre::Date);
    assert_eq!(
        r.assiette,
        Assiette {
            considerees: 12,
            portant: 9,
            decidees: 9
        }
    );
    assert_eq!(
        r.sans,
        Sans {
            duree: Some(3),
            ..Sans::default()
        }
    );
    assert_eq!(
        reponse(jeu_de_cap06_7(), 0, &plus_d_une_minute()),
        (
            "4 idées · 3 sans durée · 12 interrogées".to_string(),
            ids(["07", "06", "05", "04"])
        )
    );
    // 60 400 ms s'affiche « 1:00 » : elle n'est pas de plus d'une minute ; 61 000, « 1:01 », l'est.
    let seuil = vec![
        avec_prises(idee(20), &[Some(60_400)]),
        avec_prises(idee(21), &[Some(61_000)]),
    ];
    assert_eq!(reponse(seuil, 0, &plus_d_une_minute()).1, ids(["21"]));
}

#[test]
fn une_prise_sans_duree_compte_parmi_les_sans_duree() {
    // CAP-06 · 9 : 13, une prise de 30 s et une sans durée ; 14, une de 90 s et une sans durée.
    let mut jeu = jeu_de_cap06_7();
    jeu.push(avec_prises(idee(13), &[Some(30_000), None]));
    jeu.push(avec_prises(idee(14), &[Some(90_000), None]));
    assert_eq!(
        reponse(jeu, 0, &plus_d_une_minute()).0,
        "4 idées · 5 sans durée · 14 interrogées"
    );
}

#[test]
fn une_idee_illisible_se_compte_a_part() {
    let index = Index::construire(jeu_de_cap06_7(), 1);
    let r = index.repondre(&plus_d_une_minute(), Bornes::default(), Ordre::Date);
    assert_eq!(
        r.assiette,
        Assiette {
            considerees: 13,
            portant: 9,
            decidees: 9
        }
    );
    assert_eq!(r.illisibles, 1);
    assert_eq!(
        reponse(jeu_de_cap06_7(), 1, &plus_d_une_minute()).0,
        "4 idées · 3 sans durée · 1 illisible · 13 interrogées"
    );
}

#[test]
fn une_idee_retiree_n_est_ni_trouvee_ni_comptee() {
    // 21 « Église du port » ; 22 « une église retirée », retirée ; 23 « le phare et la jetée » ; 24 et
    // 25 sans texte ; 26 « … », un texte sans mot ; 27 « L’EGLISE, encore ».
    let r = reponse(jeu_des_textes(), 0, &cherche("eglise"));
    assert_eq!(
        r,
        (
            "2 idées · 2 sans texte · 6 interrogées".to_string(),
            ids(["27", "21"])
        )
    );
}

#[test]
fn deux_criteres_se_combinent_et_les_sans_se_comptent_parmi_les_candidates() {
    // 31 « Église du port », 90 s ; 32 « Église du port » ; 33 « le phare », 90 s ; 34, 90 s ;
    // 35, rien ; 36 « église, nuit », 30 s et une prise sans durée ; 37 « phare ».
    let requete = Requete {
        plus_de_secondes: Some(60),
        ..cherche("eglise")
    };
    let index = Index::construire(jeu_mixte(), 0);
    let r = index.repondre(&requete, Bornes::default(), Ordre::Date);
    assert_eq!(
        r.assiette,
        Assiette {
            considerees: 7,
            portant: 2,
            decidees: 2
        }
    );
    assert_eq!(
        r.sans,
        Sans {
            duree: Some(3),
            texte: Some(2),
            etiquette: None
        }
    );
    assert_eq!(
        reponse(jeu_mixte(), 0, &requete),
        (
            "1 idée · 3 sans durée · 2 sans texte · 7 interrogées".to_string(),
            ids(["31"])
        )
    );
}

#[test]
fn l_etiquette_se_compare_octet_pour_octet_et_le_texte_ne_la_lit_pas() {
    // 41 pont ; 42 Pont ; 43 ponts et nuit ; 44 pont-levis ; 45 #pont ; 46, aucune ; 47 Pönt.
    let pont = Requete {
        etiquette: Some("pont".to_string()),
        ..Requete::default()
    };
    assert_eq!(
        reponse(jeu_des_etiquettes(), 0, &pont),
        (
            "1 idée · 1 sans étiquette · 7 interrogées".to_string(),
            ids(["41"])
        )
    );
    assert_eq!(
        reponse(jeu_des_etiquettes(), 0, &cherche("pont")).0,
        "0 idée · 7 sans texte · 7 interrogées"
    );
    assert_eq!(
        Index::construire(jeu_des_etiquettes(), 0).etiquettes(),
        [
            "#pont",
            "Pont",
            "Pönt",
            "nuit",
            "pont",
            "pont-levis",
            "ponts"
        ]
    );
}

#[test]
fn la_provenance_l_etat_et_le_contenu_n_ont_pas_de_part_sans() {
    // 51 captée ; 52 importée ; une illisible.
    let importees = Requete {
        provenance: Some(ProvenanceDeLIdee::Importee),
        ..Requete::default()
    };
    assert_eq!(
        reponse(vec![idee(51), importee(idee(52))], 1, &importees),
        (
            "1 idée · 1 illisible · 3 interrogées".to_string(),
            ids(["52"])
        )
    );
    // L'état : 51, non reliée ; 53, reliée. Chacune répond à son état, sans part « sans ».
    let jeu = || {
        vec![
            idee(51),
            EnBoite {
                reliee: true,
                ..idee(53)
            },
        ]
    };
    let selon = |etat| {
        reponse(
            jeu(),
            0,
            &Requete {
                etat: Some(etat),
                ..Requete::default()
            },
        )
    };
    assert_eq!(
        selon(Etat::Reliee),
        ("1 idée · 2 interrogées".to_string(), ids(["53"]))
    );
    assert_eq!(
        selon(Etat::NonReliee),
        ("1 idée · 2 interrogées".to_string(), ids(["51"]))
    );
}

#[test]
fn la_date_se_juge_sur_les_bornes() {
    // A (0 h 30, heure d'été), B (23 h 30, heure d'hiver), C (le 26, 0 h 30), et les bornes du
    // 25 octobre 2026 à Paris (tâche 2).
    let jeu = vec![
        le_a(idee(71), 1_792_881_000),
        le_a(idee(72), 1_792_967_400),
        le_a(idee(73), 1_792_971_000),
    ];
    let bornes = Bornes {
        des: Some(1_792_879_200),
        avant: Some(1_792_969_200),
    };
    let requete = Requete {
        du: Jour::depuis_iso("2026-10-25"),
        au: Jour::depuis_iso("2026-10-25"),
        ..Requete::default()
    };
    let r = Index::construire(jeu, 0).repondre(&requete, bornes, Ordre::Date);
    assert_eq!(ids_de(&r.trouvees), ids(["72", "71"]));
    assert_eq!(
        texte_de_la_reponse(r.trouvees.len(), r.assiette, r.illisibles, r.sans),
        "2 idées · 3 interrogées"
    );
}

#[test]
fn l_ordre_est_total_et_jamais_la_pertinence() {
    // 61 « eglise a » et 62 « eglise b », de la même seconde ; 63 « eglise c », une seconde après.
    assert_eq!(
        reponse(jeu_d_une_seconde(), 0, &cherche("eglise")).1,
        ids(["63", "61", "62"])
    );
}

#[test]
fn une_requete_sans_mot_ne_pose_aucun_critere() {
    for vide in ["", "  ", "'", " « » "] {
        assert!(cherche(vide).est_vide(), "{vide:?}");
        assert_eq!(
            reponse(jeu_des_textes(), 0, &cherche(vide)),
            reponse(jeu_des_textes(), 0, &Requete::default())
        );
    }
    assert_eq!(
        reponse(jeu_des_textes(), 0, &Requete::default()).0,
        "6 idées · 6 interrogées"
    );
    assert!(
        !Requete {
            contenu: Some(Contenu::Photo),
            ..Requete::default()
        }
        .est_vide()
    );
}

#[test]
fn sans_critere_l_index_rend_la_boite() {
    // Le jeu de la boîte (`jeu_de_la_boite`, six idées, F retirée) : les ordres, les contenus et le
    // compte de la tranche 3, inchangés.
    let index = Index::construire(jeu_de_la_boite(), 0);
    let par = |ordre| {
        ids_de(
            &index
                .repondre(&Requete::default(), Bornes::default(), ordre)
                .trouvees,
        )
    };
    assert_eq!(par(Ordre::Date), ["01B", "01D", "01C", "01E", "01A"]);
    assert_eq!(par(Ordre::Etiquette), ["01C", "01E", "01A", "01D", "01B"]);
    assert_eq!(par(Ordre::Duree), ["01D", "01A", "01C", "01B", "01E"]);
    assert_eq!(par(Ordre::Reliee), ["01C", "01E", "01A", "01B", "01D"]);
    assert_eq!(par(Ordre::Provenance), ["01D", "01E", "01A", "01B", "01C"]);
    let contenu = |c| {
        ids_de(
            &index
                .repondre(
                    &Requete {
                        contenu: Some(c),
                        ..Requete::default()
                    },
                    Bornes::default(),
                    Ordre::Date,
                )
                .trouvees,
        )
    };
    assert_eq!(contenu(Contenu::Prise), ["01B", "01D", "01C", "01A"]);
    assert_eq!(contenu(Contenu::Photo), Vec::<&str>::new());
    assert_eq!(contenu(Contenu::MidiVerse), ["01B"]);
    assert_eq!(contenu(Contenu::TexteVerse), ["01C"]);
    assert_eq!(contenu(Contenu::TexteDeLIdee), ["01D", "01E", "01A"]);
    for requete in [
        Requete::default(),
        Requete {
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        },
        Requete {
            contenu: Some(Contenu::Photo),
            ..Requete::default()
        },
    ] {
        let r = index.repondre(&requete, Bornes::default(), Ordre::Date);
        assert_eq!(r.non_reliees, 3); // A, C, E ; F, retirée, n'y compte pas
        assert_eq!(
            r.assiette_du_compte,
            Assiette {
                considerees: 5,
                portant: 5,
                decidees: 5
            }
        );
    }
    let r = Index::construire(jeu_de_la_boite(), 1).repondre(
        &Requete::default(),
        Bornes::default(),
        Ordre::Date,
    );
    assert_eq!(r.non_reliees, 3);
    assert_eq!(
        r.assiette_du_compte,
        Assiette {
            considerees: 6,
            portant: 5,
            decidees: 5
        }
    );
}

#[test]
fn une_idee_aux_etiquettes_multiples_est_trouvee_par_chacune() {
    // 81 aube et pont ; 82 pont et zinc ; 83 aube ; 84, aucune. Une comparaison à la seule plus
    // petite étiquette (ou à la seule plus grande) perdrait 81 (ou 82) pour « pont ».
    let jeu = || {
        vec![
            avec_etiquettes(idee(81), &["aube", "pont"]),
            avec_etiquettes(idee(82), &["pont", "zinc"]),
            avec_etiquettes(idee(83), &["aube"]),
            idee(84),
        ]
    };
    let etiquette = |e: &str| Requete {
        etiquette: Some(e.to_string()),
        ..Requete::default()
    };
    assert_eq!(
        reponse(jeu(), 0, &etiquette("pont")),
        (
            "2 idées · 1 sans étiquette · 4 interrogées".to_string(),
            ids(["82", "81"])
        )
    );
    assert_eq!(
        reponse(jeu(), 0, &etiquette("aube")),
        (
            "2 idées · 1 sans étiquette · 4 interrogées".to_string(),
            ids(["83", "81"])
        )
    );
    assert_eq!(
        reponse(jeu(), 0, &etiquette("zinc")),
        (
            "1 idée · 1 sans étiquette · 4 interrogées".to_string(),
            ids(["82"])
        )
    );
    assert_eq!(
        Index::construire(jeu(), 0).etiquettes(),
        ["aube", "pont", "zinc"]
    );
}

#[test]
fn une_part_sans_ne_compte_que_les_idees_qui_satisfont_les_criteres_toujours_portes() {
    // 91 importée, sans durée ; 92 captée, sans durée ; 93 importée, 90 s ; 94 captée, 90 s. La
    // provenance, toujours portée, écarte 92 de la part « sans durée », et 94 des trouvées.
    let jeu = || {
        vec![
            importee(idee(91)),
            idee(92),
            importee(avec_prises(idee(93), &[Some(90_000)])),
            avec_prises(idee(94), &[Some(90_000)]),
        ]
    };
    let requete = Requete {
        provenance: Some(ProvenanceDeLIdee::Importee),
        ..plus_d_une_minute()
    };
    let r = Index::construire(jeu(), 0).repondre(&requete, Bornes::default(), Ordre::Date);
    assert_eq!(
        r.sans,
        Sans {
            duree: Some(1),
            ..Sans::default()
        }
    );
    assert_eq!(
        r.assiette,
        Assiette {
            considerees: 4,
            portant: 2,
            decidees: 2
        }
    );
    assert_eq!(
        reponse(jeu(), 0, &requete),
        (
            "1 idée · 1 sans durée · 4 interrogées".to_string(),
            ids(["93"])
        )
    );
}
