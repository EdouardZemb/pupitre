//! Le stockage de pupitre, côté natif seulement (§ 40 · 3) : la racine des données et son
//! verrou, puis les adaptateurs `Documents` et `Captations` sur fichiers, au format de
//! l'ADR-023 (§ 31 · 2.1), qui réalisent les contrats des services (`FondsSurFichiers` et
//! `Captations`). Il ne devine aucun chemin : il reçoit celui de la racine des données (I30).
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

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use automerge::{Automerge, AutomergeError, ObjId, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use noyau::donnees::acces::Acces;
use noyau::donnees::teneur::{ErreurDeTeneur, Valeur, ecrire_la_teneur};

pub use adaptateurs::FondsSurFichiers;
pub use captations::Captations;
pub use documents::Document;
#[cfg(feature = "pannes")]
pub use documents::Naissance;
pub use noyau::donnees::identifiants::empreinte;
pub use racine_des_donnees::RacineDesDonnees;

/// Ce que le stockage refuse, ou ce que le disque lui refuse. Le texte en français de chaque
/// cas vient avec les commandes, au lot 1.
#[derive(Debug)]
pub enum Erreur {
    /// `TryLockError::WouldBlock` : un autre processus tient la racine des données.
    RacineDesDonneesTenue,
    /// Le disque a refusé un geste sur `chemin` (lecture, écriture, création, renommage, retrait,
    /// ou le renommage rendu durable), ou, relu, il ne rend pas les octets écrits
    /// (`io::ErrorKind::InvalidData`).
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

/// La teneur de `doc` (§ 31 · 2.6) : son arbre, lu sur la valeur que présente Automerge
/// (`ReadDoc`, jamais `get_all`), puis écrit par le noyau, sans la lignée ni les lignées qu'elle
/// remplace.
pub fn teneur(doc: &Automerge) -> Result<String, ErreurDeTeneur> {
    ecrire_la_teneur(&objet_a_cles(doc, &ROOT))
}

/// Un objet `Map` ou `Table` : une seule sorte.
fn objet_a_cles(doc: &Automerge, objet: &ObjId) -> BTreeMap<String, Valeur> {
    doc.keys(objet)
        .map(|cle| {
            let (valeur, id) = doc
                .get(objet, cle.as_str())
                .expect("lire une clé que le document vient de nommer")
                .expect("une clé nommée a une valeur");
            let valeur = valeur_de(doc, valeur, &id);
            (cle, valeur)
        })
        .collect()
}

fn valeur_de(doc: &Automerge, valeur: Value<'_>, id: &ObjId) -> Valeur {
    match valeur {
        // En automerge 0.12.0, une `Table` n'est jamais indexée : `put` y rend `NotAnObject`, et
        // `keys` avale cette erreur. Une `Table` non vide, chargée d'un document d'une autre
        // implémentation, se lit donc vide, sans erreur.
        Value::Object(ObjType::Map | ObjType::Table) => Valeur::ObjetACles(objet_a_cles(doc, id)),
        Value::Object(ObjType::List) => Valeur::Liste(
            (0..doc.length(id))
                .map(|indice| {
                    let (valeur, id) = doc
                        .get(id, indice)
                        .expect("lire un indice que le document vient de compter")
                        .expect("un indice compté a une valeur");
                    valeur_de(doc, valeur, &id)
                })
                .collect(),
        ),
        Value::Object(ObjType::Text) => {
            Valeur::Texte(doc.text(id).expect("lire un objet texte du document"))
        }
        Value::Scalar(scalaire) => match scalaire.as_ref() {
            ScalarValue::Str(texte) => Valeur::Str(texte.to_string()),
            ScalarValue::Int(n) => Valeur::Int(*n),
            ScalarValue::Uint(n) => Valeur::Uint(*n),
            ScalarValue::F64(x) => Valeur::F64(*x),
            ScalarValue::Counter(compteur) => Valeur::Counter(i64::from(compteur)),
            ScalarValue::Timestamp(n) => Valeur::Timestamp(*n),
            ScalarValue::Boolean(b) => Valeur::Boolean(*b),
            ScalarValue::Bytes(octets) => Valeur::Bytes(octets.clone()),
            ScalarValue::Null => Valeur::Null,
            ScalarValue::Unknown { type_code, .. } => Valeur::Inconnu {
                type_code: *type_code,
            },
        },
    }
}

/// Le refus du disque sur `chemin`.
pub(crate) fn disque(chemin: &Path, erreur: io::Error) -> Erreur {
    Erreur::Disque {
        chemin: chemin.to_path_buf(),
        erreur,
    }
}
