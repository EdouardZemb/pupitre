//! Le fonds lu et les gestes (§ 31 · 3) : ce que le noyau sait d'une idée et d'une captation,
//! telles que `services::lecture` les tire du document, et les opérations nommées qu'il calcule
//! de ce qu'il reçoit, avec l'inverse de chaque geste de tri, calculé sur l'idée lue avant lui.
//! L'heure et le décalage local sont des arguments, jamais l'horloge.

use std::collections::BTreeSet;
use std::fmt;

use crate::donnees::changement::{Changement, Operation, Simple};
use crate::donnees::fonds::{
    CAPTEE, DEPUIS, DUREE, ETIQUETTES, FORMAT, IDEE, IDEES, IDENTIFIANT, IMPORTEE, LE, LIENS,
    LIGNEE, MESSAGE_DE_LA_NAISSANCE, MIDI_VERSES, PAR, PRISES, PROPRIETAIRE, PROVENANCE, RETIREE,
    TEXTE, TEXTES_VERSES,
};
use crate::donnees::idee::{cle_du_lien, est_vide};

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

/// Le plus grand entier que `JSON.parse` relit à l'identique, 2⁵³ − 1 : celui de
/// `commandes::ENTIER_SUR`, que le noyau ne voit pas.
pub const ENTIER_SUR: u64 = 9_007_199_254_740_991;

/// Un geste de tri : son opération nommée, et l'inverse de ses changements, calculé sur l'idée
/// lue avant le geste, que son annulation écrit (§ 30 · 11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GesteDeTri {
    /// L'opération du geste.
    pub operation: Operation,
    /// Les changements qui reposent les valeurs d'avant le geste.
    pub inverse: Vec<Changement>,
}

/// Pourquoi un geste ne se fait pas : un constat, jamais un ordre (§ 50 · 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Impossible {
    /// Le texte d'une note, ou celui qui complète une idée, est vide, une fois ses blancs ôtés.
    TexteVide,
    /// Aucune idée ne porte l'identifiant `idee`.
    IdeeAbsente { idee: String },
    /// L'idée `idee` est retirée : aucun geste de tri ne la touche, sauf la rétablir.
    IdeeRetiree { idee: String },
    /// L'idée `idee`, que l'on rétablirait, n'est pas retirée.
    IdeeNonRetiree { idee: String },
    /// Le texte qui complète l'idée est déjà le sien : le geste serait sans effet.
    MemeTexte,
    /// L'étiquette est vide.
    EtiquetteVide,
    /// L'étiquette `etiquette` est déjà posée sur l'idée.
    EtiquetteDejaPosee { etiquette: String },
    /// L'étiquette `etiquette` n'est pas posée sur l'idée.
    EtiquetteAbsente { etiquette: String },
    /// Une idée ne se relie pas à elle-même (§ 30 · 9.3).
    RelieeAElleMeme,
    /// Les deux idées sont déjà reliées.
    LienDejaPose,
    /// Les deux idées ne sont pas reliées.
    LienAbsent,
    /// Aucune captation ne porte l'empreinte `empreinte`.
    CaptationAbsente { empreinte: String },
    /// La captation est déjà dans l'idée où on la regrouperait.
    DejaDansLIdee,
    /// La fiche de la captation `empreinte` est déjà versée.
    CaptationDejaVersee { empreinte: String },
    /// Une durée sur une captation qui n'est pas une prise : seule une prise a une durée
    /// (§ 30 · 9.2).
    DureeHorsDUnePrise,
    /// Une durée de `duree_ms` millisecondes, au-delà d'`ENTIER_SUR` : l'interface ne la
    /// relirait pas à l'identique, une impossibilité stricte (ADR-006).
    DureeTropLongue { duree_ms: u64 },
    /// Un MIDI versé ou un texte versé capté : il n'entre que par l'import (§ 30 · 8.1).
    HorsDeLImport,
}

impl fmt::Display for Impossible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Impossible::TexteVide => f.write_str("Le texte est vide."),
            Impossible::IdeeAbsente { idee } => {
                write!(f, "Aucune idée ne porte l'identifiant {idee}.")
            }
            Impossible::IdeeRetiree { idee } => write!(f, "L'idée {idee} est retirée."),
            Impossible::IdeeNonRetiree { idee } => write!(f, "L'idée {idee} n'est pas retirée."),
            Impossible::MemeTexte => f.write_str("Le texte est déjà celui-ci."),
            Impossible::EtiquetteVide => f.write_str("L'étiquette est vide."),
            Impossible::EtiquetteDejaPosee { etiquette } => {
                write!(f, "L'étiquette #{etiquette} est déjà posée.")
            }
            Impossible::EtiquetteAbsente { etiquette } => {
                write!(f, "L'étiquette #{etiquette} n'est pas posée.")
            }
            Impossible::RelieeAElleMeme => f.write_str("Une idée ne se relie pas à elle-même."),
            Impossible::LienDejaPose => f.write_str("Ces deux idées sont déjà reliées."),
            Impossible::LienAbsent => f.write_str("Ces deux idées ne sont pas reliées."),
            Impossible::CaptationAbsente { empreinte } => {
                write!(f, "Aucune captation ne porte l'empreinte {empreinte}.")
            }
            Impossible::DejaDansLIdee => f.write_str("Cette captation est déjà dans cette idée."),
            Impossible::CaptationDejaVersee { empreinte } => {
                write!(f, "La captation {empreinte} est déjà versée.")
            }
            Impossible::DureeHorsDUnePrise => f.write_str("Seule une prise a une durée."),
            Impossible::DureeTropLongue { duree_ms } => write!(
                f,
                "Une durée de {duree_ms} ms passe le plus grand entier que l'interface relit à \
                 l'identique."
            ),
            Impossible::HorsDeLImport => {
                f.write_str("Un MIDI versé ou un texte versé n'entre que par l'import.")
            }
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

fn oter(segments: &[&str]) -> Changement {
    Changement::Oter {
        chemin: chemin(segments),
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

/// Le jour que dit le message d'un geste de tri pour l'idée : celui de sa naissance, au décalage
/// du geste, jamais celui du geste (§ 30 · 11).
fn jour(idee: &IdeeLue, decalage_minutes: i32) -> String {
    date_du_message(idee.le, decalage_minutes)
}

/// Un geste de tri ne touche pas une idée retirée, sauf pour la rétablir :
/// `Impossible::IdeeRetiree`.
fn visible(idee: &IdeeLue) -> Result<(), Impossible> {
    match idee.retiree {
        Some(_) => Err(Impossible::IdeeRetiree {
            idee: idee.id.clone(),
        }),
        None => Ok(()),
    }
}

fn geste(
    changements: Vec<Changement>,
    inverse: Vec<Changement>,
    message: String,
    heure: i64,
) -> GesteDeTri {
    GesteDeTri {
        operation: Operation {
            changements,
            message,
            heure,
        },
        inverse,
    }
}

/// L'idée lue sous l'identifiant `id` ; aucune : `Impossible::IdeeAbsente`.
pub fn presente(id: &str, lue: Option<IdeeLue>) -> Result<IdeeLue, Impossible> {
    lue.ok_or_else(|| Impossible::IdeeAbsente {
        idee: id.to_string(),
    })
}

/// Compléter : le texte de l'idée devient `texte`, rangé tel qu'il est reçu ; l'inverse repose le
/// texte d'avant, ou l'ôte s'il n'y en avait pas. Un texte vide une fois ses blancs ôtés :
/// `Impossible::TexteVide` ; le texte qu'elle porte déjà : `Impossible::MemeTexte`.
pub fn completer(
    idee: &IdeeLue,
    texte: &str,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(idee)?;
    if texte.trim().is_empty() {
        return Err(Impossible::TexteVide);
    }
    if idee.texte.as_deref() == Some(texte) {
        return Err(Impossible::MemeTexte);
    }
    let cle = [IDEES, idee.id.as_str(), TEXTE];
    let inverse = match &idee.texte {
        Some(avant) => poser(&cle, Simple::Chaine(avant.clone())),
        None => oter(&cle),
    };
    Ok(geste(
        vec![poser(&cle, Simple::Chaine(texte.to_string()))],
        vec![inverse],
        format!("Idée du {} : texte complété", jour(idee, decalage_minutes)),
        heure,
    ))
}

/// Poser l'étiquette `etiquette` sur l'idée, rangée sans `#`, que le message ajoute, et comparée
/// octet pour octet, sans normalisation ; l'inverse l'ôte. Vide : `Impossible::EtiquetteVide` ;
/// déjà posée : `Impossible::EtiquetteDejaPosee`.
pub fn poser_une_etiquette(
    idee: &IdeeLue,
    etiquette: &str,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(idee)?;
    if etiquette.is_empty() {
        return Err(Impossible::EtiquetteVide);
    }
    if idee.etiquettes.contains(etiquette) {
        return Err(Impossible::EtiquetteDejaPosee {
            etiquette: etiquette.to_string(),
        });
    }
    let cle = [IDEES, idee.id.as_str(), ETIQUETTES, etiquette];
    Ok(geste(
        vec![poser(&cle, Simple::Vrai)],
        vec![oter(&cle)],
        format!(
            "Idée du {} : étiquette #{etiquette} posée",
            jour(idee, decalage_minutes)
        ),
        heure,
    ))
}

/// Ôter l'étiquette `etiquette` de l'idée, comparée octet pour octet ; l'inverse la repose. Vide :
/// `Impossible::EtiquetteVide` ; pas posée : `Impossible::EtiquetteAbsente`.
pub fn oter_une_etiquette(
    idee: &IdeeLue,
    etiquette: &str,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(idee)?;
    if etiquette.is_empty() {
        return Err(Impossible::EtiquetteVide);
    }
    if !idee.etiquettes.contains(etiquette) {
        return Err(Impossible::EtiquetteAbsente {
            etiquette: etiquette.to_string(),
        });
    }
    let cle = [IDEES, idee.id.as_str(), ETIQUETTES, etiquette];
    Ok(geste(
        vec![oter(&cle)],
        vec![poser(&cle, Simple::Vrai)],
        format!(
            "Idée du {} : étiquette #{etiquette} ôtée",
            jour(idee, decalage_minutes)
        ),
        heure,
    ))
}

/// Relier les idées `a` et `b` d'un lien « apparenté », sous sa clé (§ 30 · 9.3) ; l'inverse
/// l'ôte. Une idée à elle-même : `Impossible::RelieeAElleMeme` ; déjà reliées :
/// `Impossible::LienDejaPose`.
pub fn relier(
    a: &IdeeLue,
    b: &IdeeLue,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(a)?;
    visible(b)?;
    if a.id == b.id {
        return Err(Impossible::RelieeAElleMeme);
    }
    let cle = cle_du_lien(&a.id, &b.id);
    if a.liens.contains(&cle) {
        return Err(Impossible::LienDejaPose);
    }
    let lien = [LIENS, cle.as_str()];
    Ok(geste(
        vec![poser(&lien, Simple::Vrai)],
        vec![oter(&lien)],
        format!(
            "Idée du {} reliée à l'idée du {}",
            jour(a, decalage_minutes),
            jour(b, decalage_minutes)
        ),
        heure,
    ))
}

/// Délier les idées `a` et `b` : leur lien « apparenté » ôté ; l'inverse le repose. Pas
/// reliées : `Impossible::LienAbsent`.
pub fn delier(
    a: &IdeeLue,
    b: &IdeeLue,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(a)?;
    visible(b)?;
    let cle = cle_du_lien(&a.id, &b.id);
    if !a.liens.contains(&cle) {
        return Err(Impossible::LienAbsent);
    }
    let lien = [LIENS, cle.as_str()];
    Ok(geste(
        vec![oter(&lien)],
        vec![poser(&lien, Simple::Vrai)],
        format!(
            "Idée du {} déliée de l'idée du {}",
            jour(a, decalage_minutes),
            jour(b, decalage_minutes)
        ),
        heure,
    ))
}

/// Ce que dit le message d'un regroupement d'une captation de sorte `sorte`.
fn regroupee(sorte: Sorte) -> &'static str {
    match sorte {
        Sorte::Prise => "prise regroupée",
        Sorte::MidiVerseSeul => "MIDI versé regroupé",
        Sorte::TexteVerse => "texte versé regroupé",
    }
}

/// Regrouper la captation dans l'idée `cible` : elle quitte `source`, son idée, qui porte
/// `captations_de_la_source` captations, elle comprise ; l'inverse la rend à la source. La source
/// que le geste laisse sans rien est retirée par le même geste, à `heure` (§ 30 · 9.3), et
/// l'inverse ôte ce retrait. La source pour cible : `Impossible::DejaDansLIdee`.
pub fn regrouper(
    captation: &CaptationLue,
    source: &IdeeLue,
    captations_de_la_source: usize,
    cible: &IdeeLue,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(source)?;
    visible(cible)?;
    if source.id == cible.id {
        return Err(Impossible::DejaDansLIdee);
    }
    let cle = format!("{}/{IDEE}", captation.empreinte);
    let fait = [captation.sorte.conteneur(), cle.as_str()];
    let mut changements = vec![poser(&fait, Simple::Chaine(cible.id.clone()))];
    let mut inverse = vec![poser(&fait, Simple::Chaine(source.id.clone()))];
    let regroupement = format!(
        "Idée du {} : {}",
        jour(cible, decalage_minutes),
        regroupee(captation.sorte)
    );
    let message = if est_vide(source, captations_de_la_source.saturating_sub(1)) {
        let retrait = [IDEES, source.id.as_str(), RETIREE];
        changements.push(poser(&retrait, Simple::Entier(heure)));
        inverse.push(oter(&retrait));
        format!(
            "{regroupement}, idée du {} retirée",
            jour(source, decalage_minutes)
        )
    } else {
        regroupement
    };
    Ok(geste(changements, inverse, message, heure))
}

/// Retirer l'idée : la date de son retrait, `heure` (§ 30 · 9.1) ; l'inverse l'ôte.
pub fn retirer(
    idee: &IdeeLue,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    visible(idee)?;
    let retrait = [IDEES, idee.id.as_str(), RETIREE];
    Ok(geste(
        vec![poser(&retrait, Simple::Entier(heure))],
        vec![oter(&retrait)],
        format!("Idée du {} retirée", jour(idee, decalage_minutes)),
        heure,
    ))
}

/// Rétablir l'idée retirée : la marque de son retrait ôtée (§ 30 · 9.1) ; l'inverse repose la
/// date d'avant. Pas retirée : `Impossible::IdeeNonRetiree`.
pub fn retablir(
    idee: &IdeeLue,
    heure: i64,
    decalage_minutes: i32,
) -> Result<GesteDeTri, Impossible> {
    let Some(avant) = idee.retiree else {
        return Err(Impossible::IdeeNonRetiree {
            idee: idee.id.clone(),
        });
    };
    let retrait = [IDEES, idee.id.as_str(), RETIREE];
    Ok(geste(
        vec![oter(&retrait)],
        vec![poser(&retrait, Simple::Entier(avant))],
        format!("Idée du {} rétablie", jour(idee, decalage_minutes)),
        heure,
    ))
}

/// Ce que dit le message du versement d'une captation de sorte `sorte`.
fn versee(sorte: Sorte) -> &'static str {
    match sorte {
        Sorte::Prise => "prise versée",
        Sorte::MidiVerseSeul => "MIDI versé",
        Sorte::TexteVerse => "texte versé",
    }
}

/// Verser la fiche d'une captation : l'idée `idee` naît avec elle, de provenance `provenance`.
///
/// L'idée est créée avec son objet des étiquettes, sa provenance, son appareil `par`, sa
/// naissance `le` à `heure` et, importée, son import `depuis` ; la captation porte son idée et
/// son format, rangé tel que la fiche le porte, et, d'une prise seulement, sa durée, si la fiche
/// la porte (§ 30 · 9.2 ; ADR-024). La date et l'appareil d'une prise ne s'écrivent pas : des
/// faits de sa captation que rien ne lit encore sur elle, ils valent `—`. Les refus : une fiche
/// déjà versée (`existante`), `Impossible::CaptationDejaVersee` ; un MIDI versé ou un texte versé
/// capté, `Impossible::HorsDeLImport` ; une durée hors d'une prise,
/// `Impossible::DureeHorsDUnePrise` ; au-delà d'`ENTIER_SUR`, `Impossible::DureeTropLongue`.
// Huit arguments : chaque donnée du versement, et la captation que le fonds porte déjà sous
// l'empreinte, que les services lisent.
#[allow(clippy::too_many_arguments)]
pub fn verser(
    sorte: Sorte,
    empreinte: &str,
    fiche: &Fiche,
    idee: &str,
    provenance: &Provenance,
    existante: Option<&CaptationLue>,
    heure: i64,
    decalage_minutes: i32,
) -> Result<Operation, Impossible> {
    if existante.is_some() {
        return Err(Impossible::CaptationDejaVersee {
            empreinte: empreinte.to_string(),
        });
    }
    let (origine, par, depuis) = match provenance {
        Provenance::Captee { par } => (CAPTEE, par, None),
        Provenance::Importee { par, depuis } => (IMPORTEE, par, Some(depuis)),
    };
    if sorte != Sorte::Prise && depuis.is_none() {
        return Err(Impossible::HorsDeLImport);
    }
    if let Some(duree_ms) = fiche.duree_ms {
        if sorte != Sorte::Prise {
            return Err(Impossible::DureeHorsDUnePrise);
        }
        if duree_ms > ENTIER_SUR {
            return Err(Impossible::DureeTropLongue { duree_ms });
        }
    }
    let mut changements = vec![
        Changement::Creer {
            chemin: chemin(&[IDEES, idee]),
            conteneurs: vec![ETIQUETTES.to_string()],
        },
        poser(
            &[IDEES, idee, PROVENANCE],
            Simple::Chaine(origine.to_string()),
        ),
        poser(&[IDEES, idee, PAR], Simple::Chaine(par.nom().to_string())),
        poser(&[IDEES, idee, LE], Simple::Entier(heure)),
    ];
    if let Some(depuis) = depuis {
        changements.push(poser(
            &[IDEES, idee, DEPUIS],
            Simple::Chaine(depuis.clone()),
        ));
    }
    let conteneur = sorte.conteneur();
    changements.push(poser(
        &[conteneur, &format!("{empreinte}/{IDEE}")],
        Simple::Chaine(idee.to_string()),
    ));
    changements.push(poser(
        &[conteneur, &format!("{empreinte}/{FORMAT}")],
        Simple::Chaine(fiche.format.clone()),
    ));
    if let Some(duree_ms) = fiche.duree_ms {
        changements.push(poser(
            &[conteneur, &format!("{empreinte}/{DUREE}")],
            Simple::Naturel(duree_ms),
        ));
    }
    Ok(Operation {
        changements,
        message: format!(
            "Idée du {} : {}",
            date_du_message(heure, decalage_minutes),
            versee(sorte)
        ),
        heure,
    })
}

/// L'annulation : l'inverse du geste, sous « Annulé : » suivi de son message, à `heure`
/// (§ 30 · 11).
pub fn annulation(geste: &GesteDeTri, heure: i64) -> Operation {
    Operation {
        changements: geste.inverse.clone(),
        message: format!("Annulé : {}", geste.operation.message),
        heure,
    }
}
