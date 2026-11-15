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

// Chaque nom de commande, une constante ; `NOMS` les liste toutes, dans cet ordre.

/// Noter une idée (`Noter`).
pub const NOTER: &str = "noter";
/// Compléter le texte d'une idée (`Completer`).
pub const COMPLETER: &str = "completer";
/// Poser une étiquette sur une idée (`Etiqueter`).
pub const POSER_UNE_ETIQUETTE: &str = "poser_une_etiquette";
/// Ôter une étiquette d'une idée (`Etiqueter`).
pub const OTER_UNE_ETIQUETTE: &str = "oter_une_etiquette";
/// Relier deux idées (`Relier`).
pub const RELIER: &str = "relier";
/// Délier deux idées (`Relier`).
pub const DELIER: &str = "delier";
/// Regrouper une captation dans une idée (`Regrouper`).
pub const REGROUPER: &str = "regrouper";
/// Retirer une idée (`Viser`).
pub const RETIRER: &str = "retirer";
/// Rétablir une idée retirée (`Viser`).
pub const RETABLIR: &str = "retablir";
/// Annuler le dernier geste de tri.
pub const ANNULER: &str = "annuler";
/// Lire la vue d'une idée (`VueDUneIdee`).
pub const VUE_D_UNE_IDEE: &str = "vue_d_une_idee";
/// Lire les idées non retirées.
pub const IDEES: &str = "idees";
/// Lire le journal du fonds (`EntreeDuJournal`).
pub const JOURNAL: &str = "journal";
/// Lire la boîte (`Boite`, `VueDeLaBoite`).
pub const BOITE: &str = "boite";
/// Lire les idées retirées (`Fenetre`, `VueDesRetirees`).
pub const RETIREES: &str = "retirees";
/// Lire l'état du fonds (`EtatDuFonds`).
pub const ETAT_DU_FONDS: &str = "etat_du_fonds";
/// Exporter le cahier des tables, la commande de l'établi depuis INC-0.
pub const EXPORTER_CAHIER: &str = "exporter_cahier";

/// Les noms de toutes les commandes de l'application : les opérations nommées et les lectures du
/// fonds, que cette crate type, et `exporter_cahier`, la commande de l'établi qui exporte le cahier
/// des tables depuis INC-0. La liste est complète : `build.rs` en tire le manifeste de l'application,
/// et chaque fichier de droits la recopie.
pub const NOMS: &[&str] = &[
    NOTER,
    COMPLETER,
    POSER_UNE_ETIQUETTE,
    OTER_UNE_ETIQUETTE,
    RELIER,
    DELIER,
    REGROUPER,
    RETIRER,
    RETABLIR,
    ANNULER,
    VUE_D_UNE_IDEE,
    IDEES,
    JOURNAL,
    BOITE,
    RETIREES,
    ETAT_DU_FONDS,
    EXPORTER_CAHIER,
];

/// Le plus grand nombre de lignes qu'une fenêtre rend (ADR-022 : des vues petites).
pub const FENETRE_MAX: u32 = 200;

/// Le plus grand nombre de caractères du texte d'une ligne ; la vue de l'idée le porte entier.
pub const LONGUEUR_D_UNE_LIGNE: usize = 200;

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
    /// Le fonds ne se lit pas en entier, ou ne se lit que sans écrire : un fichier altéré ou
    /// refusé (un fichier d'ajout altéré, par exemple), une version de schéma illisible, une
    /// migration qui ne s'applique pas, ou des octets qu'Automerge refuse. Ce qui s'en lit se
    /// montre, en lecture seule.
    Illisible,
    /// Le fonds porte une version de schéma plus récente que celle que connaît l'application.
    /// `version` : la version de schéma du document, que la genèse pose à 1 et que seules les
    /// migrations font monter, chacune d'un de plus (`noyau::donnees::version::connue` : 1 + le
    /// nombre de migrations), et que borne donc le nombre de migrations que l'application
    /// publiera.
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

/// L'ordre de la boîte : il n'écrit rien et ne s'annule pas ; le tri est un geste (question 17).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ordre {
    /// La date, la plus récente d'abord.
    Date,
    /// L'étiquette, sur la plus petite de l'idée.
    Etiquette,
    /// La durée, la plus longue d'abord.
    Duree,
    /// Les non reliées d'abord.
    Reliee,
    /// Les captées d'abord.
    Provenance,
}

/// Un contenu que le filtre de la boîte retient (CAP-02) : le texte de l'idée, une prise, une
/// photo, un MIDI versé, un texte versé (question 22).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Contenu {
    /// Le texte de l'idée.
    TexteDeLIdee,
    /// Une prise.
    Prise,
    /// Une photo.
    Photo,
    /// Un MIDI versé.
    MidiVerse,
    /// Un texte versé.
    TexteVerse,
}

/// Lire la boîte : l'ordre, le filtre, et la fenêtre demandée.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boite {
    /// L'ordre des lignes.
    pub ordre: Ordre,
    /// Le contenu que le filtre retient ; aucun, toutes les idées.
    pub filtre: Option<Contenu>,
    /// Le rang de la première ligne de la fenêtre, compté de 0.
    pub debut: u32,
    /// Le nombre de lignes demandées, que la réponse borne à `FENETRE_MAX`.
    pub nombre: u32,
}

/// Une fenêtre de lignes : son début et son nombre.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fenetre {
    /// Le rang de la première ligne de la fenêtre, compté de 0.
    pub debut: u32,
    /// Le nombre de lignes demandées, que la réponse borne à `FENETRE_MAX`.
    pub nombre: u32,
}

/// L'assiette d'un compte (I09) : considérées, portant, décidées ; considérées − portant, au lot 1,
/// ce sont les idées illisibles. Chaque champ est un compte d'idées : il ne dépasse pas le nombre
/// des idées du fonds, dont chacune est une clé du document, bien en deçà d'`ENTIER_SUR`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assiette {
    /// Les idées considérées, un compte d'idées : les idées non retirées, et les idées
    /// illisibles.
    pub considerees: u64,
    /// Celles qui portent la donnée, un compte d'idées.
    pub portant: u64,
    /// Celles dont la donnée est décidée, un compte d'idées ; au lot 1, toute donnée portée est
    /// décidée.
    pub decidees: u64,
}

/// Une ligne de la boîte, ou des idées retirées : ce que la ligne montre, sans ancienneté
/// (CAP-02 · 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneDeLaBoite {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// La date de sa naissance, en secondes depuis 1970, bornée comme `VueDUneIdee::le`.
    pub le: i64,
    /// Son texte, s'il y en a un, borné à `LONGUEUR_D_UNE_LIGNE` caractères.
    pub texte: Option<String>,
    /// Ses étiquettes, dans l'ordre de leurs octets.
    pub etiquettes: Vec<String>,
    /// Sa durée, en millisecondes, bornée comme `VueDUneIdee::duree_ms`.
    pub duree_ms: Option<u64>,
    /// Reliée, comme `VueDUneIdee::reliee`.
    pub reliee: bool,
    /// Sa provenance.
    pub provenance: Provenance,
    /// La date de son retrait, en secondes depuis 1970, si elle est retirée, bornée comme
    /// `VueDUneIdee::retiree`.
    pub retiree: Option<i64>,
}

/// La vue de la boîte : le compte des non reliées et son assiette, le nombre que retient le filtre
/// et son assiette, et les seules lignes de la fenêtre.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VueDeLaBoite {
    /// Les idées non retirées et non reliées, que le filtre ne change pas : un compte d'idées.
    pub non_reliees: u64,
    /// L'assiette du compte des non reliées.
    pub assiette_du_compte: Assiette,
    /// Les idées que retient le filtre, toutes sans filtre : un compte d'idées.
    pub retenues: u64,
    /// L'assiette du nombre que retient le filtre : au lot 1, celle du compte, la seule que
    /// calcule la boîte du noyau.
    pub assiette_du_filtre: Assiette,
    /// Les lignes de la fenêtre, `FENETRE_MAX` au plus.
    pub lignes: Vec<LigneDeLaBoite>,
}

/// La vue des idées retirées : leur nombre, que l'écran ne montre pas (il règle la hauteur de la
/// liste), et les lignes de la fenêtre.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VueDesRetirees {
    /// Les idées retirées : un compte d'idées.
    pub total: u64,
    /// Les lignes de la fenêtre, `FENETRE_MAX` au plus.
    pub lignes: Vec<LigneDeLaBoite>,
}

/// L'état du fonds à l'ouverture, et après chaque commande : né ou non, la lecture seule, les
/// avertissements de l'ouverture, et le message du geste qu'annuler vise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EtatDuFonds {
    /// Le fonds est né.
    pub ne: bool,
    /// Pourquoi le fonds n'écrit pas ; `None` s'il écrit.
    pub lecture_seule: Option<Refus>,
    /// Les avertissements de la naissance ou de l'ouverture du fonds, lisibles.
    pub avertissements: Vec<String>,
    /// Le message du dernier geste de tri, celui qu'annuler défait ; `None` sans geste à annuler.
    pub a_annuler: Option<String>,
}
