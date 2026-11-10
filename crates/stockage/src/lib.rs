//! Le stockage de pupitre, côté natif seulement (§ 40 · 3) : la racine des données et son
//! verrou, puis les adaptateurs `Documents` et `Captations` sur fichiers, au format de
//! l'ADR-023 (§ 31 · 2.1). Il ne devine aucun chemin : il reçoit celui de la racine des
//! données (I30).
//!
//! La fonction `pannes`, que seuls ses tests demandent, rend public le module `pannes`, qui
//! injecte une panne à chaque étape d'une écriture ; sans elle, ses points laissent tout passer.

mod captations;
mod documents;
pub mod fichier;
#[cfg(feature = "pannes")]
pub mod pannes;
#[cfg(not(feature = "pannes"))]
mod pannes;
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
    /// Un `Document` du fonds vit déjà dans ce processus : un seul à la fois (AM-4 (1)).
    FondsDejaOuvert,
    /// Le fonds est déjà né : il ne naît qu'une fois (§ 31 · 2.3).
    FondsDejaNe,
    /// La genèse, rangée en octets dans le noyau, ne se lit pas comme une modification, ou ne
    /// s'applique pas ; le texte est celui d'Automerge.
    Genese(String),
    /// Automerge a refusé : le geste, une écriture du document, la lecture de son fichier
    /// complet ou d'un fichier d'ajout, ou celle de sa racine.
    Automerge(AutomergeError),
    /// Le contenu du fichier d'ajout `chemin` ne donne pas l'empreinte que porte son nom : le
    /// document ne s'ouvre pas, et rien n'est retiré ni réécrit.
    AjoutAltere { chemin: PathBuf },
    /// Ses fichiers lus, le document a une modification qui attend ses dépendances
    /// (`get_missing_deps`) : un fichier d'ajout manque, et le document ne s'ouvre pas.
    DependancesManquantes,
    /// Une écriture a échoué : le document refuse d'écrire, d'envoyer, de recevoir et de se
    /// compacter, jusqu'à sa réouverture.
    ARelire,
    /// Le fichier complet de la compaction, relu, ne mène pas aux heads du document en mémoire :
    /// aucun fichier d'ajout n'est retiré (§ 31 · 2.2). Relu avant d'être écrit, il ne se lit pas
    /// ou mène ailleurs, et il ne s'écrit pas ; ou, écrit, le disque n'en rend pas les octets.
    /// Après une écriture, `ecrire` la rend, l'écriture faite ; `recevoir` de même, ce qui est
    /// reçu écrit.
    CompactionInfidele,
    /// La racine du document ne porte pas de chaîne sous `champ` : la copie ne se situe pas
    /// (§ 31 · 2.5).
    ChampAbsent { champ: &'static str },
    /// L'échange a refusé le message reçu. Ce que le message a pu apporter avant le refus est
    /// écrit.
    Synchro(synchro::Erreur),
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
