//! La source à verser sur fichiers (§ 40 · 3, le troisième contrat) : le dossier que Sacha a
//! choisi, parcouru une fois, sans suivre un lien. Elle ne devine aucun chemin : elle reçoit celui
//! du dossier (I30). La marque de dernière écriture de chaque entrée est celle que tient le système
//! de fichiers, jamais une lecture de l'horloge.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use noyau::donnees::signature;
use services::{Entree, Raison, Refus, SorteDEntree, Source};

use crate::{Erreur, disque};

/// La source à verser sur fichiers : le dossier, ses entrées, le chemin de chacune, et les octets
/// lus de chacune, comptés par un `AtomicU64`, que deux fils peuvent lire.
#[derive(Debug)]
pub struct SourceSurFichiers {
    nom: Option<String>,
    entrees: Vec<Entree>,
    chemins: Vec<PathBuf>,
    lus: Vec<AtomicU64>,
}

/// Une entrée vue au parcours, avant son rang : ses segments, son chemin, sa sorte, ses octets et
/// sa marque de dernière écriture.
struct Vue {
    segments: Vec<OsString>,
    chemin: PathBuf,
    sorte: SorteDEntree,
    octets: u64,
    marque: Option<u64>,
}

impl SourceSurFichiers {
    /// Parcourt `dossier` une fois, par une pile, sans suivre un lien (`fs::symlink_metadata`,
    /// `read_dir` sur les seuls dossiers) ; une jonction compte pour un lien. Un dossier qui ne se
    /// liste pas : `Erreur::Disque`, sur lui. La marque de dernière écriture de chaque fichier :
    /// sous Windows, `std::os::windows::fs::MetadataExt::last_write_time`, `None` s'il vaut 0 (le
    /// système de fichiers ne la tient pas) ; ailleurs, `std::os::unix::fs::MetadataExt::mtime` ×
    /// 10⁹ + `mtime_nsec`, `None` si elle est négative ou déborde ; jamais `Metadata::modified`,
    /// dont la valeur est une heure de l'horloge (I30).
    ///
    /// Un lien est ce que `FileType::is_symlink` dit tel : sous Windows, tout point d'analyse qui
    /// substitue un nom, un lien symbolique comme une jonction. Les entrées se rangent par leurs
    /// segments, comparés un à un, puis se numérotent dès 1 ; un lien ou une autre entrée a 0
    /// octet, rien ne s'en lit. Une entrée qui ne se lit pas (`symlink_metadata`) rend
    /// `Erreur::Disque`, sur elle.
    pub fn parcourir(dossier: &Path) -> Result<SourceSurFichiers, Erreur> {
        let mut vues = Vec::new();
        let mut pile = vec![(dossier.to_path_buf(), Vec::new())];
        while let Some((courant, segments)) = pile.pop() {
            for entree in fs::read_dir(&courant).map_err(|erreur| disque(&courant, erreur))? {
                let entree = entree.map_err(|erreur| disque(&courant, erreur))?;
                let chemin = entree.path();
                let metadonnees =
                    fs::symlink_metadata(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
                let mut siens: Vec<OsString> = segments.clone();
                siens.push(entree.file_name());
                if metadonnees.is_dir() {
                    pile.push((chemin, siens));
                    continue;
                }
                let (sorte, octets) = sorte_et_octets(&metadonnees);
                vues.push(Vue {
                    segments: siens,
                    chemin,
                    sorte,
                    octets,
                    marque: marque(&metadonnees),
                });
            }
        }
        vues.sort_by(|a, b| a.segments.cmp(&b.segments));
        let mut entrees = Vec::with_capacity(vues.len());
        let mut chemins = Vec::with_capacity(vues.len());
        for (rang, vue) in (1..).zip(vues) {
            entrees.push(Entree {
                rang,
                chemin: chemin_relatif(&vue.segments),
                extension: vue
                    .segments
                    .last()
                    .map(OsString::as_os_str)
                    .and_then(extension),
                octets: vue.octets,
                sorte: vue.sorte,
                marque: vue.marque,
            });
            chemins.push(vue.chemin);
        }
        let lus = chemins.iter().map(|_| AtomicU64::new(0)).collect();
        Ok(SourceSurFichiers {
            nom: dossier
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_string),
            entrees,
            chemins,
            lus,
        })
    }

    /// L'indice de l'entrée `rang` ; un rang sans entrée est refusé.
    fn indice(&self, rang: u64) -> Result<usize, Refus> {
        rang.checked_sub(1)
            .and_then(|indice| usize::try_from(indice).ok())
            .filter(|indice| *indice < self.entrees.len())
            .ok_or_else(|| Refus {
                raison: Raison::Impossible,
                message: format!("Le dossier versé n'a pas d'entrée n° {rang}."),
            })
    }

    /// L'indice du fichier `rang`, qui se lit : un rang sans entrée, ou une entrée qui n'est pas un
    /// fichier, est refusé (`Raison::Impossible`) ; un lien ne s'ouvre jamais.
    fn fichier(&self, rang: u64) -> Result<usize, Refus> {
        let indice = self.indice(rang)?;
        if self.entrees[indice].sorte != SorteDEntree::Fichier {
            return Err(Refus {
                raison: Raison::Impossible,
                message: format!(
                    "L'entrée n° {rang} du dossier versé n'est pas un fichier : elle ne se lit pas."
                ),
            });
        }
        Ok(indice)
    }

    /// Les métadonnées de l'entrée `rang` à cet instant, sans suivre un lien.
    fn metadonnees(&self, rang: u64) -> Result<Metadata, Refus> {
        let indice = self.indice(rang)?;
        fs::symlink_metadata(&self.chemins[indice]).map_err(refus_du_fichier)
    }
}

impl Source for SourceSurFichiers {
    fn nom_du_dossier(&self) -> Option<&str> {
        self.nom.as_deref()
    }

    fn entrees(&self) -> &[Entree] {
        &self.entrees
    }

    /// `seek` à `debut`, puis `take` de `longueur` octets ; le fichier refermé au retour. Les
    /// octets lus s'ajoutent à ceux de l'entrée.
    fn lire(&self, rang: u64, debut: u64, longueur: usize) -> Result<Vec<u8>, Refus> {
        let indice = self.fichier(rang)?;
        let mut fichier = File::open(&self.chemins[indice]).map_err(refus_du_fichier)?;
        fichier
            .seek(SeekFrom::Start(debut))
            .map_err(refus_du_fichier)?;
        let mut octets = Vec::new();
        fichier
            .take(longueur as u64)
            .read_to_end(&mut octets)
            .map_err(refus_du_fichier)?;
        self.lus[indice].fetch_add(octets.len() as u64, Ordering::Relaxed);
        Ok(octets)
    }

    /// Le fichier ouvert, en flux ; chaque lecture du flux ajoute ses octets à ceux de l'entrée.
    fn lire_en_flux(&self, rang: u64) -> Result<Box<dyn io::Read + '_>, Refus> {
        let indice = self.fichier(rang)?;
        let fichier = File::open(&self.chemins[indice]).map_err(refus_du_fichier)?;
        Ok(Box::new(FluxCompte {
            fichier,
            lus: &self.lus[indice],
        }))
    }

    /// Relus sans suivre un lien ; un lien ou une autre entrée : 0, comme au parcours.
    fn octets_actuels(&self, rang: u64) -> Result<u64, Refus> {
        Ok(sorte_et_octets(&self.metadonnees(rang)?).1)
    }

    /// Relue sans suivre un lien, comme au parcours.
    fn marque_actuelle(&self, rang: u64) -> Result<Option<u64>, Refus> {
        Ok(marque(&self.metadonnees(rang)?))
    }

    /// Un rang sans entrée n'a rien lu : 0.
    fn octets_lus(&self, rang: u64) -> u64 {
        self.indice(rang)
            .map_or(0, |indice| self.lus[indice].load(Ordering::Relaxed))
    }
}

/// Le flux d'un fichier de la source, qui compte ses octets lus.
struct FluxCompte<'s> {
    fichier: File,
    lus: &'s AtomicU64,
}

impl io::Read for FluxCompte<'_> {
    fn read(&mut self, tampon: &mut [u8]) -> io::Result<usize> {
        let lus = self.fichier.read(tampon)?;
        self.lus.fetch_add(lus as u64, Ordering::Relaxed);
        Ok(lus)
    }
}

/// Le refus d'une lecture du fichier à verser : `Raison::Disque`, son message le texte
/// d'[`Erreur::FichierAVerser`], le constat du genre de l'erreur, jamais le texte du système.
fn refus_du_fichier(erreur: io::Error) -> Refus {
    Refus {
        raison: Raison::Disque,
        message: Erreur::FichierAVerser { erreur }.to_string(),
    }
}

/// La sorte d'une entrée lue sans suivre un lien, et ses octets : un fichier, sa taille ; un lien
/// ou une autre entrée, 0.
fn sorte_et_octets(metadonnees: &Metadata) -> (SorteDEntree, u64) {
    let sorte = metadonnees.file_type();
    if sorte.is_symlink() {
        (SorteDEntree::Lien, 0)
    } else if sorte.is_file() {
        (SorteDEntree::Fichier, metadonnees.len())
    } else {
        (SorteDEntree::Autre, 0)
    }
}

/// Le chemin relatif d'une entrée, ses segments joints par `/`, si chacun est de l'UTF-8.
fn chemin_relatif(segments: &[OsString]) -> Option<String> {
    let segments: Option<Vec<&str>> = segments.iter().map(|segment| segment.to_str()).collect();
    Some(segments?.join("/"))
}

/// L'extension du dernier segment d'une entrée, si elle est de l'UTF-8 : celle de
/// `signature::extension` d'un segment en UTF-8, celle de `Path::extension` sinon, qui suit la
/// même règle.
fn extension(dernier: &OsStr) -> Option<String> {
    let lue = match dernier.to_str() {
        Some(segment) => signature::extension(segment),
        None => Path::new(dernier).extension().and_then(OsStr::to_str),
    };
    lue.map(str::to_string)
}

/// La marque de dernière écriture que tient le système de fichiers : `last_write_time`, `None`
/// s'il vaut 0, que rend un système de fichiers qui ne la tient pas (V4).
#[cfg(windows)]
fn marque(metadonnees: &Metadata) -> Option<u64> {
    use std::os::windows::fs::MetadataExt;
    let marque = metadonnees.last_write_time();
    (marque != 0).then_some(marque)
}

/// La marque de dernière écriture que tient le système de fichiers : `mtime` × 10⁹ +
/// `mtime_nsec`, `None` si elle est négative ou déborde.
#[cfg(unix)]
fn marque(metadonnees: &Metadata) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    let secondes = u64::try_from(metadonnees.mtime()).ok()?;
    let nanosecondes = u64::try_from(metadonnees.mtime_nsec()).ok()?;
    secondes
        .checked_mul(1_000_000_000)?
        .checked_add(nanosecondes)
}
