//! Écrire à côté, puis renommer (§ 31 · 2.1).

mod commun;

use std::fs;
use std::path::{Path, PathBuf};

use stockage::fichier::{ecrire_a_cote, voisin};

/// Les noms de ce que contient `dossier`, dans l'ordre.
fn entrees(dossier: &Path) -> Vec<String> {
    let mut noms: Vec<String> = fs::read_dir(dossier)
        .unwrap_or_else(|e| panic!("lire {} : {e}", dossier.display()))
        .map(|entree| {
            entree
                .unwrap_or_else(|e| panic!("lire {} : {e}", dossier.display()))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    noms.sort();
    noms
}

#[test]
fn le_voisin_porte_le_numero_du_processus() {
    let d = PathBuf::from("d");

    assert_eq!(
        voisin(&d.join("complet.automerge")),
        d.join(format!("complet.automerge.{}.partiel", std::process::id()))
    );
}

#[test]
fn ecrire_a_cote_ecrit_et_ne_laisse_rien_d_autre() {
    let d = commun::dossier_jetable("fichier-ecrit");
    let chemin = d.join("complet.automerge");
    let octets = commun::octets(1, 1024);

    ecrire_a_cote(&chemin, &octets).expect("écrire à côté, puis renommer");

    assert_eq!(fs::read(&chemin).expect("relire le fichier écrit"), octets);
    assert_eq!(entrees(&d), ["complet.automerge"], "rien d'autre ne reste");
}

#[test]
fn ecrire_a_cote_remplace_un_fichier_existant() {
    // La voie de Windows : `std::fs::rename` remplace le fichier en place, puis le fichier est
    // rouvert et synchronisé. L'ancien est plus long que le nouveau : rien n'en doit rester.
    let d = commun::dossier_jetable("fichier-remplace");
    let chemin = d.join("complet.automerge");
    fs::write(&chemin, commun::octets(2, 4096)).expect("écrire l'ancien fichier");
    let nouveaux = commun::octets(3, 512);

    ecrire_a_cote(&chemin, &nouveaux).expect("remplacer le fichier existant");

    assert_eq!(
        fs::read(&chemin).expect("relire le fichier écrit"),
        nouveaux
    );
    assert_eq!(entrees(&d), ["complet.automerge"], "rien d'autre ne reste");
}

#[test]
fn un_renommage_refuse_retire_le_voisin() {
    // Le chemin est celui d'un dossier qui porte un fichier : le voisin, écrit, ne peut en
    // prendre la place, et ce que le dossier porte ne bouge pas.
    let d = commun::dossier_jetable("fichier-refuse");
    let chemin = d.join("complet.automerge");
    fs::create_dir(&chemin).expect("créer le dossier qui tient la place");
    let garde = commun::octets(4, 64);
    fs::write(chemin.join("garde"), &garde).expect("écrire le fichier du dossier");

    let rendu = ecrire_a_cote(&chemin, &commun::octets(5, 256));

    assert!(
        rendu.is_err(),
        "un fichier ne prend pas la place d'un dossier : rendu {rendu:?}"
    );
    assert!(chemin.is_dir(), "le dossier a disparu");
    assert_eq!(entrees(&chemin), ["garde"], "le dossier a changé");
    assert_eq!(
        fs::read(chemin.join("garde")).expect("relire le fichier du dossier"),
        garde
    );
    assert_eq!(entrees(&d), ["complet.automerge"], "un .partiel reste");
}
