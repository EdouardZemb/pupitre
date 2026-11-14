//! Les gestes du noyau (§ 31 · 3) : la date d'un message, au décalage local reçu, la naissance,
//! noter, les gestes de tri et verser une fiche, que le noyau rend en changements sans rien
//! écrire, avec l'inverse de chaque geste de tri, son annulation et ses refus ; l'appareil,
//! retrouvé de son nom, et chaque sorte, une fois.

use std::collections::BTreeSet;

use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::gestes::{
    Appareil, CaptationLue, ENTIER_SUR, Fiche, IdeeLue, Impossible, Provenance, Sorte, annulation,
    completer, date_du_message, delier, naissance, noter, oter_une_etiquette, poser_une_etiquette,
    presente, regrouper, relier, retablir, retirer, verser,
};

const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023, 22 h 13 min 20 s UTC
/// Deux jours après `HEURE` : le 16 novembre 2023, 22 h 13 min 20 s UTC.
const DEUX_JOURS_APRES: i64 = HEURE + 2 * 86_400;

/// Trois idées, dans l'ordre de leurs octets, et l'idée d'une captation versée.
const A: &str = "01HF7YAT00000000000000000A";
const B: &str = "01HF7YAT00000000000000000B";
const C: &str = "01HF7YAT00000000000000000C";
const V: &str = "01HF7YAT00000000000000000V";
/// L'ULID d'un import.
const IMPORT: &str = "01HF7YAT00000000000000000M";
/// L'empreinte d'une captation, inventée.
const P: &str = "c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00c0ffee00";
/// La clé du lien entre A et B, ses bouts dans l'ordre de leurs octets.
const LIEN_AB: &str = "apparente:01HF7YAT00000000000000000A:01HF7YAT00000000000000000B";

fn chemin(segments: &[&str]) -> Vec<String> {
    segments.iter().map(|segment| segment.to_string()).collect()
}

fn poser(segments: &[&str], valeur: Simple) -> Changement {
    Changement::Poser {
        chemin: chemin(segments),
        valeur,
    }
}

fn oter(segments: &[&str]) -> Changement {
    Changement::Oter {
        chemin: chemin(segments),
    }
}

/// L'idée `id`, notée à l'établi et née à `HEURE`, sans texte, étiquette, lien ni retrait.
fn idee(id: &str) -> IdeeLue {
    IdeeLue {
        id: id.to_string(),
        provenance: Provenance::Captee {
            par: Appareil::Etabli,
        },
        le: HEURE,
        texte: None,
        etiquettes: BTreeSet::new(),
        retiree: None,
        liens: BTreeSet::new(),
    }
}

/// La captation de sorte `sorte` et d'empreinte `P`, dans l'idée `idee`.
fn captation(sorte: Sorte, idee: &str) -> CaptationLue {
    CaptationLue {
        sorte,
        empreinte: P.to_string(),
        idee: idee.to_string(),
        duree_ms: None,
    }
}

/// La provenance d'une idée née d'un fichier versé par l'import `IMPORT`, à l'établi.
fn importee() -> Provenance {
    Provenance::Importee {
        par: Appareil::Etabli,
        depuis: IMPORT.into(),
    }
}

/// La fiche d'une prise WAV de `duree_ms` millisecondes.
fn wav(duree_ms: u64) -> Fiche {
    Fiche {
        format: "wav".into(),
        duree_ms: Some(duree_ms),
    }
}

#[test]
fn la_date_du_message_suit_le_decalage_et_dit_le_premier() {
    assert_eq!(date_du_message(1_793_748_600, 0), "3 novembre"); // 2026-11-03, 23 h 30 UTC
    assert_eq!(date_du_message(1_793_748_600, 60), "4 novembre");
    assert_eq!(date_du_message(1_793_748_600, -300), "3 novembre");
    assert_eq!(date_du_message(1_793_491_200, 0), "1er novembre");
    assert_eq!(date_du_message(1_793_491_200, -60), "31 octobre");
    assert_eq!(date_du_message(1_798_759_800, 60), "1er janvier");
    assert_eq!(date_du_message(1_835_438_400, 0), "29 février");
    assert_eq!(date_du_message(0, 0), "1er janvier");
    assert_eq!(date_du_message(-1, 0), "31 décembre");
    // Les termes de siècle de l'algorithme ne changent l'année qu'aux abords du 1er mars :
    // doe / 36 524, aux premiers jours de mars après le premier siècle de l'ère (dès le 1er mars
    // 2100 dans celle de 2000) ; doe / 146 096, au seul 29 février qui finit l'ère (celui de 2000
    // dans l'ère de 1600).
    assert_eq!(date_du_message(951_825_600, 0), "29 février"); // 2000-02-29, 12 h UTC
    assert_eq!(date_du_message(4_107_585_600, 0), "1er mars"); // 2100-03-01, 12 h UTC
}

#[test]
fn la_naissance_pose_trois_valeurs_a_la_racine() {
    let op = naissance(
        "01HF7YAT000000000000000001",
        "01HF7YAT000000000000000002",
        "00000000-0000-4000-8000-000000000003",
        HEURE,
    );
    assert_eq!(op.message, "Fonds créé");
    assert_eq!(op.heure, HEURE);
    assert_eq!(
        op.changements,
        vec![
            poser(
                &["identifiant"],
                Simple::Chaine("01HF7YAT000000000000000001".into())
            ),
            poser(
                &["lignee"],
                Simple::Chaine("01HF7YAT000000000000000002".into())
            ),
            poser(
                &["proprietaire"],
                Simple::Chaine("00000000-0000-4000-8000-000000000003".into())
            ),
        ]
    );
}

#[test]
fn noter_cree_l_idee_captee_datee_avec_ses_etiquettes() {
    let op = noter(
        "01HF7YAT000000000000000004",
        "Un refrain qui monte, en ré",
        Appareil::Etabli,
        HEURE,
        0,
    )
    .unwrap();
    assert_eq!(op.message, "Idée du 14 novembre notée");
    let i = "01HF7YAT000000000000000004";
    assert_eq!(
        op.changements,
        vec![
            Changement::Creer {
                chemin: chemin(&["idees", i]),
                conteneurs: vec!["etiquettes".into()]
            },
            poser(
                &["idees", i, "texte"],
                Simple::Chaine("Un refrain qui monte, en ré".into())
            ),
            poser(&["idees", i, "provenance"], Simple::Chaine("captee".into())),
            poser(&["idees", i, "par"], Simple::Chaine("etabli".into())),
            poser(&["idees", i, "le"], Simple::Entier(HEURE)),
        ]
    );
    let au_carnet = noter(i, "x", Appareil::Carnet, HEURE, 0).unwrap();
    assert!(au_carnet.changements.contains(&poser(
        &["idees", i, "par"],
        Simple::Chaine("carnet".into())
    )));
    // Deux heures plus à l'est, `HEURE` (22 h 13 UTC, le 14) tombe le 15 : le message dit le jour
    // décalé, l'idée reste née à `HEURE`, et le texte se range tel qu'il est reçu, blancs compris.
    let decalee = noter(i, "  x  ", Appareil::Etabli, HEURE, 120).unwrap();
    assert_eq!(decalee.message, "Idée du 15 novembre notée");
    assert!(decalee.changements.contains(&poser(
        &["idees", i, "texte"],
        Simple::Chaine("  x  ".into())
    )));
    assert!(
        decalee
            .changements
            .contains(&poser(&["idees", i, "le"], Simple::Entier(HEURE)))
    );
}

#[test]
fn une_note_vide_est_refusee() {
    assert_eq!(
        noter("i", "", Appareil::Etabli, HEURE, 0),
        Err(Impossible::TexteVide)
    );
    assert_eq!(
        noter("i", " \t\n", Appareil::Etabli, HEURE, 0),
        Err(Impossible::TexteVide)
    );
    assert_eq!(Impossible::TexteVide.to_string(), "Le texte est vide.");
}

#[test]
fn chaque_sorte_a_son_conteneur() {
    assert_eq!(Sorte::Prise.conteneur(), "prises");
    assert_eq!(Sorte::MidiVerseSeul.conteneur(), "midi_verses");
    assert_eq!(Sorte::TexteVerse.conteneur(), "textes_verses");
    assert_eq!(
        (Appareil::Etabli.nom(), Appareil::Carnet.nom()),
        ("etabli", "carnet")
    );
}

/// Chaque appareil du noyau, une fois, à son rang (`rang_de_l_appareil`).
const APPAREILS: [Appareil; 2] = [Appareil::Etabli, Appareil::Carnet];

/// Le rang de `appareil` dans `APPAREILS`. Le `match` est exhaustif : un appareil ajouté au noyau
/// ne compile plus ici sans prendre un rang, et `APPAREILS` le reçoit alors.
fn rang_de_l_appareil(appareil: Appareil) -> usize {
    match appareil {
        Appareil::Etabli => 0,
        Appareil::Carnet => 1,
    }
}

/// Le rang de `sorte` dans `Sorte::TOUTES`. Le `match` est exhaustif : une sorte ajoutée au noyau
/// ne compile plus ici sans prendre un rang, et `Sorte::TOUTES` la reçoit alors.
fn rang_de_la_sorte(sorte: Sorte) -> usize {
    match sorte {
        Sorte::Prise => 0,
        Sorte::MidiVerseSeul => 1,
        Sorte::TexteVerse => 2,
    }
}

#[test]
fn chaque_appareil_se_retrouve_de_son_nom() {
    assert_eq!(APPAREILS.map(rang_de_l_appareil), [0, 1]);
    for appareil in APPAREILS {
        assert_eq!(Appareil::depuis_nom(appareil.nom()), Some(appareil));
    }
    assert_eq!(Appareil::depuis_nom("telephone"), None);
    assert_eq!(Appareil::depuis_nom("Etabli"), None);
    assert_eq!(Appareil::depuis_nom(""), None);
}

#[test]
fn toutes_les_sortes_y_sont_une_fois() {
    // Trois rangs distincts pour trois sortes : chacune y est, une fois, dans son ordre.
    assert_eq!(Sorte::TOUTES.map(rang_de_la_sorte), [0, 1, 2]);
}

#[test]
fn completer_pose_le_texte_et_son_inverse_repose_celui_d_avant() {
    let mut a = idee(A);
    a.texte = Some("Un pont en la mineur".into());
    let geste = completer(&a, "Un pont en la mineur, puis le refrain", HEURE, 0).unwrap();
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre : texte complété"
    );
    assert_eq!(geste.operation.heure, HEURE);
    assert_eq!(
        geste.operation.changements,
        vec![poser(
            &["idees", A, "texte"],
            Simple::Chaine("Un pont en la mineur, puis le refrain".into())
        )]
    );
    assert_eq!(
        geste.inverse,
        vec![poser(
            &["idees", A, "texte"],
            Simple::Chaine("Un pont en la mineur".into())
        )]
    );
}

#[test]
fn completer_une_idee_sans_texte_a_pour_inverse_oter() {
    let geste = completer(&idee(A), "Un couplet en sol", HEURE, 0).unwrap();
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre : texte complété"
    );
    assert_eq!(
        geste.operation.changements,
        vec![poser(
            &["idees", A, "texte"],
            Simple::Chaine("Un couplet en sol".into())
        )]
    );
    assert_eq!(geste.inverse, vec![oter(&["idees", A, "texte"])]);
}

#[test]
fn poser_une_etiquette_pose_vrai_sous_sa_cle_et_son_inverse_l_ote() {
    let geste = poser_une_etiquette(&idee(A), "pont", HEURE, 0).unwrap();
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre : étiquette #pont posée"
    );
    assert_eq!(geste.operation.heure, HEURE);
    assert_eq!(
        geste.operation.changements,
        vec![poser(&["idees", A, "etiquettes", "pont"], Simple::Vrai)]
    );
    assert_eq!(
        geste.inverse,
        vec![oter(&["idees", A, "etiquettes", "pont"])]
    );
    // Le message dit le jour où l'idée est née, au décalage du geste, jamais celui du geste :
    // deux heures plus à l'est, `HEURE` (22 h 13 UTC, le 14) tombe le 15.
    let plus_tard = poser_une_etiquette(&idee(A), "pont", DEUX_JOURS_APRES, 120).unwrap();
    assert_eq!(
        plus_tard.operation.message,
        "Idée du 15 novembre : étiquette #pont posée"
    );
    assert_eq!(plus_tard.operation.heure, DEUX_JOURS_APRES);
}

#[test]
fn oter_une_etiquette_l_ote_et_son_inverse_la_repose() {
    let mut a = idee(A);
    a.etiquettes.insert("pont".into());
    let geste = oter_une_etiquette(&a, "pont", HEURE, 0).unwrap();
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre : étiquette #pont ôtée"
    );
    assert_eq!(geste.operation.heure, HEURE);
    assert_eq!(
        geste.operation.changements,
        vec![oter(&["idees", A, "etiquettes", "pont"])]
    );
    assert_eq!(
        geste.inverse,
        vec![poser(&["idees", A, "etiquettes", "pont"], Simple::Vrai)]
    );
}

#[test]
fn relier_pose_le_lien_sous_sa_cle_dans_un_sens_ou_dans_l_autre() {
    let geste = relier(&idee(A), &idee(B), HEURE, 0).unwrap();
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre reliée à l'idée du 14 novembre"
    );
    assert_eq!(geste.operation.heure, HEURE);
    assert_eq!(
        geste.operation.changements,
        vec![poser(&["liens", LIEN_AB], Simple::Vrai)]
    );
    assert_eq!(geste.inverse, vec![oter(&["liens", LIEN_AB])]);
    // B née deux jours plus tard : le message dit le jour de chaque bout, dans l'ordre du geste ;
    // la clé range les bouts dans l'ordre de leurs octets, quel que soit le sens.
    let mut b = idee(B);
    b.le = DEUX_JOURS_APRES;
    let ab = relier(&idee(A), &b, HEURE, 0).unwrap();
    assert_eq!(
        ab.operation.message,
        "Idée du 14 novembre reliée à l'idée du 16 novembre"
    );
    let ba = relier(&b, &idee(A), HEURE, 0).unwrap();
    assert_eq!(
        ba.operation.message,
        "Idée du 16 novembre reliée à l'idée du 14 novembre"
    );
    assert_eq!(ba.operation.changements, geste.operation.changements);
    assert_eq!(ba.inverse, geste.inverse);
}

#[test]
fn delier_ote_le_lien_et_son_inverse_le_repose() {
    let (mut a, mut b) = (idee(A), idee(B));
    a.liens.insert(LIEN_AB.into());
    b.liens.insert(LIEN_AB.into());
    let geste = delier(&a, &b, HEURE, 0).unwrap();
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre déliée de l'idée du 14 novembre"
    );
    assert_eq!(geste.operation.heure, HEURE);
    assert_eq!(geste.operation.changements, vec![oter(&["liens", LIEN_AB])]);
    assert_eq!(
        geste.inverse,
        vec![poser(&["liens", LIEN_AB], Simple::Vrai)]
    );
    // Depuis l'autre bout, née deux jours plus tard : le même lien, chaque bout à son jour.
    b.le = DEUX_JOURS_APRES;
    let ba = delier(&b, &a, HEURE, 0).unwrap();
    assert_eq!(
        ba.operation.message,
        "Idée du 16 novembre déliée de l'idée du 14 novembre"
    );
    assert_eq!(ba.operation.changements, geste.operation.changements);
    assert_eq!(ba.inverse, geste.inverse);
}

#[test]
fn regrouper_la_seule_prise_d_une_idee_qui_n_a_rien_d_autre_la_retire() {
    // regrouper la seule prise P de V, qui n'a rien d'autre, dans A : V est vidée.
    let geste = regrouper(&captation(Sorte::Prise, V), &idee(V), 1, &idee(A), HEURE, 0).unwrap();
    let cle = format!("{P}/idee");
    assert_eq!(
        geste.operation.message,
        "Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée"
    );
    assert_eq!(geste.operation.heure, HEURE);
    assert_eq!(
        geste.operation.changements,
        vec![
            poser(&["prises", cle.as_str()], Simple::Chaine(A.into())),
            poser(&["idees", V, "retiree"], Simple::Entier(HEURE)),
        ]
    );
    assert_eq!(
        geste.inverse,
        vec![
            poser(&["prises", cle.as_str()], Simple::Chaine(V.into())),
            oter(&["idees", V, "retiree"]),
        ]
    );
    // La cible née deux jours plus tard : le message dit le jour de la cible, puis celui de la
    // source.
    let mut a = idee(A);
    a.le = DEUX_JOURS_APRES;
    let vers_a = regrouper(&captation(Sorte::Prise, V), &idee(V), 1, &a, HEURE, 0).unwrap();
    assert_eq!(
        vers_a.operation.message,
        "Idée du 16 novembre : prise regroupée, idée du 14 novembre retirée"
    );
}

#[test]
fn regrouper_une_prise_d_une_idee_qui_garde_autre_chose_ne_la_retire_pas() {
    let cle = format!("{P}/idee");
    let deplacee = vec![poser(&["prises", cle.as_str()], Simple::Chaine(A.into()))];
    let rendue = vec![poser(&["prises", cle.as_str()], Simple::Chaine(V.into()))];
    // La source garde un texte.
    let mut v = idee(V);
    v.texte = Some("Une boucle de basse".into());
    let avec_un_texte = regrouper(&captation(Sorte::Prise, V), &v, 1, &idee(A), HEURE, 0).unwrap();
    // La source garde une autre captation.
    let avec_une_autre =
        regrouper(&captation(Sorte::Prise, V), &idee(V), 2, &idee(A), HEURE, 0).unwrap();
    for geste in [avec_un_texte, avec_une_autre] {
        assert_eq!(
            geste.operation.message,
            "Idée du 14 novembre : prise regroupée"
        );
        assert_eq!(geste.operation.changements, deplacee);
        assert_eq!(geste.inverse, rendue);
    }
}

#[test]
fn regrouper_un_midi_ou_un_texte_verse_le_deplace_dans_son_conteneur() {
    let cle = format!("{P}/idee");
    for (sorte, conteneur, message) in [
        (
            Sorte::MidiVerseSeul,
            "midi_verses",
            "Idée du 14 novembre : MIDI versé regroupé, idée du 14 novembre retirée",
        ),
        (
            Sorte::TexteVerse,
            "textes_verses",
            "Idée du 14 novembre : texte versé regroupé, idée du 14 novembre retirée",
        ),
    ] {
        let geste = regrouper(&captation(sorte, V), &idee(V), 1, &idee(A), HEURE, 0).unwrap();
        assert_eq!(geste.operation.message, message);
        assert_eq!(
            geste.operation.changements,
            vec![
                poser(&[conteneur, cle.as_str()], Simple::Chaine(A.into())),
                poser(&["idees", V, "retiree"], Simple::Entier(HEURE)),
            ]
        );
        assert_eq!(
            geste.inverse,
            vec![
                poser(&[conteneur, cle.as_str()], Simple::Chaine(V.into())),
                oter(&["idees", V, "retiree"]),
            ]
        );
    }
}

#[test]
fn retirer_pose_la_date_du_retrait_et_son_inverse_l_ote() {
    // Retirée le lendemain de sa naissance : la date du retrait est l'heure du geste, et le
    // message dit le jour où l'idée est née.
    let le_lendemain = HEURE + 86_400;
    let geste = retirer(&idee(A), le_lendemain, 0).unwrap();
    assert_eq!(geste.operation.message, "Idée du 14 novembre retirée");
    assert_eq!(geste.operation.heure, le_lendemain);
    assert_eq!(
        geste.operation.changements,
        vec![poser(
            &["idees", A, "retiree"],
            Simple::Entier(le_lendemain)
        )]
    );
    assert_eq!(geste.inverse, vec![oter(&["idees", A, "retiree"])]);
}

#[test]
fn retablir_ote_la_marque_et_son_inverse_repose_la_date_d_avant() {
    let mut a = idee(A);
    a.retiree = Some(HEURE - 3_600);
    let geste = retablir(&a, HEURE + 7_200, 0).unwrap();
    assert_eq!(geste.operation.message, "Idée du 14 novembre rétablie");
    assert_eq!(geste.operation.heure, HEURE + 7_200);
    assert_eq!(
        geste.operation.changements,
        vec![oter(&["idees", A, "retiree"])]
    );
    assert_eq!(
        geste.inverse,
        vec![poser(
            &["idees", A, "retiree"],
            Simple::Entier(HEURE - 3_600)
        )]
    );
}

#[test]
fn presente_rend_l_idee_lue_ou_la_dit_absente() {
    assert_eq!(presente(A, Some(idee(A))), Ok(idee(A)));
    let absente = presente(A, None);
    assert_eq!(absente, Err(Impossible::IdeeAbsente { idee: A.into() }));
    assert_eq!(
        absente.unwrap_err().to_string(),
        "Aucune idée ne porte l'identifiant 01HF7YAT00000000000000000A."
    );
}

#[test]
fn un_geste_de_tri_qui_touche_une_idee_retiree_est_refuse() {
    // A, retirée, garde son texte, son étiquette et son lien avec B : chaque geste de tri qui la
    // touche est refusé, d'un bout comme de l'autre ; C, visible, n'est reliée à rien.
    let mut a = idee(A);
    a.retiree = Some(HEURE);
    a.texte = Some("Un pont en la mineur".into());
    a.etiquettes.insert("pont".into());
    a.liens.insert(LIEN_AB.into());
    let mut b = idee(B);
    b.liens.insert(LIEN_AB.into());
    let c = idee(C);
    let retiree = Err(Impossible::IdeeRetiree { idee: A.into() });
    assert_eq!(completer(&a, "Un autre texte", HEURE, 0), retiree);
    assert_eq!(poser_une_etiquette(&a, "refrain", HEURE, 0), retiree);
    assert_eq!(oter_une_etiquette(&a, "pont", HEURE, 0), retiree);
    assert_eq!(relier(&a, &c, HEURE, 0), retiree);
    assert_eq!(relier(&c, &a, HEURE, 0), retiree);
    assert_eq!(delier(&a, &b, HEURE, 0), retiree);
    assert_eq!(delier(&b, &a, HEURE, 0), retiree);
    assert_eq!(
        regrouper(&captation(Sorte::Prise, A), &a, 2, &c, HEURE, 0),
        retiree
    );
    assert_eq!(
        regrouper(&captation(Sorte::Prise, C), &c, 2, &a, HEURE, 0),
        retiree
    );
    assert_eq!(retirer(&a, HEURE, 0), retiree);
    assert_eq!(
        Impossible::IdeeRetiree { idee: A.into() }.to_string(),
        "L'idée 01HF7YAT00000000000000000A est retirée."
    );
}

#[test]
fn retablir_une_idee_qui_n_est_pas_retiree_est_refuse() {
    let refus = retablir(&idee(A), HEURE, 0);
    assert_eq!(refus, Err(Impossible::IdeeNonRetiree { idee: A.into() }));
    assert_eq!(
        refus.unwrap_err().to_string(),
        "L'idée 01HF7YAT00000000000000000A n'est pas retirée."
    );
}

#[test]
fn completer_d_un_texte_vide_est_refuse() {
    let mut a = idee(A);
    a.texte = Some("Un pont en la mineur".into());
    assert_eq!(completer(&a, "", HEURE, 0), Err(Impossible::TexteVide));
    assert_eq!(completer(&a, " \t\n", HEURE, 0), Err(Impossible::TexteVide));
    assert_eq!(
        completer(&idee(A), "", HEURE, 0),
        Err(Impossible::TexteVide)
    );
    assert_eq!(Impossible::TexteVide.to_string(), "Le texte est vide.");
}

#[test]
fn completer_du_meme_texte_est_refuse() {
    let mut a = idee(A);
    a.texte = Some("Un pont en la mineur".into());
    let refus = completer(&a, "Un pont en la mineur", HEURE, 0);
    assert_eq!(refus, Err(Impossible::MemeTexte));
    assert_eq!(
        refus.unwrap_err().to_string(),
        "Le texte est déjà celui-ci."
    );
}

#[test]
fn une_etiquette_vide_est_refusee() {
    assert_eq!(
        poser_une_etiquette(&idee(A), "", HEURE, 0),
        Err(Impossible::EtiquetteVide)
    );
    assert_eq!(
        oter_une_etiquette(&idee(A), "", HEURE, 0),
        Err(Impossible::EtiquetteVide)
    );
    assert_eq!(
        Impossible::EtiquetteVide.to_string(),
        "L'étiquette est vide."
    );
}

#[test]
fn poser_une_etiquette_deja_posee_est_refuse() {
    let mut a = idee(A);
    a.etiquettes.insert("pont".into());
    let refus = poser_une_etiquette(&a, "pont", HEURE, 0);
    assert_eq!(
        refus,
        Err(Impossible::EtiquetteDejaPosee {
            etiquette: "pont".into()
        })
    );
    assert_eq!(
        refus.unwrap_err().to_string(),
        "L'étiquette #pont est déjà posée."
    );
}

#[test]
fn oter_une_etiquette_qui_n_est_pas_posee_est_refuse() {
    let refus = oter_une_etiquette(&idee(A), "pont", HEURE, 0);
    assert_eq!(
        refus,
        Err(Impossible::EtiquetteAbsente {
            etiquette: "pont".into()
        })
    );
    assert_eq!(
        refus.unwrap_err().to_string(),
        "L'étiquette #pont n'est pas posée."
    );
}

#[test]
fn deux_formes_d_une_meme_etiquette_font_deux_cles() {
    // « été » composé (NFC, U+00E9) et décomposé (NFD, « e » suivi de U+0301) s'affichent
    // pareil : l'étiquette se compare octet pour octet, sans normalisation, et chaque forme a
    // sa clé.
    let nfc = "\u{e9}t\u{e9}";
    let nfd = "e\u{301}te\u{301}";
    assert_ne!(nfc.as_bytes(), nfd.as_bytes());
    let composee = poser_une_etiquette(&idee(A), nfc, HEURE, 0).unwrap();
    let decomposee = poser_une_etiquette(&idee(A), nfd, HEURE, 0).unwrap();
    assert_eq!(
        composee.operation.changements,
        vec![poser(&["idees", A, "etiquettes", nfc], Simple::Vrai)]
    );
    assert_eq!(
        decomposee.operation.changements,
        vec![poser(&["idees", A, "etiquettes", nfd], Simple::Vrai)]
    );
    assert_ne!(
        composee.operation.changements,
        decomposee.operation.changements
    );
    // La forme composée posée, la décomposée ne l'est pas : elle se pose, et ne s'ôte pas.
    let mut a = idee(A);
    a.etiquettes.insert(nfc.into());
    assert!(poser_une_etiquette(&a, nfd, HEURE, 0).is_ok());
    assert_eq!(
        oter_une_etiquette(&a, nfd, HEURE, 0),
        Err(Impossible::EtiquetteAbsente {
            etiquette: nfd.into()
        })
    );
}

#[test]
fn relier_une_idee_a_elle_meme_est_refuse() {
    let refus = relier(&idee(A), &idee(A), HEURE, 0);
    assert_eq!(refus, Err(Impossible::RelieeAElleMeme));
    assert_eq!(
        refus.unwrap_err().to_string(),
        "Une idée ne se relie pas à elle-même."
    );
}

#[test]
fn relier_deux_idees_deja_reliees_est_refuse() {
    let (mut a, mut b) = (idee(A), idee(B));
    a.liens.insert(LIEN_AB.into());
    b.liens.insert(LIEN_AB.into());
    assert_eq!(relier(&a, &b, HEURE, 0), Err(Impossible::LienDejaPose));
    assert_eq!(relier(&b, &a, HEURE, 0), Err(Impossible::LienDejaPose));
    assert_eq!(
        Impossible::LienDejaPose.to_string(),
        "Ces deux idées sont déjà reliées."
    );
}

#[test]
fn delier_deux_idees_qui_ne_sont_pas_reliees_est_refuse() {
    let refus = delier(&idee(A), &idee(B), HEURE, 0);
    assert_eq!(refus, Err(Impossible::LienAbsent));
    assert_eq!(
        refus.unwrap_err().to_string(),
        "Ces deux idées ne sont pas reliées."
    );
}

#[test]
fn regrouper_une_captation_dans_son_idee_est_refuse() {
    let refus = regrouper(&captation(Sorte::Prise, A), &idee(A), 2, &idee(A), HEURE, 0);
    assert_eq!(refus, Err(Impossible::DejaDansLIdee));
    assert_eq!(
        refus.unwrap_err().to_string(),
        "Cette captation est déjà dans cette idée."
    );
}

#[test]
fn une_captation_absente_se_dit_par_son_empreinte() {
    assert_eq!(
        Impossible::CaptationAbsente {
            empreinte: P.into()
        }
        .to_string(),
        format!("Aucune captation ne porte l'empreinte {P}.")
    );
}

#[test]
fn annuler_rend_l_inverse_sous_annule() {
    let geste = poser_une_etiquette(&idee(A), "pont", HEURE, 0).unwrap();
    let plus_tard = HEURE + 60;
    assert_eq!(
        annulation(&geste, plus_tard),
        Operation {
            changements: vec![oter(&["idees", A, "etiquettes", "pont"])],
            message: "Annulé : Idée du 14 novembre : étiquette #pont posée".into(),
            heure: plus_tard,
        }
    );
    // L'inverse d'un regroupement qui vide sa source : la captation rendue, le retrait ôté.
    let regroupement =
        regrouper(&captation(Sorte::Prise, V), &idee(V), 1, &idee(A), HEURE, 0).unwrap();
    let annule = annulation(&regroupement, plus_tard);
    assert_eq!(annule.changements, regroupement.inverse);
    assert_eq!(
        annule.message,
        "Annulé : Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée"
    );
}

#[test]
fn verser_une_prise_cree_son_idee_et_sa_fiche() {
    // verser la prise P, importée par l'établi, sous l'idée V qui naît avec elle : l'heure et
    // l'appareil du versement sont ceux de l'idée ; la date et l'appareil de la prise restent « — ».
    let op = verser(
        Sorte::Prise,
        P,
        &wav(30_000),
        V,
        &importee(),
        None,
        HEURE,
        0,
    )
    .unwrap();
    assert_eq!(op.message, "Idée du 14 novembre : prise versée");
    assert_eq!(op.heure, HEURE);
    assert_eq!(
        op.changements,
        vec![
            Changement::Creer {
                chemin: chemin(&["idees", V]),
                conteneurs: vec!["etiquettes".into()]
            },
            poser(
                &["idees", V, "provenance"],
                Simple::Chaine("importee".into())
            ),
            poser(&["idees", V, "par"], Simple::Chaine("etabli".into())),
            poser(&["idees", V, "le"], Simple::Entier(HEURE)),
            poser(&["idees", V, "depuis"], Simple::Chaine(IMPORT.into())),
            poser(&["prises", &format!("{P}/idee")], Simple::Chaine(V.into())),
            poser(
                &["prises", &format!("{P}/format")],
                Simple::Chaine("wav".into())
            ),
            poser(&["prises", &format!("{P}/duree")], Simple::Naturel(30_000)),
        ]
    );
    // Une prise captée se verse aussi, et son idée n'a pas de `depuis`.
    let captee = verser(
        Sorte::Prise,
        P,
        &wav(30_000),
        V,
        &Provenance::Captee {
            par: Appareil::Carnet,
        },
        None,
        HEURE,
        0,
    )
    .unwrap();
    assert_eq!(
        captee.changements,
        vec![
            Changement::Creer {
                chemin: chemin(&["idees", V]),
                conteneurs: vec!["etiquettes".into()]
            },
            poser(&["idees", V, "provenance"], Simple::Chaine("captee".into())),
            poser(&["idees", V, "par"], Simple::Chaine("carnet".into())),
            poser(&["idees", V, "le"], Simple::Entier(HEURE)),
            poser(&["prises", &format!("{P}/idee")], Simple::Chaine(V.into())),
            poser(
                &["prises", &format!("{P}/format")],
                Simple::Chaine("wav".into())
            ),
            poser(&["prises", &format!("{P}/duree")], Simple::Naturel(30_000)),
        ]
    );
}

#[test]
fn verser_un_midi_n_a_pas_de_duree() {
    for (sorte, format) in [(Sorte::MidiVerseSeul, "smf1"), (Sorte::TexteVerse, "brut")] {
        let fiche = Fiche {
            format: format.into(),
            duree_ms: Some(30_000),
        };
        assert_eq!(
            verser(sorte, P, &fiche, V, &importee(), None, HEURE, 0),
            Err(Impossible::DureeHorsDUnePrise)
        );
    }
    assert_eq!(
        Impossible::DureeHorsDUnePrise.to_string(),
        "Seule une prise a une durée."
    );
}

#[test]
fn verser_un_midi_ou_un_texte_ne_pose_que_son_idee_et_son_format() {
    for (sorte, conteneur, format, message) in [
        (
            Sorte::MidiVerseSeul,
            "midi_verses",
            "smf1",
            "Idée du 14 novembre : MIDI versé",
        ),
        (
            Sorte::TexteVerse,
            "textes_verses",
            "brut",
            "Idée du 14 novembre : texte versé",
        ),
    ] {
        let fiche = Fiche {
            format: format.into(),
            duree_ms: None,
        };
        let op = verser(sorte, P, &fiche, V, &importee(), None, HEURE, 0).unwrap();
        assert_eq!(op.message, message);
        assert_eq!(op.heure, HEURE);
        assert_eq!(
            op.changements,
            vec![
                Changement::Creer {
                    chemin: chemin(&["idees", V]),
                    conteneurs: vec!["etiquettes".into()]
                },
                poser(
                    &["idees", V, "provenance"],
                    Simple::Chaine("importee".into())
                ),
                poser(&["idees", V, "par"], Simple::Chaine("etabli".into())),
                poser(&["idees", V, "le"], Simple::Entier(HEURE)),
                poser(&["idees", V, "depuis"], Simple::Chaine(IMPORT.into())),
                poser(&[conteneur, &format!("{P}/idee")], Simple::Chaine(V.into())),
                poser(
                    &[conteneur, &format!("{P}/format")],
                    Simple::Chaine(format.into())
                ),
            ]
        );
    }
}

#[test]
fn verser_un_midi_ou_un_texte_capte_est_refuse() {
    let captee = Provenance::Captee {
        par: Appareil::Carnet,
    };
    for (sorte, format) in [(Sorte::MidiVerseSeul, "smf1"), (Sorte::TexteVerse, "brut")] {
        let fiche = Fiche {
            format: format.into(),
            duree_ms: None,
        };
        assert_eq!(
            verser(sorte, P, &fiche, V, &captee, None, HEURE, 0),
            Err(Impossible::HorsDeLImport)
        );
    }
    assert_eq!(
        Impossible::HorsDeLImport.to_string(),
        "Un MIDI versé ou un texte versé n'entre que par l'import."
    );
}

#[test]
fn verser_une_fiche_deja_versee_est_refuse() {
    let existante = CaptationLue {
        sorte: Sorte::Prise,
        empreinte: P.into(),
        idee: A.into(),
        duree_ms: Some(30_000),
    };
    let refus = verser(
        Sorte::Prise,
        P,
        &wav(30_000),
        V,
        &importee(),
        Some(&existante),
        HEURE,
        0,
    );
    assert_eq!(
        refus,
        Err(Impossible::CaptationDejaVersee {
            empreinte: P.into()
        })
    );
    assert_eq!(
        refus.unwrap_err().to_string(),
        format!("La captation {P} est déjà versée.")
    );
}

#[test]
fn verser_une_duree_au_dela_d_entier_sur_est_refusee() {
    assert_eq!(ENTIER_SUR, (1u64 << 53) - 1);
    let au_plus = verser(
        Sorte::Prise,
        P,
        &wav(ENTIER_SUR),
        V,
        &importee(),
        None,
        HEURE,
        0,
    )
    .unwrap();
    assert!(au_plus.changements.contains(&poser(
        &["prises", &format!("{P}/duree")],
        Simple::Naturel(ENTIER_SUR)
    )));
    let au_dela = verser(
        Sorte::Prise,
        P,
        &wav(ENTIER_SUR + 1),
        V,
        &importee(),
        None,
        HEURE,
        0,
    );
    assert_eq!(
        au_dela,
        Err(Impossible::DureeTropLongue {
            duree_ms: ENTIER_SUR + 1
        })
    );
    assert_eq!(
        au_dela.unwrap_err().to_string(),
        "Une durée de 9007199254740992 ms passe le plus grand entier que l'interface relit à \
         l'identique."
    );
}
