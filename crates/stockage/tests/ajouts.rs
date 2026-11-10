//! Les fichiers d'ajout (§ 31 · 2.1) : chaque écriture du fonds ajoute un fichier, nommé par
//! l'empreinte de son contenu, que l'ouverture relit après le fichier complet. Un fichier d'ajout
//! altéré ou manquant fait refuser l'ouverture, et une écriture refusée relit le document du
//! disque.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;

use automerge::{Automerge, AutomergeError, ReadDoc};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_des_ajouts, dossier_du_fonds, faire_naitre,
    lire_le_complet, ouvrir, ouvrir_le_fonds, porte_l_idee,
};
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Erreur, LectureSeule, RacineDesDonnees, empreinte};
use synchro::Echange;

#[test]
fn chaque_ecriture_ajoute_un_fichier_nomme_par_son_empreinte() {
    let d = commun::dossier_jetable("ajouts-un-par-ecriture");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let avant = fonds.automerge().get_heads();

    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("écrire");

    let ajouts = dossier_des_ajouts(racine.chemin());
    let noms = commun::entrees(&ajouts);
    assert_eq!(noms.len(), 1, "une écriture, un fichier d'ajout : {noms:?}");
    let ajout = fs::read(ajouts.join(&noms[0])).expect("lire le fichier d'ajout");
    assert_eq!(
        noms[0],
        format!("{}.automerge", empreinte(&ajout)),
        "le nom du fichier d'ajout n'est pas l'empreinte de son contenu"
    );
    let complet = lire_le_complet(racine.chemin());
    assert!(
        2 * ajout.len() < complet.len(),
        "le fichier d'ajout pèse {} octets, le fichier complet {}",
        ajout.len(),
        complet.len()
    );

    let modifications = fonds.automerge().get_changes(&avant);
    assert_eq!(modifications.len(), 1, "une écriture, une modification");
    assert_eq!(modifications[0].message(), Some("Étiquette posée"));
    assert_eq!(modifications[0].timestamp(), HEURE + 1);

    let mut relu = Automerge::load(&complet).expect("lire le fichier complet");
    assert_eq!(
        relu.get_heads(),
        avant,
        "le fichier complet porte déjà l'écriture"
    );
    relu.load_incremental(&ajout)
        .expect("lire le fichier d'ajout");
    assert_eq!(
        relu.get_heads(),
        fonds.automerge().get_heads(),
        "le fichier complet, puis le fichier d'ajout, ne mènent pas aux heads de l'écriture"
    );
}

#[test]
fn rouvrir_relit_le_complet_puis_les_ajouts() {
    let chemin = commun::dossier_jetable("ajouts-rouvrir").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("la première écriture");
    fonds
        .ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
        .expect("la seconde écriture");
    let heads = fonds.automerge().get_heads();
    let valeur = fonds.automerge().hydrate(None);
    // Le voisin d'une écriture qu'un processus n'a pas finie.
    let partiel =
        dossier_des_ajouts(&chemin).join(format!("{}.automerge.4242.partiel", "0".repeat(64)));
    fs::write(&partiel, b"une ecriture interrompue").expect("déposer le .partiel");
    // La racine des données fermée, puis rouverte, comme au lancement suivant.
    drop(fonds);
    drop(racine);

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(fonds.automerge().get_heads(), heads);
    assert_eq!(fonds.automerge().hydrate(None), valeur);
    assert_eq!(
        fs::read(&partiel).expect("relire le .partiel"),
        b"une ecriture interrompue",
        "le .partiel n'est plus tel qu'il a été déposé"
    );
}

#[test]
fn une_ecriture_sans_modification_n_ecrit_rien() {
    let d = commun::dossier_jetable("ajouts-sans-modification");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let avant = fonds.automerge().get_heads();

    // Ôter une étiquette que l'idée ne porte pas ne modifie rien.
    fonds
        .ecrire("Étiquette ôtée", HEURE + 1, |tx| {
            commun::oter_l_etiquette(tx, "idee-1", "refrain")
        })
        .expect("écrire");

    assert_eq!(
        commun::entrees(&dossier_des_ajouts(racine.chemin())),
        Vec::<String>::new(),
        "une écriture sans modification a écrit un fichier"
    );
    assert_eq!(fonds.automerge().get_heads(), avant);
}

#[test]
fn un_geste_refuse_n_ecrit_rien_et_le_document_reste_ouvert() {
    let d = commun::dossier_jetable("ajouts-geste-refuse");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let avant = fonds.automerge().get_heads();
    let valeur = fonds.automerge().hydrate(None);

    // Le geste écrit, puis refuse : rien de ce qu'il a écrit ne reste.
    let refus = fonds.ecrire("Idée notée", HEURE + 1, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")?;
        Err(AutomergeError::Fail)
    });

    match refus {
        Err(Erreur::Automerge(_)) => {}
        Err(autre) => panic!("attendu le refus du geste, rendu {autre:?}"),
        Ok(_) => panic!("un geste refusé s'est écrit"),
    }
    let ajouts = dossier_des_ajouts(racine.chemin());
    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "ni fichier d'ajout ni .partiel : rien ne s'écrit"
    );
    assert_eq!(fonds.automerge().get_heads(), avant);
    assert_eq!(
        fonds.automerge().hydrate(None),
        valeur,
        "ce que le geste refusé a écrit reste en mémoire"
    );

    fonds
        .ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
        .expect("l'écriture suivante");
    assert_eq!(
        commun::entrees(&ajouts).len(),
        1,
        "l'écriture suivante n'a pas écrit son fichier d'ajout"
    );
}

#[test]
fn un_ajout_manquant_rend_dependances_manquantes() {
    let d = commun::dossier_jetable("ajouts-manquant");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(racine.chemin());
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("la première écriture");
    let premier = commun::entrees(&ajouts);
    assert_eq!(
        premier.len(),
        1,
        "une écriture, un fichier d'ajout : {premier:?}"
    );
    fonds
        .ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
        .expect("la seconde écriture");
    drop(fonds);
    // La seconde modification dépend de la première, dont le fichier d'ajout manque.
    fs::remove_file(ajouts.join(&premier[0])).expect("retirer le premier fichier d'ajout");

    match racine.fonds() {
        Err(Erreur::DependancesManquantes) => {}
        Err(autre) => panic!("attendu DependancesManquantes, rendu {autre:?}"),
        Ok(_) => panic!("le fonds s'est ouvert sans le fichier d'ajout de sa première écriture"),
    }
}

#[test]
fn chaque_ouverture_tire_un_acteur_neuf() {
    let chemin = commun::dossier_jetable("ajouts-acteur-neuf").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("écrire avant de rouvrir");
    let ajouts = dossier_des_ajouts(&chemin);
    let deja = commun::entrees(&ajouts);
    drop(fonds);
    drop(racine);

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let mut fonds = ouvrir_le_fonds(&racine);
    let avant = fonds.automerge().get_heads();
    assert_eq!(
        avant.len(),
        1,
        "le fonds rouvert a une seule tête : {avant:?}"
    );
    fonds
        .ecrire("Étiquette posée", HEURE + 2, |tx| {
            commun::etiqueter(tx, "idee-1", "couplet")
        })
        .expect("écrire après avoir rouvert");

    let doc = fonds.automerge();
    let premiere = doc
        .get_change_by_hash(&avant[0])
        .expect("la modification de la première écriture");
    let seconde = doc.get_changes(&avant);
    assert_eq!(seconde.len(), 1, "une écriture, une modification");
    assert_ne!(
        premiere.actor_id(),
        seconde[0].actor_id(),
        "l'écriture d'avant et celle d'après la réouverture ont le même acteur"
    );

    let ajout = commun::constater_un_ajout(doc, &avant, &ajouts, &deja, &commun::entrees(&ajouts));
    let complet = lire_le_complet(&chemin);
    assert!(
        4 * ajout.len() < complet.len(),
        "le fichier d'ajout pèse {} octets, le fichier complet {}",
        ajout.len(),
        complet.len()
    );
}

#[test]
fn un_ajout_altere_se_nomme_et_rien_ne_bouge() {
    let d = commun::dossier_jetable("ajouts-altere");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("écrire");
    drop(fonds);
    let ajouts = dossier_des_ajouts(racine.chemin());
    let noms = commun::entrees(&ajouts);
    assert_eq!(noms.len(), 1, "une écriture, un fichier d'ajout : {noms:?}");
    // Un octet changé, au milieu du fichier d'ajout.
    let altere = ajouts.join(&noms[0]);
    let mut octets = fs::read(&altere).expect("lire le fichier d'ajout");
    let milieu = octets.len() / 2;
    octets[milieu] ^= 0xFF;
    fs::write(&altere, &octets).expect("altérer le fichier d'ajout");
    let dossier = dossier_du_fonds(racine.chemin());
    let avant = commun::empreintes(&dossier);

    match racine.fonds() {
        Err(Erreur::AjoutAltere { chemin }) => {
            assert_eq!(chemin, altere, "l'erreur ne nomme pas le fichier altéré");
        }
        Err(autre) => panic!("attendu AjoutAltere, rendu {autre:?}"),
        Ok(_) => panic!("le fonds s'est ouvert sur un fichier d'ajout altéré"),
    }
    assert_eq!(
        commun::empreintes(&dossier),
        avant,
        "l'ouverture a touché un fichier du fonds"
    );
}

// § 31 · 10, test 8 : relire du disque après une opération refusée, puis une opération
#[test]
fn une_ecriture_refusee_relit_le_document_du_disque() {
    let chemin = commun::dossier_jetable("ajouts-relu-du-disque").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let heads_avant = fonds.automerge().get_heads();
    let acteur_avant = fonds.automerge().get_actor().clone();
    let ajouts = dossier_des_ajouts(&chemin);
    let ajouts_avant = commun::entrees(&ajouts);

    let refus = {
        let _panne = couper(Etape::Renommage, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(rendu) => panic!("un renommage refusé a laissé l'écriture faite : {rendu:?}"),
    }
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "la relecture du disque a mis le document en lecture seule"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_avant,
        "relu du disque, le document n'a pas les heads d'avant l'écriture refusée"
    );
    // La mémoire relue a un autre acteur qu'avant : `Automerge::load` en tire un, et
    // `documents.rs` en tire un explicitement aussi.
    assert_ne!(
        fonds.automerge().get_actor(),
        &acteur_avant,
        "la relecture du disque n'a pas tiré un acteur neuf"
    );
    assert_eq!(
        commun::entrees(&ajouts),
        ajouts_avant,
        "l'écriture refusée a laissé un fichier dans ajouts/"
    );
    match racine.fonds() {
        Err(Erreur::FondsDejaOuvert) => {}
        Err(autre) => panic!("attendu FondsDejaOuvert, rendu {autre:?}"),
        Ok(_) => panic!("la relecture du disque a rendu la garde du fonds"),
    }

    match commun::ecrire(&mut fonds, |tx| {
        commun::noter(tx, "idee-3", "Une coda en suspens")
    }) {
        Ok(None) => {}
        autre => panic!("attendu l'écriture faite, rendu {autre:?}"),
    }
    commun::constater_un_ajout(
        fonds.automerge(),
        &heads_avant,
        &ajouts,
        &ajouts_avant,
        &commun::entrees(&ajouts),
    );
    assert_eq!(
        fonds.automerge().get_changes(&heads_avant).len(),
        1,
        "la mémoire porte encore l'écriture refusée"
    );
}

// § 31 · 10, test 8 : interrompre à chaque étape, puis rouvrir
#[test]
fn une_ecriture_coupee_a_chaque_etape() {
    // Avant la fin du renommage : refusée, sans voisin qui reste, et le fonds rouvert a les heads
    // d'avant.
    for (i, etape) in [
        Etape::VoisinEcrit,
        Etape::SyncAll,
        Etape::Relecture,
        Etape::Renommage,
    ]
    .into_iter()
    .enumerate()
    {
        let chemin = commun::dossier_jetable(&format!("ajouts-coupee-{i}")).join("donnees");
        let racine = ouvrir(&chemin);
        let mut fonds = faire_naitre(&racine);
        let avant = fonds.automerge().get_heads();

        let rendu = {
            let _panne = couper(etape, 1);
            commun::ecrire(&mut fonds, |tx| {
                commun::noter(tx, "idee-2", "Un pont en mineur")
            })
        };

        assert!(
            matches!(rendu, Err(Erreur::Disque { .. })),
            "{etape:?} : rendu {rendu:?}"
        );
        let restes = commun::entrees(&dossier_des_ajouts(&chemin));
        assert!(
            restes.iter().all(|nom| !nom.ends_with(".partiel")),
            "{etape:?} : un .partiel reste dans ajouts/ : {restes:?}"
        );
        drop(fonds);
        let fonds = ouvrir_le_fonds(&racine);
        assert_eq!(
            fonds.automerge().get_heads(),
            avant,
            "{etape:?} : le fonds rouvert n'a pas les heads d'avant l'écriture refusée"
        );
    }

    // Après le renommage : faite, avec un avertissement ; le fichier d'ajout est en place, et le
    // fonds rouvert porte l'idée.
    let chemin = commun::dossier_jetable("ajouts-coupee-rendre-durable").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match rendu {
        Ok(Some(Avertissement::NonDurable { chemin: ajout, .. })) => {
            assert_eq!(
                ajout.parent(),
                Some(ajouts.as_path()),
                "RendreDurable : l'avertissement ne nomme pas un fichier d'ajout : {}",
                ajout.display()
            );
            assert!(
                ajout.is_file(),
                "RendreDurable : le fichier d'ajout n'est pas en place"
            );
        }
        autre => panic!("RendreDurable : attendu NonDurable, rendu {autre:?}"),
    }
    drop(fonds);
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "RendreDurable : le fonds rouvert ne porte pas l'idée écrite"
    );
}

#[test]
fn une_relecture_qui_echoue_met_le_document_en_lecture_seule() {
    let chemin = commun::dossier_jetable("ajouts-relecture-manquee").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let heads_avant = fonds.automerge().get_heads();

    // La première relecture est celle du voisin, la seconde celle du fichier complet, quand le
    // document se relit du disque.
    let refus = {
        let _renommage = couper(Etape::Renommage, 1);
        let _relecture = couper(Etape::Relecture, 2);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(rendu) => panic!("un renommage refusé a laissé l'écriture faite : {rendu:?}"),
    }
    let complet = dossier_du_fonds(&chemin).join("complet.automerge");
    assert_eq!(
        fonds.lecture_seule(),
        Some(&LectureSeule::RelectureManquee { chemin: complet }),
        "la relecture manquée ne met pas le document en lecture seule sur le fichier complet"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "la mémoire montre l'écriture refusée"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_avant,
        "la mémoire n'a pas les heads d'avant l'écriture refusée"
    );

    match commun::ecrire(&mut fonds, |tx| {
        commun::noter(tx, "idee-3", "Une coda en suspens")
    }) {
        Err(Erreur::LectureSeule(LectureSeule::RelectureManquee { .. })) => {}
        autre => panic!("attendu LectureSeule à l'écriture, rendu {autre:?}"),
    }
    match fonds.compacter() {
        Err(Erreur::LectureSeule(LectureSeule::RelectureManquee { .. })) => {}
        autre => panic!("attendu LectureSeule à la compaction, rendu {autre:?}"),
    }
    let copie = fonds.copie().expect("la copie du fonds");
    let mut echange = Echange::ouvrir(&copie, &copie).expect("ouvrir l'échange");
    assert_eq!(
        fonds.prochain_message(&mut echange),
        None,
        "un document en lecture seule a envoyé un message"
    );
    let mut de_l_autre =
        Echange::ouvrir(&copie, &copie).expect("ouvrir l'échange de l'autre copie");
    let message = de_l_autre
        .prochain_message(fonds.automerge())
        .expect("le message de l'autre copie");
    match fonds.recevoir(&mut echange, &message) {
        Err(Erreur::LectureSeule(_)) => {}
        autre => panic!("attendu LectureSeule à la réception, rendu {autre:?}"),
    }
    racine
        .captations()
        .verser(&commun::octets(3, 100))
        .expect("une captation se verse toujours");
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "rouvert sans panne, le fonds est en lecture seule"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert porte l'écriture refusée"
    );
}

// § 31 · 10, test 8 : une opération dite faite ne manque pas
#[test]
fn un_rendre_durable_refuse_apres_le_renommage_laisse_l_operation_faite() {
    let chemin = commun::dossier_jetable("ajouts-non-durable").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match rendu {
        Ok(Some(Avertissement::NonDurable { chemin: ajout, .. })) => assert_eq!(
            ajout.parent(),
            Some(ajouts.as_path()),
            "l'avertissement ne nomme pas un fichier d'ajout : {}",
            ajout.display()
        ),
        autre => panic!("attendu NonDurable, rendu {autre:?}"),
    }
    drop(fonds);
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert ne porte pas l'opération dite faite"
    );
    drop(fonds);

    // La naissance : le fichier complet en place, son renommage non rendu durable.
    let neuve = ouvrir(&commun::dossier_jetable("ajouts-naissance-non-durable").join("donnees"));
    let nee = {
        let _panne = couper(Etape::RendreDurable, 1);
        neuve.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
    };

    let document = nee.expect("la naissance est faite, avec un avertissement");
    let complet = dossier_du_fonds(neuve.chemin()).join("complet.automerge");
    match document.avertissements() {
        [Avertissement::NonDurable { chemin, .. }] => assert_eq!(
            chemin, &complet,
            "l'avertissement de la naissance ne nomme pas le fichier complet"
        ),
        autres => panic!("attendu un seul NonDurable, rendu {autres:?}"),
    }
    drop(document);
    assert!(
        neuve.fonds().expect("rouvrir le fonds").is_some(),
        "le fonds né sans renommage durable ne se rouvre pas"
    );
}
