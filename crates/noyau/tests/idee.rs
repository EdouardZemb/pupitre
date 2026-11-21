//! Les règles d'une idée et de ses captations : les empreintes que nomment les clés
//! `<empreinte>/<clé>` des conteneurs des captations (§ 31 · 6.1), le lien « apparenté »,
//! l'état de reliée, la durée, l'idée sans rien et l'assiette d'un compte (§ 30 · 9).

use std::collections::BTreeSet;

use noyau::donnees::gestes::{Appareil, IdeeLue, Provenance};
use noyau::donnees::idee::{
    Assiette, assiette, bouts_du_lien, cle_du_lien, duree, empreintes_nommees, est_vide, reliee,
};

const P: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const S: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const M: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const E: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const R: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const FAUX: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const FAUX2: &str = "7777777777777777777777777777777777777777777777777777777777777777";

#[test]
fn les_empreintes_nommees_lisent_les_stems_leurs_midi_et_les_rapports() {
    let cles = [
        format!("{P}/idee"),
        format!("{P}/manifeste"),
        format!("{P}/stem/{S}/duree"),
        format!("{P}/stem/{S}/midi/{M}"),
        format!("{P}/stem/{S}/nom/Jeu A/voix.wav"),
        format!("{E}/nom/stem/{FAUX}/midi/{FAUX2}"),
        format!("{M}/division"),
    ];
    let nommees = empreintes_nommees(cles.iter().map(String::as_str), [R]);
    assert_eq!(nommees, BTreeSet::from([P, S, M, E, R].map(String::from)));
    assert!(!nommees.contains(FAUX) && !nommees.contains(FAUX2));
    assert_eq!(empreintes_nommees(["sans-barre"], []), BTreeSet::new());
}

#[test]
fn les_empreintes_nommees_se_coupent_au_premier_trait() {
    let cles = [
        "e1/idee",
        "e1/format",
        "e2/idee",
        "sans-trait",
        "e3/nom/a/b.wav",
    ];

    assert_eq!(
        empreintes_nommees(cles, []),
        BTreeSet::from(["e1".to_string(), "e2".to_string(), "e3".to_string()])
    );
}

fn idee(id: &str) -> IdeeLue {
    IdeeLue {
        id: id.to_string(),
        provenance: Provenance::Captee {
            par: Appareil::Etabli,
        },
        le: 0,
        texte: None,
        etiquettes: BTreeSet::new(),
        retiree: None,
        liens: BTreeSet::new(),
    }
}

#[test]
fn la_cle_du_lien_range_ses_bouts_par_octets() {
    let a = "01M76GVQ00041061050R3GG28A";
    let b = "01HF7YAT00000000000000000B";
    let attendue = "apparente:01HF7YAT00000000000000000B:01M76GVQ00041061050R3GG28A";
    assert_eq!(cle_du_lien(a, b), attendue);
    assert_eq!(cle_du_lien(b, a), attendue);
    assert_eq!(cle_du_lien("idee-10", "idee-2"), "apparente:idee-10:idee-2");

    assert_eq!(bouts_du_lien(attendue), Some((b, a)));
    assert_eq!(bouts_du_lien("autre:a:b"), None);
    assert_eq!(bouts_du_lien("apparente:a"), None);
}

#[test]
fn une_idee_n_est_reliee_que_par_un_bout_non_retire() {
    let lien = cle_du_lien("a", "b");
    let mut ida = idee("a");
    ida.liens.insert(lien.clone());
    let mut idb = idee("b");
    idb.liens.insert(lien);
    let idc = idee("c");

    let b_retiree = |id: &str| id == "b";
    assert!(!reliee(&ida, b_retiree));
    assert!(reliee(&idb, b_retiree));
    assert!(reliee(&ida, |_| false));
    assert!(!reliee(&idc, |_| false));
}

#[test]
fn la_duree_est_celle_de_la_plus_longue_prise() {
    assert_eq!(duree(&[]), None);
    assert_eq!(duree(&[Some(30_000), None]), None);
    assert_eq!(duree(&[Some(30_000), Some(64_000)]), Some(64_000));
    // La plus longue d'abord : une règle qui rendrait la dernière prise rendrait 30 000.
    assert_eq!(duree(&[Some(64_000), Some(30_000)]), Some(64_000));
}

#[test]
fn une_idee_est_vide_sans_rien() {
    assert!(est_vide(&idee("a"), 0));
    assert!(!est_vide(&idee("a"), 1));

    let mut avec_texte = idee("a");
    avec_texte.texte = Some("un motif".to_string());
    assert!(!est_vide(&avec_texte, 0));

    let mut avec_etiquette = idee("a");
    avec_etiquette.etiquettes.insert("refrain".to_string());
    assert!(!est_vide(&avec_etiquette, 0));

    let mut avec_lien = idee("a");
    avec_lien.liens.insert(cle_du_lien("a", "b"));
    assert!(!est_vide(&avec_lien, 0));
}

#[test]
fn l_assiette_ecarte_les_idees_retirees() {
    let mut idees: Vec<IdeeLue> = (0..13).map(|i| idee(&format!("i{i}"))).collect();
    for (i, une) in idees.iter_mut().enumerate() {
        if i < 9 || i == 12 {
            une.texte = Some("un motif".to_string());
        }
    }
    idees[12].retiree = Some(5);

    let a = assiette(&idees, |une| une.texte.is_some());

    assert_eq!(
        a,
        Assiette {
            considerees: 12,
            portant: 9,
            decidees: 9
        }
    );
}
