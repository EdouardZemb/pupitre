//! Ce qui est reçu s'écrit (§ 31 · 2.1, § 31 · 4.4) : ce qu'apporte un message de
//! synchronisation n'est reçu qu'une fois son fichier d'ajout en place, avant que l'échange ne
//! continue ; un message qui n'apporte rien n'écrit rien ; un document à relire n'envoie rien,
//! ne reçoit rien et ne se compacte pas.
//!
//! B est une copie de la racine des données de A, faite A fermée, comme le carnet partira de
//! l'établi (§ 31 · 2.3). Le fonds naît d'une idée au texte long, qui ne sert qu'à la
//! naissance : les écritures des tests sont courtes, et leurs fichiers d'ajout pèsent bien moins
//! que le fichier complet.

mod commun;

use std::fs;

use commun::{
    HEURE, NAISSANCE, deux_copies, dossier_des_ajouts, dossier_du_fonds, ouvrir_le_fonds,
};
use stockage::{Erreur, RacineDesDonnees, empreinte};
use synchro::{Copie, Echange};

/// Le message de l'écriture que le test fait refuser.
const MESSAGE_REFUSE: &str = "Texte changé";

#[test]
fn la_copie_se_lit_a_la_racine_du_document() {
    let (chemin_a, chemin_b) = deux_copies("synchronisation-copie");
    let racine_a = RacineDesDonnees::ouvrir(&chemin_a).expect("ouvrir la racine des données d'A");
    let racine_b = RacineDesDonnees::ouvrir(&chemin_b).expect("ouvrir la racine des données de B");
    let a = ouvrir_le_fonds(&racine_a);
    let b = ouvrir_le_fonds(&racine_b);

    let attendue = Copie {
        proprietaire: NAISSANCE.proprietaire.to_string(),
        document: NAISSANCE.identifiant.to_string(),
        lignee: NAISSANCE.lignee.to_string(),
    };
    assert_eq!(a.copie().expect("la copie d'A"), attendue);
    assert_eq!(b.copie().expect("la copie de B"), attendue);
}

#[test]
fn ce_qui_est_recu_se_relit_du_disque() {
    let (chemin_a, chemin_b) = deux_copies("synchronisation-recu");
    let racine_a = RacineDesDonnees::ouvrir(&chemin_a).expect("ouvrir la racine des données d'A");
    let racine_b = RacineDesDonnees::ouvrir(&chemin_b).expect("ouvrir la racine des données de B");
    let mut a = ouvrir_le_fonds(&racine_a);
    let mut b = ouvrir_le_fonds(&racine_b);
    b.ecrire("Étiquette posée", HEURE + 1, |tx| {
        commun::etiqueter(tx, "idee-1", "refrain")
    })
    .expect("la première écriture de B");
    b.ecrire("Idée notée", HEURE + 2, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    })
    .expect("la seconde écriture de B");
    let ajouts_a = dossier_des_ajouts(&chemin_a);

    // La boucle de `commun::reunir`, déroulée pour relire le disque d'A après chaque réception.
    let mut ea = Echange::ouvrir(
        &a.copie().expect("la copie d'A"),
        &b.copie().expect("la copie de B"),
    )
    .expect("ouvrir l'échange d'A");
    let mut eb = Echange::ouvrir(
        &b.copie().expect("la copie de B"),
        &a.copie().expect("la copie d'A"),
    )
    .expect("ouvrir l'échange de B");
    let mut messages = 0;
    let mut heads_changees = false;
    loop {
        let de_a = a.prochain_message(&mut ea);
        if let Some(m) = &de_a {
            b.recevoir(&mut eb, m).expect("B reçoit le message d'A");
            messages += 1;
        }
        let de_b = b.prochain_message(&mut eb);
        if let Some(m) = &de_b {
            let heads_d_avant = a.automerge().get_heads();
            let ajouts_d_avant = commun::entrees(&ajouts_a);
            a.recevoir(&mut ea, m).expect("A reçoit le message de B");
            messages += 1;
            assert_eq!(
                commun::relire_le_dossier_du_fonds(&chemin_a).get_heads(),
                a.automerge().get_heads(),
                "après le message {messages}, le disque d'A ne mène pas à ses heads"
            );
            if !heads_changees && a.automerge().get_heads() != heads_d_avant {
                heads_changees = true;
                let ajouts = commun::entrees(&ajouts_a);
                assert_eq!(
                    ajouts.len(),
                    ajouts_d_avant.len() + 1,
                    "la réception n'a pas ajouté un seul fichier : {ajouts_d_avant:?}, puis {ajouts:?}"
                );
                let nouveaux: Vec<String> = ajouts
                    .into_iter()
                    .filter(|nom| !ajouts_d_avant.contains(nom))
                    .collect();
                assert_eq!(nouveaux.len(), 1, "un seul fichier nouveau : {nouveaux:?}");
                let contenu =
                    fs::read(ajouts_a.join(&nouveaux[0])).expect("lire le fichier d'ajout reçu");
                assert_eq!(
                    nouveaux[0],
                    format!("{}.automerge", empreinte(&contenu)),
                    "le nom du fichier d'ajout reçu n'est pas l'empreinte de son contenu"
                );
            }
        }
        if de_a.is_none() && de_b.is_none() {
            break;
        }
        assert!(messages <= 1000, "la réunion ne s'achève pas");
    }
    assert!(
        heads_changees,
        "aucun message de B n'a changé les heads d'A"
    );
    let heads_de_b = b.automerge().get_heads();
    // A fermée, puis rouverte, comme au lancement suivant.
    drop(a);
    drop(racine_a);

    let racine_a = RacineDesDonnees::ouvrir(&chemin_a).expect("rouvrir la racine des données d'A");
    let a = ouvrir_le_fonds(&racine_a);
    assert_eq!(
        a.automerge().get_heads(),
        heads_de_b,
        "A rouverte n'a pas les heads de B"
    );
}

#[test]
fn un_message_sans_rien_de_neuf_n_ecrit_rien() {
    let (chemin_a, chemin_b) = deux_copies("synchronisation-rien-de-neuf");
    let racine_a = RacineDesDonnees::ouvrir(&chemin_a).expect("ouvrir la racine des données d'A");
    let racine_b = RacineDesDonnees::ouvrir(&chemin_b).expect("ouvrir la racine des données de B");
    let mut a = ouvrir_le_fonds(&racine_a);
    let mut b = ouvrir_le_fonds(&racine_b);
    assert_eq!(
        a.automerge().get_heads(),
        b.automerge().get_heads(),
        "A et B ne sont pas identiques"
    );
    let fonds_a = commun::empreintes(&dossier_du_fonds(&chemin_a));
    let fonds_b = commun::empreintes(&dossier_du_fonds(&chemin_b));
    let ajouts_a = commun::entrees(&dossier_des_ajouts(&chemin_a));
    let ajouts_b = commun::entrees(&dossier_des_ajouts(&chemin_b));

    let messages = commun::reunir(&mut a, &mut b);

    assert!(messages > 0, "la réunion n'a échangé aucun message");
    assert_eq!(
        commun::entrees(&dossier_des_ajouts(&chemin_a)),
        ajouts_a,
        "un message sans rien de neuf a écrit dans ajouts/ d'A"
    );
    assert_eq!(
        commun::entrees(&dossier_des_ajouts(&chemin_b)),
        ajouts_b,
        "un message sans rien de neuf a écrit dans ajouts/ de B"
    );
    assert_eq!(
        commun::empreintes(&dossier_du_fonds(&chemin_a)),
        fonds_a,
        "un message sans rien de neuf a touché un fichier du fonds d'A"
    );
    assert_eq!(
        commun::empreintes(&dossier_du_fonds(&chemin_b)),
        fonds_b,
        "un message sans rien de neuf a touché un fichier du fonds de B"
    );
}

#[test]
fn un_document_a_relire_ne_laisse_rien_partir() {
    let (chemin_a, chemin_b) = deux_copies("synchronisation-a-relire");
    let racine_a = RacineDesDonnees::ouvrir(&chemin_a).expect("ouvrir la racine des données d'A");
    let racine_b = RacineDesDonnees::ouvrir(&chemin_b).expect("ouvrir la racine des données de B");
    let mut a = ouvrir_le_fonds(&racine_a);
    let mut b = ouvrir_le_fonds(&racine_b);
    a.ecrire("Étiquette posée", HEURE + 1, |tx| {
        commun::etiqueter(tx, "idee-1", "refrain")
    })
    .expect("l'écriture d'A");
    b.ecrire("Idée notée", HEURE + 2, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    })
    .expect("l'écriture de B");
    // `ajouts/` d'A mis de côté, à côté de sa racine des données, et un fichier à sa place :
    // aucun fichier d'ajout ne s'y écrit.
    let ajouts = dossier_des_ajouts(&chemin_a);
    let de_cote = chemin_a.with_file_name("ajouts-d-a-mis-de-cote");
    fs::rename(&ajouts, &de_cote).expect("mettre ajouts/ de côté");
    fs::write(&ajouts, b"").expect("poser un fichier à la place de ajouts/");
    let dossier = dossier_du_fonds(&chemin_a);
    let avant = commun::empreintes(&dossier);

    let refus = a.ecrire(MESSAGE_REFUSE, HEURE + 3, |tx| {
        commun::changer_le_texte(tx, "idee-1", "Un refrain plus court")
    });
    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(()) => panic!("le fichier d'ajout s'est écrit sans ajouts/"),
    }
    let mut ea = Echange::ouvrir(
        &a.copie().expect("la copie d'A"),
        &b.copie().expect("la copie de B"),
    )
    .expect("ouvrir l'échange d'A");
    let mut eb = Echange::ouvrir(
        &b.copie().expect("la copie de B"),
        &a.copie().expect("la copie d'A"),
    )
    .expect("ouvrir l'échange de B");
    assert_eq!(
        a.prochain_message(&mut ea),
        None,
        "un document à relire a envoyé un message"
    );
    let de_b = b.prochain_message(&mut eb).expect("le message de B pour A");
    match a.recevoir(&mut ea, &de_b) {
        Err(Erreur::ARelire) => {}
        Err(autre) => panic!("attendu ARelire à la réception, rendu {autre:?}"),
        Ok(()) => panic!("un document à relire a reçu un message"),
    }
    match a.compacter() {
        Err(Erreur::ARelire) => {}
        Err(autre) => panic!("attendu ARelire à la compaction, rendu {autre:?}"),
        Ok(()) => panic!("un document à relire s'est compacté"),
    }
    assert_eq!(
        commun::empreintes(&dossier),
        avant,
        "un document à relire a touché un fichier du fonds"
    );
    drop(a);

    fs::remove_file(&ajouts).expect("ôter le fichier posé à la place de ajouts/");
    fs::rename(&de_cote, &ajouts).expect("rendre ajouts/");
    let mut a = ouvrir_le_fonds(&racine_a);
    commun::reunir(&mut a, &mut b);

    assert_eq!(
        a.automerge().get_heads(),
        b.automerge().get_heads(),
        "A rouverte et B ne se sont pas réunies"
    );
    for (nom, doc) in [("A", a.automerge()), ("B", b.automerge())] {
        assert!(
            doc.get_changes(&[])
                .iter()
                .all(|modification| modification.message() != Some(MESSAGE_REFUSE)),
            "{nom} porte le message de l'écriture refusée"
        );
    }
}
