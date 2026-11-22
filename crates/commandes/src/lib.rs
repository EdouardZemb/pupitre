//! Les types des commandes de pupitre (§ 40 · 3, « La frontière des commandes ») : leurs
//! arguments, leurs réponses et leurs vues, et la liste de leurs noms, que l'interface et
//! l'établi partagent. La crate ne porte que ce que l'interface envoie et reçoit : l'heure,
//! l'aléa et l'appareil d'un geste viennent du côté natif, jamais d'une commande, et le côté
//! natif ne reçoit de l'interface, pour dater un message du journal, que des décalages locaux,
//! en minutes : celui de l'instant de chaque idée que le message date (`decalage_minutes`,
//! `decalage_b_minutes`, `decalage_source_minutes`), ou, pour noter, celui du geste. Elle ne
//! dépend que de `serde`.
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
/// Lire le journal du fonds (`Fenetre`, `VueDuJournal`).
pub const JOURNAL: &str = "journal";
/// Lire la boîte (`Boite`, `VueDeLaBoite`).
pub const BOITE: &str = "boite";
/// Lire les idées retirées (`Fenetre`, `VueDesRetirees`).
pub const RETIREES: &str = "retirees";
/// Lire l'état du fonds (`EtatDuFonds`).
pub const ETAT_DU_FONDS: &str = "etat_du_fonds";
/// Lire les requêtes enregistrées (`RequeteEnregistree`).
pub const REQUETES_ENREGISTREES: &str = "requetes_enregistrees";
/// Enregistrer une requête (`Enregistrer`).
pub const ENREGISTRER_UNE_REQUETE: &str = "enregistrer_une_requete";
/// Renommer une requête enregistrée (`Renommer`).
pub const RENOMMER_UNE_REQUETE: &str = "renommer_une_requete";
/// Supprimer une requête enregistrée (`Supprimer`).
pub const SUPPRIMER_UNE_REQUETE: &str = "supprimer_une_requete";
/// Inspecter un dossier à verser (`Inspection`).
pub const INSPECTER_UN_DOSSIER: &str = "inspecter_un_dossier";
/// Ouvrir l'import, ou le reprendre (`OuvrirLImport`, `ImportOuvert`).
pub const OUVRIR_L_IMPORT: &str = "ouvrir_l_import";
/// Verser un fichier de l'import (`VerserUnFichier`, `FichierVerse`).
pub const VERSER_UN_FICHIER: &str = "verser_un_fichier";
/// Verser une prise en stems (`VerserUnePriseEnStems`).
pub const VERSER_UNE_PRISE_EN_STEMS: &str = "verser_une_prise_en_stems";
/// Clore l'import et rendre son rapport (`CloreLImport`, `ImportClos`).
pub const CLORE_L_IMPORT: &str = "clore_l_import";
/// Déclarer l'origine d'une captation ou d'un dossier versé (`DeclarerUneOrigine`).
pub const DECLARER_UNE_ORIGINE: &str = "declarer_une_origine";
/// Choisir le type d'un stem, ou garder celui qui est proposé (`ChoisirLeType`).
pub const CHOISIR_LE_TYPE_D_UN_STEM: &str = "choisir_le_type_d_un_stem";
/// Lire un texte versé (`TexteLu`).
pub const LIRE_UN_TEXTE_VERSE: &str = "lire_un_texte_verse";
/// Lire le rapport d'un import (`LireUnRapport`, `RapportDImport`).
pub const LIRE_UN_RAPPORT: &str = "lire_un_rapport";
/// Lire les noms des générateurs déclarés du fonds.
pub const GENERATEURS: &str = "generateurs";
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
    REQUETES_ENREGISTREES,
    ENREGISTRER_UNE_REQUETE,
    RENOMMER_UNE_REQUETE,
    SUPPRIMER_UNE_REQUETE,
    INSPECTER_UN_DOSSIER,
    OUVRIR_L_IMPORT,
    VERSER_UN_FICHIER,
    VERSER_UNE_PRISE_EN_STEMS,
    CLORE_L_IMPORT,
    DECLARER_UNE_ORIGINE,
    CHOISIR_LE_TYPE_D_UN_STEM,
    LIRE_UN_TEXTE_VERSE,
    LIRE_UN_RAPPORT,
    GENERATEURS,
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
    /// Le décalage local du geste, en minutes (`natif::decalage_du_geste`), qui est celui de la
    /// naissance de l'idée notée : il date le message du journal ; un `i32`, borné par son type.
    pub decalage_minutes: i32,
}

/// Compléter le texte de l'idée `idee`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Completer {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// Son texte complété.
    pub texte: String,
    /// Le décalage local, en minutes, à l'instant de l'idée qu'il date, sa naissance
    /// (`natif::decalage_a(le)`) : il date le message du journal ; un `i32`, borné par son type.
    pub decalage_minutes: i32,
}

/// Poser ou ôter l'étiquette `etiquette` sur l'idée `idee`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Etiqueter {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// L'étiquette.
    pub etiquette: String,
    /// Le décalage local, en minutes, à l'instant de l'idée qu'il date, sa naissance
    /// (`natif::decalage_a(le)`) : il date le message du journal ; un `i32`, borné par son type.
    pub decalage_minutes: i32,
}

/// Relier ou délier les idées `a` et `b`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relier {
    /// L'identifiant d'un bout, un ULID.
    pub a: String,
    /// L'identifiant de l'autre bout, un ULID.
    pub b: String,
    /// Le décalage local, en minutes, à l'instant de l'idée `a` qu'il date, sa naissance
    /// (`natif::decalage_a(le)`) : il date `a` dans le message du journal ; un `i32`, borné par
    /// son type.
    pub decalage_minutes: i32,
    /// Le décalage local, en minutes, à l'instant de l'idée `b` qu'il date, sa naissance
    /// (`natif::decalage_a(le)`) : il date `b` dans le message du journal ; un `i32`, borné par
    /// son type.
    pub decalage_b_minutes: i32,
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
    /// Le décalage local, en minutes, à l'instant de la cible qu'il date, sa naissance
    /// (`natif::decalage_a(le)`) : il date la cible dans le message du journal ; un `i32`, borné
    /// par son type.
    pub decalage_minutes: i32,
    /// Le décalage local, en minutes, à l'instant de la source qu'il date, la naissance de
    /// l'idée que la captation quitte (`natif::decalage_a(le)`) : il la date dans le message du
    /// journal, si le geste la retire ; un `i32`, borné par son type.
    pub decalage_source_minutes: i32,
}

/// Retirer ou rétablir l'idée `idee`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viser {
    /// L'identifiant de l'idée, un ULID.
    pub idee: String,
    /// Le décalage local, en minutes, à l'instant de l'idée qu'il date, sa naissance
    /// (`natif::decalage_a(le)`) : il date le message du journal ; un `i32`, borné par son type.
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
    /// Ce que la vue montre de chaque captation, une par captation, dans l'ordre de `captations`.
    pub captations_vues: Vec<CaptationVue>,
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

/// La vue du journal : le nombre d'entrées, l'empreinte du journal, et les lignes de la fenêtre,
/// la plus récente d'abord.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VueDuJournal {
    /// Les entrées du journal, une par modification du document : un compte de modifications, que
    /// borne leur nombre, bien en deçà d'`ENTIER_SUR`.
    pub total: u64,
    /// L'empreinte du journal (`noyau::donnees::journal::empreinte_du_journal`), en 64 caractères
    /// hexadécimaux minuscules : la même sur deux répliques qui portent le même journal, quel que
    /// soit l'ordre de leur historique (question 29).
    pub empreinte: String,
    /// Les lignes de la fenêtre, la plus récente d'abord, `FENETRE_MAX` au plus.
    pub lignes: Vec<EntreeDuJournal>,
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

/// Un contenu qu'une requête cherche (CAP-02, CAP-06) : le texte de l'idée, une prise, une photo,
/// un MIDI versé, un texte versé (question 22).
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

/// Un jour civil (question 9), comme `noyau::donnees::requete::Jour`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jour {
    /// L'année.
    pub annee: i32,
    /// Le mois, de 1 à 12.
    pub mois: u8,
    /// Le jour du mois, de 1 à 31.
    pub jour: u8,
}

/// Les bornes d'une date, comme `noyau::donnees::requete::Bornes` : `des` comprise, `avant`
/// exclue ; une borne absente ne borne pas.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bornes {
    /// La première seconde comprise, en secondes depuis 1970 : le début d'un jour civil
    /// (`noyau::donnees::requete::debut_du_jour`) ; d'une année de quatre chiffres, comme en lit
    /// `noyau::donnees::requete::Jour::depuis_iso`, il est à moins de 2⁵³ secondes de 1970.
    pub des: Option<i64>,
    /// La première seconde exclue, en secondes depuis 1970, bornée comme `des`.
    pub avant: Option<i64>,
}

/// L'état d'une idée, pour une requête, comme `noyau::donnees::requete::Etat` : reliée ou non.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Etat {
    /// Reliée, comme `VueDUneIdee::reliee`.
    Reliee,
    /// Non reliée.
    NonReliee,
}

/// La provenance d'une idée, pour une requête (§ 30 · 8.1, question 11), comme
/// `noyau::donnees::requete::ProvenanceDeLIdee` : captée ou importée, quel que soit l'appareil.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvenanceDeLIdee {
    /// Née d'une capture.
    Captee,
    /// Née d'un fichier versé.
    Importee,
}

/// Une requête : les champs de `noyau::donnees::requete::Requete`, que la doc de chacun nomme.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requete {
    /// Le texte saisi, tel quel (`Requete::texte`) : ses mots sont les fragments cherchés, et
    /// sans mot, il ne pose aucun critère.
    pub texte: String,
    /// Le premier jour de la période, compris (`Requete::du`).
    pub du: Option<Jour>,
    /// Le dernier jour de la période, compris (`Requete::au`).
    pub au: Option<Jour>,
    /// L'étiquette cherchée, comparée octet pour octet (`Requete::etiquette`).
    pub etiquette: Option<String>,
    /// Le seuil de la durée affichée, en secondes entières, strictement dépassé
    /// (`Requete::plus_de_secondes`).
    pub plus_de_secondes: Option<u64>,
    /// L'état cherché (`Requete::etat`).
    pub etat: Option<Etat>,
    /// La provenance cherchée (`Requete::provenance`).
    pub provenance: Option<ProvenanceDeLIdee>,
    /// Le contenu cherché (`Requete::contenu`).
    pub contenu: Option<Contenu>,
}

/// Les parts « sans … » d'une réponse (question 7), comme `noyau::donnees::requete::Sans` : pour
/// chaque critère posé qu'une idée peut manquer, l'étiquette, la durée, le texte, le nombre
/// d'idées lisibles qui en manquent l'attribut et satisfont chacun des autres critères qu'elles
/// portent ; `None` pour un critère non posé. Chaque part est un compte d'idées, borné comme ceux
/// d'`Assiette`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sans {
    /// Les idées sans étiquette.
    pub etiquette: Option<u64>,
    /// Les idées sans durée.
    pub duree: Option<u64>,
    /// Les idées sans texte.
    pub texte: Option<u64>,
}

/// Une requête enregistrée : son ULID, son nom, et sa requête, `None` si elle vient d'une version
/// plus récente de Pupitre (question 17).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequeteEnregistree {
    /// Son identifiant, un ULID.
    pub id: String,
    /// Son nom, tel qu'il est enregistré.
    pub nom: String,
    /// Sa requête ; `None` si elle porte une sorte de critère ou une valeur que cette version de
    /// Pupitre ne connaît pas : elle paraît nommée, et ne se lance pas.
    pub requete: Option<Requete>,
}

/// Enregistrer la requête `requete` sous le nom `nom`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enregistrer {
    /// Son nom ; ses blancs autour ne s'enregistrent pas, et vide, il est refusé.
    pub nom: String,
    /// La requête, qui pose au moins un critère.
    pub requete: Requete,
}

/// Renommer `nom` la requête enregistrée `requete`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Renommer {
    /// L'identifiant de la requête enregistrée, un ULID.
    pub requete: String,
    /// Son nouveau nom ; ses blancs autour ne s'enregistrent pas, et vide, il est refusé.
    pub nom: String,
}

/// Supprimer la requête enregistrée `requete` (question 19).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supprimer {
    /// L'identifiant de la requête enregistrée, un ULID.
    pub requete: String,
}

/// Lire la boîte : l'ordre, la requête en cours, les bornes de sa date (que l'interface tire de
/// `du` et `au`), et la fenêtre demandée.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boite {
    /// L'ordre des lignes.
    pub ordre: Ordre,
    /// La requête en cours ; sans critère, toutes les idées.
    pub requete: Requete,
    /// Les bornes de sa date, que l'interface tire de ses jours `du` et `au`
    /// (`noyau::donnees::requete::bornes_de_la_periode`) : la réponse juge la date sur elles.
    pub bornes: Bornes,
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

/// L'assiette d'un compte (I09) : considérées, portant, décidées, et les illisibles, comptées à
/// part (question 6) ; considérées − portant compte les illisibles et, pour une requête, les idées
/// lisibles qui manquent un attribut qu'elle interroge. Chaque champ est un compte d'idées : il ne
/// dépasse pas le nombre des idées du fonds, dont chacune est une clé du document, bien en deçà
/// d'`ENTIER_SUR`.
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
    /// Les idées illisibles, un compte d'idées : elles comptent parmi les considérées, jamais
    /// parmi celles qui portent la donnée.
    pub illisibles: u64,
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
    /// Sans texte, le premier nom vu de sa première captation, s'il y en a un (question 4).
    pub nom_de_fichier: Option<String>,
}

/// La vue de la boîte, réponse de la requête en cours : le compte des non reliées et son assiette,
/// que la requête ne change pas ; le nombre des idées qu'elle trouve, son assiette et ses parts
/// « sans … » ; les étiquettes du fonds ; et les seules lignes de la fenêtre.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VueDeLaBoite {
    /// Les idées non retirées et non reliées, que la requête ne change pas : un compte d'idées.
    pub non_reliees: u64,
    /// L'assiette du compte des non reliées.
    pub assiette_du_compte: Assiette,
    /// Les idées que la requête trouve, toutes sans critère : un compte d'idées.
    pub retenues: u64,
    /// L'assiette de la requête : considérées, les idées non retirées et les illisibles ; portant
    /// et décidées, les idées lisibles qui portent chaque attribut qu'elle interroge.
    pub assiette_de_la_requete: Assiette,
    /// Les parts « sans … » de la requête.
    pub sans: Sans,
    /// Les étiquettes du fonds, chacune une fois, dans l'ordre de leurs octets (question 10) :
    /// celles des idées non retirées qui se lisent.
    pub etiquettes_du_fonds: Vec<String>,
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

/// L'état du fonds à l'ouverture, et après chaque commande : né ou non, la lecture seule, ses
/// avertissements, et le message du geste qu'annuler vise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EtatDuFonds {
    /// Le fonds est né.
    pub ne: bool,
    /// Pourquoi le fonds n'écrit pas ; `None` s'il écrit.
    pub lecture_seule: Option<Refus>,
    /// Les avertissements de la naissance ou de l'ouverture du fonds, et ceux que des réceptions
    /// refusées n'ont pu rendre, lisibles.
    pub avertissements: Vec<String>,
    /// Le message du dernier geste de tri, celui qu'annuler défait ; `None` sans geste à annuler.
    pub a_annuler: Option<String>,
}

/// La nature d'un fichier du dossier à verser, que l'inspection lit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatureDuFichier {
    /// Un son : son format (`wav`, `m4a`, …).
    Audio {
        /// Le format, en minuscules.
        format: String,
    },
    /// Un fichier MIDI.
    Midi,
    /// Un texte : son format.
    Texte {
        /// Le format, en minuscules.
        format: String,
    },
    /// Un fichier que l'import ne verse pas.
    Ignore,
}

/// Un fichier du dossier, tel que l'inspection le voit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FichierInspecte {
    /// Son rang dans la liste de l'inspection, que l'import reprend pour le verser : un entier
    /// sous `ENTIER_SUR`, le nombre de fichiers d'un dossier en restant très loin.
    pub rang: u64,
    /// Son chemin dans le dossier versé, sans `/` initial ; `None` si son nom ne se lit pas.
    pub chemin: Option<String>,
    /// Sa taille en octets, tenue sous `ENTIER_SUR` par sa source.
    pub octets: u64,
    /// Sa nature.
    pub nature: NatureDuFichier,
    /// Il a déjà été versé, par un import qu'on reprend ou par un autre.
    pub connu: bool,
}

/// La déclaration d'un dossier (balayage Q3) : jeu de stems (`stems`), son origine, l'un, l'autre
/// ou les deux. À l'import, `OrigineADeclarer::Aucune` veut dire « aucune origine déclarée » ;
/// après l'import, elle reste refusée, « Une origine déclarée ne se retire pas. ».
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    /// Le dossier est un jeu de stems.
    pub stems: bool,
    /// L'origine déclarée.
    pub origine: OrigineADeclarer,
}

/// Un dossier du dossier versé, tel que l'inspection le voit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DossierInspecte {
    /// Le chemin du dossier dans le dossier versé, sans `/` final, `""` pour le dossier versé
    /// lui-même (balayage Q13).
    pub chemin: String,
    /// Le nombre de ses fichiers audio, tenu sous `ENTIER_SUR` par sa source.
    pub audio: u64,
    /// L'inspection propose de le verser comme un jeu de stems.
    pub jeu_propose: bool,
    /// La déclaration qu'un import repris a déjà reçue pour lui, proposée de nouveau.
    pub reproposee: Option<Declaration>,
}

/// L'inspection d'un dossier à verser.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inspection {
    /// L'import que ce dossier reprend, un ULID.
    pub reprise: Option<String>,
    /// Le nom du dossier versé, s'il se lit.
    pub nom_du_dossier: Option<String>,
    /// Ses fichiers.
    pub fichiers: Vec<FichierInspecte>,
    /// Ses dossiers.
    pub dossiers: Vec<DossierInspecte>,
    /// Les octets à verser, tenus sous `ENTIER_SUR` par leur source.
    pub a_verser: u64,
    /// La place libre, en octets, tenue sous `ENTIER_SUR` par sa source.
    pub libre: u64,
    /// Pourquoi l'import ne peut pas se faire (place, droits) ; `None` s'il le peut.
    pub manque: Option<String>,
    /// Les noms des générateurs déclarés du fonds.
    pub generateurs: Vec<String>,
    /// Les déclarations de l'import repris, que l'écran montre sans choix (R2).
    pub declarations_reprises: Vec<DeclarationDUnDossier>,
    /// Un import est ouvert, que ce dossier ne reprend pas : l'écran le dit (« Un import est resté
    /// ouvert, que ce dossier ne reprend pas : verser celui-ci le clôt, avec ce qu'il a versé. »), et
    /// l'ouverture le clora (question 35 ; V12).
    pub ouvert_ailleurs: bool,
}

/// La déclaration d'un dossier. `chemin` : comme celui de `DossierInspecte`, sans `/` final, `""`
/// pour le dossier versé ; aucune normalisation aux services (balayage Q13).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclarationDUnDossier {
    /// Le chemin du dossier.
    pub chemin: String,
    /// Sa déclaration.
    pub declaration: Declaration,
}

/// Ouvrir l'import, ou le reprendre.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OuvrirLImport {
    /// Les déclarations des dossiers.
    pub declarations: Vec<DeclarationDUnDossier>,
    /// Le décalage local du geste, en minutes ; un `i32`, borné par son type.
    pub decalage_minutes: i32,
}

/// Les fichiers d'un jeu à verser, par dossier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JeuAVerser {
    /// Le chemin du dossier du jeu.
    pub dossier: String,
    /// Les rangs de ses fichiers, chacun sous `ENTIER_SUR`.
    pub rangs: Vec<u64>,
}

/// L'import ouvert.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportOuvert {
    /// Son identifiant, un ULID.
    pub import: String,
    /// Il reprend un import resté ouvert.
    pub repris: bool,
    /// Les rangs des fichiers à verser un à un, chacun sous `ENTIER_SUR`.
    pub rangs: Vec<u64>,
    /// Les jeux à verser en prises en stems.
    pub jeux: Vec<JeuAVerser>,
    /// Un avertissement lisible, s'il y en a un.
    pub avertissement: Option<String>,
}

/// Verser un fichier de l'import.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerserUnFichier {
    /// L'import, un ULID.
    pub import: String,
    /// Le rang du fichier.
    pub rang: u64,
    /// Le décalage local du geste, en minutes.
    pub decalage_minutes: i32,
}

/// Ce que le versement d'un fichier a donné.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Issue {
    /// Versé.
    Verse,
    /// Déjà versé : il ne s'ajoute pas une seconde fois.
    Dedoublonne,
    /// Rangé dans un jeu de stems.
    Stem,
    /// Déjà fait, par une reprise.
    DejaFait,
}

/// La réponse au versement d'un fichier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FichierVerse {
    /// Son rang, sous `ENTIER_SUR`.
    pub rang: u64,
    /// Ce qu'il a donné.
    pub issue: Issue,
    /// Un avertissement lisible, s'il y en a un.
    pub avertissement: Option<String>,
}

/// Verser une prise en stems.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerserUnePriseEnStems {
    /// L'import, un ULID.
    pub import: String,
    /// Le chemin du dossier du jeu.
    pub dossier: String,
    /// Le décalage local du geste, en minutes.
    pub decalage_minutes: i32,
}

/// Clore l'import.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloreLImport {
    /// L'import, un ULID.
    pub import: String,
    /// Le décalage local du geste, en minutes.
    pub decalage_minutes: i32,
}

/// Un nombre de fichiers et leurs octets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compte {
    /// Le nombre de fichiers, tenu sous `ENTIER_SUR` par sa source.
    pub fichiers: u64,
    /// Leurs octets, tenus sous `ENTIER_SUR` par leur source.
    pub octets: u64,
}

/// Le rapport d'un import.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RapportDImport {
    /// L'import, un ULID.
    pub import: String,
    /// Les fichiers vus.
    pub vus: Compte,
    /// Les fichiers versés.
    pub verses: Compte,
    /// Les fichiers dédoublonnés.
    pub dedoublonnes: Compte,
    /// Les fichiers illisibles.
    pub illisibles: Compte,
    /// Les fichiers ignorés.
    pub ignores: Compte,
    /// Les fichiers dont le nom ne se lit pas.
    pub sans_nom: Compte,
    /// Le nombre d'origines que l'import a changées, tenu sous `ENTIER_SUR` par sa source.
    pub origines_changees: u64,
    /// Les rangs des fichiers sans nom, chacun sous `ENTIER_SUR`.
    pub fichiers_sans_nom: Vec<u64>,
}

/// L'import clos.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportClos {
    /// Son rapport.
    pub rapport: RapportDImport,
    /// Un avertissement lisible, s'il y en a un.
    pub avertissement: Option<String>,
}

/// Ce qu'une origine déclarée vise. `Prefixe::prefixe` : une barre `/` finale s'ôte
/// (`origine::sous_le_prefixe` ; balayage Q13).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Portee {
    /// Une captation.
    Captation(Captation),
    /// Les fichiers d'un import sous un préfixe de chemin.
    Prefixe {
        /// L'import, un ULID.
        import: String,
        /// Le préfixe.
        prefixe: String,
    },
}

/// Une origine à déclarer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigineADeclarer {
    /// Aucune : à l'import, « aucune origine déclarée ».
    Aucune,
    /// À la main.
    Main,
    /// Par un générateur, nommé.
    Generee {
        /// Le nom du générateur.
        nom: String,
    },
}

/// Déclarer une origine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclarerUneOrigine {
    /// Ce que la déclaration vise.
    pub portee: Portee,
    /// L'origine.
    pub origine: OrigineADeclarer,
}

/// Le type d'un stem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeDeStem {
    /// Voix.
    Voix,
    /// Chœurs.
    Choeurs,
    /// Batterie.
    Batterie,
    /// Percussions.
    Percussions,
    /// Basse.
    Basse,
    /// Guitare.
    Guitare,
    /// Clavier.
    Clavier,
    /// Cordes.
    Cordes,
    /// Cuivres.
    Cuivres,
    /// Synthé.
    Synthe,
    /// Effets.
    Effets,
    /// Autre.
    Autre,
}

/// Garder le type proposé, ou en choisir un.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoixDuType {
    /// Garder le type proposé.
    Garder,
    /// Choisir ce type.
    Choisir(TypeDeStem),
}

/// Choisir le type d'un stem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoisirLeType {
    /// L'empreinte de la prise.
    pub prise: String,
    /// L'empreinte du stem.
    pub stem: String,
    /// Le choix.
    pub choix: ChoixDuType,
}

/// Un texte versé, lu ; `None` pour un texte illisible.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TexteLu {
    /// Le texte.
    pub texte: Option<String>,
}

/// Lire le rapport d'un import.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LireUnRapport {
    /// L'import, un ULID.
    pub import: String,
}

/// L'origine déclarée d'une captation, telle que la vue la montre.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrigineDeclaree {
    /// Aucune.
    Aucune,
    /// À la main.
    Main,
    /// Par un ou plusieurs générateurs.
    Generee {
        /// Les noms des générateurs.
        noms: Vec<String>,
    },
}

/// Les faits qu'une captation porte, lus à l'import.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaitsDUneCaptation {
    /// Son format.
    pub format: Option<String>,
    /// Sa durée en millisecondes, tenue sous `ENTIER_SUR` par sa source.
    pub duree_ms: Option<u64>,
    /// Son codage (un `u16`, borné par son type).
    pub codage: Option<u16>,
    /// Ses canaux (un `u16`, borné par son type).
    pub canaux: Option<u16>,
    /// Sa fréquence (un `u32`, borné par son type).
    pub frequence: Option<u32>,
    /// Ses bits par échantillon (un `u16`, borné par son type).
    pub bits: Option<u16>,
    /// Sa division MIDI (un `u16`, borné par son type).
    pub division: Option<u16>,
}

/// Le type d'un stem, et d'où il vient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeVu {
    /// Proposé par l'import.
    Propose(TypeDeStem),
    /// Gardé.
    Garde(TypeDeStem),
    /// Choisi.
    Choisi(TypeDeStem),
}

/// Le MIDI qui accompagne un stem.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiVu {
    /// Son empreinte, 64 caractères hexadécimaux minuscules.
    pub empreinte: String,
    /// Ses faits.
    pub faits: FaitsDUneCaptation,
    /// Ses noms vus : le premier vu d'abord, puis les autres dans l'ordre de leurs octets.
    pub noms: Vec<String>,
}

/// Un stem d'une prise.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemVu {
    /// Son empreinte, 64 caractères hexadécimaux minuscules.
    pub empreinte: String,
    /// Ses faits.
    pub faits: FaitsDUneCaptation,
    /// Ses noms vus : le premier vu d'abord, puis les autres dans l'ordre de leurs octets.
    pub noms: Vec<String>,
    /// Son type.
    pub type_de_stem: TypeVu,
    /// Son MIDI, s'il en a un.
    pub midi: Option<MidiVu>,
}

/// Ce que la vue d'une idée montre d'une captation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptationVue {
    /// La captation.
    pub captation: Captation,
    /// Ses faits.
    pub faits: FaitsDUneCaptation,
    /// Ses noms vus : le premier vu d'abord, puis les autres dans l'ordre de leurs octets.
    pub noms: Vec<String>,
    /// Son origine déclarée.
    pub origine: OrigineDeclaree,
    /// Ses stems, si c'est une prise en stems.
    pub stems: Vec<StemVu>,
}
