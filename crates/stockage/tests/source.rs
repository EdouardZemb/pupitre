//! La source à verser sur fichiers (§ 40 · 3, le troisième contrat) : un dossier parcouru une
//! fois, sans suivre un lien, ses entrées rangées par leurs segments, les octets lus de chacune
//! comptés, et la marque de dernière écriture que tient le système de fichiers.

mod commun;

use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use services::{SorteDEntree, Source};
use stockage::Erreur;
use stockage::source::SourceSurFichiers;

/// Date `chemin` à `quand` (`File::set_modified`), dans le test seul : la source ne lit jamais
/// l'horloge.
fn dater(chemin: &Path, quand: SystemTime) {
    fs::File::options()
        .write(true)
        .open(chemin)
        .unwrap_or_else(|e| panic!("ouvrir {} : {e}", chemin.display()))
        .set_modified(quand)
        .unwrap_or_else(|e| panic!("dater {} : {e}", chemin.display()));
}

#[test]
fn le_parcours_range_les_entrees_et_compte_les_octets_lus() {
    let dossier = commun::dossier_jetable("source-parcours").join("Démos");
    fs::create_dir_all(dossier.join("a")).expect("créer le dossier à verser");
    let b = commun::octets(1, 300);
    let c = commun::octets(2, 5000);
    let d = commun::octets(3, 40);
    fs::write(dossier.join("b.txt"), &b).expect("écrire b.txt");
    fs::write(dossier.join("a").join("c.wav"), &c).expect("écrire a/c.wav");
    fs::write(dossier.join("a").join("d.mid"), &d).expect("écrire a/d.mid");

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    assert_eq!(source.nom_du_dossier(), Some("Démos"));
    let vues: Vec<_> = source
        .entrees()
        .iter()
        .map(|entree| {
            (
                entree.rang,
                entree.chemin.as_deref(),
                entree.extension.as_deref(),
                entree.octets,
                entree.sorte,
            )
        })
        .collect();
    assert_eq!(
        vues,
        [
            (1, Some("a/c.wav"), Some("wav"), 5000, SorteDEntree::Fichier),
            (2, Some("a/d.mid"), Some("mid"), 40, SorteDEntree::Fichier),
            (3, Some("b.txt"), Some("txt"), 300, SorteDEntree::Fichier),
        ]
    );
    for rang in 1..=3 {
        assert_eq!(
            source.octets_lus(rang),
            0,
            "rang {rang}, avant toute lecture"
        );
    }

    assert_eq!(
        source.lire(1, 0, 16).expect("lire 16 octets de a/c.wav"),
        c[..16]
    );
    assert_eq!(source.octets_lus(1), 16);

    let mut lu = Vec::new();
    source
        .lire_en_flux(1)
        .expect("ouvrir a/c.wav en flux")
        .read_to_end(&mut lu)
        .expect("lire a/c.wav en flux");
    assert_eq!(lu, c);
    assert_eq!(source.octets_lus(1), 16 + 5000);
    assert_eq!(source.octets_lus(2), 0, "a/d.mid n'a pas été lu");
    assert_eq!(source.octets_lus(3), 0, "b.txt n'a pas été lu");
    assert_eq!(source.octets_actuels(3), Ok(300));
}

#[test]
fn un_dossier_absent_ne_se_parcourt_pas() {
    let absent = commun::dossier_jetable("source-absente").join("absent");

    match SourceSurFichiers::parcourir(&absent) {
        Err(Erreur::Disque { chemin, .. }) => assert_eq!(chemin, absent),
        autre => panic!("attendu Disque sur le dossier absent, rendu {autre:?}"),
    }
}

#[cfg(unix)]
#[test]
fn un_lien_symbolique_compte_sans_etre_suivi() {
    let d = commun::dossier_jetable("source-lien");
    let vise = d.join("vise");
    fs::create_dir_all(&vise).expect("créer le dossier visé");
    fs::write(vise.join("cache.wav"), commun::octets(4, 200)).expect("écrire le fichier visé");
    let dossier = d.join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    std::os::unix::fs::symlink(&vise, dossier.join("lien")).expect("créer le lien");
    fs::write(dossier.join("seul.txt"), commun::octets(5, 60)).expect("écrire seul.txt");

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    let vues: Vec<_> = source
        .entrees()
        .iter()
        .map(|entree| (entree.rang, entree.chemin.as_deref(), entree.sorte))
        .collect();
    assert_eq!(
        vues,
        [
            (1, Some("lien"), SorteDEntree::Lien),
            (2, Some("seul.txt"), SorteDEntree::Fichier),
        ],
        "le lien n'est pas une entrée, ou ce qu'il vise en est une"
    );
    assert!(
        source.lire(1, 0, 16).is_err(),
        "le lien se lit comme un fichier"
    );
}

// R8 : un contenu changé à taille égale se voit à sa marque de dernière écriture
#[test]
fn la_marque_suit_la_derniere_ecriture() {
    let dossier = commun::dossier_jetable("source-marque").join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    let chemin = dossier.join("prise.wav");
    fs::write(&chemin, commun::octets(6, 1000)).expect("écrire prise.wav");
    let quand = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    dater(&chemin, quand);

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    let marque = source.entrees()[0].marque;
    assert!(
        marque.is_some(),
        "le système de fichiers ne tient pas la marque"
    );
    assert_eq!(source.marque_actuelle(1), Ok(marque));

    fs::write(&chemin, commun::octets(7, 1000)).expect("récrire prise.wav, à la même taille");
    dater(&chemin, quand + Duration::from_secs(1));

    assert_eq!(source.octets_actuels(1), Ok(1000));
    let actuelle = source
        .marque_actuelle(1)
        .expect("relire la marque de dernière écriture");
    assert!(actuelle.is_some(), "la marque relue est absente");
    assert_ne!(
        actuelle, marque,
        "la marque n'a pas suivi la dernière écriture"
    );
}
