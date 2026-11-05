//! Les aides des tests du stockage : le dossier jetable de chaque test, ce que contient un
//! dossier, des octets engendrés au lieu d'un fichier, et la copie d'un arbre de fichiers.

// Chaque fichier de test n'emploie qu'une partie des aides.
#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Le dossier jetable du test `nom`, dans le dossier temporaire du système, jamais dans le
/// dossier de l'application (I30) : `pupitre-stockage-<nom>-<processus>`. Ce qu'un passage
/// précédent y a laissé est effacé, puis le dossier est recréé, vide. C'est le seul endroit
/// des tests qui devine un chemin.
pub fn dossier_jetable(nom: &str) -> PathBuf {
    let dossier =
        std::env::temp_dir().join(format!("pupitre-stockage-{nom}-{}", std::process::id()));
    match fs::remove_dir_all(&dossier) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => panic!("effacer {} : {e}", dossier.display()),
    }
    fs::create_dir_all(&dossier).unwrap_or_else(|e| panic!("créer {} : {e}", dossier.display()));
    dossier
}

/// Les noms de ce que contient `dossier`, dans l'ordre.
pub fn entrees(dossier: &Path) -> Vec<String> {
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

/// Un générateur congruentiel ou xorshift semé par `graine`, dont on garde les bits hauts : une
/// suite déterministe, sans période courte, sans fichier.
///
/// Ici, le générateur congruentiel de Knuth (MMIX), modulo 2⁶⁴, de période pleine. Chaque octet
/// rendu est l'octet de tête de son état : les bits bas d'un tel générateur ont des périodes
/// courtes, ses bits hauts les plus longues.
pub fn octets(graine: u8, longueur: usize) -> Vec<u8> {
    const MULTIPLICATEUR: u64 = 6_364_136_223_846_793_005;
    const INCREMENT: u64 = 1_442_695_040_888_963_407;
    let mut etat = u64::from(graine) ^ 0x9E37_79B9_7F4A_7C15;
    (0..longueur)
        .map(|_| {
            etat = etat.wrapping_mul(MULTIPLICATEUR).wrapping_add(INCREMENT);
            (etat >> 56) as u8
        })
        .collect()
}

/// Copie l'arbre de `de` dans `vers`, dossiers et fichiers, comme une sauvegarde ou une copie
/// de la racine des données le ferait : le fichier `verrou` compris.
pub fn copier_l_arbre(de: &Path, vers: &Path) {
    fs::create_dir_all(vers).unwrap_or_else(|e| panic!("créer {} : {e}", vers.display()));
    let entrees = fs::read_dir(de).unwrap_or_else(|e| panic!("lire {} : {e}", de.display()));
    for entree in entrees {
        let entree = entree.unwrap_or_else(|e| panic!("lire {} : {e}", de.display()));
        let source = entree.path();
        let cible = vers.join(entree.file_name());
        let sorte = entree
            .file_type()
            .unwrap_or_else(|e| panic!("lire {} : {e}", source.display()));
        if sorte.is_dir() {
            copier_l_arbre(&source, &cible);
        } else {
            fs::copy(&source, &cible).unwrap_or_else(|e| {
                panic!("copier {} vers {} : {e}", source.display(), cible.display())
            });
        }
    }
}
