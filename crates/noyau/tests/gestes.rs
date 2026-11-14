//! Les gestes du noyau (§ 31 · 3) : la date d'un message, au décalage local reçu, la naissance,
//! et noter, que le noyau rend en changements sans rien écrire.

use noyau::donnees::changement::{Changement, Simple};
use noyau::donnees::gestes::{Appareil, Impossible, Sorte, date_du_message, naissance, noter};

const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023, 22 h 13 min 20 s UTC

fn chemin(segments: &[&str]) -> Vec<String> {
    segments.iter().map(|segment| segment.to_string()).collect()
}

fn poser(segments: &[&str], valeur: Simple) -> Changement {
    Changement::Poser {
        chemin: chemin(segments),
        valeur,
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
    // Les termes de siècle de l'algorithme (doe / 36 524, doe / 146 096) ne jouent qu'en fin d'ère.
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
    assert_eq!(
        Impossible::TexteVide.to_string(),
        "Le texte de la note est vide."
    );
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
