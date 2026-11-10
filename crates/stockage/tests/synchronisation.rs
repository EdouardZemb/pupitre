//! Ce qui est reçu s'écrit (§ 31 · 2.1, § 31 · 4.4) : ce qu'apporte un message de
//! synchronisation n'est reçu qu'une fois son fichier d'ajout en place, avant que l'échange ne
//! continue ; un message qui n'apporte rien n'écrit rien ; une réception refusée en chemin écrit
//! ce qu'elle a apporté ; une réception dont l'écriture est refusée relit le document du disque,
//! et l'échange reprend ; un message illisible est refusé, et rien ne s'écrit.
//!
//! B est une copie de la racine des données de A, faite A fermée, comme le carnet partira de
//! l'établi (§ 31 · 2.3). Le fonds naît d'une idée au texte long, qui ne sert qu'à la
//! naissance : les écritures des tests sont courtes, et leurs fichiers d'ajout pèsent bien moins
//! que le fichier complet.

mod commun;

use std::fs;
use std::path::Path;

use automerge::ChangeHash;
use commun::{
    HEURE, NAISSANCE, deux_copies, dossier_des_ajouts, dossier_du_fonds, faire_naitre, ouvrir,
    ouvrir_le_fonds, porte_l_idee,
};
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Document, Erreur, RacineDesDonnees, empreinte};
use synchro::{Copie, Echange};

/// Le fichier d'ajout qu'une réception d'`a` a écrit dans `ajouts`, que la réception a fait
/// passer des noms `avant` aux noms `apres` : un seul fichier de plus, nommé par l'empreinte de son
/// contenu, qui vaut `save_after` des heads `heads_d_avant` d'`a`, celles d'avant la réception.
fn constater_l_ajout_recu(
    a: &Document<'_>,
    heads_d_avant: &[ChangeHash],
    ajouts: &Path,
    avant: &[String],
    apres: &[String],
) {
    assert_eq!(
        apres.len(),
        avant.len() + 1,
        "la réception n'a pas ajouté un seul fichier : {avant:?}, puis {apres:?}"
    );
    let nouveaux = commun::nouveaux(avant, apres);
    assert_eq!(nouveaux.len(), 1, "un seul fichier nouveau : {nouveaux:?}");
    let contenu = fs::read(ajouts.join(&nouveaux[0])).expect("lire le fichier d'ajout reçu");
    assert_eq!(
        nouveaux[0],
        format!("{}.automerge", empreinte(&contenu)),
        "le nom du fichier d'ajout reçu n'est pas l'empreinte de son contenu"
    );
    assert!(
        contenu == a.automerge().save_after(heads_d_avant),
        "le fichier d'ajout reçu n'est pas save_after des heads d'avant la réception"
    );
}

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

    // Le disque d'A se relit après chaque réception.
    let (mut ea, mut eb) = commun::ouvrir_les_echanges(&a, &b);
    let mut ajouts_d_avant = commun::entrees(&ajouts_a);
    let mut recues = 0;
    let mut heads_changees = false;
    commun::reunir_en_suivant_a(
        &mut a,
        &mut ea,
        &mut b,
        &mut eb,
        |a, heads_d_avant, rendu| {
            rendu.expect("A reçoit le message de B");
            recues += 1;
            assert_eq!(
                commun::relire_le_dossier_du_fonds(&chemin_a).get_heads(),
                a.automerge().get_heads(),
                "après la réception {recues}, le disque d'A ne mène pas à ses heads"
            );
            let ajouts = commun::entrees(&ajouts_a);
            if !heads_changees && a.automerge().get_heads() != heads_d_avant {
                heads_changees = true;
                constater_l_ajout_recu(a, heads_d_avant, &ajouts_a, &ajouts_d_avant, &ajouts);
            }
            ajouts_d_avant = ajouts;
        },
    );
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
fn une_reception_refusee_en_chemin_ecrit_ce_qui_a_change() {
    // La réception de ce que B a noté est refusée une fois appliquée : ce qu'elle a apporté
    // s'écrit, et se relit.
    {
        let (chemin_a, chemin_b) = deux_copies("synchronisation-refusee-en-chemin");
        let racine_a = ouvrir(&chemin_a);
        let racine_b = ouvrir(&chemin_b);
        let mut a = ouvrir_le_fonds(&racine_a);
        let mut b = ouvrir_le_fonds(&racine_b);
        commun::ecrire(&mut b, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
        .expect("l'écriture de B");
        let ajouts_a = dossier_des_ajouts(&chemin_a);
        let (mut ea, mut eb) = commun::ouvrir_les_echanges(&a, &b);
        let mut ajouts_d_avant = commun::entrees(&ajouts_a);
        let mut refusees = 0;

        {
            let _panne = couper(Etape::Reception, 1);
            commun::reunir_en_suivant_a(
                &mut a,
                &mut ea,
                &mut b,
                &mut eb,
                |a, heads_d_avant, rendu| {
                    let ajouts = commun::entrees(&ajouts_a);
                    match rendu {
                        Err(Erreur::Synchro(_)) => {
                            refusees += 1;
                            assert_ne!(
                                a.automerge().get_heads(),
                                heads_d_avant,
                                "la réception refusée n'a pas changé les heads d'A"
                            );
                            constater_l_ajout_recu(
                                a,
                                heads_d_avant,
                                &ajouts_a,
                                &ajouts_d_avant,
                                &ajouts,
                            );
                        }
                        Ok(_) => {}
                        Err(autre) => panic!("attendu Synchro à la réception, rendu {autre:?}"),
                    }
                    ajouts_d_avant = ajouts;
                },
            );
        }

        assert_eq!(
            refusees, 1,
            "la réception qui change les heads d'A n'a pas été refusée"
        );
        assert!(
            porte_l_idee(a.automerge(), "idee-2"),
            "A ne porte pas l'idée reçue"
        );
        drop(a);
        let a = ouvrir_le_fonds(&racine_a);
        assert!(
            porte_l_idee(a.automerge(), "idee-2"),
            "A rouverte ne porte pas l'idée reçue"
        );
    }

    // Sur deux copies neuves : l'écriture de ce qui est reçu est en place, mais non durable ; le
    // refus de la réception se rend, et l'avertissement rejoint ceux du document. Le premier
    // `rendre_durable` est celui du fichier d'ajout de cette réception.
    let (chemin_a, chemin_b) = deux_copies("synchronisation-refusee-non-durable");
    let racine_a = ouvrir(&chemin_a);
    let racine_b = ouvrir(&chemin_b);
    let mut a = ouvrir_le_fonds(&racine_a);
    let mut b = ouvrir_le_fonds(&racine_b);
    commun::ecrire(&mut b, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    })
    .expect("l'écriture de B");
    let ajouts_a = dossier_des_ajouts(&chemin_a);
    let (mut ea, mut eb) = commun::ouvrir_les_echanges(&a, &b);
    let mut refusees = 0;

    {
        let _reception = couper(Etape::Reception, 1);
        let _rendre_durable = couper(Etape::RendreDurable, 1);
        commun::reunir_en_suivant_a(
            &mut a,
            &mut ea,
            &mut b,
            &mut eb,
            |_, _, rendu| match rendu {
                Err(Erreur::Synchro(_)) => refusees += 1,
                Ok(_) => {}
                Err(autre) => panic!("attendu Synchro à la réception, rendu {autre:?}"),
            },
        );
    }

    assert_eq!(
        refusees, 1,
        "la réception qui change les heads d'A n'a pas été refusée"
    );
    match a.avertissements() {
        [Avertissement::NonDurable { chemin, .. }] => {
            assert_eq!(
                chemin.parent(),
                Some(ajouts_a.as_path()),
                "l'avertissement ne nomme pas un fichier d'ajout : {}",
                chemin.display()
            );
            assert!(
                chemin.is_file(),
                "le fichier d'ajout reçu n'est pas en place"
            );
        }
        autres => panic!("attendu un seul NonDurable, rendu {autres:?}"),
    }
}

#[test]
fn une_reception_refusee_relit_le_document_et_l_echange_reprend() {
    let (chemin_a, chemin_b) = deux_copies("synchronisation-reception-refusee");
    let racine_a = ouvrir(&chemin_a);
    let racine_b = ouvrir(&chemin_b);
    let mut a = ouvrir_le_fonds(&racine_a);
    let mut b = ouvrir_le_fonds(&racine_b);
    commun::ecrire(&mut b, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    })
    .expect("l'écriture de B");
    let ajouts_a = dossier_des_ajouts(&chemin_a);
    let (mut ea, mut eb) = commun::ouvrir_les_echanges(&a, &b);
    let mut ajouts_d_avant = commun::entrees(&ajouts_a);
    let mut refusees = 0;
    let mut ecrites_apres_le_refus = 0;

    // Les mêmes échanges, jusqu'à ce que ni l'une ni l'autre n'ait rien à dire : B renvoie ce
    // qu'elle croyait envoyé.
    {
        let _panne = couper(Etape::Renommage, 1);
        commun::reunir_en_suivant_a(
            &mut a,
            &mut ea,
            &mut b,
            &mut eb,
            |a, heads_d_avant, rendu| {
                let ajouts = commun::entrees(&ajouts_a);
                let heads_changees = a.automerge().get_heads() != heads_d_avant;
                match rendu {
                    Err(Erreur::Disque { .. }) => {
                        refusees += 1;
                        assert!(
                            !heads_changees,
                            "après la réception refusée, A n'a pas les heads d'avant"
                        );
                    }
                    Ok(_) if heads_changees => {
                        assert!(
                            refusees > 0,
                            "une réception a changé les heads d'A avant celle que le disque refuse"
                        );
                        constater_l_ajout_recu(
                            a,
                            heads_d_avant,
                            &ajouts_a,
                            &ajouts_d_avant,
                            &ajouts,
                        );
                        ecrites_apres_le_refus += 1;
                    }
                    Ok(_) => {}
                    Err(autre) => panic!("attendu Disque à la réception, rendu {autre:?}"),
                }
                ajouts_d_avant = ajouts;
            },
        );
    }

    assert_eq!(
        refusees, 1,
        "la réception qui change les heads d'A n'a pas été refusée"
    );
    assert!(
        ecrites_apres_le_refus > 0,
        "après le refus, aucune réception n'a changé les heads d'A : B n'a pas renvoyé ce \
         qu'elle avait envoyé"
    );
    assert_eq!(
        a.automerge().get_heads(),
        b.automerge().get_heads(),
        "A et B ne se sont pas réunies"
    );
    drop(a);
    let a = ouvrir_le_fonds(&racine_a);
    assert!(
        porte_l_idee(a.automerge(), "idee-2"),
        "A rouverte ne porte pas l'idée de B"
    );
}

#[test]
fn un_message_illisible_rend_synchro() {
    let chemin = commun::dossier_jetable("synchronisation-illisible").join("donnees");
    let racine = ouvrir(&chemin);
    let mut a = faire_naitre(&racine);
    let copie = a.copie().expect("la copie d'A");
    let mut echange = Echange::ouvrir(&copie, &copie).expect("ouvrir l'échange");
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);

    match a.recevoir(&mut echange, b"pas un message") {
        Err(Erreur::Synchro(synchro::Erreur::MessageIllisible(_))) => {}
        autre => panic!("attendu MessageIllisible, rendu {autre:?}"),
    }
    assert_eq!(
        commun::empreintes(&dossier),
        avant,
        "un message illisible a écrit dans le fonds"
    );
}
