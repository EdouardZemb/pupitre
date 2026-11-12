//! Le démarrage dans `documents/` (§ 31 · 2.1) : à chaque ouverture du fonds, la garde prise,
//! chaque `.partiel` reçoit son sort. Le voisin d'un fichier d'ajout dont le contenu donne
//! l'empreinte prend sa place, ou se lit où il est, avec un avertissement ; le voisin lisible du
//! fichier complet se lit, puis la compaction d'ouverture le fond ; tout autre se retire. Un
//! voisin lisible du fichier complet fait exister le fonds, que la naissance ne refait pas. Un
//! fichier étranger reste tel quel, et rien ne se touche sans le verrou.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;
use std::path::{Path, PathBuf};

use automerge::Automerge;
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_des_ajouts, dossier_du_fonds, empreintes, entrees,
    faire_naitre, faite, lire_le_complet, ouvrir, ouvrir_le_fonds, porte_l_idee,
    relire_le_dossier_du_fonds,
};
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Document, Erreur, RacineDesDonnees, empreinte};

/// Les idées que notent les trois écritures des tests du voisin du fichier complet, avec leur
/// texte.
const TROIS_IDEES: [(&str, &str); 3] = [
    ("idee-2", "Un pont en mineur"),
    ("idee-3", "Une coda en suspens"),
    ("idee-4", "Un couplet plus calme"),
];

/// Note l'idée `idee`, au texte `texte`, dans `fonds` : l'écriture est faite, sans avertissement.
fn noter(fonds: &mut Document<'_>, idee: &str, texte: &str) {
    faite(
        commun::ecrire(fonds, |tx| commun::noter(tx, idee, texte)),
        &format!("noter {idee}"),
    );
}

/// Les noms en `.partiel` de `dossier`, dans l'ordre.
fn partiels(dossier: &Path) -> Vec<String> {
    entrees(dossier)
        .into_iter()
        .filter(|nom| nom.ends_with(".partiel"))
        .collect()
}

/// Le seul fichier d'ajout du fonds de la racine des données `chemin`, renommé par le test en
/// son voisin `<nom>.4242.partiel`, comme une écriture dont le renommage s'est perdu après la
/// réponse. Rend le chemin du voisin.
fn l_ajout_en_voisin(chemin: &Path) -> PathBuf {
    let ajouts = dossier_des_ajouts(chemin);
    let noms = entrees(&ajouts);
    assert_eq!(noms.len(), 1, "un seul fichier d'ajout : {noms:?}");
    let voisin = ajouts.join(format!("{}.4242.partiel", noms[0]));
    fs::rename(ajouts.join(&noms[0]), &voisin).expect("renommer le fichier d'ajout en son voisin");
    voisin
}

/// Le fonds de `racine` né, puis les trois idées de `TROIS_IDEES` notées, et lâché ; puis, comme
/// une compaction dont le renommage du fichier complet s'est perdu, les fichiers d'ajout
/// absorbés déjà retirés : le dossier du fonds porte, sous le nom `nom_du_voisin`, `save()` de
/// tout ce qu'il lit, le fichier complet de la naissance à sa place, et `ajouts/` vide. Rend le
/// chemin du voisin.
fn un_voisin_du_complet_laisse(racine: &RacineDesDonnees, nom_du_voisin: &str) -> PathBuf {
    let chemin = racine.chemin();
    let mut fonds = faire_naitre(racine);
    let c0 = lire_le_complet(chemin);
    for (idee, texte) in TROIS_IDEES {
        noter(&mut fonds, idee, texte);
    }
    drop(fonds);
    let s = relire_le_dossier_du_fonds(chemin).save();
    let dossier = dossier_du_fonds(chemin);
    let voisin = dossier.join(nom_du_voisin);
    fs::write(&voisin, s).expect("écrire le voisin du fichier complet");
    fs::write(dossier.join("complet.automerge"), c0)
        .expect("remettre le fichier complet de la naissance");
    let ajouts = dossier.join("ajouts");
    for nom in entrees(&ajouts) {
        fs::remove_file(ajouts.join(nom)).expect("vider ajouts/");
    }
    voisin
}

/// `doc` porte chacune des trois idées de `TROIS_IDEES` ; `quoi` dit ce qu'il est.
#[track_caller]
fn porte_les_trois_idees(doc: &Automerge, quoi: &str) {
    for (idee, _) in TROIS_IDEES {
        assert!(porte_l_idee(doc, idee), "{quoi} ne porte pas {idee}");
    }
}

// § 31 · 10, test 8 : renommage perdu après la réponse, le voisin entier reste
#[test]
fn le_voisin_conforme_d_un_ajout_prend_sa_place() {
    let chemin = commun::dossier_jetable("demarrage-voisin-d-ajout").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    drop(fonds);
    let voisin = l_ajout_en_voisin(&chemin);

    let fonds = ouvrir_le_fonds(&racine);

    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert ne porte pas l'écriture dont le voisin est resté"
    );
    assert!(!voisin.exists(), "le voisin n'a pas pris sa place");
    assert_eq!(
        partiels(&dossier_des_ajouts(&chemin)),
        Vec::<String>::new(),
        "ajouts/ garde un .partiel"
    );
}

#[test]
fn un_voisin_d_ajout_qui_ne_se_met_pas_en_place_se_lit_et_avertit() {
    let chemin = commun::dossier_jetable("demarrage-voisin-d-ajout-restant").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    drop(fonds);
    let voisin = l_ajout_en_voisin(&chemin);
    let octets = fs::read(&voisin).expect("lire le voisin");

    // Le premier renommage de l'ouverture est celui du démarrage.
    let ouvert = {
        let _panne = couper(Etape::Renommage, 1);
        racine.fonds()
    };

    let fonds = ouvert
        .expect("un voisin qui ne prend pas sa place n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds ouvert ne porte pas l'écriture du voisin resté"
    );
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le voisin resté met le fonds en lecture seule"
    );
    match fonds.avertissements() {
        [Avertissement::VoisinRestant { chemin: reste, .. }] => {
            assert_eq!(
                reste, &voisin,
                "l'avertissement ne nomme pas le voisin resté"
            )
        }
        autres => panic!("attendu un seul VoisinRestant, rendu {autres:?}"),
    }
    assert!(
        fs::read(&voisin).expect("relire le voisin resté") == octets,
        "le voisin resté n'est plus là, octet pour octet"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "rouvert sans panne, le fonds ne porte pas l'écriture du voisin"
    );
    assert_eq!(
        partiels(&dossier_des_ajouts(&chemin)),
        Vec::<String>::new(),
        "rouvert sans panne, ajouts/ garde un .partiel"
    );
}

// § 31 · 10, test 8 : un voisin incomplet n'est pas lu comme une écriture
#[test]
fn un_voisin_incomplet_ou_non_conforme_est_retire() {
    let chemin = commun::dossier_jetable("demarrage-voisins-retires").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    let heads = fonds.automerge().get_heads();
    let ajouts = dossier_des_ajouts(&chemin);

    // Le voisin vidé d'une opération refusée : son retrait coupé, il reste, vide.
    let refus = {
        let _renommage = couper(Etape::Renommage, 1);
        let _retrait = couper(Etape::Retrait, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-3", "Une coda en suspens")
        })
    };
    assert!(
        matches!(refus, Err(Erreur::Disque { .. })),
        "attendu l'opération refusée ; rendu {refus:?}"
    );
    let restes = partiels(&ajouts);
    assert_eq!(
        restes.len(),
        1,
        "l'opération refusée n'a pas laissé son voisin : {restes:?}"
    );
    let vide = ajouts.join(&restes[0]);
    assert!(
        fs::read(&vide).expect("lire le voisin vidé").is_empty(),
        "le voisin de l'opération refusée n'est pas vidé"
    );

    // Une écriture dont le test ôte le fichier d'ajout : son contenu fait les voisins tronqué et
    // vide, comme une écriture coupée avant son renommage.
    let avant = entrees(&ajouts);
    noter(&mut fonds, "idee-4", "Un couplet plus calme");
    let noms = commun::nouveaux(&avant, &entrees(&ajouts));
    assert_eq!(noms.len(), 1, "un seul fichier d'ajout de plus : {noms:?}");
    let ajout = ajouts.join(&noms[0]);
    let contenu = fs::read(&ajout).expect("lire le fichier d'ajout");
    fs::remove_file(&ajout).expect("ôter le fichier d'ajout");
    drop(fonds);

    let documents = chemin.join("documents");
    let deposes = [
        (
            ajouts.join(format!("{}.4242.partiel", noms[0])),
            contenu[..contenu.len() / 2].to_vec(),
        ),
        (ajouts.join(format!("{}.4243.partiel", noms[0])), Vec::new()),
        (
            dossier_du_fonds(&chemin).join("complet.automerge.4242.partiel"),
            b"pas un document".to_vec(),
        ),
        (
            documents.join("x.4242.partiel"),
            b"hors du dossier d'un document".to_vec(),
        ),
    ];
    for (depose, octets) in &deposes {
        fs::write(depose, octets).unwrap_or_else(|e| panic!("déposer {} : {e}", depose.display()));
    }

    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(
        fonds.automerge().get_heads(),
        heads,
        "le fonds rouvert n'a pas les heads d'avant"
    );
    assert!(
        !vide.exists(),
        "le voisin vidé de l'opération refusée n'est pas retiré"
    );
    for (depose, _) in &deposes {
        assert!(!depose.exists(), "{} n'est pas retiré", depose.display());
    }
    assert_eq!(
        entrees(&documents),
        ["fonds"],
        "documents/ garde un .partiel"
    );
    assert_eq!(
        entrees(&dossier_du_fonds(&chemin)),
        ["ajouts", "complet.automerge"],
        "le dossier du fonds garde un .partiel"
    );
    assert_eq!(
        entrees(&ajouts),
        Vec::<String>::new(),
        "la compaction d'ouverture a laissé un fichier dans ajouts/"
    );
}

// § 31 · 10, test 9 : renommage du fichier complet perdu, les ajouts absorbés déjà retirés, puis rouvrir
#[test]
fn le_voisin_du_complet_se_lit_puis_se_fond() {
    let chemin = commun::dossier_jetable("demarrage-voisin-du-complet").join("donnees");
    let racine = ouvrir(&chemin);
    let voisin = un_voisin_du_complet_laisse(&racine, "complet.automerge.4242.partiel");

    let fonds = ouvrir_le_fonds(&racine);

    porte_les_trois_idees(fonds.automerge(), "le fonds rouvert");
    assert!(
        !voisin.exists(),
        "le voisin du fichier complet n'est pas retiré"
    );
    let seul = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    porte_les_trois_idees(&seul, "le fichier complet relu seul");
    assert_eq!(
        seul.get_heads(),
        fonds.automerge().get_heads(),
        "le fichier complet relu seul ne mène pas aux heads du fonds rouvert"
    );
}

// § 31 · 10, test 9 : de même quand le voisin laissé porte le numéro du processus qui rouvre
#[test]
fn un_voisin_au_numero_du_processus_n_est_pas_ecrase() {
    let chemin = commun::dossier_jetable("demarrage-voisin-du-processus").join("donnees");
    let racine = ouvrir(&chemin);
    let nom_du_voisin = format!("complet.automerge.{}.partiel", std::process::id());
    let voisin = un_voisin_du_complet_laisse(&racine, &nom_du_voisin);
    let du_voisin = empreinte(&fs::read(&voisin).expect("lire le voisin laissé"));

    // Le voisin laissé se lit, sans renommage : le premier renommage de l'ouverture est celui de
    // sa compaction.
    let ouvert = {
        let _panne = couper(Etape::Renommage, 1);
        racine.fonds()
    };

    let fonds = ouvert
        .expect("une compaction d'ouverture qui échoue n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds ouvert est en lecture seule"
    );
    assert!(
        matches!(
            fonds.avertissements(),
            [Avertissement::Compaction(Erreur::Disque { .. })]
        ),
        "attendu la compaction refusée par le disque en avertissement ; rendu {:?}",
        fonds.avertissements()
    );
    porte_les_trois_idees(fonds.automerge(), "le fonds ouvert");
    assert_eq!(
        empreinte(&fs::read(&voisin).expect("relire le voisin laissé")),
        du_voisin,
        "le voisin laissé n'a plus son empreinte"
    );
    assert_eq!(
        entrees(&dossier_du_fonds(&chemin)),
        ["ajouts", "complet.automerge", nom_du_voisin.as_str()],
        "le voisin de la compaction refusée reste à côté du fichier complet"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    porte_les_trois_idees(fonds.automerge(), "rouvert sans panne, le fonds");
}

// § 31 · 10, test 8 : à la naissance, renommage du premier fichier complet perdu, puis relancer
#[test]
fn a_la_naissance_le_voisin_du_premier_complet_fait_exister_le_fonds() {
    let chemin = commun::dossier_jetable("demarrage-voisin-de-la-naissance").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let dossier = dossier_du_fonds(&chemin);
    let complet = dossier.join("complet.automerge");
    let voisin = dossier.join("complet.automerge.4242.partiel");
    fs::rename(&complet, &voisin).expect("renommer le fichier complet en son voisin");
    let du_voisin = empreinte(&fs::read(&voisin).expect("lire le voisin"));

    let seconde = racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
        commun::noter(tx, "idee-2", "Un couplet plus calme")
    });

    match seconde {
        Err(Erreur::FondsDejaNe) => {}
        Err(autre) => panic!("attendu FondsDejaNe, rendu {autre:?}"),
        Ok(_) => {
            panic!("le fonds est né une seconde fois, à côté du voisin de son fichier complet")
        }
    }
    assert_eq!(
        empreinte(&fs::read(&voisin).expect("relire le voisin")),
        du_voisin,
        "le voisin n'a plus son empreinte"
    );
    assert!(
        !complet.exists(),
        "la seconde naissance a écrit un fichier complet"
    );

    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-1"),
        "le fonds ouvert ne porte pas l'idée de sa naissance"
    );
    assert!(complet.exists(), "le fichier complet n'est pas en place");
    assert!(!voisin.exists(), "le voisin n'est pas retiré");
}

#[test]
fn un_voisin_du_complet_vide_laisse_pupitre_neuf() {
    let chemin = commun::dossier_jetable("demarrage-voisin-du-complet-vide").join("donnees");
    let racine = ouvrir(&chemin);
    let dossier = dossier_du_fonds(&chemin);
    fs::create_dir_all(&dossier).expect("créer le dossier du fonds");
    let voisin = dossier.join("complet.automerge.4242.partiel");
    fs::write(&voisin, b"").expect("déposer le voisin vide");

    match racine.fonds() {
        Ok(None) => {}
        Ok(Some(_)) => panic!("un voisin vide du fichier complet a fait exister le fonds"),
        Err(autre) => panic!("attendu aucun fonds ; rendu {autre:?}"),
    }
    assert!(!voisin.exists(), "le voisin vide n'est pas retiré");

    drop(faire_naitre(&racine));
}

// § 31 · 2.1 : un nom déjà pris reçoit un suffixe ; deux voisins laissés sous le même numéro de
// processus
#[test]
fn deux_voisins_du_complet_se_lisent_puis_se_fondent() {
    let chemin = commun::dossier_jetable("demarrage-deux-voisins-du-complet").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let naissance = lire_le_complet(&chemin);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    drop(fonds);
    let avec_l_idee = relire_le_dossier_du_fonds(&chemin).save();
    let dossier = dossier_du_fonds(&chemin);
    // Ni fichier complet, ni `ajouts/`, comme une copie qui ne garde pas un dossier vide :
    // l'ouverture lit un `ajouts/` absent comme vide, et ne le crée pas.
    fs::remove_file(dossier.join("complet.automerge")).expect("ôter le fichier complet");
    fs::remove_dir_all(dossier.join("ajouts")).expect("ôter ajouts/");
    let premier = dossier.join("complet.automerge.4242.partiel");
    let second = dossier.join("complet.automerge.4242-2.partiel");
    fs::write(&premier, naissance).expect("écrire le premier voisin");
    fs::write(&second, avec_l_idee).expect("écrire le second voisin");

    let fonds = ouvrir_le_fonds(&racine);

    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert ne porte pas l'idée du second voisin"
    );
    assert!(!premier.exists(), "le premier voisin n'est pas retiré");
    assert!(!second.exists(), "le second voisin n'est pas retiré");
    assert_eq!(
        entrees(&dossier),
        ["complet.automerge"],
        "le fonds rouvert n'a pas son seul fichier complet"
    );
}

#[test]
fn un_fichier_etranger_est_laisse_tel_quel() {
    let chemin = commun::dossier_jetable("demarrage-fichier-etranger").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    let heads = fonds.automerge().get_heads();
    drop(fonds);
    let etrangers = [
        dossier_des_ajouts(&chemin).join("desktop.ini"),
        dossier_du_fonds(&chemin).join("Thumbs.db"),
    ];
    for etranger in &etrangers {
        fs::write(etranger, "inventé")
            .unwrap_or_else(|e| panic!("déposer {} : {e}", etranger.display()));
    }

    let mut fonds = ouvrir_le_fonds(&racine);
    fonds.compacter().expect("compacter");

    for etranger in &etrangers {
        assert!(
            fs::read(etranger).unwrap_or_else(|e| panic!("relire {} : {e}", etranger.display()))
                == "inventé".as_bytes(),
            "{} n'est plus là, octet pour octet",
            etranger.display()
        );
    }
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "un fichier étranger met le fonds en lecture seule"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads,
        "le fonds rouvert n'a pas les heads d'avant"
    );
}

#[test]
fn le_demarrage_ne_touche_rien_sans_le_verrou() {
    let chemin = commun::dossier_jetable("demarrage-sans-le-verrou").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let documents = chemin.join("documents");
    let captations = chemin.join("captations");
    let zeros = "0".repeat(64);
    fs::write(
        dossier_des_ajouts(&chemin).join(format!("{zeros}.automerge.4242.partiel")),
        b"une ecriture interrompue",
    )
    .expect("déposer un .partiel dans ajouts/");
    fs::write(
        captations.join(format!("{zeros}.4242.partiel")),
        b"une captation interrompue",
    )
    .expect("déposer un .partiel dans captations/");
    let documents_avant = empreintes(&documents);
    let captations_avant = empreintes(&captations);

    match RacineDesDonnees::ouvrir(&chemin) {
        Err(Erreur::RacineDesDonneesTenue) => {}
        Err(autre) => panic!("attendu RacineDesDonneesTenue, rendu {autre:?}"),
        Ok(_) => panic!("la seconde ouverture a pris le verrou que tient la première"),
    }

    // Jamais `empreintes` de la racine des données : sous Windows, le verrou que tient la racine
    // ouverte peut refuser la lecture du fichier `verrou`.
    assert_eq!(
        empreintes(&documents),
        documents_avant,
        "documents/ a changé sans le verrou"
    );
    assert_eq!(
        empreintes(&captations),
        captations_avant,
        "captations/ a changé sans le verrou"
    );
    drop(racine);
}
