//! Les contrats des adaptateurs (§ 40 · 3) : ce que les services demandent aux documents, aux
//! captations, à la source à verser et à l'aléa, que chaque côté réalise, sur fichiers ou en
//! mémoire.

use std::collections::BTreeSet;
use std::io;

use automerge::Automerge;
use noyau::donnees::changement::Operation;
use noyau::donnees::demarrage::Provisoire;
use noyau::donnees::identifiants::empreinte;

use crate::{Raison, Refus};

/// Ce que rend une écriture : faite, sans avertissement (`None`) ou avec un avertissement, en
/// français ; ou refusée.
pub type Ecriture = Result<Option<String>, Refus>;

/// Le message du refus d'écrire sans fonds : celui de `Documents::appliquer` quand aucun fonds
/// n'est né, sur fichiers comme en mémoire, et celui d'une opération sur une requête enregistrée,
/// que les services refusent sans fonds (question 20).
pub const FONDS_NON_NE: &str = "Aucun fonds n'est né.";

/// Les documents : le fonds, tel qu'il se lit, et ce qui s'y écrit, une opération nommée à la fois.
pub trait Documents {
    /// Le fonds tel qu'il se lit ; `None` tant qu'aucun fonds n'est né.
    fn fonds(&self) -> Option<&Automerge>;
    /// Pourquoi le fonds n'écrit pas ; `None` s'il écrit.
    fn lecture_seule(&self) -> Option<Refus>;
    /// Les avertissements de la naissance ou de l'ouverture du fonds, et ceux que des réceptions
    /// refusées n'ont pu rendre, en français ; aucun, en mémoire.
    fn avertissements(&self) -> Vec<String>;
    /// La genèse, `naissance`, puis `geste`, en un premier fichier complet ; refusé, rien ne reste.
    fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture;
    /// `operation`, en une modification atomique, avec son message et son heure ; sans fonds,
    /// refusée de `Raison::Impossible`, son message [`FONDS_NON_NE`].
    fn appliquer(&mut self, operation: &Operation) -> Ecriture;
}

/// La sorte d'une entrée du dossier versé, lue sans suivre un lien.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SorteDEntree {
    /// Un fichier.
    Fichier,
    /// Un lien, une jonction comprise : ni suivi, ni lu.
    Lien,
    /// Une autre entrée, ni fichier, ni dossier, ni lien : rien ne s'en lit.
    Autre,
}

/// Une entrée du dossier versé : son rang (dès 1), son chemin relatif, segments joints par `/`,
/// s'il est de l'UTF-8 ; l'extension de son dernier segment, si elle en est ; ses octets ; sa
/// sorte ; sa marque de dernière écriture, telle que le système de fichiers la tient, `None` s'il
/// ne la tient pas (sous Windows, `last_write_time` rend alors 0, que la source lit `None` ; V4)
/// (question 14 ; R8). La marque de dernière écriture n'est pas une heure : elle ne se compare
/// qu'à elle-même.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entree {
    /// Son rang, dès 1, dans l'ordre de ses segments.
    pub rang: u64,
    /// Son chemin relatif au dossier versé, ses segments joints par `/` ; `None` si l'un n'est pas
    /// de l'UTF-8.
    pub chemin: Option<String>,
    /// L'extension de son dernier segment (`signature::extension`), si elle est de l'UTF-8.
    pub extension: Option<String>,
    /// Ses octets au parcours ; un lien ou une autre entrée : 0, rien ne s'en lit.
    pub octets: u64,
    /// Sa sorte.
    pub sorte: SorteDEntree,
    /// Sa marque de dernière écriture au parcours ; `None` si le système de fichiers ne la tient
    /// pas.
    pub marque: Option<u64>,
}

/// La source à verser (§ 40 · 3, le troisième contrat) : un dossier parcouru une fois.
pub trait Source {
    /// Le dernier segment du dossier versé, s'il est de l'UTF-8 : le nom d'un jeu versé seul.
    fn nom_du_dossier(&self) -> Option<&str>;
    /// Ses entrées, dans l'ordre de leurs rangs, rangées par leurs segments, comparés un à un ;
    /// les dossiers n'y sont pas, leurs entrées si.
    fn entrees(&self) -> &[Entree];
    /// Au plus `longueur` octets de l'entrée `rang`, depuis `debut`.
    fn lire(&self, rang: u64, debut: u64, longueur: usize) -> Result<Vec<u8>, Refus>;
    /// L'entrée `rang`, en flux, de bout en bout.
    fn lire_en_flux(&self, rang: u64) -> Result<Box<dyn io::Read + '_>, Refus>;
    /// Les octets de l'entrée `rang` à cet instant.
    fn octets_actuels(&self, rang: u64) -> Result<u64, Refus>;
    /// La marque de dernière écriture de l'entrée `rang` à cet instant.
    fn marque_actuelle(&self, rang: u64) -> Result<Option<u64>, Refus>;
    /// Les octets lus de l'entrée `rang` depuis le parcours, par `lire` et `lire_en_flux`
    /// (décision 19).
    fn octets_lus(&self, rang: u64) -> u64;
}

/// L'indice, dans `entrees`, de l'entrée `rang` d'une source à verser, sur fichiers comme en
/// mémoire : `rang - 1`. Un rang sans entrée, 0 compris, est refusé (`Raison::Impossible`).
pub fn l_entree(entrees: &[Entree], rang: u64) -> Result<usize, Refus> {
    rang.checked_sub(1)
        .and_then(|indice| usize::try_from(indice).ok())
        .filter(|indice| *indice < entrees.len())
        .ok_or_else(|| Refus {
            raison: Raison::Impossible,
            message: format!("Le dossier versé n'a pas d'entrée n° {rang}."),
        })
}

/// L'indice, dans `entrees`, du fichier `rang` d'une source à verser, qui seul se lit : un rang
/// sans entrée ([`l_entree`]), ou une entrée qui n'est pas un fichier (un lien ne s'ouvre jamais),
/// est refusé (`Raison::Impossible`).
pub fn le_fichier(entrees: &[Entree], rang: u64) -> Result<usize, Refus> {
    let indice = l_entree(entrees, rang)?;
    if entrees[indice].sorte != SorteDEntree::Fichier {
        return Err(Refus {
            raison: Raison::Impossible,
            message: format!(
                "L'entrée n° {rang} du dossier versé n'est pas un fichier : elle ne se lit pas."
            ),
        });
    }
    Ok(indice)
}

/// Au plus `longueur` octets de `contenu`, depuis `debut`, bornés à lui : une plage qui commence
/// à la fin, ou après, n'en rend aucun.
pub(crate) fn plage(contenu: &[u8], debut: u64, longueur: u64) -> &[u8] {
    let de = usize::try_from(debut)
        .unwrap_or(usize::MAX)
        .min(contenu.len());
    let longueur = usize::try_from(longueur).unwrap_or(usize::MAX);
    let a = de.saturating_add(longueur).min(contenu.len());
    &contenu[de..a]
}

/// Ce que rend une copie en flux : l'empreinte calculée, les octets copiés, si une captation la
/// portait déjà, et si la copie est rangée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versement {
    /// L'empreinte du contenu copié, calculée en chemin.
    pub empreinte: String,
    /// Les octets copiés.
    pub octets: u64,
    /// Une captation portait déjà cette empreinte : la copie ne la récrit pas, son nom provisoire
    /// se retire.
    pub deja_rangee: bool,
    /// La copie s'accorde à l'empreinte attendue, ou n'en attendait aucune : sa captation est
    /// rangée, par elle ou déjà. Faux en désaccord : rien n'est rangé.
    pub rangee: bool,
}

/// Les captations, rangées par leur empreinte.
pub trait Captations {
    /// Copie `flux` sous un nom provisoire formé sur `base` (`voisin`), l'empreinte calculée en
    /// chemin, le synchronise, le relit en flux contre elle, puis le range ; une captation déjà
    /// rangée sous cette empreinte : le nom provisoire se retire, `deja_rangee` vrai (question 8).
    /// `attendue` donnée et autre que l'empreinte calculée : le nom provisoire se retire avant le
    /// renommage, rien n'est rangé, `rangee` faux (un fichier changé depuis l'inspection ; R20).
    fn verser_en_flux(
        &self,
        base: &str,
        flux: &mut dyn io::Read,
        attendue: Option<&str>,
    ) -> Result<Versement, Refus>;
    /// La voie par octets : `verser_en_flux`, `base` l'empreinte des octets, calculée d'abord,
    /// `attendue` à `None` ; rend l'empreinte. Un adaptateur qui garde un raccourci pour une
    /// captation déjà rangée la remplace.
    fn verser(&self, octets: &[u8]) -> Result<String, Refus> {
        let base = empreinte(octets);
        let mut flux = octets;
        self.verser_en_flux(&base, &mut flux, None)
            .map(|versement| versement.empreinte)
    }
    /// Les octets de la captation qui porte `empreinte`.
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus>;
    /// Au plus `longueur` octets de la captation `empreinte`, depuis `debut` ; le fichier refermé.
    fn relire_une_plage(
        &self,
        empreinte: &str,
        debut: u64,
        longueur: u64,
    ) -> Result<Vec<u8>, Refus>;
    /// Les octets de la captation `empreinte`, `None` si aucune ne la porte.
    fn taille(&self, empreinte: &str) -> Result<Option<u64>, Refus>;
    /// Les tailles des captations rangées, chacune une fois.
    fn tailles(&self) -> Result<BTreeSet<u64>, Refus>;
    /// Les noms provisoires de `captations/` que `import::copie_de` dit de l'import `import`,
    /// chacun lu en flux (son empreinte, et si elle est rangée).
    fn copies_de_l_import(&self, import: &str) -> Result<Vec<Provisoire>, Refus>;
    /// Retire le nom provisoire `nom` de `captations/`.
    fn retirer_une_copie(&self, nom: &str) -> Result<(), Refus>;
}

/// L'aléa, que le côté natif tire et que les tests écrivent.
pub trait Alea {
    /// Remplit `octets` d'aléa.
    fn remplir(&mut self, octets: &mut [u8]);
}
