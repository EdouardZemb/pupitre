//! La racine des données et son verrou (§ 31 · 2.1).

mod commun;

use std::fs;

use stockage::{Erreur, RacineDesDonnees};

#[test]
fn ouvrir_une_racine_des_donnees_neuve_cree_ses_dossiers_et_son_verrou() {
    let chemin = commun::dossier_jetable("neuve").join("donnees");

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir une racine des données neuve");

    assert_eq!(racine.chemin(), chemin);
    assert!(chemin.join("documents").is_dir(), "documents/ manque");
    assert!(chemin.join("captations").is_dir(), "captations/ manque");
    assert!(chemin.join("verrou").is_file(), "le fichier verrou manque");
}

#[test]
fn une_racine_des_donnees_tenue_se_refuse() {
    let chemin = commun::dossier_jetable("tenue").join("donnees");
    let premiere = RacineDesDonnees::ouvrir(&chemin).expect("la première ouverture");

    match RacineDesDonnees::ouvrir(&chemin) {
        Err(Erreur::RacineDesDonneesTenue) => {}
        Err(autre) => panic!("attendu RacineDesDonneesTenue, rendu {autre:?}"),
        Ok(_) => panic!("la seconde ouverture a pris le verrou que tient la première"),
    }

    drop(premiere);
    RacineDesDonnees::ouvrir(&chemin).expect("la première fermée, la racine des données s'ouvre");
}

#[test]
fn la_seule_presence_du_verrou_ne_dit_rien() {
    // Une copie de la racine des données emporte le fichier `verrou`, sans le verrou que le
    // système tenait : il n'en reste qu'un fichier.
    let chemin = commun::dossier_jetable("presence").join("donnees");
    fs::create_dir_all(&chemin).expect("créer la racine des données");
    fs::write(chemin.join("verrou"), b"").expect("écrire le verrou d'avance");

    let racine = RacineDesDonnees::ouvrir(&chemin)
        .expect("un verrou laissé par une copie n'empêche pas d'ouvrir");

    assert_eq!(racine.chemin(), chemin);
    assert!(chemin.join("documents").is_dir(), "documents/ manque");
    assert!(chemin.join("captations").is_dir(), "captations/ manque");
}
