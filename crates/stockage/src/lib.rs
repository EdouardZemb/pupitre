//! Le stockage de pupitre, côté natif seulement (§ 40 · 3) : la racine des données et son
//! verrou, puis les adaptateurs `Documents` et `Captations` sur fichiers, au format de
//! l'ADR-023 (§ 31 · 2.1), qui réalisent les contrats des services (`FondsSurFichiers` et
//! `Captations`), et la source à verser sur fichiers (`source::SourceSurFichiers`), le troisième
//! contrat. Il ne devine aucun chemin : il reçoit celui de la racine des données, et celui du
//! dossier à verser (I30).
//!
//! La fonction `pannes`, que seuls ses tests demandent, coupe une écriture à l'étape voulue : elle
//! rend public le module `pannes`, qui injecte une panne à chaque étape d'une écriture ; sans
//! elle, ses points laissent tout passer. Elle ouvre aussi le fonds selon les migrations qu'un
//! test lui donne (`RacineDesDonnees::fonds_selon`, `FondsSurFichiers::ouvrir_selon`), et garde
//! pour les tests l'écriture et la naissance par une fermeture (`Document::ecrire`,
//! `RacineDesDonnees::faire_naitre_le_fonds`, `Naissance`).

mod adaptateurs;
mod captations;
mod demarrage;
mod documents;
pub mod fichier;
#[cfg(feature = "pannes")]
pub mod pannes;
#[cfg(not(feature = "pannes"))]
mod pannes;
mod racine_des_donnees;
pub mod source;

use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::{Component, Path, PathBuf};

use automerge::AutomergeError;
use noyau::donnees::acces::Acces;

pub use adaptateurs::{FondsSurFichiers, refus_a_l_ouverture};
pub use captations::Captations;
pub use documents::Document;
#[cfg(feature = "pannes")]
pub use documents::Naissance;
pub use noyau::donnees::identifiants::empreinte;
pub use racine_des_donnees::RacineDesDonnees;
pub use services::lecture::teneur;

/// Ce que le stockage refuse, ou ce que le disque lui refuse. Son texte (`Display`) le dit en
/// français, par un constat, jamais par un ordre (§ 50 · 5) ; chaque chemin s'y écrit par ses
/// composants joints par `/`, et le texte d'une erreur du système ou d'Automerge n'y paraît pas.
#[derive(Debug)]
pub enum Erreur {
    /// `TryLockError::WouldBlock` : un autre processus tient la racine des données.
    RacineDesDonneesTenue,
    /// Le disque a refusé un geste sur `chemin` (lecture, écriture, création, renommage, retrait,
    /// ou le renommage rendu durable), ou, relu, il ne rend pas les octets écrits
    /// (`io::ErrorKind::InvalidData`). Le message nomme le chemin, sauf sous `captations/`, où il
    /// dit « Une captation » : un nom provisoire ou une captation rangée porterait un identifiant
    /// d'import ou une empreinte (question 26).
    Disque {
        chemin: PathBuf,
        erreur: std::io::Error,
    },
    /// Le contenu relu de la captation `empreinte` ne donne pas son empreinte. Le message ne la
    /// nomme pas (question 26).
    CaptationAlteree { empreinte: String },
    /// Aucune captation ne porte le nom `empreinte`. Le message ne la nomme pas (question 26).
    CaptationAbsente { empreinte: String },
    /// Le fichier à verser ne se lit plus : la lecture de son flux a rendu `erreur`, pendant une
    /// copie en flux, ou la source à verser sur fichiers ne le lit pas. Ce n'est pas un refus du
    /// disque de la racine des données.
    FichierAVerser { erreur: std::io::Error },
    /// Un `Document` du fonds vit déjà dans ce processus : un seul à la fois (AM-4 (1)).
    FondsDejaOuvert,
    /// Le fonds est déjà né : il ne naît qu'une fois (§ 31 · 2.3).
    FondsDejaNe,
    /// Automerge a refusé : le geste, une écriture du document, ou la lecture de sa racine.
    Automerge(AutomergeError),
    /// L'opération ne s'applique pas au document (`services::application::appliquer`) : un
    /// objet manque sur son chemin, elle ne change rien, ou Automerge l'a refusée ; rien ne
    /// s'écrit.
    Inapplicable(services::application::Inapplicable),
    /// Le document est en lecture seule, pour cette raison : il refuse ce que son accès ne
    /// permet pas (§ 31 · 2.1).
    LectureSeule(LectureSeule),
    /// Le fichier complet de la compaction, relu, ne mène pas aux heads du document en mémoire :
    /// aucun fichier d'ajout n'est retiré (§ 31 · 2.2). Relu avant d'être écrit, il ne se lit pas
    /// ou mène ailleurs, et il ne s'écrit pas. Seul [`Document::compacter`], appelé seul, la
    /// rend : après une écriture ou une réception, elle vient en avertissement
    /// ([`Avertissement::Compaction`]), ce qui est écrit ou reçu écrit ; à l'ouverture aussi, et
    /// le document s'ouvre.
    CompactionInfidele,
    /// La racine du document ne porte pas de chaîne sous `champ` : la copie ne se situe pas
    /// (§ 31 · 2.5).
    ChampAbsent { champ: &'static str },
    /// L'échange a refusé le message reçu. Ce que le message a pu apporter avant le refus est
    /// écrit ; l'avertissement de cette écriture, s'il en vient un, rejoint ceux du document
    /// (`Document::avertissements`).
    Synchro(synchro::Erreur),
}

/// Pourquoi un document est en lecture seule (§ 31 · 2.1, § 31 · 2.4) : il montre ce qui se lit,
/// n'écrit d'opération ni ne se compacte ; sauf devant une version de schéma plus récente, il
/// n'envoie ni ne reçoit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LectureSeule {
    /// La relecture du disque, après une écriture refusée, a échoué sur `chemin` : le document
    /// montre sa mémoire d'avant l'écriture refusée.
    RelectureManquee { chemin: PathBuf },
    /// Le contenu du fichier d'ajout `chemin` ne donne pas l'empreinte que porte son nom : il
    /// n'est pas chargé, et rien n'est retiré ni réécrit.
    AjoutAltere { chemin: PathBuf },
    /// Un fichier d'ajout dont le contenu donne l'empreinte de son nom, mais que
    /// `load_incremental` refuse (par exemple `DuplicateSeqNumber`) ; il n'est pas chargé.
    AjoutIllisible { chemin: PathBuf },
    /// Le fichier complet `chemin` est vide, ou `Automerge::load` le refuse : le document montre
    /// ce qui s'en lit (`OnPartialLoad::Ignore`), ou rien s'il ne s'en lit rien, puis les
    /// fichiers d'ajout. Absent, sans voisin lisible qui le remplace, il ne laisse rien non plus,
    /// à la relecture du disque : un document sans genèse n'est jamais entier. `chemin` est aussi
    /// un voisin lisible du fichier complet que `load_incremental` refuse après lui.
    CompletIllisible { chemin: PathBuf },
    /// Tout lu, une modification attend ses dépendances (`get_missing_deps`) : un fichier
    /// d'ajout manque. `chemin` est le premier fichier d'ajout, dans l'ordre des noms, qui attend
    /// une modification qui manque au document lu, ni chargée ni en attente, ou, à défaut, le
    /// fichier complet : le fichier qui manque ne se nomme pas, celui qui l'attend, si.
    DependancesManquantes { chemin: PathBuf },
    /// Le document, qui se lit entier, porte une version de schéma plus récente que celle que
    /// connaît l'application (§ 31 · 2.4), dès son ouverture ou depuis qu'il l'a reçue : il se
    /// lit sans écrire, reçoit, et n'envoie que des messages sans aucune modification ; ce qu'il
    /// reçoit s'écrit en fichiers d'ajout, sans compaction. Il ne redevient entier qu'à sa
    /// réouverture, même s'il reçoit la version de schéma connue. `version` : la version de
    /// schéma qu'il porte, plus récente que celle que connaît l'application, relue à chaque
    /// réception qui change ses heads, tant qu'elle reste plus récente.
    VersionPlusRecente { version: u64 },
    /// Le document, qui se lit entier, porte une version de schéma absente, nulle, ou qui n'est
    /// pas un entier non signé (§ 31 · 2.4) : il se lit, sans écrire ni échanger.
    VersionIllisible,
    /// Le document, qui se lit entier, porte une version de schéma plus ancienne que celle que
    /// connaît l'application, et une de ses migrations ne s'applique pas, à l'ouverture
    /// (`services::application::migrer` la refuse : elle ne se lit pas, ou attend ses
    /// dépendances, comme une migration bâtie sur une autre genèse) : il n'atteint pas la version
    /// de schéma connue, et se lit sans écrire ni échanger. Rien ne s'écrit, ni la migration, ni
    /// une opération à une version de schéma plus ancienne. L'erreur d'Automerge, qui n'est ni
    /// `Clone` ni `Eq`, ne se garde pas.
    MigrationRefusee,
}

impl LectureSeule {
    /// Ce que permet un document en lecture seule pour cette raison : une version de schéma plus
    /// récente le laisse échanger (`PlusRecent`) ; un fichier illisible, une relecture manquée,
    /// une version de schéma illisible ou une migration refusée le laissent illisible, sans
    /// écriture ni échange.
    pub fn acces(&self) -> Acces {
        match self {
            LectureSeule::VersionPlusRecente { .. } => Acces::PlusRecent,
            LectureSeule::RelectureManquee { .. }
            | LectureSeule::AjoutAltere { .. }
            | LectureSeule::AjoutIllisible { .. }
            | LectureSeule::CompletIllisible { .. }
            | LectureSeule::DependancesManquantes { .. }
            | LectureSeule::VersionIllisible
            | LectureSeule::MigrationRefusee => Acces::Illisible,
        }
    }
}

/// Ce qui a échoué après qu'une écriture est faite, sans la défaire (§ 31 · 2.1) ; ou à
/// l'ouverture, ou à la naissance, sans l'empêcher.
#[derive(Debug)]
pub enum Avertissement {
    /// Le fichier `chemin` est en place, mais son renommage n'a pas été rendu durable.
    NonDurable { chemin: PathBuf, erreur: io::Error },
    /// La compaction a échoué (`Disque` ou `CompactionInfidele`) : rien n'est perdu, et elle se
    /// retente à l'écriture suivante, qui retire ce qu'elle n'a pas retiré. Coupée en chemin,
    /// elle a pu remplacer le fichier complet sans le rendre durable, ou retirer une part des
    /// fichiers d'ajout qu'il absorbe.
    Compaction(Erreur),
    /// Le voisin `chemin` d'un fichier d'ajout n'a pu prendre sa place : il se lit où il est, et se
    /// met en place à l'ouverture suivante.
    VoisinRestant { chemin: PathBuf, erreur: io::Error },
    /// Le nom provisoire `chemin` n'a pu se lire, ou prendre le nom de son empreinte : il reste, et
    /// l'ouverture suivante le relit, puis le range si le fonds le nomme (§ 31 · 6.1). Un dossier
    /// `captations/` qui ne se liste pas se nomme de même, et rien ne s'y range.
    NomProvisoireRestant { chemin: PathBuf, erreur: io::Error },
}

/// Le refus du disque sur `chemin`.
pub(crate) fn disque(chemin: &Path, erreur: io::Error) -> Erreur {
    Erreur::Disque {
        chemin: chemin.to_path_buf(),
        erreur,
    }
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ecrire(f, None)
    }
}

impl fmt::Display for LectureSeule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ecrire(f, None)
    }
}

impl fmt::Display for Avertissement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ecrire(f, None)
    }
}

/// `Relatif(valeur, racine)` : le texte de `valeur`, une erreur ou un avertissement du stockage,
/// chaque chemin qu'elle porte dit relatif à `racine`, le chemin de la racine des données, quand
/// il commence par lui (`Path::strip_prefix`). L'adaptateur sur fichiers, qui connaît la racine
/// des données, en forme le message d'un refus ou d'un avertissement : le fichier s'y nomme
/// `documents/fonds/ajouts/<empreinte>.automerge`, sans le chemin de la machine.
pub(crate) struct Relatif<'a, T>(pub(crate) &'a T, pub(crate) &'a Path);

impl fmt::Display for Relatif<'_, Erreur> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.ecrire(f, Some(self.1))
    }
}

impl fmt::Display for Relatif<'_, Avertissement> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.ecrire(f, Some(self.1))
    }
}

impl Erreur {
    /// Le texte de l'erreur, chaque chemin dit relatif à `racine`, s'il en est une.
    fn ecrire(&self, f: &mut fmt::Formatter<'_>, racine: Option<&Path>) -> fmt::Result {
        match self {
            Erreur::RacineDesDonneesTenue => {
                f.write_str("Un autre processus tient la racine des données.")
            }
            Erreur::Disque { chemin, erreur } => {
                let constat = if sous_les_captations(chemin, racine) {
                    constat_du_disque("Une captation", Accord::Feminin, erreur.kind())
                } else {
                    constat_du_disque(&chemin_dit(chemin, racine), Accord::Masculin, erreur.kind())
                };
                write!(f, "{constat}.")
            }
            Erreur::CaptationAlteree { .. } => {
                f.write_str("Une captation relue ne donne pas son empreinte.")
            }
            Erreur::CaptationAbsente { .. } => {
                f.write_str("Cette captation n'est pas dans le fonds.")
            }
            Erreur::FichierAVerser { erreur } => write!(
                f,
                "{}.",
                constat_du_disque("Le fichier à verser", Accord::Masculin, erreur.kind())
            ),
            Erreur::FondsDejaOuvert => f.write_str("Le fonds est déjà ouvert dans ce processus."),
            Erreur::FondsDejaNe => f.write_str("Le fonds est déjà né : il ne naît qu'une fois."),
            Erreur::Automerge(_) => {
                f.write_str("Le document a refusé ce geste, ou sa racine ne se lit pas.")
            }
            Erreur::LectureSeule(lecture_seule) => lecture_seule.ecrire(f, racine),
            Erreur::CompactionInfidele => f.write_str(
                "Le fichier complet de la compaction, relu, ne mène pas au document en mémoire : \
                 aucun fichier d'ajout n'est retiré.",
            ),
            Erreur::ChampAbsent { champ } => write!(
                f,
                "La racine du document ne porte pas de chaîne sous « {champ} » : la copie ne se \
                 situe pas."
            ),
            Erreur::Synchro(erreur) => write!(f, "{erreur}"),
            Erreur::Inapplicable(inapplicable) => write!(f, "{inapplicable}"),
        }
    }
}

impl LectureSeule {
    /// Le texte de la raison, chaque chemin dit relatif à `racine`, s'il en est une.
    fn ecrire(&self, f: &mut fmt::Formatter<'_>, racine: Option<&Path>) -> fmt::Result {
        match self {
            LectureSeule::RelectureManquee { chemin } => write!(
                f,
                "La relecture de {}, après une écriture refusée, a échoué : le document montre \
                 ce qu'il était avant elle, et se lit sans écrire ni échanger.",
                chemin_dit(chemin, racine)
            ),
            LectureSeule::AjoutAltere { chemin } => write!(
                f,
                "Le contenu de {} ne donne pas l'empreinte que porte son nom : il n'est pas \
                 chargé, et le document se lit sans écrire ni échanger.",
                chemin_dit(chemin, racine)
            ),
            LectureSeule::AjoutIllisible { chemin } => write!(
                f,
                "{} donne l'empreinte de son nom, mais ne se charge pas : le document se lit \
                 sans lui, sans écrire ni échanger.",
                chemin_dit(chemin, racine)
            ),
            LectureSeule::CompletIllisible { chemin } => write!(
                f,
                "{} ne se lit pas en entier : le document montre ce qui s'en lit, sans écrire ni \
                 échanger.",
                chemin_dit(chemin, racine)
            ),
            LectureSeule::DependancesManquantes { chemin } => write!(
                f,
                "{} attend une modification qu'aucun fichier lu ne porte : un fichier d'ajout \
                 manque, et le document se lit sans écrire ni échanger.",
                chemin_dit(chemin, racine)
            ),
            LectureSeule::VersionPlusRecente { .. } => f.write_str(
                "Ce fonds vient d'une version plus récente de Pupitre ; cette version-ci le lit \
                 sans y écrire.",
            ),
            LectureSeule::VersionIllisible => f.write_str(
                "La version de schéma du document ne se lit pas : il se lit sans écrire ni \
                 échanger.",
            ),
            LectureSeule::MigrationRefusee => f.write_str(
                "Une migration de ce fonds ne s'applique pas : il se lit sans écrire ni échanger.",
            ),
        }
    }
}

impl Avertissement {
    /// Le texte de l'avertissement, chaque chemin dit relatif à `racine`, s'il en est une.
    fn ecrire(&self, f: &mut fmt::Formatter<'_>, racine: Option<&Path>) -> fmt::Result {
        match self {
            Avertissement::NonDurable { chemin, .. } => write!(
                f,
                "{} est en place, mais son renommage n'est pas rendu durable.",
                chemin_dit(chemin, racine)
            ),
            Avertissement::Compaction(erreur) => {
                f.write_str(
                    "La compaction a échoué, sans rien perdre, et se retente à l'écriture \
                     suivante : ",
                )?;
                erreur.ecrire(f, racine)
            }
            Avertissement::VoisinRestant { chemin, .. } => write!(
                f,
                "Le voisin {} n'a pas pris la place de son fichier : il se lit où il est, et la \
                 prend à l'ouverture suivante.",
                chemin_dit(chemin, racine)
            ),
            Avertissement::NomProvisoireRestant { chemin, .. } => write!(
                f,
                "{} n'a pas pu se lire, ou prendre le nom de son empreinte : il reste, et \
                 l'ouverture suivante le relit.",
                chemin_dit(chemin, racine)
            ),
        }
    }
}

/// L'accord du constat du disque avec son sujet : un chemin, « Le fichier écrit à côté » ou « Le
/// fichier à verser », au masculin ; « Une captation », au féminin.
#[derive(Clone, Copy)]
pub(crate) enum Accord {
    Masculin,
    Feminin,
}

/// Le constat du disque sur `sujet`, un chemin, « Le fichier écrit à côté », « Le fichier à
/// verser » ou « Une captation », accordé selon `accord`, selon le genre de son erreur ; le texte
/// de l'erreur, celui du système, n'est jamais recopié. Le constat d'un autre genre est neutre
/// quant au geste : le disque a pu refuser une lecture, une écriture, une création, un renommage,
/// un retrait, ou le renommage rendu durable.
pub(crate) fn constat_du_disque(sujet: &str, accord: Accord, genre: io::ErrorKind) -> String {
    let (absent, refuse, tel_qu_ecrit) = match accord {
        Accord::Masculin => ("absent", "refusé", "tel qu'il a été écrit"),
        Accord::Feminin => ("absente", "refusée", "telle qu'elle a été écrite"),
    };
    match genre {
        io::ErrorKind::NotFound => format!("{sujet} est {absent}"),
        io::ErrorKind::PermissionDenied => format!("{sujet} est {refuse} par le système"),
        io::ErrorKind::StorageFull => format!("{sujet} ne tient plus sur le disque"),
        io::ErrorKind::InvalidData => format!("{sujet} ne se relit pas {tel_qu_ecrit}"),
        _ => format!("{sujet}, le disque l'a {refuse}"),
    }
}

/// `chemin` est-il sous `captations/` de la racine des données, sans être `captations/` même : un
/// nom provisoire, un sous-dossier ou une captation rangée, dont le nom porterait un identifiant
/// d'import ou une empreinte, qu'un message ne nomme jamais (question 26) ? Le chemin se lit
/// relatif à `racine`, s'il en est une et qu'il commence par elle (`Path::strip_prefix`), sinon
/// tel quel : son premier composant est `captations`, et un autre le suit. Un chemin absolu que
/// `racine` ne préfixe pas commence par sa racine, jamais par `captations`.
fn sous_les_captations(chemin: &Path, racine: Option<&Path>) -> bool {
    let relatif = racine
        .and_then(|racine| chemin.strip_prefix(racine).ok())
        .unwrap_or(chemin);
    let mut composants = relatif.components();
    composants.next() == Some(Component::Normal(OsStr::new("captations")))
        && composants.next().is_some()
}

/// `chemin`, relatif à `racine` s'il en est une et qu'il commence par elle, ses composants joints
/// par `/` : le même texte sous Linux et sous Windows. La racine du système de fichiers d'un
/// chemin absolu s'écrit `/`.
fn chemin_dit(chemin: &Path, racine: Option<&Path>) -> String {
    let chemin = racine
        .and_then(|racine| chemin.strip_prefix(racine).ok())
        .unwrap_or(chemin);
    let mut texte = String::new();
    for composant in chemin.components() {
        match composant {
            Component::RootDir => texte.push('/'),
            autre => {
                if !texte.is_empty() && !texte.ends_with('/') {
                    texte.push('/');
                }
                texte.push_str(&autre.as_os_str().to_string_lossy());
            }
        }
    }
    texte
}
