//! Écrire à côté, puis renommer (§ 31 · 2.1).

mod commun;

use std::fs;
use std::io;
use std::path::PathBuf;

use commun::{dossier_jetable, entrees};
use stockage::fichier::{Echec, ecrire_a_cote, voisin};
use stockage::pannes::{self, Etape};

#[test]
fn le_voisin_porte_le_numero_du_processus() {
    let d = dossier_jetable("fichier-voisin-nom");

    assert_eq!(
        voisin(&d.join("complet.automerge")).unwrap(),
        d.join(format!("complet.automerge.{}.partiel", std::process::id()))
    );
}

// § 31 · 2.1 : un nom de voisin pris reçoit un suffixe, et ce qu'il porte n'est jamais touché
#[test]
fn un_nom_de_voisin_pris_recoit_un_suffixe() {
    let d = dossier_jetable("fichier-voisin-suffixe");
    let chemin = d.join("complet.automerge");
    let pid = std::process::id();
    let nu = d.join(format!("complet.automerge.{pid}.partiel"));
    let deux = d.join(format!("complet.automerge.{pid}-2.partiel"));
    let trois = d.join(format!("complet.automerge.{pid}-3.partiel"));

    fs::write(&nu, "laissé").unwrap();
    assert_eq!(voisin(&chemin).unwrap(), deux);
    fs::write(&deux, "laissé").unwrap();
    assert_eq!(voisin(&chemin).unwrap(), trois);
    fs::remove_file(&deux).unwrap();
    assert_eq!(voisin(&chemin).unwrap(), deux);

    ecrire_a_cote(&chemin, b"apres").expect("écrire à côté, le nom du voisin étant pris");

    assert_eq!(fs::read(&chemin).unwrap(), b"apres");
    assert_eq!(fs::read(&nu).unwrap(), "laissé".as_bytes());
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
        matches!(rendu, Err(Echec::Refusee(_))),
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

// § 31 · 10, test 8 : interrompre une écriture à chaque étape
#[test]
fn ecrire_a_cote_coupee_a_chaque_etape() {
    // Avant la fin du renommage : refusée, le fichier garde ce qu'il portait, et le voisin est
    // retiré.
    for (i, etape) in [
        Etape::VoisinEcrit,
        Etape::SyncAll,
        Etape::Relecture,
        Etape::Renommage,
    ]
    .into_iter()
    .enumerate()
    {
        let d = dossier_jetable(&format!("fichier-coupee-{i}"));
        let chemin = d.join("complet.automerge");
        fs::write(&chemin, b"avant").unwrap();
        let _panne = pannes::couper(etape, 1);
        assert!(
            matches!(ecrire_a_cote(&chemin, b"apres"), Err(Echec::Refusee(_))),
            "{etape:?}"
        );
        assert_eq!(fs::read(&chemin).unwrap(), b"avant", "{etape:?}");
        assert_eq!(entrees(&d), ["complet.automerge"], "{etape:?}");
    }

    // Après le renommage : non durable, le fichier porte déjà ses nouveaux octets, et il n'y a
    // plus de voisin.
    let d = dossier_jetable("fichier-coupee-rendre-durable");
    let chemin = d.join("complet.automerge");
    fs::write(&chemin, b"avant").unwrap();
    let _panne = pannes::couper(Etape::RendreDurable, 1);
    let rendu = ecrire_a_cote(&chemin, b"apres");
    assert!(
        matches!(rendu, Err(Echec::NonDurable(_))),
        "RendreDurable : rendu {rendu:?}"
    );
    assert_eq!(fs::read(&chemin).unwrap(), b"apres");
    assert_eq!(entrees(&d), ["complet.automerge"], "un voisin reste");
}

// § 31 · 10, test 8 : renommage refusé, le voisin est retiré ; sa double panne le vide
#[test]
fn un_voisin_qui_ne_se_retire_pas_se_vide() {
    let d = dossier_jetable("fichier-voisin-vide");
    let chemin = d.join("complet.automerge");
    fs::write(&chemin, b"avant").unwrap();
    let _renommage = pannes::couper(Etape::Renommage, 1);
    let _retrait = pannes::couper(Etape::Retrait, 1);

    let rendu = ecrire_a_cote(&chemin, b"apres");

    assert!(matches!(rendu, Err(Echec::Refusee(_))), "rendu {rendu:?}");
    assert_eq!(fs::read(&chemin).unwrap(), b"avant");
    // Le voisin, nommé en clair et non par `voisin()`, dont le nom pourra prendre un suffixe
    // quand il est déjà pris (§ 31 · 2.1).
    let partiel =
        chemin.with_file_name(format!("complet.automerge.{}.partiel", std::process::id()));
    assert!(partiel.is_file(), "le voisin qui ne se retire pas reste");
    assert_eq!(
        fs::metadata(&partiel).expect("lire le voisin").len(),
        0,
        "le voisin qui ne se retire pas se vide"
    );
}

// LE-10, § 31 · 2.2, point 3 : un voisin relu qui ne rend pas les octets écrits ne prend pas la place
#[test]
fn un_voisin_relu_autre_ne_prend_pas_la_place() {
    let d = dossier_jetable("fichier-relu-autre");
    let chemin = d.join("complet.automerge");
    fs::write(&chemin, b"avant").unwrap();
    let _alteration = pannes::alterer_la_relecture(1);

    match ecrire_a_cote(&chemin, b"apres") {
        Err(Echec::Refusee(e)) => assert_eq!(e.kind(), io::ErrorKind::InvalidData, "{e:?}"),
        autre => panic!("attendu Refusee, rendu {autre:?}"),
    }
    assert_eq!(fs::read(&chemin).unwrap(), b"avant");
    assert_eq!(entrees(&d), ["complet.automerge"], "un voisin reste");
}
