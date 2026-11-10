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
use noyau::donnees::acces::Acces;
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
    /// Automerge a refusé : le geste, une écriture du document, la lecture de son fichier
    /// complet ou d'un fichier d'ajout, ou celle de sa racine.
    Automerge(AutomergeError),
    /// Le contenu du fichier d'ajout `chemin` ne donne pas l'empreinte que porte son nom : le
    /// document ne s'ouvre pas, et rien n'est retiré ni réécrit.
    AjoutAltere { chemin: PathBuf },
    /// Ses fichiers lus, le document a une modification qui attend ses dépendances
    /// (`get_missing_deps`) : un fichier d'ajout manque, et le document ne s'ouvre pas.
    DependancesManquantes,
    /// Le document est en lecture seule, pour cette raison : il refuse ce que son accès ne
    /// permet pas (§ 31 · 2.1).
    LectureSeule(LectureSeule),
    /// Le fichier complet de la compaction, relu, ne mène pas aux heads du document en mémoire :
    /// aucun fichier d'ajout n'est retiré (§ 31 · 2.2). Relu avant d'être écrit, il ne se lit pas
    /// ou mène ailleurs, et il ne s'écrit pas ; ou, écrit, le disque n'en rend pas les octets.
    /// Après une écriture ou une réception, elle vient en avertissement
    /// ([`Avertissement::Compaction`]), ce qui est écrit ou reçu écrit.
    CompactionInfidele,
    /// La racine du document ne porte pas de chaîne sous `champ` : la copie ne se situe pas
    /// (§ 31 · 2.5).
    ChampAbsent { champ: &'static str },
    /// L'échange a refusé le message reçu. Ce que le message a pu apporter avant le refus est
    /// écrit ; l'avertissement de cette écriture, s'il en vient un, rejoint ceux du document
    /// (`Document::avertissements`).
    Synchro(synchro::Erreur),
}

/// Pourquoi un document est en lecture seule (§ 31 · 2.1) : il montre ce qui se lit, n'écrit ni ne
/// se compacte, n'envoie ni ne reçoit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LectureSeule {
    /// La relecture du disque, après une écriture refusée, a échoué sur `chemin` : le document
    /// montre sa mémoire d'avant l'écriture refusée.
    RelectureManquee { chemin: PathBuf },
}

impl LectureSeule {
    /// Ce que permet un document en lecture seule pour cette raison : une relecture manquée le
    /// laisse illisible, sans écriture ni échange.
    pub fn acces(&self) -> Acces {
        match self {
            LectureSeule::RelectureManquee { .. } => Acces::Illisible,
        }
    }
}

/// Ce qui a échoué après qu'une écriture est faite, sans la défaire (§ 31 · 2.1).
#[derive(Debug)]
pub enum Avertissement {
    /// Le fichier `chemin` est en place, mais son renommage n'a pas été rendu durable.
    NonDurable { chemin: PathBuf, erreur: io::Error },
    /// La compaction a échoué (`Disque` ou `CompactionInfidele`) : rien n'est retiré, et elle se
    /// retente à l'écriture suivante.
    Compaction(Erreur),
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
