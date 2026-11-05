//! Le stockage de pupitre, côté natif seulement (§ 40 · 3) : la racine des données et son
//! verrou, puis les adaptateurs `Documents` et `Captations` sur fichiers, au format de
//! l'ADR-023 (§ 31 · 2.1). Il ne devine aucun chemin : il reçoit celui de la racine des
//! données (I30).

mod captations;
mod documents;
pub mod fichier;
mod racine_des_donnees;

use std::fmt::Write;
use std::io;
use std::path::{Path, PathBuf};

use automerge::AutomergeError;
use sha2::{Digest, Sha256};

pub use captations::Captations;
pub use documents::{Document, Naissance};
pub use racine_des_donnees::RacineDesDonnees;

/// Ce que le stockage refuse, ou ce que le disque lui refuse. Le texte en français de chaque
/// cas vient avec les commandes, au lot 1.
#[derive(Debug)]
pub enum Erreur {
    /// `TryLockError::WouldBlock` : un autre processus tient la racine des données.
    RacineDesDonneesTenue,
    /// Le disque a refusé une lecture, une écriture ou une création sur `chemin`.
    Disque {
        chemin: PathBuf,
        erreur: std::io::Error,
    },
    /// Le contenu relu de la captation `empreinte` ne donne pas son empreinte.
    CaptationAlteree { empreinte: String },
    /// Aucune captation ne porte le nom `empreinte`.
    CaptationAbsente { empreinte: String },
    /// Le fonds est déjà né : il ne naît qu'une fois (§ 31 · 2.3).
    FondsDejaNe,
    /// La genèse, rangée en octets dans le noyau, ne se lit pas comme une modification, ou ne
    /// s'applique pas ; le texte est celui d'Automerge.
    Genese(String),
    /// Automerge a refusé : le geste, une écriture du document, ou la lecture de son fichier
    /// complet.
    Automerge(AutomergeError),
}

/// Le SHA-256 de `octets`, en 64 caractères hexadécimaux minuscules (`Sha256::digest`).
pub fn empreinte(octets: &[u8]) -> String {
    let condense = Sha256::digest(octets);
    let mut e = String::with_capacity(64);
    for octet in condense.iter() {
        let _ = write!(e, "{octet:02x}");
    }
    e
}

/// Le refus du disque sur `chemin`.
pub(crate) fn disque(chemin: &Path, erreur: io::Error) -> Erreur {
    Erreur::Disque {
        chemin: chemin.to_path_buf(),
        erreur,
    }
}
