//! Le fonds naît, et se rouvre (§ 31 · 2.3) : la genèse, la naissance et le premier geste
//! s'écrivent ensemble, en un seul fichier complet ; un geste refusé ne laisse rien, et le fonds
//! ne naît qu'une fois. Rouvert sans sa lignée, il ne situe pas sa copie (§ 31 · 2.5).

mod commun;

use std::fs;

use automerge::{
    ActorId, Automerge, AutomergeError, Change, ObjId, ROOT, ReadDoc, ScalarValue, Value,
};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_des_ajouts, dossier_du_fonds, faire_naitre, objet,
    ouvrir, ouvrir_le_fonds,
};
use noyau::donnees::fonds::{
    ETIQUETTES, GENESE, IDEES, IDENTIFIANT, LIGNEE, MESSAGE_DE_LA_GENESE, MESSAGE_DE_LA_NAISSANCE,
    PROPRIETAIRE, TEXTE,
};
use stockage::pannes::{Etape, couper};
use stockage::{Erreur, RacineDesDonnees};

/// La valeur simple que porte la clé `cle` de `objet`.
fn valeur(doc: &Automerge, objet: &ObjId, cle: &str) -> ScalarValue {
    match doc.get(objet, cle).expect("lire le document") {
        Some((Value::Scalar(valeur), _)) => valeur.into_owned(),
        autre => panic!("{cle} ne porte pas une valeur simple : {autre:?}"),
    }
}

#[test]
fn une_racine_des_donnees_neuve_n_a_pas_de_fonds() {
    let d = commun::dossier_jetable("fonds-absent");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");

    let fonds = racine.fonds().expect("chercher le fonds");

    assert!(fonds.is_none(), "une racine des données neuve a un fonds");
    assert_eq!(
        commun::entrees(&racine.chemin().join("documents")),
        Vec::<String>::new(),
        "documents/ n'est pas vide"
    );
}

#[test]
fn naitre_ecrit_un_seul_fichier_complet() {
    let chemin = commun::dossier_jetable("fonds-ne").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    racine
        .faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
        .expect("faire naître le fonds");
    // La racine des données fermée, puis rouverte, comme au lancement suivant.
    drop(racine);

    let dossier = dossier_du_fonds(&chemin);
    assert_eq!(
        commun::entrees(&chemin.join("documents")),
        ["fonds"],
        "documents/ ne porte que le fonds"
    );
    assert_eq!(
        commun::entrees(&dossier),
        ["ajouts", "complet.automerge"],
        "le fonds né n'a que son fichier complet et ajouts/"
    );
    assert_eq!(
        commun::entrees(&dossier.join("ajouts")),
        Vec::<String>::new(),
        "ajouts/ n'est pas vide"
    );

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let fonds = ouvrir_le_fonds(&racine);
    let doc = fonds.automerge();

    let modifications = doc.get_changes(&[]);
    let messages: Vec<Option<&str>> = modifications.iter().map(Change::message).collect();
    assert_eq!(
        messages,
        [
            Some(MESSAGE_DE_LA_GENESE),
            Some(MESSAGE_DE_LA_NAISSANCE),
            Some(MESSAGE_DU_GESTE)
        ]
    );
    let heures: Vec<i64> = modifications.iter().map(Change::timestamp).collect();
    assert_eq!(heures, [0, HEURE, HEURE], "la genèse porte l'heure 0");
    let genese = Change::try_from(GENESE).expect("lire la genèse");
    assert_eq!(
        modifications[0].hash(),
        genese.hash(),
        "la première modification n'est pas la genèse"
    );

    assert_eq!(
        valeur(doc, &ROOT, IDENTIFIANT),
        ScalarValue::from(NAISSANCE.identifiant)
    );
    assert_eq!(
        valeur(doc, &ROOT, LIGNEE),
        ScalarValue::from(NAISSANCE.lignee)
    );
    assert_eq!(
        valeur(doc, &ROOT, PROPRIETAIRE),
        ScalarValue::from(NAISSANCE.proprietaire)
    );
    let idee = objet(doc, &objet(doc, &ROOT, IDEES), "idee-1");
    assert_eq!(
        valeur(doc, &idee, TEXTE),
        ScalarValue::from("Un refrain qui monte")
    );
    let etiquettes = objet(doc, &idee, ETIQUETTES);
    assert_eq!(
        doc.keys(&etiquettes).count(),
        0,
        "l'idée notée naît avec des étiquettes"
    );
}

#[test]
fn un_premier_geste_refuse_ne_fait_pas_naitre_le_fonds() {
    let d = commun::dossier_jetable("fonds-refuse");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");

    // Le geste écrit, puis refuse : rien de ce qu'il a écrit ne reste.
    let refus = racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
        commun::noter(tx, "idee-1", "Un refrain qui monte")?;
        Err(AutomergeError::Fail)
    });

    match refus {
        Err(Erreur::Automerge(AutomergeError::Fail)) => {}
        Err(autre) => panic!("attendu le refus du geste, rendu {autre:?}"),
        Ok(_) => panic!("un premier geste refusé a fait naître le fonds"),
    }
    assert!(
        racine.fonds().expect("chercher le fonds").is_none(),
        "un premier geste refusé a fait naître le fonds"
    );
    assert_eq!(
        commun::entrees(&racine.chemin().join("documents")),
        Vec::<String>::new(),
        "ni complet.automerge ni .partiel : rien ne s'écrit"
    );
}

#[test]
fn le_fonds_ne_nait_qu_une_fois() {
    let d = commun::dossier_jetable("fonds-une-fois");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    racine
        .faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
        .expect("la première naissance");
    let dossier = dossier_du_fonds(racine.chemin());
    let avant = fs::read(dossier.join("complet.automerge")).expect("lire le fichier complet");

    let seconde = racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
        commun::noter(tx, "idee-2", "Un couplet plus calme")
    });

    match seconde {
        Err(Erreur::FondsDejaNe) => {}
        Err(autre) => panic!("attendu FondsDejaNe, rendu {autre:?}"),
        Ok(_) => panic!("le fonds est né une seconde fois"),
    }
    let apres = fs::read(dossier.join("complet.automerge")).expect("relire le fichier complet");
    assert!(apres == avant, "le fichier complet du fonds né a changé");
    assert_eq!(
        commun::entrees(&dossier),
        ["ajouts", "complet.automerge"],
        "la seconde naissance a laissé un fichier"
    );
}

#[test]
fn un_second_fonds_est_refuse() {
    let chemin = commun::dossier_jetable("fonds-second").join("donnees");
    let racine = ouvrir(&chemin);
    let mut a = faire_naitre(&racine);

    match racine.fonds() {
        Err(Erreur::FondsDejaOuvert) => {}
        Err(autre) => panic!("attendu FondsDejaOuvert, rendu {autre:?}"),
        Ok(_) => panic!("un second Document du fonds s'est ouvert"),
    }
    let seconde = racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
        commun::noter(tx, "idee-2", "Un couplet plus calme")
    });
    match seconde {
        Err(Erreur::FondsDejaOuvert) => {}
        Err(autre) => panic!("attendu FondsDejaOuvert, rendu {autre:?}"),
        Ok(_) => panic!("un second Document du fonds est né"),
    }

    // Les deux refus laissent la garde au premier : il écrit encore, et un troisième est encore
    // refusé tant qu'il vit.
    let rendu = commun::ecrire(&mut a, |tx| {
        commun::noter(tx, "idee-3", "Une coda en suspens")
    });
    assert!(
        matches!(rendu, Ok(None)),
        "le premier Document n'écrit plus après les refus : rendu {rendu:?}"
    );
    match racine.fonds() {
        Err(Erreur::FondsDejaOuvert) => {}
        Err(autre) => panic!("attendu FondsDejaOuvert au troisième, rendu {autre:?}"),
        Ok(_) => panic!("un troisième Document du fonds s'est ouvert, le premier vivant"),
    }

    drop(a);
    let rouvert = ouvrir_le_fonds(&racine);
    assert!(
        commun::porte_l_idee(rouvert.automerge(), "idee-3"),
        "le fonds rouvert, le premier Document lâché, ne porte pas ce qu'il a écrit"
    );
}

#[test]
fn une_ouverture_qui_echoue_rend_la_garde() {
    let chemin = commun::dossier_jetable("fonds-garde-rendue").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let ajouts = dossier_des_ajouts(&chemin);
    let de_cote = ajouts.with_file_name("ajouts-de-cote");
    fs::rename(&ajouts, &de_cote).expect("mettre ajouts/ de côté");
    fs::write(&ajouts, b"un fichier a la place de ajouts/").expect("poser un fichier");

    match racine.fonds() {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(_) => panic!("le fonds s'est ouvert, un fichier à la place de son dossier ajouts/"),
    }

    fs::remove_file(&ajouts).expect("ôter le fichier posé à la place de ajouts/");
    fs::rename(&de_cote, &ajouts).expect("rendre ajouts/");
    assert!(
        racine.fonds().expect("rouvrir le fonds").is_some(),
        "l'ouverture en échec n'a pas rendu la garde"
    );

    // Une racine des données neuve : `Ok(None)` rend la garde aussi.
    let neuve = ouvrir(&commun::dossier_jetable("fonds-garde-neuve").join("donnees"));
    assert!(
        neuve.fonds().expect("chercher le fonds").is_none(),
        "une racine des données neuve a un fonds"
    );
    drop(faire_naitre(&neuve));
}

#[test]
fn une_naissance_refusee_avant_le_renommage_laisse_pupitre_neuf() {
    let racine = ouvrir(&commun::dossier_jetable("fonds-naissance-refusee").join("donnees"));

    // Le premier renommage de la naissance est celui du fichier complet.
    let refus = {
        let _panne = couper(Etape::Renommage, 1);
        racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
    };

    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(_) => panic!("le fonds est né sans que son fichier complet soit renommé"),
    }
    let dossier = dossier_du_fonds(racine.chemin());
    assert_eq!(
        commun::entrees(&dossier),
        ["ajouts"],
        "documents/fonds/ ne porte pas que ajouts/"
    );
    assert_eq!(
        commun::entrees(&dossier.join("ajouts")),
        Vec::<String>::new(),
        "ajouts/ n'est pas vide"
    );
    match racine.fonds() {
        Ok(None) => {}
        Ok(Some(_)) => panic!("une naissance refusée a fait naître le fonds"),
        Err(autre) => panic!("attendu aucun fonds, la garde rendue ; rendu {autre:?}"),
    }

    racine
        .faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
        .expect("la seconde naissance, sans panne");
}

#[test]
fn une_copie_sans_lignee_rend_champ_absent() {
    let chemin = commun::dossier_jetable("fonds-sans-lignee").join("donnees");
    let racine = ouvrir(&chemin);
    // Un fonds que le test bâtit : la genèse, puis l'identifiant et le propriétaire, sans lignée.
    let bati = commun::batir_un_fonds(
        ActorId::random(),
        &[
            (IDENTIFIANT, NAISSANCE.identifiant),
            (PROPRIETAIRE, NAISSANCE.proprietaire),
        ],
    );
    let dossier = dossier_du_fonds(&chemin);
    fs::create_dir_all(dossier.join("ajouts")).expect("créer ajouts/");
    fs::write(dossier.join("complet.automerge"), bati.save()).expect("écrire le fichier complet");

    let fonds = ouvrir_le_fonds(&racine);

    match fonds.copie() {
        Err(Erreur::ChampAbsent { champ: "lignee" }) => {}
        autre => panic!("attendu ChampAbsent sur la lignée, rendu {autre:?}"),
    }
}
