//! Le fonds lu et les gestes (§ 31 · 3) : ce que le noyau sait d'une idée et d'une captation,
//! telles que `services::lecture` les tire du document, et les opérations nommées qu'il calcule
//! de ce qu'il reçoit. L'heure et le décalage local sont des arguments, jamais l'horloge.

use std::collections::BTreeSet;
use std::fmt;

use crate::donnees::changement::{Changement, Operation, Simple};
use crate::donnees::fonds::{
    CAPTEE, ETIQUETTES, IDEES, IDENTIFIANT, LE, LIGNEE, MESSAGE_DE_LA_NAISSANCE, MIDI_VERSES, PAR,
    PRISES, PROPRIETAIRE, PROVENANCE, TEXTE, TEXTES_VERSES,
};

/// Le rôle de l'appareil où un geste se fait (§ 30 · 8.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appareil {
    /// L'établi.
    Etabli,
    /// Le carnet.
    Carnet,
}

impl Appareil {
    /// Son nom, tel que le fonds le range sous `par` : `etabli`, `carnet`.
    pub fn nom(self) -> &'static str {
        match self {
            Appareil::Etabli => "etabli",
            Appareil::Carnet => "carnet",
        }
    }

    /// L'appareil dont `nom` est le nom, l'inverse de `nom` ; un nom inconnu, `None`. Un appareil
    /// ajouté s'ajoute ici : le test de l'aller-retour (`chaque_appareil_se_retrouve_de_son_nom`,
    /// dans `tests/gestes.rs`) le nomme par un `match` exhaustif.
    pub fn depuis_nom(nom: &str) -> Option<Appareil> {
        match nom {
            "etabli" => Some(Appareil::Etabli),
            "carnet" => Some(Appareil::Carnet),
            _ => None,
        }
    }
}

/// La provenance d'une idée (§ 30 · 8.1), posée à sa naissance et gardée à vie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Née d'une capture, sur l'appareil `par`.
    Captee { par: Appareil },
    /// Née d'un fichier versé, sur l'appareil `par`, par l'import `depuis`, un ULID.
    Importee { par: Appareil, depuis: String },
}

/// La sorte d'une captation versée (§ 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sorte {
    /// Une prise.
    Prise,
    /// Un MIDI versé seul, qui n'accompagne aucun stem.
    MidiVerseSeul,
    /// Un texte versé.
    TexteVerse,
}

impl Sorte {
    /// Les sortes, chacune une fois, dans l'ordre de leur déclaration. Une sorte ajoutée s'ajoute
    /// ici : le test qui la garde (`toutes_les_sortes_y_sont_une_fois`, dans `tests/gestes.rs`) la
    /// nomme par un `match` exhaustif.
    pub const TOUTES: [Sorte; 3] = [Sorte::Prise, Sorte::MidiVerseSeul, Sorte::TexteVerse];

    /// Le conteneur de la racine du document qui la range : `prises`, `midi_verses` ou
    /// `textes_verses`.
    pub fn conteneur(self) -> &'static str {
        match self {
            Sorte::Prise => PRISES,
            Sorte::MidiVerseSeul => MIDI_VERSES,
            Sorte::TexteVerse => TEXTES_VERSES,
        }
    }
}

/// Une idée, telle que `services::lecture` la tire du document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdeeLue {
    /// Son identifiant, un ULID.
    pub id: String,
    /// Sa provenance.
    pub provenance: Provenance,
    /// La date de sa naissance, en secondes.
    pub le: i64,
    /// Son texte, s'il y en a un.
    pub texte: Option<String>,
    /// Ses étiquettes.
    pub etiquettes: BTreeSet<String>,
    /// La date de son retrait, en secondes, si elle est retirée.
    pub retiree: Option<i64>,
    /// Les clés des liens dont elle est un bout.
    pub liens: BTreeSet<String>,
}

/// Une captation versée, telle que `services::lecture` la tire du document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptationLue {
    /// Sa sorte.
    pub sorte: Sorte,
    /// Son empreinte.
    pub empreinte: String,
    /// L'identifiant de son idée.
    pub idee: String,
    /// Sa durée, en millisecondes, d'une prise seulement, quand elle est lue.
    pub duree_ms: Option<u64>,
}

/// Ce qui se verse d'une captation, ses faits lus : `format`, au vocabulaire du § 30 · 9.2 et de
/// l'ADR-024, jamais l'extension ; `duree_ms`, d'une prise seulement.
/// La date et l'appareil d'une prise n'y sont pas : rien ne les lit sur la captation à INC-1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fiche {
    /// Son format.
    pub format: String,
    /// Sa durée, en millisecondes.
    pub duree_ms: Option<u64>,
}

/// Pourquoi un geste ne se fait pas : un constat, jamais un ordre (§ 50 · 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Impossible {
    /// Le texte d'une note est vide, une fois ses blancs ôtés.
    TexteVide,
}

impl fmt::Display for Impossible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Impossible::TexteVide => f.write_str("Le texte de la note est vide."),
        }
    }
}

/// Les mois, de mars à février : l'année du calcul civil commence le 1er mars, et le jour de
/// février qui finit une année bissextile en est le dernier.
const MOIS_DEPUIS_MARS: [&str; 12] = [
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
    "janvier",
    "février",
];

/// « 3 novembre », « 1er novembre » : le jour que dit le message d'un geste (§ 30 · 11), celui de
/// `heure`, en secondes, au décalage local `decalage_minutes` que l'interface donne, sans année.
pub fn date_du_message(heure: i64, decalage_minutes: i32) -> String {
    let jours = (heure + i64::from(decalage_minutes) * 60).div_euclid(86_400);
    // Le calcul civil inversé (« civil from days », H. Hinnant), sur des entiers, dans une ère
    // de 400 ans, 146 097 jours, qui commence un 1er mars (celle de 1600 à 2000, par exemple).
    // `doe`, le jour de l'ère, de 0 à 146 096 : 719 468 jours vont du 1er mars de l'an 0 au
    // 1er janvier 1970.
    let doe = (jours + 719_468).rem_euclid(146_097);
    // `yoe`, l'année de l'ère, de 0 à 399. Les termes de siècle ne la changent qu'aux abords du
    // 1er mars : `doe / 36_524`, aux premiers jours de mars après le premier siècle de l'ère
    // (dès le 1er mars 1700 dans l'ère de 1600, dès le 1er mars 2100 dans celle de 2000) ;
    // `doe / 146_096`, au seul 29 février qui finit l'ère (celui de 2000 dans l'ère de 1600).
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    // `doy`, le jour de l'année, de 0 (le 1er mars) à 365.
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    // `mp`, le mois, de 0 (mars) à 11 (février).
    let mp = (5 * doy + 2) / 153;
    let jour = doy - (153 * mp + 2) / 5 + 1;
    let mois = MOIS_DEPUIS_MARS[mp as usize];
    if jour == 1 {
        format!("1er {mois}")
    } else {
        format!("{jour} {mois}")
    }
}

fn chemin(segments: &[&str]) -> Vec<String> {
    segments.iter().map(|segment| segment.to_string()).collect()
}

fn poser(segments: &[&str], valeur: Simple) -> Changement {
    Changement::Poser {
        chemin: chemin(segments),
        valeur,
    }
}

/// La naissance : l'identifiant, la lignée et l'identifiant de propriétaire posés à la racine du
/// document, sous `MESSAGE_DE_LA_NAISSANCE`.
pub fn naissance(identifiant: &str, lignee: &str, proprietaire: &str, heure: i64) -> Operation {
    Operation {
        changements: vec![
            poser(&[IDENTIFIANT], Simple::Chaine(identifiant.to_string())),
            poser(&[LIGNEE], Simple::Chaine(lignee.to_string())),
            poser(&[PROPRIETAIRE], Simple::Chaine(proprietaire.to_string())),
        ],
        message: MESSAGE_DE_LA_NAISSANCE.to_string(),
        heure,
    }
}

/// Noter : l'idée `idee` créée avec son objet des étiquettes, son texte et sa provenance `captee`,
/// par `appareil`, née à `heure` ; message « Idée du <jour> notée ». Un texte vide une fois ses
/// blancs ôtés : `Impossible::TexteVide`. Le texte se range tel qu'il est reçu.
pub fn noter(
    idee: &str,
    texte: &str,
    appareil: Appareil,
    heure: i64,
    decalage_minutes: i32,
) -> Result<Operation, Impossible> {
    if texte.trim().is_empty() {
        return Err(Impossible::TexteVide);
    }
    Ok(Operation {
        changements: vec![
            Changement::Creer {
                chemin: chemin(&[IDEES, idee]),
                conteneurs: vec![ETIQUETTES.to_string()],
            },
            poser(&[IDEES, idee, TEXTE], Simple::Chaine(texte.to_string())),
            poser(
                &[IDEES, idee, PROVENANCE],
                Simple::Chaine(CAPTEE.to_string()),
            ),
            poser(
                &[IDEES, idee, PAR],
                Simple::Chaine(appareil.nom().to_string()),
            ),
            poser(&[IDEES, idee, LE], Simple::Entier(heure)),
        ],
        message: format!("Idée du {} notée", date_du_message(heure, decalage_minutes)),
        heure,
    })
}
