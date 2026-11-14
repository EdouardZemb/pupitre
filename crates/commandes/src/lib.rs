//! Les types des commandes de pupitre (§ 40 · 3, « La frontière des commandes ») : leurs
//! arguments, leurs réponses et leurs vues, et la liste de leurs noms, que l'interface et
//! l'établi partagent. La crate ne porte que ce que l'interface envoie et reçoit : l'heure,
//! l'aléa et l'appareil d'un geste viennent du côté natif, jamais d'une commande, et le côté
//! natif ne reçoit de l'interface, pour dater un message du journal, que son décalage local,
//! `decalage_minutes`. Elle ne dépend que de `serde`.
//!
//! Aucun type n'a de flottant. Une commande et sa réponse passent par le JSON, que la WebView
//! relit par `JSON.parse` : ses nombres sont des flottants de 64 bits, exacts sur les entiers
//! jusqu'à `ENTIER_SUR`, 2⁵³ − 1. Chaque entier d'une réponse vient d'une source qui le tient
//! sous cette borne, et la doc de son champ la nomme.

use serde::{Deserialize, Serialize};

/// Les noms des commandes que cette crate type : les opérations nommées et les requêtes du fonds.
/// `exporter_cahier`, la commande de l'établi qui exporte le cahier des tables depuis INC-0, les
/// rejoindra.
pub const NOMS: &[&str] = &[
    "noter",
    "completer",
    "poser_une_etiquette",
    "oter_une_etiquette",
    "relier",
    "delier",
    "regrouper",
    "retirer",
    "retablir",
    "annuler",
    "vue_d_une_idee",
    "idees",
    "journal",
];

/// Le plus grand entier que `JSON.parse` relit à l'identique : 2⁵³ − 1. Le noyau, qui ne voit pas
/// cette crate, le répète (`noyau::donnees::gestes::ENTIER_SUR`) : un test de chacune le compare
/// à `(1 << 53) - 1`.
pub const ENTIER_SUR: u64 = 9_007_199_254_740_991;

/// Noter une idée.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Noter {
    /// Le texte de la note.
    pub texte: String,
    /// Le décalage local de l'interface, en minutes, qui date le message du journal ; un `i32`,
    /// borné par son type.
    pub decalage_minutes: i32,
}

/// Compléter le texte de l'idée `idee`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Completer {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// Son texte complété.
    pub texte: String,
    /// Le décalage local de l'interface, en minutes, qui date le message du journal ; un `i32`,
    /// borné par son type.
    pub decalage_minutes: i32,
}

/// Poser ou ôter l'étiquette `etiquette` sur l'idée `idee`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Etiqueter {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// L'étiquette.
    pub etiquette: String,
    /// Le décalage local de l'interface, en minutes, qui date le message du journal ; un `i32`,
    /// borné par son type.
    pub decalage_minutes: i32,
}

/// Relier ou délier les idées `a` et `b`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relier {
    /// L'identifiant d'un bout, un ULID.
    pub a: String,
    /// L'identifiant de l'autre bout, un ULID.
    pub b: String,
    /// Le décalage local de l'interface, en minutes, qui date le message du journal ; un `i32`,
    /// borné par son type.
    pub decalage_minutes: i32,
}

/// La sorte d'une captation versée (§ 30 · 9.2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sorte {
    /// Une prise.
    Prise,
    /// Un MIDI versé seul, qui n'accompagne aucun stem.
    MidiVerseSeul,
    /// Un texte versé.
    TexteVerse,
}

/// Une captation versée, nommée par sa sorte et son empreinte.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Captation {
    /// Sa sorte.
    pub sorte: Sorte,
    /// Son empreinte : le SHA-256 de ses octets, en 64 caractères hexadécimaux minuscules.
    pub empreinte: String,
}

/// Regrouper la captation `captation` dans l'idée `cible`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regrouper {
    /// La captation.
    pub captation: Captation,
    /// L'identifiant de l'idée qui la reçoit, un ULID.
    pub cible: String,
    /// Le décalage local de l'interface, en minutes, qui date le message du journal ; un `i32`,
    /// borné par son type.
    pub decalage_minutes: i32,
}

/// Retirer ou rétablir l'idée `idee`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viser {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// Le décalage local de l'interface, en minutes, qui date le message du journal ; un `i32`,
    /// borné par son type.
    pub decalage_minutes: i32,
}

/// Le rôle d'un appareil (§ 30 · 8.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appareil {
    /// L'établi.
    Etabli,
    /// Le carnet.
    Carnet,
}

/// La provenance d'une idée (§ 30 · 8.1), posée à sa naissance et gardée à vie.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    /// Née d'une capture, sur l'appareil `par`.
    Captee { par: Appareil },
    /// Née d'un fichier versé, sur l'appareil `par`, par l'import `depuis`, un ULID.
    Importee { par: Appareil, depuis: String },
}

/// La réponse à `noter` : l'idée née.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notee {
    /// L'identifiant de l'idée née, un ULID.
    pub idee: String,
    /// Un avertissement, lisible : l'opération est faite, mais Sacha doit en savoir quelque chose.
    pub avertissement: Option<String>,
}

/// La réponse à une opération nommée faite, autre que `noter`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faite {
    /// Un avertissement, lisible : l'opération est faite, mais Sacha doit en savoir quelque chose.
    pub avertissement: Option<String>,
}

/// Pourquoi une commande est refusée : un constat, jamais un ordre (§ 50 · 5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Raison {
    /// La commande ne se fait pas sur le fonds tel qu'il est.
    Impossible,
    /// Aucun geste à annuler.
    RienAAnnuler,
    /// Le fonds, ou ce qu'en lit l'application, ne se lit pas.
    Illisible,
    /// Le fonds porte une version de schéma plus récente que celle que connaît l'application.
    /// `version` : la version de schéma du document, que seules les migrations écrivent, chacune
    /// d'un de plus (`noyau::donnees::version::connue` : 1 + le nombre de migrations), et que
    /// borne donc le nombre de migrations que l'application publiera.
    VersionPlusRecente { version: u64 },
    /// Le disque a refusé une lecture ou une écriture, ou rend des octets altérés.
    Disque,
}

/// Un refus : sa raison, et le message à montrer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refus {
    /// Sa raison.
    pub raison: Raison,
    /// Le message, lisible.
    pub message: String,
}

/// La réponse à une commande : sa valeur, ou un refus.
pub type Reponse<T> = Result<T, Refus>;

/// La vue d'une idée, telle que l'interface la reçoit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VueDUneIdee {
    /// Son identifiant, un ULID.
    pub idee: String,
    /// Sa provenance.
    pub provenance: Provenance,
    /// La date de sa naissance, en secondes depuis 1970 : l'heure d'une opération
    /// (`noyau::donnees::changement::Operation::heure`), que seuls donnent le contexte du côté
    /// natif (son horloge), la genèse (0) ou une migration (son heure, écrite d'avance) ; une
    /// réplique reçue n'en porte pas d'autres, puisque les mêmes opérations les ont écrites.
    /// 2⁵³ secondes sont à quelque 285 millions d'années de 1970.
    pub le: i64,
    /// Son texte, s'il y en a un.
    pub texte: Option<String>,
    /// Ses étiquettes, dans l'ordre de leurs octets.
    pub etiquettes: Vec<String>,
    /// L'autre bout de chaque lien, dans l'ordre de leurs octets.
    pub liens: Vec<String>,
    /// Ses captations, dans l'ordre (sorte, empreinte).
    pub captations: Vec<Captation>,
    /// Reliée : elle porte au moins un lien vers ce qui n'est pas une idée retirée (CAP-02,
    /// Règles).
    pub reliee: bool,
    /// La durée de l'idée, en millisecondes : celle de sa prise la plus longue, et `None` dès
    /// qu'une de ses prises n'a pas la sienne (CAP-02, Règles ; `noyau::donnees::idee::duree`).
    /// Seul `noyau::donnees::gestes::verser` écrit la durée d'une prise, sous `<empreinte>/duree`,
    /// et il la refuse au-delà d'`ENTIER_SUR`.
    pub duree_ms: Option<u64>,
    /// La date de son retrait, en secondes depuis 1970, si elle est retirée : l'heure d'une
    /// opération (`Operation::heure`), bornée comme `le`.
    pub retiree: Option<i64>,
}

/// Une entrée du journal : le message d'une opération, et son heure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntreeDuJournal {
    /// Le message de l'opération, lisible.
    pub message: String,
    /// L'heure de l'opération, en secondes depuis 1970 (`Operation::heure`), bornée comme
    /// `VueDUneIdee::le`.
    pub heure: i64,
}
