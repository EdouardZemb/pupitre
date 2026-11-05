//! Les aides des tests du stockage : le dossier jetable de chaque test, ce que contient un
//! dossier, des octets engendrés au lieu d'un fichier, un texte long, la copie d'un arbre de
//! fichiers, et les gestes de test sur le fonds.

// Chaque fichier de test n'emploie qu'une partie des aides.
#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use automerge::transaction::{Transactable, Transaction};
use automerge::{AutomergeError, ObjId, ObjType, ROOT, ReadDoc, Value};
use noyau::donnees::fonds::{ETIQUETTES, FORMAT, IDEE, IDEES, PRISES, TEXTE};

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

/// 2 000 lettres de `a` à `z`, la n-ième `b'a' + octets(9, 2000)[n] % 26` : un texte que DEFLATE
/// ne ramène pas au poids d'un fichier d'ajout (`save` comprime le fichier complet, `save_after`
/// ne comprime pas les fichiers d'ajout).
pub fn texte_long() -> String {
    octets(9, 2000)
        .into_iter()
        .map(|octet| char::from(b'a' + octet % 26))
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

// Les gestes de test, qu'appellent les fermetures `geste` : les opérations nommées arrivent au
// lot 1. Une idée ou un conteneur absent rend `AutomergeError::NotAnObject`.

/// Crée l'idée `idee` sous les idées, avec son texte et son objet des étiquettes, vide, dans la
/// même modification : les conteneurs d'un objet naissent avec lui (§ 31 · 2.4).
pub fn noter(tx: &mut Transaction<'_>, idee: &str, texte: &str) -> Result<(), AutomergeError> {
    let idees = objet(tx, &ROOT, IDEES)?;
    let nee = tx.put_object(&idees, idee, ObjType::Map)?;
    tx.put(&nee, TEXTE, texte)?;
    tx.put_object(&nee, ETIQUETTES, ObjType::Map)?;
    Ok(())
}

/// Remplace le texte de l'idée `idee`.
pub fn changer_le_texte(
    tx: &mut Transaction<'_>,
    idee: &str,
    texte: &str,
) -> Result<(), AutomergeError> {
    let idee = l_idee(tx, idee)?;
    tx.put(&idee, TEXTE, texte)
}

/// Pose l'étiquette `etiquette` sur l'idée `idee`.
pub fn etiqueter(
    tx: &mut Transaction<'_>,
    idee: &str,
    etiquette: &str,
) -> Result<(), AutomergeError> {
    let etiquettes = objet(tx, &l_idee(tx, idee)?, ETIQUETTES)?;
    tx.put(&etiquettes, etiquette, true)
}

/// Ôte l'étiquette `etiquette` de l'idée `idee` ; n'écrit rien si l'étiquette manque.
pub fn oter_l_etiquette(
    tx: &mut Transaction<'_>,
    idee: &str,
    etiquette: &str,
) -> Result<(), AutomergeError> {
    let etiquettes = objet(tx, &l_idee(tx, idee)?, ETIQUETTES)?;
    if tx.get(&etiquettes, etiquette)?.is_none() {
        return Ok(());
    }
    tx.delete(&etiquettes, etiquette)
}

/// Pose, sous les prises, la prise d'empreinte `empreinte`, de l'idée `idee` et au format
/// `format`.
pub fn poser_une_prise(
    tx: &mut Transaction<'_>,
    empreinte: &str,
    idee: &str,
    format: &str,
) -> Result<(), AutomergeError> {
    let prises = objet(tx, &ROOT, PRISES)?;
    let prise = tx.put_object(&prises, empreinte, ObjType::Map)?;
    tx.put(&prise, IDEE, idee)?;
    tx.put(&prise, FORMAT, format)
}

/// L'objet de l'idée `idee`, sous les idées.
fn l_idee(tx: &Transaction<'_>, idee: &str) -> Result<ObjId, AutomergeError> {
    let idees = objet(tx, &ROOT, IDEES)?;
    objet(tx, &idees, idee)
}

/// L'objet que porte la clé `cle` de `parent` ; une clé absente, ou qui porte une valeur
/// simple, rend `AutomergeError::NotAnObject`.
fn objet(tx: &Transaction<'_>, parent: &ObjId, cle: &str) -> Result<ObjId, AutomergeError> {
    match tx.get(parent, cle)? {
        Some((Value::Object(_), objet)) => Ok(objet),
        _ => Err(AutomergeError::NotAnObject),
    }
}
