//! Écrire à côté, puis renommer (§ 31 · 2.1) : un fichier est entier, ou absent.
//!
//! Le fichier s'écrit d'abord dans son voisin, qui prend ensuite sa place, comme l'export du
//! cahier des tables : un fichier qui existait reste entier jusqu'au dernier moment.
//!
//! Chaque étape d'une écriture a sa fonction, qui passe par le point d'injection d'une panne
//! juste avant son geste sur le disque : les écritures à côté (`ecrire_a_cote`), les
//! versements des captations et le démarrage des documents passent par elles.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use noyau::donnees::demarrage::voisin_libre;

use crate::pannes::{self, Etape};

/// Pourquoi une écriture à côté n'est pas faite, ou pas tout à fait.
#[derive(Debug)]
pub enum Echec {
    /// Avant la fin du renommage : `chemin` garde ce qu'il portait, et le voisin est ôté, vidé
    /// puis retiré.
    Refusee(io::Error),
    /// Après le renommage : le fichier est en place, mais `rendre_durable` a échoué.
    NonDurable(io::Error),
}

/// Le premier nom libre de `<nom>.<processus>.partiel`, `-2`, `-3`…, dans le dossier de `chemin`,
/// lu sur le disque (§ 31 · 2.1) : un nom pris est un voisin qu'un arrêt a laissé, et il n'est
/// jamais touché. Le choix est celui de `noyau::donnees::demarrage::voisin_libre`.
///
/// Un dossier qui ne se lit pas rend l'erreur du disque, avant tout voisin.
pub fn voisin(chemin: &Path) -> io::Result<PathBuf> {
    // Le dossier d'un nom seul, sans dossier, est le dossier courant.
    let dossier = match chemin.parent() {
        Some(dossier) if !dossier.as_os_str().is_empty() => dossier,
        _ => Path::new("."),
    };
    let mut pris = Vec::new();
    for entree in fs::read_dir(dossier)? {
        pris.push(entree?.file_name().to_string_lossy().into_owned());
    }
    let nom = chemin.file_name().unwrap_or_default().to_string_lossy();
    Ok(chemin.with_file_name(voisin_libre(&nom, std::process::id(), &pris)))
}

/// Écrit le voisin, le synchronise (`sync_all`), le ferme, le relit, le renomme en `chemin`
/// (`std::fs::rename`, qui remplace au besoin), puis rend le renommage durable.
///
/// Un échec avant ou pendant le renommage rend [`Echec::Refusee`] : `chemin` garde ce qu'il
/// portait, et le voisin est ôté ; un voisin relu qui ne rend pas `octets` ne prend pas la
/// place, et l'erreur est alors `io::ErrorKind::InvalidData`. Ôter le voisin, c'est le vider
/// (zéro octet), rendre ce vide durable (`sync_all`), puis le retirer : un retrait qui échoue, ou
/// qu'une coupure perd, laisse un voisin vide, et vide, il ne se lit pas. Un échec de
/// [`rendre_durable`] arrive après le renommage, et rend [`Echec::NonDurable`] : le fichier est
/// déjà en place sous `chemin`, avec ses nouveaux octets, et il n'y a plus de voisin à ôter.
pub fn ecrire_a_cote(chemin: &Path, octets: &[u8]) -> Result<(), Echec> {
    // Un dossier qui ne se lit pas refuse avant tout voisin : il n'y a rien à ôter.
    let voisin = voisin(chemin).map_err(Echec::Refusee)?;
    if let Err(erreur) = ecrire_relire_puis_renommer(&voisin, chemin, octets) {
        oter(&voisin);
        return Err(Echec::Refusee(erreur));
    }
    // Renommé, le voisin n'est plus : il n'y a rien à ôter si le renommage ne devient pas
    // durable.
    rendre_durable_apres_le_renommage(chemin).map_err(Echec::NonDurable)
}

fn ecrire_relire_puis_renommer(voisin: &Path, chemin: &Path, octets: &[u8]) -> io::Result<()> {
    ecrire_et_synchroniser(voisin, octets)?;
    if lire(voisin)? != octets {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "le voisin relu ne rend pas les octets écrits",
        ));
    }
    renommer(voisin, chemin)
}

/// Ôte le voisin d'une écriture refusée, dans cet ordre : il se vide (ouvert sans création,
/// tronqué à zéro octet), ce vide est rendu durable (`sync_all`), le voisin se ferme, puis il se
/// retire (`retirer`). Une coupure qui perd le retrait le laisse donc vide, sous les deux
/// systèmes : seul un voisin dont le vide n'a pu être rendu durable, et qui ne se retire pas, ou
/// dont une coupure perd le retrait, peut revenir avec ses octets. Les erreurs sont tues : celle
/// que rend l'écriture reste la sienne ; un voisin qui ne se vide pas se retire quand même.
fn oter(voisin: &Path) {
    match fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(voisin)
    {
        // Fermé à la fin du bras, avant le retrait.
        Ok(fichier) => {
            let _ = fichier.sync_all();
        }
        // Le voisin n'est pas né : il n'y a rien à vider, ni à retirer.
        Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => return,
        Err(_) => {}
    }
    let _ = retirer(voisin);
}

/// Écrit `octets` dans `chemin`, le synchronise (`sync_all`), puis le ferme. Les points :
/// [`Etape::VoisinEcrit`] après la création, [`Etape::SyncAll`] avant la synchronisation.
pub(crate) fn ecrire_et_synchroniser(chemin: &Path, octets: &[u8]) -> io::Result<()> {
    let mut fichier = File::create(chemin)?;
    pannes::point(Etape::VoisinEcrit)?;
    fichier.write_all(octets)?;
    pannes::point(Etape::SyncAll)?;
    // Fermé à la sortie, avant de prendre la place : Windows ne renomme pas un fichier ouvert.
    fichier.sync_all()
}

/// Les octets de `chemin` (`fs::read`), passés par `pannes::relue`. Le point :
/// [`Etape::Relecture`].
pub(crate) fn lire(chemin: &Path) -> io::Result<Vec<u8>> {
    pannes::point(Etape::Relecture)?;
    fs::read(chemin).map(pannes::relue)
}

/// Renomme `de` en `vers` (`fs::rename`, qui remplace au besoin). Le point :
/// [`Etape::Renommage`].
pub(crate) fn renommer(de: &Path, vers: &Path) -> io::Result<()> {
    pannes::point(Etape::Renommage)?;
    fs::rename(de, vers)
}

/// Retire `chemin` (`fs::remove_file`). Le point : [`Etape::Retrait`].
pub(crate) fn retirer(chemin: &Path) -> io::Result<()> {
    pannes::point(Etape::Retrait)?;
    fs::remove_file(chemin)
}

/// Rend durable le renommage qui vient de mettre `chemin` en place, par [`rendre_durable`]. Le
/// point : [`Etape::RendreDurable`] ; la synchronisation d'un dossier né n'en est pas un.
pub(crate) fn rendre_durable_apres_le_renommage(chemin: &Path) -> io::Result<()> {
    pannes::point(Etape::RendreDurable)?;
    rendre_durable(chemin)
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
