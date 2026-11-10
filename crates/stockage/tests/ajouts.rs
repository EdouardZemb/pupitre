//! Les fichiers d'ajout (§ 31 · 2.1) : chaque écriture du fonds ajoute un fichier, nommé par
//! l'empreinte de son contenu, que l'ouverture relit après le fichier complet. Un fichier d'ajout
//! altéré ou manquant fait refuser l'ouverture, et une écriture qui échoue laisse le document à
//! relire.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;

use automerge::{Automerge, AutomergeError, ReadDoc};
use commun::{
    HEURE, dossier_des_ajouts, dossier_du_fonds, faire_naitre, lire_le_complet, ouvrir_le_fonds,
};
use stockage::{Erreur, RacineDesDonnees, empreinte};

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
        Ok(()) => panic!("un geste refusé s'est écrit"),
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

    let nouveaux: Vec<String> = commun::entrees(&ajouts)
        .into_iter()
        .filter(|nom| !deja.contains(nom))
        .collect();
    assert_eq!(
        nouveaux.len(),
        1,
        "une écriture, un fichier d'ajout : {nouveaux:?}"
    );
    let ajout = fs::read(ajouts.join(&nouveaux[0])).expect("lire le fichier d'ajout");
    assert!(
        ajout == doc.save_after(&avant),
        "le fichier d'ajout n'est pas save_after des heads d'avant l'écriture"
    );
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

#[test]
fn une_ecriture_qui_echoue_laisse_le_document_a_relire() {
    let d = commun::dossier_jetable("ajouts-a-relire");
    let chemin = d.join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("la première écriture");
    let avant = fonds.automerge().get_heads();
    // `ajouts/` mis de côté, et un fichier à sa place : aucun fichier d'ajout ne s'y écrit.
    let ajouts = dossier_des_ajouts(&chemin);
    let de_cote = d.join("ajouts-mis-de-cote");
    fs::rename(&ajouts, &de_cote).expect("mettre ajouts/ de côté");
    fs::write(&ajouts, b"").expect("poser un fichier à la place de ajouts/");

    let refus = fonds.ecrire("Idée notée", HEURE + 2, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    });
    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(()) => panic!("le fichier d'ajout s'est écrit sans ajouts/"),
    }
    let suivante = fonds.ecrire("Étiquette posée", HEURE + 3, |tx| {
        commun::etiqueter(tx, "idee-1", "couplet")
    });
    match suivante {
        Err(Erreur::ARelire) => {}
        Err(autre) => panic!("attendu ARelire, rendu {autre:?}"),
        Ok(()) => panic!("un document à relire a écrit"),
    }
    drop(fonds);

    fs::remove_file(&ajouts).expect("ôter le fichier posé à la place de ajouts/");
    fs::rename(&de_cote, &ajouts).expect("rendre ajouts/");
    let fonds = ouvrir_le_fonds(&racine);
    assert_eq!(
        fonds.automerge().get_heads(),
        avant,
        "le fonds rouvert n'a pas les heads d'avant l'écriture refusée"
    );
}
