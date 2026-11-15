//! La boîte d'idées, sans entrée-sortie : l'ordre, le filtre et le compte (CAP-02, § 30 · 9).
//! Le tri est un geste (question 17) : l'ordre n'écrit rien et ne s'annule pas.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::gestes::{IdeeLue, Provenance, Sorte};
use super::idee::{Assiette, assiette};

/// L'ordre de la boîte : il n'écrit rien et ne s'annule pas ; le tri est un geste (question 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Un contenu que le filtre retient (CAP-02) : le texte de l'idée, une prise, une photo, un MIDI
/// versé (seul ou qui accompagne un stem), un texte versé (question 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

/// Une idée dans la boîte : l'idée lue, ses contenus, sa durée (`idee::duree`) et son état de
/// reliée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnBoite {
    /// L'idée lue.
    pub idee: IdeeLue,
    /// Ses contenus.
    pub contenus: BTreeSet<Contenu>,
    /// Sa durée, en millisecondes, si elle en a une.
    pub duree_ms: Option<u64>,
    /// Reliée, au sens d'`idee::reliee`.
    pub reliee: bool,
}

/// La boîte rangée : le compte des non reliées, l'assiette, et les idées que retient le filtre,
/// dans l'ordre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boite {
    /// Les idées non retirées et non reliées, que le filtre ne change pas.
    pub non_reliees: usize,
    /// L'assiette du compte.
    pub assiette: Assiette,
    /// Les idées que retient le filtre, dans l'ordre.
    pub retenues: Vec<EnBoite>,
}

/// Les contenus d'une idée : son texte (`TexteDeLIdee` s'il y en a un), les sortes de ses
/// captations (`Prise`, `MidiVerseSeul` → `MidiVerse`, `TexteVerse`), et une photo si
/// `une_photo`.
pub fn contenus_de(
    idee: &IdeeLue,
    sortes: impl IntoIterator<Item = Sorte>,
    une_photo: bool,
) -> BTreeSet<Contenu> {
    let mut contenus = BTreeSet::new();
    if idee.texte.is_some() {
        contenus.insert(Contenu::TexteDeLIdee);
    }
    for sorte in sortes {
        contenus.insert(match sorte {
            Sorte::Prise => Contenu::Prise,
            Sorte::MidiVerseSeul => Contenu::MidiVerse,
            Sorte::TexteVerse => Contenu::TexteVerse,
        });
    }
    if une_photo {
        contenus.insert(Contenu::Photo);
    }
    contenus
}

/// L'étiquette qui range une idée : la plus petite, octet pour octet.
fn plus_petite_etiquette(idee: &IdeeLue) -> Option<&str> {
    idee.etiquettes.iter().next().map(String::as_str)
}

/// Une valeur facultative, celles qui manquent en fin ; `croissant` dit le sens des autres.
fn avec_les_absents_en_fin<T: Ord>(a: Option<T>, b: Option<T>, croissant: bool) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) if croissant => x.cmp(&y),
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Une idée captée, rangée avant une importée.
fn rang_de_provenance(idee: &IdeeLue) -> u8 {
    match idee.provenance {
        Provenance::Captee { .. } => 0,
        Provenance::Importee { .. } => 1,
    }
}

/// Range `idees` selon `ordre`, en un ordre total (question 21) : la date, la plus récente
/// d'abord ; l'étiquette, sur la plus petite de l'idée, octet pour octet, les idées sans
/// étiquette en fin ; la durée, la plus longue d'abord, les idées sans durée en fin ; les non
/// reliées d'abord ; les captées d'abord ; puis, à égalité, la date décroissante, puis
/// l'identifiant, octet pour octet (ce n'est pas l'ordre de création de deux idées notées dans la
/// même seconde).
pub fn ordonner(idees: &mut [EnBoite], ordre: Ordre) {
    idees.sort_by(|a, b| {
        let selon_l_ordre = match ordre {
            Ordre::Date => Ordering::Equal,
            Ordre::Etiquette => avec_les_absents_en_fin(
                plus_petite_etiquette(&a.idee).map(str::as_bytes),
                plus_petite_etiquette(&b.idee).map(str::as_bytes),
                true,
            ),
            Ordre::Duree => avec_les_absents_en_fin(a.duree_ms, b.duree_ms, false),
            Ordre::Reliee => a.reliee.cmp(&b.reliee),
            Ordre::Provenance => rang_de_provenance(&a.idee).cmp(&rang_de_provenance(&b.idee)),
        };
        selon_l_ordre
            .then_with(|| b.idee.le.cmp(&a.idee.le))
            .then_with(|| a.idee.id.as_bytes().cmp(b.idee.id.as_bytes()))
    });
}

/// La boîte : les idées non retirées (question 14), le compte des non reliées, qui ne suit pas
/// le filtre, l'assiette (`idee::assiette` sur les non retirées, chacune portant la donnée, puis
/// `illisibles` ajoutées aux seules considérées : question 19), et celles que retient `filtre`
/// (aucun : toutes), rangées.
pub fn boite(
    idees: Vec<EnBoite>,
    illisibles: usize,
    ordre: Ordre,
    filtre: Option<Contenu>,
) -> Boite {
    let non_retirees: Vec<EnBoite> = idees
        .into_iter()
        .filter(|e| e.idee.retiree.is_none())
        .collect();
    let non_reliees = non_retirees.iter().filter(|e| !e.reliee).count();
    let mut l_assiette = assiette(non_retirees.iter().map(|e| &e.idee), |_| true);
    l_assiette.considerees += illisibles;
    let mut retenues: Vec<EnBoite> = non_retirees
        .into_iter()
        .filter(|e| filtre.is_none_or(|contenu| e.contenus.contains(&contenu)))
        .collect();
    ordonner(&mut retenues, ordre);
    Boite {
        non_reliees,
        assiette: l_assiette,
        retenues,
    }
}

/// Les seules idées retirées de `idees`, par la date du retrait, la plus récente d'abord, puis
/// par l'identifiant.
pub fn retirees(idees: Vec<EnBoite>) -> Vec<EnBoite> {
    let mut retirees: Vec<EnBoite> = idees
        .into_iter()
        .filter(|e| e.idee.retiree.is_some())
        .collect();
    retirees.sort_by(|a, b| {
        b.idee
            .retiree
            .cmp(&a.idee.retiree)
            .then_with(|| a.idee.id.as_bytes().cmp(b.idee.id.as_bytes()))
    });
    retirees
}
