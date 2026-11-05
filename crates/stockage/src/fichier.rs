//! Écrire à côté, puis renommer (§ 31 · 2.1) : un fichier est entier, ou absent.
//!
//! Le fichier s'écrit d'abord dans son voisin, qui prend ensuite sa place, comme l'export du
//! cahier des tables : un fichier qui existait reste entier jusqu'au dernier moment.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// `<nom>.<processus>.partiel`, dans le dossier de `chemin`.
pub fn voisin(chemin: &Path) -> PathBuf {
    let mut nom = chemin.file_name().unwrap_or_default().to_os_string();
    nom.push(format!(".{}.partiel", std::process::id()));
    chemin.with_file_name(nom)
}

/// Écrit le voisin, le synchronise (`sync_all`), le ferme, le renomme en `chemin`
/// (`std::fs::rename`, qui remplace au besoin), puis rend le renommage durable.
///
/// Un échec avant ou pendant le renommage retire le voisin et rend l'erreur : `chemin` garde ce
/// qu'il portait. Un échec de [`rendre_durable`] arrive après le renommage : l'erreur se rend,
/// mais le fichier est déjà en place sous `chemin`, avec ses nouveaux octets, et il n'y a plus
/// de voisin à retirer.
pub fn ecrire_a_cote(chemin: &Path, octets: &[u8]) -> io::Result<()> {
    let voisin = voisin(chemin);
    if let Err(erreur) = ecrire_puis_renommer(&voisin, chemin, octets) {
        // Le voisin peut ne pas être né : son retrait peut alors échouer, et l'erreur rendue
        // reste celle de l'écriture.
        let _ = fs::remove_file(&voisin);
        return Err(erreur);
    }
    // Renommé, le voisin n'est plus : il n'y a rien à retirer si le renommage ne devient pas
    // durable.
    rendre_durable(chemin)
}

fn ecrire_puis_renommer(voisin: &Path, chemin: &Path, octets: &[u8]) -> io::Result<()> {
    let mut fichier = File::create(voisin)?;
    fichier.write_all(octets)?;
    fichier.sync_all()?;
    // Fermé avant de prendre la place : Windows ne renomme pas un fichier ouvert.
    drop(fichier);
    fs::rename(voisin, chemin)
}

/// Hors de Windows, synchronise le dossier de `chemin` ; sous Windows, rouvre `chemin` en
/// écriture, sans le tronquer, et le synchronise.
#[cfg(not(windows))]
pub fn rendre_durable(chemin: &Path) -> io::Result<()> {
    // Le dossier d'un nom seul, sans dossier, est le dossier courant.
    let dossier = match chemin.parent() {
        Some(dossier) if !dossier.as_os_str().is_empty() => dossier,
        _ => Path::new("."),
    };
    File::open(dossier)?.sync_all()
}

/// Hors de Windows, synchronise le dossier de `chemin` ; sous Windows, rouvre `chemin` en
/// écriture, sans le tronquer, et le synchronise.
///
/// Un dossier ne se synchronise pas sous Windows, qui garde en cache les métadonnées d'un
/// fichier : seul un fichier synchronisé (`FlushFileBuffers`) les écrit.
#[cfg(windows)]
pub fn rendre_durable(chemin: &Path) -> io::Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(chemin)?
        .sync_all()
}
