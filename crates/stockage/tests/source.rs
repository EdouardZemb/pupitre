//! La source à verser sur fichiers (§ 40 · 3, le troisième contrat) : un dossier parcouru une
//! fois, sans suivre un lien, ses entrées rangées par leurs segments, les octets lus de chacune
//! comptés, et la marque de dernière écriture que tient le système de fichiers ; un rang sans
//! entrée, un lien et un fichier retiré ne se lisent pas. Les noms que seul Windows sait écrire
//! (balayage Q20) : un chemin de plus de 300 unités UTF-16, les deux formes Unicode d'un nom, une
//! jonction, un nom sans forme UTF-8, et un fichier tenu ouvert sans partage.

mod commun;

use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use services::{Raison, Refus, SorteDEntree, Source};
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

/// Un nom sans forme UTF-8 : une demi-paire de substitution seule (`OsString::from_wide`), suivie
/// de `a.wav`.
#[cfg(windows)]
fn nom_sans_forme_utf8() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    let mut nom = OsString::from_wide(&[0xD800]);
    nom.push("a.wav");
    nom
}

/// Un nom sans forme UTF-8 : l'octet `0xFF` (`OsStr::from_bytes`), suivi de `a.wav`.
#[cfg(unix)]
fn nom_sans_forme_utf8() -> OsString {
    use std::os::unix::ffi::OsStrExt;
    let mut nom = std::ffi::OsStr::from_bytes(&[0xFF]).to_os_string();
    nom.push("a.wav");
    nom
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
    assert_eq!(
        source.lire(1, 0, 16),
        Err(Refus {
            raison: Raison::Impossible,
            message: "L'entrée n° 1 du dossier versé n'est pas un fichier : elle ne se lit pas."
                .to_string(),
        }),
        "le lien se lit comme un fichier"
    );
}

#[test]
fn un_rang_sans_entree_ne_se_lit_pas() {
    let dossier = commun::dossier_jetable("source-rang-absent").join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    fs::write(dossier.join("seul.txt"), commun::octets(8, 60)).expect("écrire seul.txt");
    let sans_entree = |rang: u64| Refus {
        raison: Raison::Impossible,
        message: format!("Le dossier versé n'a pas d'entrée n° {rang}."),
    };

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    assert_eq!(source.lire(2, 0, 1), Err(sans_entree(2)));
    assert_eq!(source.lire(0, 0, 1), Err(sans_entree(0)));
    match source.lire_en_flux(2) {
        Err(refus) => assert_eq!(refus, sans_entree(2)),
        Ok(_) => panic!("un rang sans entrée se lit en flux"),
    }
    assert_eq!(source.octets_actuels(2), Err(sans_entree(2)));
    assert_eq!(source.marque_actuelle(2), Err(sans_entree(2)));
    assert_eq!(source.octets_lus(2), 0);
}

// Balayage Q8 : un fichier retiré après le parcours ne se lit plus, au constat du fichier à verser
#[test]
fn un_fichier_retire_apres_le_parcours_est_absent() {
    let dossier = commun::dossier_jetable("source-fichier-retire").join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    let chemin = dossier.join("prise.wav");
    fs::write(&chemin, commun::octets(9, 500)).expect("écrire prise.wav");
    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    fs::remove_file(&chemin).expect("retirer prise.wav");

    let absent = Refus {
        raison: Raison::Disque,
        message: "Le fichier à verser est absent.".to_string(),
    };
    assert_eq!(source.lire(1, 0, 16), Err(absent.clone()));
    assert_eq!(source.octets_actuels(1), Err(absent.clone()));
    assert_eq!(source.marque_actuelle(1), Err(absent));
    assert_eq!(source.octets_lus(1), 0);
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

// Balayage Q20 : `std::fs` pose seul le préfixe `\\?\` d'un chemin de plus de 260 unités
#[test]
fn un_chemin_de_300_unites_utf16_se_parcourt_et_garde_son_nom() {
    let dossier = commun::dossier_jetable("source-chemin-long").join("verse");
    let segments: Vec<String> = (1..=6)
        .map(|n| format!("étage-{n}-{}", "ô".repeat(50)))
        .collect();
    let attendu = format!("{}/prise.wav", segments.join("/"));
    let unites = attendu.encode_utf16().count();
    assert!(
        unites > 300,
        "le chemin relatif ne fait que {unites} unités"
    );
    let parent = segments
        .iter()
        .fold(dossier.clone(), |chemin, segment| chemin.join(segment));
    fs::create_dir_all(&parent).expect("créer les dossiers imbriqués");
    let contenu = commun::octets(10, 700);
    fs::write(parent.join("prise.wav"), &contenu).expect("écrire prise.wav");

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    let vues: Vec<_> = source
        .entrees()
        .iter()
        .map(|entree| {
            (
                entree.rang,
                entree.chemin.clone(),
                entree.octets,
                entree.sorte,
            )
        })
        .collect();
    assert_eq!(vues, [(1, Some(attendu), 700, SorteDEntree::Fichier)]);
    let mut lu = Vec::new();
    source
        .lire_en_flux(1)
        .expect("ouvrir prise.wav en flux")
        .read_to_end(&mut lu)
        .expect("lire prise.wav en flux");
    assert_eq!(lu, contenu);
}

// Balayage Q20 : NTFS garde comme deux noms deux fichiers qui ne diffèrent que par la forme
// Unicode, composée ou décomposée
#[test]
fn les_formes_composee_et_decomposee_d_un_nom_restent_deux_noms() {
    let dossier = commun::dossier_jetable("source-formes").join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    let composee = "\u{e9}t\u{e9}.txt";
    let decomposee = "e\u{301}te\u{301}.txt";
    let de_la_composee = commun::octets(11, 30);
    let de_la_decomposee = commun::octets(12, 40);
    fs::write(dossier.join(composee), &de_la_composee).expect("écrire la forme composée");
    fs::write(dossier.join(decomposee), &de_la_decomposee).expect("écrire la forme décomposée");

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    // Rangées par les octets de leurs segments : `e` (0x65) précède `é` (0xC3 0xA9).
    let vues: Vec<_> = source
        .entrees()
        .iter()
        .map(|entree| (entree.rang, entree.chemin.as_deref(), entree.octets))
        .collect();
    assert_eq!(vues, [(1, Some(decomposee), 40), (2, Some(composee), 30)]);
    assert_eq!(source.lire(1, 0, 100), Ok(de_la_decomposee));
    assert_eq!(source.lire(2, 0, 100), Ok(de_la_composee));
}

// Balayage Q20 : une jonction est un point d'analyse que `FileType::is_symlink` dit lien, quand
// `metadata` et `read_dir` la suivraient
#[cfg(windows)]
#[test]
fn une_jonction_compte_parmi_les_ignores_sans_etre_suivie() {
    let dossier = commun::dossier_jetable("source-jonction").join("verse");
    let cible = dossier.join("cible");
    fs::create_dir_all(&cible).expect("créer la cible");
    fs::write(cible.join("porte.wav"), commun::octets(13, 120)).expect("écrire cible/porte.wav");
    let lien = dossier.join("lien");
    let sortie = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&lien)
        .arg(&cible)
        .output()
        .expect("lancer mklink");
    assert!(
        sortie.status.success(),
        "mklink /J : {}{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    );
    assert!(
        lien.join("porte.wav").is_file(),
        "la jonction ne mène pas à sa cible"
    );

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    let vues: Vec<_> = source
        .entrees()
        .iter()
        .map(|entree| {
            (
                entree.rang,
                entree.chemin.as_deref(),
                entree.octets,
                entree.sorte,
            )
        })
        .collect();
    assert_eq!(
        vues,
        [
            (1, Some("cible/porte.wav"), 120, SorteDEntree::Fichier),
            (2, Some("lien"), 0, SorteDEntree::Lien),
        ],
        "la jonction est suivie, ou n'est pas une entrée"
    );
    assert_eq!(
        source.lire(2, 0, 16),
        Err(Refus {
            raison: Raison::Impossible,
            message: "L'entrée n° 2 du dossier versé n'est pas un fichier : elle ne se lit pas."
                .to_string(),
        }),
        "la jonction se lit comme un fichier"
    );
}

// Balayage Q20 : `OsStr::to_str` rend `None` pour un nom sans forme UTF-8, et `Path::extension`
// le lit encore
#[test]
fn un_nom_sans_forme_utf8_se_parcourt_sans_chemin() {
    let dossier = commun::dossier_jetable("source-sans-forme").join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    let contenu = commun::octets(14, 90);
    fs::write(dossier.join(nom_sans_forme_utf8()), &contenu)
        .expect("écrire le fichier au nom sans forme UTF-8");

    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

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
    assert_eq!(vues, [(1, None, Some("wav"), 90, SorteDEntree::Fichier)]);
    assert_eq!(source.lire(1, 0, 100), Ok(contenu));
}

// Question 38 : un fichier qu'un autre tient ouvert sans partage ne se lit pas, puis se lit une
// fois refermé
#[cfg(windows)]
#[test]
fn un_fichier_ouvert_sans_partage_ne_se_lit_pas() {
    use std::os::windows::fs::OpenOptionsExt;
    let dossier = commun::dossier_jetable("source-sans-partage").join("verse");
    fs::create_dir_all(&dossier).expect("créer le dossier à verser");
    let chemin = dossier.join("prise.wav");
    let contenu = commun::octets(15, 200);
    fs::write(&chemin, &contenu).expect("écrire prise.wav");
    let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir le dossier");

    let tenu = fs::File::options()
        .read(true)
        .share_mode(0)
        .open(&chemin)
        .expect("ouvrir prise.wav sans partage");
    let refus = source
        .lire(1, 0, 16)
        .expect_err("un fichier tenu sans partage se lit");
    assert_eq!(refus.raison, Raison::Disque, "{}", refus.message);
    drop(tenu);

    assert_eq!(source.lire(1, 0, 16), Ok(contenu[..16].to_vec()));
}
