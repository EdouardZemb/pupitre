//! La boîte d'idées au noyau (CAP-02) : les retirées, les contenus d'une idée, le compte, la date
//! et la durée d'une ligne, l'étiquette saisie, sur des idées inventées. L'ordre, les contenus
//! cherchés et l'assiette se lisent dans la réponse d'une requête (`tests/index.rs`).

use std::collections::BTreeSet;

use noyau::donnees::boite::{
    Contenu, EnBoite, contenus_de, date_de_la_liste, duree_affichee, etiquette_saisie, retirees,
    texte_du_compte,
};
use noyau::donnees::gestes::{Appareil, IdeeLue, Provenance, Sorte};
use noyau::donnees::idee::Assiette;

/// Une idée `id`, née à `le`, avec ses étiquettes ; captée à l'établi, ou importée.
fn idee(id: &str, le: i64, etiquettes: &[&str], captee: bool) -> IdeeLue {
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

/// Une idée `id`, avec ou sans texte.
fn idee_avec_texte(id: &str, texte: Option<&str>) -> IdeeLue {
    IdeeLue {
        texte: texte.map(str::to_string),
        ..idee(id, 100, &[], true)
    }
}

fn en_boite(idee: IdeeLue, contenus: &[Contenu], duree_ms: Option<u64>, reliee: bool) -> EnBoite {
    EnBoite {
        idee,
        contenus: contenus.iter().copied().collect(),
        duree_ms,
        reliee,
    }
}

fn retiree_a(mut e: EnBoite, quand: i64) -> EnBoite {
    e.idee.retiree = Some(quand);
    e
}

fn ids_de(idees: Vec<EnBoite>) -> Vec<String> {
    idees.into_iter().map(|e| e.idee.id).collect()
}

/// F (retirée à 500), G (`01G`, retirée à 400), H (`01H`, retirée à 500) et E, non retirée.
fn jeu_des_retirees() -> Vec<EnBoite> {
    let simple = |id: &str| en_boite(idee(id, 100, &[], true), &[], None, false);
    vec![
        retiree_a(simple("01H"), 500),
        simple("01E"),
        retiree_a(simple("01G"), 400),
        retiree_a(simple("01F"), 500),
    ]
}

#[test]
fn les_retirees_se_rangent_par_la_date_du_retrait() {
    // F (retirée à 500), G (à 400), H (à 500, identifiant après F) : F, H, G ; E, non retirée,
    // n'y entre pas.
    assert_eq!(ids_de(retirees(jeu_des_retirees())), ["01F", "01H", "01G"]);
}

#[test]
fn les_contenus_d_une_idee_viennent_de_son_texte_de_ses_sortes_et_de_la_photo() {
    let avec_texte = idee_avec_texte("01A", Some("Une idée de pont"));
    assert_eq!(
        contenus_de(&avec_texte, [Sorte::Prise, Sorte::MidiVerseSeul], false),
        BTreeSet::from([Contenu::TexteDeLIdee, Contenu::Prise, Contenu::MidiVerse])
    );
    let sans_texte = idee_avec_texte("01B", None);
    assert_eq!(
        contenus_de(&sans_texte, [Sorte::TexteVerse], true),
        BTreeSet::from([Contenu::Photo, Contenu::TexteVerse])
    );
}

/// L'assiette de `c` idées considérées, dont `p` portant la donnée.
fn a(c: usize, p: usize) -> Assiette {
    Assiette {
        considerees: c,
        portant: p,
        decidees: p,
    }
}

#[test]
fn le_compte_dit_son_assiette_et_s_accorde() {
    assert_eq!(
        texte_du_compte(2, a(2, 2)),
        "2 idées non reliées · 2 interrogées"
    );
    assert_eq!(
        texte_du_compte(0, a(0, 0)),
        "0 idée non reliée · 0 interrogée"
    );
    assert_eq!(
        texte_du_compte(1, a(1, 1)),
        "1 idée non reliée · 1 interrogée"
    );
    assert_eq!(
        texte_du_compte(50, a(50, 50)),
        "50 idées non reliées · 50 interrogées"
    );
    assert_eq!(
        texte_du_compte(2, a(3, 2)),
        "2 idées non reliées · 1 illisible · 3 interrogées"
    );
    assert_eq!(
        texte_du_compte(2, a(4, 2)),
        "2 idées non reliées · 2 illisibles · 4 interrogées"
    );
    // 5 considérées, 2 portantes : 3 illisibles (ni 5 / 2, ni 5 + 2, ni 5 * 2).
    assert_eq!(
        texte_du_compte(2, a(5, 2)),
        "2 idées non reliées · 3 illisibles · 5 interrogées"
    );
}

#[test]
fn la_date_de_la_liste_porte_l_annee_au_decalage_donne() {
    assert_eq!(date_de_la_liste(1_793_748_600, 0), "3 nov. 2026"); // 2026-11-03, 23 h 30 UTC
    assert_eq!(date_de_la_liste(1_793_748_600, 60), "4 nov. 2026");
    assert_eq!(date_de_la_liste(1_798_761_600, 0), "1er janv. 2027");
    assert_eq!(date_de_la_liste(1_798_761_600, -60), "31 déc. 2026");
    // Une idée notée le 25 octobre 2026 à 0 h 30, heure d'été : au décalage de l'instant, +120,
    // elle reste du 25 ; au décalage d'hiver, +60, elle serait du 24 (L11, la piste).
    assert_eq!(date_de_la_liste(1_792_881_000, 120), "25 oct. 2026");
    assert_eq!(date_de_la_liste(1_792_881_000, 60), "24 oct. 2026");
}

#[test]
fn la_date_de_la_liste_suit_le_calendrier_hors_de_2026() {
    // 2028 est bissextile : le 29 février 2028, à midi UTC.
    assert_eq!(date_de_la_liste(1_835_438_400, 0), "29 févr. 2028");
    // 2100 ne l'est pas : le jour qui suit le 28 février 2100 est le 1er mars, à minuit UTC.
    assert_eq!(date_de_la_liste(4_107_542_400, 0), "1er mars 2100");
    // Une seconde avant 1970 : `div_euclid` range l'instant au jour d'avant, le 31 décembre 1969 ;
    // `/`, qui arrondit vers zéro, le rangerait au 1er janvier 1970.
    assert_eq!(date_de_la_liste(-1, 0), "31 déc. 1969");
}

#[test]
fn chaque_mois_a_son_abreviation() {
    // Le 15 de chaque mois de 2026, à midi UTC : 20 454 jours vont du 1er janvier 1970 au
    // 1er janvier 2026.
    let mois = [
        (14, "janv."),
        (45, "févr."),
        (73, "mars"),
        (104, "avr."),
        (134, "mai"),
        (165, "juin"),
        (195, "juil."),
        (226, "août"),
        (257, "sept."),
        (287, "oct."),
        (318, "nov."),
        (348, "déc."),
    ];
    for (jour_de_l_annee, abreviation) in mois {
        let heure = (20_454 + jour_de_l_annee) * 86_400 + 43_200;
        assert_eq!(date_de_la_liste(heure, 0), format!("15 {abreviation} 2026"));
    }
}

#[test]
fn la_duree_s_arrondit_a_la_seconde_inferieure() {
    assert_eq!(duree_affichee(None), "—");
    assert_eq!(duree_affichee(Some(999)), "0:00");
    assert_eq!(duree_affichee(Some(47_000)), "0:47");
    assert_eq!(duree_affichee(Some(59_500)), "0:59");
    assert_eq!(duree_affichee(Some(60_000)), "1:00");
    assert_eq!(duree_affichee(Some(61_000)), "1:01");
    assert_eq!(duree_affichee(Some(3_599_999)), "59:59");
    assert_eq!(duree_affichee(Some(3_600_000)), "1:00:00");
    assert_eq!(duree_affichee(Some(3_661_000)), "1:01:01");
    assert_eq!(duree_affichee(Some(36_000_000)), "10:00:00");
}

#[test]
fn l_etiquette_saisie_perd_ses_blancs_et_un_diese_de_tete() {
    assert_eq!(etiquette_saisie("  #pont "), "pont");
    assert_eq!(etiquette_saisie("##pont"), "#pont");
    assert_eq!(etiquette_saisie("pont"), "pont");
    assert_eq!(etiquette_saisie("  "), "");
    assert_eq!(etiquette_saisie("#"), "");
    assert_eq!(etiquette_saisie("# pont"), "pont");
}
